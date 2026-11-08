use std::time::Duration;
#[sim_test(seeds = 20)]
fn case1(sim: std::sim::SimRoot) {
    assert_eq!(
        run_batch(
            &sim.tasks,
            &sim.clock,
            &[10, 500, 30].map(Duration::from_millis),
            Duration::from_millis(100)
        ),
        vec![Outcome::Done, Outcome::TimedOut, Outcome::Done]
    );
    assert_eq!(sim.elapsed(), Duration::from_millis(100));
}
#[sim_test(seeds = 20)]
fn case2(sim: std::sim::SimRoot) {
    assert_eq!(
        run_batch(
            &sim.tasks,
            &sim.clock,
            &[5, 6].map(Duration::from_millis),
            Duration::from_secs(1)
        ),
        vec![Outcome::Done, Outcome::Done]
    );
    assert_eq!(sim.elapsed(), Duration::from_millis(6));
}
#[sim_test(seeds = 20)]
fn case3(sim: std::sim::SimRoot) {
    assert_eq!(
        run_batch(&sim.tasks, &sim.clock, &[], Duration::from_secs(1)),
        vec![]
    );
    assert_eq!(sim.elapsed(), Duration::ZERO);
}
