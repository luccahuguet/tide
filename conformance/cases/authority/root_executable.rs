//@ rules: ROOT-1
// Also built and run against production std by tide-admit/tests/build.rs.
#[tide::main]
fn main(root: Root) {
    let _net = root.net;
    println!("HELLO_RAN");
}

#[test]
fn arithmetic() {
    println!("TEST_RAN");
    assert_eq!(2 + 2, 4);
}
