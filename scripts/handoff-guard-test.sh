#!/usr/bin/env bash
# Offline test of scripts/handoff-guard.sh: feeds it Stop hook input over
# scratch transcripts, shaped like Claude Code's (origin kinds, task
# notifications, slash commands, synthetic responses), and a stub br, and
# checks when it blocks the stop. Spends no tokens. Needs jq and tac.
# Usage: scripts/handoff-guard-test.sh
set -uo pipefail
HOOK="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)/handoff-guard.sh"
TMP="$(mktemp -d)" || exit 1
trap 'rm -r "$TMP"' EXIT
fail=0

# The stub br prints $TMP/ready for `br ready --json` and exits with the
# status in $TMP/status (default 0), or fails when $TMP/ready is missing.
mkdir "$TMP/bin"
cat >"$TMP/bin/br" <<EOF
#!/usr/bin/env bash
[[ "\$*" == "ready --json" && -f "$TMP/ready" ]] || exit 1
cat "$TMP/ready"; exit "\$(cat "$TMP/status" 2>/dev/null || echo 0)"
EOF
chmod +x "$TMP/bin/br"
TASK='[{"id":"fx-epc","issue_type":"epic","labels":[]},
  {"id":"fx-own","issue_type":"decision","labels":["owner"]},
  {"id":"fx-tsk","issue_type":"task","labels":["design"]}]'
NONE='[{"id":"fx-epc","issue_type":"epic","labels":[]},
  {"id":"fx-own","issue_type":"decision","labels":["owner"]}]'

# transcript PROMPT TOKENS TEXT [USAGE]: write a transcript with the owner's
# PROMPT (none if empty), a tool round, a task notification and a meta entry
# that say "stop", a synthetic response, and a final response of TEXT at
# TOKENS of context. USAGE, if given, replaces the final response's usage.
transcript() {
  [[ -n "$1" ]] && jq -nc --arg p "$1" '{type: "user", origin: {kind: "human"}, message: {content: $p}}'
  jq -nc '{type: "assistant", message: {content: [{type: "tool_use", id: "t1", name: "Bash", input: {}}],
    usage: {input_tokens: 5, cache_creation_input_tokens: 5, cache_read_input_tokens: 1000}}}'
  jq -nc '{type: "user", message: {content: [{type: "tool_result", tool_use_id: "t1", content: "ok"}]}}'
  jq -nc '{type: "user", origin: {kind: "task-notification"}, message: {content: "<task-notification>review: stop</task-notification>"}}'
  jq -nc '{type: "user", isMeta: true, message: {content: "<system-reminder>stop</system-reminder>"}}'
  jq -nc --arg t "$3" --argjson n "$2" --argjson u "${4:-null}" '{type: "assistant",
    message: {content: [{type: "text", text: $t}],
      usage: ($u // {input_tokens: 2, cache_creation_input_tokens: 1000, cache_read_input_tokens: ($n - 1002)})}}'
  jq -nc '{type: "assistant", message: {model: "<synthetic>", content: [{type: "text", text: "No response requested."}],
    usage: {input_tokens: 0, cache_creation_input_tokens: 0, cache_read_input_tokens: 0}}}'
}

# check NAME WANT [EXTRA]: run the hook on $TMP/t.jsonl, merging the JSON
# object EXTRA into its input, and compare whether it blocked with WANT
# (block or allow). ENTRYPOINT overrides CLAUDE_CODE_ENTRYPOINT.
check() {
  local input out got=allow
  input="$(jq -nc --arg t "$TMP/t.jsonl" --argjson x "${3:-{\}}" \
    '{hook_event_name: "Stop", transcript_path: $t, stop_hook_active: false} + $x')"
  out="$(CLAUDE_CODE_ENTRYPOINT="${ENTRYPOINT:-cli}" PATH="$TMP/bin:$PATH" "$HOOK" <<<"$input")"
  [[ "$(jq -r '.decision // empty' <<<"$out" 2>/dev/null)" == block ]] && got=block
  if [[ "$got" == "$2" ]]; then echo "ok   $1"; else echo "FAIL $1: want $2, got $got: $out"; fail=1; fi
}

OFFER='fx-a is closed. Want me to continue with the next bead, or hand this off to a fresh session?'
echo "$TASK" >"$TMP/ready"
transcript "work fx-a" 250000 "$OFFER" >"$TMP/t.jsonl"
check "handoff offer at 250k blocks" block
out="$(CLAUDE_CODE_ENTRYPOINT=cli PATH="$TMP/bin:$PATH" "$HOOK" <<<"{\"transcript_path\":\"$TMP/t.jsonl\"}")"
jq -e '.reason | test("about 250k") and test("fx-tsk") and (test("fx-epc|fx-own") | not)' \
  <<<"$out" >/dev/null && echo "ok   reason names the count and only pickable beads" \
  || { echo "FAIL reason: $out"; fail=1; }
check "a second stop goes through" allow '{"stop_hook_active": true}'
check "a running background task allows the stop" allow '{"background_tasks": [{"id": "b1"}]}'
check "a session cron allows the stop" allow '{"session_crons": [{"id": "c1"}]}'
ENTRYPOINT=sdk-cli check "headless sessions are left alone" allow
check "last_assistant_message wins over the transcript" allow '{"last_assistant_message": "fx-a is closed."}'

# Final messages: each offers a handoff (block) or does not (allow).
while IFS='|' read -r want message; do
  check "message: $message" "$want" "$(jq -nc --arg m "$message" '{last_assistant_message: $m}')"
done <<'EOF'
block|fx-a is closed. I'll hand off now.
block|Would you like me to continue?
block|Shall I continue?
block|Should I hand off now?
block|Good place to stop: the next session can pick up fx-b.
block|Next up is `prime, then continue fx-b from its handoff`.
allow|Review complete. The tests cover resuming a bead in a new session.
allow|Round 2 runs in a fresh agent, as converge asks.
allow|Continued fx-a from its handoff; it is closed.
allow|fx-a is closed and committed as 1234abc.
EOF

# Owner prompts: each asks to stop (allow) or does not (block).
while IFS='|' read -r want prompt; do
  transcript "$prompt" 250000 "$OFFER" >"$TMP/t.jsonl"
  check "prompt: $prompt" "$want"
done <<'EOF'
block|prime, then continue fx-a from its handoff
block|keep going, don't stop until 600k
block|do not stop
allow|ok, wrap up and hand it off
allow|please stop
allow|Finish fx-a. Stop when done.
allow|<command-message>handoff</command-message>\n<command-name>/handoff</command-name>
allow|prime
allow|<command-message>prime</command-message>\n<command-name>/prime</command-name>
EOF

transcript "" 250000 "$OFFER" >"$TMP/t.jsonl"
check "no owner prompt allows the stop" allow
transcript "work fx-a" 450000 "$OFFER" >"$TMP/t.jsonl"
check "handoff offer at 450k goes through" allow
transcript "work fx-a" 250000 "$OFFER" '{}' >"$TMP/t.jsonl"
check "a response without usage allows the stop" allow
transcript "work fx-a" 250000 "$OFFER" >"$TMP/t.jsonl"
echo "$NONE" >"$TMP/ready"
check "only epics and owner beads ready" allow
echo "$TASK" >"$TMP/ready"; echo 1 >"$TMP/status"
check "br exiting non-zero allows the stop" allow
rm "$TMP/ready" "$TMP/status"
check "br failing allows the stop" allow
echo "$TASK" >"$TMP/ready"
rm "$TMP/t.jsonl"
check "a missing transcript allows the stop" allow

exit "$fail"
