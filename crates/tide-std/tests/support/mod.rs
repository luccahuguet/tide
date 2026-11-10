use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

/// Build the pinned production std and flatten its artifacts for direct rustc/driver consumers.
/// Cargo's dependency directories differ between the supported toolchains.
pub fn build_std(dir: &Path) -> PathBuf {
    let out = Command::new(env!("CARGO"))
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .args(["build", "--offline", "--locked", "-p", "tide_std", "--message-format=json"])
        .arg("--target-dir")
        .arg(dir)
        .env_remove("RUSTFLAGS")
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
    for line in std::str::from_utf8(&out.stdout).unwrap().lines() {
        let Some((_, files)) = line.split_once("\"filenames\":[") else { continue };
        let files = files.split_once(']').unwrap().0;
        for file in files.split('"').skip(1).step_by(2) {
            let path = Path::new(file);
            if matches!(path.extension().and_then(|e| e.to_str()), Some("rlib" | "so" | "dylib" | "dll")) {
                std::fs::copy(path, flat.join(path.file_name().unwrap())).unwrap();
            }
        }
    }
    assert!(flat.join("libtide_std.rlib").is_file());
    flat
}
