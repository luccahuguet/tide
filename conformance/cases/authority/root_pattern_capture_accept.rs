//@ rules: ROOT-1, ROOT-2
// A field pattern inside a closure may capture only the field it moves.
#[tide::main]
fn main(root: Root) {
    let take = move || { let Root { fs, .. } = root; fs };
    let _ = take();
}
