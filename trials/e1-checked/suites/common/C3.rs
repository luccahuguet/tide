#[test]
fn case1() {
    assert_eq!(
        merge_streams(&[vec![1, 4, 9], vec![2, 3, 10], vec![], vec![5]], 1),
        Ok(vec![1, 2, 3, 4, 5, 9, 10])
    );
    assert_eq!(merge_streams(&[], 1), Ok(vec![]));
    assert_eq!(merge_streams(&[vec![1, 1], vec![1]], 1), Ok(vec![1, 1, 1]));
}
#[test]
fn case2() {
    let inputs: Vec<Vec<i64>> = (0..8)
        .map(|i| (0..600).map(|j| j * 8 + i).collect())
        .collect();
    let mut want: Vec<_> = inputs.iter().flatten().copied().collect();
    want.sort();
    assert_eq!(merge_streams(&inputs, 2), Ok(want));
}
#[test]
fn case3() {
    assert_eq!(
        merge_streams(&[vec![1, 2], vec![3, 1], vec![5, 0]], 1),
        Err(MergeError::Unsorted { input: 1, index: 1 })
    );
}
#[test]
#[should_panic]
fn case4() {
    let _ = merge_streams(&[], 0);
}
