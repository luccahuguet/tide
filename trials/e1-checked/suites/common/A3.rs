#[test]
fn case1() {
    for (s, n) in [
        ("1 + 2 * 3", 7),
        ("(1+2)*3", 9),
        ("-(-4) % 3", 1),
        ("7 / -2", -3),
        ("- 9223372036854775807 - 1", i64::MIN),
    ] {
        assert_eq!(eval(s), Ok(n));
    }
}
#[test]
fn case2() {
    assert_eq!(eval("1 / (2 - 2)"), Err(CalcError::DivByZero { pos: 2 }));
}
#[test]
fn case3() {
    for (s, pos) in [
        ("9223372036854775807 + 1", 20),
        ("9223372036854775808", 0),
        ("-9223372036854775808", 1),
        ("-(-9223372036854775807 - 1)", 0),
    ] {
        assert_eq!(eval(s), Err(CalcError::Overflow { pos }));
    }
}
#[test]
fn case4() {
    for (s, pos) in [("2 *", 3), ("(1", 2), ("1 2", 2), ("", 0), ("a", 0)] {
        assert_eq!(eval(s), Err(CalcError::Unexpected { pos }));
    }
}
#[test]
fn case5() {
    assert_eq!(
        CalcError::DivByZero { pos: 2 }.to_string(),
        "division by zero at 2"
    );
}
