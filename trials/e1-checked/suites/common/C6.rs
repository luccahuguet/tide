#[test]
fn case1() {
    assert_eq!(
        find_any(
            &["nothing".into(), "prefix needle tail".into(), "else".into()],
            "needle"
        ),
        Some((1, 7))
    );
}
#[test]
fn case2() {
    assert_eq!(find_any(&["abc".into(), "def".into()], "x"), None);
    assert_eq!(find_any(&[], "x"), None);
}
#[test]
fn case3() {
    assert!(matches!(
        find_any(&["a xx".into(), "xx b".into()], "xx"),
        Some((0, 2)) | Some((1, 0))
    ));
}
#[test]
fn case4() {
    let mut hays = vec!["a".repeat(1000000); 8];
    hays[3].replace_range(999000..999006, "needle");
    assert_eq!(find_any(&hays, "needle"), Some((3, 999000)));
}
#[test]
#[should_panic]
fn case5() {
    find_any(&[], "");
}
