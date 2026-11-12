//@ rules: SCOPE-1, SCOPE-3, SEND-3
use std::sync::mpsc::sync_channel;
use std::task::TaskError;

#[tide::main]
fn main() -> Result<(), TaskError> {
    let sim = Root::sim(17);
    let (sender, receiver) = sync_channel::<u8>(0, &sim.tasks);
    let (blocked, unread) = sync_channel(0, &sim.tasks);
    sim.tasks.scope(|s| {
        s.spawn(move || {
            let _ = receiver.recv();
            Ok(())
        });
        let _keep_alive = (&sender, &unread);
        let _ = blocked.send(1);
        Ok(())
    })
}
