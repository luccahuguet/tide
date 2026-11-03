# Tide review, fx-kw8.10 round 4 (Opus): converge confirmation round

Candidate: branch fx-kw8.10 at 0381a13 on base 0141db4, diff SHA-256
`ebcc114ca9343c6b5ccc4074fdc4e51f56987e00c0b763d677904fc743bd7dd4`.

Model: claude-opus-5-5 · Date: 2026-11-03

## Verdict

Findings: 1 blocking, 1 non-blocking, 0 nits. Round 3's B1, N1, and nit are
fixed. N2 is fixed only for the cases it named. `cargo test --workspace` and
`conformance/run.sh` both exit 0 at 0381a13.

## New findings

- **B1, blocking** (`walk/mono.rs:66-71`): the backstop skips every generic
  user. Its comment says the walk then holds the hole by "the item whose
  types chose the impl", but that is false when the generic item names the
  concrete type itself and the impl is reached only through std's internals.
  Input: `pub fn gsort<T>(_: T, mut v: Vec<fn()>) -> Vec<fn()> { v.sort(); v }`
  with `--tide-grant=unsafe`. The artifact lists only
  `uses impl core::cmp::Ord for fn()` for `lib::gsort`; the
  `impl core::cmp::PartialOrd for fn()` that runs has no hole on any item.
  The non-generic `sorted` lists both. The holes list is therefore silently
  partial, against the module doc's "every site the `unsafe` grant admits".
  Fix: hold a generic user's granted impl unless the walk already holds
  `uses <key>` on some item (merge in `invocation.rs`, or have the backstop
  return the skipped holes separately). Add `gsort` to the holders test.
- **N1, non-blocking** (`interface.rs:312-319`): the `grows` test matches a
  held instance's type argument found at any depth inside any new argument.
  So finite recursion, even recursion that shrinks, becomes `opaque`:
  - `Node<T> { val: T, child: Option<Box<Node<Arc<Net>>>> }` at `Node<Net>`
    gives `#0.child: opaque(Node<Arc<Net>>)`; `Node<u8>` expands to
    `child.val: [net, sync]`.
  - `G<T> { t: T, g: Option<Box<G<(Net, u8)>>> }` at `G<u8>` gives
    `#0.g: opaque(..)`, though the identity listing of `G` expands it.
  - `C<T> { t: T, next: Option<Box<C<W<Net>>>> }` at `C<W<W<Net>>>` gives
    `#0.next: opaque(C<W<Net>>)`.

  The result stays sound, since `opaque` is declared. But the comment's claim
  that "the arguments grow" is wrong. Fix: decide growth from the
  definition. Recursion is polymorphic only when D's identity-form field
  types reach D with an argument that properly contains one of D's own
  parameters. A cheaper alternative: allow one growth step and report
  `opaque` only when an instance grows from one that itself grew.

## Checked

- r3 B1: probe `pub use Net as X; pub use drop as X;` lists both. So do
  `Clock as Y` beside `fn Y`, and `pub mod m { pub use Fs; }`. A private
  module's re-export and module re-exports are skipped.
- r3 N1: closures in non-generic fns hold `Ord`, `PartialOrd`, and `Debug`
  for `fn()`. A generic `debug<T>` holds none, and its caller does. A
  closure in a method of `impl<T> S<T>` is held by the walk.
- r3 N2: `T2<u8>` in `T2<Fs>`, `Swap<Net, u8>`, `W<W<Net>>`, and `List`
  expand. `Poly`, `M1`/`M2` (mutual, growing), and `A<A<T>>` stay `opaque`.
  The last is opaque at the root through Pure's too-deep check.
- r3 nit: `cargo fmt --check` is clean. The only lines over 100 columns are
  test string literals.
- README, `main.rs`, and `interface.rs` docs against the code. The
  walk/backstop count merge in `invocation.rs`.
