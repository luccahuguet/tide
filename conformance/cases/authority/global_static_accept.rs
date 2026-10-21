//@ rules: GLOBAL-1, GLOBAL-2, AFREE-2
// `Sync + Pure` statics whose values a `const fn` computes at compile time.
const fn kib(n: u32) -> u32 {
    n * 1024
}

pub static LIMIT: u32 = kib(64);
pub static NAMES: [&str; 2] = ["alpha", "beta"];
