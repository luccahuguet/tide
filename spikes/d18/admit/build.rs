// librustc_driver links libLLVM, which ships in <sysroot>/lib, not in rustlib (avoids llvm-tools).
fn main() {
    let rustc = std::env::var("RUSTC").unwrap_or_else(|_| "rustc".into());
    let out = std::process::Command::new(rustc).args(["--print", "sysroot"]).output().unwrap();
    let sysroot = String::from_utf8(out.stdout).unwrap();
    println!("cargo:rustc-link-search=native={}/lib", sysroot.trim());
}
