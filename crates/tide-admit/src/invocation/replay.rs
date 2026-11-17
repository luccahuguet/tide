//! Identity and execution boundary: never ask candidate code to identify itself.
use super::*;
use crate::replay as wire;
use crate::replay::{Build, Context, Manifest, Receipt, encoded};

impl Invocation {
    pub(super) fn receipt_path(&self) -> PathBuf {
        self.executable().with_extension("build.json")
    }

    pub(super) fn build_identity(
        &self,
        sources: &Sources,
        library: Option<(&Invocation, &Sources)>,
    ) -> Result<Build, String> {
        let mut dependencies = Vec::new();
        if let Some(std) = &self.std {
            dependencies.push(BTreeMap::from([("std".into(), file_digest(std)?)]));
        }
        // One map per dependency directory, filled from the libraries a build loads
        // (`loaded_identity`) or that a recording names (`current_artifacts`).
        dependencies.extend(self.deps.iter().map(|_| BTreeMap::new()));
        if self.link {
            let [cc, ld] = linker_tools();
            dependencies.push(BTreeMap::from([
                ("linker:cc".into(), file_digest(&cc)?),
                ("linker:ld".into(), file_digest(&ld)?),
            ]));
        }
        let mut options = self.semantic_options(library.is_some());
        if let Some((library, _)) = library {
            options.extend(
                library
                    .semantic_options(false)
                    .into_iter()
                    .map(|arg| format!("paired-library:{arg}")),
            );
        }
        Ok(Build {
            source: (*sources.logical).clone(),
            library: library
                .map(|(_, s)| (*s.logical).clone())
                .unwrap_or_default(),
            dependencies,
            compiler: env!("TIDE_COMPILER_ID").into(),
            target: env!("TIDE_COMPILER_ID")
                .split(';')
                .find_map(|line| line.strip_prefix("host: "))
                .ok_or("compiler target identity missing")?
                .into(),
            profile: self.profile.clone(),
            options,
            driver: file_digest(&std::env::current_exe().map_err(|e| e.to_string())?)?,
            versions: wire::VERSIONS.into(),
        })
    }

    fn semantic_options(&self, paired: bool) -> Vec<String> {
        // Normalize only locators the driver owns, never arbitrary strings in semantic flags.
        let normalized = Invocation {
            source: PathBuf::from(format!(
                "source/{}",
                self.source
                    .strip_prefix(&self.remap[0].0)
                    .unwrap_or(&self.source)
                    .display()
            )),
            out: "output".into(),
            std: self.std.as_ref().map(|_| "std".into()),
            deps: self
                .deps
                .iter()
                .enumerate()
                .map(|(i, _)| format!("dependency-{i}").into())
                .collect(),
            name: self.name.clone(),
            kind: self.kind.clone(),
            test: self.test,
            tested_library: self.tested_library,
            link: self.link,
            profile: self.profile.clone(),
            remap: self
                .remap
                .iter()
                .enumerate()
                .map(|(i, (_, role))| (format!("source-root-{i}").into(), role.clone()))
                .collect(),
            ..Invocation::default()
        };
        let mut options = normalized.rustc_args(
            Path::new("output"),
            paired.then(|| Path::new("paired-library")),
        );
        for value in &mut options {
            if value == env!("TIDE_SYSROOT") {
                *value = "pinned-sysroot".into();
            }
        }
        options.push(format!(
            "admission:unsafe={};backstop={};ice={}",
            self.grants.unsafe_, self.backstop_only, self.test_ice
        ));
        options
    }

    /// The dependency maps' index in `Build::dependencies`, after std's.
    fn first_dependency(&self) -> usize {
        usize::from(self.std.is_some())
    }

    /// `build` with each dependency directory's map naming the libraries in `loaded` that rustc
    /// read from it, each with the hash of its current bytes. rustc searches a `-L dependency=`
    /// directory without recursion and keeps the entry it chose as spelled there, symbolic links
    /// unresolved, so each library is one entry of a directory. The first directory in
    /// command-line order that is its parent owns it.
    fn loaded_identity(&self, build: &Build, loaded: &BTreeSet<PathBuf>) -> Result<Build, String> {
        let mut build = build.clone();
        for path in loaded {
            if self.std.as_ref() == fs::canonicalize(path).ok().as_ref() {
                continue;
            }
            let index = self
                .deps
                .iter()
                .position(|dep| path.parent() == Some(dep.as_path()))
                .ok_or_else(|| {
                    format!("the build loaded {}, which is in no --tide-dep-dir", path.display())
                })?;
            let name = path
                .file_name()
                .and_then(|name| name.to_str())
                .ok_or("non-UTF-8 dependency artifact name")?;
            let digest = file_digest(path)?;
            build.dependencies[self.first_dependency() + index].insert(name.into(), digest);
        }
        Ok(build)
    }

    /// `build` with each dependency directory's map hashing, in the current directory, the
    /// libraries `reference` names for it; a library no longer there is left out.
    fn current_artifacts(&self, build: &Build, reference: &Build) -> Result<Build, String> {
        let mut build = build.clone();
        for (index, dep) in self.deps.iter().enumerate() {
            let slot = self.first_dependency() + index;
            let Some(recorded) = reference.dependencies.get(slot) else {
                break;
            };
            for name in recorded.keys() {
                // A name is one directory entry, never a path out of the directory.
                if Path::new(name).file_name().is_none_or(|n| n != name.as_str()) {
                    return Err("replay dependency artifact name is not a file name".into());
                }
                let path = dep.join(name);
                // Only an absent entry is missing; any other stat error is reported as itself.
                let error = |e| format!("{}: {e}", wire::shown(&path.display().to_string()));
                if path.try_exists().map_err(error)? {
                    build.dependencies[slot].insert(name.clone(), file_digest(&path)?);
                }
            }
        }
        Ok(build)
    }

    pub(super) fn validate_replay(&self, build: &Build) -> Result<Option<Manifest>, String> {
        let Some(path) = &self.replay else {
            return Ok(None);
        };
        let manifest: Manifest = wire::read(path)?;
        manifest.validate()?;
        // Before the build: the libraries the recording loaded must be unchanged.
        manifest.build.compare(&self.current_artifacts(build, &manifest.build)?)?;
        Ok(Some(manifest))
    }

    pub(super) fn publish_receipt(
        &self,
        staging: &Path,
        build: &Build,
        loaded: &BTreeSet<PathBuf>,
    ) -> Result<(), String> {
        let receipt = Receipt {
            format: "tide-build/1".into(),
            build: self.loaded_identity(build, loaded)?,
            executable: file_digest(&staging.join("executable"))?,
            tests: if self.test {
                wire::read(&staging.join("test-config.json"))?
            } else {
                BTreeMap::new()
            },
        };
        fs::write(staging.join("build.json"), encoded(&receipt)).map_err(|e| e.to_string())
    }

    pub(super) fn run_test_process(
        &self,
        manifest: Option<Manifest>,
        current: &Build,
        artifacts: Option<&Path>,
    ) -> Result<ExitCode, String> {
        if self.run_test.is_none() && manifest.is_none() {
            return Ok(ExitCode::SUCCESS);
        }
        let receipt_path = artifacts.map_or_else(|| self.receipt_path(), |p| p.join("build.json"));
        let receipt: Receipt = wire::read(&receipt_path)?;
        if receipt.format != "tide-build/1" {
            return Err("replay receipt format mismatch".into());
        }
        receipt.build.compare(&self.current_artifacts(current, &receipt.build)?)?;
        // After the build: it loaded the same libraries as the recording.
        if let Some(manifest) = &manifest {
            manifest.build.compare(&receipt.build)?;
        }
        let name = self
            .run_test
            .as_deref()
            .unwrap_or_else(|| &manifest.as_ref().unwrap().test)
            .to_owned();
        if name.is_empty() || name.starts_with('-') {
            return Err("replay test identity is invalid".into());
        }
        let current_config = receipt
            .tests
            .get(&name)
            .ok_or("replay test identity/configuration is not a Tide harness test")?;
        if let Some(manifest) = &manifest {
            if &manifest.config != current_config {
                return Err(
                    "replay configuration mismatch; use fresh --seed for changed inputs".into(),
                );
            }
        }
        // Printed commands name operator-selected build inputs, never paths from a manifest.
        let mut command = vec![
            std::env::current_exe()
                .map_err(|e| e.to_string())?
                .display()
                .to_string(),
            "--test".into(),
            format!("--crate-type={}", self.kind),
            "--emit=link".into(),
            format!("--tide-profile={}", self.profile),
            format!("--crate-name={}", self.name),
            "--out-dir".into(),
            self.out.display().to_string(),
        ];
        if let Some(std) = &self.std {
            command.extend(["--tide-std".into(), std.display().to_string()]);
        }
        for dep in &self.deps {
            command.extend(["--tide-dep-dir".into(), dep.display().to_string()]);
        }
        // `test_library` is retained by the paired path after it resolves the library.
        if let Some(lib) = &self.test_library {
            command.extend(["--tide-test-library".into(), lib.display().to_string()]);
        }
        command.push(self.source.display().to_string());
        // RUN-2, RUN-3: a fresh private run directory in the driver's state; the child writes only
        // its `sim/`, and the harness publishes the artifacts to `target/sim/` afterward.
        fs::create_dir_all(&self.out).map_err(|e| e.to_string())?;
        let run = super::sandbox::RunDir::create(&self.out.join(".tide-cache"))?;
        // A concurrent publication must not replace the bytes between validation and exec.
        let executable = run.path.join("test");
        let source = artifacts.map_or_else(|| self.executable(), |p| p.join("executable"));
        let bound = fs::copy(source, &executable)
            .map_err(|e| e.to_string())
            .and_then(|_| {
                if file_digest(&executable)? == receipt.executable {
                    Ok(())
                } else {
                    Err("replay executable/receipt mismatch".into())
                }
            });
        let replaying = manifest.is_some();
        // RUN-4 hole 6: an evaluation of a simulation test gets a wall-clock limit, restarted at
        // each seed's start, as the simulation parent gave its child (`sim::runner`).
        let simulation = current_config.get("simulation").is_some_and(|v| v == "true");
        let limit = simulation.then(|| self.timeout.unwrap_or(SIMULATION_TIMEOUT));
        let directory = run.path.join("sim");
        let context = Context {
            receipt,
            completion: directory.join("reproduced").display().to_string(),
            directory: directory.display().to_string(),
            published: self.out.join("target/sim").display().to_string(),
            run: run.token(),
            command,
            replay: manifest,
        };
        let context_path = run.path.join("context.json");
        let sandbox = bound
            .and_then(|()| fs::write(&context_path, encoded(&context)).map_err(|e| e.to_string()))
            .and_then(|()| super::sandbox::Sandbox::prepare(&run));
        let sandbox = match sandbox {
            Ok(sandbox) => sandbox,
            Err(error) => {
                run.remove();
                return Err(error);
            }
        };
        let mut process = Command::new(executable);
        process
            .args([
                "--exact",
                name.as_str(),
                "--nocapture",
                "--include-ignored",
                "--test-threads=1",
            ])
            .env_clear()
            .env(wire::CONTEXT, &context_path)
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());
        if let Some(seed) = self.seed {
            process
                .env("TIDE_SIM_SEED", seed.to_string())
                .env("TIDE_TEST_SEED", seed.to_string());
        }
        eprintln!("sandbox: {}", sandbox.level());
        sandbox.apply(&mut process);
        let result = supervise(&mut process, limit).map_err(|e| {
            format!("cannot start the test child in its RUN-3 sandbox (session, core-size limit, descriptors, Landlock, or seccomp): {e}")
        });
        let reproduced = fs::read(&context.completion).is_ok_and(|bytes| bytes == b"reproduced");
        let published = run.publish(&self.out);
        let outcome = result?;
        published?;
        let seed = outcome.last_seed.map_or_else(|| "none".into(), |seed| seed.to_string());
        if outcome.timed_out {
            report_bounded(format!("test {name} failed: per-evaluation wall-clock timeout (last started seed: {seed})\n"));
            return Ok(ExitCode::from(101));
        }
        if let Some(signal) = std::os::unix::process::ExitStatusExt::signal(&outcome.status) {
            report_bounded(format!("test {name} failed: child terminated by signal {signal} (last started seed: {seed})\n"));
            return Ok(ExitCode::from(101));
        }
        if replaying && !reproduced {
            return Err("replay evaluation did not reproduce the recording (trace, outcome, roots or choice tape mismatch)".into());
        }
        Ok(ExitCode::from(outcome.status.code().unwrap_or(2) as u8))
    }
}

/// How long a timeout or crash report waits for output a stalled consumer has not taken.
const FORWARD_GRACE: std::time::Duration = std::time::Duration::from_secs(2);

/// Writes a timeout or crash report to the driver's standard error without std's stderr lock,
/// which a forwarder blocked on a stalled consumer may hold, waiting at most `FORWARD_GRACE`:
/// the driver's exit stays bounded even when the report cannot be delivered.
fn report_bounded(message: String) {
    let (sent, written) = std::sync::mpsc::channel();
    std::thread::spawn(move || {
        let mut bytes = message.as_bytes();
        while !bytes.is_empty() {
            // SAFETY: writes initialized bytes from a live buffer to descriptor 2.
            let n = unsafe { libc::write(2, bytes.as_ptr().cast(), bytes.len()) };
            if n < 0 && io::Error::last_os_error().kind() == io::ErrorKind::Interrupted {
                continue;
            }
            if n <= 0 {
                break;
            }
            bytes = &bytes[n as usize..];
        }
        let _ = sent.send(());
    });
    let _ = written.recv_timeout(FORWARD_GRACE);
}

/// The simulation parent's per-evaluation backstop (`tide-std` `sim::runner::TIMEOUT`).
const SIMULATION_TIMEOUT: u64 = 300;
const SEED_START: &str = "TIDE_SIM_SEED_START ";

struct Outcome {
    status: std::process::ExitStatus,
    last_seed: Option<u64>,
    timed_out: bool,
}

/// Runs the child with its output on pipes the harness drains (RUN-3): standard error passes
/// through, and standard output too except the seed-start records, which restart the limit.
/// Readers only drain and observe; separate forwarders write to the driver's own streams, so a
/// stalled consumer never stops seed observation. An ordinary exit waits for every byte to be
/// forwarded; a timeout or a fatal signal waits at most `FORWARD_GRACE` before reporting.
fn supervise(process: &mut Command, limit: Option<u64>) -> io::Result<Outcome> {
    use std::io::{BufRead, Read};
    use std::sync::mpsc;
    use std::time::{Duration, Instant};
    let mut child = process.spawn()?;
    let stdout = child.stdout.take().unwrap();
    let mut stderr = child.stderr.take().unwrap();
    let progress = Arc::new(Mutex::new((None, Instant::now())));
    let starts = progress.clone();
    // Each forwarder reports on `done` once its channel closes and every byte is written.
    let (done, finished) = mpsc::channel::<io::Result<()>>();
    let forwarder = |into_stdout: bool| {
        let (send, receive) = mpsc::channel::<Vec<u8>>();
        let done = done.clone();
        std::thread::spawn(move || {
            let result = receive.iter().try_for_each(|bytes| {
                if into_stdout {
                    let mut sink = io::stdout().lock();
                    sink.write_all(&bytes).and_then(|()| sink.flush())
                } else {
                    io::stderr().write_all(&bytes)
                }
            });
            let _ = done.send(result);
        });
        send
    };
    let (to_stdout, to_stderr) = (forwarder(true), forwarder(false));
    drop(done);
    let out = std::thread::spawn(move || -> io::Result<()> {
        let mut reader = io::BufReader::new(stdout);
        let mut line = Vec::new();
        while reader.read_until(b'\n', &mut line)? != 0 {
            let seed = std::str::from_utf8(&line)
                .ok()
                .and_then(|text| text.strip_prefix(SEED_START)?.trim().parse().ok());
            match seed {
                Some(seed) => *starts.lock().unwrap() = (Some(seed), Instant::now()),
                // A forwarder that failed has dropped its receiver; keep draining regardless.
                None => drop(to_stdout.send(std::mem::take(&mut line))),
            }
            line.clear();
        }
        Ok(())
    });
    let err = std::thread::spawn(move || -> io::Result<()> {
        let mut buffer = [0; 8192];
        loop {
            let n = stderr.read(&mut buffer)?;
            if n == 0 {
                return Ok(());
            }
            drop(to_stderr.send(buffer[..n].to_vec()));
        }
    });
    let mut timed_out = false;
    let status = loop {
        match child.try_wait() {
            Ok(Some(status)) => break Ok(status),
            Ok(None)
                if limit.is_none_or(|limit| {
                    progress.lock().unwrap().1.elapsed() < Duration::from_secs(limit)
                }) =>
            {
                std::thread::sleep(Duration::from_millis(10))
            }
            waiting => {
                timed_out = waiting.is_ok();
                let _ = child.kill();
                let reaped = child.wait();
                break waiting.and(reaped);
            }
        }
    };
    // The child is reaped, so its pipes reach end of file and the readers finish.
    let drained = out.join().expect("stdout reader panicked");
    let read = err.join().expect("stderr reader panicked");
    let failed = timed_out
        || status
            .as_ref()
            .is_ok_and(|s| std::os::unix::process::ExitStatusExt::signal(s).is_some());
    let deadline = Instant::now() + FORWARD_GRACE;
    for _ in 0..2 {
        let forwarded = if failed {
            match finished.recv_timeout(deadline.saturating_duration_since(Instant::now())) {
                Ok(result) => result,
                Err(_) => break,
            }
        } else {
            finished.recv().unwrap_or(Ok(()))
        };
        if !failed {
            forwarded?;
        }
    }
    drained?;
    read?;
    let last_seed = progress.lock().unwrap().0;
    Ok(Outcome { status: status?, last_seed, timed_out })
}

fn file_digest(path: &Path) -> Result<String, String> {
    // Use rustc's streaming SHA-256 for large supplied artifacts.
    // The path may name a file a replay manifest supplied.
    let error = |e: io::Error| format!("{}: {e}", wire::shown(&path.display().to_string()));
    let hash = SourceFileHash::new(
        SourceFileHashAlgorithm::Sha256,
        fs::File::open(path).map_err(error)?,
    )
    .map_err(error)?;
    Ok(hash
        .hash_bytes()
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect())
}
/// Read only macro-produced metadata from the same expanded tree that admission checks.
pub(super) fn test_configs(tcx: TyCtxt<'_>) -> BTreeMap<String, BTreeMap<String, String>> {
    let crates = crate::crates::Crates::new(tcx);
    let mut tests = BTreeMap::new();
    for id in tcx.hir_free_items() {
        let def = id.owner_id.def_id;
        if tcx.def_kind(def) != rustc_hir::def::DefKind::Fn {
            continue;
        }
        let mut config = BTreeMap::new();
        for attr in tcx.hir_attrs(id.hir_id()) {
            let Some(text) = attr.doc_str() else {
                continue;
            };
            let Some(text) = text.as_str().strip_prefix("__tide_replay_config:") else {
                continue;
            };
            let expansion = attr.span().ctxt().outer_expn_data();
            if !expansion.macro_def_id.is_some_and(|mac| {
                crates.exported(mac) && matches!(tcx.item_name(mac).as_str(), "test" | "sim_test")
            }) {
                continue;
            }
            if let Some((key, value)) = text.split_once(':') {
                config.insert(key.into(), value.into());
            }
        }
        if !config.is_empty() {
            use rustc_middle::ty::print::{
                with_crate_prefix, with_no_trimmed_paths, with_no_visible_paths,
            };
            let path = with_no_visible_paths!(with_no_trimmed_paths!(with_crate_prefix!(
                tcx.def_path_str(def)
            )));
            let name = path.strip_prefix("crate::").unwrap_or(&path);
            tests.insert(name.into(), wire::configuration(config));
        }
    }
    tests
}
