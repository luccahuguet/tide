// C3: contract and loop-invariant macros from the facade prelude (no imports).
// Built with `--test` against the facade; `main` is the erasure check.

#[requires(xs.len() > 0)]
#[ensures(result < xs.len())]
#[ensures(forall |i: usize| i < xs.len() ==> xs[result] >= xs[i])]
pub fn max_index(xs: &[i64]) -> usize {
    let mut best = 0;
    for i in 1..xs.len() {
        invariant!(best < i);
        invariant!(forall |j: usize| 0 <= j && j < i ==> xs[best] >= xs[j]);
        if xs[i] > xs[best] {
            best = i;
        }
    }
    best
}

#[ensures(dst.len() == old(dst.len()) + src.len())]
#[ensures(forall |i: usize| i < src.len() ==> dst[old(dst.len()) + i] == src[i])]
pub fn push_all(dst: &mut Vec<i64>, src: &[i64]) {
    for x in src {
        invariant!(dst.len() <= dst.capacity()); // general-iterator loop form
        dst.push(*x);
    }
}


#[ensures(dst.len() == old(dst.len()) + src.len())]
pub fn push_all_buggy(dst: &mut Vec<i64>, src: &[i64]) {
    for x in src.iter().skip(1) {
        dst.push(*x);
    }
}

#[requires(x > 0)]
#[ensures(result > x)]
pub fn bump_buggy(x: i32) -> i32 {
    if x > 10 {
        return x; // early return: ensures still checked
    }
    x + 1
}

#[ensures(result == (exists |i: usize| i < xs.len() && xs[i] == x))]
#[ensures(result ==> xs.len() > 0)]
pub fn contains(xs: &[i64], x: i64) -> bool {
    xs.iter().any(|y| *y == x)
}

#[ensures(*result == xs[0])]
pub fn first(xs: &[i32]) -> &i32 {
    &xs[0]
}

pub struct Counter {
    n: u32,
}

impl Counter {
    #[ensures(self.n == old(self.n) + 1)]
    pub fn incr(&mut self) -> Result<u32, String> {
        if self.n == u32::MAX {
            return Err("full".to_string());
        }
        self.n += 1;
        Ok(self.n)
    }
}

#[invariants]
pub fn empty_range_false() {
    for _ in 0..0 {
        invariant!(false);
    }
}

#[invariants]
pub fn while_false_at_exit() {
    let mut n = 0;
    while n < 1 {
        invariant!(n == 0);
        n += 1;
    }
}

#[invariants]
pub fn next_index(strict: bool) -> u32 {
    let mut seen = 0;
    for i in 0..3u32 {
        invariant!(strict ==> i < 3); // false at exit: i == 3 there
        invariant!(i <= 3 && seen == i);
        seen += 1;
    }
    seen
}

#[invariants]
pub fn skips_and_breaks() -> u32 {
    let mut odd = 0;
    'outer: for i in 0..10u32 {
        invariant!(i <= 10);
        if i % 2 == 0 {
            continue;
        }
        for j in 0..i {
            invariant!(j <= i);
            if j == 5 {
                break 'outer;
            }
        }
        odd += 1;
    }
    odd
}

#[invariants]
pub fn while_let(mut xs: Vec<u8>) -> u32 {
    let mut sum = 0u32;
    while let Some(x) = xs.pop() {
        invariant!(sum <= 255 * 10);
        sum += x as u32;
    }
    sum
}

#[test]
fn max_index_passes() {
    assert_eq!(max_index(&[3]), 0);
    assert_eq!(max_index(&[1, 9, 2, 9, -4]), 1);
    assert_eq!(max_index(&[-5, -1, -3]), 1);
}

#[test]
#[should_panic(expected = "requires violated: xs.len() > 0 (in `max_index`)")]
fn max_index_requires() {
    max_index(&[]);
}

#[test]
fn push_all_passes() {
    let mut v = vec![1, 2];
    push_all(&mut v, &[3, 4]);
    assert_eq!(v, vec![1, 2, 3, 4]);
}

#[test]
#[should_panic(expected = "ensures violated: dst.len() == old(dst.len()) + src.len() (in `push_all_buggy`)")]
fn push_all_buggy_ensures() {
    push_all_buggy(&mut vec![], &[1, 2]);
}

#[test]
#[should_panic(expected = "ensures violated: result > x (in `bump_buggy`)")]
fn early_return_checked() {
    bump_buggy(11);
}

#[test]
#[should_panic(expected = "requires violated: x > 0")]
fn bump_requires() {
    bump_buggy(0);
}

#[test]
fn implication_exists_refs_methods() {
    assert!(contains(&[1, 2, 3], 2));
    assert!(!contains(&[], 2));
    assert_eq!(*first(&[7, 8]), 7);
    let mut c = Counter { n: 0 };
    assert_eq!(c.incr(), Ok(1));
}

#[test]
#[should_panic(expected = "invariant violated: false (in `empty_range_false`)")]
fn empty_range_entry_check() {
    empty_range_false();
}

#[test]
#[should_panic(expected = "invariant violated: n == 0 (in `while_false_at_exit`)")]
fn while_exit_check() {
    while_false_at_exit();
}

#[test]
fn next_index_rule_holds() {
    assert_eq!(next_index(false), 3);
}

#[test]
#[should_panic(expected = "invariant violated: strict ==> i < 3")]
fn next_index_rule_exit_value() {
    next_index(true);
}

#[test]
fn continue_break_while_let() {
    assert_eq!(skips_and_breaks(), 3); // i = 1, 3, 5; i = 7 breaks at j == 5
    assert_eq!(while_let(vec![1, 2, 3]), 6);
}

// Release-mode erasure: run without debug assertions, violated clauses do not fire.
fn main() {
    empty_range_false();
    while_false_at_exit();
    bump_buggy(0);
    push_all_buggy(&mut vec![], &[1, 2]);
    println!("release: violated clauses erased, no panic");
}
