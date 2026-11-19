//! Tide diagnostics. Each is a rustc error or warning whose message starts with its TD code
//! and a colon (`TD0520: ...`), at the span in the user's code that caused it, so rustc's own
//! renderer and `--error-format` apply to it.

mod labels;
pub use labels::{OwnedClosure, RustcLabels, TideScope};

use rustc_data_structures::fx::FxHashSet;
use rustc_middle::ty::TyCtxt;
use rustc_span::{BytePos, Span};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Code {
    /// User macro definitions (D6).
    Td0101,
    /// A derive outside the closed set (D6).
    Td0102,
    /// A bang macro outside the fixed built-in forms (D6).
    Td0103,
    /// Lifetime parameters on a trait (LIFE-3).
    Td0201,
    /// An unresolved elided output lifetime (LIFE-4).
    Td0202,
    /// A borrowing closure in an owned position (BORROW-3, SCOPE-3).
    Td0203,
    /// A fn item coerced to a `fn` pointer although its signature implies outlives bounds
    /// (CLOSURE-3).
    Td0206,
    /// A literal borrows an owner it moves into another field (reference 3.6).
    Td0210,
    /// A lossy `as` cast (D3).
    Td0301,
    /// Impl placement (D4).
    Td0401,
    /// Specialization or overlapping impls (D4).
    Td0402,
    /// Generic associated types (D4).
    Td0403,
    /// Explicit higher-ranked binders (D4).
    Td0404,
    /// Negative impls or user auto traits (D4).
    Td0405,
    /// An impl bound outside the structural subterm ceiling (D4).
    Td0406,
    /// Polymorphic recursion (D3).
    Td0407,
    /// Duplicate inherent method names (D3).
    Td0408,
    /// An ambient world operation (CAP-1).
    Td0501,
    /// A non-`Pure` `static`, a `static mut`, a lazy static, or a thread-local (GLOBAL-1 to 3).
    Td0502,
    /// Naming or using the whole Root outside the entry parameter (ROOT-1, ROOT-2).
    Td0503,
    /// Simulation outside test code (ROOT-3).
    Td0504,
    /// A handle constructor missing its final &Tasks (CAP-2).
    Td0505,
    /// A call of, or other reference to, the binary's `main` (ROOT-5).
    Td0506,
    /// `dbg!` in a release build (SINK-8).
    Td0507,
    /// Printing to stdout from a library (SINK-8).
    Td0508,
    /// A `#[pure]` function whose parameter types are not all `Pure` (AFREE-1).
    Td0511,
    /// A binary's `main` without `#[tide::main]` (ROOT-1).
    Td0512,
    /// A failed `Pure` obligation that no rule with a code of its own needs (PURE-1), or an
    /// `io::Error` payload that does not reach one of alloc's four conversions (ADMIT-1).
    Td0517,
    /// A `const fn` parameter that is `&mut` or not `Pure + Sync` (AFREE-2).
    Td0518,
    /// A use outside the admission allowlist (ADMIT-1).
    Td0520,
    /// Async syntax, Future, or Pin (D8).
    Td0601,
    /// Unstructured spawning (D8).
    Td0602,
    /// A `Shared::with` closure that is not `Pure` (LOCK-4).
    Td0603,
    /// Mutex and RwLock habits (D8).
    Td0604,
    /// A select! macro (D8).
    Td0605,
    /// A `Shared` payload that is not `Pure` (LOCK-1).
    Td0606,
    /// Printing inside a `Shared::with` closure (LOCK-4).
    Td0607,
    /// An unbounded channel in a library (reference 7.3).
    Td0608,
    /// A parallel closure that is not `Pure`, `Send`, and, when shared, `Fn` and `Sync`
    /// (PAR-5, PAR-6).
    Td0610,
    /// Shared data a parallel closure captures that is not `Pure + Sync` (PAR-7).
    Td0611,
    /// Elements of a parallel slice method that are not `Pure` (PAR-4).
    Td0612,
    /// Catching an unwind instead of isolating a task (D5).
    Td0701,
    /// A Result ignored in statement position (D5).
    Td0702,
    /// Contracts on trait methods.
    Td0802,
    /// A prophetic mutable parameter in a postcondition.
    Td0803,
    /// Clause syntax, contextual result, loop placement, and bounds (toolchain macros).
    Td0804,
    Td0805,
    Td0806,
    Td0807,
    /// Unsafe code, an unsafe attribute, or `#[trusted_pure]` without the `unsafe` grant
    /// (UNSAFE-1).
    Td0901,
    /// An unsafe block without a SAFETY comment.
    Td0902,
    /// A module without a source file.
    Td0910,
    /// File-loading attributes (RUN-2).
    Td0911,
    /// Explicit extern crate items.
    Td0912,
    /// Unused imports.
    Td0913,
    /// Cyclic module imports.
    Td0914,
    /// Package build scripts (D13).
    Td0915,
}

impl Code {
    /// Every code this driver reports (its own verdicts, its labels on rustc's,
    /// and toolchain-macro errors it passes through).
    pub const ALL: &[Code] = &[
        Code::Td0101,
        Code::Td0102,
        Code::Td0103,
        Code::Td0201,
        Code::Td0202,
        Code::Td0203,
        Code::Td0206,
        Code::Td0210,
        Code::Td0301,
        Code::Td0401,
        Code::Td0402,
        Code::Td0403,
        Code::Td0404,
        Code::Td0405,
        Code::Td0406,
        Code::Td0407,
        Code::Td0408,
        Code::Td0501,
        Code::Td0502,
        Code::Td0503,
        Code::Td0504,
        Code::Td0505,
        Code::Td0506,
        Code::Td0507,
        Code::Td0508,
        Code::Td0511,
        Code::Td0512,
        Code::Td0517,
        Code::Td0518,
        Code::Td0520,
        Code::Td0601,
        Code::Td0602,
        Code::Td0603,
        Code::Td0604,
        Code::Td0605,
        Code::Td0606,
        Code::Td0607,
        Code::Td0608,
        Code::Td0610,
        Code::Td0611,
        Code::Td0612,
        Code::Td0701,
        Code::Td0702,
        Code::Td0802,
        Code::Td0803,
        Code::Td0804,
        Code::Td0805,
        Code::Td0806,
        Code::Td0807,
        Code::Td0901,
        Code::Td0902,
        Code::Td0910,
        Code::Td0911,
        Code::Td0912,
        Code::Td0913,
        Code::Td0914,
        Code::Td0915,
    ];

    pub fn as_str(self) -> &'static str {
        match self {
            Code::Td0101 => "TD0101",
            Code::Td0102 => "TD0102",
            Code::Td0103 => "TD0103",
            Code::Td0201 => "TD0201",
            Code::Td0202 => "TD0202",
            Code::Td0203 => "TD0203",
            Code::Td0206 => "TD0206",
            Code::Td0210 => "TD0210",
            Code::Td0301 => "TD0301",
            Code::Td0401 => "TD0401",
            Code::Td0402 => "TD0402",
            Code::Td0403 => "TD0403",
            Code::Td0404 => "TD0404",
            Code::Td0405 => "TD0405",
            Code::Td0406 => "TD0406",
            Code::Td0407 => "TD0407",
            Code::Td0408 => "TD0408",
            Code::Td0501 => "TD0501",
            Code::Td0502 => "TD0502",
            Code::Td0503 => "TD0503",
            Code::Td0504 => "TD0504",
            Code::Td0505 => "TD0505",
            Code::Td0506 => "TD0506",
            Code::Td0507 => "TD0507",
            Code::Td0508 => "TD0508",
            Code::Td0511 => "TD0511",
            Code::Td0512 => "TD0512",
            Code::Td0517 => "TD0517",
            Code::Td0518 => "TD0518",
            Code::Td0520 => "TD0520",
            Code::Td0601 => "TD0601",
            Code::Td0602 => "TD0602",
            Code::Td0603 => "TD0603",
            Code::Td0604 => "TD0604",
            Code::Td0605 => "TD0605",
            Code::Td0606 => "TD0606",
            Code::Td0607 => "TD0607",
            Code::Td0608 => "TD0608",
            Code::Td0610 => "TD0610",
            Code::Td0611 => "TD0611",
            Code::Td0612 => "TD0612",
            Code::Td0701 => "TD0701",
            Code::Td0702 => "TD0702",
            Code::Td0802 => "TD0802",
            Code::Td0803 => "TD0803",
            Code::Td0804 => "TD0804",
            Code::Td0805 => "TD0805",
            Code::Td0806 => "TD0806",
            Code::Td0807 => "TD0807",
            Code::Td0901 => "TD0901",
            Code::Td0902 => "TD0902",
            Code::Td0910 => "TD0910",
            Code::Td0911 => "TD0911",
            Code::Td0912 => "TD0912",
            Code::Td0913 => "TD0913",
            Code::Td0914 => "TD0914",
            Code::Td0915 => "TD0915",
        }
    }

    fn is_lint(self) -> bool {
        matches!(self, Code::Td0508 | Code::Td0607 | Code::Td0608)
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
