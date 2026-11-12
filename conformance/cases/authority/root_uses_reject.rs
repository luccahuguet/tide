//@ rules: ROOT-1, ROOT-2
// Generic calls and bindings cannot hide whole-root use, even with a field subpattern.
#![allow(warnings)]
fn accept<T>(_: T) {}
#[tide::main]
fn main(root: Root) {
    accept(&root); //~ FX0503
    let borrowed = &root; //~ FX0503
    let _ = borrowed.fs.read("settings");
    let whole @ Root { .. } = root; //~ FX0503
    let _ = whole; //~ FX0503
}
