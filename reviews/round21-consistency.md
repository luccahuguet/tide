# Tide consistency review, round 21: the E1 dialect round's reference fixes

## Verdict

**changes requested**

Reviewed the uncommitted working tree on HEAD `5d96a2e` ("Refresh open beads
for D18: fx-arx, fx-1am, fx-qzp") on 2026-10-20. The change is
`git diff -- design/` (02 §10.9, reference 3.6 and 7.1, companion header and
14.1) and §6 of `trials/e1-dialect/grading.md`. Scope, as asked:

- whether each changed line is correct against 01 (D2, D18), 02 (§5.4,
  §8.2, §10.9), and 03's TASK rules, and consistent with the rest of the
  reference pair;
- whether the dispositions in grading §6 match §5 and beads `fx-i5p` and
  `fx-eky`, and whether declining G2 is justified;
- the core's token count against the recorded 24,986 and D14's 25,000;
- whether any citation into the changed sections went stale.

The rest of the trial record (grading §1–§4, programs, notes) was read only
where §5 and §6 rely on it. It was not re-graded.

## Item status

| Item | Status | Evidence |
| --- | --- | --- |
| G1 | **Resolved** | `reference.md:599` prints `pub struct Scope<'s, 'env: 's, E> { .. }`. That is 02 §8.2's line (`02-memory.md:667`) and 01 D2's "`Scope<'s, 'env, E>` declares `'env: 's`" (`01-design.md:251–253`), and §5.4's lower bound (`02:517–521`) now follows from the printed signature. 03 §6.1 (`03:683`) gives no struct line, but it defers the origin structure to 02 (`03:672`), so nothing disagrees. The comment keeps "Copy + Send + Sync" and leaves out 02's "invariant", which the core never needs. |
| G2 | **Declined; the reason holds** | Reference 3 adopts "Rust 2024's lifetimes (syntax, ..., `+ use<..>`, ..., `impl Trait` capture)" (`reference.md:115–117`), and 3.5 says "as in Rust 2024" (`:200–204`). 02 §2.3's `Capture` rule (`02:169–172`) states G2's proposed sentence word for word. 02 §5.2's `len_of` (`02:476–478`) shows an elided input released by `use<'a>`. The writer got both methods right, and rustc accepts them (grading `:334–340`, Q2 at `:249`). With 14 tokens of headroom, a restatement would have to displace other text. |
| G7 | **Not resolved: B1** | "For every `T`" is right for `Copy`, `Eq`, `Hash`, `Pure`, `Send`, and `Sync`. It is wrong for `Owned`/`'static`. |
| G8 | **Fix correct, trigger wrong: B2** | E0283 is what rustc reports, and the annotated body compiles. But the row's condition, "several `From` impls", is false. One `#[from] TaskError` is enough to trigger it. |
| Count | **Matches; 14 tokens of headroom** | Measured below. |
| §6 vs §5 and beads | **Match**, apart from the G7 and G8 rows, which repeat B1 and B2 | G3–G6 and G9 are in `fx-i5p`'s Evidence, and G10–G12 in `fx-eky`'s, with the same contingent items (C1–C4, C8 and C5–C6, grading `:195–202`). Each bead names the owning chapter, a falsifier, and D14's budget. |
| Citations | **None stale** | No heading moved. Citations of 02 §8.2 (`02:377`, `:522`, `:723`, `:1339`; `03:672`), 02 §10.9 (`reference.md:235`), core 7.1 (`reference.md:190`, `:192`, `:336`, `:686`, `:701`, `:797`, `:814`; `companion:49`), and the README's design list (`README.md:57–60`) all resolve. |

**Measured count.** `scripts/count-tokens.sh design/reference.md </dev/null`
was run once on the working tree described above and printed **24986**. That
matches the companion's line (`companion:12`) and grading `:695–696`. It
leaves 14 tokens under D14's 25,000, so B1's fix has to fit in them.

## New blocking problems

### B1. "`Owned`/`'static` for every `T`" is false in Rust and contradicts 02 §2.2

`02-memory.md:1117–1119` now says `Id<T>` is "`Copy`, `Eq`, `Hash`, `Owned`,
`Pure`, `Send` and `Sync` for every `T`". `reference.md:234–236` says "for
every `T`, `Copy`, `Eq`, `Hash`, `'static`, `Pure`, `Send`, `Sync`". The
quantifier is new in this change. Before it, both texts read as properties
of the `Id<Node>` at hand.

- **02 contradicts itself.** By 02 §2.2, `orig(S<..; T1..Tm)` includes
  `orig(T1) ∪ … ∪ orig(Tm)` for every struct (`02:133`). §3.4 says the same
  for containers (`02:337`). By §3.7, `Owned` means `orig` is empty
  (`02:366–368`). So `orig(Id<&'a str>) = {'a}`, and `Id<&'a str>` is not
  `Owned`. §5.2 also states the rule for opaque types: "`Owned` follows
  `orig`" (`02:485–486`).
- **D18 rules it out.** Every Tide program is valid Rust. Rust's outlives
  rule for a nominal type (RFC 1214) needs every type argument to outlive
  `'static`, whatever field holds it, even `PhantomData<fn() -> T>`.
  Checked with rustc 1.96.0 (edition 2024), using the oracle stub's
  representation (`trials/e1-dialect/rustc/std/src/lib.rs:346–350`):
  `need_static(id)` with `id: Id<&'a str>` is rejected (E0521).
- **What does hold.** The same check accepts the rest of the claim for every
  `T`. `Copy`, `Eq`, and `Hash` hold through manual impls. `Send` and
  `Sync` follow from `PhantomData<fn() -> T>`. A non-generic
  `#[derive(Clone, Copy, PartialEq, Eq, Hash)] struct Edge { to:
  Id<NotHash> }` compiles, which is the case G7 was about. `Pure` for
  every `T` agrees with 02 §9's structural rule (`02:795–796`, `:826–828`),
  since Tide function pointers are `Pure`.

The reference's own definition of *owned* (`reference.md:188–189`, "borrows
nothing except `'static` data") contradicts its new sentence for
`Id<&'a str>` in the same way.

**Fix.**
- `02-memory.md:1117–1119`: "`Id<T>` (a `u32` index and a `u32` generation)
  is `Copy`, `Eq`, `Hash`, `Pure`, `Send` and `Sync` for every `T`, so it
  is a key whatever `T` implements, and `Owned` when `T` is (§2.2)."
- `reference.md:234–236`: "`Id<T>` (a `u32` index and generation) is, for
  every `T`, `Copy`, `Eq`, `Hash`, `Pure`, `Send`, `Sync`, and also `Ord`
  and `Debug` (provisional); it is `'static` when `T` is (02 §10.9)."
  This adds about 5 tokens, within the 14 left. Recount, then update
  `companion:12` and grading `:695`.
- Grading §6, G7 row (`:691`): "every listed trait but `Owned` holds for
  every `T`".

### B2. The E0283 row's trigger is narrower than what rustc does

`reference-companion.md:42` limits the row to a scope "whose error enum has
several `From` impls". Grading §6 repeats this (`:692`), and so does the
grader's Q8 (`:255`).

The ambiguity does not depend on how many `From` impls the enum has. Rust's
reflexive `impl<T> From<T> for T`, together with the `#[from] TaskError`
variant that 7.1 requires of every scope error enum (`reference.md:627–628`),
always gives at least two candidates for `E` in `MyError: From<?E>` plus
`?E: From<TaskError>`. `E = MyError` and `E = TaskError` both fit. So any
`tasks.scope(|s| { .. })?;` whose result feeds `?` instead of the return
value is ambiguous.

Checked with rustc 1.96.0 (edition 2024) against a stub of 7.1's signatures,
with an enum whose only impl is `From<TaskError>`:
- two E0283 errors: "type annotations needed for `Scope<'_, '_, _>`", with
  notes "multiple `impl`s satisfying `_: From<TaskError>`" and
  "`One: From<_>`";
- the same with a two-impl enum;
- `|s| -> Result<(), Two> { .. }` compiles.

So the code and the fix are right, and the condition is wrong. A writer
whose enum has one `#[from]` would conclude that the row does not apply.
This is G8's own wording (`:656`, "when the scope's result feeds `?` rather
than the return value").

**Fix.**
- `companion:42`: first cell "`tasks.scope(\|s\| { .. })?;` (the result
  feeds `?`, not the return value)". Second cell "annotate the body:
  `\|s\| -> Result<(), MyError> { .. }`". Naming the enum avoids reading
  `E` as the scope's type parameter.
- Grading §6, G8 row: drop "with several `From` impls", and add one line
  correcting Q8's condition (reflexive `From`).

## Non-blocking notes

### N1. The row is hard to find from where an agent would look

The core describes companion 14 as "the Rust-habit tables"
(`reference.md:7–8`). The companion says 14–16 "tabulate or illustrate
rules the core states" (`companion:8–9`). A scope result fed to `?` is
neither a Rust habit nor a rule the core states, and this round's writer
did not load §14 (grading `:11–12`). An agent that hits E0283 would find the
row only by searching for the code.

**Fix (optional, no core tokens).** In `companion:8–9`, add "and 14.1 lists
rustc errors that Tide's APIs make common", or show the annotated form in
companion 15.2's fetcher. Moving it into core 7.1, as G8 proposed, costs
more than the 14 tokens left.

### N2. Two statements go stale once `trials/e1-dialect/` lands

- 01 D2's Accepted costs still says "The trial did not exercise outlives
  bounds, `use<..>`, or `Scope<'s, 'env, E>`; E1 must"
  (`01-design.md:291–293`). E1-dialect exercised all three, and it was
  chosen for that reason (grading `:5–8`).
- The README's trials list (`README.md:124–135`) has no `e1-dialect/`
  entry. It also says grading is done "without a compiler" (`:126`), but
  this round adds a rustc oracle (grading `:58`).

Both files are outside this change, and 01 is the owner's decision record,
so this is a report, not a request.

**Fix.** When the trial is committed, add an `e1-dialect/` bullet to the
README with its result. Ask the owner whether D2's sentence should give the
result or point to the grading.

### N3. Nits

- **Headroom.** The core has 14 tokens left (24,986). After B1, any fix from
  `fx-i5p` or `fx-eky` that touches the core will have to trade text for
  text, as both beads already say.
- **Std representation.** "`Send`/`Sync` for every `T`" binds std to a
  marker such as `PhantomData<fn() -> T>`, which the oracle stub already
  uses. With `PhantomData<T>`, the claim would be false. No text change is
  needed; the requirement is recorded here for whoever writes std (02 R4,
  `02:1279`).

## Ready to implement v0

**Not yet for this change.** B1 is a false normative statement in 02 and a
contradiction under D18. B2 misstates when rustc's E0283 applies. Both fixes
are a few words each. B1's core edit fits the 14 tokens left but needs a
recount. Nothing here blocks `fx-i5p` or `fx-eky`, which are filed and
match §5.

Validation: this was a read-only review of the working tree on `5d96a2e`,
on 2026-10-20.

- **Read:** the full design diff; grading §5, §6, §2.3, Q2, Q8, the Method,
  and the scan and tally findings; `br show` for `fx-i5p`, `fx-eky`, and
  `fx-jv9`; 01 D2; 02 §2.2, §2.3, §3.4, §3.7, §5.2, §5.4, §8.2, §9, and
  §10.9; 03 §6.1; reference 3, 3.4–3.6, and 7.1; companion header and 14.1;
  the oracle stub's `Id` and `Scope`.
- **Citations:** grepped `design/0*.md`, `README.md`, and the reference
  pair for citations of 02 §8.2, §10.9, core 3.6 and 7.1, and companion
  14.1, and checked each against its target.
- **Rust checks:** throwaway stubs in the session scratchpad, compiled with
  `rustc 1.96.0 --edition 2024 --crate-type lib --emit=metadata`. They were
  not kept, and the results are quoted above.
- **Count:** `scripts/count-tokens.sh design/reference.md </dev/null`, run
  once on the working tree, printed `24986`.

No Tide checker exists, so the Tide verdicts above are inferences from the
text. The Rust verdicts were checked with rustc. Only this file was written;
nothing was staged or committed.

## Resolution

Answered in the commit that adds this review.

- **B1:** reference 3.6 reads "`Copy`, `Eq`, `Hash`, `Pure`, `Send`,
  `Sync` for every `T`, `'static` when `T` is"; 02 §10.9 reads "for every
  `T`, so it can be used as a key whatever `T` implements, and `Owned` when
  `T` is". Grading §6's G7 row says the same. The core measures 24,989
  tokens, and the companion records it.
- **B2:** the companion 14.1 row's trigger is "a scope result fed to `?`
  rather than returned", and its fix names `MyError`. Grading §2.2, Q8, and
  §6 give std's `impl<T> From<T> for T` as the cause.
- **N1:** declined. Companion 14.1 is where an agent porting Rust looks for
  a rustc error it hits, and the core has no room for another pointer.
- **N2:** README lists `e1-dialect/` and no longer says grading uses no
  compiler; 01 D2's sentence now says the held-out round exercised outlives
  bounds, `use<..>`, and `Scope` with no borrow-related error. That is a
  status line, not a decision.
- **N3:** noted. Beads fx-arx, fx-1am, fx-i5p, and fx-eky record the
  24,989-token count; the `PhantomData<fn() -> T>` remark is for whoever
  writes std.
