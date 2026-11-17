//! Tide's first running demo (fx-nkv): a lost-update bug in a small concurrent service, found by
//! a seed and replayed identically, the fix passing a fixed seed set, and the trust diff of a
//! patch that widens the service's public interface to carry `Net`. `demo/README.md` is the guide.
//!
//! `cargo run -p tide-demo` builds the admission driver and Tide's std with the pinned toolchain
//! into `$CARGO_TARGET_DIR` (default: `<repo>/target`) and works in its `tide-demo/` directory,
//! which each run replaces. It exits 0 when every step shows what it should, 1 when one does not,
//! and 2 on a setup error. `tests/demo.rs` runs it in the workspace suite.
//!
//! Each check reads what the driver gives as data where it gives it: exit statuses, the
//! `tide-replay/2` manifests and their traces (with the driver's own decoder), Cargo's artifact
//! messages, and the `--tide-diff` report's lines. Two facts exist only as the test harness's
//! text, and are read from it: each failing seed's cause line, which is where a failed
//! determinism self-check shows, and the `replay reproduced:` marker, without which a replay
//! that exits 101 may have timed out or been killed instead.

use std::collections::BTreeSet;
use std::ffi::OsStr;
use std::fmt;
use std::fs::{self, File};
use std::path::{Path, PathBuf};
use std::process::{Command, ExitStatus, Stdio};

// The driver's private wire format, read with its own strict decoder, as `tide-admit` does.
use std::collections;
#[allow(dead_code)]
#[path = "../../tide-std/src/codec.rs"]
mod __codec;
#[allow(dead_code)]
#[path = "../../tide-std/src/json.rs"]
mod json;
#[path = "../../tide-std/src/replay.rs"]
mod replay;

use __codec::{Decode, Decoder};
use replay::Manifest;

/// The suite's simulation test.
const TEST: &str = "every_view_is_counted";
/// The outcome signature of the suite's `assert_eq!(hits, views, "lost hits")` failing, as
/// tide-std's harness writes it (`Evaluation::signature` in `crates/tide-std/src/test/runner.rs`).
const LOST_HITS: &str = "panic: Some(\"assertion `left == right` failed: lost hits\\n";

/// Why a run stopped.
#[derive(Debug)]
pub enum Failure {
    /// The demo could not be set up: a build, a patch, or a file operation failed. Exit 2.
    Setup(String),
    /// A step did not show what it should. Exit 1. `log` is the run that showed it, if any.
    Step {
        step: u8,
        message: String,
        log: Option<PathBuf>,
    },
}

impl Failure {
    /// The demo's exit status for this failure.
    pub fn exit_code(&self) -> u8 {
        match self {
            Failure::Setup(_) => 2,
            Failure::Step { .. } => 1,
        }
    }

    /// Prints the failure, after the log of the run that showed it, to standard error.
    pub fn report(&self) {
        if let Failure::Step { log: Some(log), .. } = self {
            eprint!(
                "{}",
                String::from_utf8_lossy(&fs::read(log).unwrap_or_default())
            );
        }
        eprintln!("{self}");
    }
}

impl fmt::Display for Failure {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Failure::Setup(message) => write!(f, "demo: {message}"),
            Failure::Step { step, message, log } => {
                write!(f, "FAIL: step {step}: {message}")?;
                match log {
                    Some(log) => write!(f, " (log: {})", log.display()),
                    None => Ok(()),
                }
            }
        }
    }
}

fn setup(message: impl fmt::Display) -> Failure {
    Failure::Setup(message.to_string())
}

/// One revision of the service: `demo/service/src/hits.rs`, the fixed revision, or that file
/// with one of `demo/patches/` applied.
#[derive(Clone, Debug)]
pub enum Revision {
    Service,
    Patched(&'static str),
}

impl Revision {
    fn label(&self) -> String {
        match self {
            Revision::Service => "demo/service".into(),
            Revision::Patched(patch) => format!("demo/patches/{patch}"),
        }
    }
}

/// What a run uses. `Demo::default()` is the demo; the workspace test swaps a revision to show
/// that the demo catches it.
#[derive(Clone, Debug)]
pub struct Demo {
    pub fixed: Revision,
    pub buggy: Revision,
    pub widened: Revision,
    /// The work directory, relative to the target directory (or absolute). Each run replaces
    /// it.
    pub work: PathBuf,
    /// The target directory the driver and std are built into. `None` is `$CARGO_TARGET_DIR`, a
    /// relative value taken from the working directory as Cargo takes it, else `<repo>/target`.
    pub target: Option<PathBuf>,
}

impl Default for Demo {
    fn default() -> Self {
        Demo {
            fixed: Revision::Service,
            buggy: Revision::Patched("lost-update.patch"),
            widened: Revision::Patched("metrics-report.patch"),
            work: "tide-demo".into(),
            target: None,
        }
    }
}

fn step(title: impl fmt::Display) {
    println!("\n== {title}");
}

impl Demo {
    /// Runs the demo's steps in order, stopping at the first that fails.
    pub fn run(&self) -> Result<(), Failure> {
        // Physical paths: the driver rejects a package path that traverses a directory symlink.
        let root = fs::canonicalize(Path::new(env!("CARGO_MANIFEST_DIR")).join("../.."))
            .map_err(|e| setup(format!("cannot find the repository: {e}")))?;
        let target = self.target.clone().unwrap_or_else(|| {
            std::env::var_os("CARGO_TARGET_DIR").map_or_else(|| root.join("target"), PathBuf::from)
        });
        let target = fs::create_dir_all(&target)
            .and_then(|()| fs::canonicalize(&target))
            .map_err(|e| {
                setup(format!(
                    "cannot create the target directory {}: {e}",
                    target.display()
                ))
            })?;
        let work = target.join(&self.work);
        let work = match fs::remove_dir_all(&work) {
            Err(e) if e.kind() != std::io::ErrorKind::NotFound => Err(e),
            _ => fs::create_dir_all(work.join("logs")),
        }
        .and_then(|()| fs::canonicalize(&work))
        .map_err(|e| {
            setup(format!(
                "cannot replace the work directory {}: {e}",
                work.display()
            ))
        })?;
        self.steps(Run::build(root, target, work)?)
    }

    fn steps(&self, run: Run) -> Result<(), Failure> {
        let fixed = run.revision("fixed", &self.fixed)?;
        let buggy = run.revision("buggy", &self.buggy)?;
        let widened = run.revision("widened", &self.widened)?;
        let seed = run.find_seed(&buggy, &self.buggy)?;
        run.record_and_replay(&buggy, seed)?;
        run.fixed(&fixed, &self.fixed, seed)?;
        run.authority_diff(&fixed, &widened, &self.widened)?;
        let shown = run.work.strip_prefix(&run.root).unwrap_or(&run.work);
        step(format!(
            "Done: every step showed what it should. Logs and artifacts: {}",
            shown.display()
        ));
        Ok(())
    }
}

/// A Cargo message, as `--message-format=json` writes one per line; other fields are ignored.
struct CargoMessage {
    reason: String,
    filenames: Option<Vec<String>>,
    executable: Option<String>,
}

/// A trace record, as `trace.ndjson` holds one per line; other fields are ignored.
struct TraceRecord {
    kind: String,
    bytes: Option<Vec<u8>>,
}

macro_rules! decodable {
    ($name:ident { $($field:ident: $ty:ty),* }) => {
        impl Decode for $name {
            fn __decode(input: &mut Decoder<'_>) -> Result<Self, json::Error> {
                const NAMES: &[&str] = &[$(stringify!($field)),*];
                $(let mut $field: Option<$ty> = None;)*
                input.object_begin()?;
                while let Some(index) = input.next_field(NAMES)? {
                    match NAMES.get(index).copied() {
                        $(Some(stringify!($field)) => input.field(&mut $field, stringify!($field))?,)*
                        _ => input.skip_value()?,
                    }
                }
                Ok(Self { $($field: input.required($field, stringify!($field))?),* })
            }
        }
    };
}
decodable!(CargoMessage { reason: String, filenames: Option<Vec<String>>, executable: Option<String> });
decodable!(TraceRecord { kind: String, bytes: Option<Vec<u8>> });

/// The toolchain `rust-toolchain.toml` pins: its `channel`.
fn pinned_toolchain(root: &Path) -> Result<String, Failure> {
    let path = root.join("rust-toolchain.toml");
    let text = fs::read_to_string(&path).map_err(|e| setup(format!("{}: {e}", path.display())))?;
    text.lines()
        .filter_map(|line| {
            line.trim()
                .strip_prefix("channel")?
                .trim()
                .strip_prefix('=')
        })
        .find_map(|value| {
            Some(
                value
                    .trim()
                    .strip_prefix('"')?
                    .strip_suffix('"')?
                    .to_owned(),
            )
        })
        .filter(|channel| !channel.is_empty())
        .ok_or_else(|| setup(format!("{} names no channel", path.display())))
}

/// `program` from the pinned toolchain, whatever toolchain built or runs the demo: `rustup run`
/// selects it for the program and for every rustc and cargo the program starts.
fn pinned(root: &Path, toolchain: &str, program: &str) -> Command {
    let mut command = Command::new("rustup");
    command
        .current_dir(root)
        .env_remove("RUSTUP_TOOLCHAIN")
        .args(["run", toolchain, program])
        .stdin(Stdio::null());
    command
}

/// The artifacts Cargo reported building, for one `cargo build` that succeeded.
fn cargo_build(
    root: &Path,
    toolchain: &str,
    target: &Path,
    package: &str,
) -> Result<Vec<CargoMessage>, Failure> {
    // Diagnostics render to standard error; standard output carries one JSON message per line.
    let out = pinned(root, toolchain, "cargo")
        .args(["build", "--quiet", "--locked", "-p", package])
        .arg("--message-format=json-render-diagnostics")
        .arg("--target-dir")
        .arg(target)
        .stderr(Stdio::inherit())
        .output()
        .map_err(|e| setup(format!("cannot run rustup: {e}")))?;
    if !out.status.success() {
        return Err(setup(format!("cannot build {package} ({})", out.status)));
    }
    out.stdout
        .split(|&b| b == b'\n')
        .filter(|line| !line.is_empty())
        .map(json::decode::<CargoMessage>)
        .filter(|message| {
            message
                .as_ref()
                .map_or(true, |m| m.reason == "compiler-artifact")
        })
        .collect::<Result<_, _>>()
        .map_err(|e| setup(format!("cargo wrote a message that is not JSON: {e}")))
}

/// The first line of `command`'s standard output, or `unknown`.
fn first_line(command: &mut Command) -> String {
    command
        .stdin(Stdio::null())
        .stderr(Stdio::null())
        .output()
        .ok()
        .filter(|out| out.status.success())
        .and_then(|out| String::from_utf8(out.stdout).ok())
        .and_then(|text| text.lines().next().map(str::to_owned))
        .unwrap_or_else(|| "unknown".into())
}

/// A manifest a run recorded, with the trace it exported beside it.
struct Recording {
    path: PathBuf,
    manifest: Manifest,
    trace: Vec<u8>,
}

#[derive(Debug, PartialEq)]
enum Outcome {
    Passed,
    LostHits,
    Other(String),
}

impl Recording {
    fn outcome(&self) -> Outcome {
        match self.manifest.outcome.as_str() {
            "passed" => Outcome::Passed,
            outcome if outcome.starts_with(LOST_HITS) => Outcome::LostHits,
            outcome => Outcome::Other(outcome.into()),
        }
    }

    fn seed(&self) -> u64 {
        // `Manifest::validate` checked that the seed is a canonical decimal u64.
        self.manifest.seed.parse().unwrap()
    }

    /// What the test printed: the bytes of the trace's sink records, in order.
    fn output(&self) -> Result<String, String> {
        let mut output = Vec::new();
        for line in self
            .trace
            .split(|&b| b == b'\n')
            .filter(|line| !line.is_empty())
        {
            let record: TraceRecord =
                json::decode(line).map_err(|e| format!("trace record: {e}"))?;
            if record.kind == "sink" {
                output.extend(record.bytes.ok_or("a sink record without bytes")?);
            }
        }
        String::from_utf8(output).map_err(|_| "the output is not UTF-8".into())
    }
}

/// The run's paths and the built driver and std.
struct Run {
    root: PathBuf,
    target: PathBuf,
    work: PathBuf,
    driver: PathBuf,
    std: PathBuf,
    deps: Vec<PathBuf>,
}

impl Run {
    /// Step 0: builds the driver and std with the pinned toolchain.
    fn build(root: PathBuf, target: PathBuf, work: PathBuf) -> Result<Run, Failure> {
        step("0. Toolchain and build");
        let git = || {
            let mut git = Command::new("git");
            git.arg("-C").arg(&root);
            git
        };
        let status = git()
            .args(["status", "--porcelain"])
            .stdin(Stdio::null())
            .stderr(Stdio::null())
            .output();
        let dirty = match status {
            Ok(out) if out.status.success() && !out.stdout.is_empty() => {
                " (with uncommitted changes)"
            }
            _ => "",
        };
        println!(
            "revision:  {}{dirty}",
            first_line(git().args(["rev-parse", "HEAD"]))
        );
        // The pinned compiler's identity; without it, nothing can be built.
        let toolchain = pinned_toolchain(&root)?;
        let version = pinned(&root, &toolchain, "rustc")
            .arg("--version")
            .output()
            .map_err(|e| setup(format!("cannot run rustup: {e}")))?;
        let version = String::from_utf8(version.stdout)
            .ok()
            .filter(|_| version.status.success())
            .and_then(|text| text.lines().next().map(str::to_owned))
            .ok_or_else(|| setup(format!("the pinned toolchain {toolchain} is not installed")))?;
        println!("toolchain: {version}");
        let driver = cargo_build(&root, &toolchain, &target, "tide-admit")?
            .into_iter()
            .filter_map(|message| message.executable)
            .find(|path| Path::new(path).file_name() == Some(OsStr::new("tide-admit")))
            .ok_or_else(|| setup("cargo reported no tide-admit executable"))?;
        // Tide's std and every library it loads, each library's directory a dependency directory.
        let mut libraries = Vec::new();
        for message in cargo_build(&root, &toolchain, &target, "tide_std")? {
            libraries.extend(
                message
                    .filenames
                    .into_iter()
                    .flatten()
                    .map(PathBuf::from)
                    .filter(|file| {
                        matches!(
                            file.extension().and_then(OsStr::to_str),
                            Some("rlib" | "so")
                        )
                    }),
            );
        }
        let std = libraries
            .iter()
            .find(|lib| lib.file_name() == Some(OsStr::new("libtide_std.rlib")))
            .cloned()
            .ok_or_else(|| setup("cargo reported no libtide_std.rlib"))?;
        let mut deps: Vec<PathBuf> = Vec::new();
        for dir in libraries.iter().filter_map(|lib| lib.parent()) {
            if !deps.iter().any(|dep| dep == dir) {
                deps.push(dir.to_owned());
            }
        }
        println!("driver:    {driver}");
        println!("std:       {}", std.display());
        Ok(Run {
            root,
            target,
            work,
            driver: driver.into(),
            std,
            deps,
        })
    }

    /// The service's source for `revision`. A patch that no longer applies is a setup error, so
    /// the revisions cannot drift from the service.
    fn revision(&self, name: &str, revision: &Revision) -> Result<PathBuf, Failure> {
        let service = self.root.join("demo/service/src/hits.rs");
        let Revision::Patched(patch) = revision else {
            return Ok(service);
        };
        let dir = self.work.join("revisions").join(name);
        fs::create_dir_all(dir.join("src"))
            .and_then(|()| fs::copy(&service, dir.join("src/hits.rs")))
            .map_err(|e| setup(format!("cannot copy the service to {}: {e}", dir.display())))?;
        let patch = self.root.join("demo/patches").join(patch);
        // The ceiling keeps git from treating the target directory as part of the repository.
        let status = Command::new("git")
            .current_dir(&dir)
            .env("GIT_CEILING_DIRECTORIES", self.work.join("revisions"))
            .arg("apply")
            .arg(&patch)
            .stdin(Stdio::null())
            .status()
            .map_err(|e| setup(format!("cannot run git: {e}")))?;
        if !status.success() {
            return Err(setup(format!(
                "{} does not apply to demo/service/src/hits.rs",
                patch.display()
            )));
        }
        Ok(dir.join("src/hits.rs"))
    }

    /// Runs `command` with its output in `logs/<log>.log`, and returns its status and the log.
    fn logged(&self, command: &mut Command, log: &str) -> Result<(ExitStatus, PathBuf), Failure> {
        let path = self.work.join("logs").join(format!("{log}.log"));
        let file = File::create(&path).map_err(|e| setup(format!("{}: {e}", path.display())))?;
        let copy = file
            .try_clone()
            .map_err(|e| setup(format!("{}: {e}", path.display())))?;
        let status = command
            .stdin(Stdio::null())
            .stdout(file)
            .stderr(copy)
            .status()
            .map_err(|e| setup(format!("cannot run the driver: {e}")))?;
        Ok((status, path))
    }

    /// The driver, with Tide's std and its libraries.
    fn driver(&self) -> Command {
        let mut command = Command::new(&self.driver);
        command.arg("--tide-std").arg(&self.std);
        for dep in &self.deps {
            command.arg("--tide-dep-dir").arg(dep);
        }
        command
    }

    /// The simulation suite, admitted and linked with `library` as `tide_candidate`, and run in
    /// the driver's sandboxed child with `run`'s arguments.
    fn suite(
        &self,
        library: &Path,
        out: &str,
        log: &str,
        run: &[&OsStr],
    ) -> Result<(ExitStatus, PathBuf), Failure> {
        let mut command = self.driver();
        command
            .args([
                "--test",
                "--crate-type=lib",
                "--emit=link",
                "--crate-name=hits_sim",
            ])
            .arg("--out-dir")
            .arg(self.out(out))
            .arg("--tide-test-library")
            .arg(library)
            .arg(self.root.join("demo/tests/src/hits_sim.rs"))
            .args(run);
        self.logged(&mut command, log)
    }

    fn out(&self, name: &str) -> PathBuf {
        self.work.join("out").join(name)
    }

    /// Every manifest a run directory holds, each validated, by path.
    fn recordings(&self, out: &str) -> Result<Vec<Recording>, String> {
        let sim = self.out(out).join("target/sim");
        let mut paths = Vec::new();
        if sim.exists() {
            for entry in fs::read_dir(&sim).map_err(|e| format!("{}: {e}", sim.display()))? {
                let path = entry
                    .map_err(|e| e.to_string())?
                    .path()
                    .join("run.manifest.json");
                if path.exists() {
                    paths.push(path);
                }
            }
        }
        paths.sort();
        paths
            .into_iter()
            .map(|path| {
                let manifest: Manifest = replay::read(&path)?;
                manifest
                    .validate()
                    .map_err(|e| format!("{}: {e}", path.display()))?;
                let trace = path.with_file_name("trace.ndjson");
                let trace = fs::read(&trace).map_err(|e| format!("{}: {e}", trace.display()))?;
                if trace != manifest.trace.as_bytes() {
                    return Err(format!(
                        "{}: the exported trace differs from the manifest's",
                        path.display()
                    ));
                }
                Ok(Recording {
                    path,
                    manifest,
                    trace,
                })
            })
            .collect()
    }

    /// Step 1: the buggy suite over its seed set fails on some seeds and passes on others, and
    /// every failing seed lost a hit.
    fn find_seed(&self, buggy: &Path, revision: &Revision) -> Result<u64, Failure> {
        step(format!(
            "1. Find a seed that exposes the bug (buggy revision, {})",
            revision.label()
        ));
        let (status, log) = self.suite(
            buggy,
            "search",
            "search",
            &[OsStr::new("--tide-run-test"), OsStr::new(TEST)],
        )?;
        let fail = |message: String| Failure::Step {
            step: 1,
            message,
            log: Some(log.clone()),
        };
        if status.code() != Some(101) {
            return Err(fail(format!(
                "the buggy suite did not fail as a test ({status})"
            )));
        }
        let recordings = self.recordings("search").map_err(fail)?;
        let cases = seed_set(&recordings).map_err(fail)?;
        let mut failing = BTreeSet::new();
        for recording in &recordings {
            match recording.outcome() {
                Outcome::Passed => {}
                Outcome::LostHits => {
                    failing.insert(recording.seed());
                }
                Outcome::Other(outcome) => {
                    return Err(fail(format!(
                        "seed {} failed for another reason: {outcome}",
                        recording.seed()
                    )));
                }
            }
        }
        // The harness reports each failing seed with its cause, which the recording does not
        // hold when the determinism self-check failed.
        let text = read_log(&log)?;
        let marker = format!(" test={TEST}");
        let mut reports: Vec<(u64, Option<&str>)> = Vec::new();
        for line in text.lines() {
            if let Some(seed) = line
                .strip_prefix("simulation failure seed=")
                .and_then(|rest| rest.strip_suffix(&marker))
            {
                let seed = seed
                    .parse()
                    .map_err(|_| fail(format!("unreadable failure report: {line}")))?;
                reports.push((seed, None));
            } else if let (Some(cause), Some((_, slot @ None))) =
                (line.strip_prefix("  cause: "), reports.last_mut())
            {
                *slot = Some(cause);
            }
        }
        let reported: BTreeSet<u64> = reports.iter().map(|(seed, _)| *seed).collect();
        let lost = reports
            .iter()
            .filter(|(_, cause)| cause.is_some_and(|c| c.ends_with("failed: lost hits")))
            .count();
        if reported != failing || reports.len() != failing.len() || lost != failing.len() {
            return Err(fail(format!(
                "the recordings show failing seeds {failing:?}, but the harness reported {reports:?}"
            )));
        }
        let Some(&seed) = failing.first() else {
            return Err(fail("no failing seed".into()));
        };
        if failing.len() == cases {
            return Err(fail(
                "every seed failed, so this failure does not depend on the schedule".into(),
            ));
        }
        println!(
            "{}",
            summary(&text).ok_or_else(|| fail("no summary line".into()))?
        );
        let listed: Vec<String> = failing.iter().map(u64::to_string).collect();
        println!(
            "failing seeds: {} (each cause: lost hits)",
            listed.join(" ")
        );
        println!("same code and input on every seed; only the schedule differs");
        println!("first failing seed: {seed}");
        Ok(seed)
    }

    /// Step 2: the seed, run alone, records the lost hit, and two replays of the recording each
    /// reproduce it with a byte-identical trace.
    fn record_and_replay(&self, buggy: &Path, seed: u64) -> Result<(), Failure> {
        step(format!(
            "2. Record seed {seed}, then replay the recording twice"
        ));
        let seed_text = seed.to_string();
        let run = [
            OsStr::new("--tide-run-test"),
            OsStr::new(TEST),
            OsStr::new("--seed"),
            OsStr::new(&seed_text),
        ];
        let (status, log) = self.suite(buggy, "record", "record", &run)?;
        let fail = |message: String, log: &Path| Failure::Step {
            step: 2,
            message,
            log: Some(log.to_owned()),
        };
        let mut recordings = self.recordings("record").map_err(|e| fail(e, &log))?;
        let recording = match (status.code(), recordings.pop()) {
            (Some(101), Some(recording)) if recordings.is_empty() => recording,
            _ => {
                return Err(fail(
                    format!("seed {seed} did not fail again on a fresh run ({status})"),
                    &log,
                ));
            }
        };
        let cause = read_log(&log)?
            .lines()
            .find_map(|line| line.strip_prefix("  cause: ").map(str::to_owned))
            .filter(|cause| cause.ends_with("failed: lost hits"));
        let (true, Some(cause)) = (
            recording.seed() == seed
                && recording.manifest.test == TEST
                && recording.outcome() == Outcome::LostHits,
            cause,
        ) else {
            return Err(fail(
                format!(
                    "seed {seed} failed for another reason: {}",
                    recording.manifest.outcome
                ),
                &log,
            ));
        };
        println!("cause: {cause}");
        let mut runs = vec![("recording", recording)];
        for (n, name) in [(1, "replay-1"), (2, "replay-2")] {
            let before: BTreeSet<PathBuf> = self
                .recordings("record")
                .map_err(|e| fail(e, &log))?
                .into_iter()
                .map(|r| r.path)
                .collect();
            let replay = [OsStr::new("--replay"), runs[0].1.path.as_os_str()];
            let (status, log) = self.suite(buggy, "record", name, &replay)?;
            // A reproduced failing test exits 101, as the recording did; the marker tells it from
            // a replay that timed out or was killed, which also exits 101.
            let marker = format!("replay reproduced: test={TEST} seed={seed} ");
            if status.code() != Some(101)
                || !read_log(&log)?
                    .lines()
                    .any(|line| line.starts_with(&marker))
            {
                return Err(fail(
                    format!("replay {n} did not reproduce the recording ({status})"),
                    &log,
                ));
            }
            let mut added: Vec<Recording> = self
                .recordings("record")
                .map_err(|e| fail(e, &log))?
                .into_iter()
                .filter(|r| !before.contains(&r.path))
                .collect();
            match (added.pop(), added.is_empty()) {
                (Some(recording), true) => runs.push((name, recording)),
                _ => {
                    return Err(fail(
                        format!("replay {n} wrote no single new manifest"),
                        &log,
                    ));
                }
            }
        }
        println!("                 events  event trace       output trace");
        for (name, run) in &runs {
            let output = run.output().map_err(|e| fail(e, &log))?;
            let events = run
                .trace
                .split(|&b| b == b'\n')
                .filter(|line| !line.is_empty())
                .count();
            println!(
                "{name:<16} {events:>6}  {}  {}",
                &replay::digest(&run.trace)[..16],
                &replay::digest(output.as_bytes())[..16]
            );
        }
        for (name, run) in &runs[1..] {
            if run.trace != runs[0].1.trace {
                return Err(fail(
                    format!("{name}'s trace differs from the recording"),
                    &log,
                ));
            }
        }
        let output = runs[0].1.output().map_err(|e| fail(e, &log))?;
        println!("output: {}", output.trim_end());
        if !lost_a_hit(&output) {
            return Err(fail("the recorded output shows no lost hit".into(), &log));
        }
        println!("both replays reproduce the recording: identical event and output traces");
        let path = &runs[0].1.path;
        println!(
            "recording: {}",
            path.strip_prefix(&self.target).unwrap_or(path).display()
        );
        Ok(())
    }

    /// Step 3: the fixed revision passes the whole seed set and the failing seed alone.
    fn fixed(&self, fixed: &Path, revision: &Revision, seed: u64) -> Result<(), Failure> {
        step(format!(
            "3. Run the fixed revision ({}) over the fixed seed set",
            revision.label()
        ));
        let fail = |message: String, log: &Path| Failure::Step {
            step: 3,
            message,
            log: Some(log.to_owned()),
        };
        let (status, log) = self.suite(
            fixed,
            "fixed",
            "fixed",
            &[OsStr::new("--tide-run-test"), OsStr::new(TEST)],
        )?;
        let recordings = self.recordings("fixed").map_err(|e| fail(e, &log))?;
        let cases = seed_set(&recordings).map_err(|e| fail(e, &log))?;
        if status.code() != Some(0) || recordings.iter().any(|r| r.outcome() != Outcome::Passed) {
            return Err(fail(
                format!("the fixed revision failed its seed set ({status})"),
                &log,
            ));
        }
        println!(
            "seed set: the suite's #[sim_test(seeds = {cases})], seeds 0 to {}, the set step 1 searched",
            cases - 1
        );
        println!(
            "{}",
            summary(&read_log(&log)?).ok_or_else(|| fail("no summary line".into(), &log))?
        );
        let seed_text = seed.to_string();
        let run = [
            OsStr::new("--tide-run-test"),
            OsStr::new(TEST),
            OsStr::new("--seed"),
            OsStr::new(&seed_text),
        ];
        let (status, log) = self.suite(fixed, "fixed-seed", "fixed-seed", &run)?;
        let recordings = self.recordings("fixed-seed").map_err(|e| fail(e, &log))?;
        if status.code() != Some(0)
            || recordings.len() != 1
            || recordings[0].outcome() != Outcome::Passed
        {
            return Err(fail(
                format!("the fixed revision failed seed {seed} ({status})"),
                &log,
            ));
        }
        println!("seed {seed} on its own: passed");
        Ok(())
    }

    /// Step 4: the trust diff from the fixed revision to the widened one lists `hits::serve` as
    /// widened by a `metrics` path to `net`.
    fn authority_diff(&self, base: &Path, head: &Path, revision: &Revision) -> Result<(), Failure> {
        step(format!("4. Authority diff of {}", revision.label()));
        let fail = |message: String, log: Option<PathBuf>| Failure::Step {
            step: 4,
            message,
            log,
        };
        if let Revision::Patched(patch) = revision {
            let path = self.root.join("demo/patches").join(patch);
            let text =
                fs::read_to_string(&path).map_err(|e| setup(format!("{}: {e}", path.display())))?;
            text.lines()
                .filter(|line| line.starts_with("-pub fn ") || line.starts_with("+pub fn "))
                .for_each(|line| println!("{line}"));
        }
        for (side, source) in [("base", base), ("head", head)] {
            let mut command = self.driver();
            command
                .args(["--crate-type=lib", "--crate-name=hits", "--out-dir"])
                .arg(self.out(&format!("interface-{side}")))
                .arg(source);
            let (status, log) = self.logged(&mut command, &format!("interface-{side}"))?;
            if !status.success() {
                return Err(fail(
                    format!("the {side} revision was not admitted ({status})"),
                    Some(log),
                ));
            }
        }
        let report = Command::new(&self.driver)
            .arg("--tide-diff")
            .arg(self.out("interface-base/hits.interface.json"))
            .arg(self.out("interface-head/hits.interface.json"))
            .stdin(Stdio::null())
            .stderr(Stdio::inherit())
            .output()
            .map_err(|e| setup(format!("cannot run the driver: {e}")))?;
        let text = String::from_utf8_lossy(&report.stdout);
        print!("{text}");
        if !report.status.success() {
            return Err(fail(
                format!("the trust diff failed ({})", report.status),
                None,
            ));
        }
        // The item's line, then its indented path lines up to the next item.
        let mut lines = text
            .lines()
            .skip_while(|line| *line != "  ~ hits::serve (function) widened");
        let widened = lines.next().is_some()
            && lines
                .take_while(|line| line.starts_with("      "))
                .any(|line| line == "      + metrics: net");
        if !widened {
            return Err(fail(
                "the trust diff does not show hits::serve widened with metrics: net".into(),
                None,
            ));
        }
        Ok(())
    }
}

/// The size of a seed set, when `recordings` are one evaluation of each of its seeds, 0 to
/// `cases - 1`, as the test's recorded configuration gives `cases`.
fn seed_set(recordings: &[Recording]) -> Result<usize, String> {
    let cases: usize = recordings
        .first()
        .and_then(|r| r.manifest.config.get("cases")?.parse().ok())
        .ok_or("the run recorded no evaluations")?;
    let seeds: BTreeSet<u64> = recordings.iter().map(Recording::seed).collect();
    if recordings.len() != cases || seeds != (0..cases as u64).collect() {
        return Err(format!(
            "the run recorded {} evaluations, not one of each seed 0 to {}",
            recordings.len(),
            cases.saturating_sub(1)
        ));
    }
    Ok(cases)
}

fn read_log(log: &Path) -> Result<String, Failure> {
    fs::read(log)
        .map(|bytes| String::from_utf8_lossy(&bytes).into_owned())
        .map_err(|e| setup(format!("{}: {e}", log.display())))
}

/// The harness's summary of a simulation test's seed set.
fn summary(log: &str) -> Option<&str> {
    log.lines()
        .find(|line| line.starts_with(&format!("simulation {TEST}: ")))
}

/// Whether the suite's output line, `views served: V, hits recorded: H`, shows fewer hits.
fn lost_a_hit(output: &str) -> bool {
    output.lines().any(|line| {
        let counts = line
            .strip_prefix("views served: ")
            .and_then(|rest| rest.split_once(", hits recorded: "))
            .and_then(|(views, hits)| {
                Some((views.parse::<u64>().ok()?, hits.parse::<u64>().ok()?))
            });
        counts.is_some_and(|(views, hits)| hits < views)
    })
}
