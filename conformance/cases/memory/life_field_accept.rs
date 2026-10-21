//@ rules: LIFE-6
// A reference field names a lifetime parameter of its type or `'static`.
pub struct Token<'a> {
    pub text: &'a str,
    pub kind: &'static str,
}

pub enum Part<'a> {
    Text(&'a str),
    End,
}
