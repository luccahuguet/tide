#[test]
fn case1() {
    assert!((Circle::new(1.0).unwrap().area() - std::f64::consts::PI).abs() < 1e-9);
    assert!((Triangle::new(3.0, 4.0, 5.0).unwrap().area() - 6.0).abs() < 1e-9);
}
#[test]
fn case2() {
    assert_eq!(
        describe(&Rect::new(2.0, 3.0).unwrap()),
        "rect: area 6.00, perimeter 10.00"
    );
}
#[test]
fn case3() {
    assert!(Triangle::new(1.0, 2.0, 3.0).is_none());
    assert!(Circle::new(f64::NAN).is_none());
    assert!(Rect::new(0.0, 1.0).is_none());
}
#[test]
fn case4() {
    assert!(largest(&[]).is_none());
    let s: Vec<Box<dyn Shape>> = vec![
        Box::new(Circle::new(1.0).unwrap()),
        Box::new(Rect::new(2.0, 3.0).unwrap()),
        Box::new(Rect::new(1.0, 6.0).unwrap()),
    ];
    assert!((largest(&s).unwrap().perimeter() - 10.0).abs() < 1e-9);
}
#[test]
fn case5() {
    let mut s: Vec<Box<dyn Shape>> = vec![
        Box::new(Rect::new(2.0, 3.0).unwrap()),
        Box::new(Circle::new(1.0).unwrap()),
        Box::new(Triangle::new(3.0, 4.0, 5.0).unwrap()),
    ];
    sort_by_area(&mut s);
    assert_eq!(
        s.iter().map(|s| s.name()).collect::<Vec<_>>(),
        vec!["circle", "rect", "triangle"]
    );
    assert!((total_area(&s) - (12.0 + std::f64::consts::PI)).abs() < 1e-9);
}
