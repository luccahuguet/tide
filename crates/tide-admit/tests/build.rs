//! Reviewed demo programs built through the driver against production std; only this trusted
//! test caller executes them. This is not the RUN-3 sandbox or Tide's test runtime.
use std::path::{Path, PathBuf};
use std::process::{Command, Output, Stdio};
use std::sync::OnceLock;

#[path = "../../tide-std/tests/support/mod.rs"]
mod support;

const HELLO: &str = include_str!("../../../conformance/cases/authority/root_executable.rs");

#[test]
fn executable_contracts_run_in_debug_test_and_erase_in_release() {
    for (name, source, count, marker) in [
        (
            "executable-contracts",
            include_str!("../../../conformance/cases/contracts/executable.rs"),
            16,
            "CONTRACTS_RAN\n",
        ),
        (
            "scoped-clauses",
            include_str!("../../../conformance/cases/contracts/scoped_clauses.rs"),
            5,
            "SCOPED_CLAUSES_RAN\n",
        ),
        (
            "clause-expressions",
            include_str!("../../../conformance/cases/contracts/clause_expressions.rs"),
            3,
            "CLAUSE_EXPRESSIONS_RAN\n",
        ),
    ] {
        let dir = package(name, source);
        for profile in ["debug", "release"] {
            for test in [false, true] {
                let mut build = driver(&dir);
                build.arg(format!("--tide-profile={profile}"));
                if test {
                    build.arg("--test");
                }
                checked(&mut build);
                let out = checked(&mut Command::new(dir.join(if test {
                    "out/demo-test"
                } else {
                    "out/demo"
                })));
                if test {
                    assert!(
                        String::from_utf8_lossy(&out.stdout).contains(&format!("{count} passed")),
                        "{}",
                        String::from_utf8_lossy(&out.stdout)
                    );
                } else {
                    assert_eq!(out.stdout, marker.as_bytes());
                }
            }
        }
    }
}

#[test]
fn contract_requires_body_keeps_direct_call_and_panic_locations() {
    for (name, attr, body, blamed_line, expected) in [
        ("nested", "", "f(y)", "f(y)", "requires violated: x > 0"),
        ("panic", "", "xs[i]", "xs[i]", "index out of bounds"),
        (
            "tracked-panic",
            "#[track_caller]",
            "xs[i]",
            "fn main() { g(0, &[], 0); }",
            "index out of bounds",
        ),
    ] {
        let source = format!(
            r#"
#[requires(x > 0)]
fn f(x: i32) -> i32 {{ x }}
#[requires(y >= 0)]
{attr}
fn g(y: i32, xs: &[i32], i: usize) -> i32 {{
    {body}
}}
#[tide::main]
fn main() {{ g(0, &[], 0); }}
"#
        );
        let dir = package(&format!("contract-body-{name}"), &source);
        let build = checked(&mut driver(&dir));
        assert!(
            !stderr(&build).contains("unnecessary braces"),
            "{}",
            stderr(&build)
        );
        let out = Command::new(dir.join("out/demo"))
            .stdin(Stdio::null())
            .output()
            .unwrap();
        assert_eq!(out.status.code(), Some(101));
        let line = source
            .lines()
            .position(|line| line.trim() == blamed_line)
            .unwrap()
            + 1;
        let message = stderr(&out);
        assert!(message.contains(expected), "{message}");
        assert!(message.contains(&format!("demo.rs:{line}:")), "{message}");
        if name == "nested" {
            assert!(message.contains(&format!("demo.rs:{line}:5")), "{message}");
        }
    }
}

#[test]
fn contract_requires_keeps_body_items_out_of_clause_scope() {
    for (name, qualifier, attr) in [("const", "const", ""), ("tracked", "", "#[track_caller]")] {
        for (local_limit, argument, exit) in [(1, 50, 0), (1000, 150, 101)] {
            let source = format!(
                r#"
#![deny(unused_braces)]
const LIMIT: i32 = 100;
#[requires(x < LIMIT)]
{attr}
{qualifier} fn value(x: i32) -> i32 {{
    const LIMIT: i32 = {local_limit};
    x + LIMIT - LIMIT
}}
#[tide::main]
fn main() {{ assert_eq!(value({argument}), {argument}); }}
"#
            );
            let dir = package(
                &format!("contract-item-scope-{name}-{local_limit}"),
                &source,
            );
            checked(driver(&dir).arg("--tide-profile=debug"));
            let out = Command::new(dir.join("out/demo"))
                .stdin(Stdio::null())
                .output()
                .unwrap();
            assert_eq!(out.status.code(), Some(exit), "{}", stderr(&out));
            if exit != 0 {
                assert!(
                    stderr(&out).contains("requires violated: x < LIMIT"),
                    "{}",
                    stderr(&out)
                );
            }
        }
    }
}

#[test]
fn contract_requires_never_and_const_bodies_do_not_warn() {
    let dir = package(
        "contract-requires-warnings",
        r#"
#![deny(unreachable_code, unused_braces)]
#[requires(code != 0)]
fn die(code: i32) -> ! { panic!("die {code}") }
#[requires(code != 0)]
#[ensures(true)]
fn die_with_ensures(code: i32) -> ! { panic!("die {code}") }
#[requires(x > 0)]
const fn identity(x: i32) -> i32 { x }
const VALUE: i32 = identity(1);
#[tide::main]
fn main() {
    assert_eq!(VALUE + identity(2), 3);
    let _ = die as fn(i32) -> !;
    let _ = die_with_ensures as fn(i32) -> !;
}
"#,
    );
    for profile in ["debug", "release"] {
        let out = checked(driver(&dir).arg(format!("--tide-profile={profile}")));
        assert!(!stderr(&out).contains("warning:"), "{}", stderr(&out));
        checked(&mut Command::new(dir.join("out/demo")));
    }
}

#[test]
fn contract_rejects_colliding_generated_operator_spans() {
    let dir = package("contract-generated-spans", "collision::clause!();");
    let generator = dir.join("generator.rs");
    std::fs::write(
        &generator,
        r##"
extern crate proc_macro;
#[proc_macro]
pub fn clause(_: proc_macro::TokenStream) -> proc_macro::TokenStream {
    "#[tide_macros::requires({ let p = true; p ==> true })] fn probe() {}".parse().unwrap()
}
"##,
    )
    .unwrap();
    let library = dir.join(format!(
        "{}collision{}",
        std::env::consts::DLL_PREFIX,
        std::env::consts::DLL_SUFFIX
    ));
    let rustc = std::env::var_os("RUSTC").unwrap_or("rustc".into());
    checked(
        Command::new(&rustc)
            .args([
                "--edition=2024",
                "--crate-name=collision",
                "--crate-type=proc-macro",
            ])
            .arg(&generator)
            .arg("-o")
            .arg(&library),
    );
    let macros = std::fs::read_dir(std_dir())
        .unwrap()
        .map(|entry| entry.unwrap().path())
        .find(|p| {
            p.file_name()
                .unwrap()
                .to_str()
                .unwrap()
                .starts_with("libtide_macros-")
        })
        .unwrap();
    let output = Command::new(&rustc)
        .args(["--edition=2024", "--crate-type=lib", "--extern"])
        .arg(format!("collision={}", library.display()))
        .arg("--extern")
        .arg(format!("tide_macros={}", macros.display()))
        .arg(dir.join("package/src/demo.rs"))
        .arg("--out-dir")
        .arg(dir.join("out"))
        .stdin(Stdio::null())
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(1));
    assert!(
        stderr(&output).contains("ambiguous clause spans"),
        "{}",
        stderr(&output)
    );
}

#[test]
fn contract_preserves_source_conjunct_and_caller_blame() {
    let source = r#"
struct Probe;
impl Drop for Probe {
    fn drop(&mut self) {
        if let Some(v) = std::__contract::take_violation() {
            println!("RECORD {:?} {} {} {} {}:{}:{}", v.clause.kind, v.clause.index,
                v.clause.conjunct, v.clause.function, v.file, v.line, v.column);
        }
    }
}

#[requires(
    a > 0 && "é".len() == 2 &&
    b   >  0
)]
fn pair(a: u32, b: u32) {}
#[tide::main]
fn main() {
    let _probe = Probe;
    pair(1, 0);
}
"#;
    let dir = package("contract-blame", source);
    checked(&mut driver(&dir));
    let out = Command::new(dir.join("out/demo"))
        .stdin(Stdio::null())
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(101));
    let message = stderr(&out);
    let line = source
        .lines()
        .position(|line| line.trim() == "pair(1, 0);")
        .unwrap()
        + 1;
    let blamed = format!("demo.rs:{line}:5");
    assert!(message.contains("requires violated: b   >  0"), "{message}");
    assert!(message.contains(&blamed), "{message}");
    let record = String::from_utf8_lossy(&out.stdout);
    assert!(record.contains("RECORD Requires 0 2 pair"), "{record}");
    assert!(record.contains(&blamed), "{record}");
}

#[test]
fn erased_contracts_still_resolve_type_and_borrow_check() {
    for (name, clause, expected) in [
        (
            "type",
            "#[ensures(result == true)] fn invalid() -> u32 { 1 }",
            "E0308",
        ),
        (
            "name",
            "#[ensures(result == missing)] fn invalid() -> u32 { 1 }",
            "E0425",
        ),
        (
            "move",
            "#[ensures(drop(result) == ())] fn invalid() -> Vec<u32> { vec![1] }",
            "E0382",
        ),
        (
            "nested-old",
            "#[ensures(old(old(n)) == result)] fn invalid(n: u32) -> u32 { n }",
            "nested old",
        ),
        (
            "bound-old",
            "#[ensures(forall |i: usize| i < 2 ==> old(i) == i)] fn invalid(i: usize) {}",
            "bound variable",
        ),
        (
            "shared-old",
            "struct Token(u32); #[ensures(old(t).0 == result)] fn invalid(t: &Token) -> u32 { t.0 }",
            "Clone",
        ),
        (
            "alias-mut",
            "type Mut<'a> = &'a mut u32; #[ensures(*x == 4)] fn invalid(x: Mut<'_>) { *x = 4; }",
            "mutable-reference alias in ensures is not supported",
        ),
        (
            "const-old",
            "#[ensures(result == old(n))] const fn invalid(n: u32) -> u32 { n }",
            "old snapshots on const fn are not supported",
        ),
        (
            "const-bad",
            "#[requires(n > 0)] const fn invalid(n: u32) -> u32 { n } const BAD: u32 = invalid(0);",
            "requires violated",
        ),
        (
            "pattern-old",
            "#[ensures(matches!(result, Some(n) if old(n) == n))] fn invalid(n: u8) -> Option<u8> { Some(n) }",
            "bound variable",
        ),
        (
            "const-method",
            "struct S; impl S { #[ensures(true)] const fn invalid(&self) {} }",
            "ensures on a const method is not supported",
        ),
        (
            "const-pattern",
            "#[ensures(result == n)] const fn invalid((n,): (u8,)) -> u8 { n }",
            "const ensures needs identifier parameters",
        ),
        (
            "const-entry",
            "struct S(u32); #[ensures(result == s.0)] const fn invalid(s: S) -> u32 { s.0 }",
            "const ensures needs Copy entry parameters",
        ),
        (
            "for-old",
            "#[ensures(matches!({ let mut s = 0; for n in 0..2 { s += old(n); } s }, 1))] fn invalid(n: u8) {}",
            "bound variable",
        ),
        (
            "while-old",
            "#[ensures(matches!({ let mut it = Some(1); let mut s = 0; while let Some(n) = it { s += old(n); it = None; } s }, 1))] fn invalid(n: u8) {}",
            "bound variable",
        ),
        (
            "format-unused-implication",
            "#[ensures(format!(\"{p}\", p ==> result) == \"true\")] fn invalid(p: bool) -> bool { false }",
            "argument never used",
        ),
        (
            "format-leading-space",
            "#[ensures(result == format!(\"{ n}\"))] fn invalid(n: u8) -> String { String::new() }",
            "invalid format string",
        ),
        (
            "format-raw-capture",
            "#[ensures(result == format!(\"{r#n}\"))] fn invalid(n: u8) -> String { String::new() }",
            "invalid format string",
        ),
    ] {
        let dir = package(
            &format!("contract-reject-{name}"),
            &format!("{clause}\n#[tide::main] fn main() {{}}"),
        );
        for profile in ["debug", "release"] {
            let out = driver(&dir)
                .arg(format!("--tide-profile={profile}"))
                .stdin(Stdio::null())
                .output()
                .unwrap();
            if name == "const-bad" && profile == "release" {
                assert!(out.status.success(), "{}", stderr(&out));
                continue;
            }
            assert_eq!(out.status.code(), Some(1), "{}", stderr(&out));
            assert!(stderr(&out).contains(expected), "{}", stderr(&out));
            if name == "alias-mut" {
                assert!(!stderr(&out).contains("E0500"), "{}", stderr(&out));
            }
            assert!(!dir.join("out/demo").exists());
        }
    }
}

#[test]
fn contract_const_sampled_test_has_an_explicit_limit() {
    let dir = package(
        "const-sampled-limit",
        "fn positive(n: u32) -> bool { n > 0 } #[requires(positive(n))] const fn sample(n: u32) -> u32 { n } #[invariants] const fn looped(mut n: u32) { while n > 0 { invariant!(positive(n)); n -= 1; } }",
    );
    let output = driver(&dir)
        .args(["--crate-type=lib", "--test"])
        .stdin(Stdio::null())
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(1));
    assert!(
        stderr(&output).contains("sampled contracts on const fn are not supported in test builds"),
        "{}",
        stderr(&output)
    );
    assert!(!stderr(&output).contains("E0015"), "{}", stderr(&output));
    checked(driver(&dir).args(["--crate-type=lib", "--test", "--tide-profile=release"]));
    checked(&mut Command::new(dir.join("out/demo-test")));
}

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
        (
            "map",
            "std::collections::HashMap::<u8, u8, std::hash::RandomState>::default()",
        ),
        (
            "set",
            "std::collections::HashSet::<u8, std::hash::RandomState>::default()",
        ),
    ] {
        let dir = package(
            &format!("ambient-collection-{name}"),
            &format!("#[tide::main] fn main() {{ let _ = {expression}; }}"),
        );
        let out = driver(&dir).stdin(Stdio::null()).output().unwrap();
        assert_eq!(out.status.code(), Some(1), "{}", stderr(&out));
        assert!(
            stderr(&out).contains(
                "FX0520: `impl core::default::Default for std::hash::random::RandomState`"
            ),
            "{}",
            stderr(&out)
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

/// `Clock`, `Instant`, `SystemTime` and `Duration` (bead fx-o8j) pass the driver and run: clock
/// readings and their arithmetic, real sleeps of at least their duration, `Cancelled` from a
/// sleep in a cancelled scope, and the panic of `system_time()` on a `monotonic_only()` clock.
#[test]
fn clock_and_time_build_and_run_against_production_std() {
    let source = include_str!("../../../conformance/cases/authority/clock_time.rs");
    let dir = package("clock-time", source);
    for profile in ["debug", "release"] {
        checked(driver(&dir).arg(format!("--tide-profile={profile}")));
        let out = checked(&mut Command::new(dir.join("out/demo")));
        assert_eq!(out.stdout, b"TIME_RAN\n");
        assert!(out.stderr.is_empty(), "{}", stderr(&out));
    }

    let source = "#[tide::main]\nfn main(root: Root) {\n    \
                  let clock = root.clock.monotonic_only();\n    \
                  println!(\"{}\", clock.now() <= clock.now());\n    \
                  let _ = clock.system_time();\n}\n";
    let dir = package("clock-monotonic-only", source);
    checked(&mut driver(&dir));
    let out = Command::new(dir.join("out/demo"))
        .stdin(Stdio::null())
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(101), "{}", stderr(&out));
    assert_eq!(out.stdout, b"true\n");
    assert!(
        stderr(&out).contains(
            "`system_time()` called on a `Clock` from `monotonic_only()`, which cannot read wall \
             time"
        ),
        "{}",
        stderr(&out)
    );
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
fn property_test_harness_generates_shrinks_and_replays() {
    let dir = package(
        "property",
        r#"
use std::test::{Arbitrary, Gen};

#[allow(non_upper_case_globals)]
const error: u8 = 0;

#[test]
fn ordinary() {}

#[test]
fn sum(a: u8, b: u8) {
    assert!(a as u16 + b as u16 <= 300);
}

#[test]
fn vector(xs: Vec<i64>) {
    assert!(xs.iter().map(|x| *x as i128).sum::<i128>() < 100);
}

#[derive(Debug)]
struct Even(u8);
impl Arbitrary for Even {
    fn arbitrary(g: &mut Gen) -> Self { Self(u8::arbitrary(g) & !1) }
}

#[test]
fn even(x: Even) {
    assert!(x.0 < 10);
}

#[test]
fn same_failure(x: u8) {
    if x == 0 { panic!("zero is a different failure"); }
    assert!(x < 10);
}

#[derive(Debug)]
struct Positive(u8);
impl Arbitrary for Positive {
    fn arbitrary(g: &mut Gen) -> Self {
        loop {
            let n = u8::arbitrary(g);
            if n != 0 { return Self(n); }
        }
    }
}
#[test]
fn positive(p: Positive) { assert_eq!(p.0, 0); }

#[test]
fn returned_error(x: u8) -> Result<(), String> {
    if x > 3 { Err(String::from("limit")) } else { Ok(()) }
}

type NestedResult = Result<Result<(), String>, String>;
#[test]
fn nested_error(x: u8) -> NestedResult {
    Ok(if x > 3 { Err(String::from("nested limit")) } else { Ok(()) })
}

#[derive(Debug)]
struct Wrap<T>(T);
impl<T: Arbitrary> Arbitrary for Wrap<T> {
    fn arbitrary(g: &mut Gen) -> Self { Self(T::arbitrary(g)) }
}
#[derive(Debug)]
struct Leaf(u8);
#[test]
fn wrapped(x: Wrap<Leaf>) { assert!(x.0.0 < 3); }

#[derive(Debug)]
struct Many([u8; 512]);
impl Arbitrary for Many {
    fn arbitrary(g: &mut Gen) -> Self { Self(std::array::from_fn(|_| u8::try_from(g.index(16)).unwrap())) }
}

#[test]
fn budget(values: Many) {
    // Seed 4 draws a sum of 3958. Lowering any draw makes this case pass.
    assert!(values.0.iter().map(|n| *n as u32).sum::<u32>() < 3958);
}

mod outer {
    mod inner {
        use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};

        #[derive(Debug)]
        struct Private<T, const N: usize> where T: Copy {
            field: [T; N],
        }
        #[derive(Debug)]
        enum Shape { None, Tuple(Private<u16, 2>), Named { on: bool } }

        type Input = Option<Shape>;
        type Bytes = [u8];

        #[derive(Debug)]
        struct Node { children: Vec<Node> }

        #[derive(Debug)]
        struct A<T> { b: Vec<B<T>>, c: Vec<C<T>> }
        #[derive(Debug)]
        struct B<T> { a: Vec<A<T>>, value: T }
        #[derive(Debug)]
        struct C<T> { b: Vec<B<T>> }

        type Text<'a> = &'a str;
        type MutText<'a> = &'a mut str;
        type Counter<'a> = &'a mut u32;
        type Shared<'a> = &'a Private<u8, 2>;
        type Slice<'a> = &'a [u8];
        type MutSlice<'a> = &'a mut [u8];

        #[test]
        pub fn aliases(text: Text<'_>, mutable_text: MutText<'_>, counter: Counter<'_>,
                   shared: Shared<'_>, grouped: (&str), slice: Slice<'_>, mutable_slice: MutSlice<'_>) {
            *counter = 0;
            mutable_text.make_ascii_uppercase();
            mutable_slice.fill(0);
            assert_eq!(shared.field.len(), 2);
            let _ = (text, grouped, slice);
        }

        #[test] fn recursive(value: A<u8>) { let _ = value; }
        #[test] fn variants(value: Shape) { assert!(!matches!(value, Shape::Named { .. })); }
        #[test] fn structs(value: Private<u8, 2>) { assert_eq!(value.field, [0, 0]); }
        #[test] fn options(value: Option<u8>) { assert!(value.is_none()); }
        #[test] fn errors(value: Result<u8, u8>) { assert!(value.is_ok()); }
        #[test] fn flags(value: bool) { assert!(!value); }
        #[test] fn characters(value: char) { assert!(value.is_ascii()); }
        #[test] fn strings(value: String) { assert!(value.is_empty()); }
        #[test] fn arrays(value: [u8; 2]) { assert_eq!(value, [0, 0]); }
        #[test] fn tuples(value: (u8, bool)) { assert_eq!(value, (0, false)); }
        #[test] fn floats(value: (f32, f64)) { assert!(value.0 == 0.0 && value.1 == 0.0); }
        #[test] fn maps(value: HashMap<u8, u8>) { assert!(value.is_empty()); }
        #[test] fn sets(value: HashSet<u8>) { assert!(value.is_empty()); }
        #[test] fn ordered_maps(value: BTreeMap<u8, u8>) { assert!(value.is_empty()); }
        #[test] fn ordered_sets(value: BTreeSet<u8>) { assert!(value.is_empty()); }

        #[test]
        fn all(
            value: Input, flag: bool, character: char, owned: String, text: &str,
            vector: Vec<(i8, i16, i32, i64, i128, isize)>, bytes: &mut Bytes,
            integers: [u128; 2], reference: &usize, mutable: &mut u32,
            number: Result<(f32, f64), (u16, u64)>,
            map: HashMap<u8, String>, set: HashSet<u8>,
            ordered: (BTreeMap<char, u8>, BTreeSet<u8>), node: Node,
        ) {
            if let Some(shape) = value {
                match shape {
                    Shape::None => {}
                    Shape::Tuple(item) => assert_eq!(item.field.len(), 2),
                    Shape::Named { on } => { let _ = on; }
                }
            }
            bytes.fill(0);
            *mutable = 0;
            assert!(map.keys().all(|key| map.contains_key(key)));
            assert!(set.iter().all(|key| set.contains(key)));
            assert!(node.children.len() <= 32);
            let _ = (flag, character, owned, text, vector, integers, reference, number, ordered);
        }
    }
}

mod separate;
"#,
    );
    std::fs::write(
        dir.join("package/src/separate.rs"),
        "#[derive(Debug)] struct Value(u8);\n#[test] fn separate(v: Value) { let _ = v.0; }",
    )
    .unwrap();
    for _ in 0..2 {
        checked(driver(&dir).args(["--crate-type=lib", "--test"]));
    }
    let executable = dir.join("out/demo-test");
    let listing = checked(Command::new(&executable).args(["--list", "--format=terse"]));
    let listing = String::from_utf8_lossy(&listing.stdout);
    for name in [
        "ordinary",
        "outer::inner::all",
        "separate::separate",
        "sum",
        "vector",
        "even",
    ] {
        assert!(
            listing.lines().any(|line| line == format!("{name}: test")),
            "{listing}"
        );
    }
    checked(Command::new(&executable).args(["--exact", "ordinary"]));
    checked(Command::new(&executable).args(["--exact", "separate::separate"]));
    checked(Command::new(&executable).args(["--exact", "outer::inner::aliases"]));
    checked(Command::new(&executable).args(["--exact", "outer::inner::recursive"]));
    for seed in [None, Some("27")] {
        let mut run = Command::new(&executable);
        run.args([
            "--exact",
            "outer::inner::all",
            "--test-threads=1",
            "--nocapture",
        ]);
        run.env_remove("TIDE_TEST_SEED");
        if let Some(seed) = seed {
            run.env("TIDE_TEST_SEED", seed);
        }
        let out = checked(&mut run);
        assert!(stderr(&out).contains(if seed.is_some() { "1/1" } else { "100/100" }));
    }

    let failure = |name: &str, seed: Option<&str>| {
        let mut run = Command::new(&executable);
        run.args(["--exact", name, "--test-threads=1"])
            .env_remove("TIDE_TEST_SEED")
            .stdin(Stdio::null());
        if let Some(seed) = seed {
            run.env("TIDE_TEST_SEED", seed);
        }
        let out = run.output().unwrap();
        assert!(!out.status.success(), "{out:?}");
        let output = format!("{}{}", String::from_utf8_lossy(&out.stdout), stderr(&out));
        let read = |prefix| {
            output
                .lines()
                .find_map(|line| line.trim().strip_prefix(prefix))
                .unwrap_or_else(|| panic!("missing {prefix} in {output}"))
                .to_owned()
        };
        let seed = read("seed: ");
        assert!(seed.parse::<u64>().is_ok());
        assert!(output.contains(&format!("tide test {name} --seed {seed}")));
        assert!(
            output.contains(&format!(
                "TIDE_TEST_SEED={seed} '{}' --exact {name}",
                executable.display()
            )),
            "{output}"
        );
        if name == "returned_error" || name == "nested_error" {
            assert!(!output.contains("Error:"), "{output}");
            assert_eq!(output.matches("cause: ").count(), 1, "{output}");
            assert!(
                output.contains(if name == "returned_error" {
                    "cause: \"limit\""
                } else {
                    "cause: \"nested limit\""
                }),
                "{output}"
            );
        }
        let budget = output.contains("shrink budget reached");
        (seed, read("input: "), budget)
    };
    let sum = failure("sum", None);
    assert_eq!(sum, failure("sum", None));
    assert_eq!(sum, failure("sum", Some(&sum.0)));
    let (a, b) = sum.1.split_once(", b = ").unwrap();
    let a: u16 = a.strip_prefix("a = ").unwrap().parse().unwrap();
    let b: u16 = b.parse().unwrap();
    assert_eq!(a + b, 301);
    assert_eq!(failure("vector", None).1, "xs = [100]");
    assert_eq!(failure("even", None).1, "x = Even(10)");
    assert_eq!(failure("same_failure", Some("4")).1, "x = 10");
    assert!(failure("budget", Some("4")).2);
    assert_eq!(failure("positive", Some("4")).1, "p = Positive(1)");
    assert_eq!(failure("returned_error", None).1, "x = 4");
    assert_eq!(failure("nested_error", None).1, "x = 4");
    assert_eq!(failure("wrapped", None).1, "x = Wrap(Leaf(3))");
    for family in [
        "variants",
        "structs",
        "options",
        "errors",
        "flags",
        "characters",
        "strings",
        "arrays",
        "tuples",
        "floats",
        "maps",
        "sets",
        "ordered_maps",
        "ordered_sets",
    ] {
        failure(&format!("outer::inner::{family}"), None);
    }
}

#[test]
fn property_callbacks_cannot_capture_caller_state() {
    let dir = package(
        "property-capture",
        r#"
#[tide::main]
fn main() {
    let mut partial = 0;
    std::test::run("capture", |g| {
        let _ = g.index(2);
        partial += 1;
        Ok(())
    });
}
"#,
    );
    let output = driver(&dir).stdin(Stdio::null()).output().unwrap();
    assert_eq!(output.status.code(), Some(1));
    let errors = stderr(&output);
    assert!(
        errors.contains("expected fn pointer") && errors.contains("found closure"),
        "{errors}"
    );
    assert!(!dir.join("out/demo").exists());
}

#[test]
fn property_generation_diagnoses_unsupported_fields_and_attribute_arguments() {
    for (name, source, diagnostic, line) in [
        (
            "field",
            "#[derive(Debug)] struct Unsupported {\nfield: Box<u8>\n}\n#[test] fn check(missing: Box<u8>, x: Unsupported) {}",
            "field `field` of `Unsupported` has type `tide_std::boxed::Box<u8>`, which cannot be generated",
            "demo.rs:2:",
        ),
        (
            "slice",
            "#[derive(Debug)] struct Tail {\nfield: [u8]\n}\n#[test] fn check(x: &Tail) {}",
            "field `field` of `Tail` has type `[u8]`, which cannot be generated",
            "demo.rs:2:",
        ),
        (
            "requirements",
            "use std::test::{Arbitrary, Gen};\n\
             #[derive(Debug)] struct Forever<T>(T);\n\
             impl<T> Arbitrary for Forever<T> where Forever<Vec<T>>: Arbitrary {\n\
             fn arbitrary(_: &mut Gen) -> Self { panic!(\"no finite generator\"); } }\n\
             #[test] fn check(x: Forever<u8>) {}",
            "property generation requirements nest too deeply",
            "demo.rs:",
        ),
        (
            "attr",
            "#[test(args)] fn check() {}",
            "property tests take no attribute arguments",
            "demo.rs:1:",
        ),
        (
            "admission",
            "#[derive(Debug)] struct Value(u8); #[test] fn check(v: Value) { unsafe { let _ = v; } }",
            "FX0901",
            "demo.rs:1:",
        ),
    ] {
        let dir = package(&format!("property-invalid-{name}"), source);
        let out = driver(&dir)
            .args(["--crate-type=lib", "--test"])
            .stdin(Stdio::null())
            .output()
            .unwrap();
        assert_eq!(out.status.code(), Some(1));
        let errors = stderr(&out);
        let diagnosed =
            errors.contains(diagnostic) || (name == "requirements" && errors.contains("E0275"));
        assert!(diagnosed && errors.contains(line), "{errors}");
    }
}

#[test]
fn separate_library_properties_generate_suite_types_and_library_implemented_types() {
    let suite = r#"
use tide_candidate::{Level, Marker, Opaque, Wrap};

#[derive(Debug)]
struct Reading { level: Level, raw: u8 }

#[test]
fn levels_stay_in_range(reading: Reading) {
    assert!(reading.level.get() < 4, "{}", reading.raw);
}

#[test]
fn levels_are_low(reading: Reading) {
    assert!(reading.level.get() < 3);
}

#[test]
fn unused_type_argument(value: Marker<Opaque>) { let _ = value; }

#[derive(Debug)]
struct Leaf(u8);

#[test]
fn indirect_requirement(value: Wrap<Leaf>) { let _ = value.0.0; }
"#;
    let dir = package("separate-library-properties", suite);
    let candidate = dir.join("candidate/src/lib.rs");
    std::fs::create_dir_all(candidate.parent().unwrap()).unwrap();
    std::fs::write(
        &candidate,
        r#"
use std::test::{Arbitrary, Gen};

#[derive(Debug)]
pub struct Level(u8);
impl Level {
    pub fn get(&self) -> u8 { self.0 }
}
impl Arbitrary for Level {
    fn arbitrary(g: &mut Gen) -> Self { Level(u8::try_from(g.index(4)).unwrap()) }
}

#[derive(Debug)]
pub struct Opaque(pub u8);

#[derive(Debug)]
pub struct Marker<T>(std::marker::PhantomData<T>);
impl<T> Arbitrary for Marker<T> {
    fn arbitrary(_: &mut Gen) -> Self { Self(std::marker::PhantomData) }
}

pub trait Generate: Sized { fn generate(g: &mut Gen) -> Self; }
impl<T: Arbitrary> Generate for T {
    fn generate(g: &mut Gen) -> Self { T::arbitrary(g) }
}

#[derive(Debug)]
pub struct Wrap<T>(pub T);
impl<T: Generate> Arbitrary for Wrap<T> {
    fn arbitrary(g: &mut Gen) -> Self { Self(T::generate(g)) }
}
"#,
    )
    .unwrap();
    checked(&mut paired_test_driver(&dir));
    let test = dir.join("out/demo-test");
    let run = checked(Command::new(&test).args(["--exact", "levels_stay_in_range"]));
    assert!(String::from_utf8_lossy(&run.stdout).contains("1 passed"));
    checked(Command::new(&test).args(["--exact", "unused_type_argument"]));
    let run = checked(Command::new(&test).args(["--exact", "indirect_requirement"]));
    assert!(String::from_utf8_lossy(&run.stdout).contains("1 passed"));
    let run = Command::new(&test)
        .args(["--exact", "levels_are_low"])
        .stdin(Stdio::null())
        .output()
        .unwrap();
    assert!(!run.status.success());
    let report = format!(
        "{}{}",
        String::from_utf8_lossy(&run.stdout),
        String::from_utf8_lossy(&run.stderr)
    );
    assert!(report.contains("level: Level(3)"), "{report}");

    std::fs::write(
        dir.join("package/src/demo.rs"),
        "use tide_candidate::Opaque;\n\
         #[derive(Debug)] struct Wrapper { inner: Opaque }\n\
         #[test] fn wrapped(w: Wrapper) { let _ = w.inner; }\n",
    )
    .unwrap();
    let out = paired_test_driver(&dir)
        .stdin(Stdio::null())
        .output()
        .unwrap();
    let diagnostic = stderr(&out);
    assert_eq!(out.status.code(), Some(1), "{diagnostic}");
    assert!(
        diagnostic.contains(
            "this property generates `tide_candidate::Opaque`, which the paired library defines \
             without implementing `std::test::Arbitrary`"
        ) && diagnostic.contains("demo.rs:3:"),
        "{diagnostic}"
    );
    assert!(!test.exists());
}

#[test]
fn property_synthesis_preserves_diagnostic_lines() {
    let source = "mod outer {\n\
                  mod inner {\n\
                  #[derive(Debug)] struct Private<T> { n: std::collections::BTreeMap<u8, (T, Vec<T>, Option<T>, Result<Vec<T>, (T, T)>, [T; 12])> }\n\
                  #[test] fn check(v: Private<u8>) { let _ = v.n; }\n\
                  }\n\
                  fn broken() { let _: u8 = \"wrong type\"; }\n\
                  }\n";
    let dir = package("property-diagnostic", source);
    let output = driver(&dir)
        .args(["--crate-type=lib", "--test"])
        .stdin(Stdio::null())
        .output()
        .unwrap();
    assert!(!output.status.success());
    let errors = stderr(&output);
    assert!(
        errors.contains("demo.rs:6:") && errors.contains("mismatched types"),
        "{errors}"
    );
    assert!(!errors.contains("Arbitrary"), "{errors}");
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

/// `derive(Encode, Decode)` and `std::json` (bead fx-08m) pass the driver and execute: round
/// trips of structs, enums, `Option`, `Vec`, and maps, each `#[codec]` key, missing and unknown
/// keys, malformed input, float and integer limits, and a map keyed from `Rand`.
#[test]
fn codec_derives_and_json_build_and_execute_against_production_std() {
    let source = include_str!("../../../conformance/cases/codec/json.rs");
    let dir = package("codec-json", source);
    let out = checked(&mut driver(&dir));
    assert!(out.stderr.is_empty(), "{}", stderr(&out));
    let out = checked(&mut Command::new(dir.join("out/demo")));
    assert_eq!(out.stdout, b"CODEC_RAN\n");
    checked(driver(&dir).args(["--crate-type=lib", "--test"]));
    let out = checked(&mut Command::new(dir.join("out/demo-test")));
    assert!(String::from_utf8_lossy(&out.stdout).contains("1 passed"));
}

#[test]
fn codec_recursive_newtype_returns_error_in_subprocess() {
    let source = r#"
#[derive(Decode)]
struct Link(Option<Box<Link>>);

#[tide::main]
fn main(_root: Root) {
    assert!(std::json::decode::<Link>(b"null").unwrap().0.is_none());
    let error = std::json::decode::<Link>(b"0").err().unwrap();
    assert!(error.to_string().starts_with("recursion limit exceeded"));
}
"#;
    let dir = package("codec-recursive-newtype", source);
    checked(&mut driver(&dir));
    checked(Command::new("timeout").arg("10").arg(dir.join("out/demo")));
}

/// Each case's errors are all on `line`, and one of them contains `message`.
#[test]
fn codec_derives_reject_unsupported_input_at_its_line() {
    for (name, source, message, line) in [
        (
            "variant_skip",
            "#[derive(Encode)]\nenum Bad {\n    #[codec(skip)]\n    A,\n}",
            "`skip` is not allowed here, since `encode` cannot fail",
            3,
        ),
        (
            "unknown_key",
            "#[derive(Encode)]\nstruct Bad {\n    #[codec(skip_serializing_if = \"Option::is_none\")]\n    a: Option<u8>,\n}",
            "unknown `#[codec]` key; `#[codec]` takes `rename = \"..\"`, `rename_all = \"..\"`, `default`, and `skip`",
            3,
        ),
        (
            "not_in_v0",
            "#[derive(Decode)]\n#[codec(deny_unknown_fields)]\nstruct Bad { a: u8 }",
            "`deny_unknown_fields` is not in v0",
            2,
        ),
        (
            "default_path",
            "#[derive(Decode)]\nstruct Bad {\n    #[codec(default = \"make\")]\n    a: u8,\n}",
            "`default` takes no value",
            3,
        ),
        (
            "rename_forms",
            "#[derive(Decode)]\nstruct Bad {\n    #[codec(rename(deserialize = \"b\"))]\n    a: u8,\n}",
            "expected `=`",
            3,
        ),
        (
            "field_collision",
            "#[derive(Encode)]\nstruct Bad {\n    a: u8,\n    #[codec(rename = \"a\")]\n    b: u8,\n}",
            "two fields have the JSON name `a`",
            5,
        ),
        (
            "variant_collision",
            "#[derive(Decode)]\n#[codec(rename_all = \"lowercase\")]\nenum Bad {\n    Ab,\n    AB,\n}",
            "two variants have the JSON name `ab`",
            5,
        ),
        (
            "case_name",
            "#[derive(Encode)]\n#[codec(rename_all = \"Title Case\")]\nstruct Bad { a: u8 }",
            "expected one of \"lowercase\", \"UPPERCASE\", \"PascalCase\", \"camelCase\", \"snake_case\", \"SCREAMING_SNAKE_CASE\", \"kebab-case\", \"SCREAMING-KEBAB-CASE\"",
            2,
        ),
        (
            "variant_rename_all",
            "#[derive(Encode)]\nenum Bad {\n    #[codec(rename_all = \"camelCase\")]\n    A { long_name: u8 },\n}",
            "`rename_all` is not allowed here; it goes on the struct or enum",
            3,
        ),
        (
            "container_rename",
            "#[derive(Encode)]\n#[codec(rename = \"Good\")]\nstruct Bad { a: u8 }",
            "`rename` is not allowed here; it goes on a field or a variant",
            2,
        ),
        (
            "enum_default",
            "#[derive(Decode)]\n#[codec(default)]\nenum Bad { A }",
            "`default` is not allowed here; it goes on a field or a struct",
            2,
        ),
        (
            "tuple_field",
            "#[derive(Decode)]\nstruct Bad(\n    #[codec(default)]\n    u8,\n    u8,\n);",
            "`#[codec]` is not allowed on a tuple field",
            3,
        ),
        (
            "unit_default",
            "#[derive(Decode)]\n#[codec(default)]\nstruct Bad;",
            "`default` is not allowed here; it goes on a field or a struct with named fields or an array representation",
            2,
        ),
        (
            "newtype_default",
            "#[derive(Encode)]\n#[codec(default)]\nstruct Bad(u8);",
            "`default` is not allowed here; it goes on a field or a struct with named fields or an array representation",
            2,
        ),
        (
            "duplicate_key",
            "#[derive(Encode)]\nstruct Bad {\n    #[codec(rename = \"a\")]\n    #[codec(rename = \"b\")]\n    c: u8,\n}",
            "duplicate `rename`",
            4,
        ),
        (
            "union",
            "#[derive(Encode)]\nunion Bad { a: u8 }",
            "cannot be derived for a union",
            2,
        ),
        // ROOT-4: a capability has no `Decode`, and `skip` needs the `Default` it also lacks.
        (
            "capability",
            "#[derive(Decode)]\nstruct Bad {\n    net: std::net::Net,\n}",
            "Decode",
            3,
        ),
        (
            "capability_skip",
            "#[derive(Decode)]\nstruct Bad {\n    #[codec(skip)]\n    net: std::net::Net,\n}",
            "Default",
            4,
        ),
        // Decoding is owned: a borrowed field encodes only.
        (
            "borrowed",
            "#[derive(Encode, Decode)]\nstruct Bad<'a> {\n    name: &'a str,\n}",
            "Decode",
            3,
        ),
        (
            "non_string_key",
            "use std::collections::BTreeMap;\n#[derive(Encode)]\nstruct Bad {\n    by_id: BTreeMap<u32, u8>,\n}",
            "Encode",
            4,
        ),
        // `skip` needs `Default` for `Decode` only.
        (
            "missing_default",
            "struct Plain;\n#[derive(Encode)]\nstruct Fine {\n    #[codec(skip)]\n    a: Plain,\n}\n#[derive(Decode)]\nstruct Bad {\n    #[codec(skip)]\n    a: Plain,\n}",
            "Default",
            10,
        ),
    ] {
        let dir = package(&format!("codec-negative-{name}"), source);
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
        let errors: Vec<_> = err.lines().filter(|l| l.contains(": error")).collect();
        assert!(
            errors
                .iter()
                .all(|l| l.contains(&format!("demo.rs:{line}:")))
                && errors.iter().any(|l| l.contains(message)),
            "{name}: {err}"
        );
    }
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

/// Execute the shared production/conformance file implementation through admission.
#[test]
fn real_files_build_and_run_against_production_std() {
    use std::os::unix::fs::symlink;
    let source = include_str!("../../../conformance/cases/authority/files_executable.rs");
    let dir = package("real-files", source);
    let base = dir.join("files");
    std::fs::create_dir_all(base.join("links")).unwrap();
    std::fs::create_dir_all(base.join("synced")).unwrap();
    std::fs::write(base.join("in.txt"), "safe").unwrap();
    std::fs::write(dir.join("outside"), "untouched").unwrap();
    symlink("../in.txt", base.join("links/ok")).unwrap();
    symlink("../..", base.join("links/up")).unwrap();
    symlink("/etc", base.join("links/abs")).unwrap();
    symlink("../synced", base.join("links/dir")).unwrap();
    symlink("missing-target", base.join("links/dangling")).unwrap();
    symlink("synced/created", base.join("new-link")).unwrap();
    symlink(
        format!("{}synced/long-target", "./".repeat(160)),
        base.join("long-link"),
    )
    .unwrap();
    std::fs::create_dir(base.join("deep")).unwrap();
    std::fs::write(base.join("deep/target"), "unchanged").unwrap();
    symlink("deep", base.join("alias")).unwrap();
    for n in 0..40 {
        let target = if n == 39 {
            "target".to_owned()
        } else {
            format!("l{}", n + 1)
        };
        symlink(target, base.join(format!("deep/l{n}"))).unwrap();
    }
    symlink("l0", base.join("deep/too-many")).unwrap();
    checked(&mut driver(&dir));
    let out = checked(Command::new(dir.join("out/demo")).current_dir(&dir));
    assert_eq!(out.stdout, b"FILES_RAN\n");
    assert!(out.stderr.is_empty(), "{}", stderr(&out));
    assert_eq!(std::fs::read_to_string(dir.join("outside")).unwrap(), "untouched");
}

#[test]
fn simulation_tests_find_lost_updates_and_reexecute_seeds() {
    let dir = package(
        "simulation-runner",
        include_str!("../../../conformance/cases/tasks/simulation_runner.rs"),
    );
    checked(driver(&dir).args(["--crate-type=lib", "--test"]));
    let executable = dir.join("out/demo-test");
    let run = |name: &str, seed: Option<&str>| {
        let mut command = Command::new(&executable);
        command.args(["--exact", name, "--nocapture", "--include-ignored"]);
        command
            .env_remove("TIDE_SIM_SEED")
            .env_remove("__TIDE_SIM_CHILD");
        if let Some(seed) = seed {
            command.env("TIDE_SIM_SEED", seed);
        }
        command.stdin(Stdio::null()).output().unwrap()
    };
    for (name, count) in [
        ("fixed_updates", 20),
        ("default_seeds", 100),
        ("ignored_simulation", 1),
        ("r#loop", 1),
        ("empty_range", 0),
    ] {
        let out = run(&format!("simulation::{name}"), None);
        let output = format!("{}{}", String::from_utf8_lossy(&out.stdout), stderr(&out));
        assert!(out.status.success(), "{output}");
        assert!(output.contains(&format!("{count} passed; 0 failed ({count} seeds)")), "{output}");
        assert!(!output.contains("TIDE_SIM_SEED_START"), "{output}");
        assert_eq!(output.matches("test result:").count(), 1, "{output}");
    }
    let captured = checked(Command::new(&executable).args(["--exact", "simulation::fixed_updates"]));
    let output = String::from_utf8_lossy(&captured.stdout);
    assert_eq!(output.matches("test result:").count(), 1, "{output}");
    assert!(!output.contains("TIDE_SIM_SEED_START"), "{output}");
    assert!(!output.contains("simulation simulation::fixed_updates:"), "{output}");
    assert!(captured.stderr.is_empty(), "{}", stderr(&captured));
    let selected = run("simulation::fixed_updates", Some("19"));
    let output = format!(
        "{}{}",
        String::from_utf8_lossy(&selected.stdout),
        stderr(&selected)
    );
    assert!(selected.status.success(), "{output}");
    assert!(output.contains("1 passed; 0 failed (1 seeds)"), "{output}");
    let invalid = run("simulation::fixed_updates", Some("invalid"));
    assert!(!invalid.status.success());
    assert!(stderr(&invalid).contains("TIDE_SIM_SEED must be a decimal u64"));
    let out = run("simulation::plain_simulation", None);
    assert!(out.status.success(), "{}", stderr(&out));
    assert!(String::from_utf8_lossy(&out.stdout).contains("PLAIN_SIM_RAN"));
    assert!(!String::from_utf8_lossy(&out.stdout).contains("quote="));
    assert!(out.stderr.is_empty(), "{}", stderr(&out));

    let out = run("simulation::loses_updates", None);
    let output = format!("{}{}", String::from_utf8_lossy(&out.stdout), stderr(&out));
    assert!(!out.status.success(), "{output}");
    assert!(output.contains("lost update"), "{output}");
    let failed = output
        .lines()
        .find_map(|line| line.strip_prefix("simulation failure seed="))
        .and_then(|line| line.split_whitespace().next())
        .expect(&output);
    assert!(output.contains("(20 seeds)"), "{output}");
    assert!(output.contains("failing seeds:"), "{output}");
    assert!(!output.contains("last started seed"), "{output}");
    assert!(!output.contains("TIDE_SIM_SEED_START"), "{output}");
    let captured = Command::new(&executable)
        .args(["--exact", "simulation::loses_updates"])
        .env("TIDE_SIM_SEED", failed)
        .env_remove("__TIDE_SIM_CHILD")
        .stdin(Stdio::null()).output().unwrap();
    assert!(!captured.status.success());
    assert!(captured.stderr.is_empty(), "{}", stderr(&captured));
    let captured_output = String::from_utf8_lossy(&captured.stdout);
    assert!(captured_output.contains("lost update"), "{captured_output}");
    assert!(captured_output.contains("\"kind\":\"sink\""), "{captured_output}");
    // One fixed seed in two fresh child processes must give the same trace and sink output.
    let trace = |out: Output| {
        assert!(!out.status.success());
        stderr(&out)
            .lines()
            .filter(|line| line.starts_with('{'))
            .map(str::to_owned)
            .collect::<Vec<_>>()
    };
    let first = trace(run("simulation::loses_updates", Some(failed)));
    assert!(!first.is_empty());
    assert!(first.iter().any(|line| line.contains("\"kind\":\"sched\"")));
    assert!(first.iter().any(|line| line.contains("\"kind\":\"sink\"")));
    assert_eq!(first, trace(run("simulation::loses_updates", Some(failed))));
}

/// The same file program executes on production real and simulated Fs, then compares results.
#[test]
fn simulated_files_match_real_files_and_replay() {
    let dir = package("simulated-files", include_str!("../../../conformance/cases/authority/simulated_files.rs"));
    std::fs::create_dir_all(dir.join("files")).unwrap();
    checked(&mut driver(&dir));
    let real = checked(Command::new(dir.join("out/demo")).current_dir(&dir));
    checked(driver(&dir).args(["--crate-type=lib", "--test"]));
    let test = dir.join("out/demo-test");
    let simulated = checked(Command::new(&test).args(["--exact", "simulated::equivalent_program", "--nocapture"]));
    let observation = |out: &Output| String::from_utf8_lossy(&out.stdout).lines().find(|l| l.starts_with("OBS:")).unwrap().to_owned();
    assert_eq!(observation(&real), observation(&simulated));
    for name in ["fixed_seed_trace", "setup_time_and_cancellation", "cancellation_during_latency_returns_the_operation_result"] {
        let out = checked(Command::new(&test).args(["--exact", &format!("simulated::{name}"), "--nocapture"]));
        assert!(String::from_utf8_lossy(&out.stdout).contains("1 passed; 0 failed"));
    }
}

#[test]
fn trait_dispatch_recursion_keeps_rustcs_failure_without_an_uncertain_label() {
    // The parameter dispatch in bounce cannot prove a cycle back to the blanket impl.
    let dir = package(
        "trait-dispatch-recursion",
        r#"
trait Step { fn step(self); }
impl<T> Step for T { fn step(self) { bounce(Some(self)); } }
fn bounce<T: Step>(x: T) { x.step(); }
#[tide::main]
fn main(_: Root) { bounce(0u8); }
"#,
    );
    let output = driver(&dir).output().unwrap();
    let error = stderr(&output);
    assert_eq!(output.status.code(), Some(1), "{error}");
    assert!(
        error.contains("reached the recursion limit while instantiating"),
        "{error}"
    );
    assert!(!error.contains("FX0407"), "{error}");
    assert!(!dir.join("out/demo").exists());
}

#[test]
fn synthesized_impls_are_trusted_without_trusting_adjacent_user_impls() {
    let source = r#"
mod generated {
    #[derive(Debug)] struct Bag<T> { rows: Vec<T> }
    #[test] fn check(value: Bag<u8>) { let _ = value; }
}
"#;
    let dir = package("trait-synthesis-provenance-accept", source);
    checked(driver(&dir).args(["--crate-type=lib", "--test"]));
    let dir = package(
        "trait-synthesis-provenance-reject",
        &format!(
            "{source}\npub trait Marker {{}}\nimpl<T> Marker for T where Vec<T>: Clone {{}}\n"
        ),
    );
    let output = driver(&dir)
        .args(["--crate-type=lib", "--test"])
        .output()
        .unwrap();
    let error = stderr(&output);
    assert_eq!(output.status.code(), Some(1), "{error}");
    let labels: Vec<_> = error
        .lines()
        .filter(|line| line.contains("error: FX"))
        .collect();
    assert_eq!(labels.len(), 1, "{error}");
    assert!(
        labels[0].contains("demo.rs:8:") && labels[0].contains("FX0406:"),
        "{error}"
    );
}

#[test]
fn finite_trait_dispatch_chain_keeps_rustcs_limit_failure_without_a_growth_label() {
    let mut source = String::from("pub trait K { fn step(); }\nfn f<T: K>() { T::step(); }\n");
    for n in 0..9 {
        let body = if n == 0 {
            String::new()
        } else {
            format!("f::<S{}>();", n - 1)
        };
        source.push_str(&format!(
            "pub struct S{n}; impl K for S{n} {{ fn step() {{ {body} }} }}\n"
        ));
    }
    source.push_str("pub fn entry() { f::<S8>(); }\n");
    for (limit, success) in [(4, false), (128, true)] {
        let dir = package(
            &format!("finite-trait-chain-{limit}"),
            &format!("#![recursion_limit=\"{limit}\"]\n{source}"),
        );
        let output = driver(&dir)
            .args(["--crate-type=lib", "--emit=metadata"])
            .output()
            .unwrap();
        let error = stderr(&output);
        assert_eq!(output.status.success(), success, "{error}");
        assert!(!error.contains("FX0407"), "{error}");
        if !success {
            assert!(
                error.contains("reached the recursion limit while instantiating"),
                "{error}"
            );
        }
    }
}
