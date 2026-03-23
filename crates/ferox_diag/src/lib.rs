#![forbid(unsafe_code)]

pub const TRACE_ENV_VAR: &str = "FEROX_LOG";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Severity {
    Error,
    Warning,
    Note,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Span {
    pub start: usize,
    pub end: usize,
}

impl Span {
    pub const fn new(start: usize, end: usize) -> Self {
        Self { start, end }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Diagnostic {
    pub code: &'static str,
    pub severity: Severity,
    pub message: String,
    pub span: Option<Span>,
}

impl Diagnostic {
    pub fn error(code: &'static str, message: impl Into<String>) -> Self {
        Self {
            code,
            severity: Severity::Error,
            message: message.into(),
            span: None,
        }
    }

    pub fn with_span(mut self, span: Span) -> Self {
        self.span = Some(span);
        self
    }
}

#[cfg(test)]
mod tests {
    use super::{Diagnostic, Severity, Span};

    #[test]
    fn diagnostic_builder_keeps_code_message_and_span() {
        let diag = Diagnostic::error("bootstrap.smoke", "workspace ready").with_span(Span::new(1, 5));

        assert_eq!(diag.code, "bootstrap.smoke");
        assert_eq!(diag.severity, Severity::Error);
        assert_eq!(diag.message, "workspace ready");
        assert_eq!(diag.span, Some(Span::new(1, 5)));
    }
}

