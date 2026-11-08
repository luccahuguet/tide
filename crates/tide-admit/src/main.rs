//! tide-admit: rustc with Tide's admission gate (D18, architecture A).
//!
//! It constructs rustc's arguments from a fixed check/build interface, always passes rustc
//! `-Zallow-features=` so the crate cannot enable unstable features and `-Zpolonius=no` so
//! borrows get NLL's verdicts (ADMIT-2, ORIGIN-6), and checks every use the crate makes against
//! the admission allowlist (ADMIT-1), reporting each rejection as `FX0520` at the use's span.
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
//!
//! This is a partial build path for reviewed/trusted demo code only: the operator must review
//! source, grants and dependencies before building/running, including granted assembly and native
//! link inputs. There is no assembler/linker OS sandbox, simulated test Root, test-process
//! isolation, snapshot execution gate, or RUN-5 coverage verdict. Raw Rust `#[test]` is a temporary
//! executable-test path, not `tide test`/`tide sim` or the RUN-1 unreviewed-branch guarantee.
//!
//! Usage: tide-admit [--tide-grant=unsafe] [--tide-std PATH] [--tide-dep-dir DIR] <source.rs>
//!        tide-admit --tide-print-codes     the FX codes it decides, one per line
//!        tide-admit --tide-diff BASE HEAD  what changed between two trust artifacts
//!                                          (`interface/diff.rs`)
#![feature(rustc_private)]

extern crate rustc_abi;
extern crate rustc_ast;
extern crate rustc_codegen_ssa;
extern crate rustc_data_structures;
extern crate rustc_driver;
extern crate rustc_hir;
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
