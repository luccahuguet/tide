//@ rules: RAND-1, RAND-2, CAP-1
// One value/panic suite runs unchanged against production real and simulated `Rand` (RAND-2);
// the build test compares the observations and replays a simulated seed's manifest (RAND-1).
use std::collections::HashMap;
use std::collections::hash_map::RandomState;
use std::rand::Rand;
use std::task::Tasks;

// Argument checks panic before drawing; an isolated task contains the panic.
fn panics(tasks: &Tasks, rand: &Rand, f: impl FnOnce(Rand) + Send + 'static) -> bool {
    let rand = rand.clone();
    tasks.spawn_isolated(move || f(rand)).is_err()
}

// Both backends must observe exactly this.
const EXPECTED: &str = "empty ranges panic: [true, true, true, true, true] | bad p panics: [true, true, true] | \
    one-value ranges: (7, -3, 18446744073709551615) | within bounds: true | bool 0.0, 1.0: false true | \
    shuffle: [] [9] permutation true moved true | filled: true | distinct words: true | from_rand map: true";

fn suite(rand: &Rand, tasks: &Tasks) -> String {
    let mut seen = Vec::new();
    let empty = [
        panics(tasks, rand, |r| { r.random_range(5..5u32); }),
        panics(tasks, rand, |r| { r.random_range(0..0usize); }),
        panics(tasks, rand, |r| { r.random_range(5..=4i64); }),
        panics(tasks, rand, |r| { r.random_range(i8::MAX..=i8::MIN); }),
        // Iterated to its end, an inclusive range is empty though its endpoints are `0..=255`.
        panics(tasks, rand, |r| {
            let mut range = u8::MAX..=u8::MAX;
            range.next();
            r.random_range(range);
        }),
    ];
    seen.push(format!("empty ranges panic: {empty:?}"));
    let p = [
        panics(tasks, rand, |r| { r.random_bool(-0.1); }),
        panics(tasks, rand, |r| { r.random_bool(1.1); }),
        panics(tasks, rand, |r| { r.random_bool(f64::NAN); }),
    ];
    seen.push(format!("bad p panics: {p:?}"));
    let one = (rand.random_range(7..8u8), rand.random_range(-3..=-3i16), rand.random_range(u64::MAX..=u64::MAX));
    seen.push(format!("one-value ranges: {one:?}"));
    // Full 64-bit ranges return a value; any value is in range.
    let _: (i64, u64) = (rand.random_range(i64::MIN..=i64::MAX), rand.random_range(0..=u64::MAX));
    let within = (0..1000).all(|_| (10..20).contains(&rand.random_range(10..20i32)))
        && (0..1000).all(|_| (-5..=5).contains(&rand.random_range(-5..=5isize)));
    seen.push(format!("within bounds: {within}"));
    seen.push(format!("bool 0.0, 1.0: {} {}", rand.random_bool(0.0), rand.random_bool(1.0)));
    let (mut none, mut single, mut many) = ([0u8; 0], [9u8], (0..50).collect::<Vec<u32>>());
    rand.shuffle(&mut none);
    rand.shuffle(&mut single);
    rand.shuffle(&mut many);
    let moved = many != (0..50).collect::<Vec<u32>>();
    many.sort();
    seen.push(format!("shuffle: {none:?} {single:?} permutation {} moved {moved}", many == (0..50).collect::<Vec<u32>>()));
    let mut bytes = [0u8; 64];
    rand.fill_bytes(&mut []);
    rand.fill_bytes(&mut bytes);
    seen.push(format!("filled: {}", bytes.iter().any(|&b| b != 0)));
    seen.push(format!("distinct words: {}", rand.next_u64() != rand.next_u64()));
    let mut map = HashMap::with_hasher(RandomState::from_rand(rand));
    map.insert(1, "one");
    seen.push(format!("from_rand map: {}", map.get(&1) == Some(&"one")));
    let seen = seen.join(" | ");
    assert_eq!(seen, EXPECTED);
    seen
}

#[tide::main]
fn main(root: Root) {
    let real = suite(&root.rand, &root.tasks);
    println!("RAND_VALUES:{real}");
    println!("RAND_RAN");
}

// RAND-1: the suite, then two tasks drawing; the runner's determinism check reruns each seed.
// Printed lines go to the simulation's sink, so they are in its trace.
#[sim_test(seeds = 4)]
fn simulated_rand(sim: std::sim::SimRoot) {
    let values = suite(&sim.rand, &sim.tasks);
    println!("RAND_VALUES:{values}");
    let rand = &sim.rand;
    let words = sim
        .tasks
        .scope::<_, std::task::TaskError, _>(|s| {
            let a = s.spawn(move || Ok(rand.next_u64()));
            let b = s.spawn(move || Ok(rand.random_range(0..1000u64)));
            Ok((a.join()?, b.join()?))
        })
        .unwrap();
    println!("RAND_WORDS:{words:?}");
}
