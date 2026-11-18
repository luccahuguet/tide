Model: gpt-6.1-sol (high) · Date: 2026-11-18 · Candidate e9c66af
Verdict: request_changes

1. **Blocking — the witness exposes private fields of foreign types.**
   `crates/tide-admit/src/pure.rs:495` records every non-std ADT field's name,
   including foreign private fields; `crates/tide-admit/src/pure.rs:353` then
   prints the owner with `def_path_str` and appends that name without any
   accessibility check. For example, a paired library can export
   `pub struct Vault { secret_net: std::net::Net }`; a separate test crate's
   `fn check(v: &tide_candidate::Vault) { need(v) }`, with
   `fn need<T: Pure>(_: &T) {}`, gains a suffix naming `Vault.secret_net`,
   although that field is private to the library. This is a current supported
   boundary: `walk.rs:343` admits the paired library's definitions, and
   `tests/build.rs:735` already exercises Pure obligations across it.
   Trimmed paths shorten names; they do not redact the separately appended
   `field.name`. A private field containing a tuple or private helper ADT can
   also expose its internal types and descendants through later steps.

   Minimum fix: stop the rendered witness at an inaccessible foreign edge,
   suppressing that edge and all descendant steps (including tuple-type
   displays), while keeping the existing structural decision, terminal reason,
   and diagnostic code/location unchanged. Conservatively stopping witness
   rendering at foreign ADT boundaries also suffices. Do not skip those fields
   in `walk`, since that would change acceptance policy. Add one cross-crate
   regression with a private field and nested internal structure, checking that
   rejection remains and the suffix reveals neither private names nor internal
   structure. The current local-only opaque fixture does not exercise this case.

Confidence: high from source inspection; the cross-crate example was not run.
No override recorded.

Review evidence:

- Reviewed `git diff main...e9c66af` on HEAD
  `e9c66af7013debb3748fcaae05da6447fd8fcb57`, initially clean working tree.
  Diff SHA-256: `a6e1993dd994176b21b590abe6b0c6070faabf4262ad9cbc093ddec6e75d2430`.
  Read the bead, AGENTS.md, GOVERNANCE.md, builder report, affected code,
  message callers, relevant Pure/opacity design text, and pinned rustc source.
- Policy: the pinned rustc `AdtDef::all_fields` implementation is exactly
  `self.variants().iter().flat_map(|v| v.fields.iter())`. The replacement
  preserves this order and `try_for_each` early exit. Tuple enumeration also
  preserves order. `assumed.insert`, its lifetime, normalization, trusted/std
  handling, and the depth refusal are unchanged. The hash set is queried, not
  iterated, so it introduces no witness-order instability. No verdict change
  found.
- Paths: recorded steps come from the actual failing recursive calls and are
  reversed once for display. Generic field substitution is unchanged; unnamed
  reference/container transitions remain valid. Repeated types still terminate
  coinductively, and failures unwind to finite paths. `Why::TooDeep` returns
  before rendering either the terminal part or the path, preserving explicit
  inability to decide rather than claiming an authority witness.
- Opaque aliases still stop at their bounds in the non-revealing environment.
  The pinned rustc type printer displays opaque bounds, not the hidden concrete
  type. The added `ty::tls::with` retrieves the same compiler context already
  needed by the existing `Ty` Display implementation; it does not normalize or
  reveal an opaque type. This does not protect foreign field names (finding 1).
- Message consumers: searched `conformance/`, `design/`, `docs/`, and driver
  tests for `is not `Pure``, `it reaches`, `never `Pure``,
  `does not state `Pure``, and depth wording. No conformance or design text
  pins a changed full message. The existing `W<Clock>` assertion is updated.
  The owner-guide TD0603 example captures `&Fs` directly, so no new field/tuple
  step changes its quoted diagnostic. Existing `NotPure::message` callers
  require no signature changes; interface extraction still uses only the
  verdict/`too_deep` and is untouched.
- Tests: the new fixture compares the full ordered diagnostic list, including
  codes, locations, owners, field names, enum variant/index, and both tuple
  indexes. Wrong, missing, reordered, or extra steps fail those assertions.
  Its accepted recursive control must produce no Pure diagnostic; its rejected
  recursive control has a finite witness; its opaque control stops at
  `impl Sized`. These checks are strong for the local cases, but do not cover
  foreign privacy.
- Simplicity: one error payload and failure-unwinding steps reuse the verdict
  owner without a second traversal or dependency. Successful paths do not
  allocate witness vectors. No additional actionable leftovers found.
- Builder evidence inspected, not independently rerun: driver log reports
  41 passed; workspace log reports passing suites; conformance log reports
  `389 cases: 386 PASS, 3 AWAITS CHECKER, 0 FAIL`, matching the report.

Not checked: no builds or tests were run; trials/e1-checked/ was never opened.
Risk: foreign private structure is exposed until finding 1 is fixed.
DONE
