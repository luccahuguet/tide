# Tide review, fx-gcs.3 round 2 (Opus): converge round

Candidate: commit ebcca44 on main e2a7063 (worktree tide-fx-gcs.3). I
reviewed the whole change (`git diff e2a7063 ebcca44`), and looked hardest
at the round-1 fixes (`git diff 19bc1a2 ebcca44`). This round fixes what it
finds. The fixes are uncommitted in the worktree.

## Verdict

fixed

The round-1 fixes hold up: the FX0518 lists, GLOBAL-1 against reference 6.4,
`root_forge` forging `Fs`, `admit_waker`, AFREE-2's definition-site
sentence with its case, reference 2.3, `root_zeroed`, CAP-3's move to
FX0520, the `//@` validation, and `pure_accept`'s casts. The UNSAFE-1
decision (an `unsafe impl` of any trait needs the grant, declaring an
`unsafe trait` needs none) is stated in 03 and reference 11, and it has
cases on both sides. I found four cases whose verdicts would break once FX
marks are enforced, and one ROOT-1 wording that the new explicit `FX0503`
turned into a contradiction. All are fixed below.

## Findings and fixes

**F1. ROOT-1's explicit `FX0503` rejects `Root::sim` in tests.**
`design/03-authority-concurrency.md:324-329`. Round 1 added "(`FX0503`)"
to "`Root` may be named only as the parameter of the binary's `main`". Read
literally, `Root::sim(7)` in a test names `Root`, so it is `FX0503`. That
breaks two cases. The accepting `root_sim_in_test.rs:15` would get an
unmarked `FX0503`. `root_sim_outside_test.rs:4`, which expects `FX0504`
only, would get both codes. ROOT-3 and §9.2 clearly intend `Root::sim` in
test code.
*Changed:* ROOT-1 now says `Root` may be named only as `main`'s parameter
"and in the path `Root::sim`, which ROOT-3 confines to test code; any other
use is `FX0503`". This keeps `Root::sim` outside tests at `FX0504` alone.
The `conformance/rules.md` ROOT-1 row now says the same.

**F2. `admit_io_error_user_from.rs:7` would also be rejected by `FX0401`.**
`impl From<Key> for Box<dyn Error + Send + Sync>` lives in neither the
module of its type (`Box`, alloc) nor that of its trait (`From`, core).
D4 and reference 5.2 reject that impl placement (`FX0401`), so the case
would fail on an unmarked line, for an unrelated reason.
*Changed:* the case now uses a user `Into` impl,
`impl Into<Box<dyn Error + Send + Sync>> for Key`, which sits in `Key`'s
module. ADMIT-1 names user `Into` impls too ("including a user `From` or
`Into` impl"). `io::Error::other(k)` still reaches no alloc `From` impl, so
it is still `FX0517`, and rustc still accepts it.

**F3. `global_thread_local.rs:3` imported `Cell`, and the import was
unused.** `thread_local!` does not resolve, so its body is never expanded,
and rustc warned "unused import: `std::cell::Cell`". Reference 12.2 makes
unused imports errors (`FX0913`), so a checker would report an unmarked
line. I compiled every authority case in both modes; this was the only
unused-import warning.
*Changed:* I dropped the `use` and wrote `std::cell::Cell` inside the
macro.

**F4. `unsafe_impl_pure.rs:9` was missing `FX0402`.** `unsafe impl Pure for
Tag {}` overlaps std's blanket `Pure` impl. D4 ("No overlapping impls")
and reference 5.1 give overlapping impls `FX0402`, and `pure_overlap.rs`
marks rustc's overlaps `E0119 FX0402`. With FX marks enforced, this case
would fail.
*Changed:* the line is now marked `//~ E0119 FX0402`, and the comment says
"overlaps std's blanket impl (D4)". In UNSAFE-1, `03:672-673` now says
"overlaps std's blanket impl (E0119, `FX0402`; ...)" and is rewrapped, which
also fixes an over-long line. The case moves from PASS to AWAITS CHECKER.

**F5. UNSAFE-1 names `unsafe impl Send` and `Sync`, but no case covers
them.** The new text says "`unsafe impl` of any trait (`Send` and `Sync`,
...)". The only rejecting `unsafe impl` was for a user trait.
*Changed:* `unsafe_ungranted.rs` adds
`pub struct Token(std::cell::Cell<u8>); unsafe impl Sync for Token {} //~ FX0901`,
and its comment now mentions it. rustc accepts the impl.

**F6 (nit). `root_zeroed.rs:3` was 101 characters long.** It was the only
case line over 100. *Changed:* the comment is wrapped.

## Not changed

- **Reference 6.2 (`reference.md:373`) and 01 D7 (`01:417`) still say
  `Root` "can be named only in `main`('s signature)".** The next reference
  bullet and 6.8 put `Root::sim` in tests, so the exception reads clearly,
  and the core is at its 24,992-token ceiling. The fix belongs in 03,
  which owns the rule.
- **rules.md:54, the 02 §8.1 row (`unsafe impl Send`/`Sync`, `none yet`),
  is a restatement of UNSAFE-1.** It could become `= UNSAFE-1`, and 02:623
  could cite UNSAFE-1. Both are fx-gcs.2's to change (02 §8.1, and a row
  outside this bead), so I left them. This is the same as round-1 N4; the
  other session should be told.
- **Reference 11's `std::sync::raw` row gives no code**, while the
  raw-pointer row gives "(`FX0520` without it)". This is consistent but
  incomplete, and adding it would push the core over budget. 03 CAP-3 and
  ADMIT-1 state the code.
- **`global_asm!` under the grant against reference 2.1's "any other `!`
  form is `FX0103`".** This predates this change. 03 ADMIT-1 and RUN-2 treat
  `global_asm!` as admitted under the grant, so `unsafe_granted.rs:22`
  follows 03. Reference 2.1's macro list could name it as an exception;
  that edit is outside this bead.
- **`root_captured.rs:6` marks the capture at the use inside the closure,
  not at the closure expression.** No text fixes the line, and rustc's
  capture diagnostics point at the use, so I kept it.
- **ADMIT-1 still lists "a user `From` ... impl".** Under `FX0401`, such an
  impl into `Box<dyn Error ...>` can never be placed legally, so that part
  of the clause is redundant but harmless.

## Checked

- **Rule text:** GOVERNANCE.md Merge Rule, the bead, conformance/README.md,
  and both round-1 reviews. I checked each round-1 fix against its finding.
- **Codes:** I grepped design/, rules.md, and the stub for FX0502, FX0503,
  FX0517, FX0518, FX0520, FX0901, FX0402, `unsafe impl`, `unsafe trait`,
  `sync::raw`, and `zeroed`, and checked the restatements in 01 D7, D18,
  02 §8.1, §8.4, §9, 03 §1.2, §2.1, §2.2, §2.5, RUN-4, §3.1, §3.5,
  reference 2.3, 6.2, 6.4, 11, and the companion. I found no contradiction
  beyond F1 and F4.
- **02 edits:** they stay out of §1–§8.2 and the `Shared::with` escape
  bullet. The edits in §8.4 are to the `Shared<T>` and closure bullets.
- **Accept and reject coverage:** for every in-scope ID (CAP-1..3,
  ROOT-1..5, GLOBAL-1..4/6, SINK-8/10, ADMIT-1/2, UNSAFE-1, LOCK-1/4/5,
  PURE-1/2, AFREE-1/2), `run.sh --trace` lists at least one case with no
  marks and one with marks. The IDs with no cases are PAR-3, CLAUSE-*, and
  SEAL-2/3, all other beads'.
- **Wrong-reason audit:** casts against reference 4.2 (`u8 as u32` in
  `pure_deep` is a widening, so it is allowed), impl placement (F2),
  unused imports and unused `Result`s (I compiled all authority cases in
  both modes; F3 is the only hit), the macro list, and `// SAFETY:`
  comments on every granted `unsafe` block.
- **run.sh:** the new `//@` check rejects unknown or misplaced directives on
  line 2 and later, while line 1 must be the rules line. Its rejection
  message is reported. `bash` quoting and the awk regex are fine.
- **Runs:** `conformance/run.sh` on ebcca44 (load about 16 on 16 cores):
  exit 0, 32 PASS, 45 AWAITS CHECKER. After my fixes: exit 0, 31 PASS,
  46 AWAITS CHECKER, with no FAIL (`unsafe_impl_pure` moved from PASS to
  AWAITS). `git diff --check` is clean.
- **Budgets:** `scripts/count-tokens.sh design/reference.md` gives 24,992,
  and the reference is unchanged. The rules.md snapshot stays `a3b0177`.
