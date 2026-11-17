//! Builds programs through the admission driver against production std, for the test files that
//! run admitted programs (`build.rs`, `examples.rs`, `benchmarks.rs`) and the benchmark rerun
//! tool (`benches/pairs.rs`), and the two helpers more than one of them uses to build or run
//! programs outside the driver.
use std::path::{Path, PathBuf};
use std::process::{Command, Output, Stdio};
use std::sync::OnceLock;

#[path = "../../../tide-std/tests/support/mod.rs"]
mod support;
#[allow(unused_imports)] // Only the rerun tool builds std into a directory of its own.
pub use support::build_std;

pub fn checked(command: &mut Command) -> Output {
    let output = command.stdin(Stdio::null()).output().unwrap();
    assert!(output.status.success(), "{}", stderr(&output));
    output
}

pub fn stderr(output: &Output) -> String {
    String::from_utf8_lossy(&output.stderr).into_owned()
}

/// The production std, built once per test binary into one directory that `build.rs`,
/// `examples.rs`, and `benchmarks.rs` share, so the workspace suite builds it once.
/// `build_std` replaces its flattened `artifacts/`, so two test binaries must not run at once on
/// one target directory: `cargo test` runs them one at a time, while a parallel runner
/// (nextest) would race here.
pub fn std_dir() -> &'static Path {
    static DIR: OnceLock<PathBuf> = OnceLock::new();
    DIR.get_or_init(|| {
        let dir = PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join("build/std");
        support::build_std(&dir, false)
    })
}

pub fn package(name: &str, source: &str) -> PathBuf {
    let dir = PathBuf::from(env!("CARGO_TARGET_TMPDIR"))
        .join("build")
        .join(name);
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(dir.join("package/src")).unwrap();
    std::fs::write(dir.join("package/src/demo.rs"), source).unwrap();
    dir
}

#[allow(dead_code)] // The rerun tool chooses its std with `driver_with`.
pub fn driver(dir: &Path) -> Command {
    driver_with(std_dir(), dir)
}

/// The driver building `dir/package/src/demo.rs` into `dir/out` against the std that
/// `build_std` flattened into `std`.
pub fn driver_with(std: &Path, dir: &Path) -> Command {
    let mut command = Command::new(env!("CARGO_BIN_EXE_tide-admit"));
    command
        .args(["--crate-type=bin", "--emit=link"])
        .arg("--tide-std")
        .arg(std.join("libtide_std.rlib"))
        .arg("--tide-dep-dir")
        .arg(std)
        .arg("--out-dir")
        .arg(dir.join("out"))
        .arg(dir.join("package/src/demo.rs"));
    command
}

/// Runs `program` under `timeout` (killed 10 s after `seconds`), with stdin closed and without
/// the seed and capture controls the caller's environment may carry.
#[allow(dead_code)] // `build.rs` bounds its runs its own way.
pub fn timed(seconds: u32, program: &Path) -> Command {
    let mut command = Command::new("timeout");
    command
        .args(["-k", "10"])
        .arg(seconds.to_string())
        .arg(program)
        .env_remove("TIDE_TEST_SEED")
        .env_remove("TIDE_SIM_SEED")
        .env_remove("RUST_TEST_NOCAPTURE")
        .stdin(Stdio::null());
    command
}

/// Compiles the Rust file `code` to `binary` with plain rustc, outside the driver: the
/// toolchain `rust-toolchain.toml` pins, unless `RUSTC` names another.
#[allow(dead_code)] // `examples.rs` builds only through the driver.
pub fn plain_rustc(code: &Path, binary: &Path) {
    checked(
        Command::new(std::env::var_os("RUSTC").unwrap_or("rustc".into()))
            .current_dir(env!("CARGO_MANIFEST_DIR"))
            .args(["--edition=2024", "-o"])
            .arg(binary)
            .arg(code),
    );
}
