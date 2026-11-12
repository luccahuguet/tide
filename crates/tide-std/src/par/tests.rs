use super::*;
use std::panic::{AssertUnwindSafe, catch_unwind};
use std::sync::atomic::AtomicUsize;
use std::sync::{Arc, Barrier};

fn caught(f: impl FnOnce()) -> String {
    let payload = catch_unwind(AssertUnwindSafe(f)).unwrap_err();
    payload
        .downcast::<String>()
        .map(|s| *s)
        .unwrap_or_else(|p| p.downcast::<&str>().map(|s| s.to_string()).unwrap())
}

#[test]
fn values_boundaries_stability_and_owned_elements() {
    for workers in [1, 4] {
        let budget = Budget::new(workers);
        for n in [0, 1, 2, 7, 65, 1000] {
            let xs: Vec<_> = (0..n).rev().collect();
            let mapped = map(&xs, &|x| x * 2, &budget);
            assert_eq!(mapped, xs.iter().map(|x| x * 2).collect::<Vec<_>>());
            let mut updated = xs.clone();
            for_each(&mut updated, &|x| *x *= 2, &budget);
            assert_eq!(mapped, updated);
            sort(&mut updated, &budget);
            let mut expected = mapped;
            expected.sort();
            assert_eq!(updated, expected);
        }
        #[derive(Debug, Eq)]
        struct Entry(usize, String);
        impl PartialEq for Entry {
            fn eq(&self, other: &Self) -> bool {
                self.0 == other.0
            }
        }
        impl PartialOrd for Entry {
            fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
                Some(self.cmp(other))
            }
        }
        impl Ord for Entry {
            fn cmp(&self, other: &Self) -> std::cmp::Ordering {
                self.0.cmp(&other.0)
            }
        }
        let mut xs: Vec<_> = (0..150).map(|i| Entry(i % 7, i.to_string())).collect();
        sort(&mut xs, &budget);
        for key in 0..7 {
            assert_eq!(
                xs.iter()
                    .filter(|x| x.0 == key)
                    .map(|x| x.1.parse::<usize>().unwrap())
                    .collect::<Vec<_>>(),
                (0..150).filter(|i| i % 7 == key).collect::<Vec<_>>()
            );
        }
    }
}

#[test]
fn lowest_panic_sink_prefix_stop_and_token_return() {
    let budget = Budget::new(4);
    let barrier = Barrier::new(2);
    let (result, records) = crate::__sink::capture(|| {
        assert_eq!(
            caught(|| {
                fork(
                    &budget,
                    || {
                        barrier.wait();
                        crate::__sink::out(format_args!("left"), true);
                        panic!("left");
                    },
                    || {
                        crate::__sink::err(format_args!("right"), true);
                        barrier.wait();
                        panic!("right");
                    },
                );
            }),
            "left"
        );
    });
    result.unwrap();
    assert_eq!(
        records
            .iter()
            .map(|r| std::str::from_utf8(&r.message).unwrap())
            .collect::<Vec<_>>(),
        ["left\n"]
    );
    assert_eq!(budget.free.load(Ordering::Relaxed), 3);
    let caller = thread::current().id();
    let (_, helper) = fork(&budget, || (), || thread::current().id());
    assert_ne!(caller, helper);

    let executed = AtomicUsize::new(0);
    let (result, records) = crate::__sink::capture(|| {
        assert_eq!(
            caught(|| {
                blocks((0..10000).map(|i| i), 4, &budget, &|i| {
                    executed.fetch_add(1, Ordering::Relaxed);
                    crate::__sink::out(format_args!("{i}"), true);
                    if i == 0 {
                        panic!("zero");
                    }
                });
            }),
            "zero"
        );
    });
    assert!(executed.load(Ordering::Relaxed) < 10000);
    result.unwrap();
    assert_eq!(records.len(), 1);
    assert_eq!(records[0].message, b"0\n");
}

#[test]
fn out_of_order_blocks_choose_lowest_panic_and_nested_sink_order() {
    let budget = Budget::new(4);
    let barrier = Barrier::new(2);
    let (result, records) = crate::__sink::capture(|| {
        assert_eq!(
            caught(|| {
                blocks(0..2, 2, &budget, &|i| {
                    barrier.wait();
                    if i == 0 {
                        thread::sleep(std::time::Duration::from_millis(10));
                    }
                    crate::__sink::out(format_args!("{i}"), true);
                    panic!("panic {i}");
                });
            }),
            "panic 0"
        );
    });
    result.unwrap();
    assert_eq!(records.len(), 1);
    assert_eq!(records[0].message, b"0\n");
    let (result, records) = crate::__sink::capture(|| {
        fork(
            &budget,
            || {
                fork(
                    &budget,
                    || crate::__sink::out(format_args!("0"), true),
                    || crate::__sink::err(format_args!("1"), true),
                );
            },
            || crate::__sink::out(format_args!("2"), true),
        );
    });
    result.unwrap();
    assert_eq!(
        records
            .iter()
            .map(|r| (r.stream, std::str::from_utf8(&r.message).unwrap()))
            .collect::<Vec<_>>(),
        [("out", "0\n"), ("err", "1\n"), ("out", "2\n")]
    );
}

#[test]
fn recursive_forks_share_a_bounded_budget() {
    fn tree(depth: usize, budget: &Budget, ids: &Mutex<Vec<thread::ThreadId>>) -> usize {
        ids.lock().unwrap().push(thread::current().id());
        assert!(budget.free.load(Ordering::Relaxed) < budget.workers);
        if depth == 0 {
            return 1;
        }
        let (a, b) = fork(
            budget,
            || tree(depth - 1, budget, ids),
            || tree(depth - 1, budget, ids),
        );
        a + b
    }
    let budget = Budget::new(4);
    assert_eq!(tree(8, &budget, &Mutex::new(Vec::new())), 256);
    assert_eq!(budget.free.load(Ordering::Relaxed), 3);
}

#[test]
fn simulation_stays_on_calling_task_and_emits_only_ordered_sinks() {
    let sim = crate::Root::sim(23);
    let caller = thread::current().id();
    let (a, b) = join(|| thread::current().id(), || thread::current().id());
    assert_eq!((a, b), (caller, caller));
    let mut xs: Vec<_> = (0..80).rev().collect();
    assert!(
        xs.par_map(|_| thread::current().id())
            .iter()
            .all(|id| *id == caller)
    );
    xs.par_for(|_| assert_eq!(thread::current().id(), caller));
    xs.par_sort();
    assert!(xs.is_sorted());
    join(
        || crate::__sink::out(format_args!("left"), true),
        || {
            [0, 1, 2].par_for(|i| crate::__sink::out(format_args!("{i}"), true));
        },
    );
    assert_eq!(
        sim.records()
            .iter()
            .map(|r| r.message.as_str())
            .collect::<Vec<_>>(),
        ["left", "0", "1", "2"]
    );
    assert!(
        sim.trace()
            .lines()
            .all(|line| line.contains("\"kind\":\"sink\""))
    );
    drop(sim);
    assert!(!crate::task::is_simulating());
}

#[test]
fn comparator_panic_preserves_every_owned_element() {
    struct Bomb(usize, Arc<AtomicUsize>);
    impl Eq for Bomb {}
    impl PartialEq for Bomb {
        fn eq(&self, other: &Self) -> bool {
            self.0 == other.0
        }
    }
    impl PartialOrd for Bomb {
        fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
            Some(self.cmp(other))
        }
    }
    impl Ord for Bomb {
        fn cmp(&self, other: &Self) -> std::cmp::Ordering {
            if self.0 == 7 || other.0 == 7 {
                panic!("compare");
            }
            self.0.cmp(&other.0)
        }
    }
    impl Drop for Bomb {
        fn drop(&mut self) {
            self.1.fetch_add(1, Ordering::Relaxed);
        }
    }
    let drops = Arc::new(AtomicUsize::new(0));
    let mut xs: Vec<_> = (0..100).rev().map(|i| Bomb(i, drops.clone())).collect();
    assert_eq!(caught(|| sort(&mut xs, &Budget::new(4))), "compare");
    let mut ids: Vec<_> = xs.iter().map(|x| x.0).collect();
    ids.sort();
    assert_eq!(ids, (0..100).collect::<Vec<_>>());
    drop(xs);
    assert_eq!(drops.load(Ordering::Relaxed), 100);
}

#[test]
fn result_cleanup_panics_abort() {
    const ROLE: &str = "TIDE_PAR_RESULT_CLEANUP_TEST";
    if let Ok(mode) = std::env::var(ROLE) {
        struct Bomb;
        impl Drop for Bomb {
            fn drop(&mut self) {
                panic!("cleanup");
            }
        }
        let _sim = (mode == "map-simulated").then(|| crate::Root::sim(0));
        match mode.as_str() {
            "map-real" | "map-simulated" => {
                let _ = [0, 1].par_map(|i| {
                    if *i == 1 {
                        panic!("winner");
                    }
                    Bomb
                });
            }
            "map-later" => {
                let barrier = Barrier::new(2);
                let _ = map(
                    &[0, 1],
                    &|i| {
                        barrier.wait();
                        if *i == 0 {
                            panic!("winner");
                        }
                        Bomb
                    },
                    &Budget::new(2),
                );
            }
            "join" => {
                let caller = thread::current().id();
                fork(
                    &Budget::new(2),
                    || panic!("winner"),
                    || {
                        assert_ne!(thread::current().id(), caller);
                        Bomb
                    },
                );
            }
            _ => panic!("unknown cleanup test mode"),
        }
        unreachable!("panicking region returned");
    }
    for mode in ["map-real", "map-simulated", "map-later", "join"] {
        let child = std::process::Command::new("timeout")
            .arg("10")
            .arg(std::env::current_exe().unwrap())
            .args([
                "--exact",
                "par::tests::result_cleanup_panics_abort",
                "--nocapture",
            ])
            .env(ROLE, mode)
            .env("RUST_BACKTRACE", "0")
            .stdin(std::process::Stdio::null())
            .output()
            .unwrap();
        let report = String::from_utf8_lossy(&child.stderr);
        assert_eq!(
            std::os::unix::process::ExitStatusExt::signal(&child.status),
            Some(6),
            "{mode}: {}: {report}",
            child.status
        );
        assert!(
            report.contains("winner") && report.contains("cleanup"),
            "{mode}: {report}"
        );
    }
}

#[test]
fn sort_cutoff_preserves_results_and_ordered_comparisons() {
    struct Observed(usize, Arc<Mutex<Vec<thread::ThreadId>>>);
    impl Eq for Observed {}
    impl PartialEq for Observed {
        fn eq(&self, other: &Self) -> bool {
            self.0 == other.0
        }
    }
    impl PartialOrd for Observed {
        fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
            Some(self.cmp(other))
        }
    }
    impl Ord for Observed {
        fn cmp(&self, other: &Self) -> std::cmp::Ordering {
            let id = thread::current().id();
            let mut threads = self.1.lock().unwrap();
            if !threads.contains(&id) {
                threads.push(id);
            }
            drop(threads);
            crate::__sink::out(format_args!("{},{}", self.0, other.0), true);
            self.0.cmp(&other.0)
        }
    }
    for len in [4095, 4096, 4097, 8192] {
        let input: Vec<_> = (0..len).map(|i| i * 37 % len).collect();
        let mut expected = input.clone();
        expected.sort();
        let mut baseline = None;
        for workers in [1, 4] {
            let threads = Arc::new(Mutex::new(Vec::new()));
            let mut xs: Vec<_> = input
                .iter()
                .map(|&n| Observed(n, threads.clone()))
                .collect();
            let (result, records) = crate::__sink::capture(|| sort(&mut xs, &Budget::new(workers)));
            result.unwrap();
            assert_eq!(xs.iter().map(|x| x.0).collect::<Vec<_>>(), expected);
            let comparisons: Vec<_> = records.into_iter().map(|r| r.message).collect();
            if let Some(baseline) = &baseline {
                assert_eq!(&comparisons, baseline, "length {len}");
            } else {
                baseline = Some(comparisons);
            }
            let threads = threads.lock().unwrap();
            if workers == 1 || len < 4096 {
                assert_eq!(*threads, [thread::current().id()], "length {len}");
            } else {
                assert!(threads.len() > 1, "length {len} must exercise a helper");
            }
        }
    }
}

#[test]
fn simulated_panics_keep_only_the_sequential_sink_prefix() {
    fn element(i: &i32) -> i32 {
        crate::__sink::out(format_args!("{i}"), true);
        if *i == 1 {
            panic!("winner");
        }
        *i
    }
    for region in ["join", "map", "for"] {
        let sim = crate::Root::sim(7);
        assert_eq!(
            caught(|| match region {
                "join" => {
                    join(
                        || {
                            crate::__sink::out(format_args!("left"), true);
                            panic!("winner");
                        },
                        || crate::__sink::out(format_args!("right"), true),
                    );
                }
                "map" => {
                    [0, 1, 2].par_map(element);
                }
                _ => [0, 1, 2].par_for(|i| {
                    element(i);
                }),
            }),
            "winner"
        );
        let messages: Vec<_> = sim.records().into_iter().map(|r| r.message).collect();
        assert_eq!(
            messages,
            if region == "join" {
                vec!["left"]
            } else {
                vec!["0", "1"]
            }
        );
    }
}
