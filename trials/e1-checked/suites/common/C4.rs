#[test]
fn case1() {
    let w = Warehouse::new(&[("a", 5), ("b", 2)]);
    w.reserve(&[("a", 2), ("b", 2)]).unwrap();
    assert_eq!((w.stock("a"), w.stock("b")), (Some(3), Some(0)));
    assert_eq!(
        w.reserve(&[("a", 1), ("b", 1)]),
        Err(ReserveError::Insufficient("b".into()))
    );
    assert_eq!(w.stock("a"), Some(3));
}
#[test]
fn case2() {
    let w = Warehouse::new(&[("a", 3)]);
    assert_eq!(
        w.reserve(&[("a", 2), ("a", 2)]),
        Err(ReserveError::Insufficient("a".into()))
    );
}
#[test]
fn case3() {
    let w = Warehouse::new(&[("a", 3)]);
    assert_eq!(
        w.reserve(&[("z", 1)]),
        Err(ReserveError::UnknownItem("z".into()))
    );
}
#[test]
fn case4() {
    let w = Warehouse::new(&[("x", 600), ("y", 1000)]);
    let results = run_orders(&w, &vec![vec![("x", 1), ("y", 1)]; 1000], 8);
    assert_eq!(results.len(), 1000);
    assert_eq!(results.iter().filter(|r| r.is_ok()).count(), 600);
    assert_eq!(
        results
            .iter()
            .filter(|r| **r == Err(ReserveError::Insufficient("x".into())))
            .count(),
        400
    );
    assert_eq!((w.stock("x"), w.stock("y")), (Some(0), Some(400)));
}
