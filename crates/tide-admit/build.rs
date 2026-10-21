//! Lets the driver find the pinned toolchain's `librustc_driver` and `libLLVM` at link and run
//! time, so neither the binary nor its tests need `LD_LIBRARY_PATH`.
use std::process::Command;

fn main() {
    let rustc = std::env::var("RUSTC").unwrap_or_else(|_| "rustc".into());
    let out = Command::new(&rustc)
        .args(["--print", "sysroot"])
        .output()
        .expect("run rustc");
    assert!(out.status.success(), "`{rustc} --print sysroot` failed");
    let sysroot = String::from_utf8(out.stdout).expect("sysroot is UTF-8");
    let lib = format!("{}/lib", sysroot.trim());
    println!("cargo:rustc-link-search=native={lib}");
    println!("cargo:rustc-link-arg=-Wl,-rpath,{lib}");
}
