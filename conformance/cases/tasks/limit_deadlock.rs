//@ rules: SCOPE-1, SEND-3
// The driver executes this in a subprocess and checks the global deadlock report.
use std::task::TaskError;

#[test]
fn deadlock() {
    let sim = Root::sim(3);
    let tasks = sim.tasks.limit(1);
    let _: Result<(), TaskError> = tasks.scope(|s| {
        s.spawn(move || {
            s.spawn(|| Ok(()));
            Ok(())
        });
        s.spawn(|| Ok(()));
        Ok(())
    });
    panic!("deadlock was not detected");
}
