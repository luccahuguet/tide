# Tide review, fx-gcs.3 round 6 (Opus): converge round

Candidate: commit 688cbad on main 755d86e (worktree tide-fx-gcs.3). I
reviewed the whole change (`git diff 755d86e 688cbad`) with fresh eyes. The
review covered every authority case and `memory/send_unsafe_impl.rs`
against the full reference, not only the rule each one targets, plus the
design edits, rules.md, README, run.sh, the stub, and the new macros crate.
This round fixes what it finds. The fix is uncommitted in the worktree.

## Verdict

fixed

The rounds 1 to 5 fixes are in the commit and they hold up. I found no
defect in any verdict, rule text, inventory row, or the runner. The one
edit is a cheap formatting nit.

## Findings and fixes

**F1 (nit). GLOBAL-3 had a ragged wrap that this change introduced.**
`design/03-authority-concurrency.md:366-369`: adding "(`thread_local!` is
`FX0502`)" left the line "(`thread_local!` is `FX0502`). The" followed by
a full line. Round 1 fixed the same artifact in GLOBAL-1.
*Changed:* I rewrapped the bullet. The text is unchanged, and the bullet
still has the same number of lines.

## Not changed

- **03 §3.1's list of `Pure`-needing codes omits `FX0610`.** Reference 8
  and companion row 57 put a `join` closure that is not `Pure`, such as one
  that moves in a capability, under `FX0610`. 03 §8.1 gives `FX0610` only
  for the `Sync` part, so by 03 that failure falls to `FX0517`. The gap
  predates this change: 01 D18's list never had `FX0610`. rules.md's
  `none yet` row at 03:1425 already records "ref:785 places `join` under
  FX0610". The PAR rules belong to the tasks bead (fx-gcs.5), so that bead
  should settle it. I report it as out of scope.
- **`root_forge.rs` says "a struct literal" for `Fs(())`, which is a
  tuple-struct constructor call.** ROOT-4 uses the same loose wording ("no
  struct literal ... builds one"), and the verdict (E0423) is right.
- **`root_sim_outside_test.rs:5` reads `sim.fs` from an inferred
  `SimRoot`.** ROOT-3 lists `SimRoot` among items that compile only in
  tests. The line uses a field, not a path to the type. Line 4 already
  carries the `FX0504`, and a checker that resolves paths reports only that
  line.
- **`FX0408` (two inherent methods with one name).** In
  `afree_const_generic.rs` the two impl blocks define `len` and `is_empty`,
  so no name appears twice.
- **`impl<T: Send> Ship for T` and `impl<T: Pure> Describe for T` against
  `FX0406`.** Reference 5.1's fix for that code is "bound `T` directly",
  which these impls do. The design's own companion-trait idiom has the same
  shape.
- The earlier rounds' open items still stand, all out of scope or at the
  core's budget: `asm!` is `FX0103` while RUN-2 uses it under a grant;
  reference 6.2 says "only in `main`'s signature"; reference 11's
  `std::sync::raw` row gives no code; 02 §9 cites ADMIT-1 without a
  chapter; the spike's `cpuid` case was not ported; and
  `reviews/fx-gcs.3-r1-codex.md` keeps Markdown hard breaks.

## Checked

- **Rule text:** AGENTS.md's repository-local rules, GOVERNANCE.md's Merge
  Rule, the bead (`br --no-db show fx-gcs.3`), conformance/README.md, and
  the reviews from rounds 1 to 5.
- **Every authority case against the full reference:**
  - the macro list (2.1: `write!`, `format!`, `vec!`, `dbg!`, `println!`,
    granted `global_asm!`) and the attributes (2.3: `#[pure]`,
    `#[trusted_pure]`, `#[tide::main]`, `#[unsafe(no_mangle)]`);
  - elision in consts (3.1, LIFE-5) and float methods (4.3: `sqrt` and
    `abs` stay; `sin` and `to_bits` are `FX0520`);
  - the power ceiling (5.1: `FX0402`, `FX0406`, `FX0408`) and impl
    placement (`FX0401`);
  - ambient calls and the allowlist (6.9, ADMIT-1), `Pure` and
    authority-freedom (6.6, 02 §9, 03 §3.1 and §3.2), `Secret` (6.5), and
    holes (11, including `// SAFETY:` on each granted block and
    `unsafe impl`).
- **Warnings:** I compiled every authority case and `send_unsafe_impl.rs`
  in both modes. The only warnings are dead-code ones: no unused import
  (`FX0913`) and no unused `Result` (`FX0702`).
- **Design consistency:** I grepped design/ for FX0502, FX0503, FX0517,
  FX0518, FX0610, `const fn`, SEND-2, and every new ID. Each of CAP-1..3,
  UNSAFE-1, AFREE-1..2, and PURE-1..2 is defined once with a bold lead, and
  every restatement cites its owner. The only bold IDs in 02–04 missing
  from rules.md are the v1 FFI and RUST rules.
- **README and run.sh agree** on the `//@` directives, `-Zallow-features=`
  under `RUSTC_BOOTSTRAP`, bin detection, and the exit codes.
- **Coverage:** `run.sh --trace` gives an accepting case and a rejecting
  case for every in-scope ID: CAP-1..3, ROOT-1..5, GLOBAL-1..4 and 6,
  SINK-8 and 10, ADMIT-1 and 2, UNSAFE-1, AFREE-1 and 2, LOCK-1, 4, and 5,
  and PURE-1 and 2.
- **Runs:**
  - On 688cbad (load 5.5 on 16 cores), `conformance/run.sh` exited 0 with
    107 PASS, 56 AWAITS CHECKER, and no FAIL. `--trace` exited 0.
  - After the fix (load 4.2), `run.sh` exited 0 with the same 107 PASS, 56
    AWAITS CHECKER, and no FAIL. `--trace` exited 0.
  - `git diff --check` is clean for this round's edit. The commit's only
    whitespace hits are the Codex file's hard breaks. No added design or
    README prose line is over 80 characters.
- **Budget:** `scripts/count-tokens.sh design/reference.md` gives 24,992,
  and this round does not edit the core. The companion's count line
  matches. rules.md and its snapshot line (`a3b0177`) are untouched.
