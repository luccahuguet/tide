//! The benchmark rerun tool (fx-6u8m.6): builds and runs every pair in `benchmarks/` under each
//! configuration, measuring each build's wall time and peak memory and each run's wall time, and
//! writes one raw record per measurement to a results directory. `benchmarks/README.md` states
//! the configurations, the method, and the record format.
//!
//! `cargo bench -p tide-admit --bench pairs -- [--quick] [--out DIR] [--pair NAME]...`
//!
//! It is a `harness = false` bench target of the driver's package, so it builds Tide programs
//! with the same helpers and the same driver binary as the equivalence check
//! (`tests/benchmarks.rs`), and Cargo builds that driver optimized, as rustc is.
use std::fs::{self, File};
use std::io::{LineWriter, Write};
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

struct Options {
    quick: bool,
    out: Option<PathBuf>,
    pairs: Vec<String>,
}

fn options() -> Result<Options, String> {
    let mut options = Options {
        quick: false,
        out: None,
        pairs: Vec::new(),
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
                "pairs: {e}\nusage: cargo bench -p tide-admit --bench pairs -- [--quick] [--out DIR] [--pair NAME]..."
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
fn sysroot_rustc(command: &mut Command) -> Result<PathBuf, String> {
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

struct Bench {
    quick: bool,
    out: PathBuf,
    pairs: Vec<String>,
    records: LineWriter<File>,
    pinned: PathBuf,
    stable: PathBuf,
    std_dev: PathBuf,
    std_release: PathBuf,
    unstable: Vec<String>,
}

/// One series' measurements: wall seconds and peak RSS in KiB.
#[derive(Default)]
struct Series {
    wall: Vec<f64>,
    rss: Vec<f64>,
}

impl Bench {
    fn start(options: Options) -> Result<Bench, String> {
        let pairs = Self::pairs(&options)?;
        let mode = if options.quick { "quick" } else { "full" };
        let out = options
            .out
            .unwrap_or_else(|| scratch().join("results").join(format!("{}-{mode}", now())));
        if fs::read_dir(&out).is_ok_and(|mut entries| entries.next().is_some()) {
            return Err(format!(
                "{} is not empty; records go to a new directory",
                out.display()
            ));
        }
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
        // Only now, so a start that fails leaves no results directory behind.
        fs::create_dir_all(&out).map_err(|e| format!("{}: {e}", out.display()))?;
        let records = File::create(out.join("records.jsonl"))
            .map(LineWriter::new)
            .map_err(|e| format!("{}: {e}", out.display()))?;
        let mut bench = Bench {
            quick: options.quick,
            out,
            pairs,
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
             \"sizes\":{{{}}},\"configs\":[{}]",
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
        );
        self.write(&record)?;
        println!("records: {}", self.out.join("records.jsonl").display());
        Ok(())
    }

    fn run(&mut self) -> Result<(), String> {
        let measured = self.pairs.clone().iter().try_for_each(|pair| {
            let builds = self.phase(pair, "build")?;
            let runs = self.phase(pair, "run")?;
            self.summarize(pair, "build", &builds)?;
            self.summarize(pair, "run", &runs)
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

    /// Measures one phase of `pair` under every configuration in interleaved rounds, adding
    /// rounds while any configuration's wall times vary by more than 1%.
    fn phase(&mut self, pair: &str, phase: &str) -> Result<Vec<Series>, String> {
        let mut series: Vec<Series> = CONFIGS.iter().map(|_| Series::default()).collect();
        let mut expected: Option<Vec<u8>> = None;
        let (mut done, mut next) = if self.quick { (0, 1) } else { (0, START) };
        while next > 0 {
            for (round, arm) in interleaved(CONFIGS.len(), done..done + next) {
                let config = &CONFIGS[arm];
                let measured = match phase {
                    "build" => self.build(pair, config)?,
                    _ => self.execute(pair, config)?,
                };
                let size = match phase {
                    "run" => format!(
                        ",\"size\":{}",
                        json_string(self.arg(pair).unwrap_or("default"))
                    ),
                    _ => String::new(),
                };
                self.record(phase, pair, config, round, &size, &measured)?;
                if phase == "run" {
                    check_output(
                        &mut self.records,
                        Run {
                            pair,
                            config: config.id,
                            round,
                            exit: &describe(measured.status).0,
                            stdout: &measured.stdout,
                        },
                        &mut expected,
                    )?;
                }
                series[arm].wall.push(measured.wall.as_secs_f64());
                series[arm].rss.push(measured.peak_rss_kib as f64);
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
        config: &Config,
        round: usize,
        extra: &str,
        measured: &Measured,
    ) -> Result<(), String> {
        let (exit, failure) = describe(measured.status);
        let mut record = format!(
            "\"record\":{},\"pair\":{},\"config\":{},\"round\":{round}{extra},\"wall_s\":{:.6},\"peak_rss_kib\":{},\"exit\":{}",
            json_string(phase),
            json_string(pair),
            json_string(config.id),
            measured.wall.as_secs_f64(),
            measured.peak_rss_kib,
            exit
        );
        if let Some(failure) = &failure {
            let tail = tail(&measured.stderr, 4000);
            record.push_str(&format!(",\"stderr_tail\":{}", json_string(&tail)));
            self.write(&record)?;
            return Err(format!(
                "{phase} of {pair} under {} {failure}:\n{tail}",
                config.id
            ));
        }
        self.write(&record)
    }

    /// Builds `pair` under `config` in a fresh directory, so the driver's cache never hits, and
    /// keeps the first binary in `bin/<pair>/<config>` for the runs.
    fn build(&self, pair: &str, config: &Config) -> Result<Measured, String> {
        let source = root()
            .join("benchmarks")
            .join(pair)
            .join(match config.compiler {
                Compiler::Driver { .. } => "tide.rs",
                _ => "rust.rs",
            });
        let name = format!("bench-pairs/{pair}/{}", config.id);
        let (mut command, binary) = match config.compiler {
            Compiler::Driver { release_std } => {
                let code = fs::read_to_string(&source)
                    .map_err(|e| format!("{}: {e}", source.display()))?;
                let dir = package(&name, &code);
                let std = if release_std {
                    &self.std_release
                } else {
                    &self.std_dev
                };
                let mut command = driver_with(std, &dir);
                command.args(config.flags);
                (command, dir.join("out/demo"))
            }
            Compiler::Pinned | Compiler::Stable => {
                let dir = PathBuf::from(env!("CARGO_TARGET_TMPDIR"))
                    .join("build")
                    .join(&name);
                let _ = fs::remove_dir_all(&dir);
                fs::create_dir_all(&dir).map_err(|e| format!("{}: {e}", dir.display()))?;
                let rustc = match config.compiler {
                    Compiler::Stable => &self.stable,
                    _ => &self.pinned,
                };
                let mut command = Command::new(rustc);
                command
                    .current_dir(&dir)
                    .arg("--edition=2024")
                    .args(config.flags)
                    .arg("-o")
                    .arg(dir.join(pair))
                    .arg(&source);
                (command, dir.join(pair))
            }
        };
        let measured = measure(&mut command).map_err(|e| format!("cannot start the build: {e}"))?;
        let kept = self.binary(pair, config);
        if measured.status.success() && !kept.exists() {
            fs::create_dir_all(kept.parent().unwrap())
                .and_then(|()| fs::copy(&binary, &kept))
                .map_err(|e| format!("cannot keep {}: {e}", binary.display()))?;
        }
        Ok(measured)
    }

    fn binary(&self, pair: &str, config: &Config) -> PathBuf {
        self.out.join("bin").join(pair).join(config.id)
    }

    /// The size argument of `pair`'s runs: none in a quick run.
    fn arg(&self, pair: &str) -> Option<&'static str> {
        if self.quick { None } else { Self::size(pair) }
    }

    /// Runs `pair`'s binary built under `config` in a new empty directory.
    fn execute(&self, pair: &str, config: &Config) -> Result<Measured, String> {
        let cwd = self.out.join("run").join(format!("{pair}-{}", config.id));
        let _ = fs::remove_dir_all(&cwd);
        fs::create_dir_all(&cwd).map_err(|e| format!("{}: {e}", cwd.display()))?;
        let mut command = timed(RUN_LIMIT, &self.binary(pair, config));
        command.args(self.arg(pair)).current_dir(&cwd);
        let measured = measure(&mut command).map_err(|e| format!("cannot start the run: {e}"))?;
        let _ = fs::remove_dir_all(&cwd);
        Ok(measured)
    }

    /// Writes each configuration's summary record and prints its line.
    fn summarize(&mut self, pair: &str, phase: &str, series: &[Series]) -> Result<(), String> {
        println!("\n{pair} {phase}:");
        println!(
            "  {:<24} {:>3} {:>12} {:>8} {:>14}  {}",
            "config", "n", "median (s)", "CV", "peak RSS (MiB)", "stable"
        );
        for (config, s) in CONFIGS.iter().zip(series) {
            let wall = median(&s.wall).unwrap_or(f64::NAN);
            let rss = median(&s.rss).unwrap_or(f64::NAN);
            let spread = cv(&s.wall);
            let steady = (!self.quick).then(|| stable(&s.wall));
            if steady == Some(false) {
                self.unstable.push(format!("{pair} {phase} {}", config.id));
            }
            let record = format!(
                "\"record\":\"summary\",\"pair\":{},\"phase\":{},\"config\":{},\"n\":{},\"median_s\":{wall:.6},\"cv\":{},\"median_peak_rss_kib\":{rss:.0},\"stable\":{}",
                json_string(pair),
                json_string(phase),
                json_string(config.id),
                s.wall.len(),
                spread.map_or("null".into(), |c| format!("{c:.6}")),
                steady.map_or("null".into(), |s| s.to_string()),
            );
            self.write(&record)?;
            println!(
                "  {:<24} {:>3} {:>12.4} {:>8} {:>14.1}  {}{}",
                config.id,
                s.wall.len(),
                wall,
                spread.map_or("-".into(), |c| format!("{:.2}%", c * 100.0)),
                rss / 1024.0,
                match steady {
                    None => "-",
                    Some(true) => "yes",
                    Some(false) => "UNSTABLE",
                },
                if config.informational() {
                    " (informational)"
                } else {
                    ""
                }
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
