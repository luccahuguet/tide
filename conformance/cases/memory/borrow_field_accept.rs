//@ rules: BORROW-1
// Field references, borrowing types, and `dyn` bounds use a declared lifetime or `'static`.
use std::fmt::Display;

pub struct Node<'a> { val: i32, next: Option<&'a Node<'a>> }
pub struct Collector<'s, 'o> { src: &'s str, out: &'o mut Vec<&'s str> }
pub struct Lexer<'a> { src: &'a str, pos: usize }
pub struct Parser<'a> { lx: Lexer<'a>, last: Option<&'a str> }
pub struct Tagged<'t, T> { value: T, tag: &'t str }
pub struct ParseError { msg: &'static str }
pub struct Filter<'p> { pred: Box<dyn Fn(&str) -> bool + 'p>, show: &'p dyn Display }
pub struct Hook { run: Box<dyn Fn()> } // `dyn Fn() + 'static` by default
