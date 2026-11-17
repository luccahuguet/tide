//@ rules: ROOT-1
// Every binary's `main` carries `#[tide::main]`, even one that takes no `root`.
fn main() { //~ TD0512
    println!("hello");
}
