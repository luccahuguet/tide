type Fixture = std::fs::Fs;
fn with_fs(mut f: impl FnMut(&Fixture)) {
    for seed in 0..20 {
        let sim = Root::sim(seed);
        sim.fs.create_dir_all("/fixture").unwrap();
        let fs = sim.fs.dir("/fixture", std::fs::Mode::ReadWrite).unwrap();
        f(&fs);
    }
}
fn fs_write(fs: &Fixture, p: &str, text: &str) {
    fs.write(p, text).unwrap();
}
fn fs_read(fs: &Fixture, p: &str) -> String {
    fs.read_to_string(p).unwrap()
}
fn fs_exists(fs: &Fixture, p: &str) -> bool {
    fs.exists(p).unwrap()
}
fn fs_mkdir(fs: &Fixture, p: &str) {
    fs.create_dir_all(p).unwrap();
}
