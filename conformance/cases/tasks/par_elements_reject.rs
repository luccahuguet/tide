//@ rules: PAR-4
// Elements a branch splits must be `Send` (an `Rc` is not), and every element
// must be `Pure`: `Shared` handles and channel ends are `Send + Sync` but not
// `Pure`, so only the checker rejects them.
use std::rc::Rc;
use std::sync::Shared;
use std::sync::mpsc::SyncSender;

pub fn not_send(xs: &mut [Rc<u64>]) {
    xs.par_for(|_| ()); //~ E0277
}

pub fn cells(xs: &[Shared<u64>]) -> Vec<u64> {
    xs.par_map(|c| c.with(|v| *v).unwrap_or(0)) //~ FX0612
}

pub fn senders(xs: &mut [SyncSender<u64>]) {
    xs.par_for(|tx| drop(tx.send(1))); //~ FX0612
}
