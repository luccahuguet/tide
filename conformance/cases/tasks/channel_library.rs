//@ rules: CHANNEL-1
// Unbounded channels in libraries warn, including renamed, local and fn-pointer aliases.
use std::sync::mpsc::{channel as unbounded, Receiver, Sender};
use std::task::Tasks;

pub fn queues(tasks: &Tasks) {
    let _ = unbounded::<u32>(tasks); //~ WARN TD0608
    let make = unbounded::<u8>;
    let _ = make(tasks); //~ WARN TD0608
    let pointer: fn(&Tasks) -> (Sender<u8>, Receiver<u8>) = unbounded::<u8>; //~ WARN TD0608
    let _ = pointer(tasks);
    let held = Some(unbounded::<u8>);
    let pointer: fn(&Tasks) -> (Sender<u8>, Receiver<u8>) = held.unwrap(); //~ WARN TD0608
    let _ = pointer(tasks);
}
