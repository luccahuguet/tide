//! Tide diagnostics. Each is a rustc error whose message starts with its FX code and a colon
//! (`FX0520: ...`), at the span in the user's code that caused it, so rustc's own renderer
//! and `--error-format` apply to it.

use rustc_data_structures::fx::FxHashSet;
use rustc_middle::ty::TyCtxt;
use rustc_span::{BytePos, Span};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Code {
    /// A use outside the admission allowlist (ADMIT-1).
    Fx0520,
}

impl Code {
    /// Every code this driver decides, so a caller can tell its verdicts from ones it awaits.
    pub const ALL: &[Code] = &[Code::Fx0520];

    pub fn as_str(self) -> &'static str {
        match self {
            Code::Fx0520 => "FX0520",
        }
    }
}

struct Diagnostic {
    span: Span,
    code: Code,
    message: String,
    help: Option<&'static str>,
}

#[derive(Default)]
pub struct Diagnostics {
    found: Vec<Diagnostic>,
    /// (code, message, file, line): one diagnostic per item per source line, where an
    /// expression, its type, and its pattern would otherwise each report the same use.
    seen: FxHashSet<(Code, String, BytePos, usize)>,
}

impl Diagnostics {
    pub fn report(
        &mut self,
        tcx: TyCtxt<'_>,
        span: Span,
        code: Code,
        message: String,
        help: Option<&'static str>,
    ) {
        let loc = tcx.sess.source_map().lookup_char_pos(span.lo());
        if self
            .seen
            .insert((code, message.clone(), loc.file.start_pos, loc.line))
        {
            self.found.push(Diagnostic {
                span,
                code,
                message,
                help,
            });
        }
    }

    /// Emits the diagnostics in source order, one per code at each start position: the one with
    /// the narrowest span, which names the item written there (`AtomicU64` in
    /// `AtomicU64::new(0)`, not the call's type).
    pub fn emit(mut self, tcx: TyCtxt<'_>) {
        self.found.sort_by(|a, b| {
            (a.span.lo(), a.code, a.span.hi(), &a.message).cmp(&(
                b.span.lo(),
                b.code,
                b.span.hi(),
                &b.message,
            ))
        });
        self.found
            .dedup_by(|b, a| (a.span.lo(), a.code) == (b.span.lo(), b.code));
        for d in self.found {
            let mut diag = tcx
                .dcx()
                .struct_span_err(d.span, format!("{}: {}", d.code.as_str(), d.message));
            if let Some(help) = d.help {
                diag.help(help);
            }
            diag.emit();
        }
    }
}
