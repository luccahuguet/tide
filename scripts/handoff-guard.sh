#!/usr/bin/env bash
# Claude Code Stop hook: block an attended session's early handoff once
# (bead fx-h6o). Attended sessions kept stopping after a bead or two, or
# asking "continue or hand off?", at 10-30% of their context, though
# AP-SESSION-001 stops new scope only at 400k.
#
# It blocks the stop, with a reason Claude reads before going on, only when
# all of these hold:
# - the session is interactive (CLAUDE_CODE_ENTRYPOINT=cli), so headless
#   runs such as scripts/autorun.sh keep their one-bead sessions;
# - the hook has not already blocked this stop (stop_hook_active), so the
#   owner is never trapped: the next stop goes through;
# - no background task or session cron is running, since a turn that waits
#   on a review may end;
# - context in use is under 350k tokens: the last real response's fresh
#   input, cache writes, and cache reads, as the transcript records them;
# - the final message proposes a handoff or a fresh session, or asks whether
#   to go on;
# - the owner's last prompt (origin "human") exists, did not ask to stop or
#   hand off, and was not a bare prime;
# - `br ready` succeeds and lists a bead that is not an epic and not
#   labelled `owner`.
# Otherwise, including on any missing or unreadable input, it allows the
# stop and prints nothing. The context report itself is the owner's
# user-level PostToolUse hook, not this one.
#
# Input: the Stop hook's JSON on stdin. Needs jq, tac, and br.
# Test: scripts/handoff-guard-test.sh
set -u
LIMIT=350000
PROPOSAL='\bhand(ing)? ([^ .]+ ){0,3}off\b|handoff (now|here)|write (a|the) handoff|(start|open|begin|use) a (fresh|new) session|pick (it|this|that) up in a (fresh|new) session|\bthe next session\b|next session (can|should|will)|prime, then|(should|shall) i (continue|keep going|go on|proceed|hand)|(would you like|do you want|want) me to (continue|keep going|go on|proceed|take|pick|start)|continue or (hand|stop|wrap)|stop here|good (place|point) to (stop|pause)'
# Not bare "handoff": "continue <bead> from its handoff" starts sessions. A
# typed /handoff is recorded as "<command-name>/handoff</command-name>". A
# bare prime asks for no work, and its summary often recites the handoff
# rule, so it counts as a stop too: the owner directs what follows.
OWNER_STOP='(^\s*|/)handoff\b|\bhand (it |this )?off\b|(do|write) (a|the) handoff|handoff now|wrap (it )?up|\bstop\b|\bpause\b|^\s*prime\s*$|<command-name>/prime(-beads)?<'
NEGATED='\b(don.?t|do not|never|without|no need to|not) (stop|pause|hand ?off)'

[[ "${CLAUDE_CODE_ENTRYPOINT:-}" == cli ]] || exit 0
input="$(cat)"
[[ "$(jq -r '(.stop_hook_active // false) == false
  and ((.background_tasks // []) | length) == 0
  and ((.session_crons // []) | length) == 0' <<<"$input" 2>/dev/null)" == true ]] || exit 0
transcript="$(jq -r '.transcript_path // empty' <<<"$input")"
[[ -r "$transcript" ]] || exit 0

# last FILTER: print the newest transcript entry matching the jq filter.
last() { tac "$transcript" | jq -c "select($1)" 2>/dev/null | head -n1; }
# text: join an entry's message content, a string or text blocks.
text() { jq -r '.message.content | if type == "string" then .
  else [.[] | select(.type == "text") | .text] | join("\n") end'; }

# The newest real response must carry its own usage; an older count is stale.
used="$(last '.type == "assistant" and .message.model != "<synthetic>"' | jq -r '.message.usage
  | .input_tokens + (.cache_creation_input_tokens // 0) + (.cache_read_input_tokens // 0)')"
[[ "$used" =~ ^[0-9]+$ ]] && ((used > 0 && used < LIMIT)) || exit 0

message="$(jq -r '.last_assistant_message // empty' <<<"$input")"
[[ -n "$message" ]] || message="$(last '.type == "assistant" and .message.model != "<synthetic>"
  and any(.message.content[]?; .type == "text")' | text)"
grep -qiE "$PROPOSAL" <<<"$message" || exit 0

prompt="$(last '.type == "user" and .origin.kind == "human"' | text)"
[[ -n "$prompt" ]] || exit 0
sed -E "s/$NEGATED//Ig" <<<"$prompt" | grep -qiE "$OWNER_STOP" && exit 0

beads="$(br ready --json 2>/dev/null)" || exit 0
ready="$(jq -r '[.[] | select(.issue_type != "epic" and ((.labels // []) | index("owner") | not))
  | .id] | .[:3] | join(", ")' <<<"$beads" 2>/dev/null)"
[[ -n "$ready" ]] || exit 0

jq -n --arg reason "Context is about $((used / 1000))k tokens, under the 350k mark \
(AP-SESSION-001 stops new scope only at 400k), and ready beads remain: $ready. Do not \
propose a handoff or a fresh session, and do not ask whether to go on: finish the bead in \
hand, or, once it is closed, claim the next ready bead and work it (AGENTS.md, local \
exception to AP-PLAN-001 for attended runs). Stop again if the owner asked you to stop, \
if you asked the owner a scope question, or if none of these beads is yours to take; \
this hook blocks only once." '{decision: "block", reason: $reason}'
