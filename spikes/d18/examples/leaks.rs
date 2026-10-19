// Compiles under the facade: names it cannot hide. The admission checker
// (spikes/d18/admit) must reject each of these; the facade alone does not.
pub fn leaks() -> (u64, bool, i32, &'static str, f64) {
    let ambient_env = core::option_env!("HOME"); // build-time environment
    let _ = ambient_env;
    let path_io = std::path::Path::new("/etc/hostname").exists(); // Path inherent I/O
    let errno = std::io::Error::last_os_error().raw_os_error().unwrap_or(0); // reads errno
    let root = std::__rt::root(); // toolchain-only minting fn is nameable
    let _ = root.clock.now();
    let x = 0.5f64;
    (x.to_bits(), path_io, errno, core::include_str!("leaks.rs").lines().next().unwrap(), x.sin())
}
