//! The driver as a crate's build sees it: its exit status and its diagnostics. These run
//! against Rust's own std, which the allowlist checks like `core`; `conformance/run.sh` runs the
//! corpus against Tide's std.

use std::path::PathBuf;
use std::process::Command;

/// Checks `source` as a library with the driver; returns its exit code and short-format stderr.
fn admit(name: &str, source: &str, flags: &[&str]) -> (i32, String) {
    let dir = PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join(name);
    let package = dir.join("package");
    std::fs::create_dir_all(&package).unwrap();
    let file = package.join("lib.rs");
    std::fs::write(&file, source).unwrap();
    let out = Command::new(env!("CARGO_BIN_EXE_tide-admit"))
        .args(flags)
        .args([
            "--edition",
            "2024",
            "--crate-type",
            "lib",
            "--emit",
            "metadata",
            "--error-format=short",
        ])
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
