//! The examples runner (fx-rpsb.1): every `examples/<name>/` holds a `README.md` and one or more
//! variants, each a single-file Tide crate `<variant>.rs` whose `//@` lines declare what it
//! must do. This test builds each variant through the driver against production std and checks
//! every declared outcome, so adding an example is adding a directory. Like `build.rs`, it runs
//! reviewed programs from this repository only; this is not the RUN-3 sandbox.
//!
//! The declarations, one per line:
//!
//! - `//@ expect: runs`, then one `//@ stdout: <line>` for each line the program must print:
//!   built as a binary, run with no arguments, it exits 0 with exactly that standard output.
//! - `//@ expect: tests pass`: built as a test executable, its run finishes (exit code 0), at
//!   least one test passes, and none fails.
//! - `//@ expect: test fails <name>`, then one or more `//@ reports: <text>`: built as a test
//!   executable, the test whose path is `<name>` (or ends in `::<name>`) fails, and the
//!   output libtest captured for that test contains each `<text>`. The run must finish (its
//!   summary line, exit code 101); a variant's failing tests are exactly those it declares. The
//!   test generated from a function's contract goes by that function's path, and a
//!   `#[should_panic]` test by its own. A name that matches more than one test is an error.
//! - `//@ expect: rejected <TD code> [<code>...]`: admitted as a library, the driver exits 1,
//!   every error it reports carries one of the codes (rustc's `aborting due to` summary
//!   aside), and each code appears. Further codes name the rustc errors the rejected
//!   construct itself causes, such as `E0425` for an ambient function Tide's std lacks.
//!
//! A line that looks like a mistyped declaration (indented, or `// @`, `///@`) is an error.
use std::path::{Path, PathBuf};
use std::process::{Command, Output, Stdio};

#[path = "support/admitted.rs"]
mod admitted;
use admitted::{checked, driver, package, stderr};

#[derive(Debug, PartialEq)]
enum Outcome {
    Runs { stdout: Vec<String> },
    TestsPass,
    TestFails { test: String, reports: Vec<String> },
    Rejected { codes: Vec<String> },
}

/// Reads a variant's `//@` lines. A missing or malformed declaration is an error.
fn declared(source: &str) -> Result<Vec<Outcome>, String> {
    let mut outcomes = Vec::new();
    for (number, line) in source.lines().enumerate() {
        let at = format!("line {}: `{line}`", number + 1);
        let comment = line.trim_start();
        let Some(rest) = line.strip_prefix("//@") else {
            if comment.starts_with("//")
                && comment
                    .trim_start_matches('/')
                    .trim_start()
                    .starts_with('@')
            {
                return Err(format!(
                    "{at}: a mistyped declaration; write `//@ <key>: <value>` at the start of the line"
                ));
            }
            continue;
        };
        let (key, value) = rest
            .trim_start()
            .split_once(':')
            .ok_or_else(|| format!("{at}: expected `//@ <key>: <value>`"))?;
        let value = value.strip_prefix(' ').unwrap_or(value);
        match (key, outcomes.last_mut()) {
            ("expect", _) => outcomes.push(match value.split_once(' ') {
                _ if value == "runs" => Outcome::Runs { stdout: Vec::new() },
                _ if value == "tests pass" => Outcome::TestsPass,
                Some(("rejected", codes)) if error_codes(codes).is_some() => Outcome::Rejected {
                    codes: error_codes(codes).unwrap(),
                },
                _ => match value.strip_prefix("test fails ") {
                    Some(test) if !test.is_empty() && !test.contains(char::is_whitespace) => {
                        Outcome::TestFails {
                            test: test.into(),
                            reports: Vec::new(),
                        }
                    }
                    _ => {
                        return Err(format!(
                            "{at}: expected `runs`, `tests pass`, `test fails <name>`, or \
                             `rejected TD<4 digits> [E<4 digits>...]`"
                        ));
                    }
                },
            }),
            ("stdout", Some(Outcome::Runs { stdout })) => stdout.push(value.into()),
            ("reports", Some(Outcome::TestFails { reports, .. })) if !value.is_empty() => {
                reports.push(value.into())
            }
            ("stdout" | "reports", _) => {
                return Err(format!(
                    "{at}: `stdout` follows `expect: runs`, and a non-empty `reports` follows \
                     `expect: test fails`"
                ));
            }
            _ => return Err(format!("{at}: unknown key `{key}`")),
        }
    }
    if outcomes.is_empty() {
        return Err("declares no expected outcome (`//@ expect: ...`)".into());
    }
    for outcome in &outcomes {
        match outcome {
            Outcome::Runs { stdout } if stdout.is_empty() => {
                return Err("`expect: runs` needs at least one `//@ stdout:` line".into());
            }
            Outcome::TestFails { test, reports } if reports.is_empty() => {
                return Err(format!(
                    "`expect: test fails {test}` needs at least one `//@ reports:` line"
                ));
            }
            Outcome::Rejected { .. } if outcomes.len() > 1 => {
                return Err("a rejected variant declares no other outcome".into());
            }
            _ => {}
        }
    }
    let runs = outcomes
        .iter()
        .filter(|o| matches!(o, Outcome::Runs { .. }))
        .count();
    if runs > 1 {
        return Err("`expect: runs` appears at most once".into());
    }
    let tests = outcomes
        .iter()
        .filter(|o| matches!(o, Outcome::TestsPass))
        .count();
    let failures = outcomes
        .iter()
        .filter(|o| matches!(o, Outcome::TestFails { .. }))
        .count();
    if tests > 1 || (tests == 1 && failures > 0) {
        return Err("`tests pass` appears once, and not beside `test fails`".into());
    }
    Ok(outcomes)
}

fn timed(seconds: u32, program: &Path) -> Command {
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

/// Whether libtest's test path `actual` is the declared `name`. A generated contract test,
/// `<module>::__tide_contract_tests::<function>_<index>`, is named by `<module>::<function>`.
fn names(actual: &str, name: &str) -> bool {
    let generated = actual.rsplit_once("::").and_then(|(module, test)| {
        let module = module.strip_suffix("__tide_contract_tests")?;
        if !(module.is_empty() || module.ends_with("::")) {
            return None;
        }
        let (function, index) = test.rsplit_once('_')?;
        (!index.is_empty() && index.bytes().all(|b| b.is_ascii_digit()))
            .then(|| format!("{module}{function}"))
    });
    let actual = generated.as_deref().unwrap_or(actual);
    actual == name || actual.ends_with(&format!("::{name}"))
}

fn text(output: &Output) -> String {
    format!(
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        stderr(output)
    )
}

/// Reads `rejected`'s codes: a `TD` code, then any rustc `E` codes, each four digits.
fn error_codes(codes: &str) -> Option<Vec<String>> {
    let codes: Vec<String> = codes.split_whitespace().map(String::from).collect();
    let four = |code: &str, prefix: &str| {
        code.strip_prefix(prefix)
            .is_some_and(|d| d.len() == 4 && d.bytes().all(|b| b.is_ascii_digit()))
    };
    (codes.first().is_some_and(|c| four(c, "TD"))
        && codes[1..].iter().all(|c| four(c, "TD") || four(c, "E")))
    .then_some(codes)
}

/// Checks a `rejected` outcome against the driver's exit code and its `--error-format=short`
/// diagnostics.
/// Every error must carry a declared code, apart from rustc's `aborting due to` summary, and
/// every declared code must appear: a variant also broken for another reason is not rejected.
fn admission(codes: &[String], code: Option<i32>, stderr: &str) -> Result<(), String> {
    if code != Some(1) {
        return Err(format!("expected the driver to exit 1 with {codes:?}"));
    }
    let mut seen = Vec::new();
    for line in stderr.lines() {
        // `path:line:col: error[E0425]: ..`, `path:line:col: error: TD0501: ..`, `error: ..`.
        let Some(error) = (if line.starts_with("error") {
            Some(line)
        } else {
            line.find(": error").map(|at| &line[at + 2..])
        }) else {
            continue;
        };
        if error.starts_with("error: aborting due to ") {
            continue;
        }
        let found = error
            .strip_prefix("error[")
            .and_then(|rest| rest.split_once("]:"))
            .or_else(|| error.strip_prefix("error: ")?.split_once(':'))
            .map(|(code, _)| code)
            .filter(|code| codes.iter().any(|c| c == code));
        match found {
            Some(code) => seen.push(code),
            None => return Err(format!("an undeclared error: `{line}`")),
        }
    }
    match codes.iter().find(|c| !seen.contains(&c.as_str())) {
        Some(missing) => Err(format!("no `{missing}` error")),
        None => Ok(()),
    }
}

/// Checks one variant against its declared outcomes; panics with the reason on a mismatch.
fn check(label: &str, source: &str, outcomes: &[Outcome]) {
    let dir = package(&format!("examples/{label}"), source);
    let mut tests: Option<Output> = None;
    for outcome in outcomes {
        match outcome {
            Outcome::Rejected { codes } => {
                let out = driver(&dir)
                    .args(["--crate-type=lib", "--emit=metadata"])
                    .stdin(Stdio::null())
                    .output()
                    .unwrap();
                if let Err(e) = admission(codes, out.status.code(), &stderr(&out)) {
                    panic!("{label}: {e}; got {:?}:\n{}", out.status, stderr(&out));
                }
            }
            Outcome::Runs { stdout } => {
                checked(&mut driver(&dir));
                let out = timed(120, &dir.join("out/demo")).output().unwrap();
                let expected: String = stdout.iter().map(|line| format!("{line}\n")).collect();
                assert!(
                    out.status.success() && String::from_utf8_lossy(&out.stdout) == expected,
                    "{label}: expected a successful run printing\n{expected}got {:?}:\n{}",
                    out.status,
                    text(&out)
                );
            }
            Outcome::TestsPass | Outcome::TestFails { .. } => {
                let out = tests.get_or_insert_with(|| {
                    checked(driver(&dir).args(["--crate-type=lib", "--test"]));
                    timed(300, &dir.join("out/demo-test"))
                        .arg("--test-threads=1")
                        .output()
                        .unwrap()
                });
                let stdout = String::from_utf8_lossy(&out.stdout);
                if let Err(e) = tested(outcome, outcomes, out.status.code(), &stdout) {
                    panic!("{label}: {e}; got {:?}:\n{}", out.status, text(out));
                }
            }
        }
    }
}

/// A finished libtest run, read from the pinned libtest's pretty format
/// (`library/test/src/formatters/pretty.rs`).
struct Suite<'a> {
    /// Each test's name, without a ` - should panic` mode, and its result.
    results: Vec<(&'a str, &'a str)>,
    /// Each failing test's name and the output libtest captured for it.
    captured: Vec<(&'a str, String)>,
}

/// Reads a test executable's run. A run that did not finish, with its summary line and
/// libtest's exit code for that summary (0 or 101), is an error: a timeout or a signal may have
/// cut it short before later tests ran.
fn suite(code: Option<i32>, stdout: &str) -> Result<Suite<'_>, String> {
    // libtest prints a `#[should_panic]` test as `test <name> - should panic ... <result>`.
    fn name(test: &str) -> &str {
        test.strip_suffix(" - should panic").unwrap_or(test)
    }
    let summary = stdout
        .lines()
        .rev()
        .find_map(|line| line.strip_prefix("test result: "))
        .ok_or("the test run did not finish: no `test result:` line")?;
    let finished = match summary.split_once('.') {
        Some(("ok", _)) => Some(0),
        Some(("FAILED", _)) => Some(101),
        _ => None,
    };
    if finished.is_none() || code != finished {
        return Err(format!(
            "the test run did not finish normally: exit code {code:?} after `test result: {summary}`"
        ));
    }
    let results = stdout
        .lines()
        .filter_map(|line| line.strip_prefix("test ")?.rsplit_once(" ... "))
        .map(|(test, result)| (name(test), result))
        .collect();
    let mut captured: Vec<(&str, String)> = Vec::new();
    let mut open = false;
    for line in stdout.lines() {
        if let Some(test) = line
            .strip_prefix("---- ")
            .and_then(|l| l.strip_suffix(" stdout ----"))
        {
            captured.push((name(test), String::new()));
            open = true;
        } else if line == "failures:" {
            open = false;
        } else if open {
            let (_, text) = captured.last_mut().unwrap();
            text.push_str(line);
            text.push('\n');
        }
    }
    Ok(Suite { results, captured })
}

/// Checks a test outcome against a test executable's exit code and standard output.
fn tested(
    outcome: &Outcome,
    outcomes: &[Outcome],
    code: Option<i32>,
    stdout: &str,
) -> Result<(), String> {
    let suite = suite(code, stdout)?;
    let failed: Vec<&str> = suite
        .results
        .iter()
        .filter(|(_, r)| *r == "FAILED")
        .map(|(n, _)| *n)
        .collect();
    let Outcome::TestFails { test, reports } = outcome else {
        return if failed.is_empty() && suite.results.iter().any(|(_, r)| *r == "ok") {
            Ok(())
        } else {
            Err("expected at least one test and every test to pass".into())
        };
    };
    let matching: Vec<&str> = suite
        .results
        .iter()
        .map(|(n, _)| *n)
        .filter(|n| names(n, test))
        .collect();
    if matching.len() > 1 {
        return Err(format!("`{test}` names more than one test: {matching:?}"));
    }
    let Some(name) = failed.iter().find(|name| names(name, test)) else {
        return Err(format!("expected test `{test}` to fail"));
    };
    let captured = suite
        .captured
        .iter()
        .find(|(n, _)| n == name)
        .map_or("", |(_, text)| text.as_str());
    for report in reports {
        if !captured.contains(report.as_str()) {
            return Err(format!(
                "the failure of `{name}` does not report `{report}` in its own output"
            ));
        }
    }
    let declared: Vec<&str> = outcomes
        .iter()
        .filter_map(|o| match o {
            Outcome::TestFails { test, .. } => Some(test.as_str()),
            _ => None,
        })
        .collect();
    let extra: Vec<&&str> = failed
        .iter()
        .filter(|name| !declared.iter().any(|t| names(name, t)))
        .collect();
    if !extra.is_empty() {
        return Err(format!("undeclared failures {extra:?}"));
    }
    Ok(())
}

fn examples() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../examples")
}

fn sorted(dir: &Path) -> Vec<PathBuf> {
    let mut paths: Vec<PathBuf> = std::fs::read_dir(dir)
        .unwrap_or_else(|e| panic!("cannot read {}: {e}", dir.display()))
        .map(|entry| entry.unwrap().path())
        .collect();
    paths.sort();
    paths
}

#[test]
fn examples_meet_their_declared_outcomes() {
    let mut checked_variants = 0;
    for example in sorted(&examples()) {
        let name = example.file_name().unwrap().to_string_lossy().into_owned();
        assert!(
            example.is_dir(),
            "examples/{name}: an example is a directory"
        );
        assert!(
            example.join("README.md").is_file(),
            "examples/{name}: no README.md"
        );
        let variants: Vec<PathBuf> = sorted(&example)
            .into_iter()
            .filter(|path| path.extension().is_some_and(|e| e == "rs"))
            .collect();
        assert!(!variants.is_empty(), "examples/{name}: no `<variant>.rs`");
        for variant in variants {
            let label = format!("{name}/{}", variant.file_stem().unwrap().to_string_lossy());
            let source = std::fs::read_to_string(&variant).unwrap();
            let outcomes = declared(&source).unwrap_or_else(|e| panic!("examples/{label}.rs: {e}"));
            eprintln!("examples/{label}.rs: {outcomes:?}");
            check(&label, &source, &outcomes);
            checked_variants += 1;
        }
    }
    assert!(checked_variants > 0, "examples/ holds no variant");
}

#[test]
fn declarations_are_read_strictly() {
    assert_eq!(
        declared("//@ expect: runs\n//@ stdout: a\n//@ stdout:\n//@ expect: tests pass\n"),
        Ok(vec![
            Outcome::Runs {
                stdout: vec!["a".into(), "".into()]
            },
            Outcome::TestsPass,
        ])
    );
    assert_eq!(
        declared("//@ expect: test fails m::t\n//@ reports: seed: 3\n"),
        Ok(vec![Outcome::TestFails {
            test: "m::t".into(),
            reports: vec!["seed: 3".into()]
        }])
    );
    assert!(names(
        "__tide_contract_tests::insert_sorted_3",
        "insert_sorted"
    ));
    assert!(names(
        "m::__tide_contract_tests::insert_sorted_3",
        "m::insert_sorted"
    ));
    assert!(names("tests::loads", "loads") && names("tests::loads", "tests::loads"));
    assert!(!names("tests::reloads", "loads") && !names("__tide_contract_tests::f_x", "f"));
    for (source, error) in [
        ("fn main() {}\n", "declares no expected outcome"),
        ("//@ expect: runs\n", "needs at least one `//@ stdout:`"),
        (
            "//@ expect: test fails t\n",
            "needs at least one `//@ reports:`",
        ),
        ("//@ stdout: a\n", "`stdout` follows `expect: runs`"),
        ("//@ expect: rejected TD05\n", "expected `runs`"),
        (
            "//@ expect: runs\n// @expect: tests pass\n",
            "mistyped declaration",
        ),
        ("  //@ expect: runs\n", "mistyped declaration"),
        ("///@ expect: runs\n", "mistyped declaration"),
        ("//@ expect: rejected E0425 TD0501\n", "expected `runs`"),
        ("//@ expect: rejected TD0501 X0425\n", "expected `runs`"),
        (
            "//@ expect: rejected TD0501\n//@ expect: tests pass\n",
            "declares no other outcome",
        ),
        (
            "//@ expect: tests pass\n//@ expect: test fails t\n//@ reports: x\n",
            "not beside",
        ),
        ("//@ expects: runs\n", "unknown key `expects`"),
        (
            "//@ expect: runs\n//@ stdout: a\n//@ expect: runs\n//@ stdout: b\n",
            "at most once",
        ),
        ("//@ expect runs\n", "expected `//@ <key>: <value>`"),
    ] {
        let got = declared(source).unwrap_err();
        assert!(got.contains(error), "{source:?}: {got}");
    }
}

fn fails(test: &str, report: &str) -> Outcome {
    Outcome::TestFails {
        test: test.into(),
        reports: vec![report.into()],
    }
}

/// A completed run in which `a` fails with `a_says` and `b` with `b_says`.
fn two_failures(a: &str, b: &str) -> String {
    format!(
        "\nrunning 2 tests\ntest a ... FAILED\ntest b ... FAILED\n\nfailures:\n\n\
         ---- a stdout ----\nthread 'a' panicked at demo.rs:1:1:\n{a}\n\n\
         ---- b stdout ----\nthread 'b' panicked at demo.rs:2:1:\n{b}\n\n\
         failures:\n    a\n    b\n\n\
         test result: FAILED. 0 passed; 2 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s\n\n"
    )
}

#[test]
fn an_interrupted_test_run_never_meets_a_declared_failure() {
    let outcome = fails("a", "boom");
    // `timeout` stopped the suite after `a` failed (exit 124), or a signal did (no code).
    let interrupted =
        "\nrunning 2 tests\nthread 'a' panicked at demo.rs:1:1:\nboom\ntest a ... FAILED\n";
    for code in [Some(124), None] {
        assert!(
            tested(&outcome, &[fails("a", "boom")], code, interrupted).is_err(),
            "{code:?}"
        );
    }
    let finished = two_failures("boom", "boom");
    let both = [fails("a", "boom"), fails("b", "boom")];
    assert_eq!(tested(&both[0], &both, Some(101), &finished), Ok(()));
    for code in [Some(124), None, Some(0)] {
        assert!(
            tested(&both[0], &both, code, &finished).is_err(),
            "{code:?}"
        );
    }
}

#[test]
fn each_declared_failure_is_checked_against_its_own_test_output() {
    let both = [fails("a", "expected cause"), fails("b", "expected cause")];
    let output = two_failures("unrelated error", "expected cause");
    assert_eq!(tested(&both[1], &both, Some(101), &output), Ok(()));
    let error = tested(&both[0], &both, Some(101), &output).unwrap_err();
    assert!(
        error.contains("does not report `expected cause`"),
        "{error}"
    );
}

#[test]
fn a_failing_should_panic_test_goes_by_its_name() {
    let outcome = [fails("t", "panic did not contain expected string")];
    let output = "\nrunning 1 test\ntest t - should panic ... FAILED\n\nfailures:\n\n\
        ---- t stdout ----\nthread 't' panicked at demo.rs:3:1:\nother\n\
        note: panic did not contain expected string\n      panic message: \"other\"\n expected substring: \"wanted\"\n\n\
        failures:\n    t\n\n\
        test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s\n\n";
    assert_eq!(tested(&outcome[0], &outcome, Some(101), output), Ok(()));
}

#[test]
fn a_rejected_variant_fails_only_with_its_declared_codes() {
    let codes = |codes: &str| error_codes(codes).unwrap();
    let ambient = "demo.rs:19:21: error[E0425]: cannot find function `read_to_string` in module `std::fs`\n\
        demo.rs:19:12: error: TD0501: `std::fs::read_to_string` is not admitted\n\
        error: aborting due to 2 previous errors\n";
    assert_eq!(admission(&codes("TD0501 E0425"), Some(1), ambient), Ok(()));
    // The declared code beside an error nobody declared: a broken example, not a rejection.
    for (declared, stderr) in [
        ("TD0501", ambient),
        (
            "TD0511",
            "demo.rs:27:1: error: TD0511: a `#[pure]` function's parameter types must be `Pure`\n\
             demo.rs:28:57: error[E0599]: no method named `now_ms` found for reference `&Clock`\n\
             error: aborting due to 2 previous errors\n",
        ),
        (
            "TD0511",
            "demo.rs:27:1: error: TD0511: a `#[pure]` function's parameter types must be `Pure`\n\
             demo.rs:30:9: error: TD0501: `std::time::Instant::now` is not admitted\n\
             error: aborting due to 2 previous errors\n",
        ),
    ] {
        let error = admission(&codes(declared), Some(1), stderr).unwrap_err();
        assert!(error.contains("undeclared"), "{declared}: {error}");
    }
    let only = "demo.rs:27:1: error: TD0511: a `#[pure]` function's parameter types must be `Pure`\n\
        warning: unused variable: `x`\nerror: aborting due to 1 previous error\n";
    assert_eq!(admission(&codes("TD0511"), Some(1), only), Ok(()));
    // Every declared code must appear, and the driver must exit 1.
    assert!(admission(&codes("TD0511 E0599"), Some(1), only).is_err());
    assert!(admission(&codes("TD0511"), Some(101), only).is_err());
    assert!(
        admission(
            &codes("TD0511"),
            Some(1),
            "error: aborting due to 1 previous error\n"
        )
        .is_err()
    );
}

#[test]
fn a_declared_name_that_matches_two_tests_is_an_error() {
    let outcome = [fails("insert_sorted", "boom")];
    let output = "\nrunning 2 tests\ntest __tide_contract_tests::insert_sorted_3 ... ok\n\
        test insert_sorted ... FAILED\n\nfailures:\n\n---- insert_sorted stdout ----\nboom\n\n\
        failures:\n    insert_sorted\n\n\
        test result: FAILED. 1 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s\n\n";
    let error = tested(&outcome[0], &outcome, Some(101), output).unwrap_err();
    assert!(error.contains("more than one test"), "{error}");
}
