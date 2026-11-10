//@ rules: SINK-8
//@ profile: release
// Binary stdout and non-dbg sinks in release are accepted, also under --test.
#[tide::main]
fn main() {
    print!("items: ");
    println!("3");
    eprintln!("ready");
}
