//@ rules: DROP-1
// Rule DC rejects: a value with a destructor that drops after the data it
// borrows, and two values with destructors pointing at each other (B4's `Node` pair).
use std::cell::Cell;

pub struct Guard<'a> {
    pub text: &'a String,
}
impl Drop for Guard<'_> {
    fn drop(&mut self) {}
}

pub fn outlived() {
    let guard;
    let text = String::from("x");
    guard = Guard { text: &text }; //~ E0597
}

pub struct Node<'a> {
    pub next: Cell<Option<&'a Node<'a>>>,
}
impl Drop for Node<'_> {
    fn drop(&mut self) {}
}

pub fn cycle() {
    let a = Node { next: Cell::new(None) };
    let b = Node { next: Cell::new(None) };
    a.next.set(Some(&b)); //~ E0597
    b.next.set(Some(&a)); //~ E0597
}
