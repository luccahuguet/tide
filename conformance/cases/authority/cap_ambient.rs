//@ rules: CAP-1
// World operations exist only as capability methods; Tide's std has no ambient ones.
pub fn config() -> String {
    std::fs::read_to_string("/etc/app.toml").unwrap_or_default() //~ E0425 FX0501
}

pub fn started() -> std::time::Instant {
    std::time::Instant::now() //~ E0599 FX0501
}
