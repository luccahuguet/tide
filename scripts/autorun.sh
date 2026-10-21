#!/usr/bin/env bash
# Run fresh headless agent sessions back to back from br state, so a handoff
# does not wait for the owner to start the next session (bead fx-3md).
#
# Each iteration picks one bead: an in_progress bead whose latest comment is
# a handoff ("Handoff (...)", the handoff skill's template), else the top
# `br ready` bead (open, unblocked, not deferred) that is not an epic and not
# labelled `owner`. It starts a session with "prime, then continue <id> from
# its handoff" or "prime, then work <id>", waits under a timeout, judges the
# outcome from br state and git history (not from the session's report), and
# appends one line to the log: time, bead, outcome (closed, handed off,
# blocked, failed, timed out), and the new commit range. A session that ends
# without closing, blocking, or handing off its bead has failed.
#
# It stops, saying why, when no work remains, the same bead is handed off 3
# times in a row with no new commit outside .beads/, 2 sessions in a row fail
# or time out, the session or budget cap is reached, br state cannot be read,
# or a session pushed. It never starts a session while another may be
# working: the lock keeps out other runners, and it refuses to start while a
# claude or codex process works in this checkout, while any in_progress bead
# lacks a handoff as its latest comment (so a failed session that leaves its
# bead in progress also stops the run), or, before new work, while the
# working tree has uncommitted changes outside .beads/. It cannot see
# sessions in other worktrees whose claims live only there, or Codex sessions
# hosted by an app-server (the Codex app or an IDE), so do not run it beside
# one. The owner reviews the blocked beads it lists when it stops; it never
# retries a blocked bead.
#
# It also runs the correct skill, which files beads for corrections the owner
# keeps repeating (bead fx-mr4), in a session of its own: after every 10th
# session of a run, and when it stops because no work remains, unless that
# stop directly follows such a run. It skips that run while a claude or codex
# process works in this checkout. It runs under the same permissions, timeout,
# and budget as a bead session but does not count toward the session cap or
# the failure and handoff rules. Its log line holds the time, "/correct", the
# beads it filed (read from br before and after), "none", or "bead ids
# unavailable" when br cannot be read afterwards, which also stops the run,
# plus its exit status when non-zero, and its cost. The beads it files stay
# uncommitted in .beads/, which the dirty-tree check ignores, so the next
# session or the owner commits them. The runner only lists what was filed; it
# leaves duplicates to the skill, which checks existing beads before it files
# and may miss a class whose bead was closed, so a class can be filed again.
#
# Permissions: sessions run `claude -p --permission-mode auto`, the owner's
# default mode, in which a classifier gates each action and the owner's hooks
# still apply. It lets a session edit, commit, and run br, cargo, and codex;
# bypassPermissions would be broader than the work needs. Remote writes are
# denied with --disallowedTools (git push, gh), a pattern match that other
# spellings can slip past, so the runner also stops if the upstream ref moves.
#
# Usage: scripts/autorun.sh        (from anywhere inside the repository)
# Environment:
#   AUTORUN_MAX_SESSIONS   sessions per run (default 10)
#   AUTORUN_BUDGET_USD     total spend cap (default 100). Each session may
#                          spend at most the lesser of AUTORUN_SESSION_USD and
#                          what remains; a session that reports no cost is
#                          charged its whole allowance. claude checks its cap
#                          between turns, so a session can overrun by a turn.
#   AUTORUN_SESSION_USD    per-session cap (default 20)
#   AUTORUN_TIMEOUT        per-session timeout for timeout(1) (default 3h)
#   AUTORUN_AGENT          path of an agent executable that replaces claude,
#                          for tests. It runs as `"$AUTORUN_AGENT" <prompt>`
#                          with AUTORUN_BEAD and AUTORUN_SESSION_CAP set
#                          (for a correct run, the prompt is "/correct" and
#                          AUTORUN_BEAD is empty); a JSON object it prints
#                          with total_cost_usd is its cost.
# Logs, session output, and the lock live in .autorun/ (git-ignored). Ctrl-C
# stops the current session and the run. Needs br, git, jq, flock, timeout.
set -uo pipefail

ROOT="$(git rev-parse --show-toplevel)" || exit 2
cd "$ROOT" || exit 2
STATE="$ROOT/.autorun"
mkdir -p "$STATE"
LOG="$STATE/log"
MAX_SESSIONS="${AUTORUN_MAX_SESSIONS:-10}"
BUDGET="${AUTORUN_BUDGET_USD:-100}"
SESSION_USD="${AUTORUN_SESSION_USD:-20}"
TIMEOUT="${AUTORUN_TIMEOUT:-3h}"

exec 9>"$STATE/lock"
# Sessions inherit the lock, so a process one leaves running keeps the next
# runner out until it ends: fail closed rather than risk two sessions at once.
flock -n 9 || { echo "autorun: $STATE/lock is held by another runner or a process a session left running" >&2; exit 3; }

say() { echo "autorun: $*"; }
# stop <reason>: say why, list the blocked beads for the owner, and exit.
stop() {
  local blocked
  blocked="$(br list --status blocked --json 2>/dev/null | jq -r '[.issues[].id] | join(" ")' 2>/dev/null)"
  say "stopping: $*${blocked:+; blocked beads for the owner: $blocked}"
  exit 0
}
calc() { awk "BEGIN { print $1 }"; }

# brq <jq filter> <br args...>: run br with --json, apply the filter, and set
# BR to the result, or fail on any br failure, error envelope, or filter
# failure. brj does the same but stops the run on failure. Helpers that can
# stop set globals, never print into $(...), since an exit there would only
# leave the subshell.
brq() {
  local filter="$1" out
  shift
  out="$(br "$@" --json 2>/dev/null)" &&
    jq -e 'type != "object" or (has("error") | not)' <<<"$out" >/dev/null &&
    BR="$(jq -r "$filter" <<<"$out")"
}
brj() { brq "$@" || { shift; stop "cannot read br state (br $*)"; }; }
is_handoff() { jq -e '. != null and (.text | test("^Handoff"))' <<<"$1" >/dev/null; }
latest_comment() { brj 'max_by(.id) // null | tojson' comments list "$1"; COMMENT="$BR"; }
upstream() { git rev-parse -q --verify '@{u}' 2>/dev/null || echo none; }

# pick: set id and mode (continue or work), leave id empty when no work
# remains, or stop when another session may be working.
# live_agent: succeed, setting LIVE, when a claude or codex process other than
# the runner's own ancestors works in this checkout. That covers a session the
# owner started by hand, which takes no lock and may leave an old handoff as
# the bead's latest comment. Idle `codex app-server` daemons do not count.
declare -A ancestors=()
p=$$
while ((p > 1)); do ancestors[$p]=1; p="$(awk '/^PPid:/ { print $2 }' "/proc/$p/status")"; done
live_agent() {
  local d p comm cwd
  for d in /proc/[0-9]*; do
    p="${d#/proc/}"
    comm="$(cat "$d/comm" 2>/dev/null)" || continue
    [[ "$comm" =~ ^\.?(claude|codex)(-wrapped)?$ && -z "${ancestors[$p]:-}" ]] || continue
    [[ "$(tr '\0' '\n' <"$d/cmdline" 2>/dev/null | sed -n 2p)" == app-server ]] && continue
    cwd="$(readlink "$d/cwd" 2>/dev/null)" || continue
    [[ "$cwd" == "$ROOT" || "$cwd" == "$ROOT"/* ]] && { LIVE="$p ($comm)"; return 0; }
  done
  return 1
}

pick() {
  local ids b
  id="" mode=""
  live_agent && stop "process $LIVE is working in this checkout"
  brj '.issues[].id' list --status in_progress
  ids="$BR"
  for b in $ids; do
    latest_comment "$b"
    is_handoff "$COMMENT" || stop "$b is in progress without a handoff: another session may hold it, or one ended without handing it off"
  done
  for b in $ids; do id="$b" mode=continue; return; done
  brj 'map(select(.issue_type != "epic" and ((.labels // []) | index("owner") | not))) | .[0].id // empty' \
    ready --sort priority
  [[ -z "$BR" ]] && return
  [[ -n "$(git status --porcelain -- . ':!.beads')" ]] && stop "the working tree has uncommitted changes"
  id="$BR" mode=work
}

# allowance: set cap to the lesser of the per-session cap and what remains of
# the budget, or fail when nothing remains.
allowance() {
  local remaining
  remaining="$(calc "$BUDGET - $spent")"
  awk -v r="$remaining" 'BEGIN { exit !(r > 0) }' || return 1
  cap="$(awk -v r="$remaining" -v s="$SESSION_USD" 'BEGIN { print (r < s ? r : s) }')"
}
# charge: set cost from the session's output in $out and add it to spent; a
# session that reports no cost is charged its whole allowance.
charge() {
  cost="$(jq -rR 'fromjson? | select(type == "object") | .total_cost_usd | numbers' "$out" | tail -1)"
  spent="$(calc "$spent + ${cost:-$cap}")"
}

# correct: run the correct skill in its own session if the budget allows and
# no other agent works here (the next pick then stops the run), and log the
# beads it filed and its cost. The prompt carries no arguments, since the
# skill would read a slash command's arguments as an owner message.
correct() {
  local before filed rc up br_ok=1 id=""
  allowance || return
  live_agent && return
  brj '[.issues[].id]' list --status all
  before="$BR"
  up="$(upstream)"
  say "correct (cap \$$cap)"
  out="$STATE/correct-$(date -u +%Y%m%dT%H%M%SZ).out"
  run_agent /correct "$cap"
  rc=$?
  charge
  # The run is paid for, so log it even when br cannot be read, then stop.
  if brq '[.issues[].id]' list --status all; then
    filed="$(jq -rn --argjson a "$BR" --argjson b "$before" '$a - $b | join(" ")')"
    filed="${filed:-none}"
  else
    filed="bead ids unavailable" br_ok=""
  fi
  ((rc)) && filed+=" (exit $rc)"
  printf '%s\t/correct\t%s\t%s\n' "$(date -u +%FT%TZ)" "$filed" "${cost:+\$}${cost:-unreported}" >>"$LOG"
  say "correct: filed $filed (spent \$$spent)"
  [[ "$(upstream)" == "$up" ]] || stop "the upstream ref moved during the correct run"
  [[ -n "$br_ok" ]] || stop "cannot read br state (br list --status all)"
}

# run_agent <prompt> <cap>: stdout to $out, stderr to $out.err. The session
# inherits the lock (fd 9) on purpose; see the flock line.
run_agent() {
  local pid rc
  if [[ -n "${AUTORUN_AGENT:-}" ]]; then
    AUTORUN_BEAD="$id" AUTORUN_SESSION_CAP="$2" timeout --kill-after=60 "$TIMEOUT" "$AUTORUN_AGENT" "$1" \
      </dev/null >"$out" 2>"$out.err" &
  else
    timeout --kill-after=60 "$TIMEOUT" claude -p --permission-mode auto \
      --max-budget-usd "$2" --output-format json \
      --disallowedTools 'Bash(git push:*)' 'Bash(gh:*)' -- "$1" \
      </dev/null >"$out" 2>"$out.err" &
  fi
  pid=$!
  # timeout(1) runs the session in its own process group, which a terminal
  # Ctrl-C does not reach, so forward it.
  trap 'kill -TERM -- -$pid 2>/dev/null; wait $pid; stop interrupted' INT TERM
  wait $pid
  rc=$?
  trap - INT TERM
  return $rc
}

sessions=0 failures=0 spent=0 last_handoff="" handoffs=0
while :; do
  ((sessions < MAX_SESSIONS)) || stop "session cap $MAX_SESSIONS reached"
  allowance || stop "budget cap \$$BUDGET reached (spent \$$spent)"
  pick
  if [[ -z "$id" ]]; then
    ((sessions == 0 || sessions % 10)) && correct
    stop "no work remains"
  fi

  if [[ "$mode" == continue ]]; then
    prompt="prime, then continue $id from its handoff."
  else
    prompt="prime, then work $id."
  fi
  prompt+=" This run is unattended: follow AGENTS.md's Autonomous runs rules. Do not push."
  before_head="$(git rev-parse HEAD)"
  before_up="$(upstream)"
  latest_comment "$id"
  before_comment="$(jq -r '.id // 0' <<<"$COMMENT")"
  sessions=$((sessions + 1))
  say "session $sessions: $id ($mode, cap \$$cap)"
  out="$STATE/session-$(date -u +%Y%m%dT%H%M%SZ)-$sessions-$id.out"
  run_agent "$prompt" "$cap"
  rc=$?

  charge
  after_head="$(git rev-parse HEAD)"
  range="none"
  [[ "$after_head" != "$before_head" ]] && range="${before_head:0:7}..${after_head:0:7}"
  brj '.[0].status' show "$id"
  status="$BR"
  latest_comment "$id"
  c="$COMMENT"

  if [[ "$status" == closed ]]; then outcome="closed"
  elif [[ "$status" == blocked ]]; then outcome="blocked"
  elif jq -e --argjson b "$before_comment" '.id > $b' <<<"$c" >/dev/null 2>&1 && is_handoff "$c"; then
    outcome="handed off"
  elif ((rc == 124 || rc == 137)); then outcome="timed out"
  else outcome="failed"
  fi
  printf '%s\t%s\t%s\t%s\n' "$(date -u +%FT%TZ)" "$id" "$outcome" "$range" >>"$LOG"
  say "$id: $outcome ($range, spent \$$spent)"

  [[ "$(upstream)" == "$before_up" ]] || stop "the upstream ref moved during the session on $id"
  case "$outcome" in
    "timed out" | failed) failures=$((failures + 1)) ;;
    *) failures=0 ;;
  esac
  ((failures >= 2)) && stop "2 sessions in a row failed or timed out"
  # A commit that touches only .beads/ (say, the handoff itself) is not progress.
  if [[ "$outcome" == "handed off" ]] && git diff --quiet "$before_head" "$after_head" -- . ':!.beads'; then
    [[ "$id" == "$last_handoff" ]] && handoffs=$((handoffs + 1)) || handoffs=1
    last_handoff="$id"
    ((handoffs >= 3)) && stop "$id handed off 3 times in a row with no new commit"
  else
    last_handoff="" handoffs=0
  fi
  ((sessions % 10)) || correct
done
