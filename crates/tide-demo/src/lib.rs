//! Tide's first running demo (fx-nkv): a lost-update bug in a small concurrent service, found by
//! a seed and replayed identically, the fix passing a fixed seed set, the trust diff of a patch
//! that widens the service's public interface to carry `Net`, and, when gdb is available, the
//! failing seed's replay under rust-gdb. `demo/README.md` is the guide.
//!
//! `cargo run -p tide-demo` builds the admission driver and Tide's std with the pinned toolchain
//! (through rustup, or, when `PATH` has no rustup, with the rustc and cargo on `PATH` once both
//! match the pinned toolchain's recorded identity) into `$CARGO_TARGET_DIR` (default: `<repo>/target`) and works in
//! its `tide-demo/` directory, which each run replaces. It exits 0 when every step shows what it
//! should, 1 when one does not, and 2 on a setup error. `tests/demo.rs` runs it in the workspace
//! suite.
//!
//! Each check reads what the driver gives as data where it gives it: exit statuses, the
//! `tide-replay/2` manifests and their traces (with the driver's own decoder), Cargo's artifact
//! messages, and the `--tide-diff` report's lines. Two facts exist only as the test harness's
//! text, and are read from it: each failing seed's cause line, which is where a failed
//! determinism self-check shows, and the `replay reproduced:` marker, without which a replay
//! that exits 101 may have timed out or been killed instead.

use std::collections::BTreeSet;
use std::ffi::{OsStr, OsString};
use std::fmt;
use std::fs::{self, File};
use std::io::{self, Read};
use std::os::unix::process::CommandExt;
use std::path::{Path, PathBuf};
use std::process::{Command, ExitStatus, Stdio};
use std::sync::mpsc::{self, RecvTimeoutError};
use std::time::{Duration, Instant};

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
        let recording = run.record_and_replay(&buggy, seed)?;
        run.fixed(&fixed, &self.fixed, seed)?;
        run.authority_diff(&fixed, &widened, &self.widened)?;
        run.debug(&buggy, seed, &recording)?;
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

/// The toolchain `rust-toolchain.toml` pins, as its `rustc -vV` and `cargo -vV` identify it: what
/// a run without rustup checks the programs it would use against. The workspace suite's
/// `the_recorded_identity_is_the_pinned_toolchain` compares it with rustup's toolchain for the
/// pinned channel, so moving the pin without it fails the suite.
pub const PINNED: Pin = Pin {
    channel: "nightly-2026-08-09",
    rustc: &[
        ("release", "1.99.0-nightly"),
        ("commit-hash", "771916f9028e7fe56d2685f2c4f698de5d7d6a45"),
    ],
    cargo: &[
        ("release", "1.99.0-nightly"),
        ("commit-hash", "b07e5a08651da80108eb65d7dc769ee04589f9bf"),
    ],
};

/// A pinned toolchain's channel, and the `key: value` lines its rustc's and cargo's `-vV` print.
pub struct Pin {
    pub channel: &'static str,
    pub rustc: &'static [(&'static str, &'static str)],
    pub cargo: &'static [(&'static str, &'static str)],
}

/// How the demo runs the pinned toolchain's programs.
enum Toolchain {
    /// `rustup run <channel>`, which selects the pin for the program and for every rustc and
    /// cargo the program starts, whatever toolchain built or runs the demo.
    Rustup(String),
    /// When `PATH` has no rustup, as in the flake's dev shell: this rustc and cargo, as absolute
    /// paths, which `check_path_toolchain` found to be the pinned ones. Every program the demo
    /// starts gets `RUSTC` set to that rustc, which Cargo takes over its other compiler settings
    /// (`CARGO_BUILD_RUSTC`, `build.rustc`).
    Path { rustc: OsString, cargo: OsString },
}

/// `program` from the pinned toolchain.
fn pinned(root: &Path, toolchain: &Toolchain, program: &str) -> Command {
    let mut command = match toolchain {
        Toolchain::Rustup(channel) => {
            let mut command = Command::new("rustup");
            command
                .env_remove("RUSTUP_TOOLCHAIN")
                .args(["run", channel, program]);
            command
        }
        Toolchain::Path { rustc, cargo } => {
            let mut command = match program {
                "cargo" => Command::new(cargo),
                _ => Command::new(program),
            };
            command.env("RUSTC", rustc);
            command
        }
    };
    command.current_dir(root).stdin(Stdio::null());
    command
}

/// Checks, for a run without rustup, that `rustc` and `cargo` are the toolchain `channel` names,
/// by the identity `PINNED` records, and returns rustc's `-V` line.
pub fn check_path_toolchain(channel: &str, rustc: &OsStr, cargo: &OsStr) -> Result<String, String> {
    if channel != PINNED.channel {
        return Err(format!(
            "there is no rustup, and rust-toolchain.toml pins {channel}, but tide-demo records \
             the identity of {} (`PINNED` in crates/tide-demo/src/lib.rs), so it cannot check \
             the toolchain on PATH; install rustup",
            PINNED.channel
        ));
    }
    let mut version = String::new();
    for (tool, program, fields) in [
        ("rustc", rustc, PINNED.rustc),
        ("cargo", cargo, PINNED.cargo),
    ] {
        let shown = Path::new(program).display();
        let out = Command::new(program)
            .arg("-vV")
            .stdin(Stdio::null())
            .stderr(Stdio::inherit())
            .output()
            .map_err(|e| format!("there is no rustup, and {shown} cannot run: {e}"))?;
        if !out.status.success() {
            return Err(format!(
                "there is no rustup, and `{shown} -vV` failed ({})",
                out.status
            ));
        }
        let text = String::from_utf8_lossy(&out.stdout);
        for (key, wanted) in fields {
            let reported = text
                .lines()
                .find_map(|line| line.strip_prefix(key)?.strip_prefix(": "));
            if reported != Some(wanted) {
                return Err(format!(
                    "there is no rustup, and {shown} is not the {tool} of the pinned {channel}: \
                     its `-vV` reports {key}: {}, not {wanted}; install rustup, or run the demo \
                     in the flake's dev shell (`nix develop`)",
                    reported.unwrap_or("nothing")
                ));
            }
        }
        if tool == "rustc" {
            version = text.lines().next().unwrap_or_default().to_owned();
        }
    }
    Ok(version)
}

/// The artifacts Cargo reported building, for one `cargo build` that succeeded.
fn cargo_build(
    root: &Path,
    toolchain: &Toolchain,
    target: &Path,
    package: &str,
) -> Result<Vec<CargoMessage>, Failure> {
    // Diagnostics render to standard error; standard output carries one JSON message per line.
    let mut cargo = pinned(root, toolchain, "cargo");
    let program = Path::new(cargo.get_program()).display().to_string();
    let out = cargo
        .args(["build", "--quiet", "--locked", "-p", package])
        .arg("--message-format=json-render-diagnostics")
        .arg("--target-dir")
        .arg(target)
        .stderr(Stdio::inherit())
        .output()
        .map_err(|e| setup(format!("cannot run {program}: {e}")))?;
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

/// The first line of `command`'s standard output, if it ran and succeeded.
fn first_line(command: &mut Command) -> Option<String> {
    command
        .stdin(Stdio::null())
        .stderr(Stdio::null())
        .output()
        .ok()
        .filter(|out| out.status.success())
        .and_then(|out| String::from_utf8(out.stdout).ok())
        .and_then(|text| text.lines().next().map(str::to_owned))
}

/// The variable through which the flake's `tide-demo` app gives the demo its revision: its
/// sources are a copy in the Nix store, without `.git`, so git cannot name it there.
pub const REVISION_VAR: &str = "TIDE_REVISION";

/// The variables through which a caller's environment would route git to another repository,
/// index or object store than the one at a `-C` path: `git rev-parse --local-env-vars`, the set
/// git itself clears when it moves into a submodule.
const GIT_LOCAL_ENV_VARS: [&str; 15] = [
    "GIT_ALTERNATE_OBJECT_DIRECTORIES",
    "GIT_CONFIG",
    "GIT_CONFIG_PARAMETERS",
    "GIT_CONFIG_COUNT",
    "GIT_OBJECT_DIRECTORY",
    "GIT_DIR",
    "GIT_WORK_TREE",
    "GIT_IMPLICIT_WORK_TREE",
    "GIT_GRAFT_FILE",
    "GIT_INDEX_FILE",
    "GIT_NO_REPLACE_OBJECTS",
    "GIT_REPLACE_REF_BASE",
    "GIT_PREFIX",
    "GIT_SHALLOW_FILE",
    "GIT_COMMON_DIR",
];

/// Step 0's revision of the sources at `root`: git's `HEAD`, marked when the tree has
/// uncommitted changes, or when git cannot tell whether it has; else `recorded`, the value of
/// [`REVISION_VAR`], when it is not empty; else `unknown`. Git runs on `root` alone, whatever
/// repository the caller's environment routes it to.
pub fn source_revision(root: &Path, recorded: Option<&str>) -> String {
    let git = || {
        let mut git = Command::new("git");
        for var in GIT_LOCAL_ENV_VARS {
            git.env_remove(var);
        }
        git.arg("-C").arg(root);
        git
    };
    if let Some(head) = first_line(git().args(["rev-parse", "HEAD"])) {
        let status = git()
            .args(["--no-optional-locks", "status", "--porcelain"])
            .stdin(Stdio::null())
            .stderr(Stdio::null())
            .output();
        let dirty = match status {
            Ok(out) if out.status.success() && out.stdout.is_empty() => "",
            Ok(out) if out.status.success() => " (with uncommitted changes)",
            _ => " (uncommitted changes unknown)",
        };
        return format!("{head}{dirty}");
    }
    match recorded {
        Some(revision) if !revision.is_empty() => revision.to_owned(),
        _ => "unknown".into(),
    }
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
    toolchain: Toolchain,
    driver: PathBuf,
    std: PathBuf,
    deps: Vec<PathBuf>,
}

impl Run {
    /// Step 0: builds the driver and std with the pinned toolchain.
    fn build(root: PathBuf, target: PathBuf, work: PathBuf) -> Result<Run, Failure> {
        step("0. Toolchain and build");
        let recorded = std::env::var(REVISION_VAR).ok();
        println!("revision:  {}", source_revision(&root, recorded.as_deref()));
        // The pinned compiler's identity; without it, nothing can be built.
        let channel = pinned_toolchain(&root)?;
        let (toolchain, version) = if on_path(OsStr::new("rustup")).is_some() {
            let toolchain = Toolchain::Rustup(channel.clone());
            let version = pinned(&root, &toolchain, "rustc")
                .arg("--version")
                .output()
                .map_err(|e| setup(format!("cannot run rustup: {e}")))?;
            let version = String::from_utf8(version.stdout)
                .ok()
                .filter(|_| version.status.success())
                .and_then(|text| text.lines().next().map(str::to_owned))
                .ok_or_else(|| setup(format!("the pinned toolchain {channel} is not installed")))?;
            (toolchain, version)
        } else {
            // The rustc: `$RUSTC`, as the flake's dev shell sets it, else `rustc` on `PATH`.
            let rustc = std::env::var_os("RUSTC")
                .filter(|rustc| !rustc.is_empty())
                .unwrap_or_else(|| "rustc".into());
            let found = |program: &OsStr| {
                resolve(program).ok_or_else(|| {
                    let shown = Path::new(program).display();
                    setup(format!("there is no rustup, and no {shown} on PATH"))
                })
            };
            let (rustc, cargo) = (found(&rustc)?, found(OsStr::new("cargo"))?);
            let version = check_path_toolchain(&channel, &rustc, &cargo).map_err(setup)?;
            (Toolchain::Path { rustc, cargo }, version)
        };
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
            toolchain,
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
        self.logged(&mut self.suite_command(library, out, run), log)
    }

    fn suite_command(&self, library: &Path, out: &str, run: &[&OsStr]) -> Command {
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
        command
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
    /// reproduce it with a byte-identical trace. Returns the recording's manifest.
    fn record_and_replay(&self, buggy: &Path, seed: u64) -> Result<PathBuf, Failure> {
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
        Ok(path.clone())
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

    /// Step 5: the failing seed's replay under rust-gdb in batch mode. gdb follows the driver
    /// into its sandboxed test child, and its breakpoints in `record_hit` show the lost update: a
    /// task writing back `seen + 1` over a count already past the `seen` it read. Each stop must
    /// be at the source line of `buggy` it belongs to, and the replay must still reproduce the
    /// recording under the debugger. Skipped when there is no gdb.
    fn debug(&self, buggy: &Path, seed: u64, recording: &Path) -> Result<(), Failure> {
        step(format!(
            "5. Debug the replay of seed {seed} under rust-gdb, with breakpoints in record_hit"
        ));
        // Written on every run, so the by-hand commands in docs/debugging.md find it.
        let script = self.work.join("debug.gdb");
        fs::write(&script, GDB_SCRIPT).map_err(|e| setup(format!("{}: {e}", script.display())))?;
        let Some((gdb, explicit)) = find_gdb() else {
            println!(
                "skipped: no gdb (set RUST_GDB to a gdb built with Python, or put gdb on PATH)"
            );
            return Ok(());
        };
        // A gdb that only happens to be on PATH must run Python, which the script needs, or the
        // step is skipped; one named by RUST_GDB is required, so its failures fail the step.
        if !explicit && let Err(why) = probe_gdb(&gdb) {
            println!(
                "skipped: the gdb on PATH ({}) cannot run the script: {why} (set RUST_GDB to \
                 require it)",
                Path::new(&gdb).display()
            );
            return Ok(());
        }
        let replay = self.suite_command(
            buggy,
            "record",
            &[OsStr::new("--replay"), recording.as_os_str()],
        );
        let mut command = pinned(&self.root, &self.toolchain, "rust-gdb");
        command
            .env("RUST_GDB", &gdb)
            .args(["-nx", "-batch", "-x"])
            .arg(&script)
            .arg("--args")
            .arg(replay.get_program())
            .args(replay.get_args());
        // The log is complete only once the driver, which gdb detaches when it follows the test
        // child, has exited too: `run_bounded` waits until the output pipe closes and no process
        // of gdb's group is running, and kills the group at the deadline.
        let run = run_bounded(command, DEBUG_DEADLINE)
            .map_err(|e| setup(format!("cannot run rust-gdb: {e}")))?;
        let log = self.work.join("logs/debug.log");
        fs::write(&log, &run.output).map_err(|e| setup(format!("{}: {e}", log.display())))?;
        let fail = |message: String| Failure::Step {
            step: 5,
            message,
            log: Some(log.clone()),
        };
        let deadline = DEBUG_DEADLINE.as_secs();
        match run.end {
            End::Killed => {
                return Err(fail(format!(
                    "rust-gdb did not finish within {deadline} s, so its process group was killed"
                )));
            }
            End::NotCleaned(why) => {
                return Err(fail(format!(
                    "rust-gdb did not finish within {deadline} s, and its process group could not \
                     be cleaned up: {why}"
                )));
            }
            End::Exited(status) if !status.success() => {
                return Err(fail(format!("rust-gdb failed ({status})")));
            }
            End::Exited(_) => {}
        }
        let text = String::from_utf8_lossy(&run.output);
        let sandbox = text
            .lines()
            .find(|line| line.starts_with("sandbox: "))
            .ok_or_else(|| fail("the driver reported no sandbox level".into()))?;
        println!("{sandbox}");
        // The debug profile's line information puts the entry and the read on the read's line,
        // and the write on the write's line.
        let source = fs::read_to_string(buggy)
            .map_err(|e| setup(format!("{}: {e}", buggy.display())))?;
        let line_of = |code: &str| {
            source
                .lines()
                .position(|line| line.trim() == code)
                .map(|i| i + 1)
                .ok_or_else(|| setup(format!("{} has no line `{code}`", buggy.display())))
        };
        let read_line = line_of("let seen = stats.with(|s| s.hits)?;")?;
        let write_line = line_of("stats.with(|s| s.hits = seen + 1)")?;
        let mut lost = None;
        let (mut entries, mut reads, mut writes) = (0, 0, 0);
        let mut threads = BTreeSet::new();
        // Each thread's last read: its write must write back that count plus one.
        let mut last_read = std::collections::BTreeMap::new();
        for line in text.lines().filter(|line| line.starts_with("gdb: thread ")) {
            let access = Access::parse(line)
                .ok_or_else(|| fail(format!("unreadable breakpoint line: {line}")))?;
            println!("{line}");
            let expected = match access.kind {
                Kind::Write { .. } => write_line,
                Kind::Enter | Kind::Read(_) => read_line,
            };
            if access.file != "hits.rs" || access.line != expected {
                return Err(fail(format!(
                    "gdb stopped at {}:{}, not at hits.rs:{expected}",
                    access.file, access.line
                )));
            }
            threads.insert(access.thread);
            match access.kind {
                Kind::Enter => entries += 1,
                Kind::Read(value) => {
                    reads += 1;
                    last_read.insert(access.thread, value);
                }
                Kind::Write { seen, over } => {
                    writes += 1;
                    if last_read.get(&access.thread) != Some(&seen) {
                        return Err(fail(format!(
                            "thread {} wrote {seen} + 1, but its last read was {:?}",
                            access.thread,
                            last_read.get(&access.thread)
                        )));
                    }
                    if over > seen && lost.is_none() {
                        lost = Some((access.thread, seen, over));
                    }
                }
            }
        }
        if entries == 0 || entries != reads || reads != writes {
            return Err(fail(format!(
                "the breakpoints did not all hit: {entries} record_hit, {reads} reads, {writes} writes"
            )));
        }
        let Some((thread, seen, over)) = lost.filter(|_| threads.len() > 1) else {
            return Err(fail("gdb showed no lost update between two tasks".into()));
        };
        println!(
            "thread {thread} wrote {seen} + 1 over hits = {over}: it read the count before another \
             task wrote it, so that task's hit is lost"
        );
        let marker = format!("replay reproduced: test={TEST} seed={seed} ");
        if !text.lines().any(|line| line.starts_with(&marker)) {
            return Err(fail(
                "the replay under gdb did not reproduce the recording".into(),
            ));
        }
        println!("the replay under gdb reproduced the recording: the same schedule and trace");
        Ok(())
    }
}

/// The script step 5 gives rust-gdb, written to `debug.gdb` in the work directory. The driver
/// re-executes itself and then starts the test binary in its sandboxed child (`supervise`), so
/// gdb follows those two spawns and stays with the parent at every other one, such as the
/// linker. The debug profile carries debug information, so each breakpoint prints named locals
/// (`s`, the closures' `&mut Stats`, and `seen`, the write's captured count) and the source line
/// it stopped at (`$here()`). The breakpoints name functions, not lines: a line such as
/// `let seen = stats.with(|s| s.hits)?;` holds code of both `record_hit` and its closure, so a
/// breakpoint on it stops in each. Arguments are parsed as C (`s->hits`), since gdb's Rust parser
/// rejects them in `dprintf`.
const GDB_SCRIPT: &str = r#"set pagination off
set confirm off
set breakpoint pending on
set language c
handle SIGUSR1 nostop noprint pass
python
class Follow(gdb.Function):
    """$follow(): 1 when the forking process is starting the driver again or its test child."""
    def __init__(self):
        super().__init__("follow")
    def invoke(self):
        frame = gdb.newest_frame()
        while frame is not None:
            name = frame.name() or ""
            if "supervise" in name or "Command>::status" in name:
                return 1
            frame = frame.older()
        return 0
Follow()
class Here(gdb.Function):
    """$here(): the source file and line where the newest frame stopped."""
    def __init__(self):
        super().__init__("here")
    def invoke(self):
        sal = gdb.newest_frame().find_sal()
        return "%s:%d" % (sal.symtab.filename, sal.line)
Here()
end
catch fork
catch vfork
commands 1 2
  silent
  if $follow()
    set follow-fork-mode child
  else
    set follow-fork-mode parent
  end
  continue
end
dprintf tide_candidate::record_hit,"gdb: thread %d enters record_hit at %s\n",$_thread,$here()
dprintf 'tide_candidate::record_hit::{closure#0}',"gdb: thread %d reads hits = %lu at %s\n",$_thread,s->hits,$here()
dprintf 'tide_candidate::record_hit::{closure#1}',"gdb: thread %d writes hits = %lu + 1 over hits = %lu at %s\n",$_thread,seen,s->hits,$here()
run
"#;

/// One line the script's breakpoints print: a thread entering `record_hit`, reading the count,
/// or writing `seen + 1` over the count it finds, and the source file name and line it stopped at.
struct Access {
    thread: u64,
    kind: Kind,
    file: String,
    line: usize,
}

enum Kind {
    Enter,
    Read(u64),
    Write { seen: u64, over: u64 },
}

impl Access {
    fn parse(line: &str) -> Option<Access> {
        let (thread, rest) = line.strip_prefix("gdb: thread ")?.split_once(' ')?;
        let thread = thread.parse().ok()?;
        let (rest, at) = rest.rsplit_once(" at ")?;
        let (path, number) = at.rsplit_once(':')?;
        let file = Path::new(path).file_name()?.to_str()?.to_owned();
        let line = number.parse().ok()?;
        let kind = if rest == "enters record_hit" {
            Kind::Enter
        } else if let Some(value) = rest.strip_prefix("reads hits = ") {
            Kind::Read(value.parse().ok()?)
        } else {
            let (seen, over) = rest
                .strip_prefix("writes hits = ")?
                .split_once(" + 1 over hits = ")?;
            Kind::Write {
                seen: seen.parse().ok()?,
                over: over.parse().ok()?,
            }
        };
        Some(Access {
            thread,
            kind,
            file,
            line,
        })
    }
}

/// The first file `name` on `PATH`, as an absolute path.
fn on_path(name: &OsStr) -> Option<OsString> {
    let paths = std::env::var_os("PATH")?;
    std::env::split_paths(&paths)
        .map(|dir| dir.join(name))
        .find(|file| file.is_file())
        .and_then(|file| std::path::absolute(file).ok())
        .map(PathBuf::into_os_string)
}

/// `program` as an absolute path, as a shell would run it: a path is taken from the working
/// directory, and a bare name is looked up on `PATH`.
fn resolve(program: &OsStr) -> Option<OsString> {
    match Path::new(program).components().count() {
        0 => None,
        1 => on_path(program),
        _ => std::path::absolute(program)
            .ok()
            .map(PathBuf::into_os_string),
    }
}

/// The gdb rust-gdb runs, as an absolute path, since rust-gdb runs in the repository rather than
/// the caller's directory, and whether `RUST_GDB` named it: `$RUST_GDB`, else `gdb` on `PATH`. A
/// `RUST_GDB` naming a path is taken from the caller's directory, and a bare name is looked up on
/// `PATH`; one that is not found is passed on as given, so a configured debugger that does not
/// exist fails step 5 instead of skipping it. `None` when `RUST_GDB` is unset or empty and `PATH`
/// has no `gdb`.
fn find_gdb() -> Option<(OsString, bool)> {
    match std::env::var_os("RUST_GDB").filter(|gdb| !gdb.is_empty()) {
        Some(gdb) => Some((resolve(&gdb).unwrap_or(gdb), true)),
        None => on_path(OsStr::new("gdb")).map(|gdb| (gdb, false)),
    }
}

/// Whether `gdb` starts and runs Python, which step 5's script needs: why not, if it does not.
fn probe_gdb(gdb: &OsStr) -> Result<(), String> {
    let mut command = Command::new(gdb);
    command.args(["-nx", "-batch", "-ex", "python print(1)"]);
    let run = run_bounded(command, Duration::from_secs(30)).map_err(|e| e.to_string())?;
    match run.end {
        End::Exited(status)
            if status.success()
                && String::from_utf8_lossy(&run.output)
                    .lines()
                    .any(|l| l == "1") =>
        {
            Ok(())
        }
        End::Exited(status) => Err(format!("`python print(1)` failed ({status})")),
        End::Killed | End::NotCleaned(_) => Err("`python print(1)` did not finish in 30 s".into()),
    }
}

/// How long step 5's rust-gdb run may take, from start until it and every process it started
/// have exited. A run takes a few seconds; the driver's own 300 s backstop bounds only the test
/// evaluation, not a debugger stuck before or after it.
const DEBUG_DEADLINE: Duration = Duration::from_secs(120);

/// How a `run_bounded` run ended.
#[derive(Debug)]
pub enum End {
    /// The command exited, its output closed, and no process of its group was left running,
    /// all before the deadline.
    Exited(ExitStatus),
    /// The deadline passed first; the process group was killed and no member is left running.
    Killed,
    /// The deadline passed first, and the group could not be killed, or still had a running
    /// member 5 s after the kill: why.
    NotCleaned(String),
}

/// What `run_bounded` gave: how the run ended, and everything the command and its descendants
/// wrote to standard output and error.
pub struct Bounded {
    pub end: End,
    pub output: Vec<u8>,
}

/// Runs `command` in a new process group, with empty standard input and standard output and
/// error on one pipe. It returns `End::Exited` once the pipe has closed, the command has exited,
/// and no process of the group is still running, all observed before `deadline`: a descendant
/// the command left behind, such as the driver gdb detaches, is waited for too. Otherwise it
/// kills the group at the deadline and returns `End::Killed`, or `End::NotCleaned` when that
/// fails.
///
/// The contract is that no member of the group is left running when it returns `Exited` or
/// `Killed`. A killed member whose parent had already exited is reaped by init or the session's
/// subreaper, like any orphan; this function is not its parent, and becoming a subreaper would
/// change process-wide behaviour inside a test binary that runs other tests in parallel. A
/// process that left the group (`setsid`, as the driver's sandboxed test child does) is outside
/// the contract; the test child dies with the driver that started it (`PR_SET_PDEATHSIG`).
///
/// The group's id is the command's pid. The command is reaped only after the group is killed or
/// found empty, and its exit is detected with `waitid(WNOWAIT)` until then, so the id cannot be
/// reused while the group is signalled.
pub fn run_bounded(mut command: Command, deadline: Duration) -> io::Result<Bounded> {
    let end = Instant::now() + deadline;
    let (mut reader, writer) = io::pipe()?;
    let mut child = command
        .stdin(Stdio::null())
        .stdout(writer.try_clone()?)
        .stderr(writer)
        .process_group(0)
        .spawn()?;
    // The command keeps its copies of the pipe's write end until it is dropped.
    drop(command);
    let group = child.id();
    let (send, receive) = mpsc::channel::<Vec<u8>>();
    // Not joined: when cleanup fails, a surviving writer can keep it blocked.
    std::thread::spawn(move || {
        let mut buffer = [0; 8192];
        loop {
            match reader.read(&mut buffer) {
                Ok(0) => break,
                Ok(n) => {
                    if send.send(buffer[..n].to_vec()).is_err() {
                        break;
                    }
                }
                Err(e) if e.kind() == io::ErrorKind::Interrupted => {}
                Err(_) => break,
            }
        }
    });
    let mut output = Vec::new();
    let mut closed = false;
    let mut exited = false;
    while Instant::now() < end {
        if closed {
            exited = exited
                || match exited_unreaped(group) {
                    Ok(exited) => exited,
                    Err(e) => {
                        kill_group(group);
                        return Err(e);
                    }
                };
            // Success counts only if every observation, the last included, came before the
            // deadline.
            if exited && group_members(group).is_empty() && Instant::now() < end {
                let status = match child.wait() {
                    Ok(status) => status,
                    Err(e) => {
                        kill_group(group);
                        return Err(e);
                    }
                };
                return Ok(Bounded {
                    end: End::Exited(status),
                    output,
                });
            }
            std::thread::sleep(Duration::from_millis(20));
            continue;
        }
        let wait = end.saturating_duration_since(Instant::now());
        match receive.recv_timeout(wait) {
            Ok(bytes) => output.extend(bytes),
            Err(RecvTimeoutError::Disconnected) => closed = true,
            Err(RecvTimeoutError::Timeout) => {}
        }
    }
    let end = kill_group(group);
    if matches!(end, End::Killed) {
        // The command is killed or had exited, so this does not block.
        child.wait()?;
    }
    // What the killed processes wrote before they died, for at most 2 s: a writer outside the
    // group, or one that survived, can keep the channel supplied, so the clock bounds the drain.
    let grace = Instant::now() + Duration::from_secs(2);
    while Instant::now() < grace {
        match receive.recv_timeout(grace.saturating_duration_since(Instant::now())) {
            Ok(bytes) => output.extend(bytes),
            Err(_) => break,
        }
    }
    Ok(Bounded { end, output })
}

/// Whether process `pid`, a child of this process, has exited, without reaping it.
fn exited_unreaped(pid: u32) -> io::Result<bool> {
    // SAFETY: `info` is a valid, zeroed `siginfo_t` that `waitid` fills; WNOWAIT leaves the child
    // waitable, so `Child::wait` still reaps it.
    unsafe {
        let mut info: libc::siginfo_t = std::mem::zeroed();
        let flags = libc::WEXITED | libc::WNOHANG | libc::WNOWAIT;
        if libc::waitid(libc::P_PID, pid as libc::id_t, &mut info, flags) != 0 {
            return Err(io::Error::last_os_error());
        }
        // WNOHANG leaves `si_pid` zero when the child has not exited.
        Ok(info.si_pid() != 0)
    }
}

/// The processes in process group `group` that are still running (not zombies), from `/proc`.
fn group_members(group: u32) -> Vec<u32> {
    let Ok(entries) = fs::read_dir("/proc") else {
        return Vec::new();
    };
    entries
        .filter_map(|entry| entry.ok()?.file_name().to_str()?.parse::<u32>().ok())
        .filter(|pid| {
            // `/proc/PID/stat`: `pid (comm) state ppid pgrp ...`; `comm` may hold spaces.
            fs::read_to_string(format!("/proc/{pid}/stat")).is_ok_and(|stat| {
                let mut fields = stat
                    .rsplit_once(") ")
                    .map_or("", |(_, rest)| rest)
                    .split(' ');
                let state = fields.next();
                let pgrp = fields.nth(1).and_then(|pgrp| pgrp.parse::<u32>().ok());
                state != Some("Z") && pgrp == Some(group)
            })
        })
        .collect()
}

/// Sends `SIGKILL` to process group `group`, whose leader is an unreaped child of this process,
/// and waits up to 5 s for no member to be left running. `kill` does not block.
fn kill_group(group: u32) -> End {
    // SAFETY: `kill` takes plain integers and touches no memory.
    if unsafe { libc::kill(-(group as libc::pid_t), libc::SIGKILL) } != 0 {
        let error = io::Error::last_os_error();
        // ESRCH: no process is left in the group.
        if error.raw_os_error() != Some(libc::ESRCH) {
            return End::NotCleaned(format!("kill of process group {group} failed: {error}"));
        }
    }
    let wait = Instant::now() + Duration::from_secs(5);
    loop {
        let members = group_members(group);
        if members.is_empty() {
            return End::Killed;
        }
        if Instant::now() >= wait {
            return End::NotCleaned(format!(
                "processes {members:?} of group {group} were still running 5 s after SIGKILL"
            ));
        }
        std::thread::sleep(Duration::from_millis(20));
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
