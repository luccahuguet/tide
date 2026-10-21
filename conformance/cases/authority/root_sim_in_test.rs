//@ rules: ROOT-1, ROOT-3, ROOT-5
// A simulated root in a test (ROOT-1 allows the path `Root::sim`), which calls the function
// that takes the capability, not `main`.
use std::fs::Fs;

pub fn has_data(fs: &Fs) -> bool {
    fs.read("/data").is_ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_data() {
        let sim = Root::sim(7);
        sim.world.file("/data", "x");
        assert!(has_data(&sim.fs));
    }
}
