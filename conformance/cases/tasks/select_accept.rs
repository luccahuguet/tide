//@ rules: REF-0605
// Multiplex data as an enum on one channel.
use std::sync::mpsc::Receiver;
pub enum Event { Data(u8), Stop }
pub fn next(receiver: &Receiver<Event>) -> Option<u8> {
    match receiver.recv().unwrap() { Event::Data(v) => Some(v), Event::Stop => None }
}
