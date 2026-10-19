//! A lexer whose tokens borrow from the source text.

/// Punctuation, two-character forms first so that `==` wins over `=`.
const PUNCT: &[&str] = &[
    "::", "->", "=>", "==", "!=", "<=", ">=", "&&", "||",
    "(", ")", "{", "}", "[", "]", ",", ";", ":", ".",
    "+", "-", "*", "/", "%", "=", "<", ">", "!", "&", "|",
];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Kind {
    Ident,
    Int,
    Punct,
}

/// A token: its kind, its text (a slice of the source), and its byte offset.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Token {
    pub kind: Kind,
    pub text: &str,
    pub offset: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
#[error("unexpected character {ch:?} at byte {offset}")]
pub struct LexError {
    pub ch: char,
    pub offset: usize,
}

pub struct Lexer {
    src: &str,
    pos: usize,
}

impl Lexer {
    pub fn new(src: &str) -> Lexer {
        Lexer { src, pos: 0 }
    }

    /// The next token, or `None` at the end of input. An unexpected
    /// character is reported once and skipped, so lexing can continue.
    /// The token borrows the source, not the lexer.
    pub fn next_token(&mut self) -> Option<Result<Token from self.src, LexError>> {
        let src = self.src;                             // copy the `&str` out: it borrows the source
        let rest = src[self.pos..].trim_start();
        let offset = src.len() - rest.len();
        self.pos = offset;
        let c = rest.chars().next()?;
        let token = if c == '_' || c.is_alphabetic() {
            Some((Kind::Ident, prefix_len(rest, |ch| ch == '_' || ch.is_alphanumeric())))
        } else if c.is_ascii_digit() {
            Some((Kind::Int, prefix_len(rest, |ch| ch.is_ascii_digit())))
        } else {
            PUNCT.iter().find(|p| rest.starts_with(**p)).map(|p| (Kind::Punct, p.len()))
        };
        match token {
            Some((kind, len)) => {
                self.pos = offset + len;
                Some(Ok(Token { kind, text: &src[offset..self.pos], offset }))
            }
            None => {
                self.pos = offset + c.len_utf8();
                Some(Err(LexError { ch: c, offset }))
            }
        }
    }
}

impl Iterator for Lexer {
    type Item = Result<Token from Self.src, LexError>;

    fn next(&mut self) -> Option<Self::Item> {
        self.next_token()
    }
}

/// Byte length of the longest prefix of `s` whose chars all satisfy `keep`.
fn prefix_len(s: &str, keep: impl Fn(char) -> bool) -> usize {
    s.find(|ch: char| !keep(ch)).unwrap_or(s.len())
}

/// Every token of `src`, or the first error.
pub fn tokenize(src: &str) -> Result<Vec<Token>, LexError> {
    Lexer::new(src).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn kinds_and_texts(src: &str) -> Vec<(Kind, &str)> {
        Lexer::new(src)
            .map(|t| {
                let t = t.unwrap();
                (t.kind, t.text)
            })
            .collect()
    }

    #[test]
    fn lexes_identifiers_integers_and_punctuation() {
        assert_eq!(kinds_and_texts("let x_1 = 42+y;"), vec![
            (Kind::Ident, "let"), (Kind::Ident, "x_1"), (Kind::Punct, "="),
            (Kind::Int, "42"), (Kind::Punct, "+"), (Kind::Ident, "y"), (Kind::Punct, ";"),
        ]);
        assert_eq!(kinds_and_texts("a==b->c::d 12ab"), vec![
            (Kind::Ident, "a"), (Kind::Punct, "=="), (Kind::Ident, "b"), (Kind::Punct, "->"),
            (Kind::Ident, "c"), (Kind::Punct, "::"), (Kind::Ident, "d"),
            (Kind::Int, "12"), (Kind::Ident, "ab"),
        ]);
        assert!(kinds_and_texts("  \n\t").is_empty());
    }

    #[test]
    fn tokens_outlive_the_lexer() {
        let src = String::from("first second");
        let (a, b) = {
            let mut lx = Lexer::new(&src);
            let a = lx.next().unwrap().unwrap();
            let b = lx.next().unwrap().unwrap();        // fine: `a` borrows `src`, not `lx`
            (a, b)
        };
        assert_eq!((a.text, b.text), ("first", "second"));
        assert_eq!(b.offset, 6);
    }

    #[test]
    fn reports_and_skips_unexpected_characters() {
        let got: Vec<_> = Lexer::new("a $ b").collect();
        assert_eq!(got, vec![
            Ok(Token { kind: Kind::Ident, text: "a", offset: 0 }),
            Err(LexError { ch: '$', offset: 2 }),
            Ok(Token { kind: Kind::Ident, text: "b", offset: 4 }),
        ]);
        assert_eq!(tokenize("a $ b"), Err(LexError { ch: '$', offset: 2 }));
        assert_eq!(tokenize("x€1").unwrap_err(), LexError { ch: '€', offset: 1 });
    }

    #[test]
    fn token_text_is_the_source_at_its_offset(src: String) {
        for t in Lexer::new(&src).flatten() {
            assert!(!t.text.is_empty());
            assert_eq!(&src[t.offset..t.offset + t.text.len()], t.text);
        }
    }
}
