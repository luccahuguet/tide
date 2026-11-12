// The crate-root items that Tide's std shares with the conformance corpus's fixture std
// (`conformance/rustc/std`). Both crates `include!` this file at their root, so each item here
// has one definition and the same path in both, and the admission driver keys it alike
// (`tide_std::Root`). An included file cannot carry inner attributes or `//!` docs, and a
// `#[macro_export]` macro it defines cannot be named by path, so its macros come from
// `tide-macros`.

// Shared with the conformance std, like the crate-root items below.
pub mod collections;

// Rust's std modules and macros whose API Tide keeps (reference 1, 2.1).
pub use std::{
    any, array, borrow, boxed, cell, char, clone, cmp, convert, default, error, f32, f64, fmt,
    hash, iter, mem, num, ops, option, path, rc, result, slice, str, string, vec,
};
pub use std::{
    assert_eq, assert_ne, debug_assert, format, matches, todo, unimplemented, unreachable, write,
    writeln,
};

/// Authority-free names used by paths and directory entries (03 §1.1).
pub mod ffi {
    pub use std::ffi::{OsStr, OsString};
}

// `panic!` alone, since re-exporting `std::panic` would also export the module and
// `catch_unwind`; `print!` and its kin write to the sinks below.
pub use tide_macros::{Decode, Encode, Error, eprint, eprintln, panic, print, println};

/// The `Encode` and `Decode` traits, which share their derives' names, and the engine those
/// derives call; shared by production and conformance.
#[doc(hidden)]
#[path = "codec.rs"]
pub mod __codec;
pub use __codec::{Decode, Encode};
pub mod json;

/// Support for the toolchain Error derive; shared by production and conformance.
#[doc(hidden)]
pub mod __error {
    include!("error.rs");
}

/// The sinks that `print!` and its kin write to (03 §2.4). Every call evaluates the arguments and
/// formats the whole record before writing any of it (SINK-3), so the destination never decides
/// which formatting impls run, and a panic in one propagates with nothing written. The record is
/// buffered inside a parallel branch, then appended in branch order; otherwise it is written
/// whole under its stream's lock and flushed, so records keep program order, a
/// record a formatting impl prints comes before the record it is part of, and none waits in a
/// buffer at exit. As in Rust, a formatting impl that returns an error panics. A sink returns `()`
/// and never fails (SINK-1): when stdout is a closed pipe, the process exits quietly with status
/// 141, as under the default `SIGPIPE` action, and any other record that cannot be written is
/// discarded (SINK-5).
/// Under simulation, the formatted record instead enters the ordered sink trace without a
/// scheduling point or host write.
#[doc(hidden)]
pub mod __sink {
    use std::cell::RefCell;
    use std::fmt::{self, Arguments};
    use std::io::{self, Write};

    pub(crate) struct Record {
        pub(crate) stream: &'static str,
        pub(crate) message: String,
        span: &'static std::panic::Location<'static>,
    }
    thread_local! {
        static BUFFER: RefCell<Option<Vec<Record>>> = const { RefCell::new(None) };
    }

    pub(crate) fn capture<T>(f: impl FnOnce() -> T) -> (std::thread::Result<T>, Vec<Record>) {
        let previous = BUFFER.with(|slot| slot.replace(Some(Vec::new())));
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(f));
        let records = BUFFER.with(|slot| slot.replace(previous)).unwrap();
        (result, records)
    }

    pub(crate) fn append(records: Vec<Record>) {
        for record in records {
            emit(record);
        }
    }

    fn emit(record: Record) {
        let record = BUFFER.with(|slot| {
            if let Some(buffer) = slot.borrow_mut().as_mut() {
                buffer.push(record);
                None
            } else {
                Some(record)
            }
        });
        let Some(record) = record else {
            return;
        };
        if crate::task::record_sink(record.stream, &record.message, record.span) {
            return;
        }
        if record.stream == "out" {
            if let Err(e) = write(io::stdout().lock(), &record.message)
                && e.kind() == io::ErrorKind::BrokenPipe
            {
                std::process::exit(141);
            }
        } else {
            let _ = write(io::stderr().lock(), &record.message);
        }
    }

    #[track_caller]
    pub fn out(args: Arguments<'_>, newline: bool) {
        emit(Record {
            stream: "out",
            message: format(args, newline),
            span: std::panic::Location::caller(),
        });
    }

    #[track_caller]
    pub fn err(args: Arguments<'_>, newline: bool) {
        emit(Record {
            stream: "err",
            message: format(args, newline),
            span: std::panic::Location::caller(),
        });
    }

    /// The record; `fmt::format` panics when a formatting impl returns an error.
    fn format(args: Arguments<'_>, newline: bool) -> String {
        let mut record = fmt::format(args);
        if newline {
            record.push('\n');
        }
        record
    }

    fn write(mut w: impl Write, record: &str) -> io::Result<()> {
        w.write_all(record.as_bytes())?;
        w.flush()
    }
}

/// Declares `prelude::rust_2024`, which std injection glob-imports into every crate: Rust's 2024
/// prelude without the builtin macros Tide drops (`env!`, `include!`, `concat!`, `test`, ...),
/// Tide's additions defined here, and `$extra`.
macro_rules! declare_prelude {
    ($($extra:item)*) => {
        pub mod prelude {
            pub mod rust_2024 {
                pub use std::prelude::rust_2024::{
                    AsMut, AsRef, Box, Clone, Copy, Debug, Default, DoubleEndedIterator, Drop, Eq,
                    Err, ExactSizeIterator, Extend, Fn, FnMut, FnOnce, From, FromIterator, Hash,
                    Into, IntoIterator, Iterator, None, Ok, Option, Ord, PartialEq, PartialOrd,
                    Result, Send, Sized, Some, String, Sync, ToOwned, ToString, TryFrom, TryInto,
                    Unpin, Vec, assert, cfg, derive, drop, format_args, stringify,
                };
                pub use crate::{
                    Decode, Encode, Error, assert_eq, assert_ne, debug_assert, eprint, eprintln, format,
                    matches, panic, print, println, todo, unimplemented, unreachable, vec, write,
                    writeln,
                };
                pub use std::error::Error;
                pub use crate::Root;
                pub use crate::marker::Pure;
                pub use crate::par::ParSlice;
                pub use crate::tide;
                $($extra)*
            }
        }
    };
}
pub(crate) use declare_prelude;

/// Toolchain attributes spelled with a path: `#[tide::main]`.
pub mod tide {
    pub use tide_macros::main;
}

pub mod marker {
    pub use std::marker::*;

    /// No authority is reachable from the type (reference 6.6). The admission driver decides it
    /// structurally wherever a type meets a `Pure` bound; the blanket impl makes rustc accept
    /// every bound.
    ///
    /// # Safety
    ///
    /// Nothing implements it by hand: only the toolchain decides purity.
    pub unsafe trait Pure {}
    unsafe impl<T: ?Sized> Pure for T {}
}

/// Declares a capability type (reference 6.1): `Clone`, with the same authority, and
/// `Send + Sync`, but not `Pure`. Its private field keeps user code from making one from data
/// (ROOT-4): no struct literal, `Default`, or decoding builds it, so only this crate does.
macro_rules! capability {
    ($(#[$attr:meta])* $name:ident) => {
        $(#[$attr])*
        #[derive(Clone)]
        // The handle distinguishes simulated identities before their operations are modeled.
        pub struct $name(#[allow(dead_code)] pub(crate) Option<crate::task::Tasks>);
    };
}
pub(crate) use capability;

/// Every capability a program starts with (reference 6.2), which the entry that `#[tide::main]`
/// generates hands to `main`. The private field keeps user code from building one (ROOT-4). The
/// reference's `log` field, sink configuration (SINK-2), is not provided: no document fixes its
/// type.
pub struct Root {
    _private: (),
    pub fs: fs::Fs,
    pub net: net::Net,
    pub clock: time::Clock,
    pub rand: rand::Rand,
    pub env: env::Env,
    pub stdin: io::Stdin,
    pub proc: process::Proc,
    pub tasks: task::Tasks,
}

/// The runtime entry. Only the `main` that `#[tide::main]` generates may name it: the admission
/// driver rejects any other use (ADMIT-1, `FX0520`).
#[doc(hidden)]
pub mod __rt {
    use std::process::{ExitCode, Termination};
    use std::sync::atomic::{AtomicBool, Ordering};

    static ENTERED: AtomicBool = AtomicBool::new(false);

    /// Mints the process's one `Root`, runs `main` with it, and reports what `main` returns as
    /// Rust's entry point does: `Err(e)` prints `Error: {e:?}` to stderr and exits with 1, and a
    /// panic exits with 101. A second call in the same process panics before minting.
    pub fn enter<R: Termination>(main: impl FnOnce(crate::Root) -> R) -> ExitCode {
        assert!(
            !ENTERED.swap(true, Ordering::SeqCst),
            "the runtime entry runs once per process"
        );
        let root = crate::Root {
            _private: (),
            fs: crate::fs::Fs::root(),
            net: crate::net::Net(None),
            clock: crate::time::Clock::new(crate::task::Time::real()),
            rand: crate::rand::Rand(None),
            env: crate::env::Env(None),
            stdin: crate::io::Stdin(None),
            proc: crate::process::Proc(None),
            tasks: crate::task::Tasks::real(),
        };
        main(root).report()
    }
}

impl Root {
    /// Starts a seeded simulation on this test's calling thread (task 0).
    /// Dropping the returned root ends it. Nested simulations are rejected.
    #[track_caller]
    pub fn sim(seed: u64) -> crate::sim::SimRoot {
        crate::sim::SimRoot::new(seed)
    }
}
