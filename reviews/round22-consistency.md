# Tide consistency review, round 22: confirmation of the round-21 answers

## Verdict

**changes requested**

Reviewed the uncommitted working tree on HEAD `5d96a2e` ("Refresh open beads
for D18: fx-arx, fx-1am, fx-qzp") on 2026-10-20. The change is
`git diff -- design/ README.md` (01 D2, 02 §10.9, reference 3.6 and 7.1,
the companion's header and 14.1, and the README's trials list), together
with §2.2, Q8 in §3.0, and §6 of the untracked
`trials/e1-dialect/grading.md`. Scope, as asked:

- whether each round-21 item (B1, B2, N1–N3) is closed as its Resolution
  claims;
- whether the changed lines contradict 01, 02, 03, 04, the README, or the
  rest of the reference pair;
- the core's token count against the companion's recorded 24,989 and D14's
  25,000.

Rule preservation beyond the changed lines was not re-audited; round 21 did
that.

## Round-21 item status

| Item | Status | Evidence |
| --- | --- | --- |
| B1 | **Resolved** | `reference.md:234–236` reads "`Copy`, `Eq`, `Hash`, `Pure`, `Send`, `Sync` for every `T`, `'static` when `T` is (02 §10.9)". `02-memory.md:1117–1119` reads "for every `T`, so it can be used as a key whatever `T` implements, and `Owned` when `T` is". Grading §6's G7 row (`grading.md:692`) says the same. This agrees with 02 §2.2 and §3.7 (`orig(Id<T>) = orig(T)`) and with the core's own definition of *owned* (`reference.md:188–189`). rustc 1.96.0 with the oracle's representation (`PhantomData<fn() -> T>`) accepts `need_static(id)` for `Id<String>` and rejects it for `Id<&'a str>` (E0521). One wording nit is in N2. |
| B2 | **Partly resolved; the new trigger is too broad: B1** | The "several `From` impls" condition is gone from `companion:42`, `grading.md:180–182`, and Q8 (`grading.md:256`), and the row's fix names `MyError`. rustc confirms the annotated body compiles and the unannotated one fails with two E0283 errors. But all three places now claim E0283 for *every* scope result fed to `?`. That is false when the enclosing function's error type is `TaskError`, which is exactly 02 §10.12's `parallel_sum`. Round 21's own B2 analysis ("always gives at least two candidates") made the same error. |
| N1 | **Declined; the reason is acceptable** | Companion 14.1 is a table keyed by what the writer typed, and its third column already lists diagnostics, so a rustc code fits. The header sentence "Sections 14–16 tabulate or illustrate rules the core states" (`companion:8–9`) is now slightly inexact, since the core does not state this rule. That does not need a fix. |
| N2 | **Resolved**; one parallel sentence noted (N1) | `README.md:124–126` no longer says grading uses no compiler, and `README.md:136–140` adds `e1-dialect/`. Its claims match the grading: 11/11 at first check (`grading.md:144`), no definite errors (§2.2), the oracle agreeing with every type and borrow verdict (`grading.md:175–177`), and `rustc/run.sh` exists. `01-design.md:292–294` now says the held-out round exercised outlives bounds, `use<..>`, and `Scope` "with no borrow-related error", which matches grading §4 (`grading.md:600–602`). The edit records status, not a decision. |
| N3 | **Noted, as allowed** | `.beads/issues.jsonl` records 24,989 for fx-arx, fx-1am, fx-i5p, and fx-eky. |
| Count | **Matches; 11 tokens of headroom** | Measured below. |

**Measured count.** `scripts/count-tokens.sh design/reference.md </dev/null`
was run once on the working tree described above. It printed **24989**. That
equals the companion's line (`companion:12`) and grading §6
(`grading.md:696–697`), and leaves 11 tokens under D14's 25,000.

**Other changed lines.**
- `reference.md:599` prints `pub struct Scope<'s, 'env: 's, E> { .. }`, the
  same line as 02 §8.2 (`02-memory.md:667`). The impl below it carries no
  `'env: 's`. That is valid Rust, because an impl header's types supply
  implied bounds; the scratch stub compiles that way, as `std::thread::Scope`
  does.
- Grading §6's G1 row (`grading.md:689`) and its G7 row agree with the text.
- No changed line cites a heading that moved. Reference 3.6's "02 §10.9"
  resolves to `02-memory.md:1091`. 01 D2's `trials/e1-dialect/` and the
  README's `rustc/run.sh` both exist.

## New blocking problems

### B1. E0283 is claimed for every scope result fed to `?`, but 02 §10.12's `parallel_sum` compiles

The changed lines now state the trigger without a condition:

- `reference-companion.md:42`: "a scope result fed to `?` rather than
  returned", mapped to rustc E0283;
- `grading.md:180–182`: std's `impl<T> From<T> for T` "always adds a
  candidate, so any scope result fed to `?` is ambiguous";
- Q8 (`grading.md:256`): "Only where the scope's result is the operand of
  `?;`: rustc E0283 without it, whatever the error enum's `From` impls".

The inference has two constraints: `F: From<E>` from the outer `?`, where
`F` is the function's error type, and `E: From<TaskError>` from `scope`
(`reference.md:592–593`). When `F` is not `TaskError`, `E = F` and
`E = TaskError` both satisfy them, so `E` is ambiguous. When `F` is
`TaskError`, only `E = TaskError` does, and inference succeeds. It also
succeeds when the body or a child names `E` (for example
`Err(MyError::Bad)`).

- **02 contradicts the row.** 02 §10.12's `parallel_sum`
  (`02-memory.md:1173–1182`) returns `Result<u64, TaskError>` and ends
  `tasks.scope(|s| { .. })?;` with no annotation. 02 accepts it, and under
  D18 an accepted program is valid Rust.
- **The grading contradicts itself.** Its Method says the oracle accepted
  that same `parallel_sum` "as it should be" (`grading.md:112`). §2.2 then
  says any such result is ambiguous.
- **Checked with rustc 1.96.0** (edition 2024, `--emit=metadata`), against
  a scratch stub of 7.1's signatures with `Scope<'s, 'env: 's, E>`:
  - `parallel_sum` verbatim compiles;
  - the same body with error type `MyError` (whose only impl is
    `From<TaskError>`) fails with two E0283 errors: "type annotations needed
    for `Scope<'_, '_, _>`", with "multiple `impl`s satisfying
    `MyError: From<_>`";
  - with `Box<dyn Error>` it fails the same way;
  - with `MyError`, but a child that returns `Err(MyError::Bad)` or a body
    that returns it, it compiles;
  - with `|s| -> Result<(), MyError> { .. }` it compiles.

The annotation the row recommends is harmless in every one of these cases.
What is false is the claim of when rustc reports E0283. It is recorded as an
oracle finding, and the companion row implies that 02's own example fails.
The fixes cost no core tokens.

**Fix.**
- `companion:42`, first cell: "`tasks.scope(\|s\| { .. })?;` into an error
  type other than `TaskError`". Keep the second and third cells.
- `grading.md:180–182`: "because `?` converts through `From`: when the
  function's error type `F` is not `TaskError`, `E = F` and
  `E = TaskError` both satisfy `F: From<E>` and `E: From<TaskError>`
  (std's `impl<T> From<T> for T`), so `E` is ambiguous unless the body or a
  child names it (round 21, B2; 02 §10.12's `parallel_sum`, whose error
  type is `TaskError`, needs no annotation)". Rewrap to 80 columns; the
  current line 182 is 101.
- Q8 (`grading.md:256`): "Only where the scope's result is the operand of
  `?;` in a function whose error type is not `TaskError`, and nothing in the
  body names the error type: rustc E0283 without it, whatever the error
  enum's `From` impls (§2.2)."

## Non-blocking notes

### N1. 01 D2's status line drops "provisional", and 02 §12 has a parallel sentence

- `01-design.md:292–294` says the held-out round exercised the three
  features "with no borrow-related error". Grading §2.5
  (`grading.md:226–237`) calls the result provisional: one writer and
  eleven tasks against D0's thirty, and an oracle that runs no test. D2's
  sentence reads as settled.
- 02 §12 item 9 (`02-memory.md:1317–1320`) still says the origin trial "did
  not exercise outlives bounds, `use<..>`, or `Scope<'s, 'env, E>`". That
  is still true of the origin trial, so it is not stale. It is, however,
  the sentence D2's edit was paired with, and it now gives a reader of 02
  no pointer to the round that did. The item's open question (whether
  agents repair lifetimes from diagnostics) stays open: e1-dialect also
  produced no borrow error to repair.

Both are owner-facing text, so this is a report.

**Fix (optional).** In D2, append "(provisional: one writer, eleven tasks;
grading §2.5)". In 02 §12 item 9, append "`trials/e1-dialect/` did, again
with no borrow error to repair."

### N2. Nits

- **Reference 3.6's last clause.** `reference.md:235–236` ends "`'static`
  when `T` is (02 §10.9), and also `Ord` and `Debug` (provisional)". After
  a conditional clause, "and also" can be read as also conditional. The
  oracle stub implements both for every `T`
  (`trials/e1-dialect/rustc/std/src/lib.rs:362–368`). A reorder that
  should cost about the same number of tokens: "is, for every `T`, `Copy`,
  `Eq`, `Hash`, `Pure`, `Send`, `Sync`, and `Ord` and `Debug`
  (provisional); `'static` when `T` is (02 §10.9)." Recount if it changes,
  since the core has 11 tokens left.
- **02 §10.9's wrap.** `02-memory.md:1119` ends with a dangling "A" after a
  short line. Reflow the paragraph.
- **fx-jv9's progress comment** still gives the core as 24,986. Comments
  are append-only, so add the 24,989 recount in the comment that closes the
  bead, not in an edit.
- **Headroom.** 11 tokens remain. Any core fix from fx-i5p or fx-eky must
  trade text for text, as both beads say.

## Ready to implement v0

**Not yet for this change.** Round 21's B1 is closed and its N items are
answered. B2's fix overcorrected: the companion row and the grading now claim
an E0283 that 02 §10.12's accepted `parallel_sum` does not produce, and the
grading's §2.2 contradicts its own Method. The fixes are a few words in the
companion and the grading, and none touches the core. Nothing here blocks
fx-i5p or fx-eky.

Validation: this was a read-only review of the working tree on `5d96a2e`,
on 2026-10-20.

- **Read:** `reviews/round20-consistency.md` (format),
  `reviews/round21-consistency.md`, the full design and README diff, 01 D2,
  02 §8.2, §10.9, §10.12, and §12, reference 3.4, 3.6, and 7.1, the
  companion header and 14.1, the README's status and trials list, grading
  Method, §1, §2, §3.0, §4, §5, and §6, the oracle stub's `Id`, and the
  bead records in `.beads/issues.jsonl` for fx-arx, fx-1am, fx-i5p,
  fx-eky, and fx-jv9.
- **Rust checks:** scratch stubs in the session scratchpad, compiled with
  `rustc 1.96.0 --edition 2024 --crate-type lib --emit=metadata`. They were
  not kept; the results are quoted above.
- **Count:** `scripts/count-tokens.sh design/reference.md </dev/null`, run
  once on the working tree, printed `24989`.

No Tide checker exists, so the Tide verdicts above are inferences from the
text. The Rust verdicts were checked with rustc. Only this file was written;
nothing was staged or committed.

## Resolution

Answered in the commit that adds this review.

- **B1:** companion 14.1's row now reads "into an error type other than
  `TaskError`, when no `Err` in the scope names one". Grading §2.2, Q8, and
  §6 say the same, and §2.2 cites `parallel_sum` as the case that needs no
  annotation; `grading.md` lines 115 and 182 are rewrapped. Checked with
  rustc 1.96 against the oracle stub: a `?`-fed scope into `TaskError`
  compiles, into a `MyError` with `#[from] TaskError` fails with E0283, and
  the annotated body compiles.
- **N1:** 01 D2's status line and 02 §12 item 9 both say the result is
  provisional (one writer, eleven tasks).
- **N2:** 02 §10.9's dangling line is rewrapped; fx-jv9's closing comment
  gives the 24,989 count. The `Ord`/`Debug` reorder is declined: the
  sentence lists unconditional traits first, and the core has 11 tokens to
  spare.

This is the second restart for text-only fixes in this slice, so no further
confirmation round is run (AP-PROOF-001); the B1 fix was checked with rustc
instead.
