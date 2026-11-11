//@ rules: CLAUSE-1, CLAUSE-2, CLAUSE-4, CLAUSE-6, FORM-4
// Scoped entry values, loop heads, snapshots and const checks in both profiles.
#[invariants]
fn range_temporaries() {
    let values = std::cell::RefCell::new(vec![1]);
    for _ in 0..values.borrow().len() {
        invariant!(true);
        assert!(values.try_borrow_mut().is_err());
    }
    for i in (0..3) { invariant!(i <= 3); }
    for i in (0..0) { invariant!(i == 0); }
    for i in (5..2) { invariant!(i == 5); }
}
#[ensures(result == matches!(n, 0))]
fn macro_entry(mut n: u8) -> bool { let r = n == 0; n = 1; r }
fn positive(n: u32) -> bool { n > 0 }
#[ensures(matches!(positive(n), true))]
fn sampled_macro(n: u32) {}
#[invariants]
fn macro_loop() { assert!({ for _ in 0..0 { invariant!(false); } true }); }
struct Receiver(u32);
impl Receiver {
    #[ensures(self.0 == 4)]
    fn mutate(&mut self) {
        assert!({
            struct Local;
            impl Local { fn value(&self) -> u32 { 3 } }
            self.0 = Local.value() + 1;
            true
        });
    }
}
#[ensures(result == n)]
fn tuple_entry((mut n,): (u8,)) -> u8 { let r = n; n += 1; r }
struct Row { n: u8 }
#[ensures(match result { Row { n: x } => x == n })]
fn field_label(mut n: u8) -> Row { let value = n; n += 1; Row { n: value } }
#[ensures(matches!(result, Row { n: x } if x == n))]
fn macro_pattern(mut n: u8) -> Row { let value = n; n += 1; Row { n: value } }
#[ensures(result == n + *m + tail[0])]
fn patterns((mut n, ref m, tail @ [_, _]): (u8, u8, [u8; 2])) -> u8 {
    let value = n + *m + tail[0]; n += 1; value
}
fn both(a: bool, b: bool) -> bool { a && b }
#[ensures(both(true ==> result, true))]
fn implication_args() -> bool { true }
#[ensures({ let p = true ==> result; p })]
fn implication_block() -> bool { true }
#[ensures(matches!(true ==> result, true))]
fn implication_macro() -> bool { true }
#[ensures(result == { let n = 3; n })]
fn local_shadow(n: u32) -> u32 { 3 }
#[ensures(result == format!("{n}", n = n))]
fn named_format_entry(mut n: u8) -> String { let value = format!("{n}"); n += 1; value }
#[ensures(*x == 4)]
fn paren_mut(x: (&mut u32)) { *x = 4; }
#[derive(Clone)]
struct Token(u32);
#[ensures(old(t).0 == result)]
fn shared_old(t: &Token) -> u32 { t.0 }
#[ensures(t.0 == old(t).0 + 1)]
fn mutable_old(t: &mut Token) { t.0 += 1; }
#[invariants]
fn paren_middle() { for i in (0..3) { invariant!(i != 1); } }
#[requires(n > 0)]
#[ensures(result == n + 1)]
const fn const_increment(mut n: u32) -> u32 { n += 1; return n; }
const COMPILED: u32 = const_increment(2);
#[cfg(not(all(test, debug_assertions)))]
#[requires(positive(n))]
#[ensures(forall |i: usize| i < 1 ==> false)]
const fn const_sampled(n: u32) -> u32 { n }
#[cfg(not(all(test, debug_assertions)))]
const SKIPPED: u32 = const_sampled(0);

fn good() {
    range_temporaries(); assert!(macro_entry(0)); Receiver(0).mutate();
    assert_eq!(tuple_entry((0,)), 0); assert_eq!(field_label(0).n, 0);
    assert_eq!(macro_pattern(0).n, 0); assert_eq!(patterns((0, 1, [2, 3])), 3);
    assert!(implication_args()); assert!(implication_block()); assert!(implication_macro());
    assert_eq!(local_shadow(0), 3);
    assert_eq!(named_format_entry(0), "0");
    let mut n = 0; paren_mut(&mut n); assert_eq!(n, 4);
    assert_eq!(shared_old(&Token(3)), 3);
    let mut token = Token(3); mutable_old(&mut token); assert_eq!(token.0, 4);
    assert_eq!(COMPILED, 3); assert_eq!(const_increment(3), 4);
    #[cfg(not(all(test, debug_assertions)))]
    { assert_eq!(SKIPPED, 0); assert_eq!(const_sampled(0), 0); }
}
#[tide::main]
fn main() {
    good();
    #[cfg(not(test))]
    { sampled_macro(0); paren_middle(); }
    #[cfg(not(debug_assertions))]
    { macro_loop(); }
    println!("SCOPED_CLAUSES_RAN");
}
#[test] fn examples_execute() { good(); }
#[test] #[cfg_attr(debug_assertions, should_panic(expected = "ensures violated"))]
fn sampled_macro_executes_in_tests() { sampled_macro(0); }
#[test] #[cfg_attr(debug_assertions, should_panic(expected = "invariant violated"))]
fn loops_inside_macros_are_rewritten() { macro_loop(); }
#[test] #[cfg_attr(debug_assertions, should_panic(expected = "invariant violated"))]
fn parens_preserve_test_schedule() { paren_middle(); }
#[test] #[cfg_attr(debug_assertions, should_panic(expected = "requires violated"))]
fn const_requires_runs_at_runtime() { const_increment(0); }
