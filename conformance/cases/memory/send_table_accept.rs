//@ rules: SEND-1
// `Send` and `Sync` follow the table structurally; a user type built from
// `Send + Sync` fields is both without any `unsafe impl`.
use std::cell::{Cell, RefCell};
use std::fs::Fs;
use std::sync::Shared;
use std::sync::mpsc::{Receiver, SyncSender};
use std::task::{Scope, Task, Tasks};

fn send<T: Send>() {}
fn sync<T: Sync>() {}

pub struct Config {
    pub name: String,
    pub limits: Vec<u32>,
    pub cell: Shared<u64>,
}

pub fn table<'s, 'env: 's, E>() {
    send::<Cell<u8>>();
    send::<RefCell<String>>();
    send::<&u8>();
    sync::<&u8>();
    send::<&mut Cell<u8>>();
    send::<Shared<u64>>();
    sync::<Shared<u64>>();
    send::<SyncSender<u8>>();
    sync::<SyncSender<u8>>();
    send::<Receiver<u8>>();
    send::<Fs>();
    sync::<Tasks>();
    send::<Scope<'s, 'env, E>>();
    sync::<Scope<'s, 'env, E>>();
    send::<Config>();
    sync::<Config>();
}

// A `Task` is `Sync` when its result is `Send`, even if not `Sync`.
pub fn task<'s, E: Send>() {
    send::<Task<'s, Cell<u8>, E>>();
    sync::<Task<'s, Cell<u8>, E>>();
}
