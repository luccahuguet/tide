use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

/// Build the pinned production std, in Cargo's release profile when `release` is set and its dev
/// profile otherwise, and flatten its artifacts for direct rustc/driver consumers.
/// Cargo's dependency directories differ between the supported toolchains. The build is
/// hermetic (cleared environment, `--frozen`, absolute `RUSTC`, the sysroot's `lib` as the only
/// loader path); keep it that way.
pub fn build_std(dir: &Path, release: bool) -> PathBuf {
    let compiler = Command::new(std::env::var_os("RUSTC").unwrap_or("rustc".into()))
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .args(["--print", "sysroot"])
        .stdin(Stdio::null())
        .output()
        .unwrap();
    assert!(
        compiler.status.success(),
        "{}",
        String::from_utf8_lossy(&compiler.stderr)
    );
    let sysroot = String::from_utf8(compiler.stdout).unwrap();
    let out = Command::new(env!("CARGO"))
        .env_clear()
        .envs(std::env::vars_os().filter(|(key, _)| key == "HOME" || key == "CARGO_HOME"))
        .env("PATH", "/usr/bin:/bin")
        .env("RUSTC", Path::new(sysroot.trim()).join("bin/rustc"))
        // As rustup's proxy does: build scripts that run rustc (libc's, through ring) get
        // cargo's rustlib dir first on the loader path, whose rustc-dev librustc_driver cannot
        // find libLLVM through its own RUNPATH.
        .env("LD_LIBRARY_PATH", Path::new(sysroot.trim()).join("lib"))
        .env("CARGO_BUILD_JOBS", "4")
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .args([
            "build",
            "--frozen",
            "-p",
            "tide_std",
            "--message-format=json",
            "--manifest-path",
        ])
        .arg(
            Path::new(env!("CARGO_MANIFEST_DIR"))
                .parent()
                .unwrap()
                .join("tide-std/Cargo.toml"),
        )
        .args(release.then_some("--release"))
        .arg("--target-dir")
        .arg(dir)
        .stdin(Stdio::null())
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    let flat = dir.join("artifacts");
    if flat.exists() {
        std::fs::remove_dir_all(&flat).unwrap();
    }
    std::fs::create_dir_all(&flat).unwrap();
    let macro_prefix = format!("{}tide_macros-", std::env::consts::DLL_PREFIX);
    let mut built_macros = false;
    for line in std::str::from_utf8(&out.stdout).unwrap().lines() {
        let Some((_, files)) = line.split_once("\"filenames\":[") else {
            continue;
        };
        let files = files.split_once(']').unwrap().0;
        for file in files.split('"').skip(1).step_by(2) {
            let path = Path::new(file);
            let name = path.file_name().unwrap().to_str().unwrap();
            built_macros |=
                name.starts_with(&macro_prefix) && name.ends_with(std::env::consts::DLL_SUFFIX);
            if matches!(
                path.extension().and_then(|e| e.to_str()),
                Some("rlib" | "so" | "dylib" | "dll")
            ) {
                std::fs::copy(path, flat.join(path.file_name().unwrap())).unwrap();
            }
        }
    }
    assert!(
        built_macros,
        "Cargo did not report the tide_macros proc-macro artifact"
    );
    assert!(flat.join("libtide_std.rlib").is_file());
    flat
}
