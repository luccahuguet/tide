//@ rules: ADMIT-1
// Red team (fx-qlek.5): Rust std routes that reach the host without a capability: a Unix socket,
// a backtrace (which reads the environment and the binary's debug information), and DNS through
// `to_socket_addrs`. Tide's std exports none of them: `std::os` and `std::backtrace` do not
// resolve, and Tide's own `ToSocketAddrs` is sealed and has no method.
use std::net::ToSocketAddrs;

pub fn connect() -> bool {
    std::os::unix::net::UnixStream::connect("/run/bus").is_ok() //~ E0433
}

pub fn trace() -> String {
    std::backtrace::Backtrace::force_capture().to_string() //~ E0433
}

pub fn resolve() -> usize {
    ("example.com", 80).to_socket_addrs().map(|a| a.count()).unwrap_or(0) //~ E0599
}
