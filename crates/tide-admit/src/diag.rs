//! Tide diagnostics. Each is a rustc error whose message starts with its FX code and a colon
//! (`FX0520: ...`), at the span in the user's code that caused it, so rustc's own renderer
//! and `--error-format` apply to it.

use rustc_data_structures::fx::FxHashSet;
use rustc_middle::ty::TyCtxt;
use rustc_span::{BytePos, Span};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Code {
    /// User macro definitions (D6).
    Fx0101,
    /// A non-`Pure` `static`, a `static mut`, a lazy static, or a thread-local (GLOBAL-1 to 3).
    Fx0502,
    /// A `#[pure]` function whose parameter types are not all `Pure` (AFREE-1).
    Fx0511,
    /// A binary's `main` without `#[tide::main]` (ROOT-1).
    Fx0512,
    /// A failed `Pure` obligation that no rule with a code of its own needs (PURE-1), or an
    /// `io::Error` payload that does not reach one of alloc's four conversions (ADMIT-1).
    Fx0517,
    /// A `const fn` parameter that is `&mut` or not `Pure + Sync` (AFREE-2).
    Fx0518,
    /// A use outside the admission allowlist (ADMIT-1).
    Fx0520,
    /// A `Shared::with` closure that is not `Pure` (LOCK-4).
    Fx0603,
    /// A `Shared` payload that is not `Pure` (LOCK-1).
    Fx0606,
    /// A parallel closure that is not `Pure`, `Send`, and, when shared, `Fn` and `Sync`
    /// (PAR-5, PAR-6).
    Fx0610,
    /// Shared data a parallel closure captures that is not `Pure + Sync` (PAR-7).
    Fx0611,
    /// Elements of a parallel slice method that are not `Pure` (PAR-4).
    Fx0612,
    /// Unsafe code, an unsafe attribute, or `#[trusted_pure]` without the `unsafe` grant
    /// (UNSAFE-1).
    Fx0901,
    /// File-loading attributes (RUN-2).
    Fx0911,
    /// Package build scripts (D13).
    Fx0915,
}

impl Code {
    /// Every code this driver decides, so a caller can tell its verdicts from ones it awaits.
    pub const ALL: &[Code] = &[
        Code::Fx0101,
        Code::Fx0502,
        Code::Fx0511,
        Code::Fx0512,
        Code::Fx0517,
        Code::Fx0518,
        Code::Fx0520,
        Code::Fx0603,
        Code::Fx0606,
        Code::Fx0610,
        Code::Fx0611,
        Code::Fx0612,
        Code::Fx0901,
        Code::Fx0911,
        Code::Fx0915,
    ];

    pub fn as_str(self) -> &'static str {
        match self {
            Code::Fx0101 => "FX0101",
            Code::Fx0502 => "FX0502",
            Code::Fx0511 => "FX0511",
            Code::Fx0512 => "FX0512",
            Code::Fx0517 => "FX0517",
            Code::Fx0518 => "FX0518",
            Code::Fx0520 => "FX0520",
            Code::Fx0603 => "FX0603",
            Code::Fx0606 => "FX0606",
            Code::Fx0610 => "FX0610",
            Code::Fx0611 => "FX0611",
            Code::Fx0612 => "FX0612",
            Code::Fx0901 => "FX0901",
            Code::Fx0911 => "FX0911",
            Code::Fx0915 => "FX0915",
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

    pub fn is_empty(&self) -> bool {
        self.found.is_empty()
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
