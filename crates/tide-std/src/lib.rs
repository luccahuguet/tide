//! Tide's std (D18): the crate the admission driver injects as `std` (`--extern std=`), so a
//! Tide program names its items as `std::...` and cannot name Rust's std. It owns the runtime
//! behavior of every item it exposes.
//!
//! This slice (bead `fx-nkv.1`) provides:
//!
//! - the entry: `#[tide::main]`, `Root`, and the runtime entry it calls, which mints the
//!   process's one `Root` (03 ROOT-1, RUN-1);
//! - the `out` and `err` sinks: `print!`, `println!`, `eprint!`, and `eprintln!` (03 §2.4);
//! - `Pure` (reference 6.6), which the admission driver decides;
//! - the capability types `Root` carries, as identities: `Fs`, `Net`, `Clock`, `Rand`, `Env`,
//!   `Stdin`, `Proc`, and `Tasks`;
//! - Rust's std modules and macros whose API Tide keeps, as listed in `root.rs`, and the
//!   prelude's `Error` trait and parameterless Rust `#[test]`.
//!
//! Not provided yet: every capability operation and attenuation of reference 6.1 (tasks,
//! files, network, time, randomness, environment, input, processes); `Root`'s `log` field;
//! the `dbg`, `log`, and `trace` sinks and `io::stdout()`/`io::stderr()`; `Secret`;
//! `io`'s and `time`'s Rust items (`Read`, `Write`, `io::Error`, `Duration`, ...);
//! collections, `sync`, `par`, `arena`, `ptr`, `float`, `math`, and `sim`; property tests and
//! `#[sim_test]`; the contract attributes and the `Error`, `Encode`, and `Decode` derives. A
//! program that uses one fails to compile, as for any missing item.
//! Executable tests use rustc's ordinary harness, without simulated capabilities or process
//! isolation. They are for reviewed/trusted demos only, not RUN-5's unreviewed-branch guarantee.
//!
//! `root.rs` holds the crate-root items, which the conformance fixture std includes too; each
//! module below declares its capability type with `root.rs`'s `capability!`, as the fixture's
//! modules of the same names do.

include!("root.rs");

crate::declare_prelude! {
    pub use core::prelude::v1::test;
}

pub mod fs {
    crate::capability!(
        /// File-system authority. It has no operations yet.
        Fs
    );
}

pub mod net {
    crate::capability!(
        /// Network authority. It has no operations yet: a program can declare that it holds
        /// `Net`, but cannot use it.
        Net
    );
}

pub mod time {
    crate::capability!(
        /// Authority to read time and sleep. It has no operations yet.
        Clock
    );
}

pub mod rand {
    crate::capability!(
        /// Authority to draw randomness. It has no operations yet.
        Rand
    );
}

pub mod env {
    crate::capability!(
        /// Authority to read the arguments and the environment. It has no operations yet.
        Env
    );
}

pub mod io {
    crate::capability!(
        /// Authority to read standard input. It has no operations yet.
        Stdin
    );
}

pub mod process {
    pub use std::process::{ExitCode, Termination};

    crate::capability!(
        /// Authority to run processes and exit. It has no operations yet.
        Proc
    );
}

pub mod task {
    crate::capability!(
        /// Authority to run tasks and create synchronization handles. It has no operations yet.
        Tasks
    );
}
