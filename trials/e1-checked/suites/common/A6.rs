#[test]
fn case1() {
    let mut c = Lru::new(2);
    c.put("a", 1);
    c.put("b", 2);
    assert_eq!(c.get(&"a"), Some(&1));
    assert_eq!(c.put("c", 3), Some(("b", 2)));
    assert_eq!(c.keys_recent_first(), vec![&"c", &"a"]);
}
#[test]
fn case2() {
    let mut c = Lru::new(2);
    c.put("a", 1);
    c.put("b", 2);
    assert_eq!(c.peek(&"a"), Some(&1));
    assert_eq!(c.put("c", 3), Some(("a", 1)));
}
#[test]
fn case3() {
    let mut c = Lru::new(2);
    c.put("a", 1);
    c.put("b", 2);
    assert_eq!(c.put("a", 9), None);
    assert_eq!(c.peek(&"a"), Some(&9));
    assert_eq!(c.keys_recent_first(), vec![&"a", &"b"]);
}
#[test]
#[should_panic]
fn case4() {
    Lru::<u32, u32>::new(0);
}
#[test]
fn case5() {
    let mut c = Lru::new(100000);
    for i in 0..300000u32 {
        c.put(i, i);
        let _ = c.get(&i);
    }
    assert_eq!(c.len(), 100000);
}
