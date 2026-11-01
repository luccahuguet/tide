Model: Opus 5.5. Date: 2026-10-31. Round 1, candidate 5855731b (uncommitted on 1e0cbac), read-only converge round.

# fx-nkv.1 round 1 (Opus): production std bootstrap and trusted main entry

**Verdict: changes needed.** One blocking finding, in the sinks, and four nits.

Verified on the candidate: `cargo test -p tide-std -p tide-macros` passes (5 of 5); the full corpus exits 0 (run from a copy of `run.sh` with its target dirs moved). Under the driver, `print!`, `eprintln!`, `dbg!`, and `panic!` expansions are admitted, and `#[tide::main] fn main() -> Result<(), fn()>` is still rejected with FX0520 (`impl Debug for fn()`). The `#[tide::main]` wiring, the `Root` and capability shape that `root.rs` forces, ROOT-4 privacy (E0451, E0603, E0277), the once-per-process guard, and the 141 exit on a closed stdout check out. Removing the `_print`/`_eprint` rules orphans nothing; `generated_by_toolchain` still recognizes the new expansion; `policy.rs` and `pure.rs` keep their keys.

**B1 (blocking). The sinks write while they format** (`crates/tide-std/src/root.rs`, `__sink`). `write_fmt` streams each formatted piece into the locked stream and stops at the first write error. Confirmed with programs built against the candidate's rlib:

- With stderr a closed pipe, `eprintln!("{}{}{}", A(&c), A(&c), A(&c))`, each `Display` bumping a counter, ran one impl, not three; so the destination decides which formatting impls run, and a later panicking impl never panics (SINK-3).
- A formatting impl that prints lands inside the outer record: `println!("outer[{}{}]", 1, X)` with X printing `inner` wrote `outer[1inner\nx]\n`, against the doc's "written whole".
- A `Display` that panics leaves `head ` on stdout, then exits 101.

Fix: format the record into a `String` first, then lock, write, and flush; keep BrokenPipe → 141. Add a test.

**N1. `print!()` and `eprint!()` with no arguments compile**; Rust rejects them ("requires at least a format string argument"), and reference 2.1 keeps these forms as in Rust. Use `""` for empty input only with a newline.

**N2. The crate docs' "not provided" list is incomplete**: the prelude's `Error` trait (reference 12.3), `std::path`, `io`'s Rust re-exports, `time::Duration`, and `ptr`. Better, move `pub use std::error::Error` into `declare_prelude!` and `path` into `root.rs`'s re-exports, and drop the stub's copies.

**N3. A conformance README sentence is broken**: the inserted sentence took over "and mark `GUESS` ...", which belongs to the earlier one; `README.md`'s new lines are not reflowed.

**N4 (v1). The package name differs from the design's**: `design/01-design.md:827` and `design/03-authority-concurrency.md:1963` fix the export as `std = { package = "tide_std" }`; the crate is `tide-std`. Name the package `tide_std`, or record that the design text must change.
