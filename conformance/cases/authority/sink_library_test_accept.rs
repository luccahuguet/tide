//@ rules: SINK-8, CHANNEL-1
// A library's test harness is an executable: stdout and unbounded channels do not warn.
#[cfg(test)]
#[test]
fn report() {
    println!("test output");
    let sim = Root::sim(7);
    let _ = std::sync::mpsc::channel::<u32>(&sim.tasks);
}
