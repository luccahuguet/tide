//@ rules: PAR-5, PAR-7
// `join` runs two `FnOnce + Pure + Send` closures: one moves in and consumes
// an owned `String`, both borrow shared `Pure + Sync` data, and each holds
// one half of a disjoint `&mut` split.
use std::par::join;

pub fn halves(xs: &mut [u64], scale: &u64, label: String) -> (usize, u64) {
    let (left, right) = xs.split_at_mut(xs.len() / 2);
    let (n, sum) = join(
        move || {
            left[0] *= *scale;
            label.into_bytes().len()
        },
        || {
            right[0] *= *scale;
            right.iter().sum::<u64>()
        },
    );
    (n, sum)
}
