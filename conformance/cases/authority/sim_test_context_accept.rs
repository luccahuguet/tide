//@ rules: ROOT-1, ROOT-3
// Plain/property/simulation tests and test-only helper modules may use simulation.
#[cfg(all(test, not(any())))]
mod tests {
    use std::sim::SimRoot;
    pub fn setup() -> SimRoot { Root::sim(1) }
    #[test]
    fn plain() {
        fn nested() { let _ = Root::sim(2); }
        nested();
        let _ = setup();
    }
}
#[test]
fn property(seed: u64) { let _ = Root::sim(seed); }
#[sim_test(seeds = 1)]
fn simulated(sim: std::sim::SimRoot) { let _ = sim; }

pub fn nested_items() {
    #[cfg(test)]
    use std::sim::SimRoot;
    #[cfg(test)]
    struct State { sim: SimRoot }
}

#[cfg(any(test, not(unix)))]
fn target_test_helper() { let _ = Root::sim(3); }
