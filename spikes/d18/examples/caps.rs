// A Tide program as a library crate: `pub fn main(root: Root)`.
// scripts/c1-facade.sh generates the entry crate that calls it.
use std::collections::{BTreeMap, HashMap};
use std::fs::Fs;
use std::sync::{LockError, Shared, channel};
use std::time::{Clock, Duration};
use std::task::Tasks;

#[derive(Debug, Clone, PartialEq)]
struct Account {
    balance: u64,
}

fn files(fs: &Fs) -> std::io::Result<()> {
    fs.create_dir_all("data")?;
    let data = fs.dir("data")?; // attenuation: confined below ./data
    data.write("hello.txt", b"hi from Fs")?;
    assert_eq!(data.read_to_string("hello.txt")?, "hi from Fs");
    let escape = data.read("../caps.rs").unwrap_err();
    println!("fs: wrote and read data/hello.txt; ../ escape -> {:?}", escape.kind());
    Ok(())
}

fn timing(clock: &Clock) {
    let t0 = clock.now();
    clock.sleep(Duration::from_millis(2));
    let t1 = clock.now();
    assert!(t1 >= t0 + Duration::from_millis(2));
    println!("clock: monotonic, slept >= 2ms");
}

fn concurrency(tasks: &Tasks) {
    // Scoped children borrow `xs` and `tasks`.
    let xs: Vec<u64> = (1..=100).collect();
    let (lo, hi) = xs.split_at(50);
    let total = tasks.scope(|s| {
        let a = s.spawn(|| lo.iter().sum::<u64>());
        let b = s.spawn(|| hi.iter().sum::<u64>());
        a.join() + b.join()
    });
    assert_eq!(total, 5050);

    let acct = Shared::new(Account { balance: 0 }, tasks);
    tasks.scope(|s| {
        for _ in 0..4 {
            let acct = acct.clone();
            s.spawn(move || {
                for _ in 0..1000 {
                    acct.with(|a| a.balance += 1).unwrap();
                }
            });
        }
    });
    assert_eq!(acct.with(|a| a.balance), Ok(4000));

    let other = Shared::new(Account { balance: 10 }, tasks);
    let moved = Shared::with2(&acct, &other, |a, b| {
        a.balance -= 1000;
        b.balance += 1000;
        b.balance
    });
    assert_eq!(moved, Ok(1010));
    assert_eq!(Shared::with2(&acct, &acct.clone(), |_, _| ()), Err(LockError::Aliased));

    let (tx, rx) = channel(tasks);
    tasks.scope(|s| {
        for id in 0..3u32 {
            let tx = tx.clone();
            s.spawn(move || tx.send(id * 10).unwrap());
        }
    });
    drop(tx);
    let mut got: Vec<u32> = rx.iter().collect();
    got.sort();
    assert_eq!(got, vec![0, 10, 20]);
    println!("tasks: scope sum {total}, Shared 4000 -> with2 {moved:?}, aliased -> Err(Aliased), channel {got:?}");
}

fn pure_only<F: Fn(u64) -> u64 + Pure>(f: F) -> u64 {
    f(20) + 1
}

fn collections() {
    let mut m = HashMap::new();
    for w in "the quick brown fox jumps over the lazy dog the end".split(' ') {
        *m.entry(w).or_insert(0) += 1;
    }
    let order: Vec<&str> = m.keys().copied().collect();
    let b: BTreeMap<_, _> = m.iter().map(|(k, v)| (*k, *v)).collect();
    println!("hashmap: the={} order={:?} (fixed key: same every run)", m["the"], order);
    println!("btreemap: {b:?}; pure_only: {}", pure_only(|x| x * 2));
}

pub fn main(root: Root) -> Result<(), std::io::Error> {
    files(&root.fs)?;
    timing(&root.clock);
    concurrency(&root.tasks);
    collections();
    Ok(())
}
