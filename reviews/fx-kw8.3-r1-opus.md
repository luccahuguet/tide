Model: Opus 5.5. Date: 2026-10-22. Round 1, candidate fc8e9adc (uncommitted on ce0f0cc).

# fx-kw8.3 round 1 (Opus): tide-admit owns every rustc invocation

**Verdict: request_changes, medium-high confidence.** The invocation boundary holds against everything I threw at it. A crate cannot change rustc's flags. The environment is cleared. No outside bytes leak; under strace, a macro-generated `#[path]` aimed at an outside canary opened it zero times. The bead's Acceptance is substantively met. Two defects this diff introduced are in scope and cheap to fix, and both are visible to callers.

### Blocking

**B1. The pre-expansion guard emits its own diagnostics: duplicated, in the wrong format, with a fake filename.** `crates/tide-admit/src/invocation.rs:290-318`
- **Cause:** `Sources::guard` parses with `ParseSess::new()`. That uses the default human emitter, ignores `--error-format short`, and names the file `FileName::Custom`, which displays as `<path>`.
- **Trigger:** `pub fn f() { let a = 1 let b = 2; }` plus `pub fn g() -> u8 { 1 + }`.
- **Observed:** each error prints twice. First comes a human-format block with `--> </abs/path/lib.rs>:1:24`, then rustc's own short-format line. Plain pinned rustc prints only the two short lines.
- **Second case:** when the guard then rejects (recoverable syntax error followed by `#[path = ..] mod x;`), the human-format copy is the only syntax diagnostic shown.
- **Scope:** this affects every recoverable syntax error in any candidate. The interface accepts only `--error-format short`, so this breaks the caller contract.
- **Smallest fix:**
  - Call `psess.dcx().make_silent()` on the guard's `ParseSess` (`rustc_errors/src/lib.rs:491`).
  - On a hard parse `Err`, return the source without guarding. rustc's own parse of the same bytes reports it in the session's format. A failed external module becomes an empty dummy module (`rustc_expand/src/module.rs:84-87`), and a failed main file is fatal. Anything that does expand still meets the loader and `check_expanded`.
  - Also use `StripTokens::ShebangAndFrontmatter` to match rustc (`rustc_expand/src/module.rs:76`, `rustc_interface/src/passes.rs:60`). On this pin frontmatter is still feature-gated, so the mismatch only changes the diagnostic shown today. It becomes a false rejection once frontmatter stabilises.

**B2. Candidate content can turn admission into a setup error (exit 2).** `invocation.rs:235` (`fs::canonicalize(&entry)?`) and `:251-252` (`read_to_string` failing)
- **Cause:** one bad entry anywhere under the source directory aborts the whole snapshot, even if nothing references it.
- **Trigger (probed):**
  - a committed dangling symlink `src/dangling.rs -> /nonexistent/x` gives `tide-admit: cannot snapshot candidate sources` and exit 2;
  - an unreferenced non-UTF-8 `src/junk.rs` gives the same, exit 2;
  - plain rustc compiles both crates.
- **Why it matters:**
  - RUN-2 says a symbolic link that leaves the tree "reads as missing".
  - Exit 2 is the driver's setup-error class; `conformance/run.sh:119` treats `dcode > 1` as "the driver crashed".
  - So an unreviewed branch can make the gate report an infrastructure failure instead of a verdict. It still fails closed.
- **Smallest fix:** `let Ok(canonical) = fs::canonicalize(&entry) else { continue };`, and treat a `.rs` file that is not valid UTF-8 as missing. If that file is referenced, rustc then gives E0583 with exit 1. Optionally record it in the key.

### Non-blocking

- **N1. One `macro_rules!` hides every other verdict.** At `invocation.rs:455-464`, the guard's FX0101 rejection is fatal. A crate with a single `macro_rules!` gets only that diagnostic, which contradicts main.rs:8-9 ("a crate with rustc errors still gets its verdicts"). `check_expanded` (`:499-511`) already reports MacroDef without stopping, and the loader confines whatever a user macro's generated `#[path]` reads. Deleting `Guard::visit_item` would let all verdicts run. That confidentiality is unaffected is my inference, from the loader and probe below. The guard's `#[path]`/`debugger_visualizer` rejection must stay fatal, because it runs before the read.
- **N2. False positive on harmless code.** `pub const S: &str = stringify!(#[path = "x"] mod y;);` is rejected FX0911 (probed); `forbidden_macro_attributes` at `:411-437` is token-level. Acceptable as conservative; worth knowing.
- **N3. Denied reads have no location.** `denied()` at `:320-324` prints `error: FX0520: compile-time file read denied` with no file or line, and does not name the requested path. The conformance runner's `dgot` regex cannot match it. It also pre-empts FX0101 in the same crate: probed with a macro-generated `#[path]` to an outside file, the output is only this line.
- **N4. The driver binary is hashed on every run.** At `:613`, `fingerprint(&exe)` SHA-256-hashes the 82 MB debug binary on each full run, which is twice per cache miss (worker and compile child). This is a cost, not a defect. A build-time identity would avoid it.
- **N5. Duplicate validation.** The crate-name check appears twice (`:86-89` and `:115-119`). Validate once after defaulting the name.
- **N6. Removable helpers.** `Sources::canonical` (`:274-276`) is a trivial wrapper whose `&self` is unused; inline `fs::canonicalize(path).ok()`. `hash` (`:156-162`) has a single caller.
- **N7. Stale doc comment.** `crates/tide-admit/build.rs:1-2` still describes only the link-path job. It now also exports `TIDE_SYSROOT` and `TIDE_COMPILER_ID`.
- **N8. Path display.**
  - Diagnostics now always show the normalised absolute path, even for a relative invocation (probed: `./h/pkg/src/lib.rs` is reported as `/abs/.../lib.rs`).
  - `Guard::reject` prints the canonical path while rustc prints the lexical one, so the two differ for in-tree symlinked files.
  - `conformance/run.sh:11` builds `HERE` with a logical `pwd`, so a checkout under a symlinked directory would hit "package path must not traverse a directory symlink" (exit 2).
- **N9. FX0915 build-script detection is a heuristic.** It only looks in the entry's parent's parent when that parent is `src`. Entries like `src/bin/x.rs` or `tests/x.rs` miss `package/build.rs`, and Cargo's `build = "..."` key is never read. README documents the rule, and the driver runs nothing in any case.
- **N10. FX0101 exemption is broader than intended.** `in_external_macro` (`:502`) returns true for every attribute or derive expansion (`rustc_span/src/hygiene.rs:951`). A trusted proc macro that re-spanned candidate tokens at `call_site` could carry a `macro_rules!` past FX0101. This is inference only. The stub's `#[pure]` is an identity macro, and I probed `#[pure] fn` with `macro_rules!` inside `assert!`: correctly rejected with FX0101.

### New-surface disposition

All four are used only by main.rs. `pub` inside the private `invocation` module of a binary is already crate-visible, so `pub(crate)` would change nothing.
- **`Sources` (`:209`): stays.** `Admission` in main.rs holds it and installs it as `config.file_loader`.
- **`attach` (`:278`): stays.** Its only caller is main.rs's `psess_created` closure, and it is the only way to fill the thread-local source map that `foreign_source` needs.
- **`check_expanded` (`:480`): stays.** It is the post-expansion FX0911/FX0101 check (the B1 fix from Argon's preflight) and is called from `after_expansion` before `walk::check_crate` lowers the crate. That ordering is correct: `resolver_for_lowering().borrow()` happens before the steal.
- **`run` (`:523`): stays.** It is main's single entry point and keeps main.rs thin.

### What I checked

- **Context read:** AGENTS.md rules, the full fx-kw8.3 bead and comments, RUN-2/ADMIT-2 (03:560-640) and the RUN-2 conformance row (03:805), the complete diff, and both new files.
- **Pinned rustc source:**
  - `FileLoader` and `ensure_source_file_source_present`, which uses the content-hash check, so the `Ok("")` foreign reply is safe;
  - the `rustc_parse` fallback after a loader error, which is unreachable because `read_file` never returns `Err`;
  - `include*!` and `debugger_visualizer`, which both read through `load_binary_file`, with no fallback;
  - `rustc_expand` module parse-error handling.
- **Probes** (scratch dir `/tmp/claude-1000/-home-lucca-pjs-lab-tide/8a915604-6e15-4fb6-bdec-19f443dee621/scratchpad/tmp.UvjwCcCCeV`, built binary): the B1 and B2 triggers, frontmatter, stringify, a relative entry path, the macro-generated outside `#[path]` under strace (0 canary opens), and `#[pure]` with nested `macro_rules!`.
- **`timeout 1200 cargo test --workspace </dev/null`:** exit 0, 19 tests (2 + 5 + 12).
- **`timeout 1800 conformance/run.sh </dev/null`:** exit 0, 208 cases (180 PASS, 28 AWAITS CHECKER, 0 FAIL).
- **Tree:** unchanged by me (`git status` matches the start of the session).
