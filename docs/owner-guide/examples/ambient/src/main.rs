// Owner's guide, chapter 1: code that reaches for the world without being handed authority.
// Admission rejects all four marked lines.

fn load_config() -> String {
    // Rust habit: read a file from anywhere. Tide's std has no such function.
    std::fs::read_to_string("/etc/app.toml").unwrap_or_default() // TD0501
}

fn started() -> std::time::Instant {
    // Rust habit: read the clock from anywhere. Time is a capability in Tide.
    std::time::Instant::now() // TD0501
}

fn helper(root: &Root) -> bool { // TD0503: only `main` may hold the whole `Root`
    root.fs.read("/etc/app.toml").is_ok()
}

#[tide::main]
fn main(root: Root) {
    let _ = (load_config(), started());
    let _ = helper(&root); // TD0503: `root` cannot be passed on whole
}
