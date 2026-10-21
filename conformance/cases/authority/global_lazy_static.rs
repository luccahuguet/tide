//@ rules: GLOBAL-2
// `lazy_static!` declares a lazy global (`FX0502`); Tide's std has no such `!` form.
lazy_static! { //~ error FX0502
    static ref TABLE: Vec<u32> = vec![1, 2, 3];
}
