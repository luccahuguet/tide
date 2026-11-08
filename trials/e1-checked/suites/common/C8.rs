#[test]
fn case1() {
    let mut v = vec![3, 1, 4, 1, 5, 9, 2, 6];
    let mut old = v.clone();
    let n = partition(&mut v, 4);
    assert_eq!(n, 4);
    assert!(v[..n].iter().all(|x| *x < 4));
    assert!(v[n..].iter().all(|x| *x >= 4));
    v.sort();
    old.sort();
    assert_eq!(v, old);
}
#[test]
fn case2() {
    assert_eq!(partition(&mut [5, 6, 7], 4), 0);
    assert_eq!(partition(&mut [], 4), 0);
}
#[test]
fn case3() {
    let values: Vec<i32> = randoms(10000)
        .into_iter()
        .map(|x| i32::try_from(x % 1000000).unwrap())
        .collect();
    for input in [values, (0..10000).map(|x| x % 7).collect()] {
        let mut sorted = input.clone();
        sorted.sort();
        for k in [0, 1, 4999, 9999] {
            assert_eq!(select_kth(&mut input.clone(), k), sorted[k]);
        }
    }
}
#[test]
#[should_panic]
fn case4() {
    select_kth(&mut [1, 2], 2);
}
