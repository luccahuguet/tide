//! The benchmark rerun tool (fx-6u8m.6, fx-6u8m.2): builds and runs every pair in `benchmarks/`
//! under each configuration, measuring each build's wall time, peak memory, and binary size, each
//! rebuild after a one-line edit, each run's wall time, the `tasks` pair under the simulator, and
//! the `contracts` pair as a test build, and writes one raw record per measurement to a results
//! directory. `benchmarks/README.md`
//! states the configurations, the method, and the record format.
//!
//! `cargo bench -p tide-admit --bench pairs -- [--quick] [--out DIR] [--pair NAME]... [--phase NAME]...`
//!
//! It is a `harness = false` bench target of the driver's package, so it builds Tide programs
//! with the same helpers and the same driver binary as the equivalence check
//! (`tests/benchmarks.rs`), and Cargo builds that driver optimized, as rustc is.
use std::fs::{self, File};
use std::io::{LineWriter, Read, Write};
use std::path::{Path, PathBuf};
use std::process::{Command, ExitCode, ExitStatus, Stdio};
use std::time::{SystemTime, UNIX_EPOCH};

#[path = "../tests/support/admitted.rs"]
mod admitted;
// Public so `tests/benchmarks.rs`, which includes this file, tests them through it.
#[path = "support/peak_rss.rs"]
pub mod peak_rss;
#[path = "support/stats.rs"]
pub mod stats;

use admitted::{build_std, driver_with, package, timed};
use peak_rss::{Measured, measure};
use stats::{cv, interleaved, json_string, median, more_rounds, stable};

/// Each pair's size argument in a full run, chosen so that its release arms run for about
/// 0.5-2 s (0.54-1.26 s when chosen, 2026-11-17; the debug arms took up to 8.4 s). A quick run
/// passes none, so each program uses its small default.
const TIMED: [(&str, &str); 4] = [
    ("contracts", "1200000"),
    ("files", "30000"),
    ("tasks", "10000000"),
    ("tcp", "100000"),
];
/// The pair whose Tide arm the simulator phase also runs, and its size there in a full run: one
/// seed of the simulator runs one task at a time, so it takes a smaller size than `TIMED`
/// (estimated at 2-9 s per simulated run from a quick run's cost per trace event, 2026-11-17).
const SIMULATED: &str = "tasks";
const SIM_SIZE: &str = "1000000";
/// The one seed the simulator phase runs.
const SIM_SEED: u64 = 0;
/// The pair the test-build phase also builds as a debug test build, and its size there in a full
/// run: its sampled contract clauses run only in a test build. It is smaller than `TIMED`, since
/// one test-build run at `TIMED`'s 1,200,000 was estimated at about 70 s from a quick run
/// (2026-11-18).
const CONTRACTED: &str = "contracts";
const TEST_SIZE: &str = "200000";
/// Rounds a full run starts with, adds while a series' CV is over 1%, and never passes.
const START: usize = 5;
const STEP: usize = 5;
const CAP: usize = 30;
/// The limit on each run of a built program, in seconds; the debug arms run the timed size too.
const RUN_LIMIT: u32 = 120;

enum Compiler {
    /// The admission driver, against Tide's std built in Cargo's release or dev profile.
    Driver { release_std: bool },
    /// The pinned nightly's rustc, run directly from its sysroot.
    Pinned,
    /// The stable toolchain's rustc (`rustup run stable`), run directly from its sysroot.
    Stable,
}

struct Config {
    id: &'static str,
    compiler: Compiler,
    flags: &'static [&'static str],
}

impl Config {
    /// Reported for information only, never compared: the stable arm (stable rustc cannot
    /// build Tide).
    fn informational(&self) -> bool {
        matches!(self.compiler, Compiler::Stable)
    }
}

/// Cargo's dev profile's code generation: no optimization, full debug information, debug
/// assertions and overflow checks. Not its 256 codegen units or incremental mode: like the
/// driver, rustc runs non-incrementally with its default codegen units.
const DEV: &[&str] = &[
    "-Copt-level=0",
    "-Cdebuginfo=2",
    "-Cdebug-assertions=on",
    "-Coverflow-checks=on",
];
/// Cargo's release profile's code generation.
const RELEASE: &[&str] = &[
    "-Copt-level=3",
    "-Cdebug-assertions=off",
    "-Coverflow-checks=off",
];

/// The configurations, in the order each round measures them.
const CONFIGS: [Config; 6] = [
    Config {
        id: "tide-debug",
        compiler: Compiler::Driver { release_std: false },
        flags: &["--tide-profile=debug"],
    },
    Config {
        id: "tide-release",
        compiler: Compiler::Driver { release_std: true },
        flags: &["--tide-profile=release"],
    },
    Config {
        id: "rust-debug",
        compiler: Compiler::Pinned,
        flags: DEV,
    },
    Config {
        id: "rust-release",
        compiler: Compiler::Pinned,
        flags: RELEASE,
    },
    // Release with the flags the driver adds in every profile (`invocation.rs`), so the flags'
    // cost separates from the rest of Tide's.
    Config {
        id: "rust-release-tide-flags",
        compiler: Compiler::Pinned,
        flags: &[
            "-Copt-level=3",
            "-Cdebug-assertions=off",
            "-Coverflow-checks=on",
            "-Zinline-mir=no",
            "-Zcross-crate-inline-threshold=never",
            "-Zshare-generics=no",
        ],
    },
    Config {
        id: "stable-release",
        compiler: Compiler::Stable,
        flags: RELEASE,
    },
];

/// The configuration named `id`.
fn config(id: &str) -> &'static Config {
    CONFIGS.iter().find(|c| c.id == id).expect("a configuration in CONFIGS")
}

/// What a phase measures: a cold build, a rebuild after a one-line edit, a run of the built
/// program, (`tasks` only) a run against a simulated run of the same Tide build, or
/// (`contracts` only) a debug test build's run against the plain builds' runs.
#[derive(Clone, Copy, PartialEq)]
pub enum Phase {
    Build,
    Rebuild,
    Run,
    Sim,
    Test,
}

const PHASES: [Phase; 5] = [Phase::Build, Phase::Rebuild, Phase::Run, Phase::Sim, Phase::Test];

/// How an arm runs its pair: as the built program, or inside a test binary the tool builds with
/// a module appended: under the simulator, or as a test build.
#[derive(Clone, Copy, PartialEq)]
enum Harness {
    Program,
    Sim,
    Test,
}

/// One series of a phase: a configuration and how it runs.
struct Arm {
    config: &'static Config,
    harness: Harness,
}

impl Arm {
    /// The name its records give it: the configuration's, with `-sim` for a simulated run and
    /// `-test` for a test build's run.
    fn id(&self) -> String {
        match self.harness {
            Harness::Program => self.config.id.to_owned(),
            Harness::Sim => format!("{}-sim", self.config.id),
            Harness::Test => format!("{}-test", self.config.id),
        }
    }

    /// The test binary's module name and the test it times, for an arm in a test binary.
    fn module(&self) -> Option<&'static str> {
        match self.harness {
            Harness::Program => None,
            Harness::Sim => Some("bench_sim"),
            Harness::Test => Some("bench_test"),
        }
    }
}

impl Phase {
    fn name(self) -> &'static str {
        match self {
            Phase::Build => "build",
            Phase::Rebuild => "rebuild",
            Phase::Run => "run",
            Phase::Sim => "sim",
            Phase::Test => "test",
        }
    }

    /// Whether this phase measures `pair`: the simulator and test-build phases measure one pair
    /// each.
    fn measures(self, pair: &str) -> bool {
        match self {
            Phase::Sim => pair == SIMULATED,
            Phase::Test => pair == CONTRACTED,
            Phase::Build | Phase::Rebuild | Phase::Run => true,
        }
    }

    /// The arms of this phase, in the order each round measures them. A rebuild compares the
    /// edit loop's debug builds; the simulator phase, each Tide build's real and simulated runs;
    /// the test-build phase, a debug test build against the plain debug, release, and Rust
    /// debug builds.
    fn arms(self) -> Vec<Arm> {
        let arm = |id, harness| Arm {
            config: config(id),
            harness,
        };
        match self {
            Phase::Build | Phase::Run => CONFIGS
                .iter()
                .map(|config| Arm {
                    config,
                    harness: Harness::Program,
                })
                .collect(),
            Phase::Rebuild => vec![
                arm("tide-debug", Harness::Program),
                arm("rust-debug", Harness::Program),
            ],
            Phase::Sim => vec![
                arm("tide-debug", Harness::Program),
                arm("tide-debug", Harness::Sim),
                arm("tide-release", Harness::Program),
                arm("tide-release", Harness::Sim),
            ],
            Phase::Test => vec![
                arm("tide-debug", Harness::Program),
                arm("tide-debug", Harness::Test),
                arm("tide-release", Harness::Program),
                arm("rust-debug", Harness::Program),
            ],
        }
    }
}

/// The part of a test binary's standard output that is the program's: the window equal to
/// `expected`, the reference arm's output, wherever the test harness's own lines put it; the
/// whole output when no such window exists, so that the output check then fails.
pub fn within<'a>(output: &'a [u8], expected: Option<&[u8]>) -> &'a [u8] {
    match expected {
        Some(expected) if !expected.is_empty() => output
            .windows(expected.len())
            .position(|window| window == expected)
            .map_or(output, |at| &output[at..at + expected.len()]),
        _ => output,
    }
}

/// The module the test-build phase appends to the `contracts` Tide arm: `timed` runs the
/// program's `run` at `limit` (its default without one) in a test build; `sampled`, never
/// timed, passes only if the sampled clause `requires(xs.is_sorted())` runs and fails.
fn test_module(limit: Option<&str>) -> String {
    format!(
        r#"
// Appended by the benchmark rerun tool's test-build phase (crates/tide-admit/benches/pairs.rs).
#[cfg(test)]
mod bench_test {{
    use super::*;

    #[test]
    fn timed() {{
        run({limit});
    }}

    #[test]
    #[should_panic(expected = "xs.is_sorted()")]
    fn sampled() {{
        let mut xs = vec![2, 1];
        insert_sorted(&mut xs, 3);
    }}
}}
"#,
        limit = limit.unwrap_or("DEFAULT_LIMIT")
    )
}

/// `source` with the rebuild's one-line edit for `round`: a new statement as the first line of
/// `main`'s body, which must open on the one line that starts with `fn main(`.
pub fn edited(source: &str, round: usize) -> Result<String, String> {
    let mains: Vec<usize> = source
        .lines()
        .enumerate()
        .filter(|(_, l)| l.starts_with("fn main(") && l.trim_end().ends_with('{'))
        .map(|(i, _)| i)
        .collect();
    let [main] = mains[..] else {
        return Err(format!(
            "{} lines open `fn main(` and its body; the edit needs exactly one",
            mains.len()
        ));
    };
    let mut out = String::with_capacity(source.len() + 32);
    for (i, line) in source.lines().enumerate() {
        out.push_str(line);
        out.push('\n');
        if i == main {
            out.push_str(&format!("    let _edit: u64 = {round};\n"));
        }
    }
    Ok(out)
}

/// The module the simulator phase appends to the `tasks` Tide arm: `timed` runs `run` under the
/// simulator on one seed with `args` and writes its standard output to standard error, for the
/// output check; `events`, never timed, writes how many events the scheduler traced.
fn sim_module(args: &[&str]) -> String {
    let args: Vec<String> = args.iter().map(|a| format!("{a:?}")).collect();
    format!(
        r#"
// Appended by the benchmark rerun tool's simulator phase (crates/tide-admit/benches/pairs.rs).
#[cfg(test)]
mod bench_sim {{
    use super::*;
    use std::sim::Sink;

    // Each test writes only after its simulation ends: while one runs, `eprint!` writes to
    // its simulated sink.
    #[test]
    fn timed() {{
        let out: String = {{
            let sim = Root::sim({seed});
            sim.world.args([{args}]);
            run(&sim.env, &sim.tasks).unwrap();
            sim.records()
                .into_iter()
                .filter(|r| r.sink == Sink::Out)
                .map(|r| r.message + "\n")
                .collect()
        }};
        eprint!("{{out}}");
    }}

    #[test]
    fn events() {{
        let events = {{
            let sim = Root::sim({seed});
            sim.world.args([{args}]);
            run(&sim.env, &sim.tasks).unwrap();
            sim.trace().lines().count()
        }};
        eprint!("{{events}}");
    }}
}}
"#,
        args = args.join(", "),
        seed = SIM_SEED,
    )
}

/// How many bytes of an executable `binary_size` reads: enough for the ELF header and the
/// program headers that follow it. Reading only these keeps the tool's own memory small, which
/// its children's peak RSS would otherwise include.
const ELF_HEAD: u64 = 64 * 1024;

/// The file size `len` of an ELF executable whose first bytes are `bytes`, and the bytes its
/// loadable segments take in the file (the sum of its `PT_LOAD` segments' file sizes), which
/// leaves out debug information and symbol tables. `None` for anything but a 64-bit
/// little-endian ELF file whose program headers lie within `bytes`.
pub fn binary_size(bytes: &[u8], len: u64) -> Option<(u64, u64)> {
    let u16_at = |at: usize| Some(u16::from_le_bytes(bytes.get(at..at + 2)?.try_into().ok()?));
    let u32_at = |at: usize| Some(u32::from_le_bytes(bytes.get(at..at + 4)?.try_into().ok()?));
    let u64_at = |at: usize| Some(u64::from_le_bytes(bytes.get(at..at + 8)?.try_into().ok()?));
    // The magic, ELFCLASS64, and ELFDATA2LSB.
    if bytes.get(..6)? != b"\x7fELF\x02\x01" {
        return None;
    }
    let (table, entry, count) = (
        usize::try_from(u64_at(0x20)?).ok()?,
        usize::from(u16_at(0x36)?),
        usize::from(u16_at(0x38)?),
    );
    let mut loaded = 0u64;
    for i in 0..count {
        let header = table.checked_add(i.checked_mul(entry)?)?;
        // PT_LOAD; p_filesz is at 0x20 in an Elf64_Phdr.
        if u32_at(header)? == 1 {
            loaded = loaded.checked_add(u64_at(header + 0x20)?)?;
        }
    }
    Some((len, loaded))
}

pub struct Options {
    pub quick: bool,
    pub out: Option<PathBuf>,
    pub pairs: Vec<String>,
    pub phases: Vec<Phase>,
}

/// The compilers and the built Tide stds a run uses.
pub struct Toolchains {
    pub pinned: PathBuf,
    pub stable: PathBuf,
    pub std_dev: PathBuf,
    pub std_release: PathBuf,
}

fn options() -> Result<Options, String> {
    let mut options = Options {
        quick: false,
        out: None,
        pairs: Vec::new(),
        phases: Vec::new(),
    };
    let mut args = std::env::args().skip(1);
    while let Some(arg) = args.next() {
        match arg.as_str() {
            // `cargo bench` passes `--bench` to every bench target.
            "--bench" => {}
            "--quick" => options.quick = true,
            "--out" => {
                let dir = PathBuf::from(args.next().ok_or("--out needs a directory")?);
                // Cargo runs a bench in its package's directory, not the caller's.
                if !dir.is_absolute() {
                    return Err("--out needs an absolute path".into());
                }
                options.out = Some(dir);
            }
            "--pair" => options
                .pairs
                .push(args.next().ok_or("--pair needs a name")?),
            "--phase" => {
                let name = args.next().ok_or("--phase needs a name")?;
                let phase = PHASES
                    .into_iter()
                    .find(|p| p.name() == name)
                    .ok_or_else(|| format!("no phase `{name}`"))?;
                options.phases.push(phase);
            }
            _ => return Err(format!("unknown argument `{arg}`")),
        }
    }
    Ok(options)
}

fn main() -> ExitCode {
    let options = match options() {
        Ok(options) => options,
        Err(e) => {
            eprintln!(
                "pairs: {e}\nusage: cargo bench -p tide-admit --bench pairs -- [--quick] [--out DIR] [--pair NAME]... [--phase NAME]..."
            );
            return ExitCode::from(2);
        }
    };
    match Bench::start(options).and_then(|mut bench| bench.run()) {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("pairs: {e}");
            ExitCode::FAILURE
        }
    }
}

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

/// The tool's own directory under Cargo's temporary directory; its builds go to `build/bench-pairs`
/// there, beside the test suite's packages.
fn scratch() -> PathBuf {
    PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join("bench-pairs")
}

fn now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |d| d.as_secs())
}

/// The first line of what `command` prints, or why it printed nothing.
fn first_line(command: &mut Command) -> String {
    output(command).lines().next().unwrap_or("").to_owned()
}

/// What `command` prints to standard output, trimmed, or why it failed.
fn output(command: &mut Command) -> String {
    match command.stdin(Stdio::null()).stderr(Stdio::null()).output() {
        Ok(out) if out.status.success() => String::from_utf8_lossy(&out.stdout).trim().to_owned(),
        Ok(out) => format!("unavailable ({})", out.status),
        Err(e) => format!("unavailable ({e})"),
    }
}

/// How `rustc` links for the host by default, from its target spec: the linker flavor and
/// whether its bundled linker is on. `RUSTC_BOOTSTRAP` only unlocks this print on stable.
fn rust_linker(rustc: &Path) -> String {
    let spec = output(Command::new(rustc).env("RUSTC_BOOTSTRAP", "1").args([
        "-Zunstable-options",
        "--print",
        "target-spec-json",
    ]));
    let Some((_, flavor)) = spec.split_once("\"linker-flavor\":") else {
        return spec;
    };
    let flavor = flavor.split('"').nth(1).unwrap_or("unknown");
    let bundled = spec
        .split_once("\"link-self-contained\":")
        .and_then(|(_, rest)| rest.split('}').next())
        .is_some_and(|block| block.contains("\"linker\""));
    format!(
        "linker-flavor {flavor}, bundled linker {}",
        if bundled { "on" } else { "off" }
    )
}

/// The `rustc` in the sysroot of the toolchain that `command` (a `rustc`) selects, so measured
/// builds run the compiler itself, not rustup's proxy.
pub fn sysroot_rustc(command: &mut Command) -> Result<PathBuf, String> {
    let out = command
        .args(["--print", "sysroot"])
        .stdin(Stdio::null())
        .output()
        .map_err(|e| format!("cannot run {}: {e}", command.get_program().display()))?;
    if !out.status.success() {
        return Err(format!(
            "{command:?} failed: {}",
            String::from_utf8_lossy(&out.stderr)
        ));
    }
    let rustc = Path::new(String::from_utf8_lossy(&out.stdout).trim()).join("bin/rustc");
    match rustc.is_file() {
        true => Ok(rustc),
        false => Err(format!("no compiler at {}", rustc.display())),
    }
}

pub struct Bench {
    quick: bool,
    out: PathBuf,
    pairs: Vec<String>,
    phases: Vec<Phase>,
    records: LineWriter<File>,
    pinned: PathBuf,
    stable: PathBuf,
    std_dev: PathBuf,
    std_release: PathBuf,
    unstable: Vec<String>,
}

/// One series' measurements: wall seconds and peak RSS in KiB; the built binaries' file and
/// loaded sizes in bytes (builds and rebuilds); for a simulated arm, its traced events; and for
/// a test build, whether its check that a sampled clause runs passed.
#[derive(Default)]
struct Series {
    wall: Vec<f64>,
    rss: Vec<f64>,
    bytes: Vec<f64>,
    loaded: Vec<f64>,
    events: Option<u64>,
    sampled_checked: bool,
}

impl Bench {
    /// Resolves the toolchains and builds Tide's std in both profiles, then opens the run.
    fn start(options: Options) -> Result<Bench, String> {
        Self::pairs(&options)?;
        Self::out(&options)?;
        // The pinned toolchain as the equivalence check selects it: `RUSTC`, else the
        // directory's `rust-toolchain.toml`.
        let pinned = sysroot_rustc(
            Command::new(std::env::var_os("RUSTC").unwrap_or("rustc".into()))
                .current_dir(env!("CARGO_MANIFEST_DIR")),
        )?;
        let stable = sysroot_rustc(
            Command::new("rustup")
                .env_remove("RUSTUP_TOOLCHAIN")
                .args(["run", "stable", "rustc"]),
        )?;
        println!("building Tide's std in Cargo's dev and release profiles");
        let std_dev = build_std(&scratch().join("std-dev"), false);
        let std_release = build_std(&scratch().join("std-release"), true);
        Self::open(
            options,
            Toolchains {
                pinned,
                stable,
                std_dev,
                std_release,
            },
        )
    }

    /// The results directory: `--out`, else a new one under the scratch directory. It must be
    /// new or empty.
    fn out(options: &Options) -> Result<PathBuf, String> {
        let mode = if options.quick { "quick" } else { "full" };
        let out = options
            .out
            .clone()
            .unwrap_or_else(|| scratch().join("results").join(format!("{}-{mode}", now())));
        match fs::read_dir(&out).is_ok_and(|mut entries| entries.next().is_some()) {
            true => Err(format!(
                "{} is not empty; records go to a new directory",
                out.display()
            )),
            false => Ok(out),
        }
    }

    /// Opens a run with `toolchains`: creates the results directory and writes the header.
    pub fn open(options: Options, toolchains: Toolchains) -> Result<Bench, String> {
        let pairs = Self::pairs(&options)?;
        let out = Self::out(&options)?;
        let mode = if options.quick { "quick" } else { "full" };
        let Toolchains {
            pinned,
            stable,
            std_dev,
            std_release,
        } = toolchains;
        // Only now, so a start that fails leaves no results directory behind.
        fs::create_dir_all(&out).map_err(|e| format!("{}: {e}", out.display()))?;
        let records = File::create(out.join("records.jsonl"))
            .map(LineWriter::new)
            .map_err(|e| format!("{}: {e}", out.display()))?;
        let mut bench = Bench {
            quick: options.quick,
            out,
            pairs,
            phases: match options.phases.is_empty() {
                true => PHASES.to_vec(),
                false => options.phases,
            },
            records,
            pinned,
            stable,
            std_dev,
            std_release,
            unstable: Vec::new(),
        };
        bench.header(mode)?;
        Ok(bench)
    }

    /// The pairs to measure: every directory in `benchmarks/`, or those `--pair` names.
    fn pairs(options: &Options) -> Result<Vec<String>, String> {
        let dir = root().join("benchmarks");
        let mut all: Vec<String> = fs::read_dir(&dir)
            .map_err(|e| format!("{}: {e}", dir.display()))?
            .filter_map(Result::ok)
            .filter(|e| e.file_type().is_ok_and(|t| t.is_dir()))
            .map(|e| e.file_name().to_string_lossy().into_owned())
            .collect();
        all.sort();
        if let Some(missing) = options.pairs.iter().find(|p| !all.contains(p)) {
            return Err(format!("no pair `{missing}` in benchmarks/"));
        }
        if !options.pairs.is_empty() {
            all.retain(|p| options.pairs.contains(p));
        }
        if !options.quick
            && let Some(missing) = all.iter().find(|p| Self::size(p).is_none())
        {
            return Err(format!("pair `{missing}` has no timed size in TIMED"));
        }
        Ok(all)
    }

    fn size(pair: &str) -> Option<&'static str> {
        TIMED
            .iter()
            .find(|(p, _)| *p == pair)
            .map(|(_, size)| *size)
    }

    fn write(&mut self, record: &str) -> Result<(), String> {
        write_record(&mut self.records, record)
    }

    fn header(&mut self, mode: &str) -> Result<(), String> {
        let git = |args: &[&str]| output(Command::new("git").arg("-C").arg(root()).args(args));
        let dirty = !git(&["status", "--porcelain"]).is_empty();
        let cpu = fs::read_to_string("/proc/cpuinfo")
            .ok()
            .and_then(|info| {
                info.lines().find_map(|l| {
                    l.strip_prefix("model name")
                        .map(|v| v.trim_start_matches([' ', '\t', ':']).to_owned())
                })
            })
            .unwrap_or_else(|| "unknown".into());
        let read = |path: &str| {
            fs::read_to_string(path)
                .map_or_else(|e| format!("unavailable ({e})"), |s| s.trim().to_owned())
        };
        let configs: Vec<String> = CONFIGS
            .iter()
            .map(|c| {
                let (compiler, std) = match c.compiler {
                    Compiler::Driver { release_std } => {
                        ("tide-admit", Some(if release_std { "release" } else { "dev" }))
                    }
                    Compiler::Pinned => ("pinned rustc", None),
                    Compiler::Stable => ("stable rustc", None),
                };
                let flags: Vec<String> = c.flags.iter().map(|f| json_string(f)).collect();
                format!(
                    "{{\"id\":{},\"compiler\":{},\"std_profile\":{},\"flags\":[{}],\"informational\":{}}}",
                    json_string(c.id),
                    json_string(compiler),
                    std.map_or("null".into(), json_string),
                    flags.join(","),
                    c.informational()
                )
            })
            .collect();
        let sizes: Vec<String> = self
            .pairs
            .iter()
            .map(|p| {
                format!(
                    "{}:{}",
                    json_string(p),
                    json_string(self.arg(p).unwrap_or("default"))
                )
            })
            .collect();
        let vv = |rustc: &Path| output(Command::new(rustc).arg("-vV"));
        // SAFETY: `getpriority` reads this process's nice value (heavy.sh runs at 19) and
        // touches no memory of ours.
        let nice = unsafe { libc::getpriority(libc::PRIO_PROCESS, 0) };
        let record = format!(
            "\"record\":\"header\",\"mode\":{},\"started_unix\":{},\"revision\":{},\"dirty\":{dirty},\
             \"cpu\":{},\"cpus\":{},\"kernel\":{},\"loadavg\":{},\
             \"nice\":{nice},\"run_dir\":{},\
             \"pinned_rustc\":{},\"pinned_rustc_vv\":{},\"pinned_rust_linker\":{},\
             \"stable_rustc\":{},\"stable_rustc_vv\":{},\"stable_rust_linker\":{},\
             \"driver\":{},\"tide_linker\":{},\"tide_ld\":{},\"std_dev\":{},\"std_release\":{},\
             \"rounds\":{{\"start\":{},\"step\":{},\"cap\":{},\"cv_limit\":{}}},\"run_limit_s\":{RUN_LIMIT},\
             \"sizes\":{{{}}},\"configs\":[{}],\"phases\":{}",
            json_string(mode),
            now(),
            json_string(&git(&["rev-parse", "HEAD"])),
            json_string(&cpu),
            std::thread::available_parallelism().map_or(0, |n| n.get()),
            json_string(&read("/proc/sys/kernel/osrelease")),
            json_string(&read("/proc/loadavg")),
            json_string(&self.out.join("run").display().to_string()),
            json_string(&self.pinned.display().to_string()),
            json_string(&vv(&self.pinned)),
            json_string(&rust_linker(&self.pinned)),
            json_string(&self.stable.display().to_string()),
            json_string(&vv(&self.stable)),
            json_string(&rust_linker(&self.stable)),
            json_string(env!("CARGO_BIN_EXE_tide-admit")),
            json_string(&first_line(Command::new("/usr/bin/cc").arg("--version"))),
            json_string(&first_line(Command::new("/usr/bin/ld").arg("--version"))),
            json_string(&self.std_dev.display().to_string()),
            json_string(&self.std_release.display().to_string()),
            if self.quick { 1 } else { START },
            if self.quick { 0 } else { STEP },
            if self.quick { 1 } else { CAP },
            stats::CV_LIMIT,
            sizes.join(","),
            configs.join(","),
            self.phases(),
        );
        self.write(&record)?;
        println!("records: {}", self.out.join("records.jsonl").display());
        Ok(())
    }

    /// The header's `phases`: each phase's arms, and what a phase adds to its configurations.
    fn phases(&self) -> String {
        let arms = |phase: Phase| {
            let ids: Vec<String> = phase.arms().iter().map(|a| json_string(&a.id())).collect();
            format!("\"arms\":[{}]", ids.join(","))
        };
        format!(
            "{{\"measured\":[{}],\"build\":{{{}}},\"rebuild\":{{{},\"edit\":{},\"rustc_adds\":[{}]}},\"run\":{{{}}},\"sim\":{{{},\"pair\":{},\"size\":{},\"seed\":{SIM_SEED}}},\"test\":{{{},\"pair\":{},\"size\":{},\"sampled_check\":\"bench_test::sampled\"}}}}",
            self.phases
                .iter()
                .map(|p| json_string(p.name()))
                .collect::<Vec<_>>()
                .join(","),
            arms(Phase::Build),
            arms(Phase::Rebuild),
            json_string("let _edit: u64 = <round>; as the first line of main"),
            json_string("-Cincremental=<rebuild directory>/incremental"),
            arms(Phase::Run),
            arms(Phase::Sim),
            json_string(SIMULATED),
            json_string(self.phase_arg(SIMULATED, Phase::Sim).unwrap_or("default")),
            arms(Phase::Test),
            json_string(CONTRACTED),
            json_string(self.phase_arg(CONTRACTED, Phase::Test).unwrap_or("default")),
        )
    }

    pub fn run(&mut self) -> Result<(), String> {
        let measured = self.pairs.clone().iter().try_for_each(|pair| {
            let mut phases = Vec::new();
            for phase in self.phases.clone() {
                if phase.measures(pair) {
                    phases.push((phase, self.phase(pair, phase)?));
                }
            }
            phases
                .iter()
                .try_for_each(|(phase, series)| self.summarize(pair, *phase, series))
        });
        close(&mut self.records, measured)?;
        if !self.unstable.is_empty() {
            println!(
                "UNSTABLE (CV over 1% after {CAP} rounds): {}",
                self.unstable.join(", ")
            );
        }
        println!("records: {}", self.out.join("records.jsonl").display());
        Ok(())
    }

    /// Measures one phase of `pair` on each of its arms in interleaved rounds, adding rounds
    /// while any arm's wall times vary by more than 1%.
    fn phase(&mut self, pair: &str, phase: Phase) -> Result<Vec<Series>, String> {
        let arms = phase.arms();
        let mut series: Vec<Series> = arms.iter().map(|_| Series::default()).collect();
        match phase {
            Phase::Rebuild => arms.iter().try_for_each(|arm| self.warm(pair, arm.config))?,
            // A run of only these phases has no build phase to keep the programs.
            Phase::Run | Phase::Sim | Phase::Test => {
                for (arm, series) in arms.iter().zip(&mut series) {
                    match arm.harness {
                        Harness::Program => self.keep(pair, arm.config)?,
                        Harness::Sim => series.events = Some(self.build_harness(pair, arm)?),
                        Harness::Test => {
                            self.build_harness(pair, arm)?;
                            series.sampled_checked = true;
                        }
                    }
                }
            }
            Phase::Build => {}
        }
        let mut expected: Option<Vec<u8>> = None;
        let (mut done, mut next) = if self.quick { (0, 1) } else { (0, START) };
        while next > 0 {
            for (round, i) in interleaved(arms.len(), done..done + next) {
                let (arm, id) = (&arms[i], arms[i].id());
                let (measured, binary) = match phase {
                    Phase::Build => self.build(pair, arm.config)?,
                    Phase::Rebuild => self.rebuild(pair, arm.config, round)?,
                    Phase::Run | Phase::Sim | Phase::Test => {
                        (self.execute(pair, arm, self.phase_arg(pair, phase))?, None)
                    }
                };
                let mut extra = match phase {
                    Phase::Run | Phase::Sim | Phase::Test => format!(
                        ",\"size\":{}",
                        json_string(self.phase_arg(pair, phase).unwrap_or("default"))
                    ),
                    Phase::Build | Phase::Rebuild => String::new(),
                };
                if let Some(binary) = binary.filter(|_| measured.status.success()) {
                    let mut head = Vec::new();
                    let len = File::open(&binary)
                        .and_then(|file| {
                            let len = file.metadata()?.len();
                            file.take(ELF_HEAD).read_to_end(&mut head).map(|_| len)
                        })
                        .map_err(|e| format!("{}: {e}", binary.display()))?;
                    let (size, loaded) = binary_size(&head, len)
                        .ok_or_else(|| format!("{}: not a 64-bit ELF executable", binary.display()))?;
                    extra.push_str(&format!(",\"binary_bytes\":{size},\"loaded_bytes\":{loaded}"));
                    series[i].bytes.push(size as f64);
                    series[i].loaded.push(loaded as f64);
                }
                self.record(phase.name(), pair, &id, round, &extra, &measured)?;
                if matches!(phase, Phase::Run | Phase::Sim | Phase::Test) {
                    // A simulated run writes the program's output to standard error; a test
                    // build's run prints it among the test harness's lines.
                    let output = match arm.harness {
                        Harness::Program => measured.stdout.clone(),
                        Harness::Sim => measured.stderr.clone(),
                        Harness::Test => within(&measured.stdout, expected.as_deref()).to_vec(),
                    };
                    check_output(
                        &mut self.records,
                        Run {
                            pair,
                            config: &id,
                            round,
                            exit: &describe(measured.status).0,
                            stdout: &output,
                        },
                        &mut expected,
                    )?;
                }
                series[i].wall.push(measured.wall.as_secs_f64());
                series[i].rss.push(measured.peak_rss_kib as f64);
            }
            done += next;
            next = if self.quick {
                0
            } else {
                let walls: Vec<Vec<f64>> = series.iter().map(|s| s.wall.clone()).collect();
                more_rounds(&walls, done, STEP, CAP)
            };
        }
        Ok(series)
    }

    /// Writes one measurement's record; a failed process fails the whole run, after its
    /// record is written.
    fn record(
        &mut self,
        phase: &str,
        pair: &str,
        id: &str,
        round: usize,
        extra: &str,
        measured: &Measured,
    ) -> Result<(), String> {
        let (exit, failure) = describe(measured.status);
        let mut record = format!(
            "\"record\":{},\"pair\":{},\"config\":{},\"round\":{round}{extra},\"wall_s\":{:.6},\"peak_rss_kib\":{},\"exit\":{}",
            json_string(phase),
            json_string(pair),
            json_string(id),
            measured.wall.as_secs_f64(),
            measured.peak_rss_kib,
            exit
        );
        if let Some(failure) = &failure {
            let tail = tail(&measured.stderr, 4000);
            record.push_str(&format!(",\"stderr_tail\":{}", json_string(&tail)));
            self.write(&record)?;
            return Err(format!("{phase} of {pair} under {id} {failure}:\n{tail}"));
        }
        self.write(&record)
    }

    fn source(pair: &str, config: &Config) -> Result<String, String> {
        let source = root()
            .join("benchmarks")
            .join(pair)
            .join(match config.compiler {
                Compiler::Driver { .. } => "tide.rs",
                _ => "rust.rs",
            });
        fs::read_to_string(&source).map_err(|e| format!("{}: {e}", source.display()))
    }

    fn std(&self, release_std: bool) -> &Path {
        if release_std {
            &self.std_release
        } else {
            &self.std_dev
        }
    }

    /// The command that builds `config`'s arm in `dir`, from the source at `dir/package/src/demo.rs`
    /// (the driver) or `dir/<pair>.rs` (rustc), and the binary it writes. A rustc rebuild also
    /// keeps rustc's incremental cache in `dir/incremental`, as Cargo's dev profile does.
    fn command(&self, pair: &str, config: &Config, dir: &Path, rebuild: bool) -> (Command, PathBuf) {
        match config.compiler {
            Compiler::Driver { release_std } => {
                let mut command = driver_with(self.std(release_std), dir);
                command.args(config.flags);
                (command, dir.join("out/demo"))
            }
            Compiler::Pinned | Compiler::Stable => {
                let rustc = match config.compiler {
                    Compiler::Stable => &self.stable,
                    _ => &self.pinned,
                };
                let mut command = Command::new(rustc);
                command
                    .current_dir(dir)
                    .arg("--edition=2024")
                    .args(config.flags);
                if rebuild {
                    command.arg(format!("-Cincremental={}", dir.join("incremental").display()));
                }
                command
                    .arg("-o")
                    .arg(dir.join(pair))
                    .arg(dir.join(format!("{pair}.rs")));
                (command, dir.join(pair))
            }
        }
    }

    /// A new build directory `name` holding `pair`'s `source` where `command` reads it.
    fn fresh(name: &str, pair: &str, config: &Config, source: &str) -> Result<PathBuf, String> {
        match config.compiler {
            Compiler::Driver { .. } => Ok(package(name, source)),
            Compiler::Pinned | Compiler::Stable => {
                let dir = PathBuf::from(env!("CARGO_TARGET_TMPDIR"))
                    .join("build")
                    .join(name);
                let _ = fs::remove_dir_all(&dir);
                fs::create_dir_all(&dir)
                    .and_then(|()| fs::write(dir.join(format!("{pair}.rs")), source))
                    .map_err(|e| format!("{}: {e}", dir.display()))?;
                Ok(dir)
            }
        }
    }

    /// Builds `pair` under `config` in a fresh directory, so the driver's cache never hits, and
    /// keeps the first binary in `bin/<pair>/<config>` for the runs.
    fn build(&self, pair: &str, config: &Config) -> Result<(Measured, Option<PathBuf>), String> {
        let dir = Self::fresh(
            &format!("bench-pairs/{pair}/{}", config.id),
            pair,
            config,
            &Self::source(pair, config)?,
        )?;
        let (mut command, binary) = self.command(pair, config, &dir, false);
        let measured = measure(&mut command).map_err(|e| format!("cannot start the build: {e}"))?;
        let kept = self.binary(pair, config.id);
        if measured.status.success() && !kept.exists() {
            fs::create_dir_all(kept.parent().unwrap())
                .and_then(|_| fs::copy(&binary, &kept))
                .map_err(|e| format!("cannot keep {}: {e}", binary.display()))?;
        }
        Ok((measured, Some(binary)))
    }

    /// The directory where `pair`'s rebuilds under `config` keep their caches between rounds.
    fn rebuild_dir(pair: &str, config: &Config) -> String {
        format!("bench-pairs-rebuild/{pair}/{}", config.id)
    }

    /// Builds `pair` under `config` once, unmeasured, in the directory its rebuilds reuse, so
    /// they start with the caches warm: the driver's build cache and rustc's incremental cache.
    fn warm(&self, pair: &str, config: &Config) -> Result<(), String> {
        let dir = Self::fresh(
            &Self::rebuild_dir(pair, config),
            pair,
            config,
            &Self::source(pair, config)?,
        )?;
        let measured = measure(&mut self.command(pair, config, &dir, true).0)
            .map_err(|e| format!("cannot start the build: {e}"))?;
        match measured.status.success() {
            true => Ok(()),
            false => Err(format!(
                "the warm-up build of {pair} under {} failed:\n{}",
                config.id,
                tail(&measured.stderr, 4000)
            )),
        }
    }

    /// Rebuilds `pair` under `config` in its warmed directory after this round's one-line edit,
    /// which differs every round, so the driver's cache never hits and rustc's incremental cache
    /// holds the previous round's build.
    fn rebuild(
        &self,
        pair: &str,
        config: &Config,
        round: usize,
    ) -> Result<(Measured, Option<PathBuf>), String> {
        let dir = PathBuf::from(env!("CARGO_TARGET_TMPDIR"))
            .join("build")
            .join(Self::rebuild_dir(pair, config));
        let path = match config.compiler {
            Compiler::Driver { .. } => dir.join("package/src/demo.rs"),
            _ => dir.join(format!("{pair}.rs")),
        };
        let source = edited(&Self::source(pair, config)?, round)?;
        fs::write(&path, source).map_err(|e| format!("{}: {e}", path.display()))?;
        let (mut command, binary) = self.command(pair, config, &dir, true);
        let measured = measure(&mut command).map_err(|e| format!("cannot start the build: {e}"))?;
        Ok((measured, Some(binary)))
    }

    /// Builds `pair` under `config` once, unmeasured, unless the build phase already kept its
    /// binary, for a phase whose runs need it.
    fn keep(&self, pair: &str, config: &Config) -> Result<(), String> {
        if self.binary(pair, config.id).exists() {
            return Ok(());
        }
        let (built, _) = self.build(pair, config)?;
        match built.status.success() {
            true => Ok(()),
            false => Err(format!(
                "the build of {pair} under {} failed:\n{}",
                config.id,
                tail(&built.stderr, 4000)
            )),
        }
    }

    /// Builds the Tide arm of `pair` with `arm`'s module appended, as a test binary under its
    /// configuration, keeps it in `bin/<pair>/<arm>`, and runs its unmeasured check: for the
    /// simulator, the `events` test, returning how many events the scheduler traces in one
    /// simulated run; for a test build, the `sampled` test, which must pass (0 returned).
    fn build_harness(&self, pair: &str, arm: &Arm) -> Result<u64, String> {
        let Compiler::Driver { release_std } = arm.config.compiler else {
            return Err(format!("{} is not a Tide configuration", arm.config.id));
        };
        let module = match arm.harness {
            Harness::Sim => {
                let mut args = vec![pair];
                args.extend(self.phase_arg(pair, Phase::Sim));
                sim_module(&args)
            }
            _ => test_module(self.phase_arg(pair, Phase::Test)),
        };
        let source = Self::source(pair, arm.config)? + &module;
        let dir = package(&format!("bench-pairs/{pair}/{}", arm.id()), &source);
        let built = measure(
            driver_with(self.std(release_std), &dir)
                .args(arm.config.flags)
                .args(["--crate-type=lib", "--test"]),
        )
        .map_err(|e| format!("cannot start the build: {e}"))?;
        if !built.status.success() {
            return Err(format!(
                "the test-binary build of {pair} under {} failed:\n{}",
                arm.config.id,
                tail(&built.stderr, 4000)
            ));
        }
        let kept = self.binary(pair, &arm.id());
        fs::create_dir_all(kept.parent().unwrap())
            .and_then(|_| fs::copy(dir.join("out/demo-test"), &kept))
            .map_err(|e| format!("cannot keep {}: {e}", kept.display()))?;
        let check = match arm.harness {
            Harness::Sim => "bench_sim::events",
            _ => "bench_test::sampled",
        };
        let counted = measure(timed(RUN_LIMIT, &kept).args(["--exact", check, "--test-threads=1"]))
            .map_err(|e| format!("cannot start the run: {e}"))?;
        let text = String::from_utf8_lossy(&counted.stderr);
        let passed = String::from_utf8_lossy(&counted.stdout).contains("1 passed");
        match (counted.status.success(), arm.harness, text.trim().parse()) {
            (true, Harness::Sim, Ok(events)) => Ok(events),
            (true, Harness::Test, _) if passed => Ok(0),
            _ => Err(format!(
                "{check} of {pair} under {} failed ({:?}):\n{}",
                arm.id(),
                counted.status,
                tail(&counted.stderr, 4000)
            )),
        }
    }

    fn binary(&self, pair: &str, id: &str) -> PathBuf {
        self.out.join("bin").join(pair).join(id)
    }

    /// The size argument of `pair`'s runs: none in a quick run.
    fn arg(&self, pair: &str) -> Option<&'static str> {
        if self.quick { None } else { Self::size(pair) }
    }

    /// The size argument of `pair`'s runs in `phase`: none in a quick run; the simulator and
    /// test-build phases' own sizes; else the pair's timed size.
    fn phase_arg(&self, pair: &str, phase: Phase) -> Option<&'static str> {
        match phase {
            _ if self.quick => None,
            Phase::Sim => Some(SIM_SIZE),
            Phase::Test => Some(TEST_SIZE),
            _ => Self::size(pair),
        }
    }

    /// Runs `arm`'s binary of `pair` with `size` in a new empty directory: the built program,
    /// or for an arm in a test binary its `timed` test, whose size the tool built into it.
    fn execute(&self, pair: &str, arm: &Arm, size: Option<&str>) -> Result<Measured, String> {
        let id = arm.id();
        let cwd = self.out.join("run").join(format!("{pair}-{id}"));
        let _ = fs::remove_dir_all(&cwd);
        fs::create_dir_all(&cwd).map_err(|e| format!("{}: {e}", cwd.display()))?;
        let mut command = timed(RUN_LIMIT, &self.binary(pair, &id));
        match arm.module() {
            Some(module) => command.args(["--exact", &format!("{module}::timed"), "--test-threads=1"]),
            None => command.args(size),
        };
        command.current_dir(&cwd);
        let measured = measure(&mut command).map_err(|e| format!("cannot start the run: {e}"))?;
        let _ = fs::remove_dir_all(&cwd);
        Ok(measured)
    }

    /// Writes each arm's summary record and prints its line.
    fn summarize(&mut self, pair: &str, phase: Phase, series: &[Series]) -> Result<(), String> {
        let sized = matches!(phase, Phase::Build | Phase::Rebuild);
        println!("\n{pair} {}:", phase.name());
        println!(
            "  {:<24} {:>3} {:>12} {:>8} {:>14} {:>12} {:>12}  {}",
            "config", "n", "median (s)", "CV", "peak RSS (MiB)", "binary (KiB)", "loaded (KiB)", "stable"
        );
        for (arm, s) in phase.arms().iter().zip(series) {
            let id = arm.id();
            let wall = median(&s.wall).unwrap_or(f64::NAN);
            let rss = median(&s.rss).unwrap_or(f64::NAN);
            let (bytes, loaded) = (median(&s.bytes), median(&s.loaded));
            let spread = cv(&s.wall);
            let steady = (!self.quick).then(|| stable(&s.wall));
            if steady == Some(false) {
                self.unstable.push(format!("{pair} {} {id}", phase.name()));
            }
            let mut record = format!(
                "\"record\":\"summary\",\"pair\":{},\"phase\":{},\"config\":{},\"n\":{},\"median_s\":{wall:.6},\"cv\":{},\"median_peak_rss_kib\":{rss:.0},\"stable\":{}",
                json_string(pair),
                json_string(phase.name()),
                json_string(&id),
                s.wall.len(),
                spread.map_or("null".into(), |c| format!("{c:.6}")),
                steady.map_or("null".into(), |s| s.to_string()),
            );
            if sized {
                let number = |v: Option<f64>| v.map_or("null".into(), |v| format!("{v:.0}"));
                record.push_str(&format!(
                    ",\"median_binary_bytes\":{},\"median_loaded_bytes\":{}",
                    number(bytes),
                    number(loaded)
                ));
            }
            if let Some(events) = s.events {
                record.push_str(&format!(",\"trace_events\":{events}"));
            }
            if s.sampled_checked {
                record.push_str(",\"sampled_clause_ran\":true");
            }
            self.write(&record)?;
            let kib = |v: Option<f64>| v.map_or("-".into(), |v| format!("{:.0}", v / 1024.0));
            println!(
                "  {:<24} {:>3} {:>12.4} {:>8} {:>14.1} {:>12} {:>12}  {}{}{}",
                id,
                s.wall.len(),
                wall,
                spread.map_or("-".into(), |c| format!("{:.2}%", c * 100.0)),
                rss / 1024.0,
                kib(bytes),
                kib(loaded),
                match steady {
                    None => "-",
                    Some(true) => "yes",
                    Some(false) => "UNSTABLE",
                },
                if arm.config.informational() {
                    " (informational)"
                } else {
                    ""
                },
                match (s.events, s.sampled_checked) {
                    (Some(e), _) => format!(" ({e} trace events)"),
                    (None, true) => " (sampled clause ran)".into(),
                    (None, false) => String::new(),
                },
            );
        }
        Ok(())
    }
}

/// Appends `record`, the inside of one JSON object, as a line of `records.jsonl`.
fn write_record(records: &mut impl Write, record: &str) -> Result<(), String> {
    writeln!(records, "{{{record}}}").map_err(|e| format!("records.jsonl: {e}"))
}

/// The last `chars` characters of `bytes`, read as UTF-8 with replacement.
fn tail(bytes: &[u8], chars: usize) -> String {
    let text = String::from_utf8_lossy(bytes);
    let skip = text.chars().count().saturating_sub(chars);
    text.chars().skip(skip).collect()
}

/// One run whose output the tool checks.
pub struct Run<'a> {
    pub pair: &'a str,
    pub config: &'a str,
    pub round: usize,
    /// The run's `exit` value as its record has it.
    pub exit: &'a str,
    pub stdout: &'a [u8],
}

/// Checks a run's standard output against the pair's first run, which `expected` keeps. On a
/// mismatch it appends an `error` record, with the run's exit status and short tails of both
/// outputs, and returns the error.
pub fn check_output(
    records: &mut impl Write,
    run: Run,
    expected: &mut Option<Vec<u8>>,
) -> Result<(), String> {
    let expected = expected.get_or_insert_with(|| run.stdout.to_vec());
    if expected == run.stdout {
        return Ok(());
    }
    let record = format!(
        "\"record\":\"error\",\"error\":\"output mismatch\",\"pair\":{},\"config\":{},\"round\":{},\"exit\":{},\"reference\":{},\"expected_tail\":{},\"actual_tail\":{}",
        json_string(run.pair),
        json_string(run.config),
        run.round,
        run.exit,
        json_string(CONFIGS[0].id),
        json_string(&tail(expected, 400)),
        json_string(&tail(run.stdout, 400)),
    );
    write_record(records, &record)?;
    Err(format!(
        "{}: {} printed other output than {} in round {}",
        run.pair, run.config, CONFIGS[0].id, run.round
    ))
}

/// Ends the records: the `end` record after a complete measurement, none after a failure, so a
/// dataset without `end` is incomplete.
pub fn close(records: &mut impl Write, measured: Result<(), String>) -> Result<(), String> {
    measured?;
    let record = format!(
        "\"record\":\"end\",\"finished_unix\":{},\"loadavg\":{}",
        now(),
        json_string(
            fs::read_to_string("/proc/loadavg")
                .unwrap_or_default()
                .trim()
        )
    );
    write_record(records, &record)
}

/// The record's `exit` value, and what went wrong if the process failed.
fn describe(status: ExitStatus) -> (String, Option<String>) {
    use std::os::unix::process::ExitStatusExt;
    match (status.code(), status.signal()) {
        (Some(0), _) => ("0".into(), None),
        (Some(code), _) => (code.to_string(), Some(format!("exited with {code}"))),
        (None, signal) => (
            "null".into(),
            Some(format!("was killed by signal {}", signal.unwrap_or(0))),
        ),
    }
}
