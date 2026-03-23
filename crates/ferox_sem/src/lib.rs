#![forbid(unsafe_code)]

use ferox_ast::ModuleAst;
use ferox_diag::Diagnostic;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Analysis {
    pub diagnostics: Vec<Diagnostic>,
}

pub fn bootstrap_analyze(_module: &ModuleAst) -> Analysis {
    Analysis {
        diagnostics: Vec::new(),
    }
}

#[cfg(test)]
mod tests {
    use super::bootstrap_analyze;
    use ferox_ast::ModuleAst;
    use ferox_syntax::SourceFileId;

    #[test]
    fn bootstrap_analysis_starts_with_no_diagnostics() {
        let analysis = bootstrap_analyze(&ModuleAst::new(SourceFileId(1)));

        assert!(analysis.diagnostics.is_empty());
    }
}

