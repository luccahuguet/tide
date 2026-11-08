#[test]
fn case1() {
    assert_eq!(steps(1), Ok(0));
    assert_eq!(steps(6), Ok(8));
    assert_eq!(steps(27), Ok(111));
    assert_eq!(steps(6171), Err(TooLong));
}
#[test]
#[should_panic]
fn case2_zero() {
    let _ = steps(0);
}
#[test]
#[should_panic]
fn case2_overflow() {
    let _ = steps(u64::MAX);
}
#[sim_test(seeds = 20)]
fn case3(sim: std::sim::SimRoot) {
    assert_eq!(
        run_jobs(&sim.tasks, &[1, 6, 27, 0, u64::MAX, 6171], 2),
        Summary {
            ok: 119,
            done: 3,
            too_long: 1,
            panicked: 2
        }
    );
}
