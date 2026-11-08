use std::collections::BTreeMap;
fn replay(moves: &[(&str, &str)]) {
    let mut state: BTreeMap<String, String> = moves
        .iter()
        .map(|(a, _)| (a.to_string(), a.to_string()))
        .collect();
    let expected: BTreeMap<String, String> = moves
        .iter()
        .map(|(a, b)| (b.to_string(), a.to_string()))
        .collect();
    for (a, b) in plan(moves).unwrap() {
        assert!(!state.contains_key(&b), "target {b} already exists");
        let token = state.remove(&a).expect("source missing");
        state.insert(b, token);
    }
    assert_eq!(state, expected);
}
#[test]
fn case1() {
    for moves in [
        vec![("a", "b"), ("b", "a")],
        vec![("a", "b"), ("b", "c")],
        vec![("a", "b"), ("b", "c"), ("c", "a")],
    ] {
        replay(&moves);
    }
}
#[test]
fn case2() {
    assert_eq!(plan(&[("a", "a")]), Ok(vec![]));
}
#[test]
fn case3() {
    assert_eq!(
        plan(&[("a", "b"), ("a", "c")]),
        Err(PlanError::DuplicateSource("a".into()))
    );
    assert_eq!(
        plan(&[("a", "c"), ("b", "c")]),
        Err(PlanError::DuplicateTarget("c".into()))
    );
}
#[test]
fn case4() {
    with_fs(|fs| {
        fs_write(fs, "a", "1");
        fs_write(fs, "b", "2");
        apply(fs, &plan(&[("a", "b"), ("b", "a")]).unwrap()).unwrap();
        assert_eq!(fs_read(fs, "a"), "2");
        assert_eq!(fs_read(fs, "b"), "1");
    });
}
#[test]
fn case5() {
    with_fs(|fs| {
        assert_eq!(
            apply(fs, &[("x".into(), "y".into())]).unwrap_err().kind(),
            std::io::ErrorKind::NotFound
        );
        assert!(!fs_exists(fs, "x"));
        assert!(!fs_exists(fs, "y"));
    });
}
