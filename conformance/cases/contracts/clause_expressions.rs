//@ rules: CLAUSE-2, CLAUSE-4
// Clause expressions preserve entry values, lexical scopes and enclosing generics.
#[ensures(result == format!("{n}"))]
fn implicit_entry(mut n: u8) -> String { let r = format!("{n}"); n += 1; r }
#[ensures(matches!({ let mut s = 0; for n in 0..2 { s += n; } s }, 1))]
fn for_scope(n: u8) {}
#[ensures(matches!({ let mut it = Some(1); let mut s = 0; while let Some(n) = it { s += n; it = None; } s }, 1))]
fn while_scope(n: u8) {}
#[ensures(format!("{}", p ==> result) == "true")]
fn format_implication(p: bool) -> bool { false }
struct S<T>(T);
impl<T: Copy> S<T> {
    #[ensures(result == 1)]
    const fn value() -> usize { let _ = core::mem::size_of::<T>(); 1 }
    #[ensures(result == 2)]
    const fn early() -> usize {
        assert!({ return 2; true });
        3
    }
    #[ensures(result == 1)]
    const fn wrong() -> usize { assert!({ return 2; true }); 3 }
}
const COMPILED: usize = S::<u8>::value();
const EARLY: usize = S::<u8>::early();
impl S<u8> {
    #[ensures(result.0 == n)]
    const fn make(n: u8) -> Self { Self(n) }
}
const MADE: S<u8> = S::<u8>::make(3);

#[derive(Debug, PartialEq)]
struct W { n: u32 }
#[ensures(result == W { n })]
fn shorthand(mut n: u32) -> W { n += 1; W { n: n - 1 } }
#[requires({ n < 10 })]
#[ensures(result.n == (W { n }).n)]
const fn const_shorthand(mut n: u32) -> W { n += 1; W { n: n - 1 } }
const SHORTHAND: W = const_shorthand(0);
#[ensures(if !(a ==> b) { !result } else { result })]
#[ensures(match !(a ==> b) { true => !result, false => result })]
#[ensures({ while !(a ==> b) { break; } result == (!a || b) })]
#[ensures((|| { return !(a ==> b); })() == !result)]
fn keyword_negation(a: bool, b: bool) -> bool { !a || b }
#[derive(Debug)]
struct Receiver(u32);
impl Receiver {
    #[ensures(self.0 == 1)]
    fn unreachable_capture(&mut self) {
        if self.0 == 99 { unreachable!("{self:?}"); }
        if self.0 == 98 { todo!("{self:?}"); }
        if self.0 == 97 { unimplemented!("{self:?}"); }
        self.0 = 1;
    }
}

fn good() {
    assert_eq!(shorthand(0), W { n: 0 });
    assert_eq!(SHORTHAND.n, 0); assert_eq!(const_shorthand(1).n, 1);
    assert!(keyword_negation(false, false)); assert!(!keyword_negation(true, false));
    Receiver(0).unreachable_capture();
    assert_eq!(implicit_entry(0), "0"); for_scope(42); while_scope(42);
    assert!(!format_implication(false)); assert_eq!(COMPILED, 1);
    assert_eq!(S::<u16>::value(), 1); assert_eq!(EARLY, 2);
    assert_eq!(MADE.0, 3); assert_eq!(S::<u8>::make(4).0, 4);
}
#[tide::main]
fn main() { good(); println!("CLAUSE_EXPRESSIONS_RAN"); }
#[test] fn regressions_execute() { good(); }
#[test] #[cfg_attr(debug_assertions, should_panic(expected = "ensures violated"))]
fn const_macro_return_is_checked() { S::<u8>::wrong(); }

#[test] #[should_panic(expected = "Receiver(99)")]
fn receiver_panic_capture() { Receiver(99).unreachable_capture(); }
