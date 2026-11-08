#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Cents(pub i64);
#[derive(Debug, PartialEq)]
pub enum AmountError {
    Invalid,
    Overflow,
}
pub fn parse_amount(s: &str) -> Result<Cents, AmountError> {
    let (negative, digits) = s.strip_prefix('-').map_or((false, s), |s| (true, s));
    let (whole, fraction) = digits
        .split_once('.')
        .map_or((digits, None), |(a, b)| (a, Some(b)));
    if whole.is_empty()
        || !whole.bytes().all(|b| b.is_ascii_digit())
        || fraction
            .is_some_and(|f| f.is_empty() || f.len() > 2 || !f.bytes().all(|b| b.is_ascii_digit()))
    {
        return Err(AmountError::Invalid);
    }
    let mut cents = 0i128;
    for b in whole
        .bytes()
        .chain(fraction.unwrap_or("").bytes())
        .chain(std::iter::repeat_n(b'0', 2 - fraction.map_or(0, str::len)))
    {
        cents = cents
            .checked_mul(10)
            .and_then(|n| n.checked_add(i128::from(b - b'0')))
            .ok_or(AmountError::Overflow)?;
    }
    i64::try_from(if negative { -cents } else { cents })
        .map(Cents)
        .map_err(|_| AmountError::Overflow)
}
pub fn total(items: &[(Cents, u32)]) -> Result<Cents, AmountError> {
    items
        .iter()
        .try_fold(0i64, |n, (price, qty)| {
            price
                .0
                .checked_mul(i64::from(*qty))
                .and_then(|x| n.checked_add(x))
                .ok_or(AmountError::Overflow)
        })
        .map(Cents)
}
pub fn format_amount(c: Cents) -> String {
    let n = i128::from(c.0).abs();
    format!(
        "{}{}.{:02}",
        if c.0 < 0 { "-" } else { "" },
        n / 100,
        n % 100
    )
}
pub fn split_even(c: Cents, parts: u32) -> Vec<Cents> {
    assert!(parts > 0);
    let p = i64::from(parts);
    let q = c.0.div_euclid(p);
    let rem = c.0.rem_euclid(p);
    (0..p).map(|i| Cents(q + i64::from(i < rem))).collect()
}
