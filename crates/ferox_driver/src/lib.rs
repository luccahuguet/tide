#![forbid(unsafe_code)]

use ferox_ast::ModuleAst;
use ferox_diag::TRACE_ENV_VAR;
use ferox_fmt::bootstrap_format;
use ferox_sem::bootstrap_analyze;
use ferox_syntax::{bootstrap_parse, SourceFile, SourceFileId};

pub const WORKSPACE_CRATES: &[&str] = &[
    "ferox_cli",
    "ferox_driver",
    "ferox_diag",
    "ferox_syntax",
    "ferox_ast",
    "ferox_sem",
    "ferox_fmt",
];

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BootstrapStatus {
    pub workspace_crates: &'static [&'static str],
    pub trace_env_var: &'static str,
}

impl BootstrapStatus {
    pub const fn render(&self) -> &'static str {
        "Ferox Rust bootstrap workspace is wired and ready for T073."
    }
}

pub fn bootstrap_status() -> BootstrapStatus {
    BootstrapStatus {
        workspace_crates: WORKSPACE_CRATES,
        trace_env_var: TRACE_ENV_VAR,
    }
}

pub fn bootstrap_smoke_pipeline(text: &str) -> Result<String, String> {
    let file = SourceFile::new(SourceFileId(1), "smoke.ferox", text);
    let syntax = bootstrap_parse(&file).map_err(|diagnostic| diagnostic.message)?;
    let module = ModuleAst::new(syntax.file_id);
    let analysis = bootstrap_analyze(&module);

    if let Some(diagnostic) = analysis.diagnostics.first() {
        return Err(diagnostic.message.clone());
    }

    Ok(bootstrap_format(&file))
}

#[cfg(test)]
mod tests {
    use super::{bootstrap_smoke_pipeline, bootstrap_status, WORKSPACE_CRATES};

    #[test]
    fn workspace_status_reports_expected_crates() {
        let status = bootstrap_status();

        assert_eq!(status.workspace_crates, WORKSPACE_CRATES);
        assert_eq!(status.trace_env_var, "FEROX_LOG");
    }

    #[test]
    fn bootstrap_smoke_pipeline_formats_non_empty_input() {
        let formatted = bootstrap_smoke_pipeline("fn main() -> Int { 1 }").unwrap();

        assert_eq!(formatted, "fn main() -> Int { 1 }\n");
    }
}
