//@ rules: GLOBAL-2
// `lazy_static!` declares a lazy global (`TD0502`); Tide's std has no such `!` form.
lazy_static! { //~ error TD0502
    static ref TABLE: Vec<u32> = vec![1, 2, 3];
}
