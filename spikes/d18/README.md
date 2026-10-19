# D18 feasibility spike

A throwaway prototype for bead `fx-7qu`. It tests whether Tide can ship as
a checked dialect of Rust (`design/01-design.md` D18). The results and
their reading are in `research/15-rust-dialect.md`, "Spike results".

It is agent-written and is not production code. It has not been through
converge or the supervisor, and nothing in Tide depends on it.

## Layout

| Path | Criterion | What it is |
| --- | --- | --- |
| `facade/` | 1, 5 | `tide_std`, the facade std injected with `--extern std=`: capabilities, tasks, `Shared`, channels, a deterministic `HashMap`, `Pure`, float and math functions |
| `macros/` | 3 | `#[requires]`, `#[ensures]`, `#[invariants]`, and `invariant!` lowering |
| `examples/` | 1, 3, 5 | programs that must build, run, or fail; `leaks.rs` lists what the facade alone does not confine |
| `admit/` | 2, 4, A5 | `tide-admit`, a rustc_private driver that enforces the admission allowlist (`policy.txt`) and the `Pure` check; `scripts/tide-rustc` is the driver-flag gate |
| `eon/` | 6 | a trimmed Eon workspace (Orbit and orbit-protocol); sources are extracted by `scripts/fetch-eon.sh` |
| `scripts/` | all | one script per criterion; `all.sh` runs C1, C3, C5, C6, C7 |

## Running

- `scripts/all.sh`: stable Rust (1.96). C6 needs `~/pjs/eon/eon` at commit
  `2332d4b`, or `EON_SRC` set, plus Zig for libghostty-vt.
- `admit/scripts/run-tests --facade` and `admit/scripts/orbit-report`: the
  pinned nightly in `admit/rust-toolchain.toml` with `rustc-dev`.

Each script exits nonzero on failure.
