//! Tide's conformance runner: each case under `conformance/cases/` is checked with stable rustc
//! and with the admission driver, both against the stub Tide std in `conformance/rustc/`, and
//! their errors and diagnostics are compared with the case's marks. `conformance/README.md` owns
//! the mechanism, the case format, the verdicts, the requirements and the exit statuses.
//!
//! `cargo run -p tide-conformance -- [case ...]` checks cases (default: all of `cases/`), one line
//! each, then prints the counts; `cargo run -p tide-conformance -- --trace` lists the cases of each
//! rule in `rules.md`. Exit: 0 when every case is PASS or AWAITS CHECKER, 1 on any FAIL, 2 on a
//! setup error. The corpus is its own suite; `tests/runner.rs` tests the runner on a copied case.

use std::collections::{BTreeSet, HashSet};
use std::ffi::{OsStr, OsString};
use std::fmt;
use std::fs;
use std::io::Read;
use std::os::unix::process::ExitStatusExt;
use std::path::{Path, PathBuf};
use std::process::{Command, ExitCode, ExitStatus, Stdio};

// Cargo's artifact messages, read with Tide's std's strict decoder, as tide-demo reads them.
use std::collections;
#[allow(dead_code)]
#[path = "../../tide-std/src/codec.rs"]
mod __codec;
#[allow(dead_code)]
#[path = "../../tide-std/src/json.rs"]
mod json;

/// The `//@` lines a case may have after its rule line.
const DIRECTIVES: [&str; 2] = ["//@ grants: unsafe", "//@ profile: release"];
/// The independent float oracle runs with the interpreter its docstring names: the system one,
/// which loads the system MPFR through ctypes. See `python` for its flags.
const PYTHON: &str = "/usr/bin/python3";
const FLOAT_ORACLE: &str = "conformance/oracles/float_math.py";
/// The case whose expectations the float oracle generates and checks.
const FLOAT_CASE: &str = "conformance/cases/authority/float_math.rs";

fn main() -> ExitCode {
    let args: Vec<OsString> = std::env::args_os().skip(1).collect();
    match run(&args) {
        Ok(failed) => ExitCode::from(u8::from(failed)),
        Err(setup) => {
            eprintln!("{setup}");
            ExitCode::from(2)
        }
    }
}

/// Runs the command line; `Ok(true)` when a case failed.
fn run(args: &[OsString]) -> Result<bool, Setup> {
    let suite = Suite::open(std::env::var_os("CARGO_TARGET_DIR"))?;
    if args.first().is_some_and(|arg| arg == "--trace") {
        print!("{}", suite.trace()?);
        return Ok(false);
    }
    let cases = if args.is_empty() {
        suite.cases()?
    } else {
        args.iter().map(|arg| suite.case(arg)).collect()
    };
    let checker = suite.build()?;
    let mut counts = Counts::default();
    for case in &cases {
        let report = checker.check(case)?;
        println!("{report}");
        counts.add(&report.verdict);
    }
    println!("{counts}");
    Ok(counts.fail > 0)
}

/// A setup error: the runner cannot check cases. Exit 2.
#[derive(Debug)]
struct Setup(String);

impl fmt::Display for Setup {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "conformance: {}", self.0)
    }
}

fn setup(message: impl fmt::Display) -> Setup {
    Setup(message.to_string())
}

#[derive(Debug, PartialEq)]
enum Verdict {
    Pass,
    /// Both checkers match the marks they decide; `TD` marks in these other codes remain.
    AwaitsChecker(BTreeSet<String>),
    /// `log` is the output of the checker that disagreed, if any.
    Fail {
        reason: String,
        log: String,
    },
}

/// One case's verdict; its `Display` is the case's line, with any log indented below it.
struct Report {
    case: String,
    verdict: Verdict,
}

impl fmt::Display for Report {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let case = &self.case;
        match &self.verdict {
            Verdict::Pass => write!(f, "PASS  {case}"),
            Verdict::AwaitsChecker(codes) => {
                write!(f, "AWAITS CHECKER  {case}: {}", join(codes))
            }
            Verdict::Fail { reason, log } => {
                write!(f, "FAIL  {case}: {reason}")?;
                lines(log)
                    .filter(|line| !line.is_empty())
                    .try_for_each(|line| write!(f, "\n      {line}"))
            }
        }
    }
}

#[derive(Default)]
struct Counts {
    pass: usize,
    awaits: usize,
    fail: usize,
}

impl Counts {
    fn add(&mut self, verdict: &Verdict) {
        match verdict {
            Verdict::Pass => self.pass += 1,
            Verdict::AwaitsChecker(_) => self.awaits += 1,
            Verdict::Fail { .. } => self.fail += 1,
        }
    }
}

impl fmt::Display for Counts {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let total = self.pass + self.awaits + self.fail;
        write!(
            f,
            "{total} cases: {} PASS, {} AWAITS CHECKER, {} FAIL",
            self.pass, self.awaits, self.fail
        )
    }
}

/// The corpus and where its builds go.
struct Suite {
    root: PathBuf,
    cases: PathBuf,
    /// The rule IDs: the first column of `rules.md`'s tables.
    ids: HashSet<String>,
    /// Their order in `rules.md`, for `--trace`.
    id_order: Vec<String>,
    /// The driver's target directory.
    driver_target: PathBuf,
    /// The directory for both stub builds, with the case outputs in its `conformance/`.
    target: PathBuf,
}

impl Suite {
    /// `target_dir` is `$CARGO_TARGET_DIR`: unset or empty, the driver builds in `<repo>/target`
    /// and the rest in `conformance/target`; otherwise both go there, the rest in its
    /// `conformance/`. A relative value resolves from the working directory.
    fn open(target_dir: Option<OsString>) -> Result<Suite, Setup> {
        // Physical paths: the driver rejects a package path that traverses a directory symlink.
        let root = fs::canonicalize(Path::new(env!("CARGO_MANIFEST_DIR")).join("../.."))
            .map_err(|e| setup(format!("cannot find the repository: {e}")))?;
        let conformance = root.join("conformance");
        let rules = conformance.join("rules.md");
        let text = fs::read_to_string(&rules).unwrap_or_default();
        let id_order: Vec<String> = lines(&text)
            .filter_map(rule_id)
            .map(str::to_owned)
            .collect();
        if id_order.is_empty() {
            return Err(setup(format!("no rule IDs in {}", rules.display())));
        }
        let (driver_target, target) = match target_dir.filter(|dir| !dir.is_empty()) {
            Some(dir) => {
                let dir = std::path::absolute(&dir).map_err(|e| {
                    setup(format!(
                        "CARGO_TARGET_DIR {}: {e}",
                        Path::new(&dir).display()
                    ))
                })?;
                (dir.clone(), dir.join("conformance"))
            }
            None => (root.join("target"), conformance.join("target")),
        };
        Ok(Suite {
            cases: conformance.join("cases"),
            root,
            ids: id_order.iter().cloned().collect(),
            id_order,
            driver_target,
            target,
        })
    }

    /// Every case, in byte order of its path.
    fn cases(&self) -> Result<Vec<PathBuf>, Setup> {
        fn walk(dir: &Path, found: &mut Vec<PathBuf>) -> std::io::Result<()> {
            for entry in fs::read_dir(dir)? {
                let entry = entry?;
                let path = entry.path();
                if path.as_os_str().as_encoded_bytes().ends_with(b".rs") {
                    found.push(path.clone());
                }
                if entry.file_type()?.is_dir() {
                    walk(&path, found)?;
                }
            }
            Ok(())
        }
        let mut found = Vec::new();
        walk(&self.cases, &mut found)
            .map_err(|e| setup(format!("cannot list {}: {e}", self.cases.display())))?;
        found.sort_by(|a, b| {
            a.as_os_str()
                .as_encoded_bytes()
                .cmp(b.as_os_str().as_encoded_bytes())
        });
        Ok(found)
    }

    /// A case named on the command line: an absolute path, or a path under `cases/`, with or
    /// without `.rs`.
    fn case(&self, arg: &OsStr) -> PathBuf {
        if Path::new(arg).is_absolute() {
            return arg.into();
        }
        let arg = arg.to_string_lossy();
        let stem = arg.strip_suffix(".rs").unwrap_or(&arg);
        self.cases.join(format!("{stem}.rs"))
    }

    /// A case's name: its path under `cases/`, or the path it was given as.
    fn name(&self, file: &Path) -> String {
        file.strip_prefix(&self.cases)
            .unwrap_or(file)
            .display()
            .to_string()
    }

    /// One line per rule ID: the ID, then each case that names it, or `-`.
    fn trace(&self) -> Result<String, Setup> {
        let mut pairs = Vec::new();
        for file in self.cases()? {
            let text = String::from_utf8_lossy(&fs::read(&file).unwrap_or_default()).into_owned();
            for id in rules_of(&text) {
                pairs.push((id.to_owned(), self.name(&file)));
            }
        }
        let mut out = String::new();
        for id in &self.id_order {
            let hits: String = pairs
                .iter()
                .filter(|(rule, _)| rule == id)
                .map(|(_, case)| format!(" {case}"))
                .collect();
            let hits = if hits.is_empty() { " -" } else { &hits };
            out.push_str(&format!("{id:<10}{hits}\n"));
        }
        Ok(out)
    }

    /// `program` from the stable toolchain, the oracle's, whatever toolchain runs the runner.
    fn stable(&self, program: &str) -> Command {
        self.rustup("stable", program)
    }

    /// `program` from the toolchain `toolchain`; `rustup run` selects it for the program and
    /// for every rustc and cargo the program starts.
    fn rustup(&self, toolchain: &str, program: &str) -> Command {
        let mut command = Command::new("rustup");
        command
            .current_dir(&self.root)
            .env_remove("RUSTUP_TOOLCHAIN")
            .args(["run", toolchain, program])
            .stdin(Stdio::null());
        command
    }

    /// Builds the stub std with stable for the oracle, and the driver and the stub std with the
    /// pinned nightly for the driver.
    fn build(self) -> Result<Checker, Setup> {
        let out = self.target.join("conformance");
        fs::create_dir_all(out.join("admit"))
            .map_err(|e| setup(format!("cannot create {}: {e}", out.display())))?;
        let stub = self.root.join("conformance/rustc/std/Cargo.toml");
        let built = |command: &mut Command, what: &str| match command.status() {
            Ok(status) if status.success() => Ok(()),
            Ok(status) => Err(setup(format!("cannot build {what} ({status})"))),
            Err(e) => Err(setup(format!("cannot run rustup: {e}"))),
        };
        built(
            self.stable("cargo")
                .args(["build", "--quiet", "--offline", "--manifest-path"])
                .arg(&stub)
                .arg("--target-dir")
                .arg(&self.target),
            "the stub std with stable",
        )?;
        let toolchain = pinned_toolchain(&self.root)?;
        built(
            self.rustup(&toolchain, "cargo")
                .args([
                    "build",
                    "--quiet",
                    "--offline",
                    "-p",
                    "tide-admit",
                    "--target-dir",
                ])
                .arg(&self.driver_target),
            "the driver",
        )?;
        let driver = self.driver_target.join("debug/tide-admit");
        // The TD codes the driver decides.
        let printed = Command::new(&driver)
            .arg("--tide-print-codes")
            .stdin(Stdio::null())
            .stderr(Stdio::inherit())
            .output()
            .map_err(|e| setup(format!("cannot run {}: {e}", driver.display())))?;
        let codes: HashSet<String> = String::from_utf8_lossy(&printed.stdout)
            .split_whitespace()
            .map(str::to_owned)
            .collect();
        if !printed.status.success() || codes.is_empty() {
            return Err(setup(format!(
                "{} --tide-print-codes listed no codes ({})",
                driver.display(),
                printed.status
            )));
        }
        // The stub std again, for the driver; each library it loads is in a directory that
        // Cargo's artifact messages name.
        let admit = self.target.join("admit");
        let messages = self
            .rustup(&toolchain, "cargo")
            .args(["build", "--quiet", "--offline", "--manifest-path"])
            .arg(&stub)
            .arg("--target-dir")
            .arg(&admit)
            .arg("--message-format=json-render-diagnostics")
            .stderr(Stdio::inherit())
            .output()
            .map_err(|e| setup(format!("cannot run rustup: {e}")))?;
        if !messages.status.success() {
            return Err(setup(format!(
                "cannot build the stub std with {toolchain} ({})",
                messages.status
            )));
        }
        let mut admit_deps = BTreeSet::new();
        for line in messages
            .stdout
            .split(|&b| b == b'\n')
            .filter(|l| !l.is_empty())
        {
            let message: CargoMessage = json::decode(line)
                .map_err(|e| setup(format!("cargo wrote a message that is not JSON: {e}")))?;
            for file in message.filenames.into_iter().flatten().map(PathBuf::from) {
                let library = matches!(
                    file.extension().and_then(OsStr::to_str),
                    Some("rlib" | "so" | "dylib")
                );
                if let (true, Some(dir)) = (library, file.parent()) {
                    admit_deps.insert(dir.to_owned());
                }
            }
        }
        Ok(Checker {
            rlib: self.target.join("debug/libtide_std_stub.rlib"),
            deps: self.target.join("debug/deps"),
            admit_rlib: admit.join("debug/libtide_std_stub.rlib"),
            admit_deps,
            driver,
            codes,
            float_case: self.root.join(FLOAT_CASE),
            out,
            suite: self,
        })
    }
}

/// A Cargo message, as `--message-format=json` writes one per line; other fields are ignored.
struct CargoMessage {
    filenames: Option<Vec<String>>,
}

impl __codec::Decode for CargoMessage {
    fn __decode(input: &mut __codec::Decoder<'_>) -> Result<Self, json::Error> {
        let mut filenames = None;
        input.object_begin()?;
        while let Some(index) = input.next_field(&["filenames"])? {
            match index {
                0 => input.field(&mut filenames, "filenames")?,
                _ => input.skip_value()?,
            }
        }
        Ok(CargoMessage {
            filenames: input.required(filenames, "filenames")?,
        })
    }
}

/// The toolchain `rust-toolchain.toml` pins: its `channel`.
fn pinned_toolchain(root: &Path) -> Result<String, Setup> {
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

/// The built stub stds and driver.
struct Checker {
    suite: Suite,
    /// The stable stub std and its dependencies' directory.
    rlib: PathBuf,
    deps: PathBuf,
    /// The pinned-nightly stub std and every directory of a library it loads.
    admit_rlib: PathBuf,
    admit_deps: BTreeSet<PathBuf>,
    driver: PathBuf,
    /// The TD codes the driver decides.
    codes: HashSet<String>,
    float_case: PathBuf,
    /// The case outputs.
    out: PathBuf,
}

impl Checker {
    fn check(&self, file: &Path) -> Result<Report, Setup> {
        let case = self.suite.name(file);
        let report = |verdict| {
            Ok(Report {
                case: case.clone(),
                verdict,
            })
        };
        let fail = |reason: String, log: String| report(Verdict::Fail { reason, log });
        if !file.is_file() {
            return fail("no such case".into(), String::new());
        }
        let text = fs::read(file).map_err(|e| setup(format!("{}: {e}", file.display())))?;
        let text = String::from_utf8_lossy(&text);
        let rules = rules_of(&text);
        if rules.is_empty() {
            return fail("no '//@ rules:' first line".into(), String::new());
        }
        let bad: String = rules
            .iter()
            .filter(|id| !self.suite.ids.contains(**id))
            .map(|id| format!("{id} "))
            .collect();
        if !bad.is_empty() {
            return fail(format!("rule IDs not in rules.md: {bad}"), String::new());
        }
        let bad: String = lines(&text)
            .enumerate()
            .skip(1)
            .filter(|(_, line)| line.starts_with("//@") && !DIRECTIVES.contains(line))
            .map(|(index, _)| format!("{} ", index + 1))
            .collect();
        if !bad.is_empty() {
            return fail(format!("unknown '//@' lines: {bad}"), String::new());
        }
        let marks = Marks::parse(&text);
        if !marks.bad.is_empty() {
            let bad: String = marks.bad.iter().map(|mark| format!("{mark} ")).collect();
            return fail(format!("malformed marks: {bad}"), String::new());
        }
        // The TD marks the driver decides, as line:severity:code, and the codes it does not.
        let (driven, others): (BTreeSet<String>, BTreeSet<String>) = marks
            .td
            .iter()
            .cloned()
            .partition(|mark| self.codes.contains(code_of(mark)));
        let errors = driven.iter().any(|mark| mark.contains(":error:"));
        let awaits: BTreeSet<String> = others.iter().map(|mark| code_of(mark).to_owned()).collect();
        let kind = if lines(&text).any(is_main) {
            "bin"
        } else {
            "lib"
        };
        let name = crate_name(file);
        let path = file.to_string_lossy();
        let modes = [format!("--crate-type={kind}"), "--test".to_owned()];

        // Stable rustc takes -Zallow-features=, as the driver passes it (ADMIT-2), only under
        // RUSTC_BOOTSTRAP.
        let (mut log, mut code) = (String::new(), 0);
        for mode in &modes {
            let mut rustc = self.suite.stable("rustc");
            rustc
                .env("RUSTC_BOOTSTRAP", "1")
                .args(["-Zallow-features=", "--edition", "2024", "--extern"])
                .arg(prefixed("std=", &self.rlib))
                .arg("-L")
                .arg(prefixed("dependency=", &self.deps))
                .arg(mode)
                .args(["--crate-name", &name, "--emit", "metadata", "--out-dir"])
                .arg(&self.out)
                .arg("--error-format=short")
                .arg(file);
            let (status, output) = merged(rustc)?;
            append(&mut log, &output);
            if status != 0 {
                code = 1;
            }
        }
        let got = rustc_errors(&log, &path);
        if marks.want != got || (marks.want.is_empty() && code != 0) {
            let reason = format!(
                "rustc errors [{}], marked [{}], exit {code}",
                join(&got),
                join(&marks.want)
            );
            return fail(reason, log);
        }

        // The driver passes rustc -Zallow-features= itself (ADMIT-2) and takes the case's
        // grant and profile as flags.
        let (mut dlog, mut dcode) = (String::new(), 0);
        for mode in &modes {
            let mut driver = Command::new(&self.driver);
            if lines(&text).any(|line| line == DIRECTIVES[0]) {
                driver.arg("--tide-grant=unsafe");
            }
            if lines(&text).any(|line| line == DIRECTIVES[1]) {
                driver.arg("--tide-profile=release");
            }
            driver
                .arg(&modes[0])
                .args(["--edition", "2024", "--tide-std"])
                .arg(&self.admit_rlib);
            for dir in &self.admit_deps {
                driver.arg("--tide-dep-dir").arg(dir);
            }
            driver
                .arg(mode)
                .args(["--crate-name", &name, "--emit", "metadata", "--out-dir"])
                .arg(self.out.join("admit"))
                .arg("--error-format=short")
                .arg(file);
            let (status, output) = merged(driver)?;
            append(&mut dlog, &output);
            dcode = dcode.max(status);
        }
        // Exit 1 is a rejection; any other failure, or a panic, is the driver's crash.
        if dcode > 1 || dlog.contains("internal compiler error") || lines(&dlog).any(panicked) {
            return fail(format!("the driver crashed (exit {dcode})"), dlog);
        }
        let dgot = driver_diagnostics(&dlog, &path);
        // The driver must report exactly the marks it decides, and reject exactly the cases
        // that rustc or those marks reject.
        let rejects = !marks.want.is_empty() || errors;
        if driven != dgot || rejects == (dcode == 0) {
            let reason = format!(
                "driver diagnostics [{}], marked [{}], exit {dcode}",
                join(&dgot),
                join(&driven)
            );
            return fail(reason, dlog);
        }

        if fs::canonicalize(file).is_ok_and(|file| file == self.float_case) {
            if let Some(failure) = self.float_oracle()? {
                return report(failure);
            }
        }
        if awaits.is_empty() {
            report(Verdict::Pass)
        } else {
            report(Verdict::AwaitsChecker(awaits))
        }
    }

    /// Runs the independent float oracle, which checks the float case's generated section.
    fn float_oracle(&self) -> Result<Option<Verdict>, Setup> {
        float_oracle(&self.suite.root)
    }
}

/// The system interpreter, isolated (`-I`): no `PYTHON*` variable such as `PYTHONOPTIMIZE`,
/// which would strip the oracle's asserts, no user site-packages, and neither the script's
/// directory nor the working directory on `sys.path`. The oracle imports only the standard
/// library.
fn python() -> Command {
    let mut command = Command::new(PYTHON);
    command.arg("-I").stdin(Stdio::null());
    command
}

/// Runs `root`'s float oracle, which checks its float case's generated section. Without its
/// interpreter or MPFR the runner cannot check that case: a setup error.
fn float_oracle(root: &Path) -> Result<Option<Verdict>, Setup> {
    let probe = python()
        .args(["-c", "import ctypes; ctypes.CDLL('libmpfr.so.6')"])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status();
    if !probe.is_ok_and(|status| status.success()) {
        return Err(setup(format!(
            "{FLOAT_ORACLE} needs {PYTHON} with MPFR (libmpfr.so.6)"
        )));
    }
    let mut oracle = python();
    oracle.current_dir(root).arg(FLOAT_ORACLE);
    let (status, log) = merged(oracle)?;
    Ok((status != 0).then(|| Verdict::Fail {
        reason: format!("the independent oracle {FLOAT_ORACLE} exited {status}"),
        log,
    }))
}

/// A case's marks: `want` as line:code for rustc, `td` as line:severity:code for the checker,
/// and each malformed one as line:token.
#[derive(Debug, Default, PartialEq)]
struct Marks {
    want: BTreeSet<String>,
    td: BTreeSet<String>,
    bad: Vec<String>,
}

impl Marks {
    /// Each `//~` comment's codes. `WARN` switches the remaining TD codes on its line from
    /// errors to warnings.
    fn parse(text: &str) -> Marks {
        let mut marks = Marks::default();
        for (index, line) in lines(text).enumerate() {
            let number = index + 1;
            let Some(at) = line.find("//~") else {
                continue;
            };
            let tokens: Vec<&str> = line[at + 3..]
                .split([' ', '\t'])
                .filter(|token| !token.is_empty())
                .collect();
            if tokens.is_empty() {
                marks.bad.push(format!("{number}:(empty)"));
            }
            let mut severity = "error";
            for (k, token) in tokens.iter().enumerate() {
                if *token == "WARN" && severity == "error" && k + 1 < tokens.len() {
                    severity = "warning";
                } else if is_code(token, "TD") {
                    marks.td.insert(format!("{number}:{severity}:{token}"));
                } else if severity == "error" && (is_code(token, "E") || *token == "error") {
                    marks.want.insert(format!("{number}:{token}"));
                } else {
                    marks.bad.push(format!("{number}:{token}"));
                }
            }
        }
        marks
    }
}

/// Whether `token` is `prefix` and four digits.
fn is_code(token: &str, prefix: &str) -> bool {
    token
        .strip_prefix(prefix)
        .is_some_and(|digits| digits.len() == 4 && digits.bytes().all(|b| b.is_ascii_digit()))
}

/// The code of a line:severity:code mark or diagnostic.
fn code_of(mark: &str) -> &str {
    mark.rsplit_once(':').map_or(mark, |(_, code)| code)
}

/// The rule IDs the first line names: `//@ rules: ID, ID`.
fn rules_of(text: &str) -> Vec<&str> {
    lines(text)
        .next()
        .and_then(|line| line.strip_prefix("//@ rules:"))
        .map(|rest| rest.split([',', ' ']).filter(|id| !id.is_empty()).collect())
        .unwrap_or_default()
}

/// The ID in a table row of `rules.md` whose first cell is `PREFIX-N`; the prefix may have
/// hyphenated parts (`NET-HTTP-1`).
fn rule_id(line: &str) -> Option<&str> {
    let rest = line.strip_prefix('|')?.trim_start_matches(' ');
    let (id, after) = rest.split_at(rest.find([' ', '|'])?);
    after.trim_start_matches(' ').strip_prefix('|')?;
    let (prefix, number) = id.rsplit_once('-')?;
    let well_formed = prefix.split('-').all(|part| {
        part.starts_with(|c: char| c.is_ascii_uppercase())
            && part.bytes().all(|b| b.is_ascii_uppercase() || b.is_ascii_digit())
    })
        && !number.is_empty()
        && number.bytes().all(|b| b.is_ascii_digit());
    well_formed.then_some(id)
}

/// A line that starts with `fn main` as a word: the case is a binary.
fn is_main(line: &str) -> bool {
    line.strip_prefix("fn main")
        .is_some_and(|rest| !rest.starts_with(|c: char| c.is_alphanumeric() || c == '_'))
}

/// The file's name without `.rs`, with every byte that is not ASCII alphanumeric or `_` as `_`.
fn crate_name(file: &Path) -> String {
    let base = file.file_name().unwrap_or_default().as_encoded_bytes();
    let base = match base.strip_suffix(b".rs") {
        Some(stem) if !stem.is_empty() => stem,
        _ => base,
    };
    base.iter()
        .map(|&b| {
            if b.is_ascii_alphanumeric() || b == b'_' {
                b as char
            } else {
                '_'
            }
        })
        .collect()
}

/// Each rustc error as line:code in the case, path:line:code elsewhere, `?:error` if
/// unlocated; the code is `error` for an uncoded error.
fn rustc_errors(log: &str, file: &str) -> BTreeSet<String> {
    let mut errors = BTreeSet::new();
    for line in lines(log) {
        if line.starts_with("error: aborting due to") {
            continue;
        }
        let located = located(line, &["error"], |_, rest| {
            if rest.starts_with(':') {
                return Some("error".to_owned());
            }
            let number = rest.strip_prefix("[E")?;
            let end = digits(number)?;
            number[end..]
                .starts_with("]:")
                .then(|| format!("E{}", &number[..end]))
        });
        if let Some((path, number, code)) = located {
            errors.insert(format!("{}{number}:{code}", place(path, file)));
        } else if line.starts_with("error") {
            errors.insert("?:error".into());
        }
    }
    errors
}

/// Each driver diagnostic in a TD code, with its severity, as line:severity:code in the case
/// and path:line:severity:code elsewhere.
fn driver_diagnostics(log: &str, file: &str) -> BTreeSet<String> {
    let located = |line| {
        located(line, &["error", "warning"], |severity, rest| {
            let rest = rest.strip_prefix(": ")?;
            let code = rest.get(..6).filter(|code| is_code(code, "TD"))?;
            rest[6..]
                .starts_with(':')
                .then(|| format!("{severity}:{code}"))
        })
    };
    lines(log)
        .filter_map(located)
        .map(|(path, number, mark)| format!("{}{number}:{mark}", place(path, file)))
        .collect()
}

/// The leftmost `:LINE:COL: SEVERITY` in `line` that `then` accepts, given the severity word
/// and what follows it: the path before it, `LINE`, and what `then` returned.
fn located<'a, T>(
    line: &'a str,
    severities: &[&'static str],
    then: impl Fn(&'static str, &'a str) -> Option<T>,
) -> Option<(&'a str, &'a str, T)> {
    line.match_indices(':').find_map(|(at, _)| {
        let rest = &line[at + 1..];
        let (number, rest) = rest.split_at(digits(rest)?);
        let rest = rest.strip_prefix(':')?;
        let rest = rest[digits(rest)?..].strip_prefix(": ")?;
        let value = severities
            .iter()
            .find_map(|word| then(word, rest.strip_prefix(word)?))?;
        Some((&line[..at], number, value))
    })
}

/// The length of the run of ASCII digits that starts `text`, if it is not empty.
fn digits(text: &str) -> Option<usize> {
    let end = text
        .find(|c: char| !c.is_ascii_digit())
        .unwrap_or(text.len());
    (end > 0).then_some(end)
}

/// The prefix for a diagnostic at `path`: none in the case itself, else `path:`.
fn place(path: &str, file: &str) -> String {
    if path == file {
        String::new()
    } else {
        format!("{path}:")
    }
}

/// Whether a line reports a panicking thread: `thread '...' panicked`.
fn panicked(line: &str) -> bool {
    line.find("thread '")
        .is_some_and(|at| line[at + 8..].contains("' panicked"))
}

/// Runs `command` with standard output and error into one pipe, in the order written; returns
/// its exit code (128 + the signal for a killed process) and its output.
fn merged(mut command: Command) -> Result<(i32, String), Setup> {
    let program = command.get_program().to_owned();
    let cannot = |e: std::io::Error| setup(format!("cannot run {}: {e}", program.display()));
    let (mut reader, writer) = std::io::pipe().map_err(cannot)?;
    let copy = writer.try_clone().map_err(cannot)?;
    command.stdin(Stdio::null()).stdout(writer).stderr(copy);
    let mut child = command.spawn().map_err(cannot)?;
    // The command holds the pipe's write ends; reading ends only once they are closed.
    drop(command);
    let mut output = Vec::new();
    reader.read_to_end(&mut output).map_err(cannot)?;
    let status = child.wait().map_err(cannot)?;
    Ok((
        exit_code(status),
        String::from_utf8_lossy(&output).into_owned(),
    ))
}

fn exit_code(status: ExitStatus) -> i32 {
    status
        .code()
        .unwrap_or_else(|| 128 + status.signal().unwrap_or(0))
}

/// Appends one run's output to a case's log, as one block that ends in a newline.
fn append(log: &mut String, output: &str) {
    log.push_str(output.trim_end_matches('\n'));
    log.push('\n');
}

/// `prefix` followed by `path`, as one argument.
fn prefixed(prefix: &str, path: &Path) -> OsString {
    let mut arg = OsString::from(prefix);
    arg.push(path);
    arg
}

/// The lines of `text`, split at `\n` only.
fn lines(text: &str) -> impl Iterator<Item = &str> {
    text.split_terminator('\n')
}

fn join<'a>(items: impl IntoIterator<Item = &'a String>) -> String {
    items
        .into_iter()
        .map(String::as_str)
        .collect::<Vec<_>>()
        .join(" ")
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A copy of the float oracle and its fixture with one generated constant changed fails,
    /// even with `PYTHONOPTIMIZE=1` in the environment the oracle inherits, which would strip
    /// its asserts; the unchanged copy passes under it. Needs `/usr/bin/python3` with MPFR, as
    /// the corpus does.
    #[test]
    fn a_stale_float_fixture_fails_the_oracle() {
        struct Work(PathBuf);
        impl Drop for Work {
            fn drop(&mut self) {
                let _ = fs::remove_dir_all(&self.0);
            }
        }
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
        let work =
            Work(std::env::temp_dir().join(format!("tide-float-oracle-{}", std::process::id())));
        let copy = |path: &str, text: &str| {
            let to = work.0.join(path);
            fs::create_dir_all(to.parent().unwrap()).unwrap();
            fs::write(to, text).unwrap();
        };
        let fixture = fs::read_to_string(root.join(FLOAT_CASE)).unwrap();
        copy(
            FLOAT_ORACLE,
            &fs::read_to_string(root.join(FLOAT_ORACLE)).unwrap(),
        );
        copy(FLOAT_CASE, &fixture);
        let run = || {
            let mut oracle = python();
            oracle
                .current_dir(&work.0)
                .arg(FLOAT_ORACLE)
                .env("PYTHONOPTIMIZE", "1");
            merged(oracle).unwrap()
        };
        let (status, log) = run();
        assert_eq!(status, 0, "the unchanged copy fails:\n{log}");

        // The first constant of the generated section, one bit off.
        let section = fixture.find("// BEGIN independent oracle").unwrap();
        let constant = section + fixture[section..].find("0x3ef57744").unwrap();
        let stale = format!(
            "{}0x3ef57745{}",
            &fixture[..constant],
            &fixture[constant + 10..]
        );
        copy(FLOAT_CASE, &stale);
        let (status, log) = run();
        assert_ne!(status, 0, "the stale copy passes:\n{log}");
        assert!(log.contains("oracle output differs"), "{log}");
        assert!(matches!(
            float_oracle(&work.0),
            Ok(Some(Verdict::Fail { .. }))
        ));
    }

    /// WARN applies to the TD codes after it, and only before a final code; every other token
    /// is malformed.
    #[test]
    fn marks() {
        let text = "//@ rules: X-1\n\
                    f(); //~ E0425 TD0501\n\
                    g(); //~ TD0507 WARN TD0607\n\
                    h(); //~ error WARN E0308 TD0101\n\
                    i(); //~\n\
                    j(); //~ WARN\n\
                    k(); //~ TD05 E04250 WARN WARN TD0001\n";
        let set = |items: &[&str]| items.iter().map(|s| s.to_string()).collect();
        assert_eq!(
            Marks::parse(text),
            Marks {
                want: set(&["2:E0425", "4:error"]),
                td: set(&[
                    "2:error:TD0501",
                    "3:error:TD0507",
                    "3:warning:TD0607",
                    "4:warning:TD0101",
                    "7:warning:TD0001"
                ]),
                bad: vec![
                    "4:E0308",
                    "5:(empty)",
                    "6:WARN",
                    "7:TD05",
                    "7:E04250",
                    "7:WARN"
                ]
                .into_iter()
                .map(String::from)
                .collect(),
            }
        );
    }

    /// Located diagnostics in the case lose their path, others keep it; rustc's uncoded and
    /// unlocated errors get `error` and `?`.
    #[test]
    fn diagnostics() {
        let rustc = "/c/a.rs:4:5: error[E0425]: cannot find function\n\
                     /s/lib.rs:1:1: error: lifetime may not live long enough\n\
                     error: aborting due to 2 previous errors\n\
                     error: linking failed\n\
                     /c/a.rs:9:1: warning: unused\n";
        assert_eq!(
            rustc_errors(rustc, "/c/a.rs")
                .into_iter()
                .collect::<Vec<_>>(),
            ["/s/lib.rs:1:error", "4:E0425", "?:error"]
        );
        let driver = "/c/a.rs:4:5: error: TD0501: ambient\n\
                      /c/a.rs:7:2: warning: TD0607: payload\n\
                      /c/a.rs:8:2: error[E0599]: no method\n\
                      /c/a.rs:9:2: error: TD05: short\n";
        assert_eq!(
            driver_diagnostics(driver, "/c/a.rs")
                .into_iter()
                .collect::<Vec<_>>(),
            ["4:error:TD0501", "7:warning:TD0607"]
        );
    }
}
