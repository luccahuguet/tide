// Benchmark pair `contracts`, Rust arm: `tide.rs` in plain Rust, with its contracts left out.
// Usage: `contracts [limit]`; the default limit is the small size the equivalence check runs.
const DEFAULT_LIMIT: usize = 20_000;

#[derive(Debug)]
pub enum RunError {
    Size(String),
}

pub fn isqrt(x: u64) -> u64 {
    if x < 2 {
        return x;
    }
    let mut r = x / 2;
    loop {
        let next = (r + x / r) / 2;
        if next >= r {
            return r;
        }
        r = next;
    }
}

/// `result[k]` is whether `k` is prime, for every `k` below `limit`.
pub fn sieve(limit: usize) -> Vec<bool> {
    let mut prime = vec![true; limit];
    for k in 0..limit.min(2) {
        prime[k] = false;
    }
    let bound = isqrt(u64::try_from(limit).unwrap());
    let mut i = 2;
    while u64::try_from(i).unwrap() <= bound {
        if prime[i] {
            let mut multiple = i * i;
            while multiple < limit {
                prime[multiple] = false;
                multiple += i;
            }
        }
        i += 1;
    }
    prime
}

pub fn insert_sorted(xs: &mut Vec<u64>, x: u64) {
    let mut i = 0;
    while i < xs.len() && xs[i] < x {
        i += 1;
    }
    xs.insert(i, x);
}

pub fn digit_sum(mut n: u64) -> u64 {
    let mut sum = 0;
    while n > 0 {
        sum += n % 10;
        n /= 10;
    }
    sum
}

pub fn run(limit: usize) {
    let prime = sieve(limit);
    let primes: Vec<u64> = (0..limit)
        .filter(|k| prime[*k])
        .map(|k| u64::try_from(k).unwrap())
        .collect();
    let mut roots = 0;
    for k in 0..limit {
        roots += isqrt(u64::try_from(k).unwrap());
    }
    let mut sums = Vec::new();
    for p in &primes {
        insert_sorted(&mut sums, digit_sum(*p));
    }
    println!("limit: {limit}");
    println!("primes: {}", primes.len());
    println!("largest prime: {:?}", primes.last());
    println!("isqrt sum: {roots}");
    println!(
        "digit sums: min {:?}, median {:?}, max {:?}",
        sums.first(),
        sums.get(sums.len() / 2),
        sums.last()
    );
}

fn main() -> Result<(), RunError> {
    let limit = match std::env::args().nth(1) {
        Some(arg) => arg.parse().map_err(|_| RunError::Size(arg))?,
        None => DEFAULT_LIMIT,
    };
    run(limit);
    Ok(())
}
