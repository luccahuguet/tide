//! The benchmark pairs' equivalence check (fx-6u8m.1). Each `benchmarks/<pair>/` holds a Tide
//! program `tide.rs` and its plain-Rust twin `rust.rs`. This test builds the Tide arm through the
//! driver against production std and the Rust arm with plain rustc (`RUSTC`, else the pinned
//! toolchain), runs both, each in its own empty working directory, with no argument (the
//! default, small size) and with a small explicit one, and requires byte-identical standard
//! output, exit code 0 from both, and an empty directory afterwards. Nondeterminism stays out of the programs' output, never compared
//! loosely. The `tasks` pair also runs its Tide arm under the simulator: its test prints the
//! simulated run's standard output to standard error, which must match the Rust arm's output.
//! It checks every directory in `benchmarks/`, so adding a pair is adding a directory. Like
//! `examples.rs`, it runs reviewed programs from this repository only.
use std::path::{Path, PathBuf};
use std::process::Output;

#[path = "support/admitted.rs"]
mod admitted;
use admitted::{checked, driver, package, plain_rustc, stderr, timed};

/// The pairs whose Tide arm also has the simulated run, `tests::simulated`.
const SIMULATED: [&str; 1] = ["tasks"];

/// The explicit size each pair also runs with: small, not its default, and fewer numbers than
/// `tasks` has workers, so one worker gets none.
const EXPLICIT: &str = "3";

fn benchmarks() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../benchmarks")
}

fn scratch(name: &str) -> PathBuf {
    let dir = PathBuf::from(env!("CARGO_TARGET_TMPDIR"))
        .join("benchmarks")
        .join(name);
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

/// Builds `benchmarks/<pair>/rust.rs` with plain rustc, outside the driver.
fn rust_arm(pair: &str) -> PathBuf {
    let binary = scratch(&format!("{pair}/rust")).join(pair);
    plain_rustc(&benchmarks().join(pair).join("rust.rs"), &binary);
    binary
}

/// Runs `program` with `args` in a new empty directory, bounded by a timeout.
fn run(program: &Path, label: &str, args: &[&str]) -> Output {
    let cwd = scratch(&format!("run/{label}"));
    let out = timed(120, program)
        .args(args)
        .current_dir(&cwd)
        .output()
        .unwrap();
    let left: Vec<_> = std::fs::read_dir(&cwd)
        .unwrap()
        .map(|e| e.unwrap().path())
        .collect();
    assert!(
        left.is_empty(),
        "{label}: left files behind: {left:?}\n{}",
        shown(&out)
    );
    out
}

fn shown(output: &Output) -> String {
    format!(
        "{:?}\n--- stdout\n{}--- stderr\n{}",
        output.status,
        String::from_utf8_lossy(&output.stdout),
        stderr(output)
    )
}

/// Builds both arms of `pair` and checks that they print the same output.
fn check(pair: &str) {
    let source = std::fs::read_to_string(benchmarks().join(pair).join("tide.rs")).unwrap();
    let dir = package(&format!("benchmarks/{pair}/tide"), &source);
    checked(&mut driver(&dir));
    let tide = dir.join("out/demo");
    let rust = rust_arm(pair);
    let mut default = Vec::new();
    for args in [&[][..], &[EXPLICIT][..]] {
        let label = format!("{pair} {args:?}");
        let r = run(&rust, &format!("{pair}/rust-{}", args.len()), args);
        let t = run(&tide, &format!("{pair}/tide-{}", args.len()), args);
        assert!(
            r.status.success() && !r.stdout.is_empty(),
            "{label}: the Rust arm failed:\n{}",
            shown(&r)
        );
        assert!(
            t.status.code() == r.status.code() && t.stdout == r.stdout,
            "{label}: the arms differ\n=== Rust arm: {}\n=== Tide arm: {}",
            shown(&r),
            shown(&t)
        );
        if args.is_empty() {
            default = r.stdout;
        }
    }
    if SIMULATED.contains(&pair) {
        checked(driver(&dir).args(["--crate-type=lib", "--test"]));
        let out = run(
            &dir.join("out/demo-test"),
            &format!("{pair}/sim"),
            &["--exact", "tests::simulated", "--test-threads=1"],
        );
        assert!(
            out.status.success() && out.stderr == default,
            "{pair}: the simulated Tide run differs from the Rust arm\n=== Rust arm:\n{}\n=== simulated Tide arm: {}",
            String::from_utf8_lossy(&default),
            shown(&out)
        );
    }
}

/// The pairs `benchmarks/` must hold; more may be added beside them.
const EXPECTED: [&str; 4] = ["contracts", "files", "tasks", "tcp"];

/// The names in directory `dir`, sorted, each with whether it is a directory.
fn entries(dir: &Path) -> Vec<(String, bool)> {
    let mut entries: Vec<(String, bool)> = std::fs::read_dir(dir)
        .unwrap_or_else(|e| panic!("cannot read {}: {e}", dir.display()))
        .map(|e| {
            let e = e.unwrap();
            let name = e.file_name().to_string_lossy().into_owned();
            (name, e.file_type().unwrap().is_dir())
        })
        .collect();
    entries.sort();
    entries
}

/// The pairs in `root`, a `benchmarks/` directory. Its only file is `README.md`, and each
/// directory in it is a pair holding exactly the files `tide.rs` and `rust.rs`. Every expected
/// pair must be there, so there are never fewer than four. Anything else is an error, never
/// skipped, so no source escapes the check.
fn pairs_in(root: &Path) -> Result<Vec<String>, String> {
    let mut pairs = Vec::new();
    for (name, is_dir) in entries(root) {
        if !is_dir {
            if name != "README.md" {
                return Err(format!(
                    "benchmarks/{name}: not a pair directory or README.md"
                ));
            }
            continue;
        }
        let arms = entries(&root.join(&name));
        let want = [
            ("rust.rs".to_string(), false),
            ("tide.rs".to_string(), false),
        ];
        if arms != want {
            return Err(format!(
                "benchmarks/{name}: holds {arms:?}, not exactly the files tide.rs and rust.rs"
            ));
        }
        pairs.push(name);
    }
    for pair in EXPECTED.iter().chain(&SIMULATED) {
        if !pairs.iter().any(|p| p == pair) {
            return Err(format!("no benchmarks/{pair}"));
        }
    }
    Ok(pairs)
}

/// Checks the pairs at once, one thread each.
#[test]
fn every_pair_prints_the_same_output_in_both_arms() {
    let pairs = pairs_in(&benchmarks()).unwrap_or_else(|e| panic!("{e}"));
    std::thread::scope(|s| {
        for pair in &pairs {
            s.spawn(move || check(pair));
        }
    });
}

/// A `benchmarks/` layout holding `README.md`, both arms of the four expected pairs, and
/// `extra` (paths to add) less `less` (paths to leave out).
fn layout(case: &str, extra: &[&str], less: &[&str]) -> PathBuf {
    let root = scratch(&format!("layout/{case}"));
    let mut files = vec!["README.md".to_string()];
    for pair in EXPECTED {
        files.push(format!("{pair}/tide.rs"));
        files.push(format!("{pair}/rust.rs"));
    }
    files.extend(extra.iter().map(|f| f.to_string()));
    for file in files.iter().filter(|f| !less.contains(&f.as_str())) {
        let path = root.join(file);
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(path, "").unwrap();
    }
    root
}

#[test]
fn discovery_never_loses_a_pair_or_skips_a_source() {
    let four = ["contracts", "files", "tasks", "tcp"]
        .map(String::from)
        .to_vec();
    assert_eq!(pairs_in(&layout("whole", &[], &[])), Ok(four));
    let added = pairs_in(&layout("added", &["more/tide.rs", "more/rust.rs"], &[]));
    assert_eq!(added.map(|p| p.len()), Ok(5));
    for (case, extra, less) in [
        ("no-arm", &[][..], &["files/rust.rs"][..]),
        ("no-pair", &[], &["tcp/tide.rs", "tcp/rust.rs"]),
        (
            "no-simulated-pair",
            &[],
            &["tasks/tide.rs", "tasks/rust.rs"],
        ),
        ("stray-root-file", &["extra.rs"], &[]),
        ("stray-pair-file", &["files/other.rs"], &[]),
        ("nested-dir", &["files/sub/tide.rs"], &[]),
    ] {
        let got = pairs_in(&layout(case, extra, less));
        assert!(got.is_err(), "{case}: accepted {got:?}");
    }
}
