//@ rules: ADMIT-1
// Tide's `HashMap` and `HashSet` are its std's own types (03 GLOBAL-5), so building one with
// `new()`, by `collect`, from an array, with a key from `Rand`, or with a caller's hasher is
// admitted, and so are Rust's methods on them. Rust's own maps draw an OS key and are not
// (`policy.rs`).
use std::collections::hash_map::RandomState;
use std::collections::{HashMap, HashSet};
use std::hash::{BuildHasherDefault, Hasher};
use std::rand::Rand;

pub fn count<'t>(words: &[&'t str]) -> HashMap<&'t str, usize> {
    let mut counts = HashMap::new();
    for w in words {
        *counts.entry(*w).or_insert(0) += 1;
    }
    counts.insert("", counts.len()); // a two-phase borrow, as in Rust
    counts
}

pub fn distinct(words: &[&str]) -> usize {
    let seen: HashSet<&str> = words.iter().copied().collect();
    seen.len() + HashSet::from(["spam", "junk"]).len()
}

#[derive(Default)]
pub struct Fnv(u64);
impl Hasher for Fnv {
    fn finish(&self) -> u64 {
        self.0
    }
    fn write(&mut self, bytes: &[u8]) {
        for b in bytes {
            self.0 = (self.0 ^ u64::from(*b)).wrapping_mul(0x100_0000_01b3);
        }
    }
}

pub fn own_hasher(pairs: Vec<(u32, u32)>) -> usize {
    let a: HashMap<u32, u32, BuildHasherDefault<Fnv>> = pairs.into_iter().collect();
    let b: HashMap<u32, u32, BuildHasherDefault<Fnv>> = HashMap::default();
    a.len() + b.len() + HashMap::from([(1, 2)]).len()
}

pub fn keyed(rand: &Rand, pairs: Vec<(String, u64)>) -> u64 {
    let mut by_name = HashMap::with_hasher(RandomState::from_rand(rand));
    by_name.extend(pairs);
    let mut total = 0;
    for (_, v) in &by_name {
        total += v;
    }
    total + by_name.get("base").copied().unwrap_or(0)
}
