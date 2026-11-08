//! Reviewed demo programs built through the driver against production std; only this trusted
//! test caller executes them. This is not the RUN-3 sandbox or Tide's test runtime.
use std::path::{Path, PathBuf};
use std::process::{Command, Output, Stdio};
use std::sync::OnceLock;

const HELLO: &str = include_str!("../../../conformance/cases/authority/root_executable.rs");

fn checked(command: &mut Command) -> Output {
    let output = command.stdin(Stdio::null()).output().unwrap();
    assert!(output.status.success(), "{}", stderr(&output));
    output
}

fn stderr(output: &Output) -> String {
    String::from_utf8_lossy(&output.stderr).into_owned()
}

fn std_dir() -> &'static Path {
    static DIR: OnceLock<PathBuf> = OnceLock::new();
    DIR.get_or_init(|| {
        let dir = PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join("build/std");
        std::fs::create_dir_all(&dir).unwrap();
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).parent().unwrap();
        let rustc = Path::new(env!("TIDE_SYSROOT")).join("bin/rustc");
        checked(
            Command::new(&rustc)
                .env_clear()
                .args([
                    "--edition=2024",
                    "--crate-type=proc-macro",
                    "--crate-name=tide_macros",
                    "--extern=proc_macro",
                    "-Clinker=/usr/bin/cc",
                    "-Clinker-flavor=gnu-cc",
                    "-Clink-self-contained=-linker",
                    "-Clink-arg=-B/usr/bin",
                    "-Zunstable-options",
                ])
                .arg(root.join("tide-macros/src/lib.rs"))
                .arg("--out-dir")
                .arg(&dir),
        );
        checked(
            Command::new(rustc)
                .env_clear()
                .args([
                    "--edition=2024",
                    "--crate-type=rlib",
                    "--crate-name=tide_std",
                    "-Coverflow-checks=on",
                ])
                .arg(root.join("tide-std/src/lib.rs"))
                .arg(format!(
                    "--extern=tide_macros={}/libtide_macros.so",
                    dir.display()
                ))
                .arg("--out-dir")
                .arg(&dir),
        );
        dir
    })
}

fn package(name: &str, source: &str) -> PathBuf {
    let dir = PathBuf::from(env!("CARGO_TARGET_TMPDIR"))
        .join("build")
        .join(name);
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(dir.join("package/src")).unwrap();
    std::fs::write(dir.join("package/src/demo.rs"), source).unwrap();
    dir
}

fn driver(dir: &Path) -> Command {
    let std = std_dir();
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

#[test]
fn checked_hello_builds_without_running_and_ignores_candidate_configuration() {
    let dir = package("hello", HELLO);
    std::fs::create_dir(dir.join("package/.cargo")).unwrap();
    std::fs::write(dir.join("package/.cargo/config.toml"), "[build]\nrustflags=['--cfg=attacker']\nrustc-wrapper='missing'\n[target.x86_64-unknown-linux-gnu]\nlinker='missing'\n").unwrap();
    std::fs::write(
        dir.join("package/rust-toolchain.toml"),
        "[toolchain]\nchannel='nonexistent'\n",
    )
    .unwrap();
    for _ in 0..2 {
        let out = checked(
            driver(&dir)
                .current_dir(dir.join("package"))
                .env("PATH", "/nonexistent")
                .env("TMPDIR", "/nonexistent")
                .env("RUSTFLAGS", "-Coverflow-checks=off -Clinker=missing")
                .env("RUSTC_WRAPPER", "missing")
                .env("RUSTUP_TOOLCHAIN", "missing")
                .env("LD_LIBRARY_PATH", "/nonexistent"),
        );
        assert!(out.stdout.is_empty(), "the driver executed the program");
        assert!(out.stderr.is_empty(), "{}", stderr(&out));
        assert!(dir.join("out/demo.interface.json").is_file());
        let out = checked(&mut Command::new(dir.join("out/demo")));
        assert_eq!(out.stdout, b"HELLO_RAN\n");
        std::fs::remove_file(dir.join("out/demo")).unwrap();
    }
}

#[test]
fn the_backstop_rejects_address_comparisons_in_every_profile() {
    let source = r#"
fn noop() {}
fn other() {}
trait Addr { fn f(&self, which: bool) -> impl PartialOrd; }
impl Addr for u8 {
    fn f(&self, which: bool) -> impl PartialOrd {
        if which { noop as fn() } else { other as fn() }
    }
}
#[tide::main]
fn main() {
    let first = "1".parse::<u8>().unwrap() == 1;
    println!("{}", 1u8.f(first) < 1u8.f(!first));
}
"#;
    let dir = package("profile-backstop", source);
    for emit in ["link", "metadata"] {
        for profile in ["debug", "release"] {
            let out = driver(&dir)
                .arg("--tide-test-backstop")
                .arg(format!("--emit={emit}"))
                .arg(format!("--tide-profile={profile}"))
                .output()
                .unwrap();
            assert_eq!(
                out.status.code(),
                Some(1),
                "{emit}/{profile}: {}",
                stderr(&out)
            );
            assert!(
                stderr(&out).contains("FX0520: `impl core::cmp::PartialOrd for fn()`"),
                "{emit}/{profile}: {}",
                stderr(&out)
            );
            assert!(out.stdout.is_empty());
            assert!(!dir.join("out/demo").exists());
            assert!(!dir.join("out/demo.interface.json").exists());
            assert_eq!(
                std::fs::read_dir(dir.join("out/.tide-cache"))
                    .unwrap()
                    .count(),
                0
            );
        }
    }
}

#[test]
fn executable_tests_use_the_production_prelude_and_do_not_run_main() {
    let dir = package("test", HELLO);
    let out = checked(driver(&dir).args(["--crate-type=lib", "--test"]));
    assert!(out.stdout.is_empty(), "the driver executed the test");
    let out = checked(Command::new(dir.join("out/demo-test")).arg("--nocapture"));
    assert!(String::from_utf8_lossy(&out.stdout).contains("1 passed"));
    assert!(String::from_utf8_lossy(&out.stdout).contains("TEST_RAN"));
    assert!(!String::from_utf8_lossy(&out.stdout).contains("HELLO_RAN"));
}

#[test]
fn overflow_traps_in_every_profile_in_binaries_and_tests() {
    let source = "fn increment(n: u8) -> u8 { n + 1 }\n\
        fn overflow() { let n = \"255\".parse::<u8>().unwrap(); println!(\"BEFORE_OVERFLOW\"); println!(\"{}\", increment(n)); }\n\
        #[tide::main] fn main() { overflow(); }\n\
        #[test] fn overflows() { overflow(); }\n";
    let dir = package("overflow", source);
    for profile in ["debug", "release"] {
        for test in [false, true] {
            let mut build = driver(&dir);
            build.arg(format!("--tide-profile={profile}"));
            if test {
                build.arg("--test");
            }
            let out = checked(&mut build);
            assert!(out.stdout.is_empty());
            let mut run = Command::new(dir.join(if test { "out/demo-test" } else { "out/demo" }));
            run.env_clear();
            if test {
                run.arg("--nocapture");
            }
            let out = run.stdin(Stdio::null()).output().unwrap();
            assert_eq!(
                out.status.code(),
                Some(101),
                "{profile}, test={test}: {}",
                stderr(&out)
            );
            assert!(
                stderr(&out).contains("attempt to add with overflow"),
                "{}",
                stderr(&out)
            );
            assert!(String::from_utf8_lossy(&out.stdout).contains("BEFORE_OVERFLOW"));
        }
    }
}

#[test]
fn cache_separates_metadata_link_profiles_and_test_configuration() {
    let dir = package(
        "cache",
        "#[tide::main] fn main() {}\n#[test] fn passes() {}\n",
    );
    let entries = || {
        std::fs::read_dir(dir.join("out/.tide-cache"))
            .unwrap()
            .count()
    };
    checked(driver(&dir).arg("--emit=metadata"));
    assert_eq!(entries(), 1);
    assert!(!dir.join("out/demo").exists());
    for profile in ["debug", "release"] {
        for test in [false, true] {
            let mut build = driver(&dir);
            build.arg(format!("--tide-profile={profile}"));
            if test {
                build.arg("--test");
            }
            checked(&mut build);
        }
    }
    assert_eq!(entries(), 5);
    let before: Vec<_> = std::fs::read_dir(dir.join("out/.tide-cache"))
        .unwrap()
        .map(|e| {
            let e = e.unwrap();
            (e.path(), e.metadata().unwrap().modified().unwrap())
        })
        .collect();
    std::fs::remove_file(dir.join("out/demo-test")).unwrap();
    checked(driver(&dir).args(["--test", "--tide-profile=release"]));
    assert_eq!(entries(), 5);
    for (path, modified) in before {
        assert_eq!(path.metadata().unwrap().modified().unwrap(), modified);
    }
    checked(&mut Command::new(dir.join("out/demo-test")));
}

#[test]
fn rejected_source_never_links_or_runs_and_removes_stale_outputs() {
    let valid = "#[tide::main] fn main() {}\n#[test] fn passes() {}\n";
    let dir = package("reject", valid);
    for (source, flags, executable, code) in [
        (
            "#[tide::main] fn main() { println!(\"REJECTED_RAN\"); let _ = std::__rt::enter(|_| ()); }",
            vec![],
            "demo",
            "FX0520",
        ),
        (
            "fn f() {}\n#[tide::main] fn main() { println!(\"REJECTED_RAN {:?}\", f as fn()); }",
            vec!["--tide-test-backstop"],
            "demo",
            "FX0520",
        ),
        (
            "#[tide::main] fn main() { println!(\"REJECTED_RAN\"); }\n#[test] fn calls_entry() { main(); }",
            vec!["--test"],
            "demo-test",
            "FX0506",
        ),
        (
            "fn main() { println!(\"REJECTED_RAN\"); }",
            vec![],
            "demo",
            "FX0512",
        ),
        (
            "core::arch::global_asm!(\".error \\\"ASSEMBLER_RAN\\\"\");\nfn need<T: Pure>(_: T) {}\n#[tide::main] fn main(root: Root) { need(root.clock); }",
            vec!["--tide-grant=unsafe", "--tide-test-backstop"],
            "demo",
            "FX0517",
        ),
        (
            "unsafe extern \"C\" { safe fn tide_missing_link_symbol_for_test(); }\n#[tide::main] fn main() { tide_missing_link_symbol_for_test(); }",
            vec!["--tide-grant=unsafe"],
            "demo",
            "linking with",
        ),
    ] {
        std::fs::write(dir.join("package/src/demo.rs"), valid).unwrap();
        checked(driver(&dir).args(&flags));
        assert!(dir.join("out").join(executable).exists());
        let count = std::fs::read_dir(dir.join("out/.tide-cache"))
            .unwrap()
            .count();
        std::fs::write(dir.join("package/src/demo.rs"), source).unwrap();
        let out = driver(&dir).args(&flags).output().unwrap();
        assert_eq!(out.status.code(), Some(1), "{}", stderr(&out));
        assert!(stderr(&out).contains(code), "{}", stderr(&out));
        assert!(!stderr(&out).contains("ASSEMBLER_RAN"), "{}", stderr(&out));
        assert!(out.stdout.is_empty());
        assert!(!dir.join("out").join(executable).exists());
        assert!(!dir.join("out/demo.interface.json").exists());
        assert_eq!(
            std::fs::read_dir(dir.join("out/.tide-cache"))
                .unwrap()
                .count(),
            count
        );
    }
}

#[test]
fn granted_native_inputs_are_rebuilt_without_cache_reuse() {
    let dir = package("native-input", "");
    let input = dir.join("package/src/payload.bin");
    let assembly = format!(
        ".pushsection .rodata\n.global tide_payload\ntide_payload:\n.incbin {input:?}\n.popsection"
    );
    let source = format!(
        "core::arch::global_asm!({assembly:?});\nunsafe extern \"C\" {{ static tide_payload: u8; }}\n#[tide::main] fn main() {{ println!(\"{{}}\", unsafe {{ tide_payload }}); }}"
    );
    std::fs::write(dir.join("package/src/demo.rs"), source).unwrap();
    for byte in [1u8, 2] {
        std::fs::write(&input, [byte]).unwrap();
        checked(driver(&dir).arg("--tide-grant=unsafe"));
        let out = checked(&mut Command::new(dir.join("out/demo")));
        assert_eq!(out.stdout, format!("{byte}\n").as_bytes());
        assert_eq!(
            std::fs::read_dir(dir.join("out/.tide-cache"))
                .unwrap()
                .count(),
            0
        );
    }
}

#[test]
fn link_mode_keeps_the_source_loader_boundary() {
    let dir = package("loader", "");
    let canary = dir.join("private.rs");
    std::fs::write(&canary, "PRIVATE_LINK_CANARY").unwrap();
    std::fs::write(
        dir.join("package/src/demo.rs"),
        format!(
            "#[tide::main] fn main() {{ println!(\"{{}}\", core::include_str!({canary:?})); }}"
        ),
    )
    .unwrap();
    let out = driver(&dir).output().unwrap();
    assert_eq!(out.status.code(), Some(1), "{}", stderr(&out));
    assert!(!stderr(&out).contains("PRIVATE_LINK_CANARY"));
    assert!(!dir.join("out/demo").exists());
    assert_eq!(
        std::fs::read_dir(dir.join("out/.tide-cache"))
            .unwrap()
            .count(),
        0
    );
}
