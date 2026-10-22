//! Lets the driver find the pinned toolchain's `librustc_driver` and `libLLVM` at link and run
//! time, so neither the binary nor its tests need `LD_LIBRARY_PATH`, and records the toolchain's
//! sysroot (`TIDE_SYSROOT`) and identity (`TIDE_COMPILER_ID`) for the invocation and its cache.
use std::process::Command;

fn main() {
    let rustc = std::env::var("RUSTC").unwrap_or_else(|_| "rustc".into());
    let out = Command::new(&rustc)
        .args(["--print", "sysroot"])
        .output()
        .expect("run rustc");
    assert!(out.status.success(), "`{rustc} --print sysroot` failed");
    let sysroot = String::from_utf8(out.stdout).expect("sysroot is UTF-8");
    println!("cargo:rustc-env=TIDE_SYSROOT={}", sysroot.trim());
    let version = Command::new(&rustc)
        .arg("-vV")
        .output()
        .expect("rustc identity");
    assert!(version.status.success(), "rustc identity failed");
    let version = String::from_utf8(version.stdout).expect("rustc identity is UTF-8");
    println!(
        "cargo:rustc-env=TIDE_COMPILER_ID={}",
        version.replace('\n', ";")
    );
    let lib = format!("{}/lib", sysroot.trim());
    println!("cargo:rustc-link-search=native={lib}");
    println!("cargo:rustc-link-arg=-Wl,-rpath,{lib}");
}
