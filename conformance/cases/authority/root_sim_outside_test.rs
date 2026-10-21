//@ rules: ROOT-1, ROOT-3
// `Root::sim` outside test code: ROOT-3 rejects it, though ROOT-1 allows the path.
pub fn demo() -> bool {
    let sim = Root::sim(7); //~ FX0504
    sim.fs.read("/data").is_ok()
}
