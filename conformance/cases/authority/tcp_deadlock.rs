//@ rules: TCP-1
use std::io::Write;

#[tide::main]
fn main() {
    let sim = Root::sim(3);
    let listener = sim.net.bind("127.0.0.1:0").unwrap();
    let mut writer = sim.net.connect(listener.local_addr().unwrap()).unwrap();
    let (_unread, _) = listener.accept().unwrap();
    writer.write_all(&vec![1; 65_537]).unwrap();
}
