# fx-gcs.2 review, round 1 (Opus)

- Reviewer: Opus, round 1, report-only converge round (GOVERNANCE.md: runs in
  parallel with the Codex pass on the same frozen candidate; no file edited
  but this one)
- Candidate: worktree `tide-fx-gcs.2`, branch `fx-gcs.2`, `git diff e2a7063`
  plus intent-to-add files; fingerprint
  `cbd73fb9abe0a7b40cbf00fa6a747337358203aecfb1cc4cc0d2e5a842653271`, the same
  before and after this review
- Date: 2026-10-21

## Verdict

Changes needed: three small blocking findings, all in v0-tier text or cases
this change edits. Coverage is complete. Every ID in 02 §1–§8.2, plus
SHARED-1, has at least one accepting and one rejecting case, and the suite
exits 0. The meaning-level edits to 02 are correct, FX0205 and FX0206 are
justified, and the stub's `Task` fix is sound.

## Blocking

**B1. `drop_may_dangle.rs` is missing the `FX0901` mark that this diff's own
README rule requires.** `conformance/README.md:72-74` (added here) says "an
`unsafe impl` in a case is `FX0901`", and `send_unsafe_impl.rs` follows that
rule. `conformance/cases/memory/drop_may_dangle.rs:6` is an `unsafe impl`
marked only `E0658`. Since the runner does not enforce `FX` marks, the case
passes with an incomplete expected verdict, and the checker's first slice
will be graded against that verdict. Fix: mark the line `//~ E0658 FX0901`.
For consistency with `send_unsafe_impl.rs`, also add a `// SAFETY:` comment
so that `FX0902` stays out of the expected verdict.

**B2. BORROW-3 is mapped onto SCOPE-4, which contradicts SCOPE-4.**
`conformance/rules.md` (BORROW-3 row) lists "a passed-down scope, SCOPE-4"
as a place where `Owned` is required, and `scope_passed_accept.rs` and
`scope_passed_reject.rs` name BORROW-3. SCOPE-4 (`design/02-memory.md:709-715`)
says the opposite. A child on a passed-down scope may borrow what the
signature ties to the scope, and SCOPE-4 calls itself "the precise form" of
the `Owned` slogan. `scope_passed_accept.rs` itself accepts a non-`Owned`
capture (`cfg: &'s Config`). The underlying cause is BORROW-3's clause
"values handed to tasks that outlive their creator (§8.2)" (02:379, and the
same phrase at 02:512). No v0 task requires `Owned` on those grounds:
`spawn_isolated` waits for its task, and the passed-down children fall under
SCOPE-4. Fix:
- in 02:379, write "`spawn_isolated` closures (§8.2)" instead, and the same
  at 02:512;
- drop "a passed-down scope, SCOPE-4" from the rules.md row;
- drop BORROW-3 from the `//@ rules:` line of both `scope_passed_*` cases.

**B3. ORIGIN-6 contradicts CLOSURE-3, and `closure_fnptr_reject.rs` exposes
it.** ORIGIN-6 (02:291) says "v0 accepts a body exactly when Rust's NLL
borrow checker accepts it". `extend`'s body is accepted by NLL, but
CLOSURE-3/FX0206 rejects it. §11.2 R3 already treats the re-check as an
addition, and the bead's scope says to fix a small v0 contradiction that a
case exposes. Fix, in ORIGIN-6: "...accepts it (§11), apart from CLOSURE-3's
`fn`-pointer re-check."

## Non-blocking

- **N1. Duplicate case.** `loan_conflict_accept.rs:5-16` (`Lexer::rest`) is
  a LOAN-3 case: its comment describes a reborrow, and
  `loan_reborrow_accept.rs` already covers it. Delete it from the LOAN-1
  case.
- **N2. Lines that test nothing about their rule.**
  - `var_accept.rs` `same_cell` is an identity coercion.
  - `closure_hr_accept.rs` `count`/`big` returns no borrow, so it says
    nothing about CLOSURE-2.
  - `dyn_default_reject.rs` `visit`/`count` uses an explicit `+ 'static`,
    so it tests the stated bound, not DYN-1's default. Its other two lines
    do test the default.

  Delete all three.
- **N3. Weak rejecting cases.**
  - `life_const_reject.rs` would fail under any reading, since a static has
    no other lifetime in scope.
  - `send_table_accept.rs` names SEND-2, but the only acceptance SEND-2 can
    have is under a grant, which waits for manifests (README). Its
    user-type line tests SEND-1.
  - `loan_kill_reject.rs` shows no kill without an assignment; it does not
    show what an assignment leaves unkilled.

  All three are acceptable as the "suggested rewrite" pattern that LIFE-3
  and DROP-2 also use. For SEND-2, say "accepting case awaits manifests" in
  rules.md rather than claiming coverage.
- **N4. Elision rules 1 and 2 (02 §2.5) have no ID**, and no case covers
  rule 2's rejection, for example `fn get(&self, k: &str) -> &str { k }`.
  That case would catch a checker that unions the inputs' origins. Either
  move LIFE-4's lead to cover all of "output elision", or add a rule-2
  rejecting line to `life_elision_reject.rs`.
- **N5. VAR-1's "instantiating higher-ranked origins" is untested.** For
  example, `for<'a> fn(&'a u8)` <: `fn(&'static u8)` would cover it.
- **N6. FX0205 wording.** "which rustc accepts" (02:486-487, and the rules.md
  OPAQUE-2 row) holds only when the hidden type has the trait; otherwise
  rustc also gives E0277. Write "which rustc accepts when the hidden type is
  `Send`/`Sync`". `reference.md:307` could cite FX0205.
- **N7. ID order.** The new IDs do not follow document order: LOAN-3 comes
  before LOAN-1, LOAN-5 before LOAN-2, and LIFE-8 before LIFE-5. Nothing
  relies on them yet, so they can be renumbered for free now and not after
  merge.
- **N8. Stale line numbers.** rules.md's 02 line numbers are from `a3b0177`
  (as its header says), but this diff moves 02, for example VAR-3 to 02:412
  and OPAQUE-2 to 02:484. With IDs now in 02, the line column is redundant
  for those rows.
- **N9. Nits.**
  - The README line at 98 and about twenty 02 lines were not rewrapped after
    the leads were inserted.
  - The grammar comment "only on a return-position `impl Trait`" (02:169)
    now duplicates LIFE-2's first sentence.

## Stub additions

- **`Tasks::spawn_isolated`: needed.** The SCOPE-3 cases call it. Its
  signature matches 02:679 exactly.
- **`Panicked`: needed** as the return type. Its `Debug`, `Display`, and
  `Error` impls are unused by cases, but they match every other stub error
  type and reference 2.2, so keeping them is fine.
- **`Task`'s `*const ()` marker plus `unsafe impl Send`/`Sync` for
  `U, E: Send`: needed and sound.** The old `PhantomData<fn() -> (U, E)>`
  made every `Task` `Send` and `Sync` whatever `U` and `E` were, so
  `send_table_reject.rs:29-30` could not fail. The marker only removes auto
  traits. It does not change variance (`'s` invariant, `U` and `E`
  covariant), and the impls give exactly 02 §8.1's row, which matches std's
  `ScopedJoinHandle`. The impls are in std, so FX0901 does not apply.

## Checks run

- I checked the fingerprint before and after the review: `cbd73fb9…3271`
  both times.
- `conformance/run.sh </dev/null` (load 3.5 at start): exit 0, 80 PASS,
  11 AWAITS CHECKER, no FAIL. The suite includes the pre-existing
  authority/tasks cases, so the `Task` stub change broke nothing.
- `conformance/run.sh --trace`: each of the 46 new IDs (SHARED-1 included)
  traces to at least one `_accept` or unmarked case and one rejecting case.
- I grepped `design/` for the new IDs. None collides with a 03 or 04 prefix.
- FX0204 is retired (rounds 8 and 9), so FX0205 and FX0206 are the next
  unused 02xx codes, and nothing else uses them.
- No document in `design/` tells readers to annotate a closure, which the
  CLOSURE-2 edit says does not work.
- Every ID lead in 02 matches README's "bullet, list item, or sentence"
  placement and its prefix table's sections.

## Reasoning

The meaning-level edits hold.
- **CLOSURE-2.** Rust rejects `|t: &str| -> &str { t }`, and
  `closure_hr_reject.rs` confirms it, so the old fix ("annotate") was wrong
  and the new one is right.
- **SEND-2's "only".** It matches reference 11.
- **DROP-2's ADMIT-2 grounding.** It matches 03.
- **ORIGIN-2's reorder.** The meaning is unchanged.
- **LIFE-2.** It moved from the grammar into prose without change.

FX0205 is needed because rustc's auto-trait leakage accepts what OPAQUE-2
rejects. FX0206 is needed because rustc accepts the #25860 exploit. Neither
had a code, and a checker-only rejection needs one.

The blocking findings are each a one-line or few-line fix. B1 and B2 are
wrong expected verdicts or rule mappings in the corpus, the artifact the
checker will be graded against. B3 is a contradiction in edited v0 text that
a case in this change exposes, which the bead's scope says to fix.
