//@ rules: PAR-6, PAR-7
// The slice methods take one `Fn + Pure + Send + Sync` closure, shared by
// every branch: it may own `Sync` data and borrow `Pure + Sync` data, and
// `par_for` mutates only the element each branch is given.
pub fn scaled(xs: &[u64], factor: &u64) -> Vec<u64> {
    let offsets = vec![1u64, 2, 3];
    xs.par_map(move |x| x * factor + offsets[0])
}

pub fn bump(xs: &mut [u64], step: u64) {
    xs.par_for(|x| *x += step);
}
