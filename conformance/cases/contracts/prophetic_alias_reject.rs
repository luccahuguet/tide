//@ rules: CLAUSE-4
// The existing runtime alias limitation still gives E0277; admission must also
// name the shared-origin prophecy rule for uses moved into entry snapshots.
type M<'a> = &'a mut u32;
#[ensures(*x == *y)] //~ E0277 TD0803
pub fn shared<'a>(x: M<'a>, y: &'a u32) { *x = *y; }

#[ensures(old(*x) <= *x && *x == *y)] //~ E0277 TD0803
pub fn mixed<'a>(x: M<'a>, y: &'a u32) { *x = *y; }
