# Tide review, fx-h6o round 2 (Opus): converge round

Candidate: worktree /home/lucca/pjs/lab/tide-fx-h6o, branch fx-h6o, HEAD 187a22b plus the uncommitted diff, SHA-256 efca7c73af3268512afcac43668e333d8d7a97eb2b2697985511be1b93f103f4 (README.md, scripts/handoff-guard.sh, scripts/handoff-guard-test.sh). After this round's fix the diff's SHA-256 is 9cfe8bc1ab578808fd03a35cb56548f7218bb1113e5b15fcaa1370eb32752156.

## Verdict

fixed

## Findings and fixes

1. **The hook blocks a bare prime and sends the agent into a bead the owner did not choose.** scripts/handoff-guard.sh, OWNER_STOP.
   A prime summary recites the repo's workflow rules, and AP-SESSION-001's rule always contains "hand off" ("Context budget: hand off at 400k tokens", "I hand off before the context would get compacted"). PROPOSAL's `\bhand(ing)? ([^ .]+ ){0,3}off\b` matches that recitation. So, under 350k with a ready bead, the hook blocks the stop and tells the agent to "claim the next ready bead and work it". A bare prime is inspection only, and under AP-SCOPE-001 it authorizes no implementation.
   Corpus: the owner sent a bare prime six times (`prime` four times, `/prime`, `/prime-beads`). Five times the next prompt picked a different direction: "what should we do next?", "adopt latest starcompass", a Rust-vs-Bend question, "review the past claude threads…", and "work autonomously with the other agent". Only after `/prime-beads` did the owner name a bead to continue. I replayed three real transcripts, each cut just before the owner's next prompt (c697036a, 5375ee76, ca66305f), through the candidate hook with a stub `br`. All three returned `block`.
   Fix: OWNER_STOP also matches a prompt that is only `prime`, and the `<command-name>/prime</command-name>` and `/prime-beads` command forms. Each counts as a stop. "prime, then continue …", "prime and tell me what should we do next", and "prime then pick the next bead" still block. 4309a5b5's "prime and tell me…" still blocks on replay, and there the owner answered "figure out yourself. continue". The header bullet now says "and was not a bare prime". Two prompt cases were added to the test (`prime` and the `/prime` command form). With either new alternative removed, its case fails.

## Not changed

- **Round 1 Opus finding 6 (exempt prompts that only ask a question): rightly not adopted.** In the corpus the bead's targeted misses are themselves questions: "whats next?" (0af5672b, answered with "do fx-eky"), "we are at 29%. will fx-i5p fit?", and "we only used 20% of ctx. what is next?". A question-only exemption, mechanical or in the reason text, would let exactly these stop. The bare-prime exemption above covers the inspection-only class that the corpus actually shows. The reason's "if you asked the owner a scope question" escape covers the audit replies (c697036a 92k and 103k, 73454788 102k).
- **"now or handoff?" and "continue or handoff?" from the owner** (186ae567, 3536eb7f, 5f3ebd57, e1f15d82) are not stop requests. The owner delegates the choice, and the policy under 350k answers "now", which matches the owner's later 400k memory and fx-h6o's evidence. OWNER_STOP does not match them, which is correct.
- **Descriptive "hand off" and "the next session" in reports** (for example ca66305f 320k and 329k, and 4309a5b5 70k "stopped without handing off") still match PROPOSAL. Outside primes, the corpus shows no harmful case that the escape clauses, background-task checks, or the owner's reply did not already cover. Tightening the regex further would be whack-a-mole, and the one-block limit bounds the cost.
- **Round 1 findings that are fixed:** Opus 2 (origin "human"), 3 (`/handoff` command form), 4 (`background_tasks` and `session_crons`), 5 ("fresh agent" removed; `prime, then` and "the next session" added), 7 (the test's transcripts carry origins, a task notification, a meta entry, a synthetic response, and slash-command prompts), 8 (synthetic responses skipped for both usage and message), 9 ("Should I hand off", "Shall I continue"), 10 (wording), and 12 (README says the owner must add the Stop entry). Codex 1 and 2 are covered by test cases. Codex 3: `usage: {}` gives 0 and is allowed, a missing owner prompt is allowed, and `br` exiting non-zero is allowed through `|| exit 0` on the assignment. Codex 4: `mktemp … || exit 1`.
- **Round 1 Opus 1 (AP-PLAN-001 exception)** is pending the owner, as the brief says. I did not add it.
- **Round 1 Opus 11 (accuracy of the 10% context report)** concerns the bead's acceptance and proof, not the diff. It stays open, together with the behavior trial that is to be observed in the owner's first attended session.
- **Nits left alone:** the test's `/handoff` and `/prime` prompts carry a literal `\n` where a real entry has a newline. Matching does not depend on it, because both alternatives match within one line. The reason text hardcodes "350k" next to `LIMIT`.

## Checked

- Read: the candidate diff (hash verified), AGENTS.md, GOVERNANCE.md Merge Rule, `br show fx-h6o` with its comment, both round 1 reviews, and `.agent-protocols.exceptions.json` (empty).
- Ran: scripts/handoff-guard-test.sh before and after the fix (30/30 ok, then 32/32 ok, exit 0, with TMPDIR set to the scratchpad).
- Ran: a mutation check of each new OWNER_STOP alternative. Removing either one fails its test case. I also reasoned through the existing conditions: the cli gate, stop_hook_active, background_tasks, session_crons, `used > 0`, LIMIT, the synthetic filters on usage and message, the origin filter (the meta "stop" entry would trip it), the empty-prompt check, the epic and owner filters, and the `br` status. Each has a case that fails if it is mutated.
- Ran: a corpus scan (Python, read-only) over the 29 transcripts in ~/.claude/projects/-home-lucca-pjs-lab-tide. It covered 201 turn-final assistant messages, of which 38 matched PROPOSAL, and each match was classified by its owner prompt, context, and the owner's next prompt. It also covered all 124 `origin.kind == "human"` prompts against OWNER_STOP. Before the fix, 0 of 124 matched; after it, exactly the 6 bare primes match. Origin kinds present: human 124, peer 247 (subagent hand-backs), task-notification 88, auto-continuation 1 (usage-limit reset), none 2530. No sidechain entries and no malformed lines.
- Ran: a replay of real transcript prefixes through the hook with a stub `br`: c697036a, 5375ee76, and ca66305f went from block to allow, and 4309a5b5 still blocks.
- Did not invoke `claude`, commit, or touch `.claude/settings.json` or the protocol files.
