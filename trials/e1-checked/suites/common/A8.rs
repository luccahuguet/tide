use std::collections::HashMap;
#[test]
fn case1() {
    let t = Template::parse("Hi {name}, {{ok}} {name}!").unwrap();
    assert_eq!(t.names(), vec!["name"]);
    assert_eq!(
        t.render(&HashMap::from([("name", "Ann")])).unwrap(),
        "Hi Ann, {ok} Ann!"
    );
}
#[test]
fn case2() {
    assert_eq!(
        Template::parse("{{{x}}}")
            .unwrap()
            .render(&HashMap::from([("x", "1")]))
            .unwrap(),
        "{1}"
    );
    assert_eq!(
        Template::parse("}}")
            .unwrap()
            .render(&HashMap::new())
            .unwrap(),
        "}"
    );
}
#[test]
fn case3() {
    let e = Template::parse("{a}{b}")
        .unwrap()
        .render(&HashMap::from([("a", "1")]))
        .unwrap_err();
    assert_eq!(e, TemplateError::Missing { name: "b".into() });
    assert_eq!(e.to_string(), "missing value for b");
}
#[test]
fn case4() {
    assert!(matches!(
        Template::parse("x {y"),
        Err(TemplateError::Unclosed { pos: 2 })
    ));
    for s in ["{1x}", "{}", "{ a }"] {
        assert!(matches!(
            Template::parse(s),
            Err(TemplateError::BadName { pos: 0 })
        ));
    }
    assert!(matches!(
        Template::parse("a } b"),
        Err(TemplateError::StrayBrace { pos: 2 })
    ));
}
#[test]
fn case5() {
    let src = String::from("{alpha}{beta}{alpha}");
    let names = {
        let t = Template::parse(&src).unwrap();
        t.names()
    };
    assert_eq!(names, vec!["alpha", "beta"]);
}
