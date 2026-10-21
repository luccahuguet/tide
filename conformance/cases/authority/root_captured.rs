//@ rules: ROOT-2
// A closure that captures `root` whole rather than one of its fields.
#[tide::main]
fn main(root: Root) {
    let check = || {
        let all = &root; //~ FX0503
        all.fs.read("/etc").is_ok()
    };
    let _ = check();
}
