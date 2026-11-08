use std::time::Duration;
#[sim_test(seeds = 20)]
fn case1(sim: std::sim::SimRoot) {
    let mut answers = [true, true, false, true, false, false].into_iter();
    assert_eq!(
        watch(&sim.clock, Duration::from_millis(100), 2, || answers
            .next()
            .unwrap())
        .unwrap(),
        Report {
            calls: 6,
            last_ok: Some(3),
            elapsed: Duration::from_millis(500)
        }
    );
}
#[sim_test(seeds = 20)]
fn case2(sim: std::sim::SimRoot) {
    let mut first = true;
    assert_eq!(
        watch(&sim.clock, Duration::from_millis(100), 1, || {
            if first {
                first = false;
                sim.clock.sleep(Duration::from_millis(250)).unwrap();
                true
            } else {
                false
            }
        })
        .unwrap(),
        Report {
            calls: 2,
            last_ok: Some(0),
            elapsed: Duration::from_millis(300)
        }
    );
}
#[sim_test(seeds = 20)]
fn case3(sim: std::sim::SimRoot) {
    let r = watch(&sim.clock, Duration::from_millis(100), 1, || false).unwrap();
    assert_eq!((r.calls, r.last_ok), (1, None));
    assert_eq!(r.elapsed, Duration::ZERO);
}
#[test]
#[should_panic]
fn case4() {
    let sim = Root::sim(0);
    let _ = watch(&sim.clock, Duration::from_millis(100), 0, || false);
}
