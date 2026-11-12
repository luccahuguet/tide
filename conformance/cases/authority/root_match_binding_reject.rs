//@ rules: ROOT-1, ROOT-2
// A wildcard alone is not field destructuring; a whole-root fallback still binds Root.
#[tide::main]
fn main(root: Root) {
    match &root { _ => {} } //~ FX0503
    match root { //~ FX0503
        Root { ref fs, .. } if true => { let _ = fs; },
        all => { let _ = all; } //~ FX0503
    }
}
