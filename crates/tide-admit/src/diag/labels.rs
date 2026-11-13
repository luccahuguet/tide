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
    fn matches(&self, diag: &DiagInner, rustc_code: Option<u32>) -> bool {
        let Some(primary) = diag.span.primary_span() else {
            return false;
        };
        match rustc_code {
            Some(505) => self.owner.lo() == primary.lo() && self.owner.hi() == primary.hi(),
            Some(515) => {
                self.borrow.contains(primary)
                    || (primary.contains(self.owner)
                        && primary.contains(self.borrow)
                        && diag.span.span_labels_raw().iter().any(|(span, message)| {
                            self.borrow.contains(*span)
                                && format_diag_message(message, &diag.args)
                                    .contains("borrowed here")
                        }))
            }
            Some(597) => self.borrow.contains(primary),
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

// Every supplementary code has one row owning its rustc mapping, site policy and text.
// Site recording is typed (`owned`, `record_self_borrow`); arbitrary Codes cannot be recorded.
enum Site {
    Owned,
    SelfBorrow,
    None,
}

struct Rule {
    code: Code,
    rustc_codes: &'static [u32],
    wording: Option<&'static str>,
    site: Site,
    message: &'static str,
}

// Instance-depth errors alone prove no constructor growth. Certain local cycles
// already receive FX0407 from the definition graph; unresolved dispatch stays unlabelled.
const RULES: &[Rule] = &[
    Rule {
        code: Code::Fx0910,
        rustc_codes: &[583],
        wording: None,
        site: Site::None,
        message: "an out-of-line module needs its source file",
    },
    Rule {
        code: Code::Fx0203,
        rustc_codes: &[373, 521, 597, 515],
        wording: Some("lifetime may not live long enough"),
        site: Site::Owned,
        message: "a closure in an owned position cannot capture a non-static borrow; move owned data in or name its lifetime",
    },
    Rule {
        code: Code::Fx0210,
        rustc_codes: &[505, 515, 597],
        wording: None,
        site: Site::SelfBorrow,
        message: "this struct literal borrows from an owner it moves into a field; keep the owner and view apart or store indices",
    },
    Rule {
        code: Code::Fx0402,
        rustc_codes: &[119],
        wording: None,
        site: Site::None,
        message: "overlapping impls are not admitted; use one impl or a newtype",
    },
];

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
        if matches!(diag.level(), Level::Error | Level::Fatal) && diag.span.primary_span().is_some()
        {
            let rustc_code = diag.code.map(|code| code.as_u32());
            let sites = self.sites.0.lock().unwrap();
            for rule in RULES {
                let wording = rustc_code.is_none()
                    && rule.wording.is_some_and(|prefix| {
                        diag.messages.iter().any(|(message, _)| {
                            format_diag_message(message, &diag.args).starts_with(prefix)
                        })
                    });
                if !wording && !rustc_code.is_some_and(|code| rule.rustc_codes.contains(&code)) {
                    continue;
                }
                let matches = match rule.site {
                    Site::None => true,
                    Site::SelfBorrow => sites
                        .self_borrows
                        .iter()
                        .any(|site| site.matches(&diag, rustc_code)),
                    Site::Owned => sites
                        .owned
                        .iter()
                        .any(|site| site.matches(&diag, rustc_code, wording)),
                };
                if matches {
                    labels.push(rule);
                }
            }
        }
        let span = diag.span.primary_span();
        self.inner.emit_diagnostic(diag);
        for rule in labels {
            let mut label = DiagInner::new(
                Level::Error,
                format!("{}: {}", rule.code.as_str(), rule.message),
            );
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
