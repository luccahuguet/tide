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
// The rerun tool, so this suite type-checks it and tests its untimed parts (its statistics, the
// peak-RSS wrapper, `check_output`, and `close`); never its `main` or a measurement.
#[path = "../benches/pairs.rs"]
#[allow(dead_code)]
mod pairs;
use pairs::{peak_rss, stats};

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

/// The rerun tool's memory measure reports the largest process of the tree it waits for: here
/// `dd`, a child of the measured shell, filling a 64 MiB buffer.
#[test]
fn peak_rss_covers_the_measured_process_tree() {
    let measured = peak_rss::measure(std::process::Command::new("sh").args([
        "-c",
        "dd if=/dev/zero of=/dev/null bs=64M count=1 2>/dev/null; true",
    ]))
    .unwrap();
    assert!(measured.status.success(), "{}", stderr_of(&measured));
    assert!(
        measured.peak_rss_kib >= 64 * 1024,
        "peak RSS {} KiB",
        measured.peak_rss_kib
    );
}

fn stderr_of(measured: &peak_rss::Measured) -> String {
    String::from_utf8_lossy(&measured.stderr).into_owned()
}

#[test]
fn rerun_statistics_median_and_cv() {
    assert_eq!(stats::median(&[]), None);
    assert_eq!(stats::median(&[3.0, 1.0, 2.0]), Some(2.0));
    assert_eq!(stats::median(&[4.0, 1.0, 3.0, 2.0]), Some(2.5));
    assert_eq!(stats::cv(&[5.0]), None);
    assert_eq!(stats::cv(&[0.0, 0.0]), None);
    assert_eq!(stats::cv(&[2.0, 2.0, 2.0]), Some(0.0));
    // Mean 10, sample standard deviation 1.
    let cv = stats::cv(&[9.0, 10.0, 11.0]).unwrap();
    assert!((cv - 0.1).abs() < 1e-12, "{cv}");
    assert!(stats::stable(&[1.0, 1.01]));
    assert!(!stats::stable(&[1.0, 1.1]));
    assert!(!stats::stable(&[1.0]));
}

#[test]
fn rounds_rise_while_any_series_varies_and_stop_at_the_cap() {
    let steady = vec![1.0, 1.0, 1.0, 1.0, 1.0];
    let noisy = vec![1.0, 1.2, 1.0, 1.2, 1.0];
    assert_eq!(
        stats::more_rounds(&[steady.clone(), steady.clone()], 5, 5, 20),
        0
    );
    assert_eq!(
        stats::more_rounds(&[steady.clone(), noisy.clone()], 5, 5, 20),
        5
    );
    assert_eq!(stats::more_rounds(&[noisy.clone()], 18, 5, 20), 2);
    // At the cap it stops though the series still varies: the caller marks it unstable.
    assert_eq!(stats::more_rounds(&[noisy], 20, 5, 20), 0);
}

#[test]
fn rounds_interleave_the_arms() {
    assert_eq!(
        stats::interleaved(2, 0..2),
        [(0, 0), (0, 1), (1, 0), (1, 1)],
        "two arms run A B A B"
    );
    assert_eq!(stats::interleaved(3, 5..6), [(5, 0), (5, 1), (5, 2)]);
    assert!(stats::interleaved(3, 2..2).is_empty());
}

#[test]
fn json_strings_escape_quotes_backslashes_and_controls() {
    assert_eq!(
        stats::json_string("rustc \"1\"\\\nhost:\tx\u{1}"),
        r#""rustc \"1\"\\\nhost:\tx\u0001""#
    );
}

/// A run whose output differs from the pair's first run leaves an `error` record naming it, with
/// its exit status and both outputs' tails, and the records get no `end`; a complete
/// measurement gets one.
#[test]
fn an_output_mismatch_is_recorded_and_leaves_the_dataset_without_end() {
    let run = |config, stdout: &'static [u8]| pairs::Run {
        pair: "tcp",
        config,
        round: 3,
        exit: "0",
        stdout,
    };
    let mut records = Vec::new();
    let mut expected = None;
    for config in ["tide-debug", "rust-debug"] {
        pairs::check_output(&mut records, run(config, b"same\n"), &mut expected).unwrap();
    }
    assert!(records.is_empty());
    let failed = pairs::check_output(&mut records, run("rust-release", b"other\n"), &mut expected);
    assert!(pairs::close(&mut records, failed).is_err());
    let text = String::from_utf8(records).unwrap();
    assert_eq!(text.lines().count(), 1, "{text}");
    for field in [
        r#"{"record":"error","error":"output mismatch""#,
        r#""pair":"tcp","config":"rust-release","round":3,"exit":0"#,
        r#""expected_tail":"same\n","actual_tail":"other\n""#,
    ] {
        assert!(text.contains(field), "no {field} in {text}");
    }
    let mut complete = Vec::new();
    pairs::close(&mut complete, Ok(())).unwrap();
    let complete = String::from_utf8(complete).unwrap();
    assert!(complete.starts_with(r#"{"record":"end","#), "{complete}");
}

/// The rebuild's one-line edit adds a statement as the first line of `main`'s body, in every
/// arm of every pair, and refuses a source without exactly one place to put it.
#[test]
fn the_rebuild_edit_opens_main_with_one_new_line() {
    let source = "#[tide::main]\nfn main(root: Root) -> Result<(), E> {\n    go(root)\n}\n";
    assert_eq!(
        pairs::edited(source, 7).unwrap(),
        "#[tide::main]\nfn main(root: Root) -> Result<(), E> {\n    let _edit: u64 = 7;\n    go(root)\n}\n"
    );
    assert!(pairs::edited("fn other() {}\n", 0).is_err());
    assert!(pairs::edited("fn main() {\n}\nfn main() {\n}\n", 0).is_err());
    for pair in EXPECTED {
        for arm in ["tide.rs", "rust.rs"] {
            let source = std::fs::read_to_string(benchmarks().join(pair).join(arm)).unwrap();
            let edited = pairs::edited(&source, 0).unwrap_or_else(|e| panic!("{pair}/{arm}: {e}"));
            assert_eq!(edited.lines().count(), source.lines().count() + 1, "{pair}/{arm}");
        }
    }
}

/// The binary size reads an ELF file's loadable segments, which leave out its debug information,
/// and nothing from a file that is not a whole 64-bit ELF file.
#[test]
fn binary_size_counts_the_loadable_segments() {
    let exe = std::fs::read(std::env::current_exe().unwrap()).unwrap();
    let len = exe.len() as u64;
    let (size, loaded) = pairs::binary_size(&exe[..64 * 1024], len).unwrap();
    assert_eq!(size, len);
    // This test binary has debug information, which no segment loads.
    assert!(loaded > 0 && loaded < size, "{loaded} of {size}");
    // A synthetic header with two loadable segments and one other, each with a memory size
    // above its file size: only the loadable segments' file sizes count.
    let mut elf = vec![0u8; 64 + 3 * 56];
    elf[..6].copy_from_slice(b"\x7fELF\x02\x01");
    elf[0x20..0x28].copy_from_slice(&64u64.to_le_bytes()); // e_phoff
    elf[0x36..0x38].copy_from_slice(&56u16.to_le_bytes()); // e_phentsize
    elf[0x38..0x3a].copy_from_slice(&3u16.to_le_bytes()); // e_phnum
    let segments = [(1u32, 1000u64, 1500u64), (4, 70_000, 80_000), (1, 234, 4096)];
    for (i, (kind, file, memory)) in segments.into_iter().enumerate() {
        let header = 64 + i * 56;
        elf[header..header + 4].copy_from_slice(&kind.to_le_bytes()); // p_type
        elf[header + 0x20..header + 0x28].copy_from_slice(&file.to_le_bytes()); // p_filesz
        elf[header + 0x28..header + 0x30].copy_from_slice(&memory.to_le_bytes()); // p_memsz
    }
    assert_eq!(pairs::binary_size(&elf, 9999), Some((9999, 1234)));
    assert_eq!(pairs::binary_size(b"#!/bin/sh\n", 10), None);
    // Program headers past what was read.
    assert_eq!(pairs::binary_size(&exe[..64], len), None);
}

/// A test build's run prints the program's output among the test harness's lines: the check
/// takes the window equal to the reference output, and the whole output when there is none, so
/// that the comparison fails.
#[test]
fn a_test_builds_output_is_found_among_the_harness_lines() {
    let expected = b"limit: 3\nprimes: 1\n";
    let framed = b"\nrunning 1 test\ntest bench_test::timed ... limit: 3\nprimes: 1\nok\n";
    assert_eq!(pairs::within(framed, Some(expected)), expected);
    let other = b"\nrunning 1 test\nlimit: 3\nprimes: 2\nok\n";
    assert_eq!(pairs::within(other, Some(expected)), other);
    assert_eq!(pairs::within(framed, None), framed);
}

/// A run of the run phase alone, with no build phase to keep its programs, builds them
/// unmeasured and completes (Codex r2 re-check 2, item 4). It runs the tool in quick mode on
/// `contracts`, with the pinned compiler in the stable arm's place and the suite's std for both
/// profiles, so the workspace suite needs neither rustup's stable toolchain nor a release std.
#[test]
fn a_run_phase_alone_builds_its_programs_and_completes() {
    let out = scratch("phase-run");
    let pinned = pairs::sysroot_rustc(
        std::process::Command::new(std::env::var_os("RUSTC").unwrap_or("rustc".into()))
            .current_dir(env!("CARGO_MANIFEST_DIR")),
    )
    .unwrap();
    let std = admitted::std_dir().to_path_buf();
    let mut bench = pairs::Bench::open(
        pairs::Options {
            quick: true,
            out: Some(out.clone()),
            pairs: vec!["contracts".into()],
            phases: vec![pairs::Phase::Run],
        },
        pairs::Toolchains {
            pinned: pinned.clone(),
            stable: pinned,
            std_dev: std.clone(),
            std_release: std,
        },
    )
    .unwrap();
    bench.run().unwrap();
    let records = std::fs::read_to_string(out.join("records.jsonl")).unwrap();
    let count = |kind: &str| records.matches(&format!(r#"{{"record":"{kind}","#)).count();
    assert_eq!(
        (count("build"), count("run"), count("summary"), count("end")),
        (0, 6, 6, 1),
        "{records}"
    );
}
