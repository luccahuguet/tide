#[test]
fn case1() {
    let mut v: Vec<i64> = randoms(1000000).into_iter().map(i64::from).collect();
    let mut expected = v.clone();
    expected.sort();
    sort_parallel(&mut v);
    assert_eq!(v, expected);
}
#[test]
fn case2() {
    for mut v in [vec![], vec![3], vec![7; 10000]] {
        let mut expected = v.clone();
        expected.sort();
        sort_parallel(&mut v);
        assert_eq!(v, expected);
    }
}
#[test]
fn case3() {
    let mut v: Vec<_> = (0..100000).map(|i| (i % 7, i)).collect();
    let mut expected = v.clone();
    expected.sort_by_key(|p| p.0);
    sort_by_key_parallel(&mut v, |p| p.0);
    assert_eq!(v, expected);
}
