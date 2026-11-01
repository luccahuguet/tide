Model: Opus 5.5. Date: 2026-10-31. Round 2 (confirmation), candidate 888748a7 (uncommitted on 1e0cbac), read-only converge round.

# fx-nkv.1 round 2 (Opus): confirmation

**Verdict: changes needed.** One blocking finding (a regression test that does not exercise its fix), three nits, one out-of-scope note.

Proof re-run on the candidate: `cargo test --workspace` exit 0; the corpus (redirected copy of `run.sh`) exit 0, 210 cases; the workspace builds without warnings.

Round 1, rechecked: the sink finding (Opus B1, Codex 2) is resolved; records are formatted into a `String` before the lock, `records_are_formatted_before_they_are_written` distinguishes old from new, and a panicking `Display` writes nothing and exits 101. Opus N1, N2, and N4 are resolved; N3 is partly fixed (nit 2). Codex 1 (`#[tide::main]` off the crate root) is fixed in code but not tested (B1).

**B1. `conformance/cases/authority/root_main_nested.rs` does not test the crate-root restriction** (`crates.rs`, `entry_expansion`'s `opt_local_parent(def_id) == Some(CRATE_DEF_ID)`). `run.sh` checks a case as a library unless a line starts with `fn main`; every `fn main` in the case is indented, so it runs as a library (where `entry` is always `None`) and under `--test` (where the harness's `main` is the entry), and the two modes' marks are merged. A driver with that check removed still passes the case; a binary-shaped case (crate-root `#[tide::main] fn main`, a module-local `#[tide::main] pub fn main` that is called, and the nested one) fails under that mutant, admitting the module-local entry and rejecting the real one, and passes with the candidate's driver. Fix: make the case a binary with a crate-root `#[tide::main]` and `fn main() {}` at column 0. Optionally add a library-shaped case (`#[tide::main] fn main() {} //~ FX0520` on one line) to cover the `CrateType::Executable` check.

**Nit 1. A formatting error drops the record silently** (`root.rs`, `__sink::format`): `println!("head {}", Fails)` prints nothing and continues; Rust's `print!` and `format!` panic, reference 2.1 keeps these forms as in Rust, and SINK-5 covers records that cannot be written, not formatted. Panic as Rust does, or record the drop as a decision and stop citing SINK-5.

**Nit 2. Docs not reflowed**: `README.md`'s "The installed pinned toolchain is trusted..." line (99 columns), and `conformance/README.md`'s new "`Pure` is a blanket impl..." lines.

**Nit 3. Stale descriptions**: `.agent-protocols.local.md:42` (rendered into `AGENTS.md`) still describes `crates/` as only `tide-admit`; `tide-macros/src/lib.rs:1-4` omits the crate-root executable `main` condition.

**Out of scope.** A test can mint a real `Root` by calling the crate-root `main`, since `FX0506` (ROOT-5) is not enforced (`root_main_called.rs` awaits the checker); under `--test` the harness never calls `main`, so the test's call is the first `enter`. That breaks RUN-1 once fx-kw8.9 runs test executables. Recommend a bead for `FX0506` that blocks fx-kw8.9; a cheap mitigation is `#[cfg(not(test))]` on the generated `main`. Also, `main` moved to 8673252 (fx-kw8.8), which changes `walk.rs`'s `entry_point`; it should compose, but the candidate needs a rebase and a re-run proof.
