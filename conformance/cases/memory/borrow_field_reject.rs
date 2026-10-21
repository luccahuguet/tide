//@ rules: BORROW-1
// A field's reference, borrowing type, or `dyn` bound with no declared lifetime is an error.
pub struct Lexer<'a> { src: &'a str, pos: usize }

pub struct Bare { s: &str } //~ E0106
pub struct Undeclared<'a> { s: &'a str, t: &'b str } //~ E0261
pub struct Elided { lx: Lexer } //~ E0106
pub struct BehindRef { show: &dyn Fn() } //~ E0106
pub struct Bound { pred: Box<dyn Fn() + 'p> } //~ E0261
