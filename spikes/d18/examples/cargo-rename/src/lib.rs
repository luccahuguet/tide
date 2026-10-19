// Under the facade, std::fs::read does not exist; Fs does.
pub fn f(fs: &std::fs::Fs) -> usize {
    fs.read("x").map(|v| v.len()).unwrap_or(0)
}

#[cfg(probe_bad)]
pub fn ambient() -> Vec<u8> {
    std::fs::read("x").unwrap()
}
