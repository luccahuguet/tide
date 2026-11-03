# Tide review, fx-kw8.10 round 5 (Opus): converge confirmation round

Candidate: branch fx-kw8.10 at a1d6e71 on base 0141db4, diff SHA-256
`72bc8743ebfa72efef0c689c198526888d3e6c05f553e0aa20436c44bf67713b`.

Model: claude-opus-5-5 · Date: 2026-11-03

## Verdict

Findings: 1 blocking, 1 non-blocking, 0 nits. Round 4's B1 is fixed only
when no other item holds the same impl. Round 4's N1 is fixed for the cases
it named. `cargo test --workspace` and `conformance/run.sh` both exit 0 at
a1d6e71. Stable `rustfmt --check` is clean on the three edited sources.

## New findings

- **B1, blocking** (`invocation.rs:69-74`): the generic users' holes are
  filtered by `what` across the whole crate. If any other item holds the
  same impl, a generic user's hole is dropped, even when that item has no
  relation to it. Inputs, with `--tide-grant=unsafe`:
  - `pub fn gsort<T>(_: T, mut v: Vec<fn()>) -> Vec<fn()> { v.sort(); v }`
  - `pub fn g1() -> Vec<fn()> { gsort(0u8, Vec::new()) }`
  - plus either
    `pub fn sorted(mut v: Vec<fn()>) -> Vec<fn()> { v.sort(); v }` or
    `pub fn lt(a: fn(), b: fn()) -> bool { a < b }`

  `lib::gsort` then lists only `uses impl core::cmp::Ord for fn()`. Its
  `PartialOrd` hole is gone, although the local instance `gsort::<u8>` runs
  it. Without `sorted`/`lt`, the same `gsort` lists both holes. So an
  item's holes depend on unrelated items. That contradicts two docs:
  interface.rs:24-26 ("by that item when a local instance of it does") and
  the holders test's doc. It also breaks the bead's "never silently report a
  partial artifact". Fix: drop a generic user's hole only when one of the
  items that instantiate it holds the same `what`. For example, return each
  generic hole with the keys of `local_uses(users, owner instance)`, and
  filter against those holders in `invocation.rs`. A second option: keep
  the hole when the call at the site has no type parameters in the owner's
  identity MIR (`!args.has_param()`). Add `sorted` beside `sort` in the
  holders test.
- **N1, non-blocking** (`interface.rs:235-239`, module doc line 19): finite
  recursion can still go `opaque` after two growth steps, because growth
  matches any held subterm. Input:
  `pub struct P<T> { pub t: T, pub a: Option<Box<P<W<Net>>>>, pub b: Option<Box<P<Arc<Net>>>> }`
  with `pub fn p2(_: &P<Net>) {}`. It gives
  `#0.a.b: opaque(P<Arc<Net>>)` and `#0.b.a: opaque(P<W<Net>>)`. Yet `P`'s
  identity listing and `p(&P<u8>)` expand fully. The result stays sound, but
  the doc says finite recursion "does not" go opaque, and the module doc
  blames polymorphic recursion. Fix: say "one growth step expands; deeper
  growth, finite or not, is `opaque`". Alternatively, decide growth from the
  definition, as r4 suggested.

## Checked

- r4 B1 probes:
  - Two generic users of one impl (`gsort`, `hsort`) each hold `Ord` and
    `PartialOrd`.
  - An uncalled generic holds the walk's `Ord` only, as the decision doc
    says.
  - `debug<T>` with `shown` passing `fn()` or `vec![fn()]` holds nothing at
    `debug`. `shown`, and `shown2` with a tuple, hold `Debug`.
  - Generic-set dedup across instances: `Walk::holes` is a set.
- r4 N1 probes:
  - `G<u8>` gives `#0.g.t.0: net`.
  - `C<W<W<Net>>>` gives `#0.next.t.0: net`.
  - `Node<u8>` and `Node<Net>` expand.
  - `Poly` expands one step, then is `opaque`.
  - `A<A<T>>` is opaque at `n.n`.
  - The M1/M2 mutual recursion terminates three levels deep.
- Blowup: mutual chains of 8, 12, and 20 defs that each double a tuple stay
  within the 4096 budget. They take 0.3-0.9 s and about 110 MB, and write
  1.1-2.6 MB artifacts (about 2040 paths for 8 defs, against about 256
  before). The budget bounds the result; this is not a finding.
- `check_mono_items`'s split return, `take_holes` reuse, and the count
  merge in `invocation.rs`.
