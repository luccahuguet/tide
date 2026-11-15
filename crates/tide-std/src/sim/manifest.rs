//! Per-evaluation recording. A context exists only on the driver's test-running path.
use crate::replay::{self, Context, Manifest, RootRun, digest, encoded};
use crate::test::{Choice, Config};
use std::cell::{Cell, RefCell};
use std::collections::BTreeMap;
use std::fs;
use std::io::Write;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

thread_local! {
    static ACTIVE: RefCell<Option<Recording>> = const { RefCell::new(None) };
    static SUSPENDED: Cell<bool> = const { Cell::new(false) };
}
struct Recording {
    journal: Arc<Mutex<Journal>>,
    context: Option<Context>,
    tasks: Vec<crate::task::Tasks>,
}
pub(crate) struct Journal {
    manifest: Option<Manifest>,
    path: Option<PathBuf>,
    roots: Vec<RootRun>,
}

pub(crate) fn context() -> Option<Context> {
    std::env::var_os(replay::CONTEXT)
        .map(|path| replay::read(&PathBuf::from(path)).unwrap_or_else(|e| panic!("{e}")))
}
pub(crate) fn active() -> bool {
    SUSPENDED.get() || ACTIVE.with(|slot| slot.borrow().is_some()) || crate::task::is_simulating()
}

// Nested generators and roots must not append to their caller's input journal.
pub(crate) struct Suspended(Option<Recording>, bool);
pub(crate) fn suspend() -> Suspended {
    Suspended(ACTIVE.with(|slot| slot.borrow_mut().take()), SUSPENDED.replace(true))
}
impl Drop for Suspended {
    fn drop(&mut self) {
        ACTIVE.with(|slot| *slot.borrow_mut() = self.0.take());
        SUSPENDED.set(self.1);
    }
}
pub(crate) fn config(config: Config) -> BTreeMap<String, String> {
    replay::configuration(BTreeMap::from([
        ("cases".into(), config.cases.to_string()),
        ("plain".into(), config.plain.to_string()),
        ("simulation".into(), config.simulation.to_string()),
        ("fixed_seed".into(), config.fixed_seed.to_string()),
        ("should_panic".into(), config.should_panic.to_string()),
        (
            "allow_vacuous".into(),
            format!("{:?}", config.allow_vacuous),
        ),
    ]))
}

pub(crate) fn begin(path: &str, seed: u64, ordinal: u64, config_value: Config, persist: bool) {
    assert!(!active(), "nested property evaluation");
    let context = if path.is_empty() { None } else { context() };
    let name = path.split_once("::").map_or(path, |(_, name)| name);
    if let Some(expected) = context.as_ref().and_then(|c| c.replay.as_ref()) {
        assert_eq!(expected.test, name, "replay test identity mismatch");
        assert_eq!(
            expected.config,
            config(config_value),
            "replay configuration mismatch"
        );
    }
    let manifest = context.as_ref().filter(|_| persist).map(|c| {
        let mut manifest = Manifest {
            format: replay::FORMAT.into(),
            build: c.receipt.build.clone(),
            test: name.into(),
            evaluation: ordinal,
            seed: seed.to_string(),
            config: config(config_value),
            choices: vec![],
            roots: vec![],
            injected: vec![],
            complete: false,
            outcome: "incomplete".into(),
            trace: String::new(),
            trace_hash: String::new(),
            input_hash: String::new(),
        };
        manifest.seal();
        manifest
    });
    let artifact = manifest.as_ref().map(|m| {
        let build = digest(encoded(&m.build).as_bytes());
        let name = digest(name.as_bytes());
        let base = format!("{}-{seed}-{ordinal}-{}", &name[..16], &build[..16]);
        // Reserve a private directory exclusively, so even concurrent identical runs cannot
        // replace each other's artifacts. Replacements below stay within this reservation.
        for suffix in 0u64.. {
            let directory = PathBuf::from(&context.as_ref().unwrap().directory)
                .join(format!("{base}-{suffix}"));
            match fs::create_dir(&directory) {
                Ok(()) => return directory.join("run.manifest.json"),
                Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => (),
                Err(e) => panic!("cannot reserve replay artifact: {e}"),
            }
        }
        unreachable!()
    });
    let mut journal = Journal {
        manifest,
        path: artifact,
        roots: vec![],
    };
    journal.save();
    let recording = Recording {
        journal: Arc::new(Mutex::new(journal)),
        context,
        tasks: vec![],
    };
    ACTIVE.with(|slot| {
        *slot.borrow_mut() = Some(recording);
    });
}
impl Journal {
    fn save(&mut self) {
        if let (Some(manifest), Some(path)) = (&mut self.manifest, &self.path) {
            manifest.seal();
            let temporary = path.with_extension("pending");
            let mut file = fs::File::create(&temporary).expect("create replay replacement");
            file.write_all(encoded(manifest).as_bytes())
                .expect("write replay manifest");
            fs::rename(temporary, path).expect("publish replay manifest");
        }
    }
}
pub(crate) fn root(seed: u64, tasks: crate::task::Tasks) {
    ACTIVE.with(|slot| {
        let mut slot = slot.borrow_mut();
        let Some(recording) = slot.as_mut() else {
            return;
        };
        let ordinal = recording.tasks.len();
        let root = RootRun {
            ordinal: ordinal as u64,
            seed: seed.to_string(),
            inputs: vec![],
        };
        if let Some(expected) = recording.context.as_ref().and_then(|c| c.replay.as_ref()) {
            let expected = expected
                .roots
                .get(ordinal)
                .expect("replay root count mismatch");
            assert_eq!(
                (&expected.seed, expected.ordinal),
                (&root.seed, root.ordinal),
                "replay root/seed mismatch"
            );
        }
        let mut journal = recording.journal.lock().unwrap();
        tasks.sim_recording(recording.journal.clone(), ordinal);
        journal.roots.push(root);
        journal.update_roots();
        recording.tasks.push(tasks);
    });
}
impl Journal {
    fn update_roots(&mut self) {
        if let Some(manifest) = &mut self.manifest {
            manifest.roots = self.roots.clone();
        }
        self.save();
    }
    pub(crate) fn input(&mut self, ordinal: usize, kind: &str, key: Vec<u8>, value: Vec<u8>) {
        self.roots[ordinal].inputs.push(replay::Input {
            kind: kind.into(),
            key,
            value,
        });
        self.update_roots();
    }
}
pub(crate) fn choice(choice: &Choice) {
    ACTIVE.with(|slot| {
        let slot = slot.borrow();
        if let Some(recording) = slot.as_ref() {
            let mut journal = recording.journal.lock().unwrap();
            if let Some(manifest) = &mut journal.manifest {
                manifest.choices.push(choice.wire());
            }
            journal.save();
        }
    });
}

pub(crate) fn finish(choices: &[Choice], signature: &str, failed: bool) -> crate::task::Trace {
    let recording = ACTIVE.with(|slot| slot.borrow_mut().take()).unwrap();
    let mut journal = recording.journal.lock().unwrap();
    let mut trace = crate::task::Trace::default();
    for tasks in &recording.tasks {
        trace.append(tasks.sim_take_trace());
    }
    let rendered = trace.render();
    if let Some(expected) = recording.context.as_ref().and_then(|c| c.replay.as_ref()) {
        assert_eq!(expected.roots, journal.roots, "replay root count mismatch");
        assert_eq!(expected.outcome, signature, "replay outcome mismatch");
        if let Some(difference) = replay::first_difference(&expected.trace, &rendered) {
            panic!("replay {difference}");
        }
        assert_eq!(
            expected.trace_hash,
            digest(rendered.as_bytes()),
            "replay trace hash mismatch"
        );
    }
    if let Some(manifest) = &mut journal.manifest {
        manifest.choices = choices.iter().map(Choice::wire).collect();

        manifest.complete = true;
        manifest.outcome = signature.into();
        manifest.trace = rendered.clone();
    }
    journal.save();
    if let Some(path) = &journal.path {
        fs::write(path.with_file_name("trace.ndjson"), &rendered)
            .expect("write replay trace export");
        if failed {
            let mut command = recording.context.as_ref().unwrap().command.clone();
            command.extend(["--replay".into(), path.display().to_string()]);
            eprintln!(
                "  replay: {}",
                command
                    .iter()
                    .map(|s| replay::shell(s))
                    .collect::<Vec<_>>()
                    .join(" ")
            );
        }
    }
    if let Some(context) = &recording.context {
        if let Some(expected) = &context.replay {
            fs::write(&context.completion, b"reproduced").expect("report replay completion");
            eprintln!(
                "replay reproduced: test={} seed={} outcome={signature}",
                expected.test, expected.seed
            );
        }
    }
    trace
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::process::{Command, Stdio};

    #[test]
    fn abort_retains_a_valid_initial_manifest_and_consumed_inputs() {
        let path = concat!(
            module_path!(),
            "::abort_retains_a_valid_initial_manifest_and_consumed_inputs"
        );
        if std::env::var_os("__TIDE_ABORT_RECORDING").is_some() {
            begin(path, 47, 0, Config::default(), true);
            let sim = crate::Root::sim(17);
            sim.world.args(["inline"]);
            sim.world.stdin([0, 255]);
            choice(&Choice::from_wire(&replay::Choice {
                kind: "bool".into(),
                bound: "1".into(),
                value: "1".into(),
            }));
            std::process::abort();
        }
        let directory =
            std::env::temp_dir().join(format!("tide-initial-manifest-{}", std::process::id()));
        // Test artifacts have their own exclusive namespace, just like real run artifacts.
        fs::create_dir_all(&directory).unwrap();
        let context_path = directory.join("context.json");
        let build = replay::Build {
            source: BTreeMap::new(),
            library: BTreeMap::new(),
            dependencies: vec![],
            compiler: "test".into(),
            target: "test".into(),
            profile: "debug".into(),
            options: vec![],
            driver: "test".into(),
            versions: replay::VERSIONS.into(),
        };
        let context = Context {
            receipt: replay::Receipt {
                format: "tide-build/1".into(),
                build,
                executable: "test".into(),
                tests: BTreeMap::new(),
            },
            directory: directory.display().to_string(),
            command: vec![],
            completion: String::new(),
            replay: None,
        };
        fs::write(&context_path, encoded(&context)).unwrap();
        let status = Command::new(std::env::current_exe().unwrap())
            .args(["--exact", path.split_once("::").unwrap().1, "--nocapture"])
            .env_clear()
            .env(replay::CONTEXT, &context_path)
            .env("__TIDE_ABORT_RECORDING", "1")
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
            .unwrap();
        assert!(!status.success());
        let manifest = fs::read_dir(&directory)
            .unwrap()
            .map(|e| e.unwrap().path().join("run.manifest.json"))
            .find(|p| p.is_file())
            .unwrap();
        let mut manifest: Manifest = replay::read(&manifest).unwrap();
        assert!(!manifest.complete);
        assert_eq!(manifest.roots[0].seed, "17");
        assert_eq!(manifest.roots[0].inputs[1].value, [0, 255]);
        assert_eq!(manifest.choices[0].value, "1");
        let digest = manifest.input_hash.clone();
        manifest.seal();
        assert_eq!(manifest.input_hash, digest);
        assert!(manifest.validate().unwrap_err().contains("incomplete"));
        fs::remove_dir_all(directory).unwrap();
    }
}
