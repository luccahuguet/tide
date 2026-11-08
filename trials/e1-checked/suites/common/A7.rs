fn populated() -> Trie {
    let mut t = Trie::new();
    for s in ["car", "card", "care", "cat", "dog"] {
        t.insert(s);
    }
    t
}
#[test]
fn case1() {
    let t = populated();
    assert_eq!(t.with_prefix("car"), vec!["car", "card", "care"]);
    assert_eq!(t.with_prefix(""), vec!["car", "card", "care", "cat", "dog"]);
}
#[test]
fn case2() {
    let mut t = populated();
    assert!(t.remove("car"));
    assert!(!t.contains("car"));
    assert!(t.contains("card"));
    assert_eq!(t.with_prefix("ca"), vec!["card", "care", "cat"]);
}
#[test]
fn case3() {
    let mut t = populated();
    assert!(!t.remove("absent"));
    assert!(!t.insert("card"));
    assert_eq!(t.len(), 5);
}
#[test]
fn case4() {
    let mut t = Trie::new();
    assert_eq!(t.longest_common_prefix(), "");
    for s in ["interview", "internet", "interval"] {
        t.insert(s);
    }
    assert_eq!(t.longest_common_prefix(), "inter");
    t.insert("");
    assert_eq!(t.longest_common_prefix(), "");
}
#[test]
fn case5() {
    let mut t = Trie::new();
    t.insert("über");
    t.insert("üben");
    assert_eq!(t.with_prefix("üb"), vec!["üben", "über"]);
}
