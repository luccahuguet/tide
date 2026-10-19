#![forbid(unsafe_code)]

use std::ops::Range;
use std::path::{Path, PathBuf};

use ferox_diag::{Diagnostic, DiagnosticLabel, LineColumn, Span, SpanLocation};

pub const SYNTAX_TOOL: &str = "ferox syntax";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SourceFileId(pub u32);

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LineIndex {
    line_starts: Vec<usize>,
}

impl LineIndex {
    pub fn new(text: &str) -> Self {
        let mut line_starts = vec![0];
        for (idx, byte) in text.bytes().enumerate() {
            if byte == b'\n' {
                line_starts.push(idx + 1);
            }
        }
        Self { line_starts }
    }

    pub fn line_col(&self, byte_offset: usize) -> LineColumn {
        let line_idx = self
            .line_starts
            .partition_point(|start| *start <= byte_offset)
            .saturating_sub(1);
        let line_start = self.line_starts[line_idx];
        LineColumn::new(line_idx + 1, byte_offset.saturating_sub(line_start) + 1)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceFile {
    id: SourceFileId,
    path: PathBuf,
    text: String,
    line_index: LineIndex,
}

impl SourceFile {
    pub fn new(id: SourceFileId, path: impl Into<PathBuf>, text: impl Into<String>) -> Self {
        let text = text.into();
        let line_index = LineIndex::new(&text);
        Self {
            id,
            path: path.into(),
            text,
            line_index,
        }
    }

    pub const fn id(&self) -> SourceFileId {
        self.id
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    pub fn text(&self) -> &str {
        &self.text
    }

    pub fn slice(&self, span: Span) -> &str {
        &self.text[span.start_byte..span.end_byte]
    }

    pub fn locate(&self, span: Span) -> SpanLocation {
        SpanLocation::new(
            span,
            self.line_index.line_col(span.start_byte),
            self.line_index.line_col(span.end_byte),
        )
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TokenKind {
    Identifier,
    Integer,
    KeywordFn,
    KeywordLet,
    KeywordMut,
    KeywordAsync,
    LeftParen,
    RightParen,
    LeftBrace,
    RightBrace,
    LeftBracket,
    RightBracket,
    Comma,
    Colon,
    Semicolon,
    Dot,
    Arrow,
    Equal,
    Question,
    Plus,
    Minus,
    Unknown,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Token {
    pub kind: TokenKind,
    pub span: Span,
}

impl Token {
    pub fn lexeme<'a>(&self, file: &'a SourceFile) -> &'a str {
        file.slice(self.span)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SyntaxKind {
    Root,
    Token,
    Error,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SyntaxNode {
    pub kind: SyntaxKind,
    pub span: Span,
    pub children: Vec<SyntaxElement>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SyntaxElement {
    Node(SyntaxNode),
    Token(Token),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SyntaxTree {
    pub file_id: SourceFileId,
    pub root: SyntaxNode,
}

pub fn tokenize(file: &SourceFile) -> (Vec<Token>, Vec<Diagnostic>) {
    let mut tokens = Vec::new();
    let mut diagnostics = Vec::new();
    let bytes = file.text().as_bytes();
    let mut idx = 0usize;

    while idx < bytes.len() {
        let byte = bytes[idx];
        if byte.is_ascii_whitespace() {
            idx += 1;
            continue;
        }

        let start = idx;
        let kind = match byte {
            b'(' => {
                idx += 1;
                TokenKind::LeftParen
            }
            b')' => {
                idx += 1;
                TokenKind::RightParen
            }
            b'{' => {
                idx += 1;
                TokenKind::LeftBrace
            }
            b'}' => {
                idx += 1;
                TokenKind::RightBrace
            }
            b'[' => {
                idx += 1;
                TokenKind::LeftBracket
            }
            b']' => {
                idx += 1;
                TokenKind::RightBracket
            }
            b',' => {
                idx += 1;
                TokenKind::Comma
            }
            b':' => {
                idx += 1;
                TokenKind::Colon
            }
            b';' => {
                idx += 1;
                TokenKind::Semicolon
            }
            b'.' => {
                idx += 1;
                TokenKind::Dot
            }
            b'?' => {
                idx += 1;
                TokenKind::Question
            }
            b'=' => {
                idx += 1;
                TokenKind::Equal
            }
            b'+' => {
                idx += 1;
                TokenKind::Plus
            }
            b'-' => {
                if bytes.get(idx + 1) == Some(&b'>') {
                    idx += 2;
                    TokenKind::Arrow
                } else {
                    idx += 1;
                    TokenKind::Minus
                }
            }
            b'0'..=b'9' => {
                idx += 1;
                while bytes.get(idx).is_some_and(u8::is_ascii_digit) {
                    idx += 1;
                }
                TokenKind::Integer
            }
            b'a'..=b'z' | b'A'..=b'Z' | b'_' => {
                idx += 1;
                while bytes
                    .get(idx)
                    .is_some_and(|next| next.is_ascii_alphanumeric() || *next == b'_')
                {
                    idx += 1;
                }
                classify_ident(&file.text()[start..idx])
            }
            _ => {
                idx += 1;
                let span = Span::new(start, idx);
                diagnostics.push(
                    Diagnostic::error(SYNTAX_TOOL, "syntax.unknown_token", "unknown token")
                        .with_primary(
                            DiagnosticLabel::new("unknown token", file.path().to_path_buf())
                                .with_location(file.locate(span)),
                        ),
                );
                TokenKind::Unknown
            }
        };

        tokens.push(Token {
            kind,
            span: Span::new(start, idx),
        });
    }

    (tokens, diagnostics)
}

fn classify_ident(text: &str) -> TokenKind {
    match text {
        "fn" => TokenKind::KeywordFn,
        "let" => TokenKind::KeywordLet,
        "mut" => TokenKind::KeywordMut,
        "async" => TokenKind::KeywordAsync,
        _ => TokenKind::Identifier,
    }
}

pub fn bootstrap_parse(file: &SourceFile) -> Result<SyntaxTree, Diagnostic> {
    let (tokens, diagnostics) = tokenize(file);
    if let Some(diagnostic) = diagnostics.into_iter().next() {
        return Err(diagnostic);
    }

    if tokens.is_empty() {
        return Err(
            Diagnostic::error(SYNTAX_TOOL, "syntax.empty_source", "source file is empty")
                .with_primary(
                    DiagnosticLabel::new("empty source", file.path().to_path_buf())
                        .with_location(file.locate(Span::new(0, 0))),
                ),
        );
    }

    let root_span = Span::new(0, file.text().len());
    let children = tokens.into_iter().map(SyntaxElement::Token).collect();
    Ok(SyntaxTree {
        file_id: file.id(),
        root: SyntaxNode {
            kind: SyntaxKind::Root,
            span: root_span,
            children,
        },
    })
}

pub fn token_spans(tokens: &[Token]) -> Vec<Range<usize>> {
    tokens
        .iter()
        .map(|token| token.span.start_byte..token.span.end_byte)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::{
        bootstrap_parse, token_spans, tokenize, SourceFile, SourceFileId, Span, SyntaxKind,
        TokenKind,
    };

    #[test]
    fn line_index_reports_one_based_line_and_column() {
        let file = SourceFile::new(SourceFileId(1), "main.ferox", "fn main() {\n  let x = 1;\n}");
        let location = file.locate(Span::new(14, 17));

        assert_eq!(location.start.line, 2);
        assert_eq!(location.start.column, 3);
        assert_eq!(location.end.line, 2);
        assert_eq!(location.end.column, 6);
    }

    #[test]
    fn tokenize_tracks_keywords_punctuation_and_ranges() {
        let file = SourceFile::new(SourceFileId(1), "main.ferox", "fn main() -> Int { let mut x = 1; }");
        let (tokens, diagnostics) = tokenize(&file);

        assert!(diagnostics.is_empty());
        assert_eq!(tokens[0].kind, TokenKind::KeywordFn);
        assert_eq!(tokens[1].kind, TokenKind::Identifier);
        assert_eq!(tokens[4].kind, TokenKind::Arrow);
        assert_eq!(tokens[7].kind, TokenKind::KeywordLet);
        assert_eq!(tokens[8].kind, TokenKind::KeywordMut);
        assert_eq!(token_spans(&tokens)[0], 0..2);
    }

    #[test]
    fn tokenize_emits_diagnostic_for_unknown_token() {
        let file = SourceFile::new(SourceFileId(1), "main.ferox", "@");
        let (_tokens, diagnostics) = tokenize(&file);

        assert_eq!(diagnostics.len(), 1);
        assert_eq!(diagnostics[0].code, "syntax.unknown_token");
        assert!(diagnostics[0].primary.is_some());
    }

    #[test]
    fn bootstrap_parse_rejects_empty_text() {
        let file = SourceFile::new(SourceFileId(1), "empty.ferox", "");

        let diagnostic = bootstrap_parse(&file).unwrap_err();

        assert_eq!(diagnostic.code, "syntax.empty_source");
        assert!(diagnostic.primary.is_some());
    }

    #[test]
    fn bootstrap_parse_wraps_tokens_in_root_syntax_node() {
        let file = SourceFile::new(SourceFileId(1), "main.ferox", "fn main() -> Int { 1 }");

        let tree = bootstrap_parse(&file).unwrap();

        assert_eq!(tree.file_id, SourceFileId(1));
        assert_eq!(tree.root.kind, SyntaxKind::Root);
        assert!(!tree.root.children.is_empty());
    }
}
