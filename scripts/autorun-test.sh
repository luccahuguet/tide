#!/usr/bin/env bash
# Offline test of scripts/autorun.sh: in scratch repositories with their own
# beads, a stub agent closes, hands off, blocks, fails, stalls, pushes,
# leaves a process behind, or does nothing as each scenario tells it, and the
# test checks which beads the runner picks and with which prompts, the beads,
# outcomes, and commit ranges it logs, and why it stops. Spends no tokens. Needs br, git, jq, flock, timeout,
# setsid.
# Usage: scripts/autorun-test.sh
set -uo pipefail
RUNNER="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)/autorun.sh"
TMP="$(mktemp -d)"
trap 'rm -r "$TMP"' EXIT
fail=0

# The stub pops its next action for $AUTORUN_BEAD from actions/<bead>.
cat >"$TMP/stub" <<'EOF'
#!/usr/bin/env bash
f="$STUB_DIR/actions/$AUTORUN_BEAD"
action="$(head -n1 "$f" 2>/dev/null)"; sed -i 1d "$f" 2>/dev/null
echo "$AUTORUN_BEAD" >>"$STUB_DIR/picked"
echo "$AUTORUN_SESSION_CAP" >>"$STUB_DIR/caps"
echo "$1" >>"$STUB_DIR/prompts"
handoff() {
  br update "$AUTORUN_BEAD" --status in_progress >/dev/null
  br comments add "$AUTORUN_BEAD" "Handoff (context mark). Next: go on." >/dev/null
}
case "$action" in
  close) br close "$AUTORUN_BEAD" --reason done >/dev/null ;;
  cost-close) echo "warning: noise first"; echo '{"total_cost_usd":0.6}'
    br close "$AUTORUN_BEAD" --reason done >/dev/null ;;
  handoff) handoff ;;
  commit-handoff) echo x >>work; git add work; git commit -q -m step; handoff ;;
  beads-handoff) handoff; git commit -q -m handoff -- .beads ;;
  block) br update "$AUTORUN_BEAD" --status blocked >/dev/null ;;
  linger) setsid sleep 47 </dev/null >/dev/null 2>&1 &
    br close "$AUTORUN_BEAD" --reason done >/dev/null ;;
  fail) exit 1 ;;
  stall) sleep 30 ;;
  push) git commit -q --allow-empty -m pushed; git push -q 2>/dev/null ;;
  *) : ;;
esac
EOF
chmod +x "$TMP/stub"

check_extra() { :; }
# scenario <name> <stop text> <picks> <outcomes> -- <setup...>
# Setup runs in a fresh repository with beads b (P3 task, created first),
# a (P2 task), e (P0 epic), o (P0 task labelled owner), and d (P0 task, deferred).
# <outcomes> is the log's outcome column joined by commas, or "-" to skip
# both it and the check that the log's bead column matches <picks>.
# check_extra, if redefined just before a scenario, must also pass; it is
# reset afterwards.
scenario() {
  local name="$1" want_stop="$2" want_picks="$3" want_out="$4" dir out picks got logged
  shift 5
  dir="$TMP/$name"
  mkdir -p "$dir/actions" && cd "$dir" || return
  git init -q && git config user.email t@t && git config user.name t
  br init --prefix t >/dev/null 2>&1
  b=$(br create b -t task -p 3 --silent); a=$(br create a -t task -p 2 --silent)
  br create e -t epic -p 0 --silent >/dev/null
  br create o -t task -p 0 --labels owner --silent >/dev/null
  br defer "$(br create d -t task -p 0 --silent)" >/dev/null
  # As in Tide, .beads/ is tracked, so every br write dirties the tree.
  printf '.autorun/\nactions/\npicked\ncaps\nprompts\nfake.pid\n' >.git/info/exclude
  git add .beads && git commit -q -m base
  "$@"
  out="$(STUB_DIR="$dir" AUTORUN_AGENT="$TMP/stub" AUTORUN_TIMEOUT="${TO:-30s}" \
    AUTORUN_MAX_SESSIONS="${MAX:-10}" AUTORUN_BUDGET_USD="${BUDGET:-100}" \
    AUTORUN_SESSION_USD="${SESSION:-1}" PATH="${XPATH:-}$PATH" "$RUNNER" 2>&1)"
  picks="$(sed "s/^$a\$/a/; s/^$b\$/b/" "$dir/picked" 2>/dev/null | paste -sd' ')"
  got="$(cut -f3 .autorun/log 2>/dev/null | paste -sd,)"
  logged="$(cut -f2 .autorun/log 2>/dev/null | sed "s/^$a\$/a/; s/^$b\$/b/" | paste -sd' ')"
  if [[ "$out" == *"$want_stop"* && "$picks" == "$want_picks" &&
        ("$want_out" == - || ("$got" == "$want_out" && "$logged" == "$want_picks")) ]] &&
    check_extra; then
    echo "PASS  $name"
  else
    echo "FAIL  $name: picks '$picks' (want '$want_picks'), logged '$logged', outcomes '$got' (want '$want_out')"
    sed 's/^/      /' <<<"$out"
    fail=1
  fi
  check_extra() { :; }
  cd "$TMP" || return
}
acts() { printf '%s\n' "${@:2}" >"actions/${!1}"; }

scenario no-work "no work remains" "a b" "closed,closed" -- \
  eval 'acts a close; acts b close'
check_extra() { [[ "$(sed -n 1p prompts)" == "prime, then continue $b from its handoff. "* &&
                  "$(sed -n 2p prompts)" == "prime, then work $a. "* ]]; }
scenario handoff-first " (continue" "b a" "closed,closed" -- \
  eval 'br update $b --status in_progress >/dev/null; br comments add $b "Handoff (x)." >/dev/null; acts b close; acts a close'
scenario live-worker "in progress without a handoff" "" - -- \
  eval 'br update $a --status in_progress >/dev/null'
scenario stale-handoff "in progress without a handoff" "" - -- \
  eval 'br update $a --status in_progress >/dev/null; br comments add $a "Handoff (old)." >/dev/null; br comments add $a "Resumed." >/dev/null'
check_extra() { [[ "$(cut -f4 .autorun/log | paste -sd,)" == none,???????..???????,none ]]; }
scenario three-handoffs "handed off 3 times in a row with no new commit" "a a a" \
  "handed off,handed off,handed off" -- eval 'acts a handoff beads-handoff handoff'
check_extra() { [[ "$(cut -f4 .autorun/log | paste -sd,)" == none,none,???????..???????,none,none ]]; }
scenario commit-resets "no work remains" "a a a a b" \
  "handed off,handed off,handed off,closed,closed" -- \
  eval 'acts a handoff handoff commit-handoff close; acts b close'
TO=2s scenario two-failures "2 sessions in a row failed or timed out" "a a" "failed,timed out" -- \
  eval 'acts a fail stall'
scenario failures-reset "no work remains" "a a b b" "failed,closed,failed,closed" -- \
  eval 'acts a fail close; acts b fail close'
scenario noop-fails "2 sessions in a row failed" "a a" "failed,failed" -- \
  eval 'acts a noop noop'
scenario continue-noop "2 sessions in a row failed" "a a" "failed,failed" -- \
  eval 'br update $a --status in_progress >/dev/null; br comments add $a "Handoff (x)." >/dev/null; acts a noop noop'
scenario blocked "failed or timed out; blocked beads for the owner: t-" "a b b" \
  "blocked,failed,failed" -- eval 'acts a block; acts b fail fail'
MAX=2 scenario session-cap "session cap 2 reached" "a a" - -- \
  eval 'acts a handoff handoff handoff'
check_extra() { [[ "$(paste -sd' ' caps)" == "1 0.4" ]]; }
BUDGET=1 scenario budget-reported "budget cap" "a b" "closed,closed" -- \
  eval 'acts a cost-close; acts b cost-close'
check_extra() { [[ "$(paste -sd' ' caps)" == "1 0.5" ]]; }
BUDGET=1.5 scenario budget-unreported "budget cap" "a b" "closed,closed" -- \
  eval 'acts a close; acts b close'
scenario dirty-tree "uncommitted changes" "" - -- \
  eval 'echo x >stray'
mkdir -p "$TMP/badbr" && cat >"$TMP/badbr/br" <<EOF
#!/usr/bin/env bash
[[ "\$1" == ready ]] && { echo '{"error":{"code":"DATABASE_ERROR"}}'; exit 0; }
exec $(command -v br) "\$@"
EOF
chmod +x "$TMP/badbr/br"
XPATH="$TMP/badbr:" scenario br-error "cannot read br state" "" - -- true
scenario push "upstream ref moved" "a" - -- \
  eval 'git init -q --bare ../push-remote.git; git remote add origin ../push-remote.git; git push -q -u origin HEAD 2>/dev/null; acts a push'

# A session the owner started by hand in the checkout (here a stand-in named
# claude) keeps the runner from continuing the bead it resumed; an idle codex
# app-server does not.
mkdir -p "$TMP/fake" && cp "$(readlink -f "$(command -v sleep)")" "$TMP/fake/claude" &&
  cp /bin/bash "$TMP/fake/codex" && printf 'sleep 60; true\n' >"$TMP/fake/app-server"
check_extra() { kill "$(cat fake.pid)"; }
scenario manual-continue "is working in this checkout" "" - -- \
  eval 'br update ${b} --status in_progress >/dev/null; br comments add ${b} "Handoff (x)." >/dev/null; "$TMP/fake/claude" 60 & echo $! >fake.pid'
check_extra() { kill "$(cat fake.pid)"; }
scenario app-server-ignored "no work remains" "a b" "closed,closed" -- \
  eval 'PATH="$TMP/fake:$PATH" "$TMP/fake/codex" app-server & echo $! >fake.pid; acts a close; acts b close'

# SIGTERM (as Ctrl-C would) stops a stalled session and the run.
mkdir -p "$TMP/interrupt/actions" && cd "$TMP/interrupt" && git init -q &&
  git config user.email t@t && git config user.name t && git commit -q --allow-empty -m base &&
  br init --prefix t >/dev/null 2>&1 && printf '.beads/\n.autorun/\nactions/\npicked\ncaps\nprompts\n' >.git/info/exclude &&
  echo stall >"actions/$(br create a -t task -p 1 --silent)"
STUB_DIR="$TMP/interrupt" AUTORUN_AGENT="$TMP/stub" AUTORUN_TIMEOUT=20s "$RUNNER" >"$TMP/interrupt.out" 2>&1 &
runner=$!
for _ in $(seq 50); do [[ -s picked ]] && break; sleep 0.1; done
kill -TERM $runner
wait $runner
if grep -q "stopping: interrupted" "$TMP/interrupt.out" && ! pgrep -f "$TMP/stub" >/dev/null; then
  echo "PASS  interrupt"
else echo "FAIL  interrupt"; sed "s/^/      /" "$TMP/interrupt.out"; fail=1; fi

# A process a session leaves running keeps the next runner out (fail closed).
check_extra() { "$RUNNER" >/dev/null 2>&1; local rc=$?; pkill -f "^sleep 47$"; [[ $rc -eq 3 ]]; }
scenario lock-held-by-leftover "no work remains" "a b" - -- eval 'acts a linger; acts b close'

# A second runner exits while the first holds the lock.
mkdir -p "$TMP/lock" && cd "$TMP/lock" && git init -q
mkdir -p .autorun && exec 8>.autorun/lock && flock -n 8
"$RUNNER" >/dev/null 2>&1
if [[ $? -eq 3 ]]; then echo "PASS  lock"; else echo "FAIL  lock: second runner did not exit 3"; fail=1; fi
exec 8>&-

exit $fail
