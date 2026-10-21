# Tide review, fx-h6o round 3 (Opus): converge round

Candidate: worktree /home/lucca/pjs/lab/tide-fx-h6o, branch fx-h6o, HEAD 187a22b plus the uncommitted diff, SHA-256 9cfe8bc1ab578808fd03a35cb56548f7218bb1113e5b15fcaa1370eb32752156 (README.md, scripts/handoff-guard.sh, scripts/handoff-guard-test.sh). This round changed nothing, so the hash is the same afterwards.

## Verdict

clean

## Findings and fixes

None.

## Not changed

- **Unreadable input can write to stderr.** If the transcript path is a directory, tac prints "read error". If an owner prompt has `content: null`, jq prints "Cannot iterate over null". In both cases the hook exits 0 and allows the stop. The header's "prints nothing" means no decision on stdout. Claude Code does not show a Stop hook's stderr when it exits 0, so this is a nit.
- **A malformed final transcript line makes the hook allow the stop.** tac puts that line first and jq stops there. I replayed 319fa31b with a broken line appended: block became allow. This fails open, which is the intended direction, and Claude Code appends whole lines. Skipping bad lines with `fromjson?` would add code only to avoid a miss.
- **A curly apostrophe in "don’t stop".** Under a UTF-8 locale, NEGATED strips it and the hook blocks, which is correct. Under `LC_ALL=C`, the "stop" survives and the hook allows the stop, which fails safe.
- **73454788 (178k).** The owner asked "will it break this claude?", and the answer's "Start a new session in `~/pjs/personal/tide`" matches PROPOSAL, so the hook would block. That is the question-only case round 2 decided against exempting, with evidence. One block bounds the cost, and the owner can interject. I am not re-litigating it.
- **ca66305f (329k)** blocks on a descriptive "the next session starts it only after fx-kw8.1 lands". Round 2 already recorded this class.
- **Threshold test.** The 250k/450k pair follows the bead's Verification. A LIMIT moved to 400k would still pass it. The bead's contract names those two points, so this is a nit.

## Checked

- Read: the candidate diff (hash verified before and after), AGENTS.md, GOVERNANCE.md Merge Rule, `br show fx-h6o` with its decisions comment, and the round 1 Opus, round 1 Codex, and round 2 Opus reviews.
- Ran: scripts/handoff-guard-test.sh, with 32/32 ok and exit 0 (TMPDIR in the scratchpad). Also ran `git diff --check` and `bash -n` on both scripts, all clean. shellcheck is not installed.
- Replayed every transcript in ~/.claude/projects/-home-lucca-pjs-lab-tide as a whole file through the hook, with `CLAUDE_CODE_ENTRYPOINT=cli` and a stub `br` that lists one task.
  - 11 blocked. For each, I extracted the final message's matching phrase. Nine are real early-handoff offers below 350k: 319fa31b, 182a586e, 5f3ebd57, 3536eb7f, f54f050d, 0af5672b, e1f15d82, 186ae567, and 0c465b0b, almost all "prime, then pick/continue …" lines. Of the other two, ca66305f is descriptive and 73454788 is the question case above.
  - The two newest sessions (e82b34d5 at 223k and 4309a5b5 at 207k) allow the stop, and so does every transcript at or above 350k.
- Probed fail-open paths: non-JSON stdin, empty stdin, a directory as the transcript, a malformed final line, a null prompt content, a string `stop_hook_active`, and `LC_ALL=C`. Each one allows the stop.
- Confirmed in the Claude Code 2.1.293 binary that Stop input carries `background_tasks`, `session_crons`, `last_assistant_message`, and `stop_hook_active`. This session's environment has `CLAUDE_CODE_ENTRYPOINT=cli`.
- Hooks resolve `grep` to /usr/bin/grep (GNU). The ugrep wrapper exists only as a function in the agent's interactive Bash tool and is not exported to scripts.
- Ran `br ready --json` in 0.04 s, read-only. It returns an array whose items have `issue_type` and `labels`.
- Did not invoke `claude`, commit, or touch `.claude/settings.json` or the protocol files.
