#![forbid(unsafe_code)]

use ferox_syntax::SourceFileId;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ModuleAst {
    pub file_id: SourceFileId,
}

impl ModuleAst {
    pub const fn new(file_id: SourceFileId) -> Self {
        Self { file_id }
    }
}

#[cfg(test)]
mod tests {
    use super::ModuleAst;
    use ferox_syntax::SourceFileId;

    #[test]
    fn module_ast_tracks_originating_file() {
        let ast = ModuleAst::new(SourceFileId(7));

        assert_eq!(ast.file_id, SourceFileId(7));
    }
}

