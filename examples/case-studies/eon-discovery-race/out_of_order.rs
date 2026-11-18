//@ expect: test fails discovery_settles_a_run_whose_host_exits
//@ reports: cause: discovery failed for a run whose host exited cleanly: cannot inspect terminal endpoint runtime/terminal-1.sock: NotFound
//@ reports: 89 passed; 11 failed (100 seeds)
// fixed.rs's discovery against a host that breaks ORB-C13: it removes its endpoint before it
// publishes its tombstone. The fix's single reread then finds the record still Live, and
// discovery fails as it did before the fix. Eon's fix is sound only under ORB-C13's order; this
// variant shows that the order is what the fix rests on, not that Orbit keeps it.
use std::fs::Fs;
use std::io;

/// The run's record: `live` while its host serves, `tombstone` once the host has exited.
pub const RECORD: &str = "runtime/terminal-1.record";
/// The run's endpoint, standing in for the host's Unix socket.
pub const ENDPOINT: &str = "runtime/terminal-1.sock";

/// What discovery learned about the run.
#[derive(Debug, PartialEq)]
pub enum Run {
    /// The record is Live and the endpoint is there.
    Live,
    /// The record is a tombstone: the host has exited.
    Ended,
}

enum Record {
    Live,
    Tombstone,
}

fn read_record(fs: &Fs) -> Result<Record, String> {
    match fs.read(RECORD).as_deref() {
        Ok(b"live") => Ok(Record::Live),
        Ok(b"tombstone") => Ok(Record::Tombstone),
        Ok(_) => Err(format!("terminal record {RECORD} is malformed")),
        Err(e) => Err(format!(
            "cannot read terminal record {RECORD}: {:?}",
            e.kind()
        )),
    }
}

/// The host's exit, against ORB-C13's order: remove the endpoint, then publish the tombstone.
pub fn exit(fs: &Fs) -> io::Result<()> {
    fs.remove_file(ENDPOINT)?;
    fs.write(RECORD, "tombstone")
}

/// Discovery: reads the record and, for a Live run, checks its endpoint; if the endpoint is
/// missing, rereads the record once.
pub fn discover(fs: &Fs) -> Result<Run, String> {
    match read_record(fs)? {
        Record::Tombstone => Ok(Run::Ended),
        Record::Live => match fs.metadata(ENDPOINT) {
            Ok(_) => Ok(Run::Live),
            Err(e) => {
                let error = format!(
                    "cannot inspect terminal endpoint {ENDPOINT}: {:?}",
                    e.kind()
                );
                // One reread is enough: the endpoint goes only after the tombstone (ORB-C13).
                match read_record(fs)? {
                    Record::Tombstone => Ok(Run::Ended),
                    Record::Live => Err(error),
                }
            }
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sim::SimRoot;
    use std::task::TaskError;

    /// A serving run: its endpoint exists and its record says Live.
    fn serving(sim: &SimRoot) {
        sim.world.file(ENDPOINT, "");
        sim.world.file(RECORD, "live");
    }

    /// Discovery runs while the run's host exits; the seed picks the interleaving.
    #[sim_test]
    fn discovery_settles_a_run_whose_host_exits(sim: SimRoot) {
        serving(&sim);
        let fs = &sim.fs;
        let found = sim
            .tasks
            .scope::<_, TaskError, _>(|s| {
                let host = s.spawn(move || Ok(exit(fs).unwrap()));
                let discovery = s.spawn(move || Ok(discover(fs)));
                host.join()?;
                discovery.join()
            })
            .unwrap();
        assert_eq!(fs.read(RECORD).unwrap(), b"tombstone");
        if let Err(e) = found {
            panic!("discovery failed for a run whose host exited cleanly: {e}");
        }
    }
}
