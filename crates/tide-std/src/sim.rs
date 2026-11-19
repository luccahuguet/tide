// Seeded simulation for tests (03 §9). Tasks, Arc/Weak, Shared, channels, files, TCP loopback, process inputs, declared child-process models, clocks and sinks are modeled.
// Driver-backed evaluation manifests use the private harness below.
// Other capability operations/setup, faults and nodes are pending.

use crate::task::{Simulation, Tasks};
use crate::time::Duration;

#[path = "sim/manifest.rs"]
pub(crate) mod manifest;
#[path = "sim/http.rs"]
mod http;
pub use http::{Reply, Request};
pub(crate) use http::Answer;

pub(crate) mod runner {
    include!("sim/runner.rs");
}
#[doc(hidden)]
pub use runner::__run;

/// A simulated root. Its calling thread is task 0 until this value drops.
/// It cannot move between threads; cloned capabilities require their own live simulation.
pub struct SimRoot {
    pub fs: crate::fs::Fs,
    pub net: crate::net::Net,
    pub clock: crate::time::Clock,
    pub rand: crate::rand::Rand,
    pub env: crate::env::Env,
    pub stdin: crate::io::Stdin,
    pub proc: crate::process::Proc,
    pub tasks: Tasks,
    pub world: World,
    pub faults: Faults,
    _http_peers: HttpPeers,
    _coverage: Coverage,
    simulation: Simulation,
}

impl SimRoot {
    #[track_caller]
    pub(crate) fn new(seed: u64) -> Self {
        Self::new_at(seed, None)
    }

    #[track_caller]
    pub(crate) fn new_at(seed: u64, origin: Option<crate::test::Source>) -> Self {
        let simulation = Simulation::new_at(seed, origin);
        let tasks = simulation.tasks().clone();
        manifest::root(seed, tasks.clone());
        let fs = crate::fs::Fs::simulated(&tasks);
        let proc = crate::process::Proc::new(Some(tasks.clone()));
        let net = crate::net::Net::new(Some(tasks.clone()));
        Self {
            fs: fs.clone(),
            net: net.clone(),
            clock: simulation.clock().clone(),
            rand: crate::rand::Rand(Some(tasks.clone())),
            env: crate::env::Env::new(Some(tasks.clone())),
            stdin: crate::io::Stdin(Some(tasks.clone())),
            proc: proc.clone(),
            world: World {
                fs,
                proc: proc.clone(),
                net: net.clone(),
                tasks: tasks.clone(),
            },
            faults: Faults {
                _tasks: tasks.clone(),
            },
            _http_peers: HttpPeers(net),
            _coverage: Coverage(proc),
            tasks,
            simulation,
        }
    }

    /// Virtual time since the simulation began. Inspection does not schedule.
    pub fn elapsed(&self) -> Duration {
        self.simulation.tasks().sim_elapsed()
    }

    /// Ordered sink records. Simulated sinks do not write to host streams.
    pub fn records(&self) -> Vec<Record> {
        self.simulation.tasks().sim_records()
    }

    /// Source-located events as NDJSON, with provisional field names.
    pub fn trace(&self) -> String {
        self.simulation.tasks().sim_trace()
    }
}

// Clear callback/capability cycles while the runtime is still live, including on unwind.
// A field guard keeps SimRoot's fields movable.
struct HttpPeers(crate::net::Net);
impl Drop for HttpPeers {
    fn drop(&mut self) {
        self.0.clear_http_peers();
    }
}

// A process coverage failure stops the seed even when no later operation observed it. A field
// guard rather than `Drop for SimRoot`, so tests can still move fields out of the root.
struct Coverage(crate::process::Proc);
impl Drop for Coverage {
    fn drop(&mut self) {
        let failure = self.0.sim.as_ref().and_then(|s| s.lock().ok()?.failure());
        if let Some(failure) = failure.filter(|_| !std::thread::panicking()) {
            panic!("{failure}");
        }
    }
}

#[path = "sim/process.rs"]
pub(crate) mod process;
pub use process::ProcessModel;

/// Simulated input setup: files, process inputs and models, and HTTP peers.
pub struct World {
    fs: crate::fs::Fs,
    proc: crate::process::Proc,
    net: crate::net::Net,
    tasks: Tasks,
}
impl World {
    /// Appends arguments, including the program name when desired.
    pub fn args(&self, args: impl IntoIterator<Item: AsRef<std::ffi::OsStr>>) {
        let args: Vec<_> = args.into_iter().map(|s| s.as_ref().to_owned()).collect();
        for arg in &args { self.tasks.replay_input("arg", vec![], arg.as_encoded_bytes().to_vec()); }
        self.tasks.world_inputs(|inputs| inputs.args.extend(args));
    }
    /// Sets or replaces a variable. Inputs start empty.
    pub fn env_var(&self, key: impl AsRef<std::ffi::OsStr>, value: impl AsRef<std::ffi::OsStr>) {
        let key = key.as_ref();
        assert!(
            !key.is_empty()
                && !key
                    .as_encoded_bytes()
                    .iter()
                    .any(|b| matches!(*b, b'=' | 0)),
            "environment key must be nonempty and contain neither '=' nor NUL"
        );
        let (key, value) = (key.to_owned(), value.as_ref().to_owned());
        self.tasks.replay_input("environment", key.as_encoded_bytes().to_vec(), value.as_encoded_bytes().to_vec());
        self.tasks.world_inputs(|inputs| {
            inputs.env.insert(key, value);
        });
    }
    /// Appends bytes to a pipe whose writer has closed. Reads have zero latency.
    pub fn stdin(&self, bytes: impl AsRef<[u8]>) {
        let bytes = bytes.as_ref();
        self.tasks.replay_input("stdin", vec![], bytes.to_vec());
        self.tasks.world_inputs(|inputs| inputs.stdin.extend(bytes));
    }
    /// Registers a declarative model for an exact program spelling (03 §9.9): no `PATH` search,
    /// shell or host execution. Takes no point, draw or time; panics on duplicates and malformed
    /// models. Under simulation an unregistered program is a coverage failure.
    #[track_caller]
    pub fn proc(&self, program: impl AsRef<std::ffi::OsStr>, model: ProcessModel) {
        crate::process::sim::register(&self.proc, program.as_ref(), model);
    }
    /// A simulated HTTP peer at `addr`, an `only` entry's `"host:port"`, serving `http` and
    /// `https` URLs to that port. Setup takes no scheduling point, time or draw.
    ///
    /// # Panics
    ///
    /// When `addr` is not an entry, is already a peer, or a simulated listener holds it.
    #[track_caller]
    pub fn http(&self, addr: &str, peer: impl Fn(&Request) -> Reply + Send + 'static) {
        let key = self.net.world_http(addr, std::sync::Arc::new(std::sync::Mutex::new(Some(Box::new(peer)))));
        self.tasks.replay_input("http", key.into_bytes(), b"net-http-v0".to_vec());
    }
    /// Immediately creates or replaces a file and creates its parents, from simulated `/`.
    /// Takes no virtual time and does not schedule. Panics on invalid setup paths.
    #[track_caller]
    pub fn file(&self, path: impl AsRef<std::path::Path>, contents: impl AsRef<[u8]>) {
        self.tasks.replay_input("file", path.as_ref().as_os_str().as_encoded_bytes().to_vec(), contents.as_ref().to_vec());
        self.fs.world_file(path.as_ref(), contents.as_ref());
    }
}
/// Identity reserved for simulated faults; operations are not provided yet.
pub struct Faults {
    _tasks: Tasks,
}

/// A sink record (provisional, reference §6.8). The only node in this slice is `main`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Record {
    pub node: String,
    pub time: Duration,
    pub sink: Sink,
    pub message: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Sink {
    Out,
    Err,
    Dbg,
    Log(Level),
}
pub use crate::log::Level;

#[cfg(test)]
mod tests {
    use super::*;
    use std::panic::{AssertUnwindSafe, catch_unwind};

    #[test]
    fn invalid_environment_setup_panics_without_poisoning_simulation() {
        let sim = crate::Root::sim(7);
        for key in ["", "A=B", "A\0B"] {
            assert!(catch_unwind(AssertUnwindSafe(|| sim.world.env_var(key, "value"))).is_err());
        }
        assert!(sim.trace().is_empty());
        assert_eq!(sim.env.vars().len(), 0);
        sim.world.env_var("VALID", "value");
        assert_eq!(sim.env.var("VALID").unwrap(), "value");
    }

    #[test]
    fn dropping_the_root_in_a_live_scope_asserts_and_does_not_restore_a_dead_context() {
        let sim = crate::Root::sim(0);
        let tasks = sim.tasks.clone();
        let panic = catch_unwind(AssertUnwindSafe(|| {
            tasks
                .scope::<_, crate::task::TaskError, _>(|_| {
                    drop(sim);
                    Ok(())
                })
                .unwrap();
        }));
        assert!(panic.is_err());
        assert!(catch_unwind(AssertUnwindSafe(|| tasks.checkpoint())).is_err());
        crate::Root::sim(1).tasks.checkpoint().unwrap();
    }

    #[test]
    fn simulation_identities_never_use_host_files_or_randomness() {
        let (fs, rand, tasks) = {
            let sim = crate::Root::sim(0);
            assert!(
                catch_unwind(AssertUnwindSafe(|| {
                    crate::collections::hash_map::RandomState::from_rand(&sim.rand)
                }))
                .is_err()
            );
            assert!(catch_unwind(AssertUnwindSafe(|| crate::Root::sim(1))).is_err());
            // None of those unsupported operations adds a scheduling point or sink event.
            assert!(sim.trace().is_empty());
            (sim.fs.clone(), sim.rand.clone(), sim.tasks.clone())
        };
        assert!(catch_unwind(AssertUnwindSafe(|| fs.read("Cargo.toml"))).is_err());
        assert!(
            catch_unwind(AssertUnwindSafe(|| {
                crate::collections::hash_map::RandomState::from_rand(&rand)
            }))
            .is_err()
        );
        assert!(catch_unwind(AssertUnwindSafe(|| tasks.checkpoint())).is_err());
        // The old main context was removed; a later simulation on this thread works.
        crate::Root::sim(1).tasks.checkpoint().unwrap();
    }

    #[test]
    fn suppressed_task_failures_emit_one_count_only_debug_record() {
        let sim = crate::Root::sim(3);
        let result = sim.tasks.scope::<_, crate::task::TaskError, _>(|scope| {
            scope.spawn(|| Err::<(), _>(crate::task::TaskError::Cancelled));
            Err::<(), _>(crate::task::TaskError::Cancelled)
        });
        assert!(result.is_err());
        let records = sim.records();
        assert_eq!(records.len(), 1);
        assert_eq!(records[0].sink, Sink::Log(Level::Debug));
        assert_eq!(records[0].message, "1 suppressed task outcomes");
    }

    fn sinks(seed: u64) -> String {
        let sim = crate::Root::sim(seed);
        crate::__sink::out(
            format_args!("quote=\" slash=\\ newline=\n nul=\0 café"),
            true,
        );
        crate::__sink::err(format_args!("error"), false);
        let records = sim.records();
        assert_eq!(records.len(), 3);
        assert_eq!(records[0].message, "quote=\" slash=\\ newline=");
        assert_eq!(records[0].sink, Sink::Out);
        assert_eq!(records[1].message, " nul=\0 café");
        assert_eq!(records[2].sink, Sink::Err);
        let trace = sim.trace();
        assert_eq!(trace.lines().count(), 2);
        assert!(
            trace.contains(r#"quote=\" slash=\\ newline=\u000a nul=\u0000 café\u000a"#),
            "{trace}"
        );
        for line in trace.lines() {
            assert!(line.contains("\"kind\":\"sink\""), "{line}");
            assert!(line.contains("\"operation\":\"sink\""), "{line}");
            assert!(line.contains("\"wait\":null"), "{line}");
            assert!(line.contains("sim.rs"), "{line}");
        }
        trace
    }

    #[test]
    fn sink_records_are_whole_escaped_events_and_do_not_schedule() {
        assert_eq!(sinks(7), sinks(7));
    }
}
