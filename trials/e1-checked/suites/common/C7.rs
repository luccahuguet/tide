#[test]
fn case1() {
    let mut s = BoundedStack::new(3);
    assert!(s.is_empty());
    assert_eq!(s.capacity(), 3);
    for n in [1, 2, 3] {
        s.push(n);
    }
    assert!(s.is_full());
    assert_eq!(s.len(), 3);
    for expected in [Some(3), Some(2), Some(1), None] {
        assert_eq!(s.pop(), expected);
    }
}
#[test]
#[should_panic]
fn case2_push() {
    let mut s = BoundedStack::new(1);
    s.push(1);
    s.push(2);
}
#[test]
#[should_panic]
fn case2_new() {
    BoundedStack::<i64>::new(0);
}
#[test]
#[should_panic]
fn case2_drain() {
    let mut s = BoundedStack::new(3);
    s.push(1);
    s.push(2);
    s.drain_top(3);
}
#[test]
fn case3() {
    let mut s = BoundedStack::new(3);
    for c in ['a', 'b', 'c'] {
        s.push(c);
    }
    assert_eq!(s.drain_top(2), vec!['c', 'b']);
    assert_eq!(s.peek(), Some(&'a'));
    assert_eq!(s.len(), 1);
}
#[test]
fn case4() {
    let mut s = BoundedStack::new(3);
    for c in ["a", "b", "c"] {
        s.push(c.to_string());
    }
    assert!(s.is_full());
    assert_eq!(s.drain_top(2), vec![String::from("c"), String::from("b")]);
    assert_eq!(s.peek(), Some(&String::from("a")));
    assert_eq!(s.pop(), Some(String::from("a")));
    assert_eq!(s.pop(), None);
}
