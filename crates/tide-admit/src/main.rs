//! tide-admit: rustc with Tide's admission gate (D18, architecture A).
//!
//! It constructs rustc's arguments from a fixed check/build interface, always passes rustc
//! `-Zallow-features=` so the crate cannot enable unstable features and `-Zpolonius=no` so
//! borrows get NLL's verdicts (ADMIT-2, ORIGIN-6), and checks every use the crate makes against
//! the admission allowlist (ADMIT-1), reporting each rejection at the use's span as `FX0520`
//! or its policy rule's code. `walk/authority.rs` enforces ambient operations (`FX0501`),
//! Root naming and whole-value use (`FX0503`), simulation outside test code (`FX0504`), and
//! handle creation missing its final `&Tasks` (`FX0505`). Ordinary functions in a `--test`
//! build remain production code unless nested under a test or a test-only cfg. A separate
//! `--tide-test-library` suite is test code; its candidate library is checked as production.
//! `walk/habits.rs` and the policy table reject async/await and future/pin APIs (`FX0601`),
//! unstructured spawning (`FX0602`), Mutex/RwLock habits (`FX0604`), select! (`FX0605`),
//! catch_unwind (`FX0701`), and Results ignored in statement position (`FX0702`). Explicit
//! `let _ = result;` is accepted, including for Result<(), Infallible>; other must-use types
//! retain rustc's lint. FX errors ignore allow attributes.
//! It also decides `Pure` (`pure.rs`) and the rules on items that need it or the `unsafe` grant,
//! each under its own FX code (D18). It checks after name resolution and before rustc's
//! analysis, so a crate with rustc errors still gets its verdicts, and after the analysis, when
//! nothing failed, checks the instances that can run (`walk/mono.rs`) and writes the crate's
//! trust artifact (`interface.rs`).
//! `--emit=link` continues that same checked compilation through LLVM and linking, for a binary
//! (`--crate-type=bin`) or Rust test harness (`--test`). `--tide-profile=debug|release` selects
//! optimization 0 or 3 (default debug); both force overflow checks on. Metadata remains the default.
//! On the supported x86_64-unknown-linux-gnu host, linking uses `/usr/bin/cc` with
//! `/usr/bin/ld` (GNU cc flavor, `-B/usr/bin`, bundled linker disabled). Cache keys include the
//! output mode, profile, test configuration and cc/ld bytes;
//! the installed C toolchain/start files/libraries, like the sysroot, must remain immutable.
//! Unsafe-granted link builds are uncached: assembly/native inputs bypass the source snapshot.
//! Only a successful compilation publishes `<out-dir>/<crate-name>` (or `<crate-name>-test`),
//! beside its trust artifact. A linked build removes its previous executable before checking.
//! The shared `<crate-name>.interface.json` belongs to the latest invocation; separate output
//! directories retain bin/test/profile artifact pairs. The driver never runs an output.
//! `--test --tide-test-library SOURCE.rs` admits a library from a separate package, then
//! compiles the test crate with that library available as `tide_candidate`. Both use full
//! admission without grants. Both modes build a fresh library rlib in private staging.
//! Metadata mode checks the suite without linking it. Link mode publishes the test executable.
//! A killed build can leave a `.tide-test-*` directory. Later invocations never read it.
//!
//! This is a partial build path for reviewed/trusted demo code only: the operator must review
//! source, grants and dependencies before building/running, including granted assembly and native
//! link inputs. There is no assembler/linker OS sandbox, snapshot execution gate, or RUN-5
//! coverage verdict. Tide std supplies plain/property `#[test]`, `Root::sim`, and process-isolated
//! `#[sim_test]` entries in libtest; simulation children have no OS sandbox. The `tide test` and `tide sim` commands
//! are planned.
//!
//! Usage: tide-admit [--tide-grant=unsafe] [--tide-std PATH] [--tide-dep-dir DIR] <source.rs>
//!        tide-admit --test --tide-test-library SOURCE.rs [options] <suite.rs>
//!        tide-admit --tide-print-codes     the FX codes it decides, one per line
//!        tide-admit --tide-diff BASE HEAD  what changed between two trust artifacts
//!                                          (`interface/diff.rs`)
#![feature(rustc_private)]

extern crate rustc_abi;
extern crate rustc_ast;
extern crate rustc_codegen_ssa;
extern crate rustc_data_structures;
extern crate rustc_driver;
extern crate rustc_errors;
extern crate rustc_hir;
extern crate rustc_hir_pretty;
extern crate rustc_hir_typeck;
extern crate rustc_infer;
extern crate rustc_interface;
extern crate rustc_lint;
extern crate rustc_middle;
extern crate rustc_parse;
extern crate rustc_session;
extern crate rustc_span;
extern crate rustc_trait_selection;

mod crates;
mod diag;
mod interface;
mod invocation;
mod policy;
mod pure;
mod synthesis;
mod walk;

use std::process::ExitCode;

fn main() -> ExitCode {
    match invocation::run() {
        Ok(code) => code,
        Err(error) => {
            eprintln!("tide-admit: {error}");
            ExitCode::from(2)
        }
    }
}
