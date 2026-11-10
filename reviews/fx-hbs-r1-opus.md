Model: claude-opus-5-5 · Effort: medium · Date: 2026-11-09

# fx-hbs round 1 — Opus review (docs review round)

## Verdict

request_changes, confidence 0.75. The rules match the owner's decisions and the eon-overseer agreement. The exception record has all six AP-EXCEPTION-001 fields. The drift check is clean and AGENTS.md is 22,626 bytes. Two rule sentences need fixing before landing:

- The pane-launch sentence applies to every session. Headless and converge-driven sessions cannot follow it, and the owner's decision only covered the coordinator's own tab.
- The Sessions bullet tells another session to claim a bead, but the Writes bullet forbids that session from writing through `br`.

Both fixes are one clause each.

## Scope

Candidate `d8f2ed8` on base `b4f1302` (`git diff b4f1302 d8f2ed8`): `.agent-protocols.local.md` (new Parallel work section), `.agent-protocols.exceptions.json` (AP-PLAN-001 shared-file exception), the re-rendered `AGENTS.md` and `.agent-protocols.json`, and `README.md:323-324`.

What I read:
- the bead and all its comments (`br --no-db show fx-hbs`);
- `HBS-REPORT.md`, `apply-brief.md` and `apply-report.md`;
- `GOVERNANCE.md` in full;
- the AGENTS.md diff and its Agent roles, Autonomous runs and Verification sections;
- the README Project Process section;
- the header and the `pick`, `live_agent` and `is_handoff` functions of `scripts/autorun.sh`;
- `design/reference-companion.md:12-15` and D14;
- the `converge` skill's launch steps;
- the owner-decision memory notes the coordinator relied on (`agent-concurrency-limit`, `heavy-run-agreement`, `codex-jobs-in-eon-panes`).

What I ran:
- I grepped the repository (outside archive/research/reviews/trials) for other text about sessions, fronts, coordinators, `heavy.sh` and quiet windows. Only `GOVERNANCE.md:75` ("parallel sessions") and the README line touch this, and neither conflicts.
- I re-ran the check at the candidate commit (HEAD = d8f2ed8 with a clean tracked tree) against the pinned source. The source is clean at 37bc4b0. `starcompass --root …/starcompass-37bc check …` returned `valid: AGENTS.md matches .agent-protocols.json`, exit 0. `git show d8f2ed8:AGENTS.md | wc -c` gives 22626.

## Blocking

**B1 — The launch rule binds sessions that cannot follow it and goes beyond the owner's decision.** `.agent-protocols.local.md:110-112` (AGENTS.md "Parallel work", Agents bullet): "Launch Opus or Codex (`gpt-6.1-sol`) as interactive CLIs in panes of the coordinator's Eon tab."

The owner's words were "run two agents in this tab, in panes" and "use opus and sol, not fable" (bead comment 2026-11-10 00:07 UTC). That is about the attended coordinator's tab. The rule as written applies to every Tide session, and two kinds of session cannot follow it:

- **Unattended runs.** By the overlay's own definition ("the one that started first"), a headless `scripts/autorun.sh` session (`claude -p`) is the coordinator. It has no Eon tab, and the runner's header says its sessions "run br, cargo, and codex". Under this sentence such a session cannot launch the GOVERNANCE.md Codex pass or the Opus review in a compliant way. Under Autonomous runs it would then block the bead.
- **Converge.** AP-PROOF-001 says "in Claude Code, use the converge skill", and GOVERNANCE.md makes the Opus review "a converge round". Converge launches its reviewers as an Agent-tool subagent and as `codex exec` (converge SKILL.md, lines 40 and 49), not as interactive panes. Read literally, the sentence forbids the review mechanism the canonical protocol names. AP-SESSION-001's default ("subagents are authorized unless … local rules say otherwise") becomes unclear for the same reason.

The text also leaves open whether in-session subagents count toward the four. The coordinator's record of the owner's intent says they do ("Count every live agent … Opus review subagents").

**Smallest fix:** limit the sentence to what the owner decided, and say how subagents count. For example:

> "When the coordinator runs in Eon, it launches them as interactive Opus or Codex (`gpt-6.1-sol`) CLIs in panes of its tab. Subagents and background review passes count toward the four."

Leave unattended sessions under their own rules. This fix changes a rule's reach, not just wording, so it needs a re-check.

**B2 — The Sessions and Writes bullets contradict each other on claiming.**

- `.agent-protocols.local.md:116-117`: "Any other Tide session reports to the coordinator before it claims a bead."
- `.agent-protocols.local.md:118-119`: "only the coordinator writes to the main checkout, to `main`, and through `br`."

A claim is a `br` write. The first sentence assumes the other session claims the bead itself. The second forbids it. A session that follows the first sentence writes `.beads/issues.jsonl` from a second session, which is the failure this decision exists to prevent: the 2026-10-21 dirty ledger that aborted `git merge --ff-only` (bead comment 2026-10-21). AP-COORDINATOR-001 already says other sessions "follow its schedule and report … to it", so the coordinator should do the claim.

**Smallest fix:** "Any other Tide session asks the coordinator to claim a bead for it." Alternatively, scope the Writes bullet to workers. The first fix keeps the decision's intent.

## Non-blocking

**N1 — The bead cannot close yet.** Its Acceptance item 1 is still open: "The owner records the decision on this bead: no fronts, or named fronts". The comments record the owner's cap, the pane and model choice, and the exception approval. "No standing fronts" itself appears only as the decision agent's conclusion in the 23:15 checkpoint. When closing, post the decision comment and say whose call it is: the owner's, or the coordinator's under the owner's delegation. The bead labels it an owner call, so this needs saying. Acceptance item 2 is conditional on adopting fronts. The candidate still meets its substance: the rules are in the overlay, rendered, and the drift check is clean.

**N2 — The exception's attribution to GOVERNANCE.md is loose.** The exception scope says the reviewers re-check what the rebase changed "under GOVERNANCE.md's review loop". GOVERNANCE.md's loop defines re-checks only per finding, by the reviewer that found it. It has no rebase re-check and does not say which reviewers. The exception itself is what creates this duty, as approved: "that rebase is re-checked". Consider naming who does it, for example "each first-round reviewer re-checks what the rebase changed, taking the next round number", and drop the attribution. The current wording works and was approved, so this does not block.

**N3 — "Full suites" is not defined.** In the Heavy runs bullet (`.agent-protocols.local.md:131-138`), only "full suites" go through `heavy.sh`, and only "focused single-crate checks" may run outside it. The agreement defines full suites as `cargo test --workspace` and `conformance/run.sh`. The bullet does not, and it says nothing about other compiling runs (spike or trial scripts, a whole-workspace `cargo build`), which fall between the two classes. The draft had covered them. Naming the two suites and saying where other compiling scripts go would close the gap.

**N4 — The exception renders awkwardly.** The scope is several sentences starting with "The", so the Local exceptions entry reads "`AP-PLAN-001` in The 'no shared files' test … review loop.: Most ready …" (`AGENTS.md:34`). A noun-phrase first clause would read cleanly, with the details moved into later sentences. This is a cosmetic issue with the renderer's format.

**N5 — The exception's evidence is slightly overstated.**
- "On 2026-11-07 fx-xrx, fx-aw6, fx-kw8.9 and fx-zr3 … landed by rebasing": fx-xrx landed first, without a rebase, and the others landed on 11-07 and 11-08 local time (HBS report §2 and §4).
- Part of that overlap was on `design/reference.md`, which this decision now makes a single-holder file.

The claim still holds for the design chapters and `conformance/rules.md`. Optional rewording: "overlapped on such files; those that rebased landed with clean rebase checks."

**N6 — Who rebases is split between two texts.** The overlay's Landing bullet says the coordinator rebases. The exception says "The later bead rebases, and its implementer resolves any textual conflict". These are compatible: the coordinator rebases and the implementer resolves conflicts in its worktree. One phrase in the exception, "the coordinator rebases the later bead; its implementer resolves …", would remove the ambiguity.

**N7 — The README sentence is mostly backed by the script.** README.md:323-324 says the runner "does not run while a coordinator has beads in flight". The script mostly enforces this:
- `live_agent` sees any claude or codex process whose working directory is in the checkout, including `.worktrees/*`;
- `pick` stops on an in-progress bead whose latest comment is not a "Handoff".

A gap remains only if the coordinator is not running and every in-flight bead ends in a "Handoff…" comment. The sentence cites AGENTS.md, so it reads as a rule as well as a description. That is acceptable as written.

## Reasoning

**Decisions matched against the sources:**
- **Cap.** Four besides the coordinator, with implementers, reviewers and research agents all counted. This matches the owner's "i allow a third and fourth slot" (comment 00:53 UTC).
- **Opus and Sol only.** Matches "use opus and sol, not fable".
- **No cap on beads in flight; nearest-to-landing first.** Matches the apply brief's decision 2.
- **Heavy runs.** One slot; full suites run serially by the coordinator; focused single-crate checks under `nice -n 19` with `CARGO_BUILD_JOBS=4`, one at a time, outside `quiet-until` windows; booked quiet slots avoided; timed runs ask for a quiet slot; eon-overseer owns the policy files. This matches the recorded agreement point for point.
- **Single-holder files.** `design/reference.md`, its companion count (`reference-companion.md:12-14`, budget from D14) and the governance and rendered files. This matches the decision report §5 and risk 7.
- **Removed sentences.** The draft's serial-landing, branch-freeze and worktree-cleanup sentences were cut. All three are already in AP-PLAN-001, AP-COORDINATOR-001 and AP-GIT-001, so nothing was lost.
- **Main, `br` and ledger.** The new text adds only the default choice of coordinator and the main/`br`/ledger ownership. I found no conflict with GOVERNANCE.md (unchanged) or the Agent roles section.

**Exception record (AP-EXCEPTION-001):**
- Fields: protocol `AP-PLAN-001`, a precise scope, reason, approver `lucca (owner)`, date `2026-11-09`, and a review condition with an end condition. All six are present, and the manifest copy is identical.
- Approval: the question put to the owner named this exception and its recommendation. The owner answered "whatevery you prefer", which delegates this specific choice to the recommendation. That is unlike the blanket delegation the decision report rightly refused to treat as approval. The question was asked directly, not relayed, so the AP-COORDINATOR-001 relay rule does not apply.
- Date: the comment's UTC time is 2026-11-10 00:07, which is 2026-11-09 local, so the recorded date is right.
- Scope: the recorded scope only narrows what was asked (adds "no dependency" and "not a single-holder file"), so the approval covers it.
- Stale wording: the "narrow re-check" phrase that fx-efo made stale is gone.

**Why B1 and B2 block:** each is a rule that, read as written, either cannot be followed or recreates a recorded failure. The fix for each changes what the rule requires, so neither is a text-only fix and both need the re-check. Everything else above is wording or completeness and does not block.

DONE
