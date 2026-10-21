//@ rules: ROOT-1
// A `main` that takes `root` without `#[tide::main]`: rustc rejects the parameter.
fn main(root: Root) { //~ E0580 FX0512
    let _ = root.fs.read("/etc").is_ok();
}
