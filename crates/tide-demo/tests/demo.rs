//! The workspace suite runs the demo, and two runs with one revision swapped show that it fails
//! when the fix is missing or the patch under review no longer widens the interface. Each run
//! works in its own directory under Cargo's test scratch directory, unique to the test process,
//! and removes it when the case ends, whether it passes, fails, or panics.
use std::path::PathBuf;
use tide_demo::{Demo, Failure, Revision};

/// A case's work directory; dropping it removes the directory.
struct Work(PathBuf);

impl Drop for Work {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

fn demo(case: &str) -> (Demo, Work) {
    // Cargo's scratch directory for integration tests is `<target-dir>/tmp`, however the target
    // dir was set (a relative `CARGO_TARGET_DIR`, `--target-dir`, `build.target-dir`): the demo
    // builds into the suite's own target dir, so its builds reuse the suite's.
    let scratch = PathBuf::from(env!("CARGO_TARGET_TMPDIR"));
    let work = scratch.join(format!("tide-demo-{case}-{}", std::process::id()));
    let demo = Demo {
        work: work.clone(),
        target: scratch.parent().map(PathBuf::from),
        ..Demo::default()
    };
    (demo, Work(work))
}

/// The step a run failed at, or a panic with what else it did.
fn failed_step(demo: Demo) -> u8 {
    match demo.run() {
        Err(Failure::Step { step, .. }) => step,
        other => panic!("expected a failed step, got {other:?}"),
    }
}

#[test]
fn the_demo_passes() {
    let (demo, _work) = demo("pass");
    if let Err(failure) = demo.run() {
        failure.report();
        panic!("{failure}");
    }
}

/// The buggy patch applied as the fixed revision: step 3 runs the bug over the seed set.
#[test]
fn a_buggy_fixed_revision_fails_step_3() {
    let (demo, _work) = demo("buggy-fixed");
    let demo = Demo {
        fixed: Revision::Patched("lost-update.patch"),
        ..demo
    };
    assert_eq!(failed_step(demo), 3);
}

/// The widening patch dropped: the trust diff no longer shows `hits::serve` widened.
#[test]
fn a_dropped_widening_fails_step_4() {
    let (demo, _work) = demo("no-widening");
    let demo = Demo {
        widened: Revision::Service,
        ..demo
    };
    assert_eq!(failed_step(demo), 4);
}
