//! The runner on a copy of one corpus case, never the corpus itself: the case passes, and with
//! one expectation changed the runner fails, exits 1, and names the case. The full corpus is its
//! own suite (`cargo run -p tide-conformance`), outside `cargo test --workspace`.
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output, Stdio};

/// The test's work directory; dropping it removes the directory.
struct Work(PathBuf);

impl Drop for Work {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

/// The runner on `case`, building into the suite's own target directory, as tide-demo's test
/// does, so its builds reuse the suite's.
fn runner(case: &Path) -> Output {
    let scratch = Path::new(env!("CARGO_TARGET_TMPDIR"));
    Command::new(env!("CARGO_BIN_EXE_tide-conformance"))
        .arg(case)
        .env("CARGO_TARGET_DIR", scratch.parent().unwrap())
        .stdin(Stdio::null())
        .stderr(Stdio::inherit())
        .output()
        .expect("run tide-conformance")
}

#[test]
fn a_changed_expectation_fails_its_case() {
    let scratch = PathBuf::from(env!("CARGO_TARGET_TMPDIR"));
    let work = Work(scratch.join(format!("tide-conformance-{}", std::process::id())));
    fs::create_dir_all(&work.0).unwrap();
    // Physical paths: the driver rejects a package path that traverses a directory symlink.
    let case = fs::canonicalize(&work.0).unwrap().join("cap_ambient.rs");
    let source = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../conformance/cases/authority/cap_ambient.rs");
    let text = fs::read_to_string(source).unwrap();
    fs::write(&case, &text).unwrap();
    let name = case.display();

    let out = runner(&case);
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert_eq!(out.status.code(), Some(0), "{stdout}");
    assert_eq!(
        stdout,
        format!("PASS  {name}\n1 cases: 1 PASS, 0 AWAITS CHECKER, 0 FAIL\n")
    );

    // The driver's code on line 4 changed to another code it decides.
    let changed = text.replace("//~ E0425 TD0501", "//~ E0425 TD0503");
    assert_ne!(
        changed, text,
        "the case no longer has the mark this test changes"
    );
    fs::write(&case, changed).unwrap();
    let out = runner(&case);
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert_eq!(out.status.code(), Some(1), "{stdout}");
    assert!(
        stdout.starts_with(&format!(
            "FAIL  {name}: driver diagnostics [4:error:TD0501 8:error:TD0501], \
             marked [4:error:TD0503 8:error:TD0501], exit 1\n"
        )),
        "{stdout}"
    );
    assert!(
        stdout.ends_with("\n1 cases: 0 PASS, 0 AWAITS CHECKER, 1 FAIL\n"),
        "{stdout}"
    );
}
