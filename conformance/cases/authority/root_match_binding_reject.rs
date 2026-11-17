//@ rules: ROOT-1, ROOT-2
// A wildcard alone is not field destructuring; a whole-root fallback still binds Root.
#[tide::main]
fn main(root: Root) {
    match &root { _ => {} } //~ TD0503
    match root { //~ TD0503
        Root { ref fs, .. } if true => { let _ = fs; },
        all => { let _ = all; } //~ TD0503
    }
}
