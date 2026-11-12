//@ rules: ROOT-1, ROOT-2
// Entry parameters may move and borrow fields directly.
#[tide::main]
fn main(Root { fs, ref clock, .. }: Root) {
    let _ = fs;
    let _ = clock.now();
}
