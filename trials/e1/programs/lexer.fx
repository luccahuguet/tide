// src/lexer.fx of a library package
//
// A lexer over a borrowed source string. Tokens borrow their text from the
// source, not from the lexer, so they stay usable after the lexer advances or
// is dropped.

/// Two-character operators, tried before single characters.
const TWO_CHAR_PUNCT: [&str; 9] = ["==", "!=", "<=", ">=", "->", "=>", "&&", "||", "::"];
const ONE_CHAR_PUNCT: &str = "+-*/%=<>!&|^~()[]{};:,.?@#";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Token {
    Ident(&str),
    /// Decimal digits, `_` separators allowed; parse with `str::parse` when needed.
    Int(&str),
    Punct(&str),
}

impl Token {
    /// The source text of the token. Borrows from the source the token
    /// borrows from (`*self`), not from the `&self` reference.
    pub fn text(&self) -> &str from *self {
        match *self {
            Token::Ident(s) | Token::Int(s) | Token::Punct(s) => s,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Error)]
#[error("unexpected character {ch:?} at byte {at}")]
pub struct LexError {
    pub ch: char,
    pub at: usize,
}

pub struct Lexer {
    src: &str,
    pos: usize,
}

impl Lexer {
    pub fn new(src: &str) -> Lexer {
        Lexer { src, pos: 0 }
    }

    /// Byte offset of the next unread character.
    pub fn position(&self) -> usize {
        self.pos
    }
}

fn is_ident_start(c: char) -> bool {
    c.is_alphabetic() || c == '_'
}

fn is_ident_continue(c: char) -> bool {
    c.is_alphanumeric() || c == '_'
}

fn is_int_continue(c: char) -> bool {
    c.is_ascii_digit() || c == '_'
}

/// Authority-free. The longest prefix of `s` whose chars satisfy `pred`.
fn take_while(s: &str, pred: impl Fn(char) -> bool) -> &str {
    let end = s.find(|c: char| !pred(c)).unwrap_or(s.len());
    &s[..end]
}

/// Authority-free. The token at the very start of `rest`, if any is valid there.
fn scan(rest: &str) -> Option<Token> {
    let c = rest.chars().next()?;
    let tok = if is_ident_start(c) {
        Token::Ident(take_while(rest, is_ident_continue))
    } else if c.is_ascii_digit() {
        Token::Int(take_while(rest, is_int_continue))
    } else if let Some(op) = TWO_CHAR_PUNCT.iter().copied().find(|op| rest.starts_with(*op)) {
        Token::Punct(&rest[..op.len()])
    } else if ONE_CHAR_PUNCT.contains(c) {
        Token::Punct(&rest[..c.len_utf8()])
    } else {
        return None;
    };
    Some(tok)
}

impl Iterator for Lexer {
    type Item = Result<Token from Self.src, LexError>;

    fn next(&mut self) -> Option<Result<Token from self.src, LexError>> {
        let rest = &self.src[self.pos..]; // borrows the source, not `self`
        let trimmed = rest.trim_start();
        self.pos += rest.len() - trimmed.len();
        let c = trimmed.chars().next()?; // end of input
        match scan(trimmed) {
            Some(tok) => {
                self.pos += tok.text().len();
                Some(Ok(tok))
            }
            None => {
                let at = self.pos;
                self.pos += c.len_utf8(); // skip it so the next call makes progress
                Some(Err(LexError { ch: c, at }))
            }
        }
    }
}

/// Authority-free. All identifiers in `src`, skipping invalid characters.
/// The lexer is local; the returned slices borrow `src` (elision).
pub fn identifiers(src: &str) -> Vec<&str> {
    Lexer::new(src)
        .filter_map(|t| match t {
            Ok(Token::Ident(name)) => Some(name),
            _ => None,
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn lex(src: &str) -> Result<Vec<Token>, LexError> {
        Lexer::new(src).collect()
    }

    #[test]
    fn lexes_a_statement() {
        let got = lex("let x_1 = 42 + y;").unwrap();
        assert_eq!(
            got,
            vec![
                Token::Ident("let"),
                Token::Ident("x_1"),
                Token::Punct("="),
                Token::Int("42"),
                Token::Punct("+"),
                Token::Ident("y"),
                Token::Punct(";"),
            ]
        );
    }

    #[test]
    fn prefers_two_char_operators() {
        let got = lex("a<=b->c::d").unwrap();
        let texts: Vec<&str> = got.iter().map(|t| t.text()).collect();
        assert_eq!(texts, ["a", "<=", "b", "->", "c", "::", "d"]);
    }

    #[test]
    fn reports_and_skips_bad_characters() {
        let mut lx = Lexer::new("a $ b");
        assert_eq!(lx.next(), Some(Ok(Token::Ident("a"))));
        assert_eq!(lx.next(), Some(Err(LexError { ch: '$', at: 2 })));
        assert_eq!(lx.next(), Some(Ok(Token::Ident("b"))));
        assert_eq!(lx.next(), None);
        assert_eq!(lx.position(), 5);
    }

    #[test]
    fn tokens_outlive_the_lexer() {
        let src = String::from("alpha beta");
        let (first, second) = {
            let mut lx = Lexer::new(&src);
            let a = lx.next().unwrap().unwrap();
            let b = lx.next().unwrap().unwrap(); // `a` borrows `src`, not `lx`
            (a, b)
        }; // `lx` dropped here
        assert_eq!((first, second), (Token::Ident("alpha"), Token::Ident("beta")));
        assert_eq!(identifiers(&src), ["alpha", "beta"]);
    }

    #[test]
    fn unicode_identifiers_and_whitespace() {
        let got = lex("  δx\t\n7_000 ").unwrap();
        assert_eq!(got, vec![Token::Ident("δx"), Token::Int("7_000")]);
    }

    #[test]
    fn never_panics_and_covers_the_input(src: String) {
        let mut lx = Lexer::new(&src);
        let mut covered = 0;
        while let Some(item) = lx.next() {
            let len = match item {
                Ok(tok) => tok.text().len(),
                Err(e) => e.ch.len_utf8(),
            };
            covered += len;
        }
        assert_eq!(covered, src.chars().filter(|c| !c.is_whitespace()).map(char::len_utf8).sum::<usize>());
    }
}
