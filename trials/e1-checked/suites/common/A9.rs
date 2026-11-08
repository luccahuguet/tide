const INV: &[u8] = br#"{"warehouse":"w1","items":[{"sku":"a","quantity":3,"unitPriceCents":250},{"sku":"b","quantity":0,"unitPriceCents":100,"tags":["x"]}]}"#;
#[test]
fn case1() {
    let inv = load(INV).unwrap();
    assert!(inv.items[0].tags.is_empty());
    assert!(inv.items.iter().all(|i| i.discontinued.is_none()));
}
#[test]
fn case2() {
    let dup = br#"{"warehouse":"w1","items":[{"sku":"a","quantity":3,"unitPriceCents":250},{"sku":"a","quantity":0,"unitPriceCents":100}]}"#;
    assert!(matches!(load(dup),Err(LoadError::Duplicate(s)) if s=="a"));
    assert!(matches!(load(b"{bad"), Err(LoadError::Json(_))));
}
#[test]
fn case3() {
    let mut inv = load(INV).unwrap();
    apply(
        &mut inv,
        br#"[{"Sell":{"sku":"a","qty":2}},{"Restock":{"sku":"b","qty":5}}]"#,
    )
    .unwrap();
    assert_eq!((inv.items[0].quantity, inv.items[1].quantity), (1, 5));
    let mut inv = load(INV).unwrap();
    let old = inv.clone();
    assert!(
        matches!(apply(&mut inv,br#"[{"Restock":{"sku":"b","qty":1}},{"Sell":{"sku":"a","qty":9}}]"#),Err(ApplyError::Insufficient(s)) if s=="a")
    );
    assert_eq!(inv, old);
    let mut inv = load(INV).unwrap();
    assert!(
        matches!(apply(&mut inv,br#"[{"Sell":{"sku":"z","qty":1}}]"#),Err(ApplyError::UnknownSku(s)) if s=="z")
    );
    let mut inv = load(INV).unwrap();
    let old = inv.clone();
    assert!(
        matches!(apply(&mut inv,br#"[{"Restock":{"sku":"b","qty":1}},{"Restock":{"sku":"b","qty":4294967295}}]"#),Err(ApplyError::Overflow(s)) if s=="b")
    );
    assert_eq!(inv, old);
}
#[test]
fn case4() {
    let mut inv = load(INV).unwrap();
    apply(&mut inv, br#"[{"Discontinue":{"sku":"a"}}]"#).unwrap();
    assert_eq!(inv.items[0].discontinued, Some(true));
}
#[test]
fn case5() {
    let inv = load(INV).unwrap();
    let saved = save(&inv);
    assert_eq!(load(&saved).unwrap(), inv);
    assert!(
        String::from_utf8(saved)
            .unwrap()
            .contains("\"unitPriceCents\"")
    );
}
