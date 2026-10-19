//! Generic binary search with executable contracts.

/// True when `xs` is in non-decreasing order. Public, so public contracts may call it.
pub fn is_sorted<T: Ord>(xs: &[T]) -> bool {
    xs.windows(2).all(|w| w[0] <= w[1])
}

/// The first index whose element is not less than `target`, or `xs.len()`.
pub fn lower_bound<T: Ord>(xs: &[T], target: &T) -> usize
    requires is_sorted(xs)
    ensures result <= xs.len()
    ensures forall |j: usize| j < result ==> xs[j] < *target
    ensures forall |j: usize| result <= j && j < xs.len() ==> xs[j] >= *target
{
    let mut lo = 0;
    let mut hi = xs.len();
    while lo < hi
        invariant lo <= hi
        invariant hi <= xs.len()
        invariant forall |j: usize| j < lo ==> xs[j] < *target
        invariant forall |j: usize| hi <= j && j < xs.len() ==> xs[j] >= *target
    {
        let mid = lo + (hi - lo) / 2;                   // (lo + hi) / 2 could overflow
        if xs[mid] < *target {
            lo = mid + 1;
        } else {
            hi = mid;
        }
    }
    lo
}

/// The index of the first element equal to `target`, if there is one.
pub fn bsearch<T: Ord>(xs: &[T], target: &T) -> Option<usize>
    requires is_sorted(xs)
    ensures match result {
        Some(i) => i < xs.len() && xs[i] == *target && (i == 0 || xs[i - 1] < *target),
        None => !xs.contains(target),
    }
{
    let i = lower_bound(xs, target);
    if i < xs.len() && xs[i] == *target { Some(i) } else { None }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn small_cases() {
        let xs = [1, 3, 3, 3, 7, 9];
        assert_eq!(bsearch(&xs, &3), Some(1));          // the first of equal elements
        assert_eq!(bsearch(&xs, &9), Some(5));
        assert_eq!(bsearch(&xs, &4), None);
        assert_eq!(lower_bound(&xs, &4), 4);
        assert_eq!(lower_bound(&xs, &10), 6);
        let empty: [i32; 0] = [];
        assert_eq!(bsearch(&empty, &1), None);
        let words = ["apple", "fig", "kiwi"];
        assert_eq!(bsearch(&words, &"fig"), Some(1));
    }

    #[test]
    #[should_panic]
    fn unsorted_input_violates_the_precondition() {
        let _ = bsearch(&[3, 1, 2], &1);
    }

    #[test]
    fn agrees_with_a_linear_scan(mut xs: Vec<i32>, target: i32) {
        xs.sort();                                      // construct sorted input; filtering would be vacuous
        assert_eq!(bsearch(&xs, &target), xs.iter().position(|x| *x == target));
    }

    #[test]
    fn finds_every_element(mut xs: Vec<u16>, pick: usize)
        requires !xs.is_empty()
    {
        xs.sort();
        let target = xs[pick % xs.len()];
        let i = bsearch(&xs, &target).unwrap();
        assert_eq!(xs[i], target);
    }
}
