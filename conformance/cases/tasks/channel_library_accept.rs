//@ rules: CHANNEL-1
// A bounded library channel does not get the unbounded-channel lint.
use std::sync::mpsc::sync_channel;
use std::task::Tasks;

pub fn queue(tasks: &Tasks) {
    let _ = sync_channel::<u32>(4, tasks);
}
