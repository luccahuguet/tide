type Fixture = std::path::Path;
fn with_fs(mut f: impl FnMut(&Fixture)) {
    struct Temp(std::path::PathBuf);
    impl Drop for Temp {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.0);
        }
    }
    let dir = Temp(std::env::temp_dir().join(format!("e1-checked-{}", std::process::id())));
    std::fs::create_dir_all(&dir.0).unwrap();
    f(&dir.0);
}
fn fs_write(fs: &Fixture, p: &str, text: &str) {
    std::fs::write(fs.join(p), text).unwrap();
}
fn fs_read(fs: &Fixture, p: &str) -> String {
    std::fs::read_to_string(fs.join(p)).unwrap()
}
fn fs_exists(fs: &Fixture, p: &str) -> bool {
    fs.join(p).exists()
}
fn fs_mkdir(fs: &Fixture, p: &str) {
    std::fs::create_dir_all(fs.join(p)).unwrap();
}
