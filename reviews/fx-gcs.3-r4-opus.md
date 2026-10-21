# Tide review, fx-gcs.3 round 4 (Opus): converge round after the rebase onto 755d86e

Candidate: commit c39ea60 on main 755d86e (worktree tide-fx-gcs.3). I
reviewed the whole change (`git diff 755d86e c39ea60`). I looked hardest at
the two conflict resolutions (README's ID paragraph, 02 §8.4) and at the
edits agreed with the fx-gcs.2 session: the SEND-2 retirement, the `= DYN-2`
row, and the granted `unsafe impl Sync`. This round fixes what it finds. The
fixes are uncommitted in the worktree.

## Verdict

fixed

Both conflict resolutions are right. 02 §8.4 keeps main's SHARED-1 bullet
word for word, and this change's citations sit in the bullets around it
(LOCK-1 on `Shared<T>`, "03 LOCK-4" on the closure). README keeps main's
ID-format paragraph. lib.rs merged cleanly: this change adds `Arc`, `Weak`,
`sync::raw`, `Secret`, and `Root`'s private field, and fx-gcs.2 adds
`spawn_isolated`, `Panicked`, and the conditional `Task` auto traits. None
of these touch each other, and the memory cases still pass against the
merged stub. rules.md has no duplicated or contradicting rows. Each `= ID`
row names a different restatement site. No case's rules line names a
retired ID: run.sh would fail one, since SEND-2 is no longer in the first
column. I found one contradiction that the merge introduced, one
restatement that does not cite its owner, and one fragile accepting case.

## Findings and fixes

**F1. README contradicted its own `//@ grants: unsafe` directive.** Main's
case-format bullet said "A case has no manifest, so its package holds no
grants: ... a case accepted under a grant waits for manifests." This
change adds `//@ grants: unsafe` a few bullets earlier, along with
accepting cases under the grant (`unsafe_granted.rs`,
`cap_sync_raw_granted.rs`, `global_pointer_granted.rs`,
`unsafe_trusted_pure_granted.rs`). After the merge, the README said both
things. The directive bullet already says the grant is one "which it
otherwise lacks", so the old bullet has nothing left to add.
*Changed:* I deleted the bullet. I also reworded `memory/send_unsafe_impl.rs`'s
comment from "A case has no manifest, so its package holds no `unsafe`
grant" to "Without `//@ grants: unsafe`, a case's package has no `unsafe`
grant".

**F2. 03 §3.1's `Pure` trait-object paragraph did not cite DYN-2.** The
agreed edit made rules.md's 03:762 row `= DYN-2`. README says "A rule
another chapter restates keeps its owner's ID, and the restatement cites
it." Every other `= ID` row this change adds has a citation in its text
(02 §8.1 and §9 cite 03 UNSAFE-1, 02 §8.4 cites LOCK-1 and LOCK-4, 02 §9's
table cites AFREE-2, 03 §3.1 cites PURE-1, PURE-2, and UNSAFE-1). This
paragraph had none.
*Changed:* 03 §3.1 now says "uses a companion supertrait with a blanket
impl (`02-memory.md` DYN-2):".

**F3. The granted `unsafe impl` lines had no `// SAFETY:` comment.**
Reference 11 requires the comment on "each block" (`FX0902`, provisional
error). Rust's `undocumented_unsafe_blocks` lint, the model for that rule,
also covers `unsafe impl`. fx-gcs.2's `send_unsafe_impl.rs` adds the comment
"so the missing grant is the only error". The newly added accepting
`unsafe impl Sync for Token` in `unsafe_granted.rs` had none, and neither did
`unsafe impl Zeroable for u32`. A checker that reads `FX0902` the way the
lint does would reject that accepting case.
*Changed:* both `unsafe impl` lines in `unsafe_granted.rs` and in
`unsafe_ungranted.rs` now carry a `// SAFETY:` comment. In the ungranted
case, that keeps the missing grant as the only error. The granted case's
header comment now says "on each block and `unsafe impl`".

**F4 (nit). README did not say where a retired ID is recorded.** 02 §8.1's
"(`SEND-2` is retired.)" is the first such record. Without a stated
convention it looks ad hoc, and the next removal might go unrecorded.
*Changed:* README's ID paragraph now reads "a removed rule's ID stays
retired, noted where the rule stood, and a new rule takes the next number".

## Not changed

- **Whether "(`SEND-2` is retired.)" belongs in 02.** I kept it. SEND-2
  reached main in 755d86e and cannot be renumbered (README). SEND-1 and
  SEND-3 now leave a gap at the spot where an editor would assign the next
  SEND ID. The note is four words at that spot, and it stops the ID from
  being reused. rules.md is a snapshot at `a3b0177`, so it is not where the
  live record goes. F4 makes this the stated convention.
- **`memory/send_unsafe_impl.rs` and `unsafe_ungranted.rs` both reject an
  ungranted `unsafe impl Sync`.** The overlap is small. The memory case is
  the agreed retarget of fx-gcs.2's file, and it also covers `Send`. The
  ungranted line mirrors the granted one in `unsafe_granted.rs`.
- **02 §9 cites 03's ADMIT-1 without the chapter ("(ADMIT-1)", "items
  ADMIT-1 admits").** That text predates this change.
- **`reviews/fx-gcs.3-r1-codex.md` has trailing whitespace.** These are
  Markdown hard breaks in recorded Codex output, and other `*-codex.md`
  files on main have them too.
- **Rounds 2 and 3 open items** still stand: `asm!` is `FX0103` while RUN-2's
  test row uses it under a grant, reference 6.2's "only in `main`'s
  signature", and reference 11's `std::sync::raw` row gives no code. All are
  out of scope, and the core is at its budget.
- rules.md's snapshot line and its `file:line` references stay as they are
  on the branch.

## Checked

- **Rule text:** AGENTS.md's local rules, GOVERNANCE.md's Merge Rule, the
  bead, conformance/README.md, and the reviews from rounds 1 through 3.
- **Shared items:** rules.md's ID columns (`= ID` rows are skipped by
  run.sh's ID regex, so nothing is counted twice), lib.rs, README, and 02
  §8.1, §8.4, and §9 against main 755d86e. I grepped for SEND-2 outside
  `reviews/`. Its only mention is 02 §8.1's note.
- **Coverage:** `run.sh --trace` gives an accepting case and a rejecting
  case for every in-scope ID: CAP-1..3, ROOT-1..5, GLOBAL-1..4 and 6,
  SINK-8 and 10, ADMIT-1 and 2, UNSAFE-1, AFREE-1 and 2, LOCK-1, 4, and 5,
  and PURE-1 and 2. PAR-4 has only a rejecting case. It belongs to the
  tasks bead.
- **Runs** (load about 6, then 1.4, on 16 cores):
  - On c39ea60, `conformance/run.sh` exited 0: 107 PASS, 56 AWAITS
    CHECKER, no FAIL. `--trace` exited 0.
  - After the fixes, `run.sh` exited 0 with the same 107 PASS, 56 AWAITS
    CHECKER, and no FAIL. `--trace` exited 0 with output identical to
    before.
- **Budget:** `scripts/count-tokens.sh design/reference.md` gives 24,992
  (unchanged; this round does not edit the core). The companion's count line
  matches.
- `git diff --check` is clean for this round's edits. No authority case line
  is over 100 characters.
