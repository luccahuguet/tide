# Tide review, fx-h6o round 1 (Opus): converge round (report only)

Candidate: worktree /home/lucca/pjs/lab/tide-fx-h6o, branch fx-h6o, HEAD 187a22b plus uncommitted diff SHA-256 1d7e792d48be3fb0c18c3803a4e54f9aab47d04d0a37685190d0ef5d0f557b1e (README.md, scripts/handoff-guard.sh, scripts/handoff-guard-test.sh), plus the proposed `.claude/settings.json` content quoted in the brief.

## Verdict

changes requested

## Findings

1. **The block reason tells the agent to break AP-PLAN-001 (blocking).** scripts/handoff-guard.sh:58-62.
   AP-PLAN-001 (AGENTS.md:151-160) says that "a run includes continuations until control returns", that a writing run "owns at most one issue", and that it must "return at completion". A Stop-hook block is a continuation, so control has not returned. "Claim the next ready bead and work it" therefore asks for a second issue in the same run. `.agent-protocols.exceptions.json` is empty and the local overlay says nothing about this case.
   Failure scenario: the agent closes fx-a and offers a handoff, and the hook blocks. A careful agent follows AGENTS.md over the hook and stops again, so the bead's first acceptance item fails in the behaviour trial. A less careful agent claims fx-b and breaks the imported protocol.
   Smallest fix: record an AP-EXCEPTION-001 exception in `.agent-protocols.exceptions.json`, with the owner as approver, and re-render AGENTS.md. The bead's scope already allows the protocol files. Scope the exception to attended sessions where scripts/handoff-guard.sh blocked a stop, and say that closing the bead in hand counts as returning control for AP-PLAN-001, so the next ready bead starts a new run. Then have the reason cite that exception. Narrowing an import needs the owner's approval, and the owner is already creating settings.json by hand, so both can happen in one step.

2. **"Owner's last prompt" also matches task notifications, `!` shell input, and compaction summaries.** scripts/handoff-guard.sh:49-51.
   In the 29 local transcripts, 87 of about 240 entries that pass the filter are `<task-notification>` entries (`origin.kind: "task-notification"`, `promptSource: "system"`). Others are `<bash-input>`/`<bash-stdout>` entries and "This session is being continued…" summaries. This repo runs reviews in background subagents, so a task notification is often the newest entry. When it is, the owner's real stop request is hidden.
   Reproduced on a scratch transcript: owner prompt "wrap up and hand it off", then a task notification, then a final message "Handoff written on fx-a. Next session: prime, then continue fx-a from its handoff." The hook returns `block`.
   Smallest fix: select `.type == "user" and .origin.kind == "human"`. In the corpus, every typed or queued prompt carries it, including a skill invoked by slash command.

3. **The `/handoff` slash command is not treated as a request to stop.** scripts/handoff-guard.sh:29.
   A typed `/handoff` is recorded as `"<command-message>handoff</command-message>\n<command-name>/handoff</command-name>"` (same shape as the real `/prime` and `/prime-beads` entries). `^\s*/?handoff\b` matches neither line.
   Reproduced: an owner `/handoff` followed by "Start the next session with: prime, then continue fx-a from its handoff." returns `block`. This is the handoff skill's own required closing line, so the hook fires on every `/handoff` taken under 350k with a ready bead.
   Smallest fix: `(^\s*|/)handoff\b` in place of `^\s*/?handoff\b`.

4. **Turns that end to wait for background work get blocked.** scripts/handoff-guard.sh:31-35.
   Real turn-final messages in this corpus end the turn while a review runs, for example 9cc7dc47 "A fresh agent is now running the round-35 confirmation review." (three times), e1f15d82 "Converge round 1 is running … with a fresh agent", and 119d07a5 "Next session: … The round-8 review is still running". The hook blocks these with "finish the bead in hand, or claim the next ready bead". The agent cannot finish until the review reports, so it either wastes a turn or starts a second bead alongside the review.
   Claude Code 2.1.293's Stop input carries `background_tasks`, described in the binary as: 'In-flight background work … Lets hooks distinguish "session is done" from "session is paused waiting for background work to wake it". Empty array when nothing is in flight.' The same applies to `session_crons`.
   Smallest fix: after line 33, add `[[ "$(jq '(.background_tasks // []) + (.session_crons // []) | length' <<<"$input" 2>/dev/null)" == 0 ]] || exit 0`.

5. **`(fresh|new|next) (agent|session)` is the main false-positive source, and the regex misses the standard next-session line.** scripts/handoff-guard.sh:27.
   I ran the PROPOSAL regex over every turn-final assistant message in the 29 local transcripts. It matched 29 messages. Nine were "fresh agent" in ordinary reports or answers, where "fresh agent" is this repo's word for reviewers and trial subjects: 0c465b0b (explaining converge), 3748816d (trial design), 73454788 (diagnostic scoring), 9cc7dc47 ×3, c697036a, e1f15d82 ×2. In the other direction, 73454788's "Next up is still `prime, then continue fx-origin-syntax-trial-u49 from its handoff`" did not match, and the owner answered it with "we have a lot of context continue fx-origin-syntax-trial-u49". That is the exact behaviour the bead targets.
   Smallest fix: replace that alternative with `(fresh|new|next) session|prime, then`. On the same corpus this gives 24 matches, none on "fresh agent", and it catches both 73454788 next-session lines. The remaining matches are real offers, plus a few quotes such as "continue or handoff?" and "new session" used to describe autorun.

6. **The reason does not exempt turns where the owner only asked a question.** scripts/handoff-guard.sh:60-62.
   AP-SCOPE-001 says explanation and advice authorize no implementation. Suppose the owner asks "why did round 3 fail?" and the answer happens to match PROPOSAL (reproduced with the current "fresh agent" alternative). The hook then orders "claim the next ready bead and work it", and the escape clause covers only "asked you to stop" or "none of these beads is yours".
   Smallest fix: extend the escape clause to "If the owner's last prompt asked only a question or asked you to stop, or none of these beads is yours to take, stop again". Fold this into the rewording for finding 1.

7. **The tests use simplified synthetic transcripts and missed findings 2-5.** scripts/handoff-guard-test.sh:27-36.
   The `transcript` helper writes user entries without `origin`. It has no task-notification entry, no slash-command prompt shape, and no `background_tasks` input. The bead's Verification asks for "a recorded transcript tail". As written, the test pins behaviour that is wrong on real transcripts.
   Smallest fix: add `origin: {kind: "human"}` to the prompt line, and add four cases: a task notification after "wrap up and hand it off" (allow), a `/handoff` command entry (allow), non-empty `background_tasks` (allow), and "Next up is `prime, then continue fx-b`" (block). Optionally replace the synthetic lines with a few real redacted lines.

8. **Nit: synthetic assistant entries count as zero context.** scripts/handoff-guard.sh:40.
   Entries with `.message.model == "<synthetic>"` ("No response requested.", "You've hit your session limit …") have all-zero `usage`; four are in the corpus. If one is the newest usage entry, `used=0` passes the 350k check whatever the real context is.
   Fix: add `and .message.model != "<synthetic>"` to the usage filter.

9. **Nit: `hand (it|this|.{0,30}) off` is redundant and misses the common phrasings.** scripts/handoff-guard.sh:27.
   `.{0,30}` already covers `it` and `this`. No alternative matches "hand off" with a single space, and "should i" does not cover "shall I". Reproduced: "Should I hand off now?" and "Shall I continue with fx-b?" are both allowed.
   Fix, if wanted: `hand( \S+){0,2} off|(should|shall) i (hand|continue|keep going|go on)`.

10. **Nit: one phrase reads as self-contradictory.** scripts/handoff-guard.sh:59 and README wording source.
   "AP-SESSION-001 adds no new scope only from 400k" reads backwards. Fix: "AP-SESSION-001 stops new scope only at 400k".

11. **Non-blocking, proof gap for author decision 1 and the bead's second acceptance item.**
   ~/.claude/hooks/context-usage.py uses the same formula as the guard, which is good, but it reports only when usage crosses a new 100k step. Between steps the agent's newest figure can be up to about 100k old. For example, it may still say "about 200k" at 290k, which is 31% off against the bead's 10% bound. The script and its `claude-context` query (on PATH at ~/.local/bin) are user-level only, and nothing in the repo tells agents that `claude-context` exists. Not adding a second reporter is the right call, but record on the bead that the 10% item rests on the user hook, and check it in the behaviour trial. The guard's own reason carries the exact count, so the number is exact at the moment the guard blocks.

12. **Nit, sequencing: the README says the hook is wired in `.claude/settings.json` (README.md:187-188), but the candidate has no such file.** Land the owner-created settings.json in the same commit, or the README is false until it exists.

## Not changed

- **Pipefail / SIGPIPE:** the script uses `set -u` without pipefail, and `tac | jq | head -n1` only needs head's first line. I re-ran it with SIGPIPE ignored, as a parent process might set it: correct result, no stderr noise.
- **Cost:** the hook takes 0.01-0.03 s on the largest transcripts (4.9 MB and 0c465b0b). The 30 s timeout is ample.
- **Attended detection (decision 2):** confirmed in the 2.1.293 binary. If `CLAUDE_CODE_ENTRYPOINT` is inherited as `cli` and the run is non-interactive, Claude Code resets it to `sdk-cli`. So `claude -p` from scripts/autorun.sh is left alone even when launched from inside an attended session. Desktop and VS Code entrypoints are also excluded, which only causes misses, so it is fail-safe.
- **Dropping "no bead in progress for this session" (decision 3):** agreed. br cannot attribute a claim to a session, and the reason already says to finish the bead in hand first.
- **One block per owner prompt via `stop_hook_active`:** correct, and it can never trap the owner. The cost is that the hook adds at most one extra bead per owner prompt. That is an acceptable trade-off and is documented.
- **OWNER_STOP over-matching** ("stop the timer bug first", "don't pause"): these errors allow the stop, so they are fail-safe. Not worth tightening.
- **Fail-open:** I checked truncated stdin JSON, a non-boolean `stop_hook_active`, a missing transcript, br failing, and a ready list with only epic or owner beads. Each exits 0 silently. A corrupt transcript line only stops jq early, which also allows the stop.
- **`br ready --json` shape:** it is an array whose items carry `issue_type` and `labels`. The live ready list correctly drops fx-hbs (`owner`). It gives the same result from the worktree.
- **Proposed settings.json:** valid. Stop takes no matcher, `"$CLAUDE_PROJECT_DIR"` is quoted, and the timeout is 30 s. No change.
- **Bare "handoff" kept out of PROPOSAL:** right. In the corpus it appears mostly in rule recitations and "from its handoff". `prime, then` (finding 5) is the narrower signal for the standard next-session line.

## Checked

- Read: the candidate diff (hash verified), AGENTS.md (AP-PLAN-001, AP-SCOPE-001, AP-EXCEPTION-001, local rules), `br show fx-h6o`, ~/.claude/hooks/context-usage.py, ~/.claude/settings.json hooks, `.agent-protocols.exceptions.json`, `.agent-protocols.local.md`, and the scripts/autorun.sh invocation.
- Ran: scripts/handoff-guard-test.sh (14/14 ok, exit 0).
- Ran: the hook with `bash -x` against real transcripts 3748816d (519k, allowed at the limit check) and 0c465b0b (302k), with timing.
- Ran: scratch transcripts in my scratchpad with a stub br for each reproduction above.
- Ran: a corpus scan of the PROPOSAL regex and of the proposed variant over all turn-final assistant messages in ~/.claude/projects/-home-lucca-pjs-lab-tide/*.jsonl, classifying matches by the owner's next prompt.
- Ran: a classification of the entries the prompt filter accepts, by `origin.kind` and `promptSource`.
- Ran: `br ready --json` (read-only) from the main checkout and the worktree.
- Grepped: the Claude Code 2.1.293 binary for the `CLAUDE_CODE_ENTRYPOINT` handling and the Stop input schema (`last_assistant_message`, `background_tasks`, `session_crons`).
- Did not invoke `claude`, and did not edit the worktree.
