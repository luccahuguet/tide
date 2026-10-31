Model: Opus 5.5. Date: 2026-10-31. Round 1, candidate 25642709 (uncommitted on 5472a93).

# fx-kw8.4 round 1 (Opus): mono-item backstop

**Verdict: request_changes (confidence 0.85).** The backstop does catch the r2–r5 routes. But `-Clink-dead-code` switches the collector to its Eager strategy, which instantiates code that never runs, and legitimate programs are then rejected.

The candidate hash was `25642709…db7` at the start and at the end. `cargo test --workspace` passes. `conformance/run.sh` exits 0 (180 PASS, 28 AWAITS CHECKER).

Checked against the pinned compiler source, and they hold:
- a metadata-only build has no reachable-non-generic roots (`rustc_codegen_ssa/src/back/symbol_export.rs:55-57`);
- `-Zshare-generics=no` makes `upstream_monomorphization` return `None` (`rustc_middle/src/ty/instance.rs:224`);
- `items_of_instance` is the query the collector itself caches (`collector.rs:475`).

### Blocking

**B1. Eager collection makes the backstop reject code that never runs.** Verified by probes.
- **Where:** `crates/tide-admit/src/invocation.rs:158` with `crates/tide-admit/src/walk/mono.rs:43-75`.
- **Why:** `-Clink-dead-code` selects `MonoItemCollectionStrategy::Eager`. For every non-generic trait impl it instantiates each provided method the impl does not override (`collector.rs:1763-1830`, `create_mono_items_for_default_impls`). It also makes every const value a root (`:1585`) and adds drop glue for every non-generic ADT (`:1529`). `check_mono_items` checks all of these.
- **Probes** (an unrelated type error, which stops the backstop, shows the walk alone admits each):
  - `impl Iterator for Fns { type Item = fn(); … }` with `for f in Fns(3) { f() }`: `lib.rs:1:1: FX0520: impl core::cmp::Ord for fn() is not admitted`, from `Iterator::max`/`min`, which nothing calls.
  - a dispatch-table iterator with `Item = (u8, fn() -> u8)`: the same, against Rust's std and the stub std.
  - `Item = W<fn()>` with a local `impl<T: Ord> Ord for W<T>`: FX0520 at a local span inside the never-called `W::cmp`.
  - against the stub std, a local trait whose provided `fn snapshot(&self) where Self: Pure`, implemented for a struct holding a `Clock`, never called: FX0517. `show_pure` in `pure_routes.rs:107-110` has this shape, and the corpus treats it as legitimate.
- **Smallest fix:** keep `-Clink-dead-code` for the lib roots, but check only items reachable through the used-items graph from lazy-style roots: local `MonoItem::Fn` instances whose def does not require monomorphization, the `start` instance, and local statics. Add accept cases for the fn-pointer iterator and the provided `Pure` method, and correct the docs that say every non-generic item is a root.

### Non-blocking

- **N1. Failures reported with no location.** `collect_alloc` records vtable and fn-pointer uses from a const value with `DUMMY_SP` (`collector.rs:1303`); a const holding a vtable gives a bare `error: FX0520: …`. Fix: fall back to the def span when the span is dummy.
- **N2. Static initializers have no recorded users,** so their failures land at the crate root (`lib.rs:1:1` for a static holding a vtable; the walk catches it today). Fix: record a static's uses, or say so.
- **N3. Cost.** A 60-function, 601-line library: 1.74–2.29 s and 178 MB with the candidate, 1.11–1.21 s and 126 MB at HEAD. `mono.rs:51-66` searches for uses and decides `Pure` once per use, for every local or Tide-std instance, before anything has failed. Decide once per instance, and search for uses only on a failure.
- **N4. The FX0520 and `Pure` checks exclude each other** (`if`/`else if`, `mono.rs:51/67`): a Tide-std trait-impl method never gets the rule lookup. Latent; no rule keys a `tide_std` impl. Make them independent.
- **N5. The `Pure` half has no automated test:** the driver tests use Rust's std, and `conformance/run.sh` never passes the flag. Manually, the backstop alone gives FX0517 in `pure_routes.rs` at 40, 44, 48, 62, 77 (the route marked at 82, inside `make`) and 154; the other marks are generic-only.
- **N6. `-Zshare-generics=no` does not cover `#[inline(never)]` upstream generics** (`instance.rs:225-230`); ADMIT-2's "instantiate upstream generics itself" overstates it.
- **N7. `--tide-test-backstop` is accepted by any build** and switches off most of the walk's `Pure` checks. Gate it, or accept it as an operator-trusted flag.
- **N8. The new test never asserts the `HashMap: From` message** on its first line.

None needs to go on fx-djp: every fix lands in code or in tiers this change already edits.
