//! Extra Tide names for rustc-owned verdicts; labels never decide acceptance.
use std::sync::{Arc, Mutex};

use rustc_errors::annotate_snippet_emitter_writer::AnnotateSnippetEmitter;
use rustc_errors::emitter::{DynEmitter, Emitter, stderr_destination};
use rustc_errors::{ColorConfig, DiagInner, Level, format_diag_message};
use rustc_session::parse::ParseSess;
use rustc_span::Span;
use rustc_span::source_map::SourceMap;

use super::Code;

struct SelfBorrow {
    owner: Span,
    borrow: Span,
}

impl SelfBorrow {
    fn matches(&self, primary: Span, rustc_code: Option<u32>) -> bool {
        match rustc_code {
            Some(505) => self.owner.lo() == primary.lo() && self.owner.hi() == primary.hi(),
            Some(515 | 597) => self.borrow.contains(primary),
            _ => false,
        }
    }
}

pub struct OwnedClosure {
    pub declaration: Span,
    pub expression: Span,
    pub requirements: Vec<Span>,
    pub uses: Vec<Span>,
    pub origins: Vec<Span>,
}

impl OwnedClosure {
    // Label-layer policy: supplementary explanations must never be false. Prefer no
    // label when provenance is uncertain; rustc keeps its original error and rejection.
    // E0373 uses rustc's closure/borrowed-place spans, independently of provenance.
    // Other errors require a fully certain whole-binding chain; any unsupported step
    // discards that capture's evidence, never broadens it to a containing expression.
    // Steps are whole-local aliases, &/&mut local, and receiver-tied borrow methods;
    // projections, destructuring, reassignment and opaque borrowed values invalidate it.
    fn matches(&self, diag: &DiagInner, rustc_code: Option<u32>, region_error: bool) -> bool {
        let Some(primary) = diag.span.primary_span() else {
            return false;
        };
        if rustc_code == Some(373) {
            return self.declaration.contains(primary)
                && diag.span.span_labels_raw().iter().any(|(span, _)| {
                    self.expression.contains(*span) && !self.declaration.contains(*span)
                });
        }
        let requirement = |span: Span| {
            self.requirements
                .iter()
                .any(|target| target.lo() == span.lo() && target.hi() == span.hi())
        };
        // A call/return span includes the closure body. It proves the destination,
        // not which capture caused the failure (another argument may be at fault).
        let touches = |anchors: &[Span]| {
            diag.span.span_labels_raw().iter().any(|(span, _)| {
                !requirement(*span) && anchors.iter().any(|anchor| anchor.overlaps(*span))
            })
        };
        let static_requirement =
            diag.span.span_labels_raw().iter().any(|(span, message)| {
                requirement(*span) && format_diag_message(message, &diag.args).contains("'static")
            }) || diag.children.iter().any(|child| {
                child.messages.iter().any(|(message, _)| {
                    format_diag_message(message, &diag.args).contains("'static")
                }) && child
                    .span
                    .primary_spans()
                    .iter()
                    .any(|&span| requirement(span))
            });
        let capture = touches(&self.origins) || touches(&self.uses);
        match rustc_code {
            Some(521 | 597) => static_requirement && capture,
            // Returning a captured reference alias can be E0515 instead of E0597.
            Some(515) => requirement(primary) && capture,
            None if region_error => static_requirement && capture,
            _ => false,
        }
    }
}

#[derive(Default)]
struct Sites {
    self_borrows: Vec<SelfBorrow>,
    owned: Vec<OwnedClosure>,
}

#[derive(Clone, Default)]
pub struct RustcLabels(Arc<Mutex<Sites>>);

impl RustcLabels {
    pub fn record_self_borrow(&self, owner: Span, borrow: Span) {
        self.0
            .lock()
            .unwrap()
            .self_borrows
            .push(SelfBorrow { owner, borrow });
    }

    pub fn owned(&self, site: OwnedClosure) {
        self.0.lock().unwrap().owned.push(site);
    }

    pub fn install(&self, psess: &mut ParseSess) {
        // The operator interface fixes rustc's output to --error-format=short.
        let inner = AnnotateSnippetEmitter::new(stderr_destination(ColorConfig::Never))
            .sm(Some(psess.clone_source_map()))
            .short_message(true);
        psess.dcx().set_emitter(Box::new(Labels {
            inner: Box::new(inner),
            sites: self.clone(),
        }));
    }
}

struct Labels {
    inner: Box<DynEmitter>,
    sites: RustcLabels,
}

impl Emitter for Labels {
    fn emit_diagnostic(&mut self, diag: DiagInner) {
        let mut labels = Vec::new();
        if diag.level() == Level::Error
            && let Some(span) = diag.span.primary_span()
        {
            let rustc_code = diag.code.map(|code| code.as_u32());
            let region_error = rustc_code.is_none()
                && diag.messages.iter().any(|(message, _)| {
                    format_diag_message(message, &diag.args) == "lifetime may not live long enough"
                });
            let sites = self.sites.0.lock().unwrap();
            // A self-borrow elsewhere in the literal does not explain another field's error.
            if sites
                .self_borrows
                .iter()
                .any(|site| site.matches(span, rustc_code))
            {
                labels.push(Code::Fx0210);
            }
            if sites
                .owned
                .iter()
                .any(|site| site.matches(&diag, rustc_code, region_error))
            {
                labels.push(Code::Fx0203);
            }
        }
        let span = diag.span.primary_span();
        self.inner.emit_diagnostic(diag);
        for code in labels {
            let message = match code {
                Code::Fx0203 => {
                    "a closure in an owned position cannot capture a non-static borrow; move owned data in or name its lifetime"
                }
                Code::Fx0210 => {
                    "this struct literal borrows from an owner it moves into a field; keep the owner and view apart or store indices"
                }
                _ => unreachable!(),
            };
            let mut label = DiagInner::new(Level::Error, format!("{}: {message}", code.as_str()));
            label.span = span.unwrap().into();
            // Forward directly: rustc already counted the rejecting error, and a label
            // must neither re-enter the diagnostic context nor label itself.
            self.inner.emit_diagnostic(label);
        }
    }

    fn should_show_explain(&self) -> bool {
        self.inner.should_show_explain()
    }

    fn supports_color(&self) -> bool {
        self.inner.supports_color()
    }

    fn source_map(&self) -> Option<&SourceMap> {
        self.inner.source_map()
    }
}
