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
fn the_backstop_keeps_exported_library_roots_in_every_profile() {
    for (name, source) in [
        (
            "backstop-exported-vtable",
            "pub fn leaf(x: &fn()) -> &dyn PartialOrd<fn()> { x }\n",
        ),
        (
            "backstop-exported-reify",
            "pub fn sorter() -> fn(&mut [fn()]) { <[fn()]>::sort }\n",
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
                fx_lines(&stderr)
                    .iter()
                    .any(|line| line.contains("FX0520:") && line.contains("for fn()`")),
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
    let unmarked = "lib.rs:1:5: error: FX0512: `main` does not carry `#[tide::main]`, which \
                    starts the runtime";
    assert!(fx_lines(&stderr).contains(&unmarked), "{stderr}");
}

/// A minimal Tide std, built with the pinned rustc: Rust's std with `Pure`, which rustc meets
/// for every type and the driver decides, and a `Clock`, which is never `Pure`.
fn tide_std(name: &str) -> PathBuf {
    let dir = PathBuf::from(env!("CARGO_TARGET_TMPDIR"))
        .join(name)
        .join("tide-std");
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
pub mod net {
    pub struct Net(());
}
pub mod sync {
    pub struct Arc<T>(T);
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
        "unsafe { s.as_bytes().as_ptr().read() as usize }",
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

    // `#![no_std]` is not admitted, even with no `extern crate std` (FX0520), and the failed
    // check leaves no artifact; an inactive `cfg_attr` of it is no `#![no_std]`.
    let (code, stderr) = admit("artifact", "#![no_std]\npub fn f() {}\n", &flags);
    assert_eq!(code, 1, "{stderr}");
    assert!(stderr.contains("FX0520: `#![no_std]`"), "{stderr}");
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
    let block = "error: FX0901: an `unsafe extern` block needs the package's `unsafe` grant";
    let item = "error: FX0901: a foreign item needs the package's `unsafe` grant";
    assert_eq!(
        fx_lines(&stderr),
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
            .replace("req.len()", "unsafe { req.as_ptr().read() as usize }"),
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
            "FX0401",
        ),
        (
            "overlap",
            "pub trait K {} impl<T> K for T {} impl K for u8 {}",
            "pub trait K {} pub struct S; impl K for S {}",
            "FX0402",
        ),
        (
            "gat",
            "pub trait K { type A<T>; }",
            "pub trait K { type A; }",
            "FX0403",
        ),
        (
            "binder",
            "pub type Callback = for<'a> fn(&'a u8) -> &'a u8;",
            "pub type Callback = fn(&u8) -> &u8;",
            "FX0404",
        ),
        ("auto", "pub auto trait K {}", "pub trait K {}", "FX0405"),
        (
            "bounds",
            "pub trait K {} impl<T> K for T where Vec<T>: K {}",
            "pub trait K {} impl<T: K> K for Vec<T> {}",
            "FX0406",
        ),
        (
            "recursion",
            "pub fn f<T>(x: T) { f(vec![x]); }",
            "pub fn f<A, B>(a: A, b: B) { f(b, a); }",
            "FX0407",
        ),
        (
            "names",
            "pub struct S<T>(T); impl S<u8> { pub fn new() {} } impl S<u16> { pub fn new() {} }",
            "pub struct S<T>(T); impl S<u8> { pub fn new_byte() {} } impl S<u16> { pub fn new_word() {} }",
            "FX0408",
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
fn trait_placement_explains_when_no_module_can_hold_the_impl() {
    let source = "pub struct Local; impl From<Local> for u16 { fn from(_: Local) -> u16 { 0 } }";
    let (exit, err) = admit("trait-placement-no-module", source, &[]);
    assert_eq!(exit, 1, "{err}");
    assert!(err.contains("FX0401: no module can hold this impl"), "{err}");
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
        err.matches("error: FX0407:").count(),
        rejected.matches("//~ FX0407").count(),
        "{err}"
    );
}
