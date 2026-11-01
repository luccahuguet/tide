//@ rules: ROOT-1, ADMIT-1
// `#[tide::main]` is the toolchain's entry point only on the binary's crate-root `main`. On any
// other `fn main`, in a module or nested in a function, the call it generates to Tide's std's
// runtime entry is code's own, which could mint a `Root` (RUN-1). They come first here, so a
// driver that took the first generated `main` for the entry point would admit them.
mod inner {
    #[tide::main] //~ FX0520
    pub fn main() {}
}

pub fn forge() {
    #[tide::main] //~ FX0520
    fn main(root: Root) {
        let _net = root.net;
    }
    let _ = main();
}

#[tide::main]
fn main() {}
