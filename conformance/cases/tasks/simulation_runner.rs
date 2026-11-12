//@ rules: ROOT-1, ROOT-3, LOCK-4, LOCK-5
// Shared production/fixture surface; the driver build test executes selected entries.
#[cfg(test)]
mod simulation {
    use std::sim::{SimRoot, Sink};
    use std::sync::Shared;
    use std::task::TaskError;

    fn counter(sim: &SimRoot, split: bool) -> u32 {
        let cell = Shared::new(0, &sim.tasks);
        sim.tasks.scope::<_, TaskError, _>(|scope| {
            for _ in 0..2 {
                scope.spawn(|| {
                    if split {
                        let old = cell.with(|n| *n).unwrap();
                        cell.with(|n| *n = old + 1).unwrap();
                    } else {
                        cell.with(|n| *n += 1).unwrap();
                    }
                    if split { println!("worker updated"); }
                    Ok(())
                });
            }
            Ok(())
        }).unwrap();
        cell.with(|n| *n).unwrap()
    }

    #[sim_test(seeds = 20)]
    fn loses_updates(sim: SimRoot) {
        let count = counter(&sim, true);
        println!("counter={count}");
        assert_eq!(count, 2, "lost update");
    }

    #[sim_test(seeds = 20)]
    fn fixed_updates(sim: SimRoot) {
        assert_eq!(counter(&sim, false), 2);
    }

    #[sim_test]
    fn default_seeds(sim: SimRoot) {
        sim.tasks.scope::<_, TaskError, _>(|scope| {
            assert_eq!(scope.spawn(|| Ok(7)).join()?, 7);
            Ok(())
        }).unwrap();
    }

    #[sim_test(seeds = 1)]
    #[ignore]
    fn ignored_simulation(sim: SimRoot) {
        assert_eq!(counter(&sim, false), 2);
    }

    #[sim_test(seeds = 1)]
    fn r#loop(sim: SimRoot) {
        assert_eq!(counter(&sim, false), 2);
    }

    #[sim_test(seeds = 0)]
    fn empty_range(_sim: SimRoot) { panic!("empty seed range ran a body"); }

    fn observed(seed: u64) -> String {
        let sim = Root::sim(seed);
        assert_eq!(counter(&sim, false), 2);
        println!("quote=\" slash=\\ newline=\n nul=\0 café");
        eprint!("error");
        let records = sim.records();
        assert_eq!(records.len(), 3);
        assert!(matches!(records[0].sink, Sink::Out));
        assert!(matches!(records[1].sink, Sink::Out));
        assert!(matches!(records[2].sink, Sink::Err));
        assert_eq!(records[0].node, "main");
        assert_eq!(records[0].time, sim.elapsed());
        assert_eq!(records[2].message, "error");
        sim.trace()
    }

    #[test]
    fn plain_simulation() {
        assert_eq!(observed(3), observed(3));
        println!("PLAIN_SIM_RAN");
    }
}
