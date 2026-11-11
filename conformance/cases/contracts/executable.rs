//@ rules: CLAUSE-1, CLAUSE-2, CLAUSE-4, CLAUSE-6, FORM-4
// Shared compile-time surface; tests/build.rs executes this with production std.
#[requires(n > 0)]
#[ensures(result == n + 1)]
fn increment(mut n: u32) -> u32 { n += 1; n }

#[ensures(result == v.len() + 1)]
fn grow(mut v: Vec<u32>) -> usize { v.push(7); v.len() }

struct Token(u32);
#[ensures(result == t.0)]
fn read(t: Token) -> u32 { t.0 }

#[ensures(v.len() == old(v.len()) + src.len())]
fn append(v: &mut Vec<u32>, src: &[u32]) { v.extend_from_slice(src); }

#[derive(Debug)]
struct Node { value: u32, next: Option<Box<Node>> }
#[ensures(n.value == old(n.value))]
fn walk(mut n: &mut Node) {
    while let Some(ref mut next) = n.next { n = next; }
    n.value += 1;
}
impl Node {
    #[ensures(self.value == old(self.value) + 1)]
    fn bump(&mut self) {
        self.value += 1;
        let caption = "node";
        assert!(format!("{self:?} {caption}").len() > 0);
        assert_eq!(format!("{{self}}"), "{self}", "{self:?}");
        struct Local;
        impl Local { fn value(&self) -> u32 { 3 } }
        assert_eq!(Local.value(), 3);
    }
    #[ensures(*result == old(self.value))]
    fn value(&mut self) -> &u32 { &self.value }
}

#[ensures(result.len() == 2)]
fn owned() -> Vec<u32> { vec![3, 4] }
#[ensures(result.len() == 1)]
fn opaque() -> impl ExactSizeIterator<Item = u32> { [3].into_iter() }

#[ensures(result.is_some())]
fn exits(mode: u8) -> Option<Vec<u32>> {
    if mode == 0 { return None; }
    if mode == 1 { let _: u32 = None?; }
    Some(vec![None?])
}

#[requires(n > 0)]
#[ensures(old(v[0]) == result)]
fn guarded(n: u32, v: &[u32]) -> u32 { n }

#[ensures(result == 0)]
fn wrong() -> u32 { 1 }
#[ensures(v[0] == 3)]
fn panicking(v: &[u32]) {}
#[ensures(old(v[0]) == 3)]
fn bad_snapshot(v: &[u32]) {}

#[ensures(forall |i: usize| i < xs.len() ==> xs[i] <= result)]
fn maximum(xs: &[u32]) -> u32 { xs.iter().copied().max().unwrap_or(0) }
#[ensures(exists |i: i32| -3 <= i && i < 0 && i == result)]
fn negative() -> i32 { -2 }
#[ensures(forall |x: u32| xs.contains(&x) ==> x < 20)]
#[ensures(exists |x: u32| xs.contains(&x) && x == 2)]
fn collection(xs: &[u32]) {}
#[ensures(result == (exists |i: usize| i < 0 && i == 1))]
#[ensures(forall |i: usize| i < 0 ==> false)]
fn empty() -> bool { false }
#[ensures(forall |i: usize| i < 2 ==> false)]
fn sampled_false() {}
#[ensures(forall |i: usize| i < 100_000 ==> i < 100_001)]
fn over_budget() {}
#[ensures(false ==> true ==> false)]
fn implication() {}

#[ensures(match result { 0 => forall |i: usize| i < 0 ==> false, _ => true })]
fn match_quantifier() -> u8 { 0 }
#[ensures(match result { Some(n) => n > 0, None => true })]
fn shadow(n: u8) -> Option<u8> { Some(n + 1) }
#[ensures(if let Some(n) = result { n > 0 } else { true })]
fn if_shadow(n: u8) -> Option<u8> { Some(n + 1) }
#[ensures(result == vec![n])]
fn macro_entry(mut n: u8) -> Vec<u8> { let value = n; n += 1; vec![value] }
#[ensures(result == old(it.len()))]
fn argument_impl(it: &mut impl ExactSizeIterator<Item = u32>) -> usize { it.len() }
#[requires(a || b && c)]
fn precedence(a: bool, b: bool, c: bool) {}

#[invariants]
fn loops() {
    for i in 5..2 { invariant!(i == 5); }
    for c in 'a'..'d' { invariant!(c >= 'a' && c <= 'd'); }
    let mut n = 0;
    while n < 3 { invariant!(n <= 3); n += 1; }
    while let Some(x) = (n > 0).then_some(n) { invariant!(n <= 3); n = x - 1; }
    'outer: loop { invariant!(n == 0); break 'outer; }
    for _ in vec![1, 2].iter() { invariant!(n == 0); }
    let closure = || { for _ in 0..0 { invariant!(true); } };
    closure();
    let values = std::cell::RefCell::new(vec![1]);
    while let Some(_) = values.borrow_mut().pop() {
        invariant!(true);
        assert!(values.try_borrow_mut().is_err());
    }
    values.borrow_mut().push(1);
    for _ in values.borrow().iter() {
        invariant!(true);
        assert!(values.try_borrow_mut().is_err());
    }
    let mut head = 0;
    loop { invariant!(head == 0); head += 1; break; }
    for i in 0..70 {
        invariant!(forall |j: usize| j < 1 ==> i != 65);
    }
}
#[invariants]
fn middle() { for i in 0..4 { invariant!(i != 2); } }
#[invariants]
fn zero() { for _ in 0..0 { invariant!(false); } }
#[invariants]
fn while_exit() { let mut i = 0; while i < 1 { invariant!(i == 0); i += 1; } }
#[invariants]
fn loop_entry() { loop { invariant!(false); break; } }
#[invariants]
fn inclusive() { for _ in 0..=0 { invariant!(false); } }

fn good() {
    assert_eq!(increment(2), 3);
    assert_eq!(grow(vec![1]), 2);
    assert_eq!(read(Token(7)), 7);
    let mut xs = vec![1]; append(&mut xs, &[2, 3]); assert_eq!(xs, vec![1, 2, 3]);
    let mut node = Node { value: 3, next: Some(Box::new(Node { value: 4, next: None })) };
    walk(&mut node); assert_eq!(node.next.as_ref().unwrap().value, 5);
    node.bump(); assert_eq!(*node.value(), 4);
    assert_eq!(owned(), vec![3, 4]); assert_eq!(opaque().len(), 1);
    assert_eq!(maximum(&[1, 9, 2]), 9); assert_eq!(negative(), -2);
    collection(&[1, 2]); assert!(!empty()); implication(); loops();
    assert_eq!(match_quantifier(), 0); assert_eq!(shadow(0), Some(1));
    assert_eq!(if_shadow(0), Some(1));
    assert_eq!(macro_entry(3), vec![3]);
    assert_eq!(argument_impl(&mut [1, 2].into_iter()), 2);
    precedence(true, false, false);
    #[cfg(all(debug_assertions, test))]
    {
        over_budget();
        assert!(std::__contract::take_unchecked().iter().any(|c| c.reason == "step budget exceeded"));
    }
}

#[tide::main]
fn main() {
    good();
    #[cfg(not(test))]
    { middle(); sampled_false(); }
    #[cfg(not(debug_assertions))]
    {
        increment(0); wrong(); panicking(&[]); bad_snapshot(&[]);
        guarded(0, &[]); exits(0); zero(); while_exit(); loop_entry(); inclusive();
        assert!(std::__contract::take_unchecked().is_empty());
    }
    println!("CONTRACTS_RAN");
}

#[test] fn clauses_execute() { good(); }
#[test] #[cfg_attr(debug_assertions, should_panic(expected = "requires violated: n > 0"))]
fn requires_before_snapshot() { guarded(0, &[]); }
#[test] #[cfg_attr(debug_assertions, should_panic(expected = "ensures violated: result == 0"))]
fn postcondition() { wrong(); }
#[test] #[cfg_attr(debug_assertions, should_panic(expected = "ensures violated: v[0] == 3"))]
fn clause_panic() { panicking(&[]); }
#[test] #[cfg_attr(debug_assertions, should_panic(expected = "ensures violated: old(v[0]) == 3"))]
fn snapshot_panic() { bad_snapshot(&[]); }
#[test] #[cfg_attr(debug_assertions, should_panic(expected = "ensures violated"))]
fn early_return() { exits(0); }
#[test] #[cfg_attr(debug_assertions, should_panic(expected = "ensures violated"))]
fn question_mark() { exits(1); }
#[test] #[cfg_attr(debug_assertions, should_panic(expected = "ensures violated"))]
fn question_mark_in_macro() { exits(2); }
#[test] #[cfg_attr(debug_assertions, should_panic(expected = "ensures violated: forall"))]
fn sampled_executes() { sampled_false(); }
#[test] #[cfg_attr(debug_assertions, should_panic(expected = "invariant violated: i != 2"))]
fn middle_head() { middle(); }
#[test] #[cfg_attr(debug_assertions, should_panic(expected = "invariant violated: false"))]
fn empty_loop_entry() { zero(); }
#[test] #[cfg_attr(debug_assertions, should_panic(expected = "invariant violated: i == 0"))]
fn exit_head() { while_exit(); }
#[test] #[cfg_attr(debug_assertions, should_panic(expected = "invariant violated: false"))]
fn unconditional_loop_head() { loop_entry(); }
#[test] #[cfg_attr(debug_assertions, should_panic(expected = "invariant violated: false"))]
fn inclusive_loop_head() { inclusive(); }
#[requires(n <= 255)]
#[test]
fn contract_above_test(n: u8) { assert!(n <= 255); }
#[requires(true)]
#[sim_test(seeds = 1)]
fn contract_above_sim_test(sim: std::sim::SimRoot) {
    assert_eq!(sim.elapsed(), std::time::Duration::ZERO);
}
