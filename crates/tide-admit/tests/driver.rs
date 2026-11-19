//! The driver as a crate's build sees it: its exit status and its diagnostics. These run
//! against Rust's own std, which the allowlist checks like `core`; the conformance runner
//! (`crates/tide-conformance`) runs the corpus against Tide's std.

use std::path::PathBuf;
use std::process::Command;

/// Checks `source` as a library with the driver, unless `flags` say otherwise; returns its exit
/// code and short-format stderr.
fn admit(name: &str, source: &str, flags: &[&str]) -> (i32, String) {
    let dir = PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join(name);
    let package = dir.join("package");
    std::fs::create_dir_all(&package).unwrap();
    let file = package.join("lib.rs");
    std::fs::write(&file, source).unwrap();
    let out = Command::new(env!("CARGO_BIN_EXE_tide-admit"))
        .args([
            "--edition",
            "2024",
            "--crate-type",
            "lib",
            "--emit",
            "metadata",
            "--error-format=short",
        ])
        .args(flags)
        .arg("--out-dir")
        .arg(&dir)
        .arg(&file)
        .output()
        .unwrap();
    let stderr = String::from_utf8(out.stderr)
        .unwrap()
        .replace(&format!("{}/", package.display()), "");
    (out.status.code().unwrap(), stderr)
}

fn td_lines(stderr: &str) -> Vec<&str> {
    stderr
        .lines()
        .filter(|l| l.contains(": error: TD"))
        .collect()
}

mod reference_allowlist {
    include!("support/reference_allowlist.rs");
}

#[test]
fn reference_kept_families_pass_the_allowlist_without_reexport_bypasses() {
    let (code, stderr) = admit("reference-allowlist", &reference_allowlist::source(), &[]);
    assert_eq!(code, 0, "{stderr}");
    assert!(td_lines(&stderr).is_empty(), "{stderr}");
}

#[test]
fn the_backstop_keeps_exported_library_roots_in_every_profile() {
    for (name, source) in [
        (
            "backstop-exported-vtable",
            "pub fn leaf(x: &fn()) -> &dyn PartialOrd<fn()> { x }\n",
        ),
        (
            "backstop-exported-reify",
            // A generic std function without references, so TD0206 (CLOSURE-3) has no bound
            // to reject and the backstop alone finds `Ord for fn()`.
            "pub fn chooser() -> fn(fn(), fn()) -> fn() { std::cmp::max }\n",
        ),
    ] {
        for profile in ["debug", "release"] {
            let (code, stderr) = admit(
                name,
                source,
                &["--tide-test-backstop", &format!("--tide-profile={profile}")],
            );
            assert_eq!(code, 1, "{name}/{profile}: {stderr}");
            assert!(
                td_lines(&stderr)
                    .iter()
                    .any(|line| line.contains("TD0520:") && line.contains("for fn()`")),
                "{name}/{profile}: {stderr}"
            );
        }
    }
}

#[test]
fn rejects_each_use_outside_the_allowlist_at_its_span() {
    let source = "\
pub fn run(x: f64) -> f64 {
    let v: Vec<u32> = (1..4).map(|n| n * 2).collect();
    let _c = core::sync::atomic::AtomicU64::new(0);
    let s = format!(\"{v:?}\");
    x.sqrt() + x.sin() + f64::from(u32::try_from(s.len()).unwrap())
}
mod sub;
";
    // The same use on the same line of another file is reported there too.
    let dir = PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join("reject");
    let sub = "\n\npub fn f() -> u64 { core::sync::atomic::AtomicU64::new(0).into_inner() }\n";
    std::fs::create_dir_all(dir.join("package")).unwrap();
    std::fs::write(dir.join("package/sub.rs"), sub).unwrap();
    let (code, stderr) = admit("reject", source, &[]);
    assert_eq!(code, 1, "{stderr}");
    assert_eq!(
        td_lines(&stderr),
        // One per position: the item written there, not the call's type.
        [
            "lib.rs:3:14: error: TD0520: `core::sync::atomic::AtomicU64` is not admitted",
            "lib.rs:5:16: error: TD0520: `std::f64::<impl f64>::sin` is not admitted",
            "sub.rs:3:21: error: TD0520: `core::sync::atomic::AtomicU64` is not admitted",
        ],
        "{stderr}"
    );

    // Reference §6.9's closed denials (and §2.1's compile-time reads). Keep each
    // use on its own line: unrelated diagnostics must not satisfy another row.
    let mut mismatches = Vec::new();
    for (name, statement, key) in [
        (
            "atomic",
            "let _ = core::sync::atomic::AtomicU64::new(0);",
            "core::sync::atomic::AtomicU64",
        ),
        (
            "once-lock",
            "let _ = std::sync::OnceLock::<u8>::new();",
            "std::sync::once_lock::OnceLock::<T>::new",
        ),
        (
            "lazy-lock",
            "let _ = std::sync::LazyLock::new(|| 1u8);",
            "std::sync::lazy_lock::LazyLock::<T, F>::new",
        ),
        ("raw-pointer", "let _: *const u8 = &0u8;", "*const T"),
        (
            "pointer-address",
            "let _ = <*const u8>::addr;",
            "core::ptr::const_ptr::<impl *const T>::addr",
        ),
        (
            "pointer-format",
            "let _ = format!(\"{:p}\", &0u8);",
            "core::fmt::rt::Argument::<'_>::new_pointer",
        ),
        ("fn-address", "let _ = run as usize;", "fn() as integer"),
        (
            "last-os-error",
            "let _ = std::io::Error::last_os_error;",
            // The type prefix is also unlisted; build.rs checks Tide's own method key.
            "core::io::error::Error",
        ),
        (
            "waker",
            "let _: Option<core::task::Waker> = None;",
            "core::task::wake::Waker",
        ),
        (
            "include-str",
            "let _ = core::include_str!(\"lib.rs\");",
            "core::macros::builtin::include_str",
        ),
        (
            "env",
            "let _ = core::env!(\"RUSTC_ICE\");",
            "core::macros::builtin::env",
        ),
        (
            "option-env",
            "let _ = core::option_env!(\"RUSTC_ICE\");",
            "core::macros::builtin::option_env",
        ),
    ] {
        let (code, stderr) = admit(
            &format!("reference-deny-{name}"),
            &format!("pub fn run() {{\n    {statement}\n}}\n"),
            &[],
        );
        if code != 1
            || !td_lines(&stderr).iter().any(|line| {
                line.starts_with("lib.rs:2:")
                    && line.contains(&format!("error: TD0520: `{key}` is not admitted"))
            })
        {
            mismatches.push(format!("{name} (exit {code}): {stderr}"));
        }
    }
    for method in [
        "exists",
        "try_exists",
        "metadata",
        "symlink_metadata",
        "canonicalize",
        "read_link",
        "read_dir",
        "is_file",
        "is_dir",
        "is_symlink",
    ] {
        // A function-item use isolates the method from Rust's unlisted io::Error
        // return type, whose rejection would otherwise win at the same call span.
        let (code, stderr) = admit(
            &format!("reference-deny-path-{method}"),
            &format!("pub fn run() {{\n    let _ = std::path::Path::{method};\n}}\n"),
            &[],
        );
        assert_eq!(code, 1, "{method}: {stderr}");
        assert_eq!(
            td_lines(&stderr),
            [format!(
                "lib.rs:2:13: error: TD0520: `std::path::Path::{method}` is not admitted"
            )],
            "{stderr}"
        );
    }
    for ty in ["f32", "f64"] {
        for (method, args, owner) in [
            ("to_bits", "", "core"),
            ("is_sign_positive", "", "core"),
            ("is_sign_negative", "", "core"),
            ("copysign", "x", "core"),
            ("signum", "", "core"),
            ("total_cmp", "&x", "core"),
            ("min", "x", "core"),
            ("max", "x", "core"),
            ("sin", "", "std"),
            ("exp", "", "std"),
            ("powf", "x", "std"),
            ("ln", "", "std"),
        ] {
            let (code, stderr) = admit(
                &format!("reference-deny-{ty}-{method}"),
                &format!("pub fn run(x: {ty}) {{\n    let _ = x.{method}({args});\n}}\n"),
                &[],
            );
            assert_eq!(code, 1, "{ty}::{method}: {stderr}");
            assert_eq!(
                td_lines(&stderr),
                [format!(
                    "lib.rs:2:13: error: TD0520: `{owner}::{ty}::<impl {ty}>::{method}` is not admitted"
                )],
                "{stderr}"
            );
        }
    }
    assert!(mismatches.is_empty(), "{}", mismatches.join("\n"));
}

/// Safe hardware intrinsics read the host's processor (D18). `core::arch` has one module per
/// target, so this runs on x86-64 hosts only, as `conformance/cases/authority/admit_hardware.rs`
/// does.
#[cfg(target_arch = "x86_64")]
#[test]
fn hardware_queries_are_rejected() {
    let source = "pub fn vendor() -> u32 {\n    core::arch::x86_64::__cpuid(0).ebx\n}\n";
    let (code, stderr) = admit("hardware", source, &[]);
    assert_eq!(code, 1, "{stderr}");
    assert_eq!(
        td_lines(&stderr),
        ["lib.rs:2:5: error: TD0520: `core::core_arch::x86::cpuid::__cpuid` is not admitted"],
        "{stderr}"
    );
    let (code, stderr) = admit(
        "reference-deny-asm",
        "pub fn run() {\n    // SAFETY: the empty asm block touches no memory or stack.\n    unsafe { core::arch::asm!(\"\", options(nomem, nostack)); }\n}\n",
        &[],
    );
    assert_eq!(code, 1, "{stderr}");
    assert!(
        td_lines(&stderr)
            .contains(&"lib.rs:3:14: error: TD0520: `core::arch::asm` is not admitted"),
        "{stderr}"
    );
}

#[test]
fn the_unsafe_grant_admits_address_observers_and_features_stay_closed() {
    let source = "\
pub fn address(x: &u8) -> usize {
    (x as *const u8).addr()
}
";
    let (code, stderr) = admit("ungranted", source, &[]);
    assert_eq!(code, 1, "{stderr}");
    assert_eq!(
        td_lines(&stderr),
        ["lib.rs:2:5: error: TD0520: `*const T` is not admitted"],
        "{stderr}"
    );

    let (code, stderr) = admit("granted", source, &["--tide-grant=unsafe"]);
    assert_eq!((code, stderr.as_str()), (0, ""));

    // Source feature gates stay closed independently of CLI rejection (ADMIT-2).
    let gated = "#![feature(core_intrinsics)]\npub fn f() {}\n";
    let (code, stderr) = admit("feature", gated, &[]);
    assert_eq!(code, 1, "{stderr}");
    assert!(stderr.contains("lib.rs:1:12: error[E0725]"), "{stderr}");
}

#[test]
fn hidden_random_keys_and_unchecked_obligations_are_rejected() {
    // `From<[_; N]>` builds a `RandomState` inside, which no bound of the impl shows (GLOBAL-5),
    // and an obligation chain past the walk's depth is rejected, not admitted unchecked.
    let deep = format!("{}u8{}", "W<".repeat(70), ">".repeat(70));
    let source = format!(
        "\
use std::collections::{{HashMap, HashSet}};
#[derive(Default)]
pub struct W<T>(T);
pub fn maps() -> (HashMap<u8, u8>, HashSet<u8>) {{
    (HashMap::from([(1, 2)]), HashSet::from([1]))
}}
pub fn into() -> HashMap<u8, u8> {{
    [(1, 2)].into()
}}
pub fn deep() -> {deep} {{
    Default::default()
}}
"
    );
    let (code, stderr) = admit("hidden", &source, &[]);
    assert_eq!(code, 1, "{stderr}");
    let lines = td_lines(&stderr);
    let map =
        "impl core::convert::From<[(K, V); N]> for std::collections::hash::map::HashMap<K, V>";
    let set = "impl core::convert::From<[T; N]> for std::collections::hash::set::HashSet<T>";
    assert_eq!(
        lines[..3],
        [
            format!("lib.rs:5:6: error: TD0520: `{map}` is not admitted"),
            format!("lib.rs:5:31: error: TD0520: `{set}` is not admitted"),
            format!("lib.rs:8:5: error: TD0520: `{map}` is not admitted"),
        ],
        "{stderr}"
    );
    assert_eq!(lines.len(), 4, "{stderr}");
    assert!(
        lines[3].starts_with("lib.rs:11:5: error: TD0520: `lib::W<lib::W<")
            && lines[3].ends_with(
                ": core::default::Default` is not admitted (its obligations nest too deep to check)"
            ),
        "{stderr}"
    );
}

#[test]
fn impl_items_that_do_not_match_their_trait_are_left_to_rustc() {
    // rustc reports the mismatch (E0049); the walk skips the items rather than instantiate the
    // trait item's bounds and where-clauses with the wrong arguments.
    let source = "\
pub trait Tr {
    type A<V>: Clone;
    fn f<U: Clone>(u: U);
}
pub struct X;
impl Tr for X {
    type A = u8;
    fn f(u: u8) {}
}
";
    let (code, stderr) = admit("mismatch", source, &[]);
    assert_eq!(code, 1, "{stderr}");
    assert!(stderr.contains("lib.rs:7:11: error[E0049]"), "{stderr}");
    assert!(stderr.contains("lib.rs:8:9: error[E0049]"), "{stderr}");
}

/// The backstop alone, with the walk's obligation steps off, rejects each route by which
/// fx-kw8.1's reviews (rounds 2 to 5) reached a denied impl, at the nearest local mono item that
/// runs it: `?` into a `HashMap`, `?` and `for` through a user impl, an autoderef, an ADT's
/// `Drop` impl, a constructor passed as a function, a supertrait through a generic bound and
/// through `dyn`, `Box<dyn Error>`, an associated-type bound, a return-position `impl Trait` in
/// a trait, builtin `Clone` of a tuple and a closure, and a test's and `main`'s `Termination`.
#[test]
fn the_backstop_rejects_denied_impls_whatever_route_reaches_them() {
    let source = "\
use std::collections::HashMap;
use std::error::Error;
use std::fmt::Debug;
use std::ops::Deref;
pub fn noop() {}
pub fn into_map() -> Result<(), HashMap<u8, u8>> {
    let r: Result<(), [(u8, u8); 1]> = Err([(1, 2)]);
    r?;
    Ok(())
}
pub struct Wrap<T>(pub T);
pub struct Failure;
impl<T: Debug> From<Wrap<T>> for Failure {
    fn from(w: Wrap<T>) -> Failure {
        let _ = format!(\"{:?}\", w.0);
        Failure
    }
}
pub fn convert() -> Result<(), Failure> {
    let r: Result<(), Wrap<fn()>> = Err(Wrap(noop));
    r?;
    Ok(())
}
pub struct Items<T>(pub T);
impl<T: Debug> IntoIterator for Items<T> {
    type Item = String;
    type IntoIter = std::vec::IntoIter<String>;
    fn into_iter(self) -> Self::IntoIter {
        vec![format!(\"{:?}\", self.0)].into_iter()
    }
}
pub fn iterate() {
    for _ in Items(noop as fn()) {}
}
pub struct Named<T>(pub T, pub String);
impl<T: Debug> Deref for Named<T> {
    type Target = String;
    fn deref(&self) -> &String {
        let _ = format!(\"{:?}\", self.0);
        &self.1
    }
}
pub fn autoderef(s: String) -> usize {
    Named(noop as fn(), s).len()
}
pub struct Logged<T: Debug>(pub T);
impl<T: Debug> Drop for Logged<T> {
    fn drop(&mut self) {
        let _ = format!(\"{:?}\", self.0);
    }
}
pub fn mapped() -> usize {
    [noop as fn()].into_iter().map(Logged).count()
}
pub trait Shown: Debug {}
impl Shown for fn() {}
fn show<T: Shown>(t: T) -> String {
    format!(\"{t:?}\")
}
pub fn shown() -> String {
    show(noop as fn())
}
pub fn shown_dyn(f: fn()) -> String {
    let d = &f as &dyn Shown;
    format!(\"{d:?}\")
}
#[derive(Debug)]
pub struct E<T>(pub T);
impl std::fmt::Display for E<fn()> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(\"e\")
    }
}
impl Error for E<fn()> {}
pub fn boxed() -> String {
    let b: Box<dyn Error> = Box::new(E(noop as fn()));
    b.to_string()
}
pub trait Make {
    type Out: Debug;
    fn make(&self) -> Self::Out;
}
impl Make for u8 {
    type Out = fn();
    fn make(&self) -> fn() {
        noop
    }
}
fn made<T: Make>(t: T) -> String {
    format!(\"{:?}\", t.make())
}
pub fn assoc() -> String {
    made(1u8)
}
pub trait Rpit {
    fn f(&self) -> impl Debug;
}
impl Rpit for u8 {
    fn f(&self) -> impl Debug {
        noop as fn()
    }
}
pub fn rpitit() -> String {
    format!(\"{:?}\", 1u8.f())
}
pub struct C<T>(pub T);
impl<T: Debug> Clone for C<T> {
    fn clone(&self) -> Self {
        let _ = format!(\"{:?}\", self.0);
        loop {}
    }
}
pub fn tuple_clone(t: &(C<fn()>,)) -> (C<fn()>,) {
    t.clone()
}
pub fn closure_clone(c: C<fn()>) {
    let k = move || {
        let _ = &c;
    };
    let _ = k.clone();
}
#[test]
fn t() -> Result<(), fn()> {
    Err(noop)
}
";
    let (code, stderr) = admit("backstop", source, &["--tide-test-backstop"]);
    assert_eq!(code, 1, "{stderr}");
    let lines = td_lines(&stderr);
    let at: Vec<_> = lines
        .iter()
        .map(|l| l.split(": error").next().unwrap())
        .collect();
    // One per route, in the local item that runs the impl: `into_map`, then the user impls and
    // generic functions that format, and the derived `Debug` that `dyn Error`'s vtable holds.
    assert_eq!(
        at,
        [
            "lib.rs:8:5",
            "lib.rs:15:17",
            "lib.rs:29:14",
            "lib.rs:39:17",
            "lib.rs:49:17",
            "lib.rs:58:5",
            "lib.rs:64:13",
            "lib.rs:67:10",
            "lib.rs:90:5",
            "lib.rs:104:5",
            "lib.rs:109:17",
        ],
        "{stderr}"
    );
    let fn_debug = "TD0520: `impl core::fmt::Debug for fn()` is not admitted \
                    (it runs as `std::ptr::<impl Debug for fn()>::fmt`)";
    assert!(
        lines[0].ends_with(
            "TD0520: `impl core::convert::From<[(K, V); N]> for \
             std::collections::hash::map::HashMap<K, V>` is not admitted (it runs as \
             `<HashMap<u8, u8> as From<[(u8, u8); 1]>>::from`)"
        ),
        "{stderr}"
    );
    assert!(lines[1..].iter().all(|l| l.ends_with(fn_debug)), "{stderr}");

    // A test build runs only what its tests reach: the test's result.
    let (code, stderr) = admit("backstop-test", source, &["--test", "--tide-test-backstop"]);
    assert_eq!(code, 1, "{stderr}");
    assert_eq!(
        td_lines(&stderr),
        [format!("lib.rs:122:1: error: {fn_debug}")],
        "{stderr}"
    );

    // `main`'s result reaches the impl through rustc's entry point.
    let main = "fn noop() {}\nfn main() -> Result<(), fn()> {\n    Err(noop)\n}\n";
    let (code, stderr) = admit(
        "backstop-main",
        main,
        &["--crate-type=bin", "--tide-test-backstop"],
    );
    assert_eq!(code, 1, "{stderr}");
    assert_eq!(
        td_lines(&stderr),
        [format!("lib.rs:2:1: error: {fn_debug}")],
        "{stderr}"
    );
}

/// An imported `main` is never the one `#[tide::main]` generates (ROOT-1), even from another
/// crate, whose entry points the walk does not otherwise check.
#[test]
fn an_imported_main_is_unmarked() {
    let std = tide_std("imported-main");
    let (code, stderr) = admit(
        "imported-main",
        "use std::process::abort as main;\n",
        &["--crate-type=bin", "--tide-std", std.to_str().unwrap()],
    );
    assert_eq!(code, 1, "{stderr}");
    let unmarked = "lib.rs:1:5: error: TD0512: `main` does not carry `#[tide::main]`, which \
                    starts the runtime";
    assert!(td_lines(&stderr).contains(&unmarked), "{stderr}");
}

/// fx-qlek.19: a root item named `task` that is not a module, beside `mod task`, does not send
/// the scope hint's lookup of `Tasks::scope` into a non-module, which would crash the driver.
#[test]
fn a_root_fn_named_task_does_not_crash_the_scope_lookup() {
    let std = tide_std_with("root-fn-task", "pub fn task() {}\n");
    let (code, stderr) = admit("root-fn-task", "pub fn f() {}\n", &["--tide-std", std.to_str().unwrap()]);
    assert_eq!(code, 0, "{stderr}");
}

/// A minimal Tide std, built with the pinned rustc: Rust's std with `Pure`, which rustc meets
/// for every type and the driver decides, and a `Clock`, which is never `Pure`.
fn tide_std(name: &str) -> PathBuf {
    tide_std_with(name, "")
}

/// `tide_std` with `items` first at its root.
fn tide_std_with(name: &str, items: &str) -> PathBuf {
    let dir = PathBuf::from(env!("CARGO_TARGET_TMPDIR"))
        .join(name)
        .join("tide-std");
    std::fs::create_dir_all(&dir).unwrap();
    let source = dir.join("lib.rs");
    std::fs::write(
        &source,
        items.to_owned()
            + "\
pub use std::*;
pub mod prelude {
    pub mod rust_2024 {
        pub use crate::marker::Pure;
        pub use std::prelude::rust_2024::*;
    }
}
pub mod marker {
    pub use std::marker::*;
    pub unsafe trait Pure {}
    unsafe impl<T: ?Sized> Pure for T {}
}
pub mod time {
    pub struct Clock(());
}
pub mod net {
    pub struct Net(());
    pub struct Url(());
}
pub mod sync {
    pub struct Arc<T>(T);
}
pub mod future {}
pub mod par {
    use crate::marker::Pure;
    pub fn join<A: FnOnce() -> R + Pure + Send, B: FnOnce() -> S + Pure + Send, R, S>(a: A, b: B) -> (R, S) { (a(), b()) }
    pub trait ParSlice<T> {
        fn par_for<F: Fn(&mut T) + Pure + Send + Sync>(&mut self, f: F);
    }
    impl<T> ParSlice<T> for [T] {
        fn par_for<F: Fn(&mut T) + Pure + Send + Sync>(&mut self, f: F) { self.iter_mut().for_each(f); }
    }
}
pub mod task { pub struct Tasks; }
pub struct Handle;
impl Handle { pub fn new(_: &task::Tasks) -> Self { Self } }
",
    )
    .unwrap();
    let status = Command::new(PathBuf::from(env!("TIDE_SYSROOT")).join("bin/rustc"))
        .args([
            "--edition=2024",
            "--crate-type=rlib",
            "--crate-name=tide_std",
        ])
        .arg("--out-dir")
        .arg(&dir)
        .arg(&source)
        .status()
        .unwrap();
    assert!(status.success());
    dir.join("libtide_std.rlib")
}

/// The backstop alone decides the `Pure` where-clauses of the instances that run: an ADT's
/// `Drop` impl's, an impl method's own, a trait method's that its impl does not repeat, one
/// reached through a method's type argument, and one reached through a static's value. A
/// provided method that needs `Self: Pure`, which no one calls, is not a use.
#[test]
fn the_backstop_decides_pure_where_clauses_at_their_uses() {
    let std = tide_std("backstop-pure");
    let source = "\
use std::time::Clock;
pub struct Guard<T>(pub T);
impl<T: Pure> Drop for Guard<T> {
    fn drop(&mut self) {}
}
pub fn guarded(c: Clock) {
    let _ = Guard(c);
}
pub trait Show {
    fn show(&self);
    fn check(&self)
    where
        Self: Pure;
    fn snapshot(&self)
    where
        Self: Pure,
    {
    }
}
pub struct W<T>(pub T);
impl<T> Show for W<T> {
    fn show(&self)
    where
        T: Pure,
    {
    }
    fn check(&self) {}
}
pub fn shown(c: Clock) {
    W(c).show()
}
pub fn checked(w: &W<Clock>) {
    w.check()
}
pub fn make<V: Default>() {
    let _ = V::default();
}
pub fn inside() {
    make::<Vec<Guard<Clock>>>()
}
fn need<T: Pure>() {}
pub static NEED: fn() = need::<Clock>;
impl Show for Clock {
    fn show(&self) {}
    fn check(&self) {}
}
";
    let std = std.to_str().unwrap();
    let (code, stderr) = admit(
        "backstop-pure",
        source,
        &["--tide-std", std, "--tide-test-backstop"],
    );
    assert_eq!(code, 1, "{stderr}");
    let clock = "TD0517: `Clock` is never `Pure`";
    assert_eq!(
        td_lines(&stderr),
        [
            format!("lib.rs:7:21: error: {clock}"),
            format!("lib.rs:30:5: error: {clock}"),
            "lib.rs:33:5: error: TD0517: `W<Clock>` is not `Pure`: it reaches `Clock`, which is \
             never `Pure`, via `W.0`"
                .to_string(),
            // In `make`, the nearest local item that drops the guards.
            format!("lib.rs:36:25: error: {clock}"),
            format!("lib.rs:42:1: error: {clock}"),
        ],
        "{stderr}"
    );
}

/// A `Pure` failure names one path from the obligation's type to what is not `Pure`, each step
/// a field of the type that declares it (a struct's, an enum variant's, a tuple's), under the
/// same code and location; a recursive type stays `Pure` unless it reaches authority, and the
/// path stops at an opaque type without revealing what it hides (fx-kw8.24).
#[test]
fn pure_failures_name_one_witness_path() {
    let std = tide_std("pure-witness");
    let source = "\
use std::net::Net;
use std::task::Tasks;
fn need<T: Pure>(_: &T) {}
pub struct Client {
    pub id: u32,
    pub socket: Net,
}
pub struct Payload {
    pub name: String,
    pub client: Client,
}
pub fn nested(p: &Payload) {
    need(p)
}
pub enum Msg {
    A(u8),
    B(u8, Vec<Tasks>),
}
pub fn variant(m: &Msg) {
    need(m)
}
pub fn tuple(t: &(u8, (Client, u32))) {
    need(t)
}
pub struct List {
    pub next: Option<Box<List>>,
    pub items: Vec<u8>,
}
pub fn recursive_ok(l: &List) {
    need(l)
}
pub struct Ring {
    pub next: Option<Box<Ring>>,
    pub tasks: Tasks,
}
pub fn recursive_bad(r: &Ring) {
    need(r)
}
pub fn hidden() -> impl Sized {
    0u8
}
pub struct Wrap<T>(pub T);
pub fn opaque() {
    need(&Wrap(hidden()))
}
";
    let (code, stderr) = admit(
        "pure-witness",
        source,
        &["--tide-std", std.to_str().unwrap()],
    );
    assert_eq!(code, 1, "{stderr}");
    assert_eq!(
        td_lines(&stderr),
        [
            "lib.rs:13:5: error: TD0517: `Payload` is not `Pure`: it reaches `Net`, which is \
             never `Pure`, via `Payload.client` -> `Client.socket`",
            "lib.rs:20:5: error: TD0517: `Msg` is not `Pure`: it reaches `Tasks`, which is never \
             `Pure`, via `Msg::B.1`",
            "lib.rs:23:5: error: TD0517: `(u8, (Client, u32))` is not `Pure`: it reaches `Net`, \
             which is never `Pure`, via `(u8, (Client, u32)).1` -> `(Client, u32).0` -> \
             `Client.socket`",
            "lib.rs:37:5: error: TD0517: `Ring` is not `Pure`: it reaches `Tasks`, which is never \
             `Pure`, via `Ring.tasks`",
            "lib.rs:44:5: error: TD0517: `Wrap<impl Sized>` is not `Pure`: it reaches \
             `impl Sized`, which does not state `Pure` in its bounds, via `Wrap.0`",
        ],
        "{stderr}"
    );
}

/// A static whose value points at an upstream function that rustc links rather than
/// instantiates has no uses to look for: the backstop must not ask for that function's MIR,
/// which a linked function lacks (fx-kw8.4 round 2).
#[test]
fn the_backstop_skips_upstream_functions_a_static_points_at() {
    let std = tide_std("backstop-static");
    let std = std.to_str().unwrap();
    let source = "pub static LOWER: fn(&str) -> String = str::to_lowercase;\n";
    for flags in [
        &["--tide-std", std][..],
        &["--tide-std", std, "--tide-test-backstop"],
    ] {
        let (code, stderr) = admit("backstop-static", source, flags);
        assert_eq!((code, stderr.as_str()), (0, ""));
    }
}

/// The trust artifact (D11; 03 §4.1) lists each exported item's declared authority with its
/// access paths, none for a `Pure` signature, through handles, aliases, re-exports, and nested
/// instances of one type, a type and a function of one path apart, through finite recursion,
/// with polymorphic recursion `opaque`; a body edit under the grant adds its holes and no
/// authority;
/// a failed check, whatever stops it, leaves no artifact behind; and a crate whose `std` is not
/// Tide's fails.
#[test]
fn the_trust_artifact_lists_declared_authority_and_holes() {
    let std = tide_std("artifact");
    let std = std.to_str().unwrap();
    let dir = PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join("artifact");
    let file = dir.join("lib.interface.json");
    let artifact = || std::fs::read_to_string(&file).unwrap();
    let source = "\
use std::net::Net;
use std::sync::Arc;
use std::time::Clock;
pub struct Db {
    net: Net,
}
pub struct App {
    pub db: Db,
    clock: Clock,
}
pub fn handle(app: &App, req: &str) -> usize {
    req.len()
}
pub fn len(s: &str) -> usize {
    s.len()
}
pub fn shared(_: Arc<Net>) {}
pub type Handle = Net;
pub use std::net::Net as Cap;
pub struct Poly<T> {
    pub a: T,
    pub b: Option<Box<Poly<(T, T)>>>,
}
pub struct W<T>(pub T);
pub fn nested(_: W<W<Net>>) {}
pub struct Node<T> {
    pub val: T,
    pub child: Option<Box<Node<Arc<Net>>>>,
}
pub fn node(_: Node<Net>) {}
pub struct Config {
    pub net: Net,
}
#[allow(non_snake_case)]
pub fn Config(_: Clock) {}
fn private(_: Net) {}
";
    let (code, stderr) = admit("artifact", source, &["--tide-std", std]);
    assert_eq!(code, 0, "{stderr}");
    let items = r##"{
  "format": "tide-interface/1",
  "crate": "lib",
  "items": [
    {"item": "lib::App", "kind": "struct", "authority": {"clock": ["clock"], "db.net": ["net"]}},
    {"item": "lib::Cap", "kind": "re-exported struct", "authority": {"": ["net"]}},
    {"item": "lib::Config", "kind": "function", "authority": {"#0": ["clock"]}},
    {"item": "lib::Config", "kind": "struct", "authority": {"net": ["net"]}},
    {"item": "lib::Db", "kind": "struct", "authority": {"net": ["net"]}},
    {"item": "lib::Handle", "kind": "type alias", "authority": {"": ["net"]}},
    {"item": "lib::Node", "kind": "struct", "authority": {"child.val": ["net", "sync"], "val": ["opaque(T)"]}},
    {"item": "lib::Poly", "kind": "struct", "authority": {"a": ["opaque(T)"], "b.a.0": ["opaque(T)"], "b.a.1": ["opaque(T)"], "b.b": ["opaque(Poly<((T, T), (T, T))>)"]}},
    {"item": "lib::W", "kind": "struct", "authority": {"0": ["opaque(T)"]}},
    {"item": "lib::handle", "kind": "function", "authority": {"app.clock": ["clock"], "app.db.net": ["net"]}},
    {"item": "lib::len", "kind": "function", "authority": {}},
    {"item": "lib::nested", "kind": "function", "authority": {"#0.0.0": ["net"]}},
    {"item": "lib::node", "kind": "function", "authority": {"#0.child.val": ["net", "sync"], "#0.val": ["net"]}},
    {"item": "lib::shared", "kind": "function", "authority": {"#0": ["net", "sync"]}}
  ],
  "holes": ["##;
    assert_eq!(artifact(), format!("{items}\n  ]\n}}\n"));

    let granted = source.replace(
        "s.len()",
        "\n    // SAFETY: this metadata-only fixture is never executed.\n    unsafe { s.as_bytes().as_ptr().read() as usize }",
    );
    let flags = ["--tide-std", std, "--tide-grant=unsafe"];
    let (code, stderr) = admit("artifact", &granted, &flags);
    assert_eq!(code, 0, "{stderr}");
    let holes = r#"
    {"item": "lib::len", "kind": "function", "hole": "an `unsafe` block", "count": 1},
    {"item": "lib::len", "kind": "function", "hole": "uses `*const T`", "count": 1},
    {"item": "lib::len", "kind": "function", "hole": "uses `core::ptr::const_ptr::<impl *const T>::read`", "count": 1}"#;
    assert_eq!(artifact(), format!("{items}{holes}\n  ]\n}}\n"));

    let (code, _) = admit("artifact", &granted, &["--tide-std", std]);
    assert_eq!(code, 1);
    assert!(!file.exists());
    let (code, stderr) = admit("artifact", &granted, &flags);
    assert_eq!(code, 0, "{stderr}");
    assert!(file.exists());
    let build = dir.join("package/build.rs");
    std::fs::write(&build, "fn main() {}\n").unwrap();
    let (code, _) = admit("artifact", &granted, &flags);
    std::fs::remove_file(&build).unwrap();
    assert_eq!(code, 1);
    assert!(!file.exists());

    // `#![no_std]` is not admitted, even with no `extern crate std` (TD0520), and the failed
    // check leaves no artifact; an inactive `cfg_attr` of it is no `#![no_std]`.
    let (code, stderr) = admit("artifact", "#![no_std]\npub fn f() {}\n", &flags);
    assert_eq!(code, 1, "{stderr}");
    assert!(stderr.contains("TD0520: `#![no_std]`"), "{stderr}");
    assert!(!file.exists());
    let inactive = "#![cfg_attr(any(), no_std)]\npub fn f() {}\n";
    let (code, stderr) = admit("artifact", inactive, &flags);
    assert_eq!(code, 0, "{stderr}");
    assert!(file.exists());
}

/// Each hole is held by the innermost item it is written in, a nested `fn`'s by that `fn`, and
/// a granted macro's or impl's by the item that uses it, once per item: a generic function that
/// formats its `T` holds none, the caller whose `fn()` chose the impl does, and a generic
/// function holds the impls its own `fn()` reaches through std's internals, whatever other
/// items hold; an instance no local item runs, as a `global_asm!` operand's, is the crate's.
#[test]
fn trust_holes_are_held_by_the_item_that_writes_them() {
    let std = tide_std("holders");
    let source = "\
pub fn a() -> bool {
    let x = 1u8;
    // SAFETY: these empty asm blocks touch no memory or stack.
    unsafe { core::arch::asm!(\"\", options(nomem, nostack)); core::arch::asm!(\"\", options(nomem, nostack)); }
    core::ptr::from_ref(&x).is_null() || core::ptr::from_ref(&x).is_null()
}
pub fn b() -> bool {
    let x = 1u8;
    // SAFETY: the empty asm block touches no memory or stack.
    unsafe { core::arch::asm!(\"\", options(nomem, nostack)); }
    core::ptr::from_ref(&x).is_null()
}
pub fn show(f: fn()) -> String {
    format!(\"{f:?}\")
}
pub fn outer() {
    unsafe fn inner() {}
    // SAFETY: inner performs no operations.
    unsafe { inner() }
}
pub fn debug<T: core::fmt::Debug>(t: T) -> String {
    format!(\"{t:?}\")
}
pub fn shown() -> String {
    debug(shown as fn() -> String)
}
pub fn sort<T>(_: T, mut v: Vec<fn()>) -> Vec<fn()> {
    v.sort();
    v
}
pub fn sorted() -> Vec<fn()> {
    sort(0, Vec::new())
}
pub fn lt(a: fn(), b: fn()) -> bool {
    a < b
}
core::arch::global_asm!(\".quad {}\", sym <fn(u8) as core::cmp::PartialOrd>::lt);
";
    let flags = ["--tide-std", std.to_str().unwrap(), "--tide-grant=unsafe"];
    let (code, stderr) = admit("holders", source, &flags);
    assert_eq!(code, 0, "{stderr}");
    let file = PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join("holders/lib.interface.json");
    let artifact = std::fs::read_to_string(file).unwrap();
    let holes = &artifact[artifact.find("\"holes\"").unwrap()..];
    let expected = r#""holes": [
    {"item": "lib", "kind": "module", "hole": "uses `impl core::cmp::PartialOrd for fn()`", "count": 1},
    {"item": "lib::a", "kind": "function", "hole": "an `unsafe` block", "count": 1},
    {"item": "lib::a", "kind": "function", "hole": "uses `*const T`", "count": 1},
    {"item": "lib::a", "kind": "function", "hole": "uses `core::arch::asm`", "count": 1},
    {"item": "lib::a", "kind": "function", "hole": "uses `core::ptr::const_ptr::<impl *const T>::is_null`", "count": 1},
    {"item": "lib::a", "kind": "function", "hole": "uses `core::ptr::from_ref`", "count": 1},
    {"item": "lib::b", "kind": "function", "hole": "an `unsafe` block", "count": 1},
    {"item": "lib::b", "kind": "function", "hole": "uses `*const T`", "count": 1},
    {"item": "lib::b", "kind": "function", "hole": "uses `core::arch::asm`", "count": 1},
    {"item": "lib::b", "kind": "function", "hole": "uses `core::ptr::const_ptr::<impl *const T>::is_null`", "count": 1},
    {"item": "lib::b", "kind": "function", "hole": "uses `core::ptr::from_ref`", "count": 1},
    {"item": "lib::lt", "kind": "function", "hole": "uses `impl core::cmp::PartialOrd for fn()`", "count": 1},
    {"item": "lib::outer", "kind": "function", "hole": "an `unsafe` block", "count": 1},
    {"item": "lib::outer::inner", "kind": "function", "hole": "an `unsafe fn`", "count": 1},
    {"item": "lib::show", "kind": "function", "hole": "uses `impl core::fmt::Debug for fn()`", "count": 1},
    {"item": "lib::shown", "kind": "function", "hole": "uses `impl core::fmt::Debug for fn()`", "count": 1},
    {"item": "lib::sort", "kind": "function", "hole": "uses `impl core::cmp::Ord for fn()`", "count": 1},
    {"item": "lib::sort", "kind": "function", "hole": "uses `impl core::cmp::PartialOrd for fn()`", "count": 1},
    {"item": "lib::{global_asm#0}", "kind": "global assembly block", "hole": "uses `core::arch::global_asm`", "count": 1},
    {"item": "lib::{global_asm#0}", "kind": "global assembly block", "hole": "uses `impl core::cmp::PartialOrd for fn()`", "count": 1}
  ]
}
"#;
    assert_eq!(holes, expected);
}

/// An `unsafe extern` block needs the `unsafe` grant, even an empty one, which still links the
/// library its `#[link]` names, and so does each foreign item, once, a `safe` one too, since its
/// callers reach foreign code with no `unsafe` block; under the grant each is a trust hole
/// (UNSAFE-1).
#[test]
fn extern_blocks_and_their_items_need_the_unsafe_grant() {
    let std = tide_std("foreign");
    let std = std.to_str().unwrap();
    let source = "\
unsafe extern \"C\" {
    pub safe fn abs(x: i32) -> i32;
    fn probe();
}
pub fn magnitude(x: i32) -> i32 {
    abs(x)
}
#[link(name = \"m\")]
unsafe extern \"C\" {}
";
    let (code, stderr) = admit("foreign-ungranted", source, &["--tide-std", std]);
    assert_eq!(code, 1, "{stderr}");
    let block = "error: TD0901: an `unsafe extern` block needs the package's `unsafe` grant";
    let item = "error: TD0901: a foreign item needs the package's `unsafe` grant";
    assert_eq!(
        td_lines(&stderr),
        [
            format!("lib.rs:1:1: {block}"),
            format!("lib.rs:2:5: {item}"),
            format!("lib.rs:3:5: {item}"),
            format!("lib.rs:9:1: {block}"),
        ],
        "{stderr}"
    );

    let flags = ["--tide-std", std, "--tide-grant=unsafe"];
    let (code, stderr) = admit("foreign", source, &flags);
    assert_eq!(code, 0, "{stderr}");
    let file = PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join("foreign/lib.interface.json");
    let artifact = std::fs::read_to_string(file).unwrap();
    let holes = &artifact[artifact.find("\"holes\"").unwrap()..];
    let expected = r#""holes": [
    {"item": "lib", "kind": "foreign module", "hole": "an `unsafe extern` block", "count": 2},
    {"item": "lib::abs", "kind": "function", "hole": "a foreign item", "count": 1},
    {"item": "lib::probe", "kind": "function", "hole": "a foreign item", "count": 1}
  ]
}
"#;
    assert_eq!(holes, expected);
}

/// NET-HTTP-4: a public item that takes `Net` (to call `get`) shows `net` in the trust diff, and
/// `Url` adds no authority (the fake std here has no `get`; companion 15.2's build admits calls).
#[test]
fn net_get_is_admitted_with_net_in_the_trust_diff() {
    let std = tide_std("trust-diff-net");
    let std = std.to_str().unwrap();
    let dir = PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join("trust-diff-net");
    let snapshot = |name: &str, source: &str| {
        let (code, stderr) = admit("trust-diff-net", source, &["--tide-std", std]);
        assert_eq!(code, 0, "{stderr}");
        let copy = dir.join(name);
        std::fs::copy(dir.join("lib.interface.json"), &copy).unwrap();
        copy
    };
    let base = snapshot("base.json", "pub fn path(url: &std::net::Url) -> usize { let _ = url; 0 }\n");
    let head = snapshot(
        "head.json",
        "pub fn path(url: &std::net::Url) -> usize { let _ = url; 0 }\n\
         pub fn fetch(net: &std::net::Net, url: &std::net::Url) -> bool { let _ = (net, url); true }\n",
    );
    let out = Command::new(env!("CARGO_BIN_EXE_tide-admit"))
        .arg("--tide-diff")
        .args([&base, &head])
        .output()
        .unwrap();
    assert_eq!(
        String::from_utf8(out.stdout).unwrap(),
        "declared authority (no exercised authority, effects, or proof status)\n  \
         + lib::fetch (function)\n      \
         + net: net\n"
    );
}

/// `--tide-diff` compares two trust artifacts: a `Net` a public item can now reach is listed with
/// its item and path, a new `unsafe` block as a new hole, and a body-only edit that changes no
/// authority or holes gives an empty report.
#[test]
fn the_trust_diff_reports_authority_and_hole_changes() {
    let std = tide_std("trust-diff");
    let std = std.to_str().unwrap();
    let dir = PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join("trust-diff");
    let snapshot = |name: &str, source: &str| {
        let (code, stderr) = admit(
            "trust-diff",
            source,
            &["--tide-std", std, "--tide-grant=unsafe"],
        );
        assert_eq!(code, 0, "{stderr}");
        let copy = dir.join(name);
        std::fs::copy(dir.join("lib.interface.json"), &copy).unwrap();
        copy
    };
    let diff = |base: &PathBuf, head: &PathBuf| {
        let out = Command::new(env!("CARGO_BIN_EXE_tide-admit"))
            .arg("--tide-diff")
            .args([base, head])
            .output()
            .unwrap();
        assert_eq!(
            out.status.code(),
            Some(0),
            "{}",
            String::from_utf8_lossy(&out.stderr)
        );
        String::from_utf8(out.stdout).unwrap()
    };
    let source = "\
use std::time::Clock;
pub struct Db {
    pub clock: Clock,
}
pub struct App {
    pub db: Db,
}
pub fn handle(app: &App, req: &str) -> usize {
    req.len()
}
";
    let base = snapshot("base.json", source);
    let body = snapshot(
        "body.json",
        &source.replace("req.len()", "req.len() + req.trim().len()"),
    );
    assert_eq!(diff(&base, &body), "");

    let head = snapshot(
        "head.json",
        &source
            .replace(
                "    pub clock: Clock,\n",
                "    pub clock: Clock,\n    net: std::net::Net,\n",
            )
            .replace("req.len()", "\n    // SAFETY: this metadata-only fixture is never executed.\n    unsafe { req.as_ptr().read() as usize }"),
    );
    let expected = "\
declared authority (no exercised authority, effects, or proof status)
  ~ lib::App (struct) widened
      + db.net: net
  ~ lib::Db (struct) widened
      + net: net
  ~ lib::handle (function) widened
      + app.db.net: net
trust holes
  + lib::handle (function): an `unsafe` block (1)
  + lib::handle (function): uses `*const T` (1)
  + lib::handle (function): uses `core::ptr::const_ptr::<impl *const T>::read` (1)
";
    assert_eq!(diff(&base, &head), expected);

    let out = Command::new(env!("CARGO_BIN_EXE_tide-admit"))
        .args(["--tide-diff", "missing.json"])
        .arg(&base)
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(2));
}

#[test]
fn trait_ceiling_rejects_definitions_and_accepts_their_rewrites() {
    for (name, rejected, accepted, code) in [
        (
            "placement",
            "pub struct S; mod child { impl super::S {} }",
            "pub struct S; impl S {}",
            "TD0401",
        ),
        (
            "overlap",
            "pub trait K {} impl<T> K for T {} impl K for u8 {}",
            "pub trait K {} pub struct S; impl K for S {}",
            "TD0402",
        ),
        (
            "gat",
            "pub trait K { type A<T>; }",
            "pub trait K { type A; }",
            "TD0403",
        ),
        (
            "binder",
            "pub type Callback = for<'a> fn(&'a u8) -> &'a u8;",
            "pub type Callback = fn(&u8) -> &u8;",
            "TD0404",
        ),
        ("auto", "pub auto trait K {}", "pub trait K {}", "TD0405"),
        (
            "bounds",
            "pub trait K {} impl<T> K for T where Vec<T>: K {}",
            "pub trait K {} impl<T: K> K for Vec<T> {}",
            "TD0406",
        ),
        (
            "recursion",
            "pub fn f<T>(x: T) { f(vec![x]); }",
            "pub fn f<A, B>(a: A, b: B) { f(b, a); }",
            "TD0407",
        ),
        (
            "names",
            "pub struct S<T>(T); impl S<u8> { pub fn new() {} } impl S<u16> { pub fn new() {} }",
            "pub struct S<T>(T); impl S<u8> { pub fn new_byte() {} } impl S<u16> { pub fn new_word() {} }",
            "TD0408",
        ),
    ] {
        let (exit, err) = admit(
            &format!("traits-{name}-reject"),
            &format!("#![allow(warnings)]\n{rejected}"),
            &[],
        );
        assert_eq!(exit, 1, "{name}: {err}");
        assert!(err.contains(&format!("error: {code}:")), "{name}: {err}");
        if name == "overlap" {
            assert!(err.contains("E0119"), "{err}");
        }
        let (exit, err) = admit(&format!("traits-{name}-accept"), accepted, &[]);
        assert_eq!(exit, 0, "{name}: {err}");
    }
}

#[test]
fn trait_placement_follows_fundamental_wrappers_to_the_exact_type_module() {
    let accepted = include_str!("../../../conformance/cases/memory/trait_placement_accept.rs");
    let rejected = include_str!("../../../conformance/cases/memory/trait_placement_reject.rs");
    let (exit, err) = admit("trait-placement-wrappers-accept", accepted, &[]);
    assert_eq!(exit, 0, "{err}");
    let (exit, err) = admit("trait-placement-wrappers-reject", rejected, &[]);
    assert_eq!(exit, 1, "{err}");
    let count = rejected.matches("//~ TD0401").count();
    assert_eq!(td_lines(&err).len(), count, "{err}");
    assert_eq!(err.matches("error: TD0401:").count(), count, "{err}");
    assert_eq!(err.matches("move the impl").count(), count - 1, "{err}");
    assert_eq!(
        err.matches("no module can hold this impl").count(),
        1,
        "{err}"
    );
}

#[test]
fn trait_placement_explains_when_no_module_can_hold_the_impl() {
    let source = "pub struct Local; impl From<Local> for u16 { fn from(_: Local) -> u16 { 0 } }";
    let (exit, err) = admit("trait-placement-no-module", source, &[]);
    assert_eq!(exit, 1, "{err}");
    assert!(err.contains("TD0401: no module can hold this impl"), "{err}");
    assert!(err.contains("Into<u16> for Local"), "{err}");
    assert!(err.contains("newtype"), "{err}");
    assert!(!err.contains("move the impl"), "{err}");
}

#[test]
fn trait_recursion_distinguishes_constructors_projections_and_selected_impl_args() {
    let accepted = include_str!("../../../conformance/cases/memory/trait_recursion_accept.rs");
    let rejected = include_str!("../../../conformance/cases/memory/trait_recursion_reject.rs");
    let (exit, err) = admit("trait-recursion-regressions-accept", accepted, &[]);
    assert_eq!(exit, 0, "{err}");
    let (exit, err) = admit("trait-recursion-regressions-reject", rejected, &[]);
    assert_eq!(exit, 1, "{err}");
    assert_eq!(
        err.matches("error: TD0407:").count(),
        rejected.matches("//~ TD0407").count(),
        "{err}"
    );
}

#[test]
fn ambient_operations_use_the_closed_policy_code() {
    let source = r#"pub fn inspect(start: std::time::Instant, rx: &std::sync::mpsc::Receiver<u8>) {
    let _ = std::fs::read("settings");
    let _ = std::fs::File::open("settings");
    let _ = std::env::args();
    let _ = std::env::var("MODE");
    let _ = std::time::Instant::now();
    let _ = start.elapsed();
    std::thread::sleep(std::time::Duration::ZERO);
    let _ = std::process::exit;
    let _ = std::process::abort;
    let _ = std::process::Command::new("tool");
    let _ = std::io::stdin();
    let _ = std::net::TcpStream::connect("127.0.0.1:1");
    let _ = std::net::TcpListener::bind("127.0.0.1:1");
    let _ = std::net::UdpSocket::bind("127.0.0.1:1");
    let _ = rx.recv_timeout(std::time::Duration::ZERO);
    std::thread::park_timeout(std::time::Duration::ZERO);
    std::thread::park_timeout_ms(0);
    std::thread::sleep_ms(0);
}"#;
    let (code, stderr) = admit("ambient-policy", source, &[]);
    assert_eq!(code, 1, "{stderr}");
    for line in 2..=19 {
        assert!(
            td_lines(&stderr)
                .iter()
                .any(|d| d.starts_with(&format!("lib.rs:{line}:")) && d.contains("TD0501:")),
            "line {line}: {stderr}"
        );
    }
}

#[test]
fn concurrency_and_error_rules_reject_with_their_own_codes() {
    for (name, source, expected) in [
        ("async", "pub async fn run() {}", "TD0601"),
        (
            "future",
            "pub fn run(_: impl core::future::Future<Output = ()>) {}",
            "TD0601",
        ),
        ("pin", "pub fn run(_: core::pin::Pin<&u8>) {}", "TD0601"),
        (
            "spawn",
            "pub fn run() { let _ = std::thread::spawn(|| ()); }",
            "TD0602",
        ),
        (
            "mutex",
            "pub fn run() { let _ = std::sync::Mutex::new(1); }",
            "TD0604",
        ),
        ("select", "pub fn run() { select! {}; }", "TD0605"),
        (
            "catch",
            "pub fn run() { let _ = std::panic::catch_unwind(|| ()); }",
            "TD0701",
        ),
        (
            "result",
            "#[allow(unused_must_use)] pub fn run() { Ok::<(), core::convert::Infallible>(()); }",
            "TD0702",
        ),
    ] {
        let (status, stderr) = admit(&format!("habits-{name}"), source, &[]);
        assert_eq!(status, 1, "{name}: {stderr}");
        assert!(
            stderr.contains(&format!("error: {expected}:")),
            "{name}: {stderr}"
        );
        assert!(
            !stderr.contains(&format!("warning: {expected}:")),
            "{name}: {stderr}"
        );
    }
}

#[test]
fn concurrency_select_catches_resolved_and_renamed_macros() {
    for (name, source) in [
        (
            "resolved",
            "macro_rules! select { () => { () } } pub fn run() { select!(); }",
        ),
        (
            "renamed",
            "macro_rules! branch { () => { () } } use branch as select; pub fn run() { select!(); }",
        ),
    ] {
        let (status, stderr) = admit(&format!("habits-select-{name}"), source, &[]);
        assert_eq!(status, 1, "{stderr}");
        assert!(stderr.contains("error: TD0605:"), "{stderr}");
        assert!(!stderr.contains("TD0103:"), "{stderr}");
    }
}

#[test]
fn concurrency_tokio_module_imports_keep_the_spawn_diagnostic() {
    for (name, source, line) in [
        (
            "alias",
            "use tokio as runtime;\npub fn run() { let _ = runtime::spawn(|| 1); }",
            2,
        ),
        (
            "reexport",
            "mod exports { pub use tokio as runtime; }\npub fn run() { let _ = exports::runtime::spawn(|| 1); }",
            2,
        ),
        (
            "reimport",
            "mod exports { pub use tokio as runtime; }\nuse exports::runtime as relay;\npub fn run() { let _ = relay::spawn(|| 1); }",
            3,
        ),
        (
            "reexport-name",
            "mod exports { pub use tokio; }\npub fn run() { let _ = exports::tokio::spawn(|| 1); }",
            2,
        ),
        (
            "import-name",
            "use tokio;\npub fn run() { let _ = tokio::spawn(|| 1); }",
            2,
        ),
    ] {
        let (status, stderr) = admit(&format!("habits-tokio-{name}"), source, &[]);
        assert_eq!(status, 1, "{stderr}");
        assert!(
            td_lines(&stderr).iter().any(|diagnostic| diagnostic
                .starts_with(&format!("lib.rs:{line}:"))
                && diagnostic.contains("error: TD0602:")),
            "{name}: {stderr}"
        );
    }
}

#[test]
fn a_module_file_inside_the_snapshot_is_accepted() {
    let dir = PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join("module-file/package");
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(dir.join("leaf.rs"), "pub struct T;").unwrap();
    let (code, stderr) = admit(
        "module-file",
        "mod leaf; pub fn make() -> leaf::T { leaf::T }",
        &[],
    );
    assert_eq!(code, 0, "{stderr}");
}

#[test]
fn a_rejected_import_has_no_secondary_unused_diagnostic() {
    let (code, stderr) = admit("rejected-import-usage", "use core::sync::atomic::AtomicU64;", &[]);
    assert_eq!(code, 1, "{stderr}");
    assert!(stderr.contains("error: TD0520:"), "{stderr}");
    assert!(!stderr.contains("TD0913"), "{stderr}");
}

#[test]
fn a_rejected_import_does_not_hide_an_unused_list_sibling() {
    let (code, stderr) = admit(
        "rejected-import-sibling",
        "use core::{\n    sync::atomic::AtomicU64,\n    cmp::Ordering,\n};\npub fn f(_: AtomicU64) {}",
        &[],
    );
    assert_eq!(code, 1, "{stderr}");
    assert!(stderr.contains("error: TD0520:"), "{stderr}");
    assert!(
        stderr.lines().any(|line| line.starts_with("lib.rs:3:") && line.contains("error: TD0913:")),
        "{stderr}"
    );
}

#[test]
fn unresolved_traits_keep_unrelated_walk_diagnostics() {
    for (name, broken, rustc) in [
        ("bound", "pub fn b<F: Missing>(_: F) {}", "E0405"),
        (
            "path-bound",
            "pub fn b<F: std::nope::Missing>(_: F) {}",
            "E0433",
        ),
        ("type", "pub fn b(_: Missing) {}", "E0425"),
        (
            "wrong-kind",
            "pub struct Missing; pub fn b<F: Missing>(_: F) {}",
            "E0404",
        ),
        (
            "tainted-ambient",
            "pub fn b<F: Missing>(_: F) { let _ = std::time::Instant::now(); }",
            "E0405",
        ),
    ] {
        let source = format!(
            "pub fn a() {{ let _ = std::fs::read(\"x\"); }}\n{broken}\npub fn c(x: u16) -> u8 {{ x as u8 }}\npub fn d() {{ Ok::<(), ()>(()); }}\npub fn e() {{ println!(\"hello\"); }}\n"
        );
        let (status, stderr) = admit(&format!("unresolved-trait-{name}"), &source, &[]);
        assert_eq!(status, 1, "{stderr}");
        assert!(stderr.contains(&format!("error[{rustc}]")), "{stderr}");
        if name == "tainted-ambient" {
            assert!(
                td_lines(&stderr)
                    .iter()
                    .any(|line| line.starts_with("lib.rs:2:") && line.contains("TD0501:")),
                "{stderr}"
            );
        }
        assert!(
            td_lines(&stderr)
                .iter()
                .any(|line| line.starts_with("lib.rs:1:") && line.contains("error: TD0501:")),
            "{stderr}"
        );
        for (line, severity, code) in [
            (3, "error", "TD0301"),
            (4, "error", "TD0702"),
            (5, "warning", "TD0508"),
        ] {
            assert!(
                stderr.lines().any(
                    |diagnostic| diagnostic.starts_with(&format!("lib.rs:{line}:"))
                        && diagnostic.contains(&format!("{severity}: {code}:"))
                ),
                "{stderr}"
            );
        }
        assert!(
            !stderr.contains("internal compiler error") && !stderr.contains("panicked"),
            "{stderr}"
        );
    }
}

#[test]
fn unresolved_traits_do_not_invent_diagnostics_on_unknown_types() {
    for (name, source) in [
        (
            "call",
            "pub fn bad<T: Missing>(_: T) {} pub fn caller() { bad(1); }",
        ),
        (
            "method",
            "pub struct S; impl S { pub fn bad<T: Missing>(&self, _: T) {} } pub fn caller(s: S) { s.bad(1); }",
        ),
        (
            "impl",
            "pub trait Good {} pub struct S; impl Good for S where S: Missing {} pub fn caller<T: Good>(_: T) {} pub fn run() { caller(S); }",
        ),
        (
            "supertrait",
            "pub trait Good: Missing { fn call(&self); } pub fn caller<T: Good>(x: T) { x.call(); }",
        ),
        (
            "alias",
            "pub type Unknown = dyn Missing; pub fn caller(_: &Unknown) {}",
        ),
        (
            "closure",
            "pub fn caller() { let _ = |_: &dyn Missing| {}; }",
        ),
        (
            "nested",
            "pub fn caller() { fn bad<T: Missing>(_: T) {} bad(1); }",
        ),
        (
            "unknown",
            "pub fn caller<T: Missing>(x: T) { x.absent(); x; }",
        ),
        (
            "opaque-return",
            "pub fn unknown() -> impl Missing { 0 } pub fn caller() { let _ = unknown(); }",
        ),
        (
            "blanket-impl",
            "impl<T: Missing> core::fmt::Debug for T { fn fmt(&self, _: &mut core::fmt::Formatter<'_>) -> core::fmt::Result { Ok(()) } } pub fn caller() { let _ = format!(\"{:?}\", 0u8); }",
        ),
        ("unsafe-impl", "pub struct S; unsafe impl Missing for S {}"),
        (
            "nested-impl",
            "pub struct S; impl Missing for S { fn f() { mod nested { pub fn g() {} } nested::g(); nested::absent(); } }",
        ),
    ] {
        let source = format!("{source}\npub fn other() {{ let _ = std::fs::read(\"x\"); }}");
        let flags: &[&str] = if name == "unsafe-impl" {
            &["--tide-grant=unsafe"]
        } else {
            &[]
        };
        let (status, stderr) = admit(&format!("unresolved-dependent-{name}"), &source, flags);
        assert_eq!(status, 1, "{name}: {stderr}");
        assert!(stderr.contains("E0405"), "{name}: {stderr}");
        assert!(
            td_lines(&stderr)
                .iter()
                .any(|line| line.starts_with("lib.rs:2:") && line.contains("TD0501:")),
            "{name}: {stderr}"
        );
        assert!(
            td_lines(&stderr)
                .iter()
                .all(|line| line.starts_with("lib.rs:2:")
                    || (name == "blanket-impl"
                        && line.starts_with("lib.rs:1:")
                        && line.contains("error: TD0401:"))),
            "{name}: {stderr}"
        );
        if name == "blanket-impl" {
            // A foreign trait on the known type parameter is misplaced even with a bad bound.
            assert_eq!(
                td_lines(&stderr)
                    .iter()
                    .filter(|line| line.starts_with("lib.rs:1:") && line.contains("error: TD0401:"))
                    .count(),
                1,
                "{stderr}"
            );
        }
        assert!(
            !stderr.contains("internal compiler error") && !stderr.contains("panicked"),
            "{name}: {stderr}"
        );
    }
}

#[test]
fn unresolved_future_traits_keep_their_walk_diagnostic() {
    let std = tide_std("unresolved-future");
    for (name, source, rustc, line) in [
        (
            "impl",
            "pub struct S;\nimpl std::future::Future for S {}",
            "E0405",
            2,
        ),
        (
            "bound",
            "pub fn b<F: std::future::Future>(_: F) {}",
            "E0405",
            1,
        ),
        (
            "dyn",
            "pub fn b(_: &dyn std::future::Future<Output = ()>) {}",
            "E0405",
            1,
        ),
        (
            "impl-arg",
            "pub fn b(_: impl std::future::Future<Output = ()>) {}",
            "E0405",
            1,
        ),
        (
            "import",
            "use std::future::Future;\npub fn b(_: impl Future<Output = ()>) {}",
            "E0432",
            1,
        ),
    ] {
        let (status, stderr) = admit(
            &format!("unresolved-future-{name}"),
            source,
            &["--tide-std", std.to_str().unwrap()],
        );
        assert_eq!(status, 1, "{stderr}");
        assert!(stderr.contains(&format!("error[{rustc}]")), "{stderr}");
        assert!(
            td_lines(&stderr).iter().any(|diagnostic| diagnostic
                .starts_with(&format!("lib.rs:{line}:"))
                && diagnostic.contains("error: TD0601:")),
            "{stderr}"
        );
        assert!(
            !stderr.contains("internal compiler error") && !stderr.contains("panicked"),
            "{stderr}"
        );
    }
}

#[test]
fn typeck_error_recovery_keeps_known_authority_diagnostics() {
    let std = tide_std("tainted-authority");
    let source = "pub fn run() { let _: Missing; let _ = std::Handle::new(); let _ = std::time::Instant::now(); }";
    let (status, stderr) = admit(
        "tainted-authority",
        source,
        &["--tide-std", std.to_str().unwrap()],
    );
    assert_eq!(status, 1, "{stderr}");
    for code in ["E0425", "E0061"] {
        assert!(stderr.contains(&format!("error[{code}]")), "{stderr}");
    }
    for code in ["TD0501", "TD0505"] {
        assert!(
            td_lines(&stderr)
                .iter()
                .any(|line| line.starts_with("lib.rs:1:")
                    && line.contains(&format!("error: {code}:"))),
            "{stderr}"
        );
    }
}

#[test]
fn unresolved_traits_cover_anonymous_const_bodies() {
    let std = tide_std("unresolved-anon");
    for (name, body, expected) in [
        (
            "call",
            "pub fn b<T: Missing>(_: T) {}\npub fn c() { let _: [u8; { b(0); 0 }]; }",
            "TD0501",
        ),
        (
            "future",
            "pub fn c() { let _: [u8; { let _: &dyn std::future::Future<Output = ()>; 0 }]; }",
            "TD0601",
        ),
    ] {
        let source = format!("pub fn a() {{ let _ = std::fs::read(\"x\"); }}\n{body}");
        let (status, stderr) = admit(
            &format!("unresolved-anon-{name}"),
            &source,
            &["--tide-std", std.to_str().unwrap()],
        );
        assert_eq!(status, 1, "{stderr}");
        assert!(stderr.contains("E0405"), "{stderr}");
        assert!(
            td_lines(&stderr)
                .iter()
                .any(|line| line.starts_with("lib.rs:1:") && line.contains("TD0501:")),
            "{stderr}"
        );
        assert!(stderr.contains(&format!("error: {expected}:")), "{stderr}");
    }
}

#[test]
fn unresolved_trait_aliases_guard_implicit_method_lookup() {
    for (name, declarations, caller, rustc) in [
        (
            "method",
            "pub trait Bad = Missing;",
            "pub fn c(x: u8) { x.absent(); }",
            "E0405",
        ),
        (
            "valid-method",
            "pub trait Bad = Missing;",
            "pub fn c(x: u8) { let _ = x.to_string(); }",
            "E0405",
        ),
        (
            "import",
            "mod inner { pub trait Bad = Missing; } use inner::Bad;",
            "pub fn c(x: u8) { x.absent(); }",
            "E0405",
        ),
        (
            "nested",
            "",
            "pub fn c(x: u8) { trait Bad = Missing; x.absent(); }",
            "E0405",
        ),
        (
            "wrong-kind",
            "pub struct Missing; pub trait Bad = Missing;",
            "pub fn c(x: u8) { x.absent(); }",
            "E0404",
        ),
        (
            "suggestion",
            "mod inner { pub trait Bad = Missing; }",
            "pub fn c(x: u8) { x.absent(); }",
            "E0405",
        ),
    ] {
        let source =
            format!("pub fn a() {{ let _ = std::fs::read(\"x\"); }}\n{declarations}\n{caller}");
        let (status, stderr) = admit(&format!("unresolved-trait-alias-{name}"), &source, &[]);
        assert_eq!(status, 1, "{stderr}");
        assert!(
            stderr.contains("E0658") && stderr.contains(rustc),
            "{stderr}"
        );
        assert!(
            td_lines(&stderr)
                .iter()
                .any(|line| line.starts_with("lib.rs:1:") && line.contains("error: TD0501:")),
            "{stderr}"
        );
    }
}

#[test]
fn unresolved_global_traits_keep_independent_typed_diagnostics() {
    for (name, bad) in [
        ("trait", "pub trait B: Missing {}"),
        ("implementee", "pub struct S; impl Missing for S {}"),
        (
            "inherent",
            "pub struct S; impl<T: Missing> S { pub fn m() {} }",
        ),
        (
            "blanket",
            "impl<T: Missing> core::fmt::Debug for T { fn fmt(&self, _: &mut core::fmt::Formatter<'_>) -> core::fmt::Result { Ok(()) } }",
        ),
        (
            "supertrait-impl",
            "pub trait B: Missing {} pub struct S; impl B for S {}",
        ),
    ] {
        let source = format!(
            "{bad}\npub fn c(x: u16) -> u8 {{ x as u8 }}\npub fn d() {{ Ok::<(), ()>(()); }}"
        );
        let (status, stderr) = admit(&format!("unresolved-global-{name}"), &source, &[]);
        assert_eq!(status, 1, "{stderr}");
        assert!(stderr.contains("E0405"), "{stderr}");
        for (line, code) in [(2, "TD0301"), (3, "TD0702")] {
            assert!(
                td_lines(&stderr).iter().any(|diagnostic| diagnostic
                    .starts_with(&format!("lib.rs:{line}:"))
                    && diagnostic.contains(&format!("error: {code}:"))),
                "{stderr}"
            );
        }
    }
}

#[test]
fn partial_typeck_keeps_known_casts_without_inventing_unknown_ones() {
    for (name, value, expected) in [("known", "x", true), ("unknown", "missing", false)] {
        let source =
            format!("pub fn c(x: u16) -> u8 {{\n    let _: Missing;\n    {value} as u8\n}}");
        let (status, stderr) = admit(&format!("partial-cast-{name}"), &source, &[]);
        assert_eq!(status, 1, "{stderr}");
        assert!(stderr.contains("E0425"), "{stderr}");
        assert_eq!(
            td_lines(&stderr)
                .iter()
                .any(|line| line.starts_with("lib.rs:3:") && line.contains("error: TD0301:")),
            expected,
            "{stderr}"
        );
    }
}

#[test]
fn unresolved_impl_traits_guard_coercions_and_bounded_adts() {
    let local = "pub trait Good {} pub struct S; impl Good for S where S: Missing {}";
    let blanket = "impl<T: Missing> core::fmt::Debug for T { fn fmt(&self, _: &mut core::fmt::Formatter<'_>) -> core::fmt::Result { Ok(()) } }";
    for (name, bad, consumer) in [
        ("local-coerce", local, "pub fn c(x: &S) -> &dyn Good { x }"),
        (
            "local-cast",
            local,
            "pub fn c(x: &S) { let _ = x as &dyn Good; }",
        ),
        (
            "local-ctor",
            local,
            "pub struct Wrap<T: Good>(T); pub fn c(x: S) { let _ = Wrap(x); }",
        ),
        (
            "local-literal",
            local,
            "pub struct Wrap<T: Good> { x: T } pub fn c(x: S) { let _ = Wrap { x }; }",
        ),
        (
            "local-pattern",
            local,
            "pub struct Wrap<T: Good>(T); pub fn c(Wrap(_): Wrap<S>) {}",
        ),
        ("local-rpit", local, "pub fn c(x: S) -> impl Good { x }"),
        (
            "blanket-debug",
            blanket,
            "pub fn c(x: &u8) -> &dyn core::fmt::Debug { x }",
        ),
        (
            "implicit-deref",
            "pub struct S; impl core::ops::Deref for S where S: Missing { type Target = u8; fn deref(&self) -> &u8 { &0 } }",
            "pub fn c(x: &S) -> &u8 { x }",
        ),
        (
            "external-trait-callee",
            "pub struct S; impl<T: Missing> core::fmt::Binary for T { fn fmt(&self, _: &mut core::fmt::Formatter<'_>) -> core::fmt::Result { Ok(()) } }",
            "pub fn c(x: &S, f: &mut core::fmt::Formatter<'_>) { let _ = <S as core::fmt::Binary>::fmt(x, f); }",
        ),
    ] {
        let source = format!(
            "pub fn a() {{ let _ = std::fs::read(\"x\"); }}\n{bad}\n{consumer}\npub fn later(x: u16) -> u8 {{ x as u8 }}\npub fn result() {{ Ok::<(), ()>(()); }}"
        );
        let (status, stderr) = admit(&format!("unresolved-impl-trait-{name}"), &source, &[]);
        assert_eq!(status, 1, "{stderr}");
        assert!(stderr.contains("E0405"), "{stderr}");
        for (line, code) in [(1, "TD0501"), (4, "TD0301"), (5, "TD0702")] {
            assert_eq!(
                td_lines(&stderr)
                    .iter()
                    .filter(
                        |diagnostic| diagnostic.starts_with(&format!("lib.rs:{line}:"))
                            && diagnostic.contains(&format!("error: {code}:"))
                    )
                    .count(),
                1,
                "{stderr}"
            );
        }
    }
}

#[test]
fn unrelated_bad_traits_keep_independent_const_parameter_checks() {
    let std = tide_std("unrelated-const");
    let source = "pub trait Bad: Missing {}\npub const fn c(_: std::time::Clock) {}";
    let (status, stderr) = admit(
        "unrelated-const",
        source,
        &["--tide-std", std.to_str().unwrap()],
    );
    assert_eq!(status, 1, "{stderr}");
    assert!(stderr.contains("E0405"), "{stderr}");
    assert!(
        td_lines(&stderr)
            .iter()
            .any(|line| line.starts_with("lib.rs:2:") && line.contains("error: TD0518:")),
        "{stderr}"
    );
}

#[test]
fn fatal_global_queries_keep_later_walk_diagnostics() {
    for (name, bad, hazard) in [
        (
            "operator-index",
            "pub struct W<T: ?Sized>(Box<T>); impl core::ops::Add for W<dyn Missing> { type Output = u8; fn add(self, _: Self) -> u8 { 0 } }",
            "pub fn hazard(x: u8) -> u8 { x + 1 }",
        ),
        (
            "inherent-index",
            "impl dyn Missing { pub fn m(&self) {} }",
            "pub fn hazard(x: i32) -> i32 { x.abs() }",
        ),
        (
            "drop-coherence",
            "pub struct W<T: ?Sized>(Box<T>); impl Drop for W<dyn Missing> { fn drop(&mut self) {} } pub struct T;",
            "pub fn hazard(x: u16) { let _ = x as u8; Ok::<(), ()>(()); let _ = T; }",
        ),
        (
            "derive-precision",
            "#[derive(Debug)] pub struct S<T: Missing>(T);",
            "pub fn hazard() {}",
        ),
    ] {
        let method = if name == "derive-precision" {
            "let _ = \"a\".len(); "
        } else {
            ""
        };
        let source = format!(
            "pub fn a() {{ let _ = std::fs::read(\"x\"); }}\n{bad}\n{hazard}\npub fn later(x: u16) -> u8 {{ {method}x as u8 }}\npub fn result() {{ {method}Ok::<(), ()>(()); }}"
        );
        let (status, stderr) = admit(&format!("fatal-global-{name}"), &source, &[]);
        assert_eq!(status, 1, "{stderr}");
        assert!(stderr.contains("error[E0405]"), "{stderr}");
        let mut expected = vec![(1, "TD0501"), (4, "TD0301"), (5, "TD0702")];
        if name == "drop-coherence" {
            expected.extend([(3, "TD0301"), (3, "TD0702")]);
        }
        for (line, code) in expected {
            assert_eq!(
                td_lines(&stderr)
                    .iter()
                    .filter(
                        |diagnostic| diagnostic.starts_with(&format!("lib.rs:{line}:"))
                            && diagnostic.contains(&format!("error: {code}:"))
                    )
                    .count(),
                1,
                "{name}: {stderr}"
            );
        }
        assert!(
            !stderr.contains("internal compiler error") && !stderr.contains("panicked"),
            "{name}: {stderr}"
        );
    }
}

#[test]
fn partial_typeck_keeps_known_pure_and_parallel_obligations() {
    let std = tide_std("partial-obligations");
    let source = "\
use std::par::ParSlice;
fn need_send<T: Send + ?Sized>(_: &T) {} fn need_pure<T: Pure + ?Sized>(_: &T) {}
pub fn object(f: &dyn Fn()) { need_send(f); need_pure(f); }
pub fn owned() -> (u64, u64) { let n = std::rc::Rc::new(1); std::par::join(move || *n, || 2) }
pub fn captured(log: &std::cell::RefCell<Vec<u64>>) -> (usize, u64) { std::par::join(|| log.borrow().len(), || 2) }
pub fn mutable(xs: &mut [u64]) { let mut f = |_: &mut u64| {}; xs.par_for(&mut f); }
pub fn unknown(x: Missing) { std::par::join(x, || 2u64); }
";
    let (status, stderr) = admit(
        "partial-obligations",
        source,
        &["--tide-std", std.to_str().unwrap()],
    );
    assert_eq!(status, 1, "{stderr}");
    assert!(stderr.contains("error[E0277]"), "{stderr}");
    assert!(stderr.contains("error[E0425]"), "{stderr}");
    for (line, code) in [(3, "TD0517"), (4, "TD0610"), (5, "TD0611"), (6, "TD0610")] {
        assert_eq!(
            td_lines(&stderr)
                .iter()
                .filter(
                    |diagnostic| diagnostic.starts_with(&format!("lib.rs:{line}:"))
                        && diagnostic.contains(&format!("error: {code}:"))
                )
                .count(),
            1,
            "{stderr}"
        );
    }
    assert!(
        !td_lines(&stderr)
            .iter()
            .any(|line| line.starts_with("lib.rs:7:")),
        "{stderr}"
    );
    assert!(
        !stderr.contains("panicked") && !stderr.contains("internal compiler error"),
        "{stderr}"
    );
}

#[test]
fn contract_scan_keeps_recovery_paths_inside_an_unresolved_impl() {
    let source = "\
pub struct S;
impl Missing for S {
    fn f() {
        mod nested { pub fn g() {} }
        macro_rules! call { () => { nested::g() } }
        call!();
        let _ = std::fs::read(\"x\");
    }
}
";
    let (status, stderr) = admit("contract-scan-recovery", source, &[]);
    assert_eq!(status, 1, "{stderr}");
    assert!(stderr.contains("error[E0405]"), "{stderr}");
    for (line, code) in [(5, "TD0101"), (6, "TD0103"), (7, "TD0501")] {
        assert_eq!(
            td_lines(&stderr)
                .iter()
                .filter(
                    |diagnostic| diagnostic.starts_with(&format!("lib.rs:{line}:"))
                        && diagnostic.contains(&format!("error: {code}:"))
                )
                .count(),
            1,
            "{stderr}"
        );
    }
    assert!(
        !stderr.contains("panicked") && !stderr.contains("internal compiler error"),
        "{stderr}"
    );
}

#[test]
fn partial_typeck_keeps_raw_read_pure_obligations() {
    let std = tide_std("partial-raw-reads");
    let source = "\
use std::time::Clock;
use std::mem::ManuallyDrop;
pub union Pun { bits: u64, clock: ManuallyDrop<Clock> }
pub fn deref(p: *const Clock) -> &'static Clock {
    let _: u8 = \"x\";
    unsafe { &*p }
}
pub fn field(p: &Pun) -> &Clock {
    let _: u8 = \"x\";
    unsafe { &p.clock }
}
pub fn pattern(p: Pun) {
    let _: u8 = \"x\";
    unsafe { let Pun { clock: _clock } = p; }
}
";
    let (status, stderr) = admit(
        "partial-raw-reads",
        source,
        &["--tide-std", std.to_str().unwrap(), "--tide-grant=unsafe"],
    );
    assert_eq!(status, 1, "{stderr}");
    for (error, td) in [(5, 6), (9, 10), (13, 14)] {
        assert!(
            stderr
                .lines()
                .any(|line| line.starts_with(&format!("lib.rs:{error}:"))
                    && line.contains("error[E0308]")),
            "{stderr}"
        );
        assert_eq!(
            td_lines(&stderr)
                .iter()
                .filter(|line| line.starts_with(&format!("lib.rs:{td}:"))
                    && line.contains("error: TD0517:"))
                .count(),
            1,
            "{stderr}"
        );
    }
    assert!(
        !stderr.contains("panicked") && !stderr.contains("internal compiler error"),
        "{stderr}"
    );
}

/// A Tide diagnostic as an agent reads it, placed relative to the fixture block that owns it:
/// (block, line within the block, column, severity, code, message).
type Placed = (&'static str, usize, usize, String, String, String);

/// A fixture block: its name, whether it owns diagnostics, and its independent top-level items.
type Block = (&'static str, bool, &'static str);

/// Lays out `blocks` in order as one library, with `gap` before each; returns the source and each
/// block's name with its first line and the line after its last.
fn lay_out(blocks: &[Block], gap: &str) -> (String, Vec<(&'static str, usize, usize)>) {
    let mut source = String::new();
    let mut lines = Vec::new();
    for &(name, _, items) in blocks {
        source.push_str(gap);
        let first = source.lines().count() + 1;
        source.push_str(items);
        lines.push((name, first, first + items.lines().count()));
    }
    (source, lines)
}

/// Places each Tide diagnostic in `stderr` in the block whose lines hold its start; the multiset
/// comes back sorted. A diagnostic that is malformed or that no block owns is an error.
fn place(stderr: &str, lines: &[(&'static str, usize, usize)]) -> Result<Vec<Placed>, String> {
    let mut placed = Vec::new();
    for diagnostic in stderr.lines() {
        let Some(severity) = ["error", "warning"]
            .into_iter()
            .find(|s| diagnostic.contains(&format!(": {s}: TD")))
        else {
            continue;
        };
        let parts = (|| {
            let (line, rest) = diagnostic.strip_prefix("lib.rs:")?.split_once(':')?;
            let (column, rest) = rest.split_once(": ")?;
            let (code, message) = rest
                .strip_prefix(severity)?
                .strip_prefix(": ")?
                .split_once(": ")?;
            Some((
                line.parse::<usize>().ok()?,
                column.parse().ok()?,
                code,
                message,
            ))
        })();
        let Some((line, column, code, message)) = parts else {
            return Err(format!("unparsed: {diagnostic}"));
        };
        let Some(&(block, first, _)) = lines.iter().find(|&&(_, a, b)| (a..b).contains(&line))
        else {
            return Err(format!("in no block: {diagnostic}"));
        };
        placed.push((
            block,
            line - first + 1,
            column,
            severity.into(),
            code.into(),
            message.into(),
        ));
    }
    placed.sort();
    Ok(placed)
}

/// Harmless edits keep every Tide diagnostic (D14): reversing independent items, inserting blank
/// and comment lines, adding an unrelated accepted item, and running the driver again change no
/// diagnostic's block, place within it, severity, code, or message, and lose or add none. A
/// walk rejection, a backstop rejection (which runs only when the walk finds nothing), and an
/// accepted control each go through the same edits.
#[test]
fn diagnostics_are_stable_under_harmless_source_edits() {
    let walk: &[Block] = &[
        (
            "atomic",
            true,
            "pub fn counter() -> u64 {\n    core::sync::atomic::AtomicU64::new(0).into_inner()\n}\n",
        ),
        (
            "env",
            true,
            "pub fn mode() -> bool {\n    std::env::var(\"MODE\").is_ok()\n}\n",
        ),
        (
            "print",
            true,
            "pub fn say(n: u32) {\n    println!(\"{n}\");\n}\n",
        ),
        (
            "double",
            false,
            "pub fn double(x: u32) -> u32 {\n    x * 2\n}\n",
        ),
    ];
    let backstop: &[Block] = &[
        (
            "vtable",
            true,
            "pub fn leaf(x: &fn()) -> &dyn PartialOrd<fn()> {\n    x\n}\n",
        ),
        (
            "format",
            true,
            "pub fn debug<T: std::fmt::Debug>(t: T) -> String {\n    format!(\"{t:?}\")\n}\n\
             pub fn shown(f: fn()) -> String {\n    debug(f)\n}\n",
        ),
    ];
    let accepted: &[Block] = &[
        (
            "total",
            false,
            "pub fn total(xs: &[u32]) -> u32 {\n    xs.iter().sum()\n}\n",
        ),
        (
            "point",
            false,
            "pub struct Point {\n    pub x: i32,\n}\nimpl Point {\n    pub fn shifted(&self) -> Point {\n        \
             Point { x: self.x + 1 }\n    }\n}\n",
        ),
    ];
    let unrelated: Block = (
        "unrelated",
        false,
        "pub fn unrelated(x: u32) -> u32 {\n    x + 1\n}\n",
    );
    let comments = "\n// A comment between independent items.\n\n";
    let mut failures = Vec::new();
    for (case, blocks, flags, exit) in [
        ("walk", walk, &[][..], 1),
        ("backstop", backstop, &["--tide-test-backstop"][..], 1),
        ("accepted", accepted, &[][..], 0),
    ] {
        let check = |variant: &str, blocks: &[Block], gap: &str| {
            let (source, lines) = lay_out(blocks, gap);
            let (code, stderr) = admit(&format!("stable-{case}-{variant}"), &source, flags);
            (code, place(&stderr, &lines), stderr)
        };
        let (code, base, stderr) = check("original", blocks, "");
        assert_eq!(code, exit, "{case}: {stderr}");
        let base = base.unwrap_or_else(|e| panic!("{case}: {e}\n{stderr}"));
        for &(name, flagged, _) in blocks {
            assert_eq!(
                base.iter().any(|d| d.0 == name),
                flagged,
                "{case}/{name}: {stderr}"
            );
        }
        if case == "backstop" {
            // Only the backstop names the instance that runs a denied impl.
            assert!(
                base.iter().all(|d| d.5.contains("(it runs as `")),
                "{stderr}"
            );
        }

        // The comparison sees a lost, a duplicated, or a recoded diagnostic, and a moved line
        // or column.
        if let Some(first) = stderr.lines().find(|l| l.contains(": error: TD")) {
            let (_, lines) = lay_out(blocks, "");
            let (line, rest) = first["lib.rs:".len()..].split_once(':').unwrap();
            let (column, rest) = rest.split_once(':').unwrap();
            let line: usize = line.parse().unwrap();
            let column: usize = column.parse().unwrap();
            let code = &rest[rest.find("TD").unwrap()..][..6];
            for corrupt in [
                stderr.replacen(&format!("{first}\n"), "", 1),
                stderr.replacen(&format!("{first}\n"), &format!("{first}\n{first}\n"), 1),
                stderr.replacen(first, &first.replacen(code, "TD0000", 1), 1),
                stderr.replacen(first, &format!("lib.rs:{}:{column}:{rest}", line + 1), 1),
                stderr.replacen(first, &format!("lib.rs:{line}:{}:{rest}", column + 1), 1),
            ] {
                assert_ne!(place(&corrupt, &lines), Ok(base.clone()), "{corrupt}");
            }
        }

        let reversed: Vec<Block> = blocks.iter().rev().copied().collect();
        let mut extra = vec![unrelated];
        extra.extend_from_slice(blocks);
        let mut all = vec![unrelated];
        all.extend_from_slice(&reversed);
        for (variant, blocks, gap) in [
            ("repeat", blocks, ""),
            ("reversed", &reversed[..], ""),
            ("comments", blocks, comments),
            ("unrelated", &extra[..], ""),
            ("combined", &all[..], comments),
        ] {
            let (code, placed, stderr) = check(variant, blocks, gap);
            if code != exit || placed.as_ref() != Ok(&base) {
                failures.push(format!(
                    "{case}/{variant}: exit {code}, {placed:#?}\nexpected {base:#?}\n{stderr}"
                ));
            }
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}
