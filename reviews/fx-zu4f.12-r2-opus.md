Model: Claude Opus 5.5 · Date: 2026-11-19 · Candidate 1caca4f (research record)

## Verdict

**changes required.** One blocking finding (privacy, identification through committed project state), plus one escalation outside the record that must be handled before the repository goes public. The method findings are sound: "not caught in either arm" follows from the registered rules as written, and the arm-A split and the candidate's status are stated correctly.

## Scope

Frozen candidate: worktree `tide-fx-zu4f.12`, HEAD `1caca4fe1ce6048e3b3ced5b17c85445ebc0787f`; the only file the branch adds after the run merge is `research/50-fx-zu4f.12-crypto-crates-round.md` (SHA-256 `04df37fa…7797e`; `git diff b79cbbe 1caca4f` SHA-256 `1bcab2a9…55a9`). Read: AGENTS.md, GOVERNANCE.md (Merge Rule), `br --no-db show` and `comments` for fx-zu4f.12, the registered protocol (§0, Disclosure and privacy, Safety, Neutral framing, §5-§8), the record in full, target/work/r1-codex.md, r1-recheck-codex.md, brief-r1-fixes.md, answer-r1-b2-privacy.md and report.md (reading-observation tallies), the private property log (digest checked: `2628892b…bdd2b`, matches the record), the round-1 run notes and the private reading-observation file headings. I also checked `main:.beads/issues.jsonl` and the branch's own commit messages for leaks. Codex's verified items (hashes, OTS, pool reconstruction, arm-B evidence, log contents) were not repeated. No build, run, network, subagent, commit or `.beads/` write. Nothing private is quoted here, and no rank or crate is tied to a defect.

## Blocking findings

### B1. The record's per-crate arm-A counts, differenced against the committed ledger, identify the private candidate's crate

- **Where.** `research/50-fx-zu4f.12-crypto-crates-round.md:361-371`: the per-crate Rows, Modeled and Tests columns. Read them with `:225`, `:355-356` and `:399-410`, which say round 1 modeled exactly 2 claims and that those 2 claims are the candidate.
- **Evidence.** fx-zu4f.12's run comments are already committed on `main` in `.beads/issues.jsonl`, through commit 53afa55 and earlier ledger commits. The repository will go public with its full history. Those comments give per-rank arm-A counts written before round 1 (properties listed and modeled per rank). Exactly one rank's modeled count in the record is 2 higher than the ledger's earlier figure, and the record says round 1 modeled 2 claims and they are the candidate. A reader who compares the two public sources gets the candidate's crate (through `pool.tsv`), which breaks protocol Disclosure and privacy: "only a count of private candidates, with no project, crate or revealing detail". The ledger's Rows figures also differ for the ranks whose conflicting-source rows round 1 listed per claim. report.md notes that conflicting-source rows went to reading observations, so this is a second, weaker pointer. Codex's re-check covered the cross-version hazard against the retired record (1270d9e is unreachable) but not this one. tide-lead's decision to keep per-crate modeled counts (answer-r1-b2-privacy.md item 1) predates it.
- **Minimum fix (either):** (a) In the record, drop the per-crate Rows, Modeled and Tests cells for the class P ranks, and the "N modeled" text in the pool table's Arm A column. Keep arm totals and per-crate dispositions without counts, such as "≥1 modeled; not caught" and the unportable reasons. Then re-check that no remaining cell differences against the ledger. Or (b) remove the per-rank counts from the ledger's published history before publication. That is a history rewrite on `main`, so it needs owner authority under AP-GIT-001. Option (a) is in the record's own scope. B1's leak and E1 below share a root, so (b) fixes both.

### E1 (escalation, outside the record; blocks publication, not this record's text): the committed ledger already exposes private reading-observation detail

- **Where.** fx-zu4f.12's run comments in `main:.beads/issues.jsonl`.
- **Evidence.** [Redacted by tide-lead before commit: what the earlier ledger comments expose is withheld; the escalation went to the owner.]
- **Minimum fix.** tide-lead raises this with the owner before the repository goes public. The options are to rewrite or redact those comments in the ledger's history, or to accept the exposure explicitly. Future run comments should give counts only, with no rank, classification or property identifier. This is not fixable in the record and is not an author finding against it.

## Non-blocking findings

1. **The correction notice leaves out the step-3 tension** (`:28-34`). It says the round-1 corrections "apply registered rules, not new rules". That is true, but applying §7 step 1 per claim meant listing claims after the first runs, against step 3's "no property added after a run". That is disclosed at `:63-64` and `:225`, but not where the notice and the Arm A result (`:111-116`) present the candidate. Add one clause to each saying the candidate's 2 properties were listed after the first runs, from rows already in the frozen list. This does not change the negative result.
2. **Say why the control fails** (`:113-114`, `:405-410`). The frozen property the control fails is the competing claim from the same doc-versus-own-test conflict, so no control can pass both by construction. Stating that, with no crate, keeps readers from taking "not caught" for either a near-catch or a faulty control. The disposition is not open to a fairer reading: §7 step 6 requires passing "every frozen property", and the record applies that as written.
3. [Redacted by tide-lead before commit: this passage described how private items relate to each other; it is withheld under the round's disclosure rule.] (tide-lead did not adopt this suggestion; see the re-check.)
4. **Fixed-input wording in Framing** (`:503-504`). "A property that passes says the model passes on 1,000 seeds" should add "(one repeated case for the 16 fixed-input tests)", to match `:46-48`.
5. **OTS wording** (`:58-59`). "existed at their attested times" should read "existed by", matching protocol §0.

## Reasoning

**1. Not caught.** In arm B, all 9 rows fail E1 on the full P10 §3 evidence. Codex re-check B1 verified the reading, and I accept it. The correction applied the registered evidence rule as written, not a new rule, and `:213` and `:224` disclose that it came after the first record and the Codex pass. In arm A, the class C core is unportable under P10 §6's location-first order. In class P, the private run notes confirm that the 2 round-1 tests fail every seed, that the lowest failing seed is 0, that two replays are byte-identical, and that the control fails a frozen modeled property (the crate's own test), so §7 step 6 is not met. The record says exactly that and claims no reproduction. Withdrawing the conflict exclusion applied §7 steps 1-4 as written, and the record says so. Its one cost, a post-run listing, is disclosed in the deviations table and the limitations (non-blocking 1). The arithmetic reconciles: per-crate Rows sum to 430; Modeled 47 = 31 generated + 16 fixed rows; Tests 45 = 29 + 16; 430 = 47 + 23 + 315 + 44 + 1; 12 reading observations across 6 REQUESTs (report.md tallies 1 + 3 + 4 + 4).

**2. The arm-A split.** The 29 generated-input and 16 fixed-input split appears in the limitations (`:43-48`), next to the first result (`:109-111`) and in the counts table (`:347`). The fixed-input tests are called "fixed checks, not seeded explorations" and are named as a §7 step 5 departure. "Seeded exploration" is never claimed for them. The candidate's "caught rule not met" matches §7 step 6's wording, and the record does not overstate it: model-only, no real-crate run, no `cargo fetch`.

**3. Privacy.** Read alone, the record names nothing private, and its deviation rows name no rank for round-1 work. The branch's commit message and other files carry nothing private. The leak is the combination with committed ledger text (B1), and the ledger also exposes material of its own (E1). No private file is committed on the branch.

**4. Cold reader.** Disclosure comes first, then limitations, then results. Class C, class P, E1-E5, "not assertable", "caught" and RC are defined where first used. The record is long because of the sources table, but each section can be audited on its own. Non-blocking 1-5 are clarity fixes.

DONE
