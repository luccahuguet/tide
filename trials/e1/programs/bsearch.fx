// src/search.fx of a library package
//
// Generic binary search over a sorted slice, proved under `prove` mode:
// total correctness, no overflow, no out-of-bounds indexing.
//
// The specs compare elements with `cmp` rather than `<` / `==`: in specs `==`
// is structural model equality, but a generic `Ord` may treat structurally
// different values as equal (for example case-insensitive keys), and the
// search can only promise what `cmp` says.

use std::cmp::Ordering;

/// `xs` is in non-decreasing order under `T`'s `Ord`.
spec fn sorted<T: Ord>(xs: &[T]) -> bool {
    forall |i: usize, j: usize| i < j && j < xs.len() ==> xs[i].cmp(&xs[j]) != Ordering::Greater
}

/// Like `slice::binary_search`: `Ok(i)` with `xs[i]` equal to `target` under
/// `cmp`, or `Err(i)` where `i` is the insertion point that keeps `xs` sorted.
#[contracts(prove)]
pub fn binary_search<T: Ord>(xs: &[T], target: &T) -> Result<usize, usize>
    requires sorted(xs)
    ensures match result {
        Ok(i) => i < xs.len() && xs[i].cmp(target) == Ordering::Equal,
        Err(i) => i <= xs.len()
            && (forall |j: usize| j < i ==> xs[j].cmp(target) == Ordering::Less)
            && (forall |j: usize| i <= j && j < xs.len() ==> xs[j].cmp(target) == Ordering::Greater),
    }
{
    let mut lo = 0;
    let mut hi = xs.len();
    while lo < hi
        invariant lo <= hi && hi <= xs.len()
        invariant forall |j: usize| j < lo ==> xs[j].cmp(target) == Ordering::Less
        invariant forall |j: usize| hi <= j && j < xs.len() ==> xs[j].cmp(target) == Ordering::Greater
        decreases hi - lo
    {
        let mid = lo + (hi - lo) / 2; // (lo + hi) / 2 could overflow
        match xs[mid].cmp(target) {
            Ordering::Less => {
                // sorted(xs) and Ord's transitivity law extend `Less` down to index 0.
                proof {
                    assert forall |j: usize| j <= mid ==> xs[j].cmp(target) == Ordering::Less;
                }
                lo = mid + 1;
            }
            Ordering::Greater => {
                proof {
                    assert forall |j: usize| mid <= j && j < xs.len() ==> xs[j].cmp(target) == Ordering::Greater;
                }
                hi = mid;
            }
            Ordering::Equal => return Ok(mid),
        }
    }
    Err(lo)
}

/// Callers see only `binary_search`'s contract, which is enough to prove this one.
#[contracts(prove)]
pub fn contains<T: Ord>(xs: &[T], target: &T) -> bool
    requires sorted(xs)
    ensures result == (exists |i: usize| i < xs.len() && xs[i].cmp(target) == Ordering::Equal)
{
    binary_search(xs, target).is_ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn agrees_with_linear_scan(mut xs: Vec<i32>, x: i32) {
        xs.sort();
        match binary_search(&xs, &x) {
            Ok(i) => assert_eq!(xs[i], x),
            Err(i) => {
                assert!(!xs.contains(&x));
                assert_eq!(i, xs.partition_point(|y| *y < x));
            }
        }
        assert_eq!(contains(&xs, &x), xs.contains(&x));
    }

    #[test]
    fn works_for_strings_and_edges() {
        let words = ["apple", "fig", "kiwi", "pear"];
        assert_eq!(binary_search(&words, &"kiwi"), Ok(2));
        assert_eq!(binary_search(&words, &"grape"), Err(2));
        assert_eq!(binary_search(&words, &"aardvark"), Err(0));
        assert_eq!(binary_search(&words, &"zucchini"), Err(4));
        let empty: [u8; 0] = [];
        assert_eq!(binary_search(&empty, &7), Err(0));
    }

    #[test]
    fn duplicates_return_some_matching_index() {
        let xs = [1, 2, 2, 2, 3];
        let i = binary_search(&xs, &2).unwrap();
        assert!((1..=3).contains(&i));
    }
}
