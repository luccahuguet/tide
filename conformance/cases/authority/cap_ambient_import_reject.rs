//@ rules: CAP-1
// Missing Tide types keep their Rust source identity through renames and re-exports
// (`UdpSocket`; Tide's `Command` exists, so `Command::new` is E0599 in cap_ambient_paths_reject).
use std::net::UdpSocket as Cmd; //~ E0432
pub fn renamed() { let _ = Cmd::bind("tool"); } //~ TD0501

mod exports {
    pub use std::net::UdpSocket as Launch; //~ E0432
}
pub fn reexported() { let _ = exports::Launch::bind("tool"); } //~ TD0501
use exports::Launch as LaunchAgain;
pub fn reimported() { let _ = LaunchAgain::bind("tool"); } //~ TD0501

pub fn scoped() {
    use std::net::UdpSocket as Local; //~ E0432
    let _ = Local::bind("tool"); //~ TD0501
}

pub fn shadowed() {
    use missing_module::Command as Cmd; //~ E0432
    let _ = Cmd::new("data");
}

mod globbed {
    mod exports {
        pub use std::net::UdpSocket as Cmd; //~ E0432
    }
    use exports::*;
    pub fn construct() { let _ = Cmd::bind("tool"); } //~ TD0501
    mod relay { pub use super::exports::*; }
    use relay::*;
    pub fn relayed() { let _ = relay::Cmd::bind("tool"); } //~ TD0501
    pub fn shadowed() {
        use missing_module::Command as Cmd; //~ E0432
        let _ = Cmd::new("data");
    }
}

mod ambiguous_globs {
    mod left { pub use std::net::UdpSocket as Cmd; } //~ E0432
    mod right { pub use std::net::SocketAddrV4 as Cmd; } //~ E0432
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
