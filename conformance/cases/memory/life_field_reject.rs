//@ rules: LIFE-6
// Fields have no elision.
pub struct Token {
    pub text: &str, //~ E0106
}

pub enum Part {
    Text(&str), //~ E0106
    End,
}
