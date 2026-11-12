//@ rules: CAP-1
// A same-spelled local type and its renames/re-exports are ordinary data constructors.
mod local {
    pub struct Command;
    impl Command { pub fn new(_: &str) -> Self { Self } }
}
use local::Command as Cmd;
mod exports { pub use super::local::Command as Launch; }
use exports::Launch as LaunchAgain;

pub fn construct() {
    let _ = Cmd::new("data");
    let _ = exports::Launch::new("data");
    let _ = LaunchAgain::new("data");
    {
        struct Cmd;
        impl Cmd { fn new(_: &str) -> Self { Self } }
        let _ = Cmd::new("data");
    }
}

mod globbed {
    mod exports { pub use crate::local::Command as Cmd; }
    use exports::*;
    mod relay { pub use super::exports::*; }
    use relay::*;
    pub fn construct() {
        let _ = Cmd::new("data");
        let _ = relay::Cmd::new("data");
        {
            struct Cmd;
            impl Cmd { fn new(_: &str) -> Self { Self } }
            let _ = Cmd::new("data");
        }
    }
}
