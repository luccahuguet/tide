pub fn f() -> Vec<u8> {
    std::fs::read("/etc/hostname").unwrap()
}
