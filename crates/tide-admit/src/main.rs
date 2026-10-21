//! tide-admit: rustc with Tide's admission gate (D18, architecture A).
//!
//! It takes rustc's arguments plus its own `--tide-*` flags, always passes rustc
//! `-Zallow-features=` so the crate cannot enable unstable features and `-Zpolonius=no` so
//! borrows get NLL's verdicts (ADMIT-2, ORIGIN-6), and checks every use the crate makes against
//! the admission allowlist (ADMIT-1), reporting each rejection as `FX0520` at the use's span. It
//! checks after name resolution and before rustc's analysis, so a crate with rustc errors still
//! gets its admission verdicts.
//!
//! Usage: tide-admit [--tide-grant=unsafe] <rustc args...>
//!        tide-admit --tide-print-codes     the FX codes it decides, one per line
#![feature(rustc_private)]

extern crate rustc_data_structures;
extern crate rustc_driver;
extern crate rustc_hir;
extern crate rustc_infer;
extern crate rustc_interface;
extern crate rustc_middle;
extern crate rustc_span;
extern crate rustc_trait_selection;

mod crates;
mod diag;
mod policy;
mod walk;

use std::process::ExitCode;

use rustc_driver::{Callbacks, Compilation};
use rustc_interface::interface::Compiler;
use rustc_middle::ty::TyCtxt;

use crate::walk::Grants;

struct Admission {
    grants: Grants,
}

impl Callbacks for Admission {
    fn after_expansion<'tcx>(&mut self, _: &Compiler, tcx: TyCtxt<'tcx>) -> Compilation {
        walk::check_crate(tcx, self.grants);
        Compilation::Continue
    }
}

fn main() -> ExitCode {
    let mut grants = Grants::default();
    let mut args = Vec::new();
    for arg in std::env::args_os() {
        let Ok(arg) = arg.into_string() else {
            eprintln!("tide-admit: an argument is not valid UTF-8");
            return ExitCode::from(2);
        };
        match arg.strip_prefix("--tide-") {
            None => args.push(arg),
            Some("grant=unsafe") => grants.unsafe_ = true,
            Some("print-codes") => {
                for code in diag::Code::ALL {
                    println!("{}", code.as_str());
                }
                return ExitCode::SUCCESS;
            }
            Some(_) => {
                eprintln!("tide-admit: unknown flag `{arg}`");
                return ExitCode::from(2);
            }
        }
    }
    // Last, so they override earlier ones (ADMIT-2). The pinned nightly borrow-checks with
    // Polonius by default; v0 has NLL's verdicts (`design/02-memory.md`, ORIGIN-6).
    args.push("-Zallow-features=".into());
    args.push("-Zpolonius=no".into());
    rustc_driver::install_ice_hook("the tide-admit driver (crates/tide-admit)", |_| ());
    let mut callbacks = Admission { grants };
    rustc_driver::catch_with_exit_code(|| rustc_driver::run_compiler(&args, &mut callbacks))
}
