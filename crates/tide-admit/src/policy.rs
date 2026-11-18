//! The admission allowlist (ADMIT-1): which items of `core`, `alloc`, Rust's std, and Tide's std
//! a Tide crate may use. It is versioned with the rustc pin in `rust-toolchain.toml` and reviewed
//! whenever the pin moves.
//!
//! A key is an item's canonical path in its defining crate as rustc prints it
//! (`core::f64::<impl f64>::sin`, not a re-export's path), an impl's
//! `impl <Trait> for <SelfTy>` (`impl <Trait> for fn()` when it is selected for a function
//! pointer), or the pseudo-items `*const T`, `*mut T`, `fn() as integer`, and `thread_rng`. Items of
//! Tide's std print under `tide_std::`, whichever crate is injected as `std`: `crates/tide-std`
//! for a program, or the conformance corpus's fixture std (`conformance/rustc/std`), which
//! includes Tide's std's crate-root items and capability types, so both key them alike. `*` in
//! a pattern matches any run of characters, `::` included, except a leading `*`, which is
//! literal: `*mut T` matches only the pseudo-item, not `impl core::fmt::Debug for &mut T`. The
//! first rule whose pattern matches decides; `walk.rs` decides keys no rule matches.

use crate::diag::Code;

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
    /// The rule-specific rejection; ordinary admission rules use TD0520.
    pub code: Code,
    /// The replacement std offers, shown with the rejection.
    pub help: Option<&'static str>,
}

const fn rule(verdict: Verdict, pattern: &'static str, help: Option<&'static str>) -> Rule {
    Rule {
        pattern,
        verdict,
        code: Code::Td0520,
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

const fn deny_as(pattern: &'static str, code: Code, help: &'static str) -> Rule {
    Rule {
        code,
        ..deny(pattern, help)
    }
}

const fn ambient(pattern: &'static str) -> Rule {
    Rule {
        code: Code::Td0501,
        ..deny(
            pattern,
            "use a capability in scope, or thread that capability through the parameters",
        )
    }
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
const SPAWN: &str = "spawn into `tasks.scope` instead";
const ENTRY: &str = "`#[tide::main]` is the only way to start the runtime";
const RANDOM_KEY: &str = "Rust's `RandomState` draws its key from the OS; use \
                          `HashMap::with_hasher(RandomState::from_rand(&rand))`";
const BUILD_READ: &str = "a crate cannot read files or the build environment at compile time; \
                          generate a `const` instead";

/// The rules, in order. Narrow rules come before the broad ones they carve out of.
pub const RULES: &[Rule] = &[
    // ---- Tide's std's internal runtime entry points (ADMIT-1, RUN-1)
    deny("tide_std::__rt::*", ENTRY),
    // ---- TCP's plain address/shutdown data (fx-hxw); no resolver or socket constructor.
    allow("core::net::*"),
    allow("std::net::Shutdown::*"),
    // ---- ambient world operations (CAP-1, reference 6.9); Rust items only.
    ambient("std::fs::read"),
    ambient("std::fs::read_to_string"),
    ambient("std::fs::write"),
    ambient("std::fs::copy"),
    ambient("std::fs::rename"),
    ambient("std::fs::hard_link"),
    ambient("std::fs::soft_link"),
    ambient("std::fs::canonicalize"),
    ambient("std::fs::metadata"),
    ambient("std::fs::symlink_metadata"),
    ambient("std::fs::read_link"),
    ambient("std::fs::read_dir"),
    ambient("std::fs::create_dir"),
    ambient("std::fs::create_dir_all"),
    ambient("std::fs::remove_file"),
    ambient("std::fs::remove_dir"),
    ambient("std::fs::remove_dir_all"),
    ambient("std::fs::set_permissions"),
    ambient("std::fs::exists"),
    ambient("std::fs::File::open"),
    ambient("std::fs::File::create"),
    ambient("std::fs::File::create_new"),
    ambient("std::fs::File::open_buffered"),
    ambient("std::fs::File::create_buffered"),
    ambient("std::fs::OpenOptions::open"),
    ambient("std::fs::DirBuilder::create"),
    ambient("std::env::args"),
    ambient("std::env::args_os"),
    ambient("std::env::var"),
    ambient("std::env::var_os"),
    ambient("std::env::vars"),
    ambient("std::env::vars_os"),
    ambient("std::env::current_dir"),
    ambient("std::env::current_exe"),
    ambient("std::env::home_dir"),
    ambient("std::env::temp_dir"),
    ambient("std::time::Instant::now"),
    ambient("std::time::Instant::elapsed"),
    ambient("std::time::SystemTime::now"),
    ambient("std::time::SystemTime::elapsed"),
    ambient("std::thread::functions::sleep"),
    ambient("std::thread::functions::sleep_until"),
    ambient("std::thread::functions::sleep_ms"),
    ambient("std::thread::functions::park_timeout"),
    ambient("std::thread::functions::park_timeout_ms"),
    ambient("std::process::exit"),
    ambient("std::process::abort"),
    ambient("std::process::Command::new"),
    ambient("std::io::stdio::stdin"),
    ambient("std::sync::mpsc::Receiver::<T>::recv_timeout"),
    ambient("std::sync::mpmc::Receiver::<T>::recv_timeout"),
    ambient("std::net::tcp::TcpListener::bind"),
    ambient("std::net::tcp::TcpStream::connect"),
    ambient("std::net::tcp::TcpStream::connect_timeout"),
    ambient("std::net::udp::UdpSocket::bind"),
    // thread_rng is external to std; the walk uses this pseudo-key for that name.
    ambient("thread_rng"),
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
    // Unresolved paths name the final item: type denials must be module globs
    // (e.g. std::sync::lazy_lock::*); an exact type deny misses Type::assoc.
    deny("core::sync::*", ATOMIC),
    deny(
        "alloc::sync::*",
        "use Tide's `std::sync::Arc`, created with a `&Tasks`",
    ),
    deny("std::sync::lazy_lock::*", LAZY),
    deny("std::sync::once_lock::*", LAZY),
    // ---- concurrency and panic habits (D8, D5); defining paths on the pinned rustc.
    // Tokio is a pseudo-key shared by resolved and recovered uses.
    deny_as("tokio::spawn", Code::Td0602, SPAWN),
    deny_as(
        "std::thread::functions::spawn",
        Code::Td0602,
        SPAWN,
    ),
    deny_as(
        "std::thread::builder::*",
        Code::Td0602,
        SPAWN,
    ),
    deny_as(
        "std::thread::scoped::*",
        Code::Td0602,
        "use `tasks.scope` instead of Rust thread scopes",
    ),
    deny_as(
        "std::sync::poison::mutex::*",
        Code::Td0604,
        "use `Shared::with` instead of a Mutex guard",
    ),
    deny_as(
        "std::sync::poison::rwlock::*",
        Code::Td0604,
        "use `Shared::with` instead of a RwLock guard",
    ),
    deny_as(
        "std::panic::catch_unwind",
        Code::Td0701,
        "use `tasks.spawn_isolated` instead",
    ),
    // ---- async machinery (D8); core::task keeps its admission code.
    deny_as(
        "core::future::*",
        Code::Td0601,
        "use ordinary functions and structured tasks; remove async/await",
    ),
    deny_as(
        "core::pin::*",
        Code::Td0601,
        "use ordinary owned or borrowed values instead of Pin",
    ),
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
    // ---- std::io: Tide's std owns `io::Error`, `ErrorKind`, and the traits (03 §1.1), which
    // are admitted as its own items, and re-exports Rust's `Cursor` and `SeekFrom`.
    deny("tide_std::io::Error::last_os_error", OS_ERROR),
    // The Cursor and SeekFrom type keys and std::macros::panic serve the Rust-std checks;
    // Tide re-exports or replaces them in programs. The method globs matter in programs.
    allow("core::io::cursor::Cursor"),
    allow("core::io::cursor::Cursor::*"),
    allow("core::io::seek::SeekFrom"),
    allow("core::io::seek::SeekFrom::*"),
    // ---- core
    allow("core::alloc::Allocator"),
    allow("core::any::*"),
    allow("<(dyn core::any::Any*)>::*"), // `is` and `downcast_ref` on `dyn Any`
    allow("core::array::*"),
    allow("core::ascii::*"),
    allow("core::bool::*"),
    allow("core::borrow::*"),
    allow("core::cell::*"),
    allow("core::char::*"),
    allow("core::clone::*"),
    allow("core::cmp::*"),
    allow("core::convert::*"),
    allow("core::default::*"),
    allow("core::error::*"),
    allow("<(dyn core::error::Error*)>::*"), // Error downcasts, as Rust's trait API
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
    allow("alloc::macros::*"),
    allow("alloc::rc::*"),
    allow("alloc::slice::*"),
    allow("alloc::str::*"),
    allow("alloc::string::*"),
    allow("alloc::vec::*"),
    // Rust's `panic!` entry expands to core's panicking support (reference §2.1).
    allow("std::macros::panic"),
    // ---- Rust's std: inherent methods of the types Tide's std re-exports
    // Channel try-errors are data-only Rust enums retained by Tide's mpsc API.
    allow("std::sync::mpsc::TryRecvError::*"),
    allow("std::sync::mpsc::TrySendError::*"),
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
    // `std::path` item by item (Achilles W5, fx-qlek.5): an item a later rustc adds, such as
    // another I/O method, is not admitted until the pin review lists it here.
    allow("std::path::Path"),
    allow("std::path::Path::new"),
    allow("std::path::Path::as_os_str"),
    allow("std::path::Path::as_mut_os_str"),
    allow("std::path::Path::to_str"),
    allow("std::path::Path::to_string_lossy"),
    allow("std::path::Path::to_path_buf"),
    allow("std::path::Path::is_absolute"),
    allow("std::path::Path::is_relative"),
    allow("std::path::Path::has_root"),
    allow("std::path::Path::parent"),
    allow("std::path::Path::ancestors"),
    allow("std::path::Path::file_name"),
    allow("std::path::Path::strip_prefix"),
    allow("std::path::Path::starts_with"),
    allow("std::path::Path::ends_with"),
    allow("std::path::Path::file_stem"),
    allow("std::path::Path::file_prefix"),
    allow("std::path::Path::extension"),
    allow("std::path::Path::join"),
    allow("std::path::Path::with_file_name"),
    allow("std::path::Path::with_extension"),
    allow("std::path::Path::with_added_extension"),
    allow("std::path::Path::components"),
    allow("std::path::Path::iter"),
    allow("std::path::Path::display"),
    allow("std::path::Path::into_path_buf"),
    allow("std::path::Path::is_empty"),
    allow("std::path::PathBuf"),
    allow("std::path::PathBuf::new"),
    allow("std::path::PathBuf::with_capacity"),
    allow("std::path::PathBuf::as_path"),
    allow("std::path::PathBuf::leak"),
    allow("std::path::PathBuf::push"),
    allow("std::path::PathBuf::pop"),
    allow("std::path::PathBuf::set_file_name"),
    allow("std::path::PathBuf::set_extension"),
    allow("std::path::PathBuf::add_extension"),
    allow("std::path::PathBuf::as_mut_os_string"),
    allow("std::path::PathBuf::into_os_string"),
    allow("std::path::PathBuf::into_string"),
    allow("std::path::PathBuf::into_boxed_path"),
    allow("std::path::PathBuf::capacity"),
    allow("std::path::PathBuf::clear"),
    allow("std::path::PathBuf::reserve"),
    allow("std::path::PathBuf::try_reserve"),
    allow("std::path::PathBuf::reserve_exact"),
    allow("std::path::PathBuf::try_reserve_exact"),
    allow("std::path::PathBuf::shrink_to_fit"),
    allow("std::path::PathBuf::shrink_to"),
    allow("std::path::Ancestors"),
    allow("std::path::Display"),
    allow("std::path::StripPrefixError"),
    allow("std::path::Component"),
    allow("std::path::Component::Prefix"),
    allow("std::path::Component::RootDir"),
    allow("std::path::Component::CurDir"),
    allow("std::path::Component::ParentDir"),
    allow("std::path::Component::Normal"),
    allow("std::path::Component*::as_os_str"),
    allow("std::path::Components"),
    allow("std::path::Components*::as_path"),
    allow("std::path::Iter"),
    allow("std::path::Iter*::as_path"),
    allow("std::path::Prefix"),
    allow("std::path::Prefix::Verbatim"),
    allow("std::path::Prefix::VerbatimUNC"),
    allow("std::path::Prefix::VerbatimDisk"),
    allow("std::path::Prefix::DeviceNS"),
    allow("std::path::Prefix::UNC"),
    allow("std::path::Prefix::Disk"),
    allow("std::path::Prefix*::is_verbatim"),
    allow("std::path::PrefixComponent"),
    allow("std::path::PrefixComponent*::kind"),
    allow("std::path::PrefixComponent*::as_os_str"),
    allow("std::path::MAIN_SEPARATOR"),
    allow("std::path::MAIN_SEPARATOR_STR"),
    allow("std::path::is_separator"),
    allow("std::env::VarError::*"), // data-only errors from Env, never ambient reads
    allow("std::ffi::os_str::*"), // `OsStr` and `OsString` do no I/O (03 §1.1)
    // What `main` may return (reference 6.2); `ExitCode::from(u8)` is a `From` impl.
    allow("std::process::ExitCode::SUCCESS"),
    allow("std::process::ExitCode::FAILURE"),
    // A child's status, re-exported by `tide_std::process` (research/40 §2): data observers only.
    allow("std::process::ExitStatus::code"),
    allow("std::process::ExitStatus::success"),
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
    allow("std::hash::random::DefaultHasher"),
    allow("std::hash::random::DefaultHasher::*"),
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

    #[allow(dead_code)] // This consumer checks the keys; integration tests use source().
    mod reference_allowlist {
        include!("../tests/support/reference_allowlist.rs");
    }

    #[test]
    fn reference_kept_families_have_allow_rules() {
        for (family, keys, _) in reference_allowlist::ITEMS {
            assert!(!keys.is_empty(), "{family} has no allowlist witness");
            for key in *keys {
                assert_eq!(
                    lookup(key).map(|rule| rule.verdict),
                    Some(Verdict::Allow),
                    "{family}: {key}"
                );
            }
        }
    }

    /// Achilles W5 (fx-qlek.5): `std::path` is listed item by item, so a method or function a
    /// later rustc adds there matches no rule, which `walk.rs` rejects for Rust's std.
    #[test]
    fn std_path_admits_listed_items_only() {
        for key in ["std::path::Path::join", "std::path::PathBuf::push", "std::path::Component::Normal"] {
            assert_eq!(lookup(key).map(|r| r.verdict), Some(Verdict::Allow), "{key}");
        }
        for key in ["std::path::Path::exists", "std::path::absolute"] {
            assert_eq!(lookup(key).map(|r| r.verdict), Some(Verdict::Deny), "{key}");
        }
        for key in ["std::path::Path::normalize_lexically", "std::path::Path::future_io", "std::path::future"] {
            assert!(lookup(key).is_none(), "{key}");
        }
    }

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
    fn ambient_rules_leave_existing_owners_and_capability_methods_alone() {
        assert_eq!(lookup("std::fs::read").unwrap().code, Code::Td0501);
        assert_eq!(
            lookup("std::time::Instant::elapsed").unwrap().code,
            Code::Td0501
        );
        assert_eq!(
            lookup("std::net::tcp::TcpStream::connect").unwrap().code,
            Code::Td0501
        );
        for key in [
            "std::path::Path::exists",
            "tide_std::io::Error::last_os_error",
            "core::sync::atomic::AtomicU64",
        ] {
            assert_eq!(lookup(key).unwrap().code, Code::Td0520);
        }
        for key in [
            "tide_std::net::Net::bind",
            "tide_std::net::Net::connect",
            "tide_std::fs::Fs::read",
            "tide_std::time::Clock::now",
        ] {
            assert!(lookup(key).is_none());
        }
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
        assert_eq!(
            verdict("std::sync::mpsc::TryRecvError::Empty"),
            Some(Verdict::Allow)
        );
        assert_eq!(
            verdict("std::sync::mpsc::TrySendError::Full"),
            Some(Verdict::Allow)
        );
        assert_eq!(verdict("std::sync::mpsc::channel"), None);
        assert_eq!(verdict("std::fs::read"), Some(Verdict::Deny));
        assert_eq!(verdict("std::macros::panic"), Some(Verdict::Allow));
        assert_eq!(verdict("core::io::cursor::Cursor"), Some(Verdict::Allow));
        assert_eq!(verdict("core::io::seek::SeekFrom"), Some(Verdict::Allow));
        assert_eq!(verdict("core::ascii::EscapeDefault"), Some(Verdict::Allow));
    }
}
