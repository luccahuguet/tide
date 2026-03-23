#![forbid(unsafe_code)]

use ferox_diag::{Diagnostic, Span};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SourceFileId(pub u32);

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceFile {
    pub id: SourceFileId,
    pub path: String,
    pub text: String,
}

impl SourceFile {
    pub fn new(id: SourceFileId, path: impl Into<String>, text: impl Into<String>) -> Self {
        Self {
            id,
            path: path.into(),
            text: text.into(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SyntaxTree {
    pub file_id: SourceFileId,
}

pub fn bootstrap_parse(file: &SourceFile) -> Result<SyntaxTree, Diagnostic> {
    if file.text.trim().is_empty() {
        Err(Diagnostic::error("syntax.empty_source", "source file is empty").with_span(Span::new(0, 0)))
    } else {
        Ok(SyntaxTree { file_id: file.id })
    }
}

#[cfg(test)]
mod tests {
    use super::{bootstrap_parse, SourceFile, SourceFileId};

    #[test]
    fn bootstrap_parse_rejects_empty_text() {
        let file = SourceFile::new(SourceFileId(1), "empty.ferox", "");

        let diagnostic = bootstrap_parse(&file).unwrap_err();

        assert_eq!(diagnostic.code, "syntax.empty_source");
    }

    #[test]
    fn bootstrap_parse_accepts_non_empty_text() {
        let file = SourceFile::new(SourceFileId(1), "main.ferox", "fn main() -> Int { 1 }");

        let tree = bootstrap_parse(&file).unwrap();

        assert_eq!(tree.file_id, SourceFileId(1));
    }
}

