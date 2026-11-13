//! The driver owns compiler inputs; these tests use Rust std only as a development fixture.
use std::path::PathBuf;
use std::process::{Command, Output};

fn package(name: &str, source: &str) -> PathBuf {
    let dir = PathBuf::from(env!("CARGO_TARGET_TMPDIR"))
        .join("invocation")
        .join(name);
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(dir.join("package/src")).unwrap();
    std::fs::write(dir.join("package/src/lib.rs"), source).unwrap();
    dir
}

fn driver(dir: &std::path::Path) -> Command {
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_tide-admit"));
    cmd.args([
        "--crate-type=lib",
        "--emit=metadata",
        "--error-format=short",
    ])
    .arg("--out-dir")
    .arg(dir.join("out"))
    .arg(dir.join("package/src/lib.rs"));
    cmd
}

#[test]
fn surface_errors_are_errors_even_under_allow_warnings() {
    let dir = package("surface-errors", "#![allow(warnings)]\npub trait Parse<'a> {}\npub fn narrow(n: u64) -> u8 { n as u8 }\n");
    let out = driver(&dir).output().unwrap();
    assert_eq!(out.status.code(), Some(1), "{}", stderr(&out));
    for code in ["FX0201", "FX0301"] {
        assert!(stderr(&out).contains(&format!("error: {code}:")), "{}", stderr(&out));
    }
    assert!(stderr(&out).contains(":2:17: error: FX0201:"), "{}", stderr(&out));
}

#[test]
fn borrow_labels_preserve_rustc_errors_and_leave_unrelated_borrows_alone() {
    for (name, source, rustc_error, label) in [
        ("closure-return", "pub fn f(s: &str) -> Box<dyn Fn() -> usize> { Box::new(move || s.len()) }", "lifetime may not live long enough", "FX0203"),
        ("closure-bound-earlier", r#"pub fn f() -> Box<dyn Fn() -> usize> { let s = String::from("x"); let job = || s.len(); Box::new(job) }"#, "E0373", "FX0203"),
        ("closure-field", "pub struct Hook { f: Box<dyn Fn() -> usize> }\npub fn f(s: &str) -> Hook { Hook { f: Box::new(move || s.len()) } }", "lifetime may not live long enough", "FX0203"),
        ("closure-explicit-return", "pub fn f(s: &str) -> Box<dyn Fn() -> usize> { return Box::new(move || s.len()); }", "lifetime may not live long enough", "FX0203"),
        ("closure-argument", "pub fn takes(_: Box<dyn Fn() -> usize>) {} pub fn f(s: &str) { takes(Box::new(move || s.len())); }", "lifetime may not live long enough", "FX0203"),
        ("borrowing-literal", "pub struct Doc<'a> { text: String, word: &'a str }\npub fn f<'a>() -> Doc<'a> { let text = String::from(\"x\"); Doc { word: &text, text } }", "E0515", "FX0210"),
        ("unrelated-borrow", "pub fn f() { let r; { let x = String::from(\"x\"); r = &x; } let _ = r.len(); }", "E0597", ""),
        ("opaque-capture", "pub fn f() -> impl FnMut() -> u8 { let mut n = 0; || { n += 1; n } }", "E0373", ""),
        ("wrong-input-elision", "pub struct Token<'a>(&'a str); impl Token<'_> { pub fn f(&self, other: &str) -> &str { other } }", "lifetime may not live long enough", ""),
    ] {
        let dir = package(name, source);
        let out = driver(&dir).output().unwrap();
        let err = stderr(&out);
        assert_eq!(out.status.code(), Some(1), "{name}: {err}");
        assert!(err.contains(rustc_error), "{name}: {err}");
        let fx: Vec<_> = err.lines().filter(|line| line.contains("error: FX")).collect();
        if label.is_empty() {
            assert!(fx.is_empty(), "{name}: {err}");
        } else {
            assert!(!fx.is_empty() && fx.iter().all(|line| line.contains(label)), "{name}: {err}");
        }
    }
}

#[test]
fn self_borrow_e0515_labels_only_the_paired_borrow() {
    let dir = package(
        "self-borrow-secondary",
        r#"
pub struct Doc<'a> { word: &'a str, other: &'a str, text: String }
pub fn f<'a>() -> Doc<'a> {
    let text = String::from("owner");
    let y = String::from("other");
    Doc {
        word: &text,
        other: &y,
        text,
    }
}
"#,
    );
    let out = driver(&dir).output().unwrap();
    let error = stderr(&out);
    assert_eq!(out.status.code(), Some(1), "{error}");
    assert_eq!(
        error
            .lines()
            .filter(|line| line.contains(":6:") && line.contains("E0515"))
            .count(),
        2,
        "{error}"
    );
    assert_eq!(
        error
            .lines()
            .filter(|line| line.contains(":6:") && line.contains("error: FX0210:"))
            .count(),
        1,
        "{error}"
    );
}

#[test]
fn malformed_and_empty_arguments_fail_before_compilation() {
    for args in [
        vec![],
        vec!["--out-dir"],
        vec!["--emit=link"],
        vec!["demo.rs", "--emit=link"],
        vec!["demo.rs", "--tide-profile=fast"],
        vec!["demo.rs", "--emit=metadata,link"],
        vec!["--crate-type=proc-macro"],
        vec!["--tide-grant=all"],
        vec!["--tide-print-codes", "--cfg=x"],
    ] {
        let out = Command::new(env!("CARGO_BIN_EXE_tide-admit"))
            .args(args)
            .output()
            .unwrap();
        assert_eq!(out.status.code(), Some(2), "{}", stderr(&out));
    }
}

#[test]
fn separate_library_arguments_require_source_and_full_admission() {
    let full = "`--tide-test-library` requires `--test` and full admission without grants";
    for (args, diagnostic) in [
        (vec!["--tide-test-library=lib.rs"], full),
        (
            vec!["--test", "--tide-test-library=libfake.rlib"],
            "`--tide-test-library` requires a .rs source",
        ),
        (
            vec![
                "--test",
                "--tide-test-library=lib.rs",
                "--crate-name=tide_candidate",
            ],
            "`--tide-test-library` reserves crate name `tide_candidate`",
        ),
        (
            vec![
                "--test",
                "--tide-test-library=lib.rs",
                "--tide-grant=unsafe",
            ],
            full,
        ),
        (
            vec![
                "--test",
                "--tide-test-library=lib.rs",
                "--tide-test-backstop",
            ],
            full,
        ),
    ] {
        let out = Command::new(env!("CARGO_BIN_EXE_tide-admit"))
            .arg("demo.rs")
            .args(args)
            .output()
            .unwrap();
        assert_eq!(out.status.code(), Some(2), "{}", stderr(&out));
        assert_eq!(stderr(&out).trim_end(), format!("tide-admit: {diagnostic}"));
    }
}

#[test]
fn inherited_environment_and_candidate_configuration_cannot_configure_rustc() {
    let dir = package("environment", "pub fn f() {}\n");
    let wrapper = dir.join("wrapper");
    std::fs::write(&wrapper, "this is not an executable\n").unwrap();
    std::fs::create_dir(dir.join("package/.cargo")).unwrap();
    std::fs::write(
        dir.join("package/.cargo/config.toml"),
        "[build]\nrustflags=['--cfg=attacker']\nrustc-wrapper='missing'\n",
    )
    .unwrap();
    std::fs::write(
        dir.join("package/rust-toolchain.toml"),
        "[toolchain]\nchannel='nonexistent'\n",
    )
    .unwrap();
    let out = driver(&dir)
        .current_dir(dir.join("package"))
        .env(
            "RUSTFLAGS",
            "--cfg attacker -Zallow-features=core_intrinsics",
        )
        .env("RUSTC_WRAPPER", wrapper)
        .env("RUSTUP_TOOLCHAIN", "missing")
        .env("LD_LIBRARY_PATH", "/nonexistent")
        .env("TMPDIR", "/nonexistent")
        .env("PATH", "/nonexistent")
        .output()
        .unwrap();
    assert!(out.status.success(), "{}", stderr(&out));

    for (i, source) in [
        "pub const A: &str = env!(\"TIDE_TEST_CANARY\");",
        "use core::env as probe; pub const A: &str = probe!(\"TIDE_TEST_CANARY\");",
        "pub const A: Option<&str> = option_env!(\"TIDE_TEST_CANARY\");",
        "use core::option_env as probe; #[doc = probe!(\"TIDE_TEST_CANARY\")] pub fn f() {}",
        // `compile_error!` expands `env!` eagerly, so an inherited value would reach stderr.
        "compile_error!(env!(\"TIDE_TEST_CANARY\"));",
    ]
    .iter()
    .enumerate()
    {
        let dir = package(&format!("env-canary-{i}"), source);
        let out = driver(&dir)
            .env("TIDE_TEST_CANARY", "PRIVATE_ENV_CANARY")
            .output()
            .unwrap();
        assert_eq!(out.status.code(), Some(1), "{}", stderr(&out));
        assert!(stderr(&out).contains("FX0520"), "{}", stderr(&out));
        assert!(!stderr(&out).contains("PRIVATE_ENV_CANARY"));
        assert!(!dir.join("out/liblib.rmeta").exists());
        assert_no_canary(&dir.join("out"), b"PRIVATE_ENV_CANARY");
    }
}

#[test]
fn outside_reads_and_static_symlinks_never_disclose_file_bytes() {
    let dir = package("outside", "pub fn f() {}\n");
    let canary = dir.join("private.rs");
    std::fs::write(&canary, b"PRIVATE_FILE_CANARY\xff\xfe").unwrap();
    for source in [
        format!("include!({canary:?});"),
        format!("pub fn f() {{ let _ = include!({canary:?}); }}"),
        format!("pub const A: &str = include_str!({canary:?});"),
        format!("pub const A: &[u8] = include_bytes!({canary:?});"),
        format!("#[doc = include_str!({canary:?})] pub fn f() {{}}"),
    ] {
        std::fs::write(dir.join("package/src/lib.rs"), source).unwrap();
        let out = driver(&dir).output().unwrap();
        assert_eq!(out.status.code(), Some(1), "{}", stderr(&out));
        assert!(
            !stderr(&out).contains("PRIVATE_FILE_CANARY"),
            "{}",
            stderr(&out)
        );
        assert_no_canary(&dir.join("out"), b"PRIVATE_FILE_CANARY");
    }
    #[cfg(unix)]
    {
        let text = dir.join("package/src/private.txt");
        std::fs::write(&text, "compile_error!(\"PRIVATE_NON_RS_CANARY\");\n").unwrap();
        std::os::unix::fs::symlink(&text, dir.join("package/src/text.rs")).unwrap();
        std::fs::write(dir.join("package/src/lib.rs"), "mod text;\n").unwrap();
        let out = driver(&dir).output().unwrap();
        assert!(!out.status.success());
        assert!(
            !stderr(&out).contains("PRIVATE_NON_RS_CANARY"),
            "{}",
            stderr(&out)
        );
        std::fs::remove_file(dir.join("package/src/text.rs")).unwrap();
        std::os::unix::fs::symlink(&canary, dir.join("package/src/escape.rs")).unwrap();
        std::fs::write(dir.join("package/src/lib.rs"), "mod escape;\n").unwrap();
        let out = driver(&dir).output().unwrap();
        assert_eq!(out.status.code(), Some(1), "{}", stderr(&out));
        assert!(!stderr(&out).contains("PRIVATE_FILE_CANARY"));
        std::fs::remove_file(dir.join("package/src/lib.rs")).unwrap();
        std::os::unix::fs::symlink(&canary, dir.join("package/src/lib.rs")).unwrap();
        let out = driver(&dir).output().unwrap();
        assert!(!out.status.success());
        assert!(!stderr(&out).contains("PRIVATE_FILE_CANARY"));
    }
}

#[test]
fn each_loaded_source_is_guarded_before_expansion() {
    let dir = package("attributes", "mod sub;\n");
    for source in [
        "#[path = \"/private.rs\"] mod hidden;",
        "#[r#path = \"/private.rs\"] mod hidden;",
        "#[cfg_attr(any(), cfg_attr(any(), path = \"/private.rs\"))] mod hidden;",
        "#[debugger_visualizer(natvis_file = \"/private.rs\")] pub fn f() {}",
        "std::thread_local! { #[path = \"/private.rs\"] static X: u8 = 0; }",
    ] {
        std::fs::write(dir.join("package/src/sub.rs"), source).unwrap();
        let out = driver(&dir).output().unwrap();
        assert_eq!(out.status.code(), Some(1), "{}", stderr(&out));
        assert!(stderr(&out).contains("FX0911"), "{}", stderr(&out));
    }
}

#[test]
fn attributes_and_macro_definitions_hidden_in_macro_arguments_are_rejected() {
    let dir = package("expanded", "mod sub;\n");
    std::fs::write(dir.join("package/src/inner.rs"), "pub fn g() {}\n").unwrap();
    for (source, codes) in [
        (
            "pub fn f() { assert!({ macro_rules! forge { () => { true }; } forge!() }); }",
            &["FX0101"][..],
        ),
        (
            "pub fn f() { assert!({ macro_rules! forge { ($a:ident) => { #[$a = \"inner.rs\"] mod generated; }; } forge!(path); true }); }",
            &["FX0101", "FX0911"],
        ),
        (
            "pub fn f() { assert!({ macro_rules! forge { ($a:ident) => { #[$a(natvis_file = \"inner.rs\")] mod generated {} }; } forge!(debugger_visualizer); true }); }",
            &["FX0101", "FX0911"],
        ),
    ] {
        std::fs::write(dir.join("package/src/sub.rs"), source).unwrap();
        let out = driver(&dir).output().unwrap();
        assert_eq!(out.status.code(), Some(1), "{}", stderr(&out));
        for code in codes {
            assert!(stderr(&out).contains(code), "{source}: {}", stderr(&out));
        }
    }
}

#[test]
fn candidate_content_gets_rustc_diagnostics_and_every_verdict() {
    // An identifier named `path` in an attribute that `cfg_attr` applies loads nothing.
    let dir = package(
        "content",
        "#[cfg_attr(all(), allow(path))] pub fn f() {}\n\
         #[cfg_attr(all(), cfg_attr(all(), allow(path)))] pub fn g() {}\n\
         pub fn h() { assert!({ #[cfg_attr(all(), allow(path))] fn i() {} true }); }\n",
    );
    let out = driver(&dir).output().unwrap();
    assert!(out.status.success(), "{}", stderr(&out));
    // Unreferenced, unreadable candidate files read as missing, not as a setup error.
    std::fs::write(dir.join("package/src/junk.rs"), b"\xff\xfe").unwrap();
    #[cfg(unix)]
    std::os::unix::fs::symlink("/nonexistent/tide", dir.join("package/src/dangling.rs")).unwrap();
    let out = driver(&dir).output().unwrap();
    assert!(out.status.success(), "{}", stderr(&out));
    // Syntax errors come once, in the session's short format.
    std::fs::write(
        dir.join("package/src/lib.rs"),
        "pub fn f() { let a = 1 let b = 2; }\n",
    )
    .unwrap();
    let out = driver(&dir).output().unwrap();
    assert_eq!(out.status.code(), Some(1), "{}", stderr(&out));
    assert_eq!(
        stderr(&out).matches("error: expected").count(),
        1,
        "{}",
        stderr(&out)
    );
    assert!(!stderr(&out).contains("-->"), "{}", stderr(&out));
    // A macro definition does not hide the crate's other verdicts.
    std::fs::write(
        dir.join("package/src/lib.rs"),
        "macro_rules! m { () => {} }\npub fn f() { unsafe {} }\n",
    )
    .unwrap();
    let out = driver(&dir).output().unwrap();
    assert_eq!(out.status.code(), Some(1), "{}", stderr(&out));
    assert!(stderr(&out).contains("FX0101"), "{}", stderr(&out));
    assert!(stderr(&out).contains("FX0901"), "{}", stderr(&out));
}

#[test]
fn trusted_input_directories_are_not_readable_by_candidate_source() {
    let dir = package("trusted-read", "pub fn f() {}\n");
    let deps = dir.join("deps");
    std::fs::create_dir(&deps).unwrap();
    let text = deps.join("notes.txt");
    std::fs::write(&text, "PRIVATE_TRUSTED_CANARY").unwrap();
    let rust = deps.join("notes.rs");
    std::fs::write(&rust, "compile_error!(\"PRIVATE_TRUSTED_CANARY\");\n").unwrap();
    for source in [
        format!("compile_error!(include_str!({text:?}));"),
        format!("pub const A: &[u8] = include_bytes!({text:?});"),
        format!("include!({rust:?});"),
    ] {
        std::fs::write(dir.join("package/src/lib.rs"), source).unwrap();
        let out = driver(&dir)
            .arg("--tide-dep-dir")
            .arg(&deps)
            .output()
            .unwrap();
        assert_eq!(out.status.code(), Some(1), "{}", stderr(&out));
        assert!(
            !stderr(&out).contains("PRIVATE_TRUSTED_CANARY"),
            "{}",
            stderr(&out)
        );
        assert_no_canary(&dir.join("out"), b"PRIVATE_TRUSTED_CANARY");
    }
}

#[test]
fn build_scripts_are_rejected_without_running() {
    let accepted = package("no-build-script", "pub fn f() {}\n");
    let out = driver(&accepted).output().unwrap();
    assert_eq!(out.status.code(), Some(0), "{}", stderr(&out));
    assert!(!stderr(&out).contains("FX0915"), "{}", stderr(&out));
    let dir = package("build-script", "pub fn f() {}\n");
    std::fs::write(
        dir.join("package/build.rs"),
        "compile_error!(\"BUILD_SCRIPT_EXECUTED\");",
    )
    .unwrap();
    let out = driver(&dir).output().unwrap();
    assert_eq!(out.status.code(), Some(1), "{}", stderr(&out));
    assert!(stderr(&out).starts_with("error: FX0915:"), "{}", stderr(&out));
    assert!(!stderr(&out).contains("BUILD_SCRIPT_EXECUTED"));
    // A `..` in the source path must not hide the package root.
    std::fs::create_dir(dir.join("package/src/sub")).unwrap();
    let out = Command::new(env!("CARGO_BIN_EXE_tide-admit"))
        .arg("--out-dir")
        .arg(dir.join("out"))
        .arg(dir.join("package/src/sub/../lib.rs"))
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(1), "{}", stderr(&out));
    assert!(stderr(&out).starts_with("error: FX0915:"), "{}", stderr(&out));
}

fn entries(dir: &std::path::Path) -> usize {
    std::fs::read_dir(dir.join("out/.tide-cache"))
        .unwrap()
        .count()
}

#[test]
fn cache_keys_cover_sources_membership_grants_and_trusted_inputs_and_skip_failures() {
    let dir = package("cache", "pub fn f() {}\n");
    let run = || {
        let out = driver(&dir).output().unwrap();
        assert!(out.status.success(), "{}", stderr(&out));
    };
    run();
    assert_eq!(entries(&dir), 1);
    let entry = std::fs::read_dir(dir.join("out/.tide-cache"))
        .unwrap()
        .next()
        .unwrap()
        .unwrap()
        .path();
    let modified = entry.metadata().unwrap().modified().unwrap();
    run();
    assert_eq!(entries(&dir), 1);
    assert_eq!(entry.metadata().unwrap().modified().unwrap(), modified);
    std::fs::write(dir.join("package/src/lib.rs"), "pub fn f() -> u8 { 1 }\n").unwrap();
    run();
    assert_eq!(entries(&dir), 2);
    std::fs::write(dir.join("package/src/added.rs"), "pub fn added() {}\n").unwrap();
    run();
    assert_eq!(entries(&dir), 3);
    let out = driver(&dir).arg("--tide-grant=unsafe").output().unwrap();
    assert!(out.status.success(), "{}", stderr(&out));
    assert_eq!(entries(&dir), 4);
    std::fs::create_dir(dir.join("deps")).unwrap();
    for text in ["one", "two"] {
        std::fs::write(dir.join("deps/input"), text).unwrap();
        let out = driver(&dir)
            .arg("--tide-dep-dir")
            .arg(dir.join("deps"))
            .output()
            .unwrap();
        assert!(out.status.success(), "{}", stderr(&out));
    }
    assert_eq!(entries(&dir), 6);
    std::fs::write(
        dir.join("package/src/lib.rs"),
        "pub fn f() { let _ = core::option_env!(\"TIDE_TEST_CANARY\"); }\n",
    )
    .unwrap();
    let out = driver(&dir).output().unwrap();
    assert_eq!(out.status.code(), Some(1), "{}", stderr(&out));
    assert_eq!(entries(&dir), 6);
    std::fs::write(dir.join("package/src/lib.rs"), "pub fn f() {}\n").unwrap();
    run();
    assert_eq!(entries(&dir), 7);
    let host = env!("TIDE_COMPILER_ID")
        .split(';')
        .find_map(|line| line.strip_prefix("host: "))
        .unwrap();
    let std_dir = PathBuf::from(env!("TIDE_SYSROOT")).join(format!("lib/rustlib/{host}/lib"));
    let std = std::fs::read_dir(std_dir)
        .unwrap()
        .map(|entry| entry.unwrap().path())
        .find(|path| {
            path.file_name()
                .unwrap()
                .to_string_lossy()
                .starts_with("libstd-")
                && path.extension().is_some_and(|ext| ext == "rmeta")
        })
        .unwrap();
    let trusted_std = dir.join("libtrusted_std.rmeta");
    std::fs::copy(std, &trusted_std).unwrap();
    let out = driver(&dir)
        .arg("--tide-std")
        .arg(&trusted_std)
        .output()
        .unwrap();
    assert!(out.status.success(), "{}", stderr(&out));
    assert_eq!(entries(&dir), 8);
    std::fs::write(&trusted_std, "invalid trusted artifact").unwrap();
    let out = driver(&dir)
        .arg("--tide-std")
        .arg(&trusted_std)
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(1), "{}", stderr(&out));
    assert_eq!(entries(&dir), 8);
}

/// rustc's crash hook writes a report (`rustc-ice-*.txt`) to the current directory, which may be
/// the candidate's tree (RUN-2). A crash under the driver writes none, and a cached success does
/// not hide it; nor does a panic before compilation, such as a failed write of the codes.
#[test]
fn crashes_write_no_report_files() {
    let dir = package("ice", "pub fn f() {}\n");
    let package = dir.join("package");
    let out = driver(&dir).current_dir(&package).output().unwrap();
    assert!(out.status.success(), "{}", stderr(&out));
    let out = driver(&dir)
        .arg("--tide-test-ice")
        .current_dir(&package)
        .output()
        .unwrap();
    assert!(!out.status.success());
    let err = stderr(&out);
    assert!(err.contains("the compiler unexpectedly panicked"), "{err}");
    assert!(err.contains("a seeded compiler crash"), "{err}");
    let full = std::fs::File::create("/dev/full").unwrap();
    let out = Command::new(env!("CARGO_BIN_EXE_tide-admit"))
        .arg("--tide-print-codes")
        .env_remove("RUSTC_ICE")
        .stdout(full)
        .current_dir(&package)
        .output()
        .unwrap();
    assert!(!out.status.success());
    let mut files = Vec::new();
    let mut dirs = vec![dir.clone()];
    while let Some(d) = dirs.pop() {
        for entry in std::fs::read_dir(d).unwrap() {
            let path = entry.unwrap().path();
            if path.is_dir() {
                dirs.push(path);
            } else if !path.starts_with(dir.join("out/.tide-cache")) {
                files.push(path);
            }
        }
    }
    files.sort();
    assert_eq!(
        files,
        [dir.join("out/liblib.rmeta"), package.join("src/lib.rs")]
    );
}

fn stderr(out: &Output) -> String {
    String::from_utf8_lossy(&out.stderr).into_owned()
}

#[test]
fn unavailable_trusted_dependency_source_does_not_crash_error_rendering() {
    let dir = package(
        "dependency-diagnostic",
        "pub fn f() { std::need(std::rc::Rc::new(0u8)); }\n",
    );
    let fixture = dir.join("toolchain");
    std::fs::create_dir(&fixture).unwrap();
    std::fs::write(
        fixture.join("fixture.rs"),
        "pub mod prelude { pub mod rust_2024 {} } pub use std::rc; pub fn need<T: Send>(_: T) {} #[macro_export] macro_rules! touch { () => { mod unused; } }\n",
    )
    .unwrap();
    let library = fixture.join("libfixture.rlib");
    let out = Command::new(PathBuf::from(env!("TIDE_SYSROOT")).join("bin/rustc"))
        .env_clear()
        .args([
            "--crate-type=rlib",
            "--crate-name=fixture",
            "--edition=2024",
        ])
        .arg(format!(
            "--remap-path-prefix={}=/tide-unavailable",
            fixture.display()
        ))
        .arg(fixture.join("fixture.rs"))
        .arg("-o")
        .arg(&library)
        .output()
        .unwrap();
    assert!(out.status.success(), "{}", stderr(&out));
    let out = driver(&dir)
        .arg("--tide-std")
        .arg(&library)
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(1), "{}", stderr(&out));
    assert!(stderr(&out).contains("error[E0277]"), "{}", stderr(&out));
    assert!(
        !stderr(&out).contains("internal compiler error"),
        "{}",
        stderr(&out)
    );
    // A registered foreign filename must not grant include! read access to that path.
    let canary = dir.join("private.rs");
    std::fs::write(&canary, b"PRIVATE_FOREIGN_CANARY\xff").unwrap();
    let out = Command::new(PathBuf::from(env!("TIDE_SYSROOT")).join("bin/rustc"))
        .env_clear()
        .args([
            "--crate-type=rlib",
            "--crate-name=fixture",
            "--edition=2024",
        ])
        .arg(format!(
            "--remap-path-prefix={}={}",
            fixture.join("fixture.rs").display(),
            canary.display()
        ))
        .arg(fixture.join("fixture.rs"))
        .arg("-o")
        .arg(&library)
        .output()
        .unwrap();
    assert!(out.status.success(), "{}", stderr(&out));
    std::fs::write(
        dir.join("package/src/lib.rs"),
        format!("pub fn f() {{ std::need(0u8); }} core::include!({canary:?});\n"),
    )
    .unwrap();
    let out = driver(&dir)
        .arg("--tide-std")
        .arg(&library)
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(1), "{}", stderr(&out));
    assert!(stderr(&out).contains("FX0520"), "{}", stderr(&out));
    assert!(!stderr(&out).contains("PRIVATE_FOREIGN_CANARY"));
    assert_no_canary(&dir.join("out"), b"PRIVATE_FOREIGN_CANARY");
    let unused = dir.join("package/src/unused.rs");
    std::fs::write(&unused, "macro_rules! hidden { () => {} }\n").unwrap();
    let out = Command::new(PathBuf::from(env!("TIDE_SYSROOT")).join("bin/rustc"))
        .env_clear()
        .args([
            "--crate-type=rlib",
            "--crate-name=fixture",
            "--edition=2024",
        ])
        .arg(format!(
            "--remap-path-prefix={}={}",
            fixture.join("fixture.rs").display(),
            unused.display()
        ))
        .arg(fixture.join("fixture.rs"))
        .arg("-o")
        .arg(&library)
        .output()
        .unwrap();
    assert!(out.status.success(), "{}", stderr(&out));
    std::fs::write(
        dir.join("package/src/lib.rs"),
        "pub fn f() { std::need(std::rc::Rc::new(0u8)); }\n",
    )
    .unwrap();
    let out = driver(&dir)
        .arg("--tide-std")
        .arg(&library)
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(1), "{}", stderr(&out));
    assert!(stderr(&out).contains("error[E0277]"), "{}", stderr(&out));
    assert!(!stderr(&out).contains("FX0101"), "{}", stderr(&out));
    std::fs::write(
        dir.join("package/src/lib.rs"),
        "std::touch!(); pub fn f() { std::need(0u8); }\n",
    )
    .unwrap();
    let out = driver(&dir)
        .arg("--tide-std")
        .arg(&library)
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(1), "{}", stderr(&out));
    assert!(stderr(&out).contains("FX0101"), "{}", stderr(&out));
}

fn assert_no_canary(path: &std::path::Path, canary: &[u8]) {
    if !path.exists() {
        return;
    }
    for entry in std::fs::read_dir(path).unwrap() {
        let path = entry.unwrap().path();
        if path.is_dir() {
            assert_no_canary(&path, canary);
        } else {
            assert!(
                !std::fs::read(&path)
                    .unwrap()
                    .windows(canary.len())
                    .any(|bytes| bytes == canary),
                "{}",
                path.display()
            );
        }
    }
}

#[test]
fn output_and_trusted_inputs_cannot_be_candidate_owned() {
    let dir = package("state-boundary", "pub fn f() {}\n");
    let out = driver(&dir)
        .arg("--out-dir")
        .arg(dir.join("package/new/output"))
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(2), "{}", stderr(&out));
    assert!(!dir.join("package/new").exists());
    let out = driver(&dir)
        .arg("--tide-dep-dir")
        .arg(dir.join("package/src"))
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(2), "{}", stderr(&out));
    #[cfg(unix)]
    {
        std::fs::create_dir_all(dir.join("outside-cache")).unwrap();
        std::os::unix::fs::symlink(dir.join("outside-cache"), dir.join("out/.tide-cache")).unwrap();
        let out = driver(&dir).output().unwrap();
        assert_eq!(out.status.code(), Some(2), "{}", stderr(&out));
        assert!(
            std::fs::read_dir(dir.join("outside-cache"))
                .unwrap()
                .next()
                .is_none()
        );
    }
}

#[cfg(unix)]
#[test]
fn source_directory_symlink_cannot_redefine_the_candidate_root() {
    let dir = package("source-root-escape", "pub fn f() {}\n");
    let outside = dir.join("outside");
    std::fs::create_dir(&outside).unwrap();
    std::fs::write(
        outside.join("lib.rs"),
        "compile_error!(\"PRIVATE_ROOT_CANARY\");\n",
    )
    .unwrap();
    std::fs::remove_dir_all(dir.join("package/src")).unwrap();
    std::os::unix::fs::symlink(&outside, dir.join("package/src")).unwrap();
    let out = driver(&dir).output().unwrap();
    assert!(!out.status.success(), "{}", stderr(&out));
    assert!(
        !stderr(&out).contains("PRIVATE_ROOT_CANARY"),
        "{}",
        stderr(&out)
    );
    assert_no_canary(&dir.join("out"), b"PRIVATE_ROOT_CANARY");
}

#[test]
fn raw_compiler_flags_and_response_files_are_rejected() {
    let dir = package("raw", "pub fn f() {}\n");
    std::fs::write(dir.join("flags"), "--cfg attacker\n").unwrap();
    for flag in [
        "--cfg=attacker",
        "--extern=std=evil",
        "-Ldependency=evil",
        "--sysroot=evil",
        "-Copt-level=3",
        "-Zallow-features=core_intrinsics",
        "@flags",
    ] {
        let out = driver(&dir).arg(flag).current_dir(&dir).output().unwrap();
        assert_eq!(out.status.code(), Some(2), "{flag}: {}", stderr(&out));
        assert!(
            stderr(&out).contains("unsupported argument"),
            "{flag}: {}",
            stderr(&out)
        );
    }
}
