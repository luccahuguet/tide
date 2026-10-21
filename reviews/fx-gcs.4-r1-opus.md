# fx-gcs.4 review, round 1 (Opus)

Reviewer: Opus, round 1, report-only converge round (GOVERNANCE.md; run in
parallel with the Codex pass on the same frozen candidate).
Candidate: worktree `tide-fx-gcs.4`, branch `fx-gcs.4`, `git diff 755d86e`
with new files intent-to-added; fingerprint
`7fc93014466bfacfea6d9ceff528debfb51b3a0b767fdcdd5748295cbc7e438e`.
Date: 2026-10-21.

## Verdict

Changes needed (one blocking finding, small).

## Blocking

**B1. FORM-1's accepting case does not test the rule's boundary.**
`conformance/cases/contracts/form_comma_accept.rs:2-3` says "a comma that is
not at the top level, as in a call's arguments, is allowed", but no clause in
the file contains a comma (`a.max(b)` at line 6 and `i.min(n)` at line 16 take
one argument each). The case only shows that clauses without commas are
accepted, so a checker that rejected every comma in a clause would pass it.
The rejecting case has the top-level comma, but nothing pins the accept side
of "top-level". Fix: give one clause a comma inside a call, for example
`#[ensures(result >= u32::max(a, b))]`. I checked this against the D18 lowering
with a scratch case: `u32::max(a, b)` and `core::cmp::max(a, b)` in
`requires` and `ensures` both pass rustc. This blocks because the fix changes
a v0 case that the change adds.

## Non-blocking

**N1. FORM-2's fix wording** (`design/04-contracts.md:187-189`). "Either error
is `FX0805`, and the safe fix renames the parameter" puts the rename fix on
both errors. The first error, `result` outside an `ensures` (`grow` and `walk`
in `form_result_reject.rs`), has no parameter to rename. Suggested wording:
"either is `FX0805`; for the parameter, the safe fix renames it."

**N2. FORM-2's `check` verdict is derived, not stated.**
`form_result_reject.rs:10` marks `#[requires(result > 0)]` on
`fn check(result: u32)` (no `ensures`) as `FX0805`. That follows from §1.1:
inside a clause argument, `result` is the spec word. But CLAUSE-2 says "names
resolve as in the body", and the accepting case allows the parameter itself,
so a reader could expect the clause to name the parameter. One sentence in the
case comment, or in FORM-2, would remove the doubt.

**N3. Wrong count in `clause_param_reject.rs:5-6`.** "rustc accepts the D18
spike's lowering of the first two, and rejects the moved one". rustc accepts
the first three (`copy_in`, `spend`, `reset`) and rejects only `burn`.

**N4. FX0809's boundary has no accepting case.** No case has a
`Clone + Pure + Sync`, non-`Copy` parameter that the body mutates and the
`ensures` names, which CLAUSE-4 accepts by cloning at entry. Example:
`#[ensures(result == v.len() + 1)] fn grow(mut v: Vec<u32>) -> usize { v.push(1); v.len() }`.
Without it, a checker that rejected every mutated non-`Copy` parameter would
pass. rustc accepts the lowering (scratch probe).

**N5. CLAUSE-1's v1-forms half has no rejecting case.** The rejecting cases
cover only unbounded quantifiers. A v1 form used in v0 has none, such as
`#[decreases(n)]`, which the stub leaves as an unresolved attribute (an uncoded
rustc error). One line would do.

**N6. CLAUSE-7's spike list (`04:494-499`) omits the signed binder.** The
spike assumes `lo = 0` for a signed binder with no lower bound, so it accepts
and checks a narrower range than written, where the design gives `FX0807`.
The case comment (`clause_forms_reject.rs:2-3`) records this. The README's new
pointer ("04 §1.8, CLAUSE-7, lists where it differs") suggests that list is
complete. Add the signed binder to the list, or limit the README's claim to
runtime gaps.

**N7. The CLAUSE-4 row in `rules.md` omits E0382.** It lists "E0425, E0599,
and E0500", but `clause_param_reject.rs:27` also expects E0382.

**N8. SEAL-2/3 are deferred only in a `rules.md` note.** The note is
correct: a one-file case cannot carry `tide.seal`. But fx-gcs's acceptance
requires cases for every v0 rule, so record the gap on fx-gcs or file a bead,
so the epic does not close past it (AP-PLAN-001).

**N9 (nit).** Rule 10, which this change rewrote (`04:214-221`), still lists
`#[decreases]`, `#[diverges]`, and `#[contracts]` among the v0 attributes that
satisfy FORM-4. That is the open "Tier unclear" item in `rules.md:191`. It
could be settled here, but nothing requires it.

## Checks run

- Fingerprint before the review: matches
  `7fc93014466bfacfea6d9ceff528debfb51b3a0b767fdcdd5748295cbc7e438e`.
- `/proc/loadavg` 3.15, then `conformance/run.sh </dev/null` once:
  exit 0. All 20 contract cases are PASS or AWAITS CHECKER: FX0802, FX0803,
  FX0804, FX0805, FX0806, FX0807, FX0808, FX0809.
- `conformance/run.sh --trace`: FORM-1..4, CLAUSE-1, 2, 3, 4, and 6 each have
  an accepting and a rejecting case. SEAL-2/3 have none, by the note.
- Scratch probe (a file outside the repo, run with an absolute path): nested
  commas in `requires`/`ensures` pass; a mutated `Vec` parameter in `ensures`
  passes; an `invariant!` in a `loop` gives the spike's uncoded
  `compile_error!`, which confirms CLAUSE-7's new item.
- `grep -rE 'FX080[4-9]'` across `design/ conformance/ trials/ reviews/
  research/ spikes/ archive/` and `git log --all -S` for FX0804 and FX0806:
  the only uses are this diff's. FX0801 (retired) and FX0810–0812 (draft-1
  history in `reviews/round2-simplicity.md`) are untouched.
- Read `spikes/d18/macros/src/lib.rs` to confirm each lowering claim: E0600
  from `!(tuple)`; E0500 from the body closure capturing `v` while a snapshot
  borrows it; E0425 for the bindings, since the `for` check comes before
  `let Some(pat) = next`; `loop` is not rewritten; `lo` defaults to 0;
  `ItemFn` parsing fails on a bodiless trait method.

## Reasoning

- **IDs.** FORM-1..4 use bold leads in the CLAUSE style, numbered within the
  prefix as the README requires. Rules 2, 4, 5, 7, 8, and 9 of §1.2 get no ID
  because they reject nothing at compile time, are v1, or are restated by
  CLAUSE-4 (rule 4 already cites it). The 04 citations in `rules.md` are
  1–3 lines stale after this edit, but `rules.md` pins them to `a3b0177`, as
  the 02 rows do, so I raise nothing there.
- **rustc marks.** E0600 and E0500 record the spike's lowering, not the
  design's. The README and the case comments say so, and they don't change any
  verdict: the design rejects both programs too (FX0804, and CLAUSE-4's
  borrow-checked snapshot). They are acceptable as long as the README keeps
  saying that a contract case's rustc marks are the spike's. The uncoded
  `error` marks for FORM-4 hold under any lowering, because the design does
  not export `invariant!`.
- **New codes.** FX0804–FX0809 each name a rejection that 04 already
  required with no code (rules 1, 3, 10; CLAUSE-1, 3, 4). They are justified,
  because `FX` marks need codes. They are in the contracts range and collide
  with nothing.
- **04 text.** Rule 10's conditional safe fix is right: adding
  `#[invariants]` does not fix a misplaced clause in a function that already
  has one, and 04:33 still points at rule 10. CLAUSE-7's `loop` item matches
  the spike. CLAUSE-6 as rustc-decided E0425 follows from the head coming
  before `next`.
- **Deferrals.** `prove`, CLAUSE-5/7/8, and the test-driver schedule stay in
  the excluded list, which is correct. On SEAL-2/3, see N8.
- **Contradictions.** None in `design/`. D10 (`01:618`) and `reference.md`
  §10.1 call these "an error" with no code, which is consistent.
- **Needless cases.** `form_loop_accept`'s `sum` repeats the first loop of
  `clause_loop_names_accept`. Merging them is optional, since each rule keeps
  its own pair, so I don't count it as a finding.
