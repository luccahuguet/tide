//@ rules: SEND-1
// Types the table makes not `Send` or not `Sync`, directly and through a field.
use std::cell::{Cell, OnceCell, RefCell};
use std::rc::Rc;
use std::sync::mpsc::Receiver;
use std::task::{Task, TaskError};

fn send<T: Send>() {}
fn sync<T: Sync>() {}

pub struct Counter {
    pub hits: Rc<u64>,
}

pub fn table() {
    send::<Rc<u8>>(); //~ E0277
    sync::<Cell<u8>>(); //~ E0277
    sync::<RefCell<u8>>(); //~ E0277
    sync::<OnceCell<u8>>(); //~ E0277
    send::<Cell<Rc<u8>>>(); //~ E0277
    send::<&Cell<u8>>(); //~ E0277
    sync::<Receiver<u8>>(); //~ E0277
    send::<Counter>(); //~ E0277
}

// A `Task` whose result or error is not `Send` is neither `Send` nor `Sync`.
pub fn task<'s>() {
    send::<Task<'s, Rc<u8>, TaskError>>(); //~ E0277
    sync::<Task<'s, u8, Rc<u8>>>(); //~ E0277
}
