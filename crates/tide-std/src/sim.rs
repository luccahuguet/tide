// Seeded simulation for tests (03 §9). Tasks, Shared, files, clocks and sinks are modeled.
// Other capability operations/setup, faults, nodes and replay manifests are pending.

use crate::task::{Simulation, Tasks};
use crate::time::Duration;

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
    simulation: Simulation,
}

impl SimRoot {
    #[track_caller]
    pub(crate) fn new(seed: u64) -> Self {
        let simulation = Simulation::new(seed);
        let tasks = simulation.tasks().clone();
        let fs = crate::fs::Fs::simulated(&tasks);
        Self {
            fs: fs.clone(),
            net: crate::net::Net(Some(tasks.clone())),
            clock: simulation.clock().clone(),
            rand: crate::rand::Rand(Some(tasks.clone())),
            env: crate::env::Env(Some(tasks.clone())),
            stdin: crate::io::Stdin(Some(tasks.clone())),
            proc: crate::process::Proc(Some(tasks.clone())),
            world: World { fs },
            faults: Faults {
                _tasks: tasks.clone(),
            },
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

/// Simulated input setup. Only file setup is currently modeled.
pub struct World {
    fs: crate::fs::Fs,
}
impl World {
    /// Immediately creates or replaces a file and creates its parents, from simulated `/`.
    /// Takes no virtual time and does not schedule. Panics on invalid setup paths.
    #[track_caller]
    pub fn file(&self, path: impl AsRef<std::path::Path>, contents: impl AsRef<[u8]>) {
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
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Level {
    Error,
    Warn,
    Info,
    Debug,
    Trace,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::panic::{AssertUnwindSafe, catch_unwind};

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

    fn sinks(seed: u64) -> String {
        let sim = crate::Root::sim(seed);
        crate::__sink::out(
            format_args!("quote=\" slash=\\ newline=\n nul=\0 café"),
            true,
        );
        crate::__sink::err(format_args!("error"), false);
        let records = sim.records();
        assert_eq!(records.len(), 2);
        assert_eq!(
            records[0].message,
            "quote=\" slash=\\ newline=\n nul=\0 café\n"
        );
        assert_eq!(records[0].sink, Sink::Out);
        assert_eq!(records[1].sink, Sink::Err);
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
