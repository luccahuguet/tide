//@ rules: ROOT-1, ROOT-2
// Field patterns move or borrow fields, without binding the whole Root.
#[tide::main]
fn main(root: Root) {
    let Root { clock, .. }: &Root = &root;
    let _ = clock.now();
    let Root { net, .. } = root;
    let _ = net;
    let Root { ref clock, .. } = root;
    let _ = clock.now();
    let Root { fs, .. } = root;
    let _ = fs;
    match root {
        Root { env, .. } if true => { let _ = env; },
        _ => {}
    }
}
