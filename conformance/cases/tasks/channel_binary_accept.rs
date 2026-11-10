//@ rules: CHANNEL-1
// An unbounded channel is accepted in a binary, also under --test.
use std::Root;
use std::sync::mpsc::{channel, Receiver, Sender};
use std::task::Tasks;

#[tide::main]
fn main(root: Root) {
    let _ = channel::<u32>(&root.tasks);
    let pointer: fn(&Tasks) -> (Sender<u8>, Receiver<u8>) = channel::<u8>;
    let _ = pointer(&root.tasks);
    let held = Some(channel::<u8>);
    let pointer: fn(&Tasks) -> (Sender<u8>, Receiver<u8>) = held.unwrap();
    let _ = pointer(&root.tasks);
}
