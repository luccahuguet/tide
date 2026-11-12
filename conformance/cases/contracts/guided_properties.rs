//@ rules: CLAUSE-1
// Production macros/runtime in the stub; build.rs executes every entry below.
#[cfg(test)]
mod properties {
    use std::sim::SimRoot;

    fn opaque(n: u16) -> bool {
        n == 54321
    }
    fn never(_n: u8) -> bool {
        false
    }
    fn even(n: u16) -> bool {
        n % 2 == 0
    }

    #[requires(i < xs.len())]
    #[test]
    fn index(i: usize, xs: Vec<u16>) {
        assert!(xs.get(i).is_some());
    }

    #[test]
    #[requires(i < xs.len())]
    fn borrowed_index(i: usize, xs: &[u16]) {
        assert!(xs.get(i).is_some());
    }

    #[test]
    #[requires(lo <= hi)]
    fn borrowed(lo: &u16, hi: &u16) {
        assert!(lo <= hi);
    }

    #[test]
    #[requires(xs.len() == 3 && xs.contains(&x) && x == c && x == &54321 && o.is_some())]
    fn borrowed_shapes(xs: &mut [u16], x: &u16, c: &u16, o: &Option<u8>) {
        assert!(xs.len() == 3 && xs.contains(x) && x == c && o.is_some());
    }

    #[test]
    #[requires(flag == other && c == letter && flag == &true && c == &'z')]
    fn borrowed_scalars(flag: &bool, other: &bool, c: &char, letter: &char) {
        assert!(flag == other && c == letter);
    }

    #[test]
    #[requires(xs.contains(&x))]
    fn borrowed_string(xs: &mut [String], x: &String) {
        assert!(xs.contains(x));
    }

    const LIMIT: &u16 = &32768;
    #[test]
    #[requires(n <= LIMIT)]
    fn borrowed_constant(n: &u16) {
        assert!(n <= LIMIT);
    }

    #[test]
    #[requires(n >= 9 && n <= 12 && even(n))]
    fn filter_always(n: u16) {
        assert!(n == 10 || n == 12);
    }

    #[test]
    #[requires(hi >= lo && lo >= -200 && lo <= -150 && hi < -100)]
    fn endpoints(hi: i16, lo: i16) {
        assert!(lo <= hi && hi < -100);
    }

    #[test]
    #[requires(text.len() == 17 && xs.len() >= 12 && xs.len() < 15)]
    fn lengths(text: String, xs: &mut [u8]) {
        assert!(text.len() == 17 && xs.len() >= 12);
    }

    #[test]
    #[requires(xs.is_sorted() && xs.contains(&x) && o.is_some() && x == 57)]
    fn shapes(xs: Vec<u16>, x: u16, o: Option<u8>) {
        assert!(xs.is_sorted() && xs.contains(&57) && o.is_some());
    }

    #[test]
    #[requires(xs.len() == 1 && xs.contains(&x) && x == 54321)]
    fn contains_one(xs: Vec<u16>, x: u16) {
        assert_eq!(xs, vec![54321]);
    }

    #[test]
    #[requires(flag == true && c == 'z' && x > -3.0 && x < 2.0)]
    fn other_scalars(flag: bool, c: char, x: f64) {
        assert!(flag && c == 'z' && x > -3.0 && x < 2.0);
    }

    #[test]
    #[requires(n < 10 && opaque(n))]
    fn vacuous(n: u16) {
        let _ = n;
    }

    #[test]
    #[requires(n < 0 && n > 100)]
    fn contradictory(n: i16) {
        let _ = n;
    }

    #[test]
    #[requires(a < b && b < a)]
    fn cyclic(a: u8, b: u8) {
        let _ = (a, b);
    }

    #[requires(n > 0)]
    fn positive(n: u8) {
        assert!(n > 0);
    }
    fn deeper(n: u8) {
        positive(n);
    }

    #[test]
    fn direct(n: u8) {
        positive(n);
    }
    #[test]
    fn nested(n: u8) {
        deeper(n);
    }
    #[test]
    fn nested_local(n: u8) {
        fn local(n: u8) {
            positive(n);
        }
        local(n);
    }
    #[test]
    fn nested_closure(n: u8) {
        let call = || positive(n);
        call();
    }
    #[test]
    fn shadowed(shadowed: u8) {
        let _ = shadowed;
    }
    #[test]
    #[requires((n >= 12 && n <= 14))]
    fn grouped(n: u16) {
        assert!((12..=14).contains(&n));
    }
    #[test]
    #[should_panic(expected = "requires violated")]
    #[requires(never(n))]
    fn expected(n: u8) {
        let _ = n;
    }

    #[test]
    #[requires(xs.len() >= 3)]
    fn shrink_filtered(xs: Vec<u16>) {
        assert!(xs.len() < 3);
    }

    #[test]
    #[requires(i < xs.len())]
    fn shrink_index(i: usize, xs: Vec<u16>) {
        assert!(i < 3);
    }

    #[test]
    #[requires(lo <= hi)]
    fn shrink_span(lo: u16, hi: u16) {
        assert!(hi - lo < 5);
    }

    #[test]
    fn explicit_seed(seed: u64, sizes: Vec<u16>) {
        let sim = Root::sim(seed);
        sim.tasks.yield_now().unwrap();
        assert!(sizes.len() < 3);
    }

    #[sim_test(seeds = 4)]
    #[requires(sizes.len() >= 2)]
    fn simulated(sim: SimRoot, sizes: Vec<u16>) {
        sim.tasks.yield_now().unwrap();
        assert!(sizes.len() < 3);
    }

    #[requires(i < xs.len())]
    #[sim_test(seeds = 5)]
    fn simulated_pass(sim: SimRoot, i: usize, xs: Vec<u16>) {
        sim.tasks.yield_now().unwrap();
        assert!(xs.get(i).is_some());
    }

    #[sim_test(seeds = 2)]
    #[requires(seed == 54321)]
    fn simulated_guided_seed(sim: SimRoot, seed: u64) {
        let _ = sim;
        assert_eq!(seed, 54321);
    }

    #[sim_test(seeds = 2)]
    fn simulated_generated_seed(sim: SimRoot, seed: u64) {
        let _ = sim;
        assert!(seed < 2);
    }

    #[sim_test(seeds = 5)]
    #[requires(even(n))]
    fn simulated_filtered(sim: SimRoot, n: u16) {
        let _ = sim;
        assert!(even(n));
    }

    #[sim_test(seeds = 2)]
    #[requires(never(n))]
    fn simulated_vacuous(sim: SimRoot, n: u8) {
        let _ = (sim, n);
    }

    #[derive(Debug)]
    struct Local {
        field: u8,
    }
    impl std::test::Arbitrary for Local {
        fn arbitrary(g: &mut std::test::Gen) -> Self {
            Self {
                field: <u8 as std::test::Arbitrary>::arbitrary(g),
            }
        }
    }
    #[sim_test]
    #[cfg(any())]
    fn disabled(sim: SimRoot, local: Missing) {
        let _ = (sim, local);
    }

    #[sim_test(seeds = 2)]
    fn simulated_struct(sim: SimRoot, local: Local) {
        let _ = (sim, local.field);
    }
}
