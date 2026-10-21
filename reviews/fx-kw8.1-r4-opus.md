# Tide review, fx-kw8.1 round 4 (Opus): converge round

Candidate: HEAD 9df7071 on 187a22b (worktree `/home/lucca/pjs/lab/tide-fx-kw8.1`, branch
fx-kw8.1). This round reviewed `git diff 187a22b HEAD` in full, checked round 3's walk.rs fix,
and went through the routes by which user code could select an impl without the walk seeing
it. It fixed what it found in the working tree and did not commit. Probes are in this
scratchpad's `t4/` and run through `adm4.sh` (round 3's `adm3.sh`, writing to `t4/`).
`sweep4.sh <name>` runs every probe from rounds 1-4 and every E1 program as lib, `--test`, and
`--bin`, and writes `sweep4-<name>.txt`.

## Verdict

fixed

## Findings and fixes

**F1. A trait impl's supertraits and associated-type bounds were never obligations
(admission hole).** rustc proves these where the impl is written. Code that relies on the impl
then uses them, but selecting the impl does not show them. `obligation` followed only the
impl's own where-clauses, so each route below formatted `fn()` without the grant. Every probe
exited 0 at HEAD:
- **Supertraits through a generic bound or `dyn`.** Take `trait Shown: Debug {}` and
  `impl Shown for fn() {}`. Then `show(noop as fn())`, with `fn show<T: Shown>` formatting `t`,
  and `&f as &dyn Shown` formatted with `{:?}`, were both admitted (`t4/sup.rs`).
- **`Error`'s `Debug` through `Box<dyn Error>`.** Take `#[derive(Debug)] struct E<T>(T)` and
  `impl Error for E<fn()> {}`. Then `r?` into `Result<(), Box<dyn Error>>`, and
  `Box::new(E(noop as fn()))` as `Box<dyn Error>`, were admitted. Printing such an error is
  how `main` reports one (`t4/sup.rs`).
- **Associated-type bounds.** `trait Tr { type A: Debug; .. }` with `impl Tr for u8 { type A =
  fn(); .. }` was admitted, and generic code formats `T::A`. The same held for a GAT
  (`type It<X>: Debug`) and for return-position `impl Debug` in a trait, which desugars to an
  associated type: both `fn f(&self) -> fn()` and `-> impl Debug { noop as fn() }` in the impl
  were admitted (`t4/assoc.rs`, `t4/rpit.rs`).

*Fix (`walk.rs`):* `visit_item` calls a new `trait_impl` for each local trait impl. It checks
the obligations rustc proves there, in the impl's typing environment:
- the trait's clauses at the impl's trait ref, which include the supertraits;
- each associated type's `explicit_item_bounds`, at the impl's args rebased onto the
  associated type's own, as rustc's `check_type_bounds` does. This also covers GATs and
  return-position `impl Trait` in traits.

A clause that names the impl's own parameters is met by the impl's where-clauses. Those are
checked wherever the impl is selected. Every other clause is checked at the impl: its header,
or its `type` item. Round 3 wrote the clause-to-trait-ref conversion out twice. It is now one
helper, `where_clauses`, used in three places.

**F2. A builtin impl's obligations on a type's parts were not followed (admission hole).**
`Clone` for tuples and closures is builtin in rustc, and so it clones each element or capture
through that element's impl. Take `impl<T: Debug> Clone for C<T>` that formats `self.0`. Then
`(C<fn()>,).clone()` and `.clone()` of a `move` closure capturing a `C<fn()>` were admitted.
The array version was already rejected, because core has a library impl for arrays
(`t4/bclone.rs`).
*Fix:* `select` (renamed from `select_impl`) returns either the user-written impl or a builtin
impl's nested trait obligations, and `obligation` follows the nested ones too. It skips auto
traits' builtin impls. Those run no code, and following them would reach the private fields of
std's types and bring false rejections. The doc comment on `obligation` and the module doc
name the new routes.

`conformance/cases/authority/admit_obligation_routes.rs` gets three new marks: the supertrait
route (`impl Shown for fn()`), the associated-type route (`type Out = fn();`), and the tuple
`clone()`. The header comment names them. At HEAD's `walk.rs` the driver reports only the four
old marks, so the case FAILs. With the fix it reports all seven, so it PASSes. Stable rustc
accepts the case.

## Not changed

- **Where F1 reports.** F1's rejections fall on the impl, not on the use. rustc proves the
  obligation at the impl, and the impl is user code. A rejected impl rejects the crate,
  whether or not it is used. In the corpus, the E1 programs, and every earlier probe, no
  impl was newly rejected. That includes derived impls and `#[derive(Error)]`.
- **Routes probed and found closed at HEAD** (`t4/ops.rs`, `t4/misc.rs`, `t4/rpit.rs`):
  - overloaded operators: binary, compound assignment, unary, `==`, and `<`;
  - `Index`, `IndexMut`, and explicit `*`/`**` through user `Deref` and `DerefMut`;
  - `as &dyn Debug` casts, and `dyn` built in a return value, in a struct field, and in
    `Box<dyn Fn>` capturing a value;
  - `assert_eq!`, `assert!` with a message, `dbg!`, `matches!`, and closures calling user
    impls;
  - opaque `impl Trait` returns, associated consts through `<W<fn()> as Tr>::C`, default
    methods with `where Self: Debug`, and generic calls;
  - `sort`, `max`, and `BTreeSet` on `fn()`;
  - `static`/`const` initializers coercing `&fn()` to `&dyn Debug`.
  Each gave FX0520 at the use.
- **Const patterns** call no user impl. A const pattern of an ADT needs structural equality,
  and `&str` patterns call std's `str: PartialEq`.
- **Drop glue for fields.** A type with a `Drop` impl must declare that impl's bounds itself
  (E0367), and rustc's well-formedness rules carry them to every type that contains it. So
  some expression or pattern in user code has a type that names them, and `use_ty` checks it.
  No probe found a value made without one.
- **Opaque types' bounds are not checked where the opaque is defined.** Every use creates an
  obligation, and post-analysis typing reveals the opaque in it (round 2, and `t4/misc.rs:10`).
- **Ambiguous selection and object candidates** still check nothing. That is as in rounds 1-3.
  The impl behind an object was checked where the `dyn` was made, and now its supertraits are
  checked too (F1).
- `async` was out of scope, as briefed.
- `design/reference.md` is untouched, at 24,992 tokens. ADMIT-1 already says "impl selected
  for an obligation", so no design text changes.

## Checked

- `git diff 187a22b HEAD`: every non-review file was read, along with the four earlier
  reviews, the bead, and GOVERNANCE.md's Merge Rule.
- Probes (`adm4.sh`, pinned-nightly stub std):
  - at HEAD, `sup.rs` exited 0, `assoc.rs` exited 0, `rpit.rs` rejected only the
    `static`/`const` lines, and `bclone.rs` rejected only the array line;
  - after the fix, every route above gives FX0520. `assoc.rs`'s admitted impls
    (`type A = Vec<u8>`, `impl<T: Debug> .. type A = Vec<T>`, and a supertrait met by a
    derive) stay admitted.
- `sweep4.sh`: rounds 1-3's probes in `t/`, `t2/`, and `t3/` (including `t3/mf/`), the E1
  programs, and `t4/`, each in three modes (192 runs). HEAD's `walk.rs` and the fixed one give
  identical output except for the new `t4/` holes. That covers all E1 programs and all earlier
  probes, with no ICE or panic.
- The extended case reports 4 of its 7 marks at HEAD's `walk.rs` and all 7 after the fix.
- `CARGO_BUILD_JOBS=4 cargo test --workspace`: 2 + 4 passed.
- `conformance/run.sh`: exit 0, 140 PASS, 57 AWAITS CHECKER, 0 FAIL.
- `rustfmt --edition 2024 --check` (stable) on the crate's sources and tests is clean.
  `git diff --check` is clean.
- `scripts/count-tokens.sh design/reference.md`: 24,992.
- Load stayed below 1.1, and builds ran one at a time.
