//@ rules: GLOBAL-3
// No thread-locals in safe code (`FX0502`); Tide's std has no `thread_local!`.
thread_local! { //~ error FX0502
    static HITS: std::cell::Cell<u32> = std::cell::Cell::new(0);
}
