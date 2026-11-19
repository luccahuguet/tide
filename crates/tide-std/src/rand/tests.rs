use super::os::seam::{self, Reply};
use super::Rand;
use crate::collections::HashMap;
use crate::collections::hash_map::RandomState;
use crate::task::TaskError;
use std::collections::BTreeSet;
use std::panic::{AssertUnwindSafe, catch_unwind};

const TIDERAND: u64 = 0x7469_6465_7261_6e64;

// An independent SplitMix64, so RAND-3 does not test the scheduler's helper against itself.
fn reference(seed: u64, n: usize) -> Vec<u64> {
    let mut state = seed ^ TIDERAND;
    (0..n)
        .map(|_| {
            state = state.wrapping_add(0x9e37_79b9_7f4a_7c15);
            let mut z = state;
            z = (z ^ (z >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
            z = (z ^ (z >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
            z ^ (z >> 31)
        })
        .collect()
}

// Each `rand` event's operation and word count, in order.
fn words(trace: &str) -> Vec<(String, u64)> {
    trace
        .lines()
        .filter(|l| l.contains("\"kind\":\"rand\""))
        .map(|l| {
            let field = |key: &str| l.split(key).nth(1).unwrap().split([',', '"', '}']).next().unwrap().to_owned();
            (field("\"operation\":\""), field("\"words\":").parse().unwrap())
        })
        .collect()
}

// Iterated to its end: endpoints `0..=255`, but empty.
fn exhausted() -> std::ops::RangeInclusive<u8> {
    let mut range = u8::MAX..=u8::MAX;
    assert_eq!(range.next(), Some(u8::MAX));
    assert!(range.is_empty());
    range
}

fn panics(f: impl FnOnce()) -> String {
    let payload = catch_unwind(AssertUnwindSafe(f)).expect_err("expected a panic");
    payload
        .downcast_ref::<String>()
        .cloned()
        .or_else(|| payload.downcast_ref::<&str>().map(|s| s.to_string()))
        .unwrap_or_default()
}

/// RAND-3: simulated words, byte fills, rejection, bools and shuffles follow research/53 §2.1.
#[test]
fn simulated_values_follow_the_stated_methods() {
    let sim = crate::Root::sim(7);
    let w = reference(7, 6);
    assert_eq!([sim.rand.next_u64(), sim.rand.next_u64()], [w[0], w[1]]);
    let mut bytes = [0; 11];
    sim.rand.fill_bytes(&mut bytes);
    assert_eq!(bytes[..8], w[2].to_le_bytes());
    assert_eq!(bytes[8..], w[3].to_le_bytes()[..3]);
    for (p, x) in [(0.25, w[4]), (0.75, w[5])] {
        assert_eq!(sim.rand.random_bool(p), x < (p * 18_446_744_073_709_551_616.0) as u64);
    }
    let ops = words(&sim.trace());
    let expected = [("rand.next_u64", 1), ("rand.next_u64", 1), ("rand.fill_bytes", 2), ("rand.random_bool", 1), ("rand.random_bool", 1)];
    assert_eq!(ops, expected.map(|(o, n)| (o.to_owned(), n)));
    drop(sim);

    // `low + x % s` through the signed offset, then Fisher-Yates from the end over
    // `random_range(0..=i)`, each against the reference words in order.
    let sim = crate::Root::sim(11);
    let mut w = reference(11, 64).into_iter();
    let mut below = |s: u64| loop {
        let x = w.next().unwrap();
        if x >= s.wrapping_neg() % s {
            break x % s;
        }
    };
    assert_eq!(sim.rand.random_range(-5i8..5), -5 + below(10) as i8);
    let mut expected = [0, 1, 2, 3];
    for i in (1..4).rev() {
        expected.swap(i, below(i as u64 + 1) as usize);
    }
    let mut v = [0, 1, 2, 3];
    sim.rand.shuffle(&mut v);
    assert_eq!(v, expected);
    drop(sim);

    // `0..=2^63`: s = 2^63 + 1, t = 2^63 - 1. A seed whose first word is below t redraws.
    let s = (1u64 << 63) + 1;
    let t = s.wrapping_neg() % s;
    assert_eq!(t, (1 << 63) - 1);
    let seed = (0..64).find(|&seed| reference(seed, 1)[0] < t).expect("a seed that rejects");
    let w = reference(seed, 64);
    let accepted = w.iter().copied().find(|&x| x >= t).unwrap();
    let sim = crate::Root::sim(seed);
    assert_eq!(sim.rand.random_range(0..=1u64 << 63), accepted % s);
    let taken = w.iter().position(|&x| x >= t).unwrap() as u64 + 1;
    assert!(taken >= 2);
    assert_eq!(words(&sim.trace()), [("rand.random_range".to_owned(), taken)]);
    drop(sim);

    let orders: BTreeSet<_> = (0..1000)
        .map(|seed| {
            let sim = crate::Root::sim(seed);
            let mut v = [0, 1, 2, 3];
            sim.rand.shuffle(&mut v);
            v
        })
        .collect();
    assert_eq!(orders.len(), 24);
}

/// RAND-2's word-count and before-drawing clauses on the simulated backend, through the
/// `rand` event: a rejected argument takes no point and no event; empty work takes no word.
#[test]
fn simulated_edge_cases_take_no_word() {
    let sim = crate::Root::sim(3);
    let rand = &sim.rand;
    for message in [
        panics(|| { rand.random_range(5..5); }),
        panics(|| { rand.random_range(5..=4); }),
        panics(|| { rand.random_range(exhausted()); }),
        panics(|| { rand.random_bool(f64::NAN); }),
        panics(|| { rand.random_bool(1.1); }),
    ] {
        assert!(message.starts_with("Rand::random_"), "{message}");
    }
    assert!(sim.trace().is_empty(), "{}", sim.trace());
    rand.fill_bytes(&mut []);
    rand.shuffle(&mut [0; 0]);
    rand.shuffle(&mut [9]);
    assert_eq!(rand.random_range(-3i8..=-3), -3);
    rand.random_range(i64::MIN..=i64::MAX);
    // A word for every `p`, so the stream position does not depend on `p`.
    assert!(!rand.random_bool(0.0));
    assert!(rand.random_bool(1.0));
    let expected = [
        ("rand.fill_bytes", 0),
        ("rand.shuffle", 0),
        ("rand.shuffle", 0),
        ("rand.random_range", 1),
        ("rand.random_range", 1),
        ("rand.random_bool", 1),
        ("rand.random_bool", 1),
    ];
    assert_eq!(words(&sim.trace()), expected.map(|(o, n)| (o.to_owned(), n)));
}

/// RAND-4: the seam stands in for `getrandom(2)`. Words are its bytes little-endian; a short
/// count and `EINTR` continue until full; another error panics naming it; RAND-2's
/// before-drawing and no-word clauses make no call on the real backend.
// The requested lengths; every request passes flags 0 (research/53 §3).
fn lens(calls: &[seam::Request]) -> Vec<usize> {
    assert!(calls.iter().all(|(_, flags)| flags.is_empty()), "getrandom flags must be 0: {calls:?}");
    calls.iter().map(|&(len, _)| len).collect()
}

#[test]
fn real_draws_read_getrandom_through_the_seam() {
    let rand = Rand(None);
    let bytes: Vec<u8> = (1..=8).collect();
    let (x, calls) = seam::with(vec![Reply::Bytes(bytes.clone())], || rand.next_u64());
    assert_eq!((x, lens(&calls)), (u64::from_le_bytes(bytes.clone().try_into().unwrap()), vec![8]));

    let replies = vec![
        Reply::Bytes(vec![1, 2, 3]),
        Reply::Fail(rustix::io::Errno::INTR),
        Reply::Bytes(vec![4, 5, 6, 7, 8, 9, 10]),
        Reply::Bytes(vec![11, 12, 13]),
    ];
    let (filled, calls) = seam::with(replies, || {
        let mut dest = [0; 12];
        rand.fill_bytes(&mut dest);
        dest
    });
    assert_eq!(filled, [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12]);
    assert_eq!(lens(&calls), [12, 9, 9, 2]);

    let message = panics(|| {
        seam::with(vec![Reply::Fail(rustix::io::Errno::PERM)], || rand.next_u64());
    });
    assert!(message.contains("getrandom(2) failed with EPERM"), "{message}");

    // Each word is one 8-byte request: a one-value range still takes one, as on sim.
    let (_, calls) = seam::with(vec![Reply::Bytes(vec![0; 8])], || rand.random_range(4u8..5));
    assert_eq!(lens(&calls), [8]);
    let ((), calls) = seam::with(Vec::new(), || {
        rand.fill_bytes(&mut []);
        rand.shuffle(&mut [0; 0]);
        rand.shuffle(&mut [1]);
        for message in [
            panics(|| { rand.random_range(5..5); }),
            panics(|| { rand.random_range(5..=4); }),
            panics(|| { rand.random_range(exhausted()); }),
            panics(|| { rand.random_bool(-0.1); }),
        ] {
            assert!(message.starts_with("Rand::random_"), "{message}");
        }
    });
    assert!(calls.is_empty(), "{calls:?}");

    // Unseamed, the OS answers.
    assert_ne!(rand.next_u64(), rand.next_u64());
}

fn run(seed: u64, bytes: usize) -> (Vec<u64>, Vec<u64>, String) {
    let sim = crate::Root::sim(seed);
    let (rand, fs) = (&sim.rand, &sim.fs);
    let drawn = std::sync::Mutex::new(Vec::new());
    sim.tasks
        .scope::<_, TaskError, _>(|s| {
            for task in 0..2u64 {
                let drawn = &drawn;
                drop(s.spawn(move || {
                    let mut buf = vec![0; bytes];
                    rand.fill_bytes(&mut buf);
                    let _ = fs.read("missing");
                    let mut v: Vec<u64> = (0..5).collect();
                    rand.shuffle(&mut v);
                    let x = rand.random_range(task * 10..task * 10 + 7);
                    let b = rand.random_bool(0.5) as u64;
                    drawn.lock().unwrap().extend([rand.next_u64(), x, b, v[0]]);
                    Ok(())
                }));
            }
            Ok(())
        })
        .unwrap();
    let mut map = HashMap::with_hasher(RandomState::from_rand(rand));
    map.extend((0..16).map(|k| (k, ())));
    let order = map.keys().copied().collect();
    (drawn.into_inner().unwrap(), order, sim.trace())
}

/// RAND-1: two tasks draw with every method and `from_rand`; a seed replays its draws, trace
/// and map order exactly, seeds 0 and 1 differ from the first word, and the words a call takes
/// leave every other stream's draws (schedule and file latencies) unchanged.
#[test]
fn simulated_draws_replay_and_keep_other_streams() {
    let first = run(5, 8);
    assert_eq!(first, run(5, 8));
    assert_eq!(words(&first.2).iter().filter(|(o, _)| o == "rand.from_rand").count(), 1);
    assert_ne!(first.1, run(6, 8).1, "map order varies by seed");
    let firsts = [0, 1].map(|seed| crate::Root::sim(seed).rand.next_u64());
    assert_ne!(firsts[0], firsts[1]);

    // Eight bytes take one word, 800 take 100: only the `rand` lines differ.
    let other = |trace: &str| trace.lines().filter(|l| !l.contains("rand.")).collect::<Vec<_>>().join("\n");
    let wide = run(5, 800);
    assert_ne!(first.2, wide.2);
    assert_eq!(other(&first.2), other(&wide.2));
    assert!(other(&first.2).contains("\"latency\":"));
    assert!(other(&first.2).contains("\"kind\":\"sched\""));
}

/// RAND-1's identity clauses: a real `Rand` inside a simulation, and a simulated one after its
/// simulation ended, panic before drawing.
#[test]
fn draws_check_their_simulation_identity() {
    let stale = {
        let sim = crate::Root::sim(0);
        let message = panics(|| { Rand(None).next_u64(); });
        assert_eq!(message, "real capability used inside a simulation");
        let message = panics(|| { RandomState::from_rand(&Rand(None)); });
        assert_eq!(message, "real capability used inside a simulation");
        assert!(sim.trace().is_empty());
        sim.rand.clone()
    };
    let message = panics(|| { stale.next_u64(); });
    assert_eq!(message, "simulation handle used outside its simulation");
}
