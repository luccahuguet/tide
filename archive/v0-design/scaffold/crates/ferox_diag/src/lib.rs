#![forbid(unsafe_code)]

use std::path::PathBuf;

pub const TRACE_ENV_VAR: &str = "FEROX_LOG";
pub const DIAGNOSTICS_SCHEMA_VERSION: &str = "1";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Severity {
    Error,
    Warning,
    Note,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Span {
    pub start_byte: usize,
    pub end_byte: usize,
}

impl Span {
    pub const fn new(start_byte: usize, end_byte: usize) -> Self {
        Self {
            start_byte,
            end_byte,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct LineColumn {
    pub line: usize,
    pub column: usize,
}

impl LineColumn {
    pub const fn new(line: usize, column: usize) -> Self {
        Self { line, column }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SpanLocation {
    pub span: Span,
    pub start: LineColumn,
    pub end: LineColumn,
}

impl SpanLocation {
    pub const fn new(span: Span, start: LineColumn, end: LineColumn) -> Self {
        Self { span, start, end }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DiagnosticLabel {
    pub message: String,
    pub file: PathBuf,
    pub location: Option<SpanLocation>,
}

impl DiagnosticLabel {
    pub fn new(message: impl Into<String>, file: impl Into<PathBuf>) -> Self {
        Self {
            message: message.into(),
            file: file.into(),
            location: None,
        }
    }

    pub fn with_location(mut self, location: SpanLocation) -> Self {
        self.location = Some(location);
        self
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SuggestedFix {
    pub message: String,
}

impl SuggestedFix {
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Diagnostic {
    pub schema_version: &'static str,
    pub tool: &'static str,
    pub code: &'static str,
    pub severity: Severity,
    pub message: String,
    pub primary: Option<DiagnosticLabel>,
    pub related: Vec<DiagnosticLabel>,
    pub suggested_fixes: Vec<SuggestedFix>,
}

impl Diagnostic {
    pub fn new(
        tool: &'static str,
        severity: Severity,
        code: &'static str,
        message: impl Into<String>,
    ) -> Self {
        Self {
            schema_version: DIAGNOSTICS_SCHEMA_VERSION,
            tool,
            code,
            severity,
            message: message.into(),
            primary: None,
            related: Vec::new(),
            suggested_fixes: Vec::new(),
        }
    }

    pub fn error(tool: &'static str, code: &'static str, message: impl Into<String>) -> Self {
        Self::new(tool, Severity::Error, code, message)
    }

    pub fn with_primary(mut self, label: DiagnosticLabel) -> Self {
        self.primary = Some(label);
        self
    }

    pub fn with_related(mut self, label: DiagnosticLabel) -> Self {
        self.related.push(label);
        self
    }

    pub fn with_fix(mut self, fix: SuggestedFix) -> Self {
        self.suggested_fixes.push(fix);
        self
    }
}

#[cfg(test)]
mod tests {
    use super::{
        Diagnostic, DiagnosticLabel, LineColumn, Severity, Span, SpanLocation, SuggestedFix,
        DIAGNOSTICS_SCHEMA_VERSION,
    };

    #[test]
    fn diagnostic_builder_keeps_machine_contract_shape() {
        let primary = DiagnosticLabel::new("workspace ready", "src/main.ferox").with_location(
            SpanLocation::new(Span::new(1, 5), LineColumn::new(1, 2), LineColumn::new(1, 6)),
        );
        let related = DiagnosticLabel::new("created here", "src/lib.ferox");
        let fix = SuggestedFix::new("remove the duplicate binding");

        let diag = Diagnostic::error("ferox check", "bootstrap.smoke", "workspace ready")
            .with_primary(primary.clone())
            .with_related(related.clone())
            .with_fix(fix.clone());

        assert_eq!(diag.schema_version, DIAGNOSTICS_SCHEMA_VERSION);
        assert_eq!(diag.tool, "ferox check");
        assert_eq!(diag.code, "bootstrap.smoke");
        assert_eq!(diag.severity, Severity::Error);
        assert_eq!(diag.message, "workspace ready");
        assert_eq!(diag.primary, Some(primary));
        assert_eq!(diag.related, vec![related]);
        assert_eq!(diag.suggested_fixes, vec![fix]);
    }
}
