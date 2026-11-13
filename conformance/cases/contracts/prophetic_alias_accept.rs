//@ rules: CLAUSE-4
type M<'a> = &'a mut u32;
// old alone does not need the return-time entry snapshot or a prophecy.
#[ensures(old(*x) <= *y)]
pub fn snapshot<'a>(x: M<'a>, y: &'a u32) { *x = *y; }
