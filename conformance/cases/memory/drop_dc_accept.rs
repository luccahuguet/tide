//@ rules: DROP-1
// Rule DC accepts: a type without a destructor, and an audited std container,
// may drop after the data they borrow; a destructor may borrow what outlives it.
pub struct View<'a> {
    pub text: &'a String,
}

pub struct Guard<'a> {
    pub text: &'a String,
}
impl Drop for Guard<'_> {
    fn drop(&mut self) {}
}

pub fn ok() {
    let view;
    let list;
    let text = String::from("x");
    view = View { text: &text };
    list = vec![&text];
    let guard = Guard { text: &text };
    let _ = (view.text, list.len(), guard.text);
}
