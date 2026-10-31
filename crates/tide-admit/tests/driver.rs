//! The driver as a crate's build sees it: its exit status and its diagnostics. These run
//! against Rust's own std, which the allowlist checks like `core`; `conformance/run.sh` runs the
//! corpus against Tide's std.

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

fn fx_lines(stderr: &str) -> Vec<&str> {
    stderr
        .lines()
        .filter(|l| l.contains(": error: FX"))
        .collect()
}

#[test]
fn rejects_each_use_outside_the_allowlist_at_its_span() {
    let source = "\
pub fn run(x: f64) -> f64 {
    let v: Vec<u32> = (1..4).map(|n| n * 2).collect();
    let _c = core::sync::atomic::AtomicU64::new(0);
    let s = format!(\"{v:?}\");
    x.sqrt() + x.sin() + s.len() as f64
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
        fx_lines(&stderr),
        // One per position: the item written there, not the call's type.
        [
            "lib.rs:3:14: error: FX0520: `core::sync::atomic::AtomicU64` is not admitted",
            "lib.rs:5:16: error: FX0520: `std::f64::<impl f64>::sin` is not admitted",
            "sub.rs:3:21: error: FX0520: `core::sync::atomic::AtomicU64` is not admitted",
        ],
        "{stderr}"
    );
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
        fx_lines(&stderr),
        ["lib.rs:2:5: error: FX0520: `core::core_arch::x86::cpuid::__cpuid` is not admitted"],
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
        fx_lines(&stderr),
        ["lib.rs:2:5: error: FX0520: `*const T` is not admitted"],
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
    let lines = fx_lines(&stderr);
    let map =
        "impl core::convert::From<[(K, V); N]> for std::collections::hash::map::HashMap<K, V>";
    let set = "impl core::convert::From<[T; N]> for std::collections::hash::set::HashSet<T>";
    assert_eq!(
        lines[..3],
        [
            format!("lib.rs:5:6: error: FX0520: `{map}` is not admitted"),
            format!("lib.rs:5:31: error: FX0520: `{set}` is not admitted"),
            format!("lib.rs:8:5: error: FX0520: `{map}` is not admitted"),
        ],
        "{stderr}"
    );
    assert_eq!(lines.len(), 4, "{stderr}");
    assert!(
        lines[3].starts_with("lib.rs:11:5: error: FX0520: `lib::W<lib::W<")
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
    let lines = fx_lines(&stderr);
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
    let fn_debug = "FX0520: `impl core::fmt::Debug for fn()` is not admitted \
                    (it runs as `std::ptr::<impl Debug for fn()>::fmt`)";
    assert!(
        lines[0].ends_with(
            "FX0520: `impl core::convert::From<[(K, V); N]> for \
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
        fx_lines(&stderr),
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
        fx_lines(&stderr),
        [format!("lib.rs:2:1: error: {fn_debug}")],
        "{stderr}"
    );
}

/// A minimal Tide std, built with the pinned rustc: Rust's std with `Pure`, which rustc meets
/// for every type and the driver decides, and a `Clock`, which is never `Pure`.
fn tide_std() -> PathBuf {
    let dir = PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join("tide-std");
    std::fs::create_dir_all(&dir).unwrap();
    let source = dir.join("lib.rs");
    std::fs::write(
        &source,
        "\
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
    let std = tide_std();
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
    let clock = "FX0517: `Clock` is never `Pure`";
    assert_eq!(
        fx_lines(&stderr),
        [
            format!("lib.rs:7:21: error: {clock}"),
            format!("lib.rs:30:5: error: {clock}"),
            "lib.rs:33:5: error: FX0517: `W<Clock>` is not `Pure`: it reaches `Clock`, which is \
             never `Pure`"
                .to_string(),
            // In `make`, the nearest local item that drops the guards.
            format!("lib.rs:36:25: error: {clock}"),
            format!("lib.rs:42:1: error: {clock}"),
        ],
        "{stderr}"
    );
}
