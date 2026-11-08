#[test]
fn case1() {
    assert_eq!(histogram(&[0, 1, 5, 9, 10, 99], &[0, 5, 10]), vec![2, 2]);
}
#[test]
fn case2() {
    let values = randoms(1000000);
    let mut edges = randoms(17);
    edges.sort();
    let want: Vec<u64> = edges
        .windows(2)
        .map(|e| u64::try_from(values.iter().filter(|&&v| e[0] <= v && v < e[1]).count()).unwrap())
        .collect();
    assert_eq!(histogram(&values, &edges), want);
}
#[test]
fn case3() {
    assert_eq!(histogram(&[], &[0, 5, 10]), vec![0, 0]);
}
#[test]
#[should_panic]
fn case3_equal_edges() {
    histogram(&[], &[5, 5]);
}
#[test]
#[should_panic]
fn case3_short_edges() {
    histogram(&[], &[5]);
}
