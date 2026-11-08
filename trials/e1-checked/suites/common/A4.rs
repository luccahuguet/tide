#[test]
fn case1() {
    assert_eq!(Poly::new(vec![5, -1, 3]).to_string(), "3x^2 - x + 5");
    assert_eq!(Poly::new(vec![-2, 0, 0, -1]).to_string(), "-x^3 - 2");
}
#[test]
fn case2() {
    let p = Poly::new(vec![0, 0]);
    assert_eq!(p.to_string(), "0");
    assert_eq!(p.degree(), None);
}
#[test]
fn case3() {
    let x = Poly::new(vec![0, 1]);
    let one = Poly::new(vec![1]);
    assert_eq!(
        ((x.clone() + one.clone()) * (x - one)).coeffs(),
        &[-1, 0, 1]
    );
    let a = Poly::new(vec![1, 1]);
    let b = Poly::new(vec![-1, 1]);
    assert_eq!((&a + &b).coeffs(), &[0, 2]);
    assert_eq!(a.coeffs(), &[1, 1]);
    assert_eq!(b.coeffs(), &[-1, 1]);
}
#[test]
fn case4() {
    let a = Poly::new(vec![7, 2, 1]);
    assert_eq!((a.clone() - a).degree(), None);
    assert_eq!((-Poly::new(vec![0, 1])).to_string(), "-x");
}
#[test]
fn case5() {
    assert_eq!(Poly::new(vec![-1, 0, 1]).eval(3), Some(8));
    assert_eq!(Poly::new(vec![0, 0, 0, 1]).eval(3000000), None);
}
