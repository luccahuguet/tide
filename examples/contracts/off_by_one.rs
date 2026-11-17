//@ expect: test fails insert_sorted
//@ reports: cause: ensures violated: xs.is_sorted() (in `insert_sorted`
//@ reports: input: xs = [0], x = 1
//@ reports: fresh rerun: TIDE_TEST_SEED=
// sorted_insert.rs with an off-by-one loop bound: an `x` larger than every element goes in
// before the last one. Nobody wrote a test for it; the test build generates one from the
// contract.
#[requires(xs.is_sorted())]
#[ensures(xs.is_sorted())]
#[ensures(xs.len() == old(xs.len()) + 1)]
#[ensures(xs.contains(&x))]
pub fn insert_sorted(xs: &mut Vec<i64>, x: i64) {
    let mut i = 0;
    while i + 1 < xs.len() && xs[i] < x {
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
