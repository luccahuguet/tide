#[sim_test(seeds = 20)]
fn case1(sim: std::sim::SimRoot) {
    assert_eq!(
        merge_streams(
            &sim.tasks,
            &[vec![1, 4, 9], vec![2, 3, 10], vec![], vec![5]],
            1
        ),
        Ok(vec![1, 2, 3, 4, 5, 9, 10])
    );
    assert_eq!(merge_streams(&sim.tasks, &[], 1), Ok(vec![]));
    assert_eq!(
        merge_streams(&sim.tasks, &[vec![1, 1], vec![1]], 1),
        Ok(vec![1, 1, 1])
    );
}
#[sim_test(seeds = 20)]
fn case2(sim: std::sim::SimRoot) {
    let inputs: Vec<Vec<i64>> = (0..8)
        .map(|i| (0..10000).map(|j| j * 8 + i).collect())
        .collect();
    let mut want: Vec<_> = inputs.iter().flatten().copied().collect();
    want.sort();
    assert_eq!(merge_streams(&sim.tasks, &inputs, 2), Ok(want));
}
#[sim_test(seeds = 20)]
fn case3(sim: std::sim::SimRoot) {
    assert_eq!(
        merge_streams(&sim.tasks, &[vec![1, 2], vec![3, 1], vec![5, 0]], 1),
        Err(MergeError::Unsorted { input: 1, index: 1 })
    );
}
#[sim_test(seeds = 20)]
#[should_panic]
fn case4(sim: std::sim::SimRoot) {
    let _ = merge_streams(&sim.tasks, &[], 0);
}
