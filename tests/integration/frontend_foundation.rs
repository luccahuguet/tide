use ferox_diag::Severity;
use ferox_syntax::{bootstrap_parse, tokenize, SourceFile, SourceFileId, TokenKind};

#[test]
fn frontend_foundation_reports_tokens_and_structured_diagnostics() {
    let file = SourceFile::new(SourceFileId(7), "tests/sample.ferox", "fn main() -> Int { @ }");
    let (tokens, diagnostics) = tokenize(&file);

    assert_eq!(tokens[0].kind, TokenKind::KeywordFn);
    assert_eq!(diagnostics.len(), 1);
    assert_eq!(diagnostics[0].severity, Severity::Error);
    assert_eq!(diagnostics[0].code, "syntax.unknown_token");
    assert!(diagnostics[0].primary.is_some());
}

#[test]
fn frontend_foundation_builds_a_syntax_root_for_simple_source() {
    let file = SourceFile::new(SourceFileId(8), "tests/simple.ferox", "fn main() -> Int { 1 }");
    let tree = bootstrap_parse(&file).expect("simple source should parse into the bootstrap tree");

    assert_eq!(tree.file_id, SourceFileId(8));
    assert!(!tree.root.children.is_empty());
}
