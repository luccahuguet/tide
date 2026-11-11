//! Reviewed demo programs built through the driver against production std; only this trusted
//! test caller executes them. This is not the RUN-3 sandbox or Tide's test runtime.
use std::path::{Path, PathBuf};
use std::process::{Command, Output, Stdio};
use std::sync::OnceLock;

#[path = "../../tide-std/tests/support/mod.rs"]
mod support;

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
        support::build_std(&dir)
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

fn paired_test_driver(dir: &Path) -> Command {
    let mut command = driver(dir);
    command
        .args(["--crate-type=lib", "--test", "--tide-test-library"])
        .arg(dir.join("candidate/src/lib.rs"))
        .stdin(Stdio::null());
    command
}

#[test]
fn separate_library_tests_require_public_items_and_admit_both_crates() {
    let suite = "#[test] fn paints() { assert_eq!(tide_candidate::pigment(), 7); }";
    let dir = package("separate-library-tests", suite);
    let candidate = dir.join("candidate/src/lib.rs");
    std::fs::create_dir_all(candidate.parent().unwrap()).unwrap();
    let test = |emit: &str| {
        let mut command = paired_test_driver(&dir);
        command.arg(format!("--emit={emit}"));
        command
    };
    let executable = dir.join("out/demo-test");
    for emit in ["metadata", "link"] {
        let library = "fn pigment() -> u8 { 7 } #[cfg(test)] mod writer { compile_error!(\"writer tests enabled\"); }";
        std::fs::write(&candidate, format!("pub {library}")).unwrap();
        checked(&mut test(emit));
        if emit == "link" {
            let run = checked(Command::new(&executable).arg("--exact").arg("paints"));
            assert!(String::from_utf8_lossy(&run.stdout).contains("1 passed"));
        }
        std::fs::write(&candidate, format!("pub(crate) {library}")).unwrap();
        let out = test(emit).output().unwrap();
        assert_eq!(out.status.code(), Some(1), "{}", stderr(&out));
        assert!(stderr(&out).contains("E0603"), "{}", stderr(&out));
        assert!(std::fs::read_dir(dir.join("out")).unwrap().next().is_none());
    }

    let unsafe_library = "pub fn pigment() -> u8 { unsafe { 7 } }";
    let unsafe_suite = "#[test] fn paints() { unsafe { tide_candidate::pigment(); } }";
    for (library, suite) in [
        (unsafe_library, suite),
        ("pub fn pigment() -> u8 { 7 }", unsafe_suite),
    ] {
        std::fs::write(&candidate, library).unwrap();
        std::fs::write(dir.join("package/src/demo.rs"), suite).unwrap();
        let out = test("link").output().unwrap();
        assert_eq!(out.status.code(), Some(1), "{}", stderr(&out));
        assert!(stderr(&out).contains("FX0901"), "{}", stderr(&out));
        assert!(!executable.exists());
    }

    let out = test("link")
        .arg("--tide-dep-dir")
        .arg(dir.join("candidate"))
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(2), "{}", stderr(&out));
    assert!(stderr(&out).contains("trusted dependencies must be separate"));
}

#[test]
fn separate_library_tests_cannot_read_each_others_sources() {
    let dir = package("separate-library-source-reads", "");
    let candidate = dir.join("candidate/src/lib.rs");
    let suite = dir.join("package/src/demo.rs");
    std::fs::create_dir_all(candidate.parent().unwrap()).unwrap();
    for (reader, target, prefix) in [
        (&candidate, &suite, ""),
        (&suite, &candidate, "extern crate tide_candidate;"),
    ] {
        std::fs::write(target, "// foreign source canary").unwrap();
        std::fs::write(
            reader,
            format!("{prefix}\ncore::compile_error!(core::include_str!({target:?}));"),
        )
        .unwrap();
        let out = paired_test_driver(&dir).output().unwrap();
        let diagnostic = stderr(&out);
        assert_eq!(out.status.code(), Some(1), "{diagnostic}");
        assert!(diagnostic.contains("FX0520"), "{diagnostic}");
        assert!(
            !diagnostic.contains("foreign source canary"),
            "{diagnostic}"
        );
    }
}

#[test]
fn separate_library_tests_accept_local_impls_and_check_pure_at_use() {
    let suite = r#"
extern crate tide_candidate as solution;
mod test {
    use solution::Shape;

    fn sides<T: Shape>(shape: T) -> u8 { shape.sides() }
    fn empty() {}

    #[test] fn counts_sides() { assert_eq!(sides(empty as fn()), 3); }
}
"#;
    let dir = package("separate-library-impls-and-clauses", suite);
    let candidate = dir.join("candidate/src/lib.rs");
    std::fs::create_dir_all(candidate.parent().unwrap()).unwrap();
    std::fs::write(
        &candidate,
        r#"
pub trait Shape { fn sides(&self) -> u8; }
impl Shape for fn() {
    #[inline]
    fn sides(&self) -> u8 { 3 }
}
pub fn require_pure<T: Pure>() {}
"#,
    )
    .unwrap();
    checked(&mut paired_test_driver(&dir));
    let run =
        checked(Command::new(dir.join("out/demo-test")).args(["--exact", "test::counts_sides"]));
    assert!(String::from_utf8_lossy(&run.stdout).contains("1 passed"));

    std::fs::write(
        dir.join("package/src/demo.rs"),
        r#"
#[test]
fn check_clause() { tide_candidate::require_pure::<std::time::Clock>(); }
"#,
    )
    .unwrap();
    let out = paired_test_driver(&dir).output().unwrap();
    let diagnostic = stderr(&out);
    assert_eq!(out.status.code(), Some(1), "{diagnostic}");
    assert!(
        diagnostic
            .lines()
            .any(|line| line.contains("demo.rs:")
                && line.contains("FX0517: `Clock` is never `Pure`")),
        "{diagnostic}"
    );
    assert!(!dir.join("out/demo-test").exists());
}

#[test]
fn task_lifecycle_builds_and_runs_against_production_std() {
    let source = include_str!("../../../conformance/cases/tasks/task_lifecycle.rs");
    let dir = package("task-lifecycle", source);
    for profile in ["debug", "release"] {
        checked(driver(&dir).arg(format!("--tide-profile={profile}")));
        let out = checked(&mut Command::new(dir.join("out/demo")));
        assert_eq!(out.stdout, b"TASKS_RAN\n");
        assert!(out.stderr.is_empty(), "{}", stderr(&out));
    }
}

#[test]
fn shared_counter_runs_against_production_std() {
    let source = include_str!("../../../conformance/cases/tasks/shared_counter.rs");
    let dir = package("shared-counter", source);
    checked(&mut driver(&dir));
    checked(&mut Command::new(dir.join("out/demo")));
}

#[test]
fn collections_execute_with_fixed_and_capability_seeded_keys() {
    let dir = package(
        "collections",
        include_str!("../../../conformance/cases/authority/collections_executable.rs"),
    );
    checked(&mut driver(&dir));
    let first = checked(&mut Command::new(dir.join("out/demo")));
    let second = checked(&mut Command::new(dir.join("out/demo")));
    assert!(!first.stdout.is_empty());
    assert_eq!(
        first.stdout, second.stdout,
        "fixed-key iteration changed across processes"
    );
    assert_ne!(
        first.stderr, second.stderr,
        "OS-seeded hash fingerprints repeated"
    );
}

#[test]
fn collections_disjoint_lookups_enforce_safety() {
    let dir = package(
        "collection-unchecked-lookup",
        r#"
#[tide::main]
fn main() {
    let mut map = std::collections::HashMap::from([(1, 10), (2, 20)]);
    // Distinct keys satisfy the unchecked lookup's safety contract.
    let [first, missing, second] = unsafe { map.get_disjoint_unchecked_mut([&1, &0, &2]) };
    *first.unwrap() += 1;
    assert!(missing.is_none());
    *second.unwrap() += 2;
    assert_eq!(map[&1], 11);
    assert_eq!(map[&2], 22);
}
"#,
    );
    let out = driver(&dir).stdin(Stdio::null()).output().unwrap();
    assert_eq!(out.status.code(), Some(1), "{}", stderr(&out));
    assert!(stderr(&out).contains("FX0901"), "{}", stderr(&out));
    assert!(!dir.join("out/demo").exists());
    checked(driver(&dir).arg("--tide-grant=unsafe"));
    checked(&mut Command::new(dir.join("out/demo")));

    let dir = package(
        "collection-overlapping-lookup",
        r#"
#[tide::main]
fn main() {
    let mut map = std::collections::HashMap::from([(1, 10)]);
    let _ = map.get_disjoint_mut([&1, &1]);
}
"#,
    );
    checked(&mut driver(&dir));
    let out = Command::new(dir.join("out/demo"))
        .stdin(Stdio::null())
        .output()
        .unwrap();
    assert!(!out.status.success());
    assert!(
        stderr(&out).contains("duplicate keys found"),
        "{}",
        stderr(&out)
    );
}

#[test]
fn collections_cannot_draw_ambient_keys_through_a_generic_hasher() {
    for (name, expression) in [
        ("map", "std::collections::HashMap::<u8, u8, std::hash::RandomState>::default()"),
        ("set", "std::collections::HashSet::<u8, std::hash::RandomState>::default()"),
    ] {
        let dir = package(
            &format!("ambient-collection-{name}"),
            &format!("#[tide::main] fn main() {{ let _ = {expression}; }}"),
        );
        let out = driver(&dir).stdin(Stdio::null()).output().unwrap();
        assert_eq!(out.status.code(), Some(1), "{}", stderr(&out));
        assert!(
            stderr(&out).contains("FX0520: `impl core::default::Default for std::hash::random::RandomState`"),
            "{}", stderr(&out)
        );
        assert!(!dir.join("out/demo").exists());
    }
}

#[test]
fn task_failures_and_cancellation_build_and_run_against_production_std() {
    let source = include_str!("../../../conformance/cases/tasks/task_cancellation.rs");
    let dir = package("task-cancellation", source);
    for profile in ["debug", "release"] {
        checked(driver(&dir).arg(format!("--tide-profile={profile}")));
        let out = checked(&mut Command::new(dir.join("out/demo")));
        assert_eq!(out.stdout, b"CANCELLATION_RAN\n");
        assert!(out.stderr.is_empty(), "{}", stderr(&out));
    }
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

/// std::io's core (bead fx-ecb) passes the driver and runs: an in-memory reader through
/// `BufReader` and `BufRead::lines`, writes through `Write` and `BufWriter`, `Cursor` and
/// `SeekFrom`, an `io::Error` from an `ErrorKind` through `?`, and `is_cancelled()`.
#[test]
fn io_core_builds_and_runs() {
    let source = r#"
use std::io::{self, BufRead, BufReader, BufWriter, Cursor, Read, Seek, SeekFrom, Write};

fn numbered(input: impl Read, out: &mut impl Write) -> io::Result<usize> {
    let mut words = 0;
    for (i, line) in BufReader::new(input).lines().enumerate() {
        let line = line?;
        words += line.split_whitespace().count();
        writeln!(out, "{}: {line}", i + 1)?;
    }
    out.flush()?;
    Ok(words)
}

fn find(found: bool) -> io::Result<()> {
    if found { Ok(()) } else { Err(io::ErrorKind::NotFound.into()) }
}

#[tide::main]
fn main() -> io::Result<()> {
    let mut out = Vec::new();
    let words = numbered(&b"alpha beta\r\ngamma\n"[..], &mut out)?;
    print!("{}", String::from_utf8(out).unwrap());

    let mut cursor = Cursor::new(Vec::new());
    {
        let mut w = BufWriter::new(&mut cursor);
        w.write_all(b"xyz")?;
        w.flush()?;
    }
    cursor.seek(SeekFrom::Start(1))?;
    let mut rest = String::new();
    cursor.read_to_string(&mut rest)?;

    let missing = find(false).unwrap_err();
    let cancelled = io::Error::from(io::ErrorKind::Cancelled);
    let custom = io::Error::new(io::ErrorKind::InvalidData, "bad header");
    println!(
        "words={words} rest={rest} missing={missing} {:?} cancelled={} {} custom={custom}",
        missing.kind(),
        missing.is_cancelled(),
        cancelled.is_cancelled(),
    );
    Ok(())
}
"#;
    let dir = package("io-core", source);
    let out = checked(&mut driver(&dir));
    assert!(out.stderr.is_empty(), "{}", stderr(&out));
    let out = checked(&mut Command::new(dir.join("out/demo")));
    assert_eq!(
        String::from_utf8_lossy(&out.stdout),
        "1: alpha beta\n2: gamma\nwords=3 rest=yz missing=entity not found NotFound \
         cancelled=false true custom=bad header\n"
    );
}

#[test]
fn float_math_builds_and_executes_against_production_std() {
    let source = include_str!("../../../conformance/cases/authority/float_math.rs");
    let dir = package("float-math", source);
    for profile in ["debug", "release"] {
        checked(driver(&dir).arg(format!("--tide-profile={profile}")));
        let out = checked(&mut Command::new(dir.join("out/demo")));
        assert_eq!(out.stdout, b"FLOAT_MATH_RAN\n");
        assert!(out.stderr.is_empty(), "{}", stderr(&out));
        checked(driver(&dir).args(["--test", &format!("--tide-profile={profile}")]));
        let out = checked(&mut Command::new(dir.join("out/demo-test")));
        assert!(String::from_utf8_lossy(&out.stdout).contains("2 passed"));
    }
}

#[test]
fn float_banned_methods_still_report_fx0520_with_production_std() {
    // Each rejection is isolated so one diagnostic cannot conceal an unguarded method.
    for ty in ["f32", "f64"] {
        for (method, args) in [
            ("to_bits", ""),
            ("is_sign_positive", ""),
            ("is_sign_negative", ""),
            ("copysign", "x"),
            ("signum", ""),
            ("total_cmp", "&x"),
            ("min", "x"),
            ("max", "x"),
            ("sin", ""),
            ("cos", ""),
            ("tan", ""),
            ("asin", ""),
            ("acos", ""),
            ("atan", ""),
            ("sinh", ""),
            ("cosh", ""),
            ("tanh", ""),
            ("asinh", ""),
            ("acosh", ""),
            ("atanh", ""),
            ("exp", ""),
            ("exp2", ""),
            ("exp_m1", ""),
            ("ln", ""),
            ("ln_1p", ""),
            ("log2", ""),
            ("log10", ""),
            ("cbrt", ""),
            ("round_ties_even", ""),
            ("powf", "x"),
            ("hypot", "x"),
            ("atan2", "x"),
            ("abs_sub", "x"),
            ("powi", "3"),
            ("sin_cos", ""),
            ("log", "x"),
            ("fract", ""),
            ("recip", ""),
            ("to_degrees", ""),
            ("to_radians", ""),
            ("div_euclid", "x"),
            ("rem_euclid", "x"),
            ("midpoint", "x"),
            ("clamp", "0.0, 1.0"),
        ] {
            let dir = package(
                &format!("float-reject-{ty}-{method}"),
                &format!(
                    "#[tide::main] fn main() {{ let x = 0.5{ty}; let _ = x.{method}({args}); }}"
                ),
            );
            let out = driver(&dir)
                .arg("--emit=metadata")
                .stdin(Stdio::null())
                .output()
                .unwrap();
            assert_eq!(
                out.status.code(),
                Some(1),
                "{ty}::{method}: {}",
                stderr(&out)
            );
            assert!(
                stderr(&out).contains("FX0520:"),
                "{ty}::{method}: {}",
                stderr(&out)
            );
            assert!(
                stderr(&out).contains(method),
                "{ty}::{method}: {}",
                stderr(&out)
            );
        }
    }
}

#[test]
fn error_derive_builds_and_executes_against_production_std() {
    let source = include_str!("../../../conformance/cases/errors/derive.rs");
    let dir = package("error-derive", source);
    checked(&mut driver(&dir));
    let out = checked(&mut Command::new(dir.join("out/demo")));
    assert_eq!(out.stdout, b"ERROR_DERIVE_RAN\n");
    checked(driver(&dir).args(["--crate-type=lib", "--test"]));
    let out = checked(&mut Command::new(dir.join("out/demo-test")));
    assert!(String::from_utf8_lossy(&out.stdout).contains("1 passed"));
}

#[test]
fn error_derive_rejects_malformed_helpers_at_the_input() {
    for (name, source, message, line) in [
        (
            "literal",
            "#[derive(Debug, Error)]\n#[error(12)]\nstruct Bad;",
            "expected a string literal or `transparent`",
            2,
        ),
        (
            "duplicate",
            "#[derive(Debug, Error)]\n#[error(\"one\")]\n#[error(\"two\")]\nstruct Bad;",
            "duplicate #[error]",
            3,
        ),
        (
            "transparent",
            "#[derive(Debug, Error)]\n#[error(transparent)]\nstruct Bad(u8, u8);",
            "requires exactly one field",
            2,
        ),
        (
            "from",
            "#[derive(Debug, Error)]\n#[error(\"bad\")]\nstruct Bad(#[from] std::io::Error, u8);",
            "no fields other than the source",
            3,
        ),
        (
            "sources",
            "#[derive(Debug, Error)]\n#[error(\"bad\")]\nstruct Bad { #[source] a: std::io::Error, #[source] b: std::io::Error }",
            "more than one source",
            3,
        ),
        (
            "field",
            "#[derive(Debug, Error)]\n#[error(\"bad\")]\nstruct Bad { #[error(\"field\")] n: u8 }",
            "not allowed on a field",
            3,
        ),
        (
            "placement",
            "#[derive(Debug, Error)]\n#[source]\n#[error(\"bad\")]\nstruct Bad;",
            "only allowed on a field",
            2,
        ),
        (
            "source_args",
            "#[derive(Debug, Error)]\n#[error(\"bad\")]\nstruct Bad(#[source(wrong)] std::io::Error);",
            "unexpected token",
            3,
        ),
        (
            "format",
            "#[derive(Debug, Error)]\n#[error(\"{missing}\")]\nstruct Bad;",
            "missing",
            2,
        ),
        (
            "empty_enum",
            "#[derive(Debug, Error)]\n#[error(12)]\nenum Bad {}",
            "expected a string literal or `transparent`",
            2,
        ),
        (
            "overridden_bad_default",
            "#[derive(Debug, Error)]\n#[error(12)]\nenum Bad { #[error(\"override\")] Unit }",
            "expected a string literal or `transparent`",
            2,
        ),
        (
            "empty_duplicate",
            "#[derive(Debug, Error)]\n#[error(\"one\")]\n#[error(\"two\")]\nenum Bad {}",
            "duplicate #[error]",
            3,
        ),
        (
            "empty_placement",
            "#[derive(Debug, Error)]\n#[from]\nenum Bad {}",
            "only allowed on a field",
            2,
        ),
        (
            "positional_ambiguity",
            "#[derive(Debug, Error)]\n#[error(\"{} {1}\", .0, 7)]\nstruct Bad(u8);",
            "ambiguous positional argument",
            2,
        ),
        (
            "invalid_index",
            "#[derive(Debug, Error)]\n#[error(\"{+0}\")]\nstruct Bad(u8);",
            "invalid format string",
            2,
        ),
        (
            "overflowing_index",
            "#[derive(Debug, Error)]\n#[error(\"{184467440737095516160}\")]\nstruct Bad(u8);",
            "invalid format string",
            2,
        ),
        (
            "option_arity",
            "type Option<A, B> = (A, B);\n#[derive(Debug, Error)]\n#[error(\"bad\")]\nstruct Bad(#[from] Option<std::io::Error, u8>);",
            "as_dyn_error",
            2,
        ),
        (
            "missing_variant",
            "#[derive(Debug, Error)]\nenum Bad {\n    #[error(\"one\")]\n    One,\n    Two,\n}",
            "missing #[error] attribute on variant",
            5,
        ),
    ] {
        let dir = package(&format!("error-negative-{name}"), source);
        let out = driver(&dir)
            .args([
                "--crate-type=lib",
                "--emit=metadata",
                "--error-format=short",
            ])
            .output()
            .unwrap();
        assert_eq!(out.status.code(), Some(1), "{name}: {}", stderr(&out));
        let err = stderr(&out);
        assert!(
            err.lines()
                .any(|l| l.contains(&format!("demo.rs:{line}:")) && l.contains(message)),
            "{name}: {err}"
        );
        assert!(!dir.join("out/demo").exists());
    }
}

#[test]
fn error_derive_formats_the_original_pointer_with_an_unsafe_grant() {
    let dir = package(
        "error-pointer",
        r#"
#[derive(Debug, Error)]
#[error("{address:p}")]
struct Address<'a> { address: &'a u8 }
#[tide::main]
fn main() {
    let byte = 7;
    let address = &byte;
    assert_eq!(Address { address }.to_string(), format!("{address:p}"));
    println!("POINTER_RAN");
}
"#,
    );
    checked(driver(&dir).arg("--tide-grant=unsafe"));
    let out = checked(&mut Command::new(dir.join("out/demo")));
    assert_eq!(out.stdout, b"POINTER_RAN\n");
}

#[test]
fn generic_error_types_still_require_pure() {
    let dir = package(
        "error-generic-pure",
        r#"
#[derive(Debug, Error)]
#[error("bounded")]
enum Bounded<T: Pure> { Value { value: T } }
#[tide::main]
fn main(root: Root) { let _ = Bounded::Value { value: root.net }; }
"#,
    );
    let out = driver(&dir).output().unwrap();
    assert_eq!(out.status.code(), Some(1), "{}", stderr(&out));
    assert!(stderr(&out).contains("FX0517"), "{}", stderr(&out));
    assert!(!dir.join("out/demo").exists());
}

#[test]
fn generic_error_patterns_keep_distinct_type_arguments() {
    let dir = package(
        "error-pattern-args",
        r#"
#[derive(Debug, Error)]
#[error("bounded")]
enum Bounded<P: Pure, T> { Value { marker: P, value: T } }
#[tide::main]
fn main(root: Root) {
    let e = Bounded::Value { marker: 7u8, value: root.net };
    assert_eq!(e.to_string(), "bounded");
    println!("PATTERNS_RAN");
}
"#,
    );
    let out = checked(&mut driver(&dir));
    assert!(out.stderr.is_empty(), "{}", stderr(&out));
    let out = checked(&mut Command::new(dir.join("out/demo")));
    assert_eq!(out.stdout, b"PATTERNS_RAN\n");
}
