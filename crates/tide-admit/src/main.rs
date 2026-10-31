//! tide-admit: rustc with Tide's admission gate (D18, architecture A).
//!
//! It constructs rustc's arguments from a fixed metadata-check interface, always passes rustc
//! `-Zallow-features=` so the crate cannot enable unstable features and `-Zpolonius=no` so
//! borrows get NLL's verdicts (ADMIT-2, ORIGIN-6), and checks every use the crate makes against
//! the admission allowlist (ADMIT-1), reporting each rejection as `FX0520` at the use's span.
//! It also decides `Pure` (`pure.rs`) and the rules on items that need it or the `unsafe` grant,
//! each under its own FX code (D18). It checks after name resolution and before rustc's
//! analysis, so a crate with rustc errors still gets its verdicts, and after the analysis, when
//! nothing failed, checks the instances that can run (`walk/mono.rs`).
//!
//! Usage: tide-admit [--tide-grant=unsafe] [--tide-std PATH] [--tide-dep-dir DIR] <source.rs>
//!        tide-admit --tide-print-codes     the FX codes it decides, one per line
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
mod invocation;
mod policy;
mod pure;
mod walk;

use std::process::ExitCode;

fn main() -> ExitCode {
    rustc_driver::install_ice_hook("the tide-admit driver (crates/tide-admit)", |_| ());
    match invocation::run() {
        Ok(code) => code,
        Err(error) => {
            eprintln!("tide-admit: {error}");
            ExitCode::from(2)
        }
    }
}
