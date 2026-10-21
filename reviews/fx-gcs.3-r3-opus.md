# Tide review, fx-gcs.3 round 3 (Opus): converge round

Candidate: commit 1b14a18 on main e2a7063 (worktree tide-fx-gcs.3). I
reviewed the whole change (`git diff e2a7063 1b14a18`) and looked hardest at
round 2's edits (`git diff ebcca44 1b14a18`). This round fixes what it
finds. The fixes are uncommitted in the worktree.

## Verdict

fixed

Round 2's edits hold up. ROOT-1's `Root::sim` allowance clears the
`FX0503` that `root_sim_in_test` and `root_sim_outside_test` would have
drawn. The `FX0402` mark on `unsafe impl Pure` matches D4 and reference 5.1.
The `Into`-based io::Error case sits in `Key`'s module, so it draws no
`FX0401`. D4 forbids disjointness only by a missing auto or marker trait,
so core's blanket `Into` does not make it an `FX0402` overlap, and 02 §9
names a user `Into` impl as `FX0517`. The `thread_local!` case has no unused
import. `unsafe impl Sync for Token` is placed in its type's module and
draws only `FX0901`. I found one design contradiction that a case's verdict
depends on, one incomplete inventory row, and two small gaps.

## Findings and fixes

**F1. Reference 2.1 makes `global_asm!` `FX0103`, against 03 and two
cases.** `design/reference.md:61-64` says "Std supplies exactly these" and
"Any other `!` form is `FX0103`". `global_asm!` is not on the list.
03 ADMIT-1, UNSAFE-1, and RUN-4's test row make `global_asm!` `FX0520`
without the grant and admit it under the grant. 01 D7 agrees. So, read
against the core reference,
`unsafe_global_asm.rs:4` (marked `FX0520` only) would also draw `FX0103`,
and `unsafe_granted.rs:22` (accepting) would draw `FX0103`. This change
rewrote UNSAFE-1 and added both cases, so their verdicts force the fix.
*Changed:* reference 2.1 now reads "`include_str!`, `env!`, `option_env!`,
and ungranted `global_asm!` are `FX0520` (6.9)". To stay within budget,
reference 11's `#[trusted_pure]` row now says "with a reason, not
`unsafe impl Pure`" instead of "(`unsafe impl Pure` conflicts with std's
blanket impl)". 03 UNSAFE-1 still gives the reason and the codes. The core
measures 24,992 tokens, the same as before, so the companion's count line
stays correct.

**F2. rules.md's UNSAFE-1 row left out round 2's `unsafe impl Pure`
clause.** The row listed only `FX0901` and `global_asm!` `FX0520`, but
`unsafe_impl_pure.rs` names only UNSAFE-1 and expects `E0119 FX0402`.
*Changed:* the row now adds "no grant admits `unsafe impl Pure`", with the
codes "`unsafe impl Pure` E0119 / FX0402". Decided by is now `both`. The
snapshot stays `a3b0177`.

**F3. ROOT-1's new `Root::sim` allowance had no case that names ROOT-1.**
`run.sh --trace` showed no ROOT-1 case that exercises the path.
`root_sim_in_test.rs` accepts `Root::sim` in a test. `root_sim_outside_test.rs`
shows the path is `FX0504` alone, not `FX0503`. Both verdicts depend on
ROOT-1's wording.
*Changed:* both cases now list ROOT-1 on their rules line, and their
comments say ROOT-1 allows the path.

**F4 (nit). 02 §8.4 cited "(LOCK-4)" without a chapter.** Everywhere else,
02 cites 03's IDs as "03 X-N" or "`03-authority-concurrency.md` X-N".
*Changed:* the citation is now "(03 LOCK-4)", in the closure bullet. The
`Shared::with` escape bullet is untouched.

## Not changed

- **`asm!` is still `FX0103` by reference 2.1.** 03 RUN-2's test row uses
  it under an `unsafe` grant, and ADMIT-1 does not exclude it. No case
  covers it, this change does not touch it, and the core is at its
  ceiling. This is an out-of-scope finding to report.
- **`global_lazy_lock.rs` imports `LazyLock` and `OnceLock` and uses
  neither.** rustc does not report an unresolved import as unused, and
  the driver replaces E0432 with `FX0520`. So a checker that maps rustc's
  `unused_imports` to `FX0913` adds nothing here. Using the names would add
  a second `FX0520` at each use. The import is also the form agents write.
- **Reference 6.2 still says `Root` "can be named only in `main`'s
  signature".** The next bullet puts `Root::sim` in test code, and the core
  is at budget, as round 2 noted. 03 owns the rule.
- **`(x as *const u8)` in `global_pointer_*.rs`.** Reference 4.2's allowed
  list covers numeric casts only, but it rejects only lossy `as`
  (`FX0301`). Reference 11 treats pointer casts as raw-pointer spellings,
  admitted under the grant. No rule rejects a reference-to-pointer cast.
- **Reference 6.9 lists "hardware intrinsics (`core::arch`)" with no grant
  qualifier.** `global_asm!` lives under `core::arch` but is a macro, not
  an intrinsic. 03 ADMIT-1 is precise. This is minor and predates the
  change.
- Round 2's other open items still stand: the 02 §8.1 `unsafe impl
  Send`/`Sync` row belongs to fx-gcs.2, and reference 11's
  `std::sync::raw` row gives no code.

## Checked

- **Rule text:** AGENTS.md's local rules, GOVERNANCE.md's Merge Rule, the
  bead, conformance/README.md, and the reviews from rounds 1 and 2.
- **Every authority case against the full reference, not only its target
  rule.** This covered impl placement (`FX0401`), overlap (`FX0402`),
  lossy `as` (`FX0301`), the macro list (`FX0103`, which gave F1), unused
  imports (`FX0913`), `// SAFETY:` comments, `print!` in libraries
  (`FX0508`), `fmt::Arguments` and `Formatter` purity, and 02 §9's
  io::Error conversions.
- **Codes:** I grepped design/ for FX0402, FX0503, FX0504, FX0512, FX0517,
  FX0518, FX0520, FX0901, `global_asm`, `blanket`, `unsafe impl`, `trait
  Pure`, `Root::`, and `const fn`. The stub's `pub unsafe trait Pure`
  matches research/15, so `unsafe impl Pure` draws E0119 and no E0199.
- **Coverage:** `run.sh --trace` lists accepting and rejecting cases for
  every in-scope ID (CAP-1..3, ROOT-1..5, GLOBAL-1..4/6, SINK-8/10,
  ADMIT-1/2, UNSAFE-1, LOCK-1/4/5, PURE-1/2, AFREE-1/2).
- **run.sh:** the `//@` check, mark parsing, the `RUSTC_BOOTSTRAP` scope,
  and bin detection are fine. I found no bug.
- **Runs:** `conformance/run.sh` on 1b14a18 (load about 12 on 16 cores)
  exited 0 with 31 PASS and 46 AWAITS CHECKER. After the fixes (load
  about 11) it exited 0 with 31 PASS, 46 AWAITS CHECKER, and no FAIL.
  `git diff --check` is clean, and no case line is over 100 characters.
- **Budget:** `scripts/count-tokens.sh design/reference.md` gave 24,992
  before and after the fixes. An intermediate wording measured 24,995 and
  was trimmed.
