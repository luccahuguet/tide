#[test]
fn case1() {
    for (s, n) in [("12.34", 1234), ("-0.5", -50), ("7", 700)] {
        assert_eq!(parse_amount(s), Ok(Cents(n)));
    }
}
#[test]
fn case2() {
    for s in ["1.234", "", "-", ".5", "1.", "+1", "1,00"] {
        assert_eq!(parse_amount(s), Err(AmountError::Invalid));
    }
}
#[test]
fn case3() {
    assert_eq!(parse_amount("92233720368547758.07"), Ok(Cents(i64::MAX)));
    assert_eq!(parse_amount("-92233720368547758.08"), Ok(Cents(i64::MIN)));
    assert_eq!(
        parse_amount("92233720368547758.08"),
        Err(AmountError::Overflow)
    );
}
#[test]
fn case4() {
    assert_eq!(total(&[(Cents(250), 3), (Cents(-100), 2)]), Ok(Cents(550)));
    assert_eq!(total(&[(Cents(1), u32::MAX)]), Ok(Cents(4294967295)));
    assert_eq!(total(&[(Cents(i64::MAX), 2)]), Err(AmountError::Overflow));
}
#[test]
fn case5() {
    for (n, s) in [
        (0, "0.00"),
        (-1205, "-12.05"),
        (123450, "1234.50"),
        (i64::MIN, "-92233720368547758.08"),
    ] {
        assert_eq!(format_amount(Cents(n)), s);
    }
}
#[test]
fn case6() {
    for (n, p, want) in [
        (100, 3, vec![34, 33, 33]),
        (-5, 2, vec![-2, -3]),
        (0, 1, vec![0]),
    ] {
        assert_eq!(
            split_even(Cents(n), p),
            want.into_iter().map(Cents).collect::<Vec<_>>()
        );
    }
}
#[test]
#[should_panic]
fn case6_zero() {
    split_even(Cents(1), 0);
}
