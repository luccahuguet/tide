#[test]
fn case1() {
    let text = "listen silent enlist google tinsel";
    assert_eq!(
        AnagramIndex::new(text).groups(),
        vec![vec!["listen", "silent", "enlist", "tinsel"]]
    );
}
#[test]
fn case2() {
    assert_eq!(
        AnagramIndex::new("Dusty study, night thing; dusty").groups(),
        vec![vec!["Dusty", "study", "dusty"], vec!["night", "thing"]]
    );
}
#[test]
fn case3() {
    for text in ["", "abc def"] {
        assert!(AnagramIndex::new(text).groups().is_empty());
    }
}
#[test]
fn case4() {
    let text = String::from("listen silent enlist google tinsel");
    let groups = {
        let index = AnagramIndex::new(&text);
        index.groups()
    };
    assert_eq!(groups, vec![vec!["listen", "silent", "enlist", "tinsel"]]);
}
#[test]
fn case5() {
    let text = String::from("listen silent enlist google tinsel");
    let result = {
        let index = AnagramIndex::new(&text);
        let word = String::from("listen");
        index.partners(&word)
    };
    assert_eq!(result, vec!["silent", "enlist", "tinsel"]);
}
