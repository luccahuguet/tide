//! Tide diagnostics. Each is a rustc error or warning whose message starts with its FX code
//! and a colon (`FX0520: ...`), at the span in the user's code that caused it, so rustc's own
//! renderer and `--error-format` apply to it.

mod labels;
pub use labels::{OwnedClosure, RustcLabels};

use rustc_data_structures::fx::FxHashSet;
use rustc_middle::ty::TyCtxt;
use rustc_span::{BytePos, Span};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Code {
    /// User macro definitions (D6).
    Fx0101,
    /// A derive outside the closed set (D6).
    Fx0102,
    /// A bang macro outside the fixed built-in forms (D6).
    Fx0103,
    /// Lifetime parameters on a trait (LIFE-3).
    Fx0201,
    /// An unresolved elided output lifetime (LIFE-4).
    Fx0202,
    /// A borrowing closure in an owned position (BORROW-3, SCOPE-3).
    Fx0203,
    /// A literal borrows an owner it moves into another field (reference 3.6).
    Fx0210,
    /// A lossy `as` cast (D3).
    Fx0301,
    /// Impl placement (D4).
    Fx0401,
    /// Specialization or overlapping impls (D4).
    Fx0402,
    /// Generic associated types (D4).
    Fx0403,
    /// Explicit higher-ranked binders (D4).
    Fx0404,
    /// Negative impls or user auto traits (D4).
    Fx0405,
    /// An impl bound outside the structural subterm ceiling (D4).
    Fx0406,
    /// Polymorphic recursion (D3).
    Fx0407,
    /// Duplicate inherent method names (D3).
    Fx0408,
    /// An ambient world operation (CAP-1).
    Fx0501,
    /// A non-`Pure` `static`, a `static mut`, a lazy static, or a thread-local (GLOBAL-1 to 3).
    Fx0502,
    /// Naming or using the whole Root outside the entry parameter (ROOT-1, ROOT-2).
    Fx0503,
    /// Simulation outside test code (ROOT-3).
    Fx0504,
    /// A handle constructor missing its final &Tasks (CAP-2).
    Fx0505,
    /// A call of, or other reference to, the binary's `main` (ROOT-5).
    Fx0506,
    /// `dbg!` in a release build (SINK-8).
    Fx0507,
    /// Printing to stdout from a library (SINK-8).
    Fx0508,
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
    /// Async syntax, Future, or Pin (D8).
    Fx0601,
    /// Unstructured spawning (D8).
    Fx0602,
    /// A `Shared::with` closure that is not `Pure` (LOCK-4).
    Fx0603,
    /// Mutex and RwLock habits (D8).
    Fx0604,
    /// A select! macro (D8).
    Fx0605,
    /// A `Shared` payload that is not `Pure` (LOCK-1).
    Fx0606,
    /// Printing inside a `Shared::with` closure (LOCK-4).
    Fx0607,
    /// An unbounded channel in a library (reference 7.3).
    Fx0608,
    /// A parallel closure that is not `Pure`, `Send`, and, when shared, `Fn` and `Sync`
    /// (PAR-5, PAR-6).
    Fx0610,
    /// Shared data a parallel closure captures that is not `Pure + Sync` (PAR-7).
    Fx0611,
    /// Elements of a parallel slice method that are not `Pure` (PAR-4).
    Fx0612,
    /// Catching an unwind instead of isolating a task (D5).
    Fx0701,
    /// A Result ignored in statement position (D5).
    Fx0702,
    /// Contracts on trait methods.
    Fx0802,
    /// A prophetic mutable parameter in a postcondition.
    Fx0803,
    /// Clause syntax, contextual result, loop placement, and bounds (toolchain macros).
    Fx0804,
    Fx0805,
    Fx0806,
    Fx0807,
    /// Unsafe code, an unsafe attribute, or `#[trusted_pure]` without the `unsafe` grant
    /// (UNSAFE-1).
    Fx0901,
    /// An unsafe block without a SAFETY comment.
    Fx0902,
    /// A module without a source file.
    Fx0910,
    /// File-loading attributes (RUN-2).
    Fx0911,
    /// Explicit extern crate items.
    Fx0912,
    /// Unused imports.
    Fx0913,
    /// Cyclic module imports.
    Fx0914,
    /// Package build scripts (D13).
    Fx0915,
}

impl Code {
    /// Every code this driver reports (its own verdicts, its labels on rustc's,
    /// and toolchain-macro errors it passes through).
    pub const ALL: &[Code] = &[
        Code::Fx0101,
        Code::Fx0102,
        Code::Fx0103,
        Code::Fx0201,
        Code::Fx0202,
        Code::Fx0203,
        Code::Fx0210,
        Code::Fx0301,
        Code::Fx0401,
        Code::Fx0402,
        Code::Fx0403,
        Code::Fx0404,
        Code::Fx0405,
        Code::Fx0406,
        Code::Fx0407,
        Code::Fx0408,
        Code::Fx0501,
        Code::Fx0502,
        Code::Fx0503,
        Code::Fx0504,
        Code::Fx0505,
        Code::Fx0506,
        Code::Fx0507,
        Code::Fx0508,
        Code::Fx0511,
        Code::Fx0512,
        Code::Fx0517,
        Code::Fx0518,
        Code::Fx0520,
        Code::Fx0601,
        Code::Fx0602,
        Code::Fx0603,
        Code::Fx0604,
        Code::Fx0605,
        Code::Fx0606,
        Code::Fx0607,
        Code::Fx0608,
        Code::Fx0610,
        Code::Fx0611,
        Code::Fx0612,
        Code::Fx0701,
        Code::Fx0702,
        Code::Fx0802,
        Code::Fx0803,
        Code::Fx0804,
        Code::Fx0805,
        Code::Fx0806,
        Code::Fx0807,
        Code::Fx0901,
        Code::Fx0902,
        Code::Fx0910,
        Code::Fx0911,
        Code::Fx0912,
        Code::Fx0913,
        Code::Fx0914,
        Code::Fx0915,
    ];

    pub fn as_str(self) -> &'static str {
        match self {
            Code::Fx0101 => "FX0101",
            Code::Fx0102 => "FX0102",
            Code::Fx0103 => "FX0103",
            Code::Fx0201 => "FX0201",
            Code::Fx0202 => "FX0202",
            Code::Fx0203 => "FX0203",
            Code::Fx0210 => "FX0210",
            Code::Fx0301 => "FX0301",
            Code::Fx0401 => "FX0401",
            Code::Fx0402 => "FX0402",
            Code::Fx0403 => "FX0403",
            Code::Fx0404 => "FX0404",
            Code::Fx0405 => "FX0405",
            Code::Fx0406 => "FX0406",
            Code::Fx0407 => "FX0407",
            Code::Fx0408 => "FX0408",
            Code::Fx0501 => "FX0501",
            Code::Fx0502 => "FX0502",
            Code::Fx0503 => "FX0503",
            Code::Fx0504 => "FX0504",
            Code::Fx0505 => "FX0505",
            Code::Fx0506 => "FX0506",
            Code::Fx0507 => "FX0507",
            Code::Fx0508 => "FX0508",
            Code::Fx0511 => "FX0511",
            Code::Fx0512 => "FX0512",
            Code::Fx0517 => "FX0517",
            Code::Fx0518 => "FX0518",
            Code::Fx0520 => "FX0520",
            Code::Fx0601 => "FX0601",
            Code::Fx0602 => "FX0602",
            Code::Fx0603 => "FX0603",
            Code::Fx0604 => "FX0604",
            Code::Fx0605 => "FX0605",
            Code::Fx0606 => "FX0606",
            Code::Fx0607 => "FX0607",
            Code::Fx0608 => "FX0608",
            Code::Fx0610 => "FX0610",
            Code::Fx0611 => "FX0611",
            Code::Fx0612 => "FX0612",
            Code::Fx0701 => "FX0701",
            Code::Fx0702 => "FX0702",
            Code::Fx0802 => "FX0802",
            Code::Fx0803 => "FX0803",
            Code::Fx0804 => "FX0804",
            Code::Fx0805 => "FX0805",
            Code::Fx0806 => "FX0806",
            Code::Fx0807 => "FX0807",
            Code::Fx0901 => "FX0901",
            Code::Fx0902 => "FX0902",
            Code::Fx0910 => "FX0910",
            Code::Fx0911 => "FX0911",
            Code::Fx0912 => "FX0912",
            Code::Fx0913 => "FX0913",
            Code::Fx0914 => "FX0914",
            Code::Fx0915 => "FX0915",
        }
    }

    fn is_lint(self) -> bool {
        matches!(self, Code::Fx0508 | Code::Fx0607 | Code::Fx0608)
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

    pub fn has_error_in(&self, span: Span) -> bool {
        self.found
            .iter()
            .any(|d| !d.code.is_lint() && span.contains(d.span))
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
            if d.code.is_lint() {
                let mut diag = tcx
                    .dcx()
                    .struct_span_warn(d.span, format!("{}: {}", d.code.as_str(), d.message));
                if let Some(help) = d.help {
                    diag.help(help);
                }
                diag.emit();
                continue;
            }
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
