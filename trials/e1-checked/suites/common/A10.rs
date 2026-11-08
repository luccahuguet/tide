#[test]
fn case1() {
    for (text, expected) in [
        (
            "Hi there. How are you? Fine!",
            vec!["Hi there.", "How are you?", "Fine!"],
        ),
        ("a.b. c", vec!["a.b.", "c"]),
        ("No end", vec!["No end"]),
        ("   ", vec![]),
    ] {
        assert_eq!(Doc::new(text).sentences().collect::<Vec<_>>(), expected);
    }
}
#[test]
fn case2() {
    assert_eq!(Doc::new("ab. cd. e.").longest(), Some("ab."));
    let d = Doc::new("Hi there. How are you? Fine!");
    assert_eq!(d.words_of(1), Some(vec!["How", "are", "you"]));
    assert_eq!(d.words_of(9), None);
}
#[test]
fn case3() {
    assert_eq!(
        Doc::new("Hi there. How are you? Fine!")
            .with_word("HI")
            .collect::<Vec<_>>(),
        vec!["Hi there."]
    );
}
#[test]
fn case4() {
    let text = String::from("Hi there. How are you? Fine!");
    let word = String::from("HI");
    let iter = {
        let d = Doc::new(&text);
        d.sentences()
    };
    assert_eq!(
        iter.collect::<Vec<_>>(),
        vec!["Hi there.", "How are you?", "Fine!"]
    );
    let iter = {
        let d = Doc::new(&text);
        d.with_word(&word)
    };
    assert_eq!(iter.collect::<Vec<_>>(), vec!["Hi there."]);
}
