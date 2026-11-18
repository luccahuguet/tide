//@ expect: test fails discovery_settles_a_run_whose_host_exits
//@ reports: cause: discovery failed for a run whose host exited cleanly: cannot inspect terminal endpoint runtime/terminal-1.sock: NotFound
//@ reports: 87 passed; 13 failed (100 seeds)
// Eon's discovery before 27b39095 (eon-8ehhn), modelled. Discovery reads a run's record, and
// for a Live record checks the run's endpoint. The run's host, exiting, first publishes its
// tombstone and then removes its endpoint (ORB-C13's order). When both steps land between
// discovery's read and its check, discovery sees a Live run with no endpoint and fails, though
// the run simply ended.
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

/// The host's exit, in ORB-C13's order: publish the tombstone, then remove the endpoint.
pub fn exit(fs: &Fs) -> io::Result<()> {
    fs.write(RECORD, "tombstone")?;
    fs.remove_file(ENDPOINT)
}

/// Discovery: reads the record and, for a Live run, checks its endpoint.
pub fn discover(fs: &Fs) -> Result<Run, String> {
    match read_record(fs)? {
        Record::Tombstone => Ok(Run::Ended),
        Record::Live => match fs.metadata(ENDPOINT) {
            Ok(_) => Ok(Run::Live),
            Err(e) => Err(format!(
                "cannot inspect terminal endpoint {ENDPOINT}: {:?}",
                e.kind()
            )),
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
