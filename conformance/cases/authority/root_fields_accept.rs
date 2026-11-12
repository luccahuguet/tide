//@ rules: ROOT-1, ROOT-2
// Moving or borrowing individual fields, including disjoint closure captures, is legal.
#[tide::main]
fn main(root: Root) {
    let _net = root.net;
    let fs = root.fs;
    let _ = fs.read("settings");
    let clock = &root.clock;
    let _ = clock.now();
    let read = || root.env.var("MODE");
    let _ = read();
    let tasks = move || root.tasks.checkpoint();
    let _ = tasks();
}
