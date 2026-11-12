Model: claude-opus-5-5 · Date: 2026-11-12

Verdict: approve_with_alternatives

Scope: three parts, read-only.
- (A) Re-check of my r4 findings 1, 2 and 4, plus 3 and 6, in `git diff e992283 33b0a43`.
- (B) Design review of the reference, chapter 03 and companion edits in `33b0a43`.
- (C) The rebase delta, from `git range-diff dd40b78..33b0a43 aed282b..5649816`.

The worktree is at `5649816` and clean. Nothing blocks. Three non-blocking items and one nit follow; none needs a new round.

## (A) Re-check of r4 findings

- **Finding 1 (helper parameter patterns): resolved.**
  - Both parameter arms now require `entry` (`crates/tide-admit/src/walk/authority.rs:130`).
  - `root_parameter_pattern_reject.rs` expects FX0503 on both helper signatures.
  - The positive helper lines are gone from `root_destructure_accept.rs`.
  - The `rules.md` ROOT-1/ROOT-2 rows and `conformance/README.md` now say "field patterns destructuring `root` (including `main`'s parameter)".
  - Probe B: the r4 file now gets FX0503 on lines 1, 2 and 3. Entry, let, if-let and match forms remain accepted (cases unchanged).
- **Finding 2 (local `thread_rng`): resolved.**
  - The name check now runs only when `admit_def(def)` rejects (`authority.rs:84`). That reuses the admission owner, which is better than my suggested role test.
  - Probe A (local `thread_rng` and its caller) exits 0. `cap_ambient_accept.rs` and the paired-library build test cover it.
  - The unresolved-name route still reports, for example `thread_rng()` at `cap_ambient_paths_reject.rs:16` (probe I).
- **Finding 4 (one owner for unresolved-path recovery): resolved.**
  - The old module walk in `walk.rs` is deleted, and `Res::Err` goes straight to `authority_unresolved`.
  - `denied_key` (`authority.rs:488`) applies every `Verdict::Deny` rule with its `Rule.code`, keeping the legacy " (Tide's std does not export it)" note for non-FX0501 codes.
  - The resolved-def `ambient_key` call is gone; `use_def` decides those.
  - Probe H: aliases now keep FX0520. `s::LazyLock::new` (module alias) gets 5:13 FX0520, and `O::<u32>::new()` (renamed import) gets 6:13 FX0520. The old driver reported neither.
- **Existing diagnostics.**
  - I ran the e992283 driver (`r3-before-driver`) and the current driver on the same files against the current production std.
  - `cap_ambient_paths_reject.rs` and `global_lazy_lock.rs` produce identical FX lines: code, line, column and message.
  - In probe H, every pre-existing FX line keeps its code and position. See non-blocking item 2 for one message-text change.
- **Finding 3: resolved.** `park_timeout`, `park_timeout_ms` and `sleep_ms` rows were added. Probe C gets FX0501 at `sleep_ms` and `park_timeout` under Tide std. The native-std driver test now covers 18 sites. DNS (`ToSocketAddrs`) is deferred to fx-djp as directed. Tide's `net::ToSocketAddrs` is its own sealed trait with no `to_socket_addrs`, so the deferral doesn't affect `Net`.
- **Finding 6: resolved.** The CAP-1 row lists E0433 and E0432.

## (B) Design review

- **Exact and narrow: yes.**
  - Reference §6.2 (`design/reference.md:375-376`): "Name `Root` only in `main`'s signature or as `Root::sim`. Field patterns may destructure `root`, including `main`'s parameter."
  - It names `root`, the entry's value, so a helper's own `Root` parameter is not covered. That matches the rejected helper case and the companion's unchanged `fn helper(root: &Root)` FX0503 row.
  - Chapter 03 ROOT-1 says the same.
  - ROOT-2's "directly or through field patterns" replaces "one by one", which was misleading for a multi-field pattern.
  - `01-design.md:432` ("`Root` can be named only in `main`") stays consistent, because every allowed pattern lies in `main`.
- **Tightening: no rule meaning lost.** Every element is kept:
  - the attribute requirement with FX0512 and its safe fix;
  - minting `root` through the parameterless entry;
  - `root: Root` or nothing;
  - the three taught `Termination` forms;
  - `Err(e)` printing `Error: {e:?}` and exiting 1;
  - logging `Display` and the `source()` chain, then returning `ExitCode`;
  - panics exiting 101;
  - the full FX0503 prohibition list with its field examples;
  - FX0506, plus "tests call functions taking capabilities".

  Only the causal "so" before that last point and the two example comments were dropped (see the nit).
- **Count: confirmed.** `(ulimit -c 0; timeout 650 scripts/count-tokens.sh design/reference.md </dev/null)` on `5649816` printed **24987** and exited 0. That matches the companion's 24,987 and is 13 under D14's budget. The rebase changed no reference bytes; the range-diff has no `design/reference.md` delta, and the 03 delta between 33b0a43 and 5649816 is fx-hxw's own text from main.

## (C) Rebase delta

- **Conflicts kept both beads.**
  - `policy.rs`: order is `deny("tide_std::__rt::*")`, then fx-hxw's `allow("core::net::*")` and `allow("std::net::Shutdown::*")`, then the whole ambient table. No ambient key starts with `core::net` or `std::net::Shutdown`, so first-match cannot hide one.
  - `build.rs`: all four `tcp_*` tests and all three `authority_*` tests are present.
  - `rules.md`: TCP-1 sits between CAP-1 and CAP-2, and the bead's CAP-1, CAP-2 and ROOT rows are intact.
- **`cap_ambient_paths_reject.rs:14-15`** now expect `E0599 FX0501`. Probe I confirms E0599 at 14:34 and 15:36, and FX0501 at 14:13 and 15:13.
- **TCP fixtures: coverage kept.**
  - `tcp.rs` now has `Root::sim` only in `#[cfg(test)]` helpers (`simulated`, `simulation_model`) and in the `simulated_tcp` test. The production `main` runs only `suite` on the real `root`.
  - Real/sim equivalence is still compared. The harness takes the binary's `TCP_VALUES` and requires exactly `vec![real; 4]` from the four seeds, which is the old in-process `assert_eq!(real, values)` moved across artifacts.
  - Trace rerun determinism, trace contents, the per-seed model and `simulated(7)` are kept.
  - Unwind-hook totals went from 9/9/4 (binary only) to 1/1/0 in the binary plus 10/10/5 in the test. The +2/+2/+1 is the seed-7 run that was previously unchecked, so no execution was lost.
  - `tcp_deadlock.rs` became an exact `#[test] fn deadlock`. The harness keeps its failure, non-timeout, connection and site checks.
- **FX0501 never fires on Tide's `Net` (probe G).**
  - The `Net` calls `net.only(..).no_bind()`, `net.bind(..)`, `connect(addr)`, `local_addr`, `shutdown(Shutdown::Write)` and `accept` produce no FX diagnostic.
  - The missing Rust constructors on Tide's types do get FX0501: `TcpStream::connect`, `TcpListener::bind` and `TcpStream::connect_timeout`, plus `std::net::UdpSocket::bind`.
  - Mechanism: no table row has a `tide_std::` key, and `rust_counterpart(tide_std::net::Net)` finds no Rust `std::net::Net`.

## Non-blocking items

1. **The typed destructuring `let` goes slightly beyond §6.2's wording.**
   - `authority.rs:97-102` exempts a `Root` type annotation on a field-pattern `let`, and `root_destructure_accept.rs:5` relies on it (`let Root { clock, .. }: &Root = &root;`).
   - §6.2 permits naming `Root` only in `main`'s signature, `Root::sim`, and field patterns; an annotation is a type, not a pattern. It grants no authority, since the value still comes from `root`.
   - Alternative: drop the annotation exemption and move line 5 to a reject case (one FX0503), so the checker matches the reference exactly. Otherwise keep it as a deliberate leniency. Either is acceptable. If not folded in, it goes to fx-djp.
2. **One message-text change in the unresolved-path differential.**
   - The old walk reported an absolute path through a missing type at the type: `std::sync::LazyLock::new` gave "`std::sync::lazy_lock::LazyLock` is not admitted". The single owner names the final item: "`…LazyLock::<T, F>::new` is not admitted". Code, line and column are unchanged (probe H, lines 4 and 7).
   - So the report's "byte-for-byte unchanged" holds for its five sets but not for this path shape. The new text is more precise.
   - I found no FX0520 lost. Every type-level deny in `policy.rs` is a module glob (`std::sync::lazy_lock::*`, `std::sync::once_lock::*`, `core::sync::*`), so the associated items match too. `std::hash::RandomState::new` now gains coverage it lacked under the old walk.
   - For the future: an exact (non-glob) deny on a type would no longer cover `Type::assoc` paths. Add a comment in `policy.rs` or keep type denials as globs.
3. **Nit (text only): the FX0503 message (`authority.rs:143`) still says "in a field pattern".** That now misleads in the helper case (probe B's lines 1-2). Suggested wording: "…as main's parameter type, in a field pattern destructuring `root`, or as `Root::sim`…".
4. **Nit (text only):** the §6.2 example lost its two attenuation comments ("Fs below one directory", "Net to one host"). Their meaning is still in §6.1's Attenuation column, so the loss is small and buys tokens.

## Probes

Environment:
- Linux x86_64, pinned nightly.
- Drivers: the current `target/debug/tide-admit`, built 14:52:55 -03, after the rebase sources (`find crates -newer` lists nothing), and the e992283 driver copy `r3-before-driver`.
- Std: production `target/tmp/build/std/artifacts/libtide_std.rlib`, built 14:53:03.
- Command template: `--edition 2024 --emit=metadata --error-format=short --tide-std … --tide-dep-dir …`, under `ulimit -c 0`, `CARGO_BUILD_JOBS=4 nice -n 19 timeout 60` and closed stdin, through the session scratchpad's `probes/run.sh`.
- `build-gate.py 120` passed at 18:00:47Z before the one serial batch.
- The token recount is separate: `scripts/count-tokens.sh`, as above.

| Probe | Driver | Source | Result |
| --- | --- | --- | --- |
| A | new | local `fn thread_rng` and its caller | exit 0 |
| B | new | helper field-pattern params `&Root`/`Root`, plus `helper2(root: &Root)` | FX0503 at 1:33, 2:41, 3:23 |
| C | new | `thread::sleep_ms`, `park_timeout`, `to_socket_addrs`, `UdpSocket::bind` | FX0501 at 3, 4 and 6; `to_socket_addrs` gets E0599 only (fx-djp) |
| G | new | Tide `Net`/`TcpListener`/`TcpStream`/`Shutdown` use, plus Rust ambient constructors | no FX on lines 1-10; FX0501 at 12, 13, 14, 15 |
| H | new / old | absolute, module-alias and renamed-import `LazyLock`/`OnceLock`, `env::args`, unbounded `channel` | new: FX0520 at 2, 4, 5, 6, 7; FX0501 at 8; FX0505 at 9. Old: FX0520 at 2, 4, 7 only, with type-level keys |
| I | new / old | `cap_ambient_paths_reject.rs` | identical FX lines; E0599 at 14 and 15 |
| J | new / old | `global_lazy_lock.rs` | identical |

I ran no full suites and no `heavy.sh`. I made no repository or `br` writes, and did not open or search `trials/e1-checked/`.

DONE
