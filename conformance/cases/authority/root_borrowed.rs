//@ rules: ROOT-1, ROOT-2
// `Root` named outside `main`, and `root` borrowed into a call.
fn has_config(root: &Root) -> bool { //~ FX0503
    root.fs.read("/etc/app.toml").is_ok()
}

#[tide::main]
fn main(root: Root) {
    let _ = has_config(&root); //~ FX0503
}
