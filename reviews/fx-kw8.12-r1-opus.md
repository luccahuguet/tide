Model: Opus 5.5. Date: 2026-10-31. Round 1, candidate 4d51f6e9 (uncommitted on bb17a0a).

# fx-kw8.12 round 1 (Opus): static-to-upstream-fn regression test

**Verdict: clean. No blocking findings. The test exercises the fixed path and fails on the pre-fix behaviour.**

### Blocking

None.

### Non-blocking

- `crates/tide-admit/tests/driver.rs:380-382`: the `tide_std` doc comment does not say why it takes a `name` (tests run in parallel and need separate directories). Nit.
- `crates/tide-admit/tests/driver.rs:515-516`: a failed assertion does not say which of the two flag sets failed; the ICE output makes it clear in practice. Nit.

### What I verified

- `git diff HEAD -- crates | sha256sum` = 4d51f6e9…74898; only `crates/tide-admit/tests/driver.rs` changes under `crates/`.
- The test meets the Acceptance: the exact program the bead names, against the minimal Tide std, in normal and `--tide-test-backstop` mode, expecting `(0, "")`.
- `cargo test -p tide-admit --test driver` on the candidate: 8 passed, exit 0.
- It catches the regression: in a copy of the crates with its own `target/`, a worklist search over everything `uses` returns (so `uses` asks `items_of_instance` about a static's pointees rustc never collected) made only this test fail, on its normal-mode pass, with exit 101 and round 2's ICE (`optimized_mir` for `alloc::str::<impl str>::to_lowercase`, through `items_of_instance` in `walk::mono::uses`). The other 7 tests passed, so this test alone protects the fix. Reaching the ICE in normal mode also shows the backstop runs there.
- `tide_std(name)` has no leftovers: both callers pass distinct names, nothing else calls it, and `<name>/tide-std` does not collide with `admit`'s output directory.
- Stable `rustfmt --edition 2024 --check` passes on `driver.rs`.
- Nothing for fx-djp.
