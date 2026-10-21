//@ rules: MOVE-3
// `mem::take`, `mem::replace` and `Option::take` move out from behind a reference
// by leaving a value in place.
use std::mem;

pub struct Node {
    pub name: String,
    pub next: Option<Box<Node>>,
}

pub fn take_name(n: &mut Node) -> String {
    mem::take(&mut n.name)
}

pub fn rename(n: &mut Node, s: String) -> String {
    mem::replace(&mut n.name, s)
}

pub fn unlink(n: &mut Node) -> Option<Box<Node>> {
    n.next.take()
}
