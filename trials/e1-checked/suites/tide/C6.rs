#[sim_test(seeds = 20)]
fn case1(sim: std::sim::SimRoot) {
    assert_eq!(
        find_any(
            &sim.tasks,
            &["nothing".into(), "prefix needle tail".into(), "else".into()],
            "needle"
        ),
        Some((1, 7))
    );
}
#[sim_test(seeds = 20)]
fn case2(sim: std::sim::SimRoot) {
    assert_eq!(
        find_any(&sim.tasks, &["abc".into(), "def".into()], "x"),
        None
    );
    assert_eq!(find_any(&sim.tasks, &[], "x"), None);
}
#[sim_test(seeds = 20)]
fn case3(sim: std::sim::SimRoot) {
    assert!(matches!(
        find_any(&sim.tasks, &["a xx".into(), "xx b".into()], "xx"),
        Some((0, 2)) | Some((1, 0))
    ));
}
#[sim_test(seeds = 20)]
fn case4(sim: std::sim::SimRoot) {
    let mut hays = vec!["a".repeat(1000000); 8];
    hays[3].replace_range(999000..999006, "needle");
    assert_eq!(find_any(&sim.tasks, &hays, "needle"), Some((3, 999000)));
}
#[sim_test(seeds = 20)]
#[should_panic]
fn case5(sim: std::sim::SimRoot) {
    find_any(&sim.tasks, &[], "");
}
