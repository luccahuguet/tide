//@ expect: runs
//@ stdout: [3, 7, 9, 14, 20]
//@ expect: tests pass
// `insert_sorted` states its contract once: given a sorted vector, it leaves the vector sorted
// with one more element, `x`. Debug and test builds check the clauses; the test build also
// generates inputs that meet the `requires` and checks the `ensures` on each. This file has no
// hand-written test.
#[requires(xs.is_sorted())]
#[ensures(xs.is_sorted())]
#[ensures(xs.len() == old(xs.len()) + 1)]
#[ensures(xs.contains(&x))]
pub fn insert_sorted(xs: &mut Vec<i64>, x: i64) {
    let mut i = 0;
    while i < xs.len() && xs[i] < x {
        i += 1;
    }
    xs.insert(i, x);
}

#[tide::main]
fn main() {
    let mut xs = vec![3, 9, 14];
    insert_sorted(&mut xs, 7);
    insert_sorted(&mut xs, 20);
    println!("{xs:?}");
}
