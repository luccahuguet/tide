//@ rules: CAP-1
// Missing Tide types keep their Rust source identity through renames and re-exports.
use std::process::Command as Cmd; //~ E0432
pub fn renamed() { let _ = Cmd::new("tool"); } //~ TD0501

mod exports {
    pub use std::process::Command as Launch; //~ E0432
}
pub fn reexported() { let _ = exports::Launch::new("tool"); } //~ TD0501
use exports::Launch as LaunchAgain;
pub fn reimported() { let _ = LaunchAgain::new("tool"); } //~ TD0501

pub fn scoped() {
    use std::process::Command as Local; //~ E0432
    let _ = Local::new("tool"); //~ TD0501
}

pub fn shadowed() {
    use missing_module::Command as Cmd; //~ E0432
    let _ = Cmd::new("data");
}

mod globbed {
    mod exports {
        pub use std::process::Command as Cmd; //~ E0432
    }
    use exports::*;
    pub fn construct() { let _ = Cmd::new("tool"); } //~ TD0501
    mod relay { pub use super::exports::*; }
    use relay::*;
    pub fn relayed() { let _ = relay::Cmd::new("tool"); } //~ TD0501
    pub fn shadowed() {
        use missing_module::Command as Cmd; //~ E0432
        let _ = Cmd::new("data");
    }
}

mod ambiguous_globs {
    mod left { pub use std::process::Command as Cmd; } //~ E0432
    mod right { pub use std::process::Stdio as Cmd; } //~ E0432
    use left::*;
    use right::*;
    pub fn construct() { let _ = Cmd::new("data"); }
}

mod cyclic_globs {
    mod left { pub use super::right::*; } //~ TD0914
    mod right { pub use super::left::*; } //~ TD0914
    use left::*;
    pub fn missing() { let _ = Cmd::new("data"); } //~ E0433
}
