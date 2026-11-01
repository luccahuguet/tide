//! The admission allowlist (ADMIT-1): which items of `core`, `alloc`, Rust's std, and Tide's std
//! a Tide crate may use. It is versioned with the rustc pin in `rust-toolchain.toml` and reviewed
//! whenever the pin moves.
//!
//! A key is an item's canonical path in its defining crate as rustc prints it
//! (`core::f64::<impl f64>::sin`, not a re-export's path), an impl's
//! `impl <Trait> for <SelfTy>` (`impl <Trait> for fn()` when it is selected for a function
//! pointer), or one of the pseudo-items `*const T`, `*mut T`, and `fn() as integer`. Items of
//! Tide's std print under `tide_std::`, whichever crate is injected as `std`: `crates/tide-std`
//! for a program, or the conformance corpus's fixture std (`conformance/rustc/std`), which
//! includes Tide's std's crate-root items and capability types, so both key them alike. `*` in
//! a pattern matches any run of characters, `::` included, except a leading `*`, which is
//! literal: `*mut T` matches only the pseudo-item, not `impl core::fmt::Debug for &mut T`. The
//! first rule whose pattern matches decides; `walk.rs` decides keys no rule matches.

/// What a rule decides for the keys it matches.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Verdict {
    Allow,
    /// Admitted only in a package granted `unsafe` (UNSAFE-1, GLOBAL-4, CAP-3).
    Unsafe,
    /// Never admitted. Unlike a key no rule lists, a denied path to Rust's std is reported even
    /// where Tide's std does not export it (`OnceLock`, `Waker`).
    Deny,
}

#[derive(Debug)]
pub struct Rule {
    pub pattern: &'static str,
    pub verdict: Verdict,
    /// The replacement std offers, shown with the rejection.
    pub help: Option<&'static str>,
}

const fn rule(verdict: Verdict, pattern: &'static str, help: Option<&'static str>) -> Rule {
    Rule {
        pattern,
        verdict,
        help,
    }
}
const fn allow(pattern: &'static str) -> Rule {
    rule(Verdict::Allow, pattern, None)
}
const fn granted(pattern: &'static str) -> Rule {
    rule(Verdict::Unsafe, pattern, Some(GRANT))
}
const fn deny(pattern: &'static str, help: &'static str) -> Rule {
    rule(Verdict::Deny, pattern, Some(help))
}

const GRANT: &str = "this needs the package's `unsafe` grant";
const FLOAT: &str = "use `std::float` (`float::bits`, `float::min`, `float::max`, \
                     `float::total_cmp`) or `std::math` (`math::sin`, `math::exp`, ...)";
const ATOMIC: &str =
    "create atomics through Tide's std with a `&Tasks`: `AtomicU64::new(v, tasks)`";
const LAZY: &str = "build the value first and pass it down; between tasks use \
                    `Shared<Option<T>>`, within one task `OnceCell`";
const FS: &str = "use a capability method: `fs.exists(path)?`, `fs.metadata(path)?`";
const CWD: &str = "it reads the working directory; use `env.current_dir()?.join(path)`";
const OS_ERROR: &str = "an OS error code is read from a capability's result";
const ENTRY: &str = "`#[tide::main]` is the only way to start the runtime";
const RANDOM_KEY: &str = "Rust's `RandomState` draws its key from the OS; use \
                          `HashMap::with_hasher(RandomState::from_rand(&rand))`";
const BUILD_READ: &str = "a crate cannot read files or the build environment at compile time; \
                          generate a `const` instead";

/// The rules, in order. Narrow rules come before the broad ones they carve out of.
pub const RULES: &[Rule] = &[
    // ---- Tide's std's internal runtime entry points (ADMIT-1, RUN-1)
    deny("tide_std::__rt::*", ENTRY),
    // ---- address observers and raw pointers (GLOBAL-4)
    granted("*const T"),
    granted("*mut T"),
    granted("core::ptr::*"),
    granted("core::fmt::Pointer"),
    granted("impl core::fmt::Pointer for *"),
    granted("core::fmt::rt::Argument::<'_>::new_pointer"),
    // A function pointer's code address: its ordering, hashing, formatting, and integer casts
    // observe it; `==` compares identity, as `ptr::eq` does.
    allow("impl core::cmp::PartialEq* for fn()"),
    allow("impl core::cmp::Eq for fn()"),
    granted("impl * for fn()"),
    granted("fn() as integer"),
    // ---- machine code and hardware queries (UNSAFE-1)
    granted("core::arch::asm"),
    granted("core::arch::global_asm"),
    granted("core::arch::naked_asm"),
    deny("core::arch::*", "hardware intrinsics are not admitted"),
    deny("core::core_arch::*", "hardware intrinsics are not admitted"),
    // ---- synchronization without `Tasks` (CAP-3, 02 §9)
    granted("tide_std::sync::raw::*"),
    deny("core::sync::*", ATOMIC),
    deny(
        "alloc::sync::*",
        "use Tide's `std::sync::Arc`, created with a `&Tasks`",
    ),
    deny("std::sync::lazy_lock::*", LAZY),
    deny("std::sync::once_lock::*", LAZY),
    // ---- async machinery (D8)
    deny(
        "core::task::*",
        "Tide has no async; `Waker` and `Context` are not admitted",
    ),
    deny(
        "alloc::task::*",
        "Tide has no async; `Wake` is not admitted",
    ),
    // ---- float observers and math (D3, PAR-3): what reads no sign or bits stays, as do the
    // correctly rounded operations. `f*` covers `f32` and `f64`.
    allow("core::f*::<impl f*>::from_bits"),
    allow("core::f*::<impl f*>::abs"),
    allow("core::f*::<impl f*>::is_nan"),
    allow("core::f*::<impl f*>::is_finite"),
    allow("core::f*::<impl f*>::EPSILON"),
    allow("core::f*::<impl f*>::MAX"),
    allow("core::f*::<impl f*>::MIN"),
    allow("core::f*::<impl f*>::MIN_POSITIVE"),
    allow("core::f*::<impl f*>::INFINITY"),
    allow("core::f*::<impl f*>::NEG_INFINITY"),
    allow("core::f*::<impl f*>::NAN"),
    allow("core::f*::<impl f*>::RADIX"),
    allow("core::f*::<impl f*>::MANTISSA_DIGITS"),
    allow("core::f*::<impl f*>::DIGITS"),
    allow("core::f*::<impl f*>::MIN_EXP"),
    allow("core::f*::<impl f*>::MAX_EXP"),
    allow("core::f*::<impl f*>::MIN_10_EXP"),
    allow("core::f*::<impl f*>::MAX_10_EXP"),
    allow("std::f*::<impl f*>::sqrt"),
    allow("std::f*::<impl f*>::mul_add"),
    allow("std::f*::<impl f*>::floor"),
    allow("std::f*::<impl f*>::ceil"),
    allow("std::f*::<impl f*>::round"),
    allow("std::f*::<impl f*>::trunc"),
    deny("core::f*::<impl f*>::*", FLOAT),
    deny("std::f*::<impl f*>::*", FLOAT),
    // ---- compile-time reads (RUN-2)
    deny("core::macros::builtin::include*", BUILD_READ),
    deny("core::macros::builtin::env", BUILD_READ),
    deny("core::macros::builtin::option_env", BUILD_READ),
    // ---- core
    allow("core::alloc::Allocator"),
    allow("core::any::*"),
    allow("<(dyn core::any::Any*)>::*"), // `is` and `downcast_ref` on `dyn Any`
    allow("core::array::*"),
    allow("core::bool::*"),
    allow("core::borrow::*"),
    allow("core::cell::*"),
    allow("core::char::*"),
    allow("core::clone::*"),
    allow("core::cmp::*"),
    allow("core::convert::*"),
    allow("core::default::*"),
    allow("core::error::*"),
    allow("core::f32::consts::*"),
    allow("core::f64::consts::*"),
    allow("core::fmt::*"),
    allow("core::hash::*"),
    allow("core::hint::must_use"),
    // `derive(PartialEq)` expansions; both are unstable to name directly.
    allow("core::intrinsics::discriminant_value"),
    allow("core::intrinsics::unreachable"),
    // `mem::transmute`, an `unsafe` fn, which needs the grant; what it makes is `Pure` (ROOT-4).
    allow("core::intrinsics::transmute"),
    allow("core::iter::*"),
    allow("core::macros::*"), // `assert!`, `matches!`, `write!`, `format_args!`, derives
    allow("core::marker::*"),
    allow("core::io::*"),  // `io::Error` and `io::ErrorKind`
    allow("core::mem::*"), // `zeroed` is an `unsafe` fn, which needs the grant
    allow("core::num::*"), // integer methods: `checked_*`, `wrapping_*`, `saturating_*`, ...
    allow("core::ops::*"),
    allow("core::option::*"),
    allow("core::panic::*"),
    allow("core::panicking::*"),
    allow("core::result::*"),
    allow("core::slice::*"),
    allow("core::str::*"),
    allow("core::time::*"),
    // ---- alloc
    allow("alloc::alloc::Global"),
    allow("alloc::borrow::*"),
    allow("alloc::boxed::*"),
    allow("alloc::collections::*"),
    allow("alloc::fmt::*"),
    allow("alloc::intrinsics::write_box_via_move"), // `vec!` expansions
    allow("alloc::io::*"), // `io::Error::new` and `io::Error::other` (their `Pure` rule is FX0517)
    allow("alloc::macros::*"),
    allow("alloc::rc::*"),
    allow("alloc::slice::*"),
    allow("alloc::str::*"),
    allow("alloc::string::*"),
    allow("alloc::vec::*"),
    // ---- Rust's std: inherent methods of the types Tide's std re-exports
    deny("std::path::Path::exists", FS),
    deny("std::path::Path::try_exists", FS),
    deny("std::path::Path::metadata", FS),
    deny("std::path::Path::symlink_metadata", FS),
    deny("std::path::Path::canonicalize", FS),
    deny("std::path::Path::read_link", FS),
    deny("std::path::Path::read_dir", FS),
    deny("std::path::Path::is_file", FS),
    deny("std::path::Path::is_dir", FS),
    deny("std::path::Path::is_symlink", FS),
    deny("std::path::absolute", CWD),
    allow("std::path::*"),
    allow("std::ffi::os_str::*"), // `OsStr` and `OsString` do no I/O (03 §1.1)
    deny(
        "std::io::error::<impl core::io::error::Error>::last_os_error",
        OS_ERROR,
    ),
    allow("std::io::error::<impl core::io::error::Error>::from_raw_os_error"),
    // What `main` may return (reference 6.2); `ExitCode::from(u8)` is a `From` impl.
    allow("std::process::ExitCode::SUCCESS"),
    allow("std::process::ExitCode::FAILURE"),
    // `HashMap` and `HashSet` with Rust's `RandomState`: their methods, but nothing that draws
    // a random key (GLOBAL-5); Tide's std builds maps with a fixed key or a key from `Rand`.
    deny("std::hash::random::RandomState::new", RANDOM_KEY),
    deny(
        "impl core::default::Default for std::hash::random::RandomState",
        RANDOM_KEY,
    ),
    deny(
        "std::collections::hash::map::HashMap::<K, V>::*",
        RANDOM_KEY,
    ),
    deny("std::collections::hash::set::HashSet::<T>::*", RANDOM_KEY),
    // `HashMap::from([..])` and `HashSet::from([..])` build a `RandomState` inside.
    deny(
        "impl core::convert::From<*> for std::collections::hash::*",
        RANDOM_KEY,
    ),
    allow("std::collections::hash::*"),
    allow("std::hash::random::RandomState"),
];

/// Glob match where `*` matches any run of characters, `::` included; a leading `*` is literal.
pub fn matches(pattern: &str, key: &str) -> bool {
    match pattern.strip_prefix('*') {
        Some(rest) => key.strip_prefix('*').is_some_and(|key| glob(rest, key)),
        None => glob(pattern, key),
    }
}

fn glob(pattern: &str, key: &str) -> bool {
    match pattern.split_once('*') {
        None => pattern == key,
        Some((head, rest)) => key.strip_prefix(head).is_some_and(|tail| {
            tail.char_indices()
                .map(|(i, _)| i)
                .chain([tail.len()])
                .any(|i| glob(rest, &tail[i..]))
        }),
    }
}

/// The first rule whose pattern matches `key`.
pub fn lookup(key: &str) -> Option<&'static Rule> {
    RULES.iter().find(|r| matches(r.pattern, key))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn glob_spans_path_separators_and_anchors_both_ends() {
        assert!(matches(
            "core::sync::*",
            "core::sync::atomic::AtomicU64::new"
        ));
        assert!(matches(
            "core::macros::builtin::include*",
            "core::macros::builtin::include_str"
        ));
        assert!(matches(
            "impl core::fmt::Pointer for *",
            "impl core::fmt::Pointer for &T"
        ));
        assert!(matches("a*b*c", "a::b::x::c"));
        assert!(!matches("core::sync::*", "core::syncx"));
        assert!(!matches("core::mem::size_of", "core::mem::size_of_val"));
        assert!(!matches(
            "core::*::new",
            "core::sync::atomic::AtomicU64::new_in"
        ));
        assert!(matches("a::*", "a::"));
        // A leading `*` is literal, for the raw pointer pseudo-items.
        assert!(matches("*mut T", "*mut T"));
        assert!(!matches("*mut T", "impl core::fmt::Debug for &mut T"));
    }

    #[test]
    fn first_matching_rule_decides() {
        let verdict = |key| lookup(key).map(|r| r.verdict);
        // A narrow rule carves out of a broad one in either direction.
        assert_eq!(verdict("std::f64::<impl f64>::sqrt"), Some(Verdict::Allow));
        assert_eq!(verdict("std::f32::<impl f32>::sin"), Some(Verdict::Deny));
        assert_eq!(verdict("core::f32::<impl f32>::MAX"), Some(Verdict::Allow));
        assert_eq!(
            verdict("core::f64::<impl f64>::to_bits"),
            Some(Verdict::Deny)
        );
        assert_eq!(verdict("core::arch::global_asm"), Some(Verdict::Unsafe));
        assert_eq!(verdict("core::arch::asm"), Some(Verdict::Unsafe));
        assert_eq!(
            verdict("core::core_arch::x86::cpuid::__cpuid"),
            Some(Verdict::Deny)
        );
        assert_eq!(
            verdict("impl core::cmp::PartialEq for fn()"),
            Some(Verdict::Allow)
        );
        assert_eq!(
            verdict("impl core::hash::Hash for fn()"),
            Some(Verdict::Unsafe)
        );
        assert_eq!(verdict("std::path::Path::exists"), Some(Verdict::Deny));
        assert_eq!(verdict("std::path::Path::join"), Some(Verdict::Allow));
        assert_eq!(
            verdict("core::fmt::rt::Argument::<'_>::new_pointer"),
            Some(Verdict::Unsafe)
        );
        assert_eq!(
            verdict("core::fmt::rt::Argument::<'_>::new_display"),
            Some(Verdict::Allow)
        );
        assert_eq!(verdict("std::fs::read"), None);
    }
}
