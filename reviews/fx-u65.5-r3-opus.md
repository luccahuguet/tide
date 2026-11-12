Model: claude-opus-5-5 · Date: 2026-11-12

Verdict: approve_with_alternatives

Reviewed frozen candidate `fa660dc` (one commit on `42dd813`), `git diff 42dd813..fa660dc`, with the
implementer's report (including Fix round 1), Codex r1/r2, `br --no-db show fx-u65.5` / `fx-u65`
(2026-11-11 decisions, fx-u65.2 round-5 `select!` note, held-out rule), research/27 §1.4, C9, C10,
§5 item 2, reference §7 and §9, and the touched files with their callers. No finding blocks: none
needs a v0-core change, and none of the in-diff points changes a verdict.

## What holds

- **Severity.** All six codes are registered in `diag.rs`. None is on the lint list. Each is
  reported as `error`, and FX errors ignore `#[allow]` (`ignored_result_reject.rs` sets
  `#![allow(unused_must_use)]`).
- **FX0601.**
  - The AST check (`walk/habits.rs:8-62`) covers `async fn`, including trait methods, `async` blocks,
    `async` closures, `async move` closures and `.await`. Tokens inside a std macro (`vec![async {}]`)
    are still flagged.
  - `core::future::*` and `core::pin::*` are FX0601 (`policy.rs:216-226`); `core::task::*` and
    `alloc::task::*` stay FX0520.
  - Unresolved `std::future::ready`, `std::pin::Pin`, `use std::future::{IntoFuture, Ready}`,
    `use std::future::Future as F`, and `Box::pin` / `Rc::pin` (through the `Pin` return type) all
    give FX0601 (probes). Finding 1 covers the bound/impl-header gap.
  - The desugaring skip in `walk.rs:426-432` cannot open a hole, because any span it skips is
    inside written async, which is already rejected.
- **Shared impl owner** (`walk.rs:325-334`). Its codes are now FX0520 (`Role::Other`, unlisted,
  ungranted), FX0601 (`Future`/`IntoFuture` traits, `Pin`/`Ready` heads), FX0602 (`Builder`/`Scope`
  heads), and FX0604 (`Mutex`/`RwLock`/guard heads).
  - FX0501 is unreachable: every `ambient(..)` pattern names a function or method, never a trait or
    ADT.
  - FX0701 is unreachable, since `catch_unwind` is a function.
  - `habit_def` matches only items named `spawn`, which are never traits or types.
  - `Admitted::Granted` handling is unchanged: both sides OK still gives `Free`.
  - `admit_impl`'s only caller is `obligation` (`walk.rs:1349`).
- **FX0702 (C10, choice A).**
  - Rejected (probes):
    - `r;`, `Ok::<(), Infallible>(());`, `writeln!(..);`, `{ Ok(..) };`, `if .. { r } else { .. };`;
    - generic `Result<T, _>`, trait-method results, `io::Result` aliases, Tide `tasks.checkpoint();`;
    - in closures, in nested blocks inside `vec![..]`, and on the inner `Result` from `r?` when `r`
      is `Result<Result<_, _>, _>`.
  - Accepted: `let _ = r;`, `r?;`, `r.ok();`, `r.unwrap_or(0);`, `(r,);`, an `Option`, an unused
    iterator adapter, `assert!` / `println!`, a local `#[must_use]` struct and a local `Result` type.
  - The macro-span change (`habits.rs:105-123`) has no knock-on effects I could find:
    - Tide's `print!`/`println!`/log sinks return `()` (`tide-std/src/root.rs:119-142`);
    - the contract helpers return `()`;
    - `#[tide::main]` and `#[test]` output counts as toolchain-generated;
    - derive output carries foreign macro spans;
    - no Rust std form used in statement position expands to a `Result` statement in the caller's
      block that the caller didn't write.
  - Including `StmtKind::Expr` is harmless, because a non-tail `Result` there is already a rustc E0308.
  - The existing corpus and the reference's examples hold no bare ignored `Result` (grep).
- **FX0602/FX0604/FX0701** use the pinned defining paths.
  - Method and path forms are covered: `Builder::new().spawn`, `scope(|s| s.spawn ..)`,
    `m.lock()`, `MutexGuard`, `catch_unwind(AssertUnwindSafe(..))`.
  - `Condvar` stays FX0520 (§5 item 2), and `std::sync::raw::*` keeps its grant rule
    (`cap_sync_raw*.rs` are unaffected: the stub's raw `Mutex` is a Tide-std type).
- **FX0605.**
  - Caught: unresolved `select!`, `tokio::select!`, `crossbeam::select!`, a resolved local
    `macro_rules! select`, and a macro renamed to `select`.
  - FX0103's carve-out (`surface.rs:73-87`) defers `select`, so FX0605 is the only code.
  - FX0605 short-circuits the macro-admission check (`walk.rs:1424`), so nothing is double-coded.
- **Ownership.**
  - `Rule.code` and `deny_as` live in `policy.rs`. The `habit_path` matcher is gone, and
    `walk/authority.rs::source_path` stays the single unresolved-path recovery owner.
  - The `origin` skip (`authority.rs:489-491`) only excludes the use's own binding, so the
    ambiguity and cycle handling (`seen`) are intact.
  - Codes the driver already reported stay with their owners (FX0103, FX0501, FX0520, FX0502).
- **Acceptance.** Each code has a violating and an accepted case. Every `//@ rules:` ID (REF-0601,
  REF-0602, REF-0604, REF-0605, REF-0701, REF-0702) exists in `conformance/rules.md`. `run.sh` reads
  `Code::ALL`, so these marks are compared, not awaited. I did not rerun the corpus.
- **fx-yg5.5.** It conflicts only textually.
  - Its rows (`core::io::cursor::Cursor`, `core::io::seek::SeekFrom`, `core::ascii::*`,
    `std::macros::panic`) share no prefix with this change's patterns, so first-match order is
    unaffected.
  - Its `reference_allowlist` witnesses include `Unpin`, which is `core::marker`, not `core::pin`.
  - Its snippets consume every `Result` (`write!(..).unwrap()`), so FX0702 will not trip them after
    the rebase.
  - Its test rows go near the tops of `build.rs`/`driver.rs`, while this change appends at the ends.

## Findings

1. **Non-blocking, older than this change, out of scope: an unresolved trait in a bound or
   trait-ref position suppresses every walk diagnostic in the file, so the most common Rust
   spelling of `Future` gets no FX0601.**
   - Evidence (probes 2–4):
     - `impl std::future::Future for S {..}`, `fn x<F: std::future::Future>(_: F)`,
       `&dyn std::future::Future<..>`, `impl std::future::Future<..>` in argument position, and
       `use std::future::Future; fn x(_: impl Future<..>)` all print only E0432/E0433, with no FX
       line.
     - The same files lose unrelated FX codes too. In p02, `std::future::ready(1)` on another line,
       which alone gives FX0601 (q1), is silent.
     - The cause is older than this change. `pub fn a() { let _ = std::fs::read("x"); }` plus
       `pub fn b<F: Missing>(_: F) {}` loses FX0501 (s1, s3), while an unresolved *type*
       (`fn b(_: Missing)`, s4) keeps it.
     - The AST check survives (s2 keeps FX0601 for `async fn`), so the loss is in the HIR walk run
       by `check_crate` (`invocation.rs:84`).
   - rustc still rejects every such file, so nothing is admitted. But any E1 Rust-writer file with
     one unresolved trait gets no FX labels at all.
   - This change's acceptance holds: each code's case gets its code.
   - Fix: not in this change. Record it on the fx-u65 epic, or as a bead the coordinator schedules,
     so the walk (or a fallback over unresolved trait paths) still reports when rustc fails trait
     resolution. The falsifier is s1: FX0501 must appear beside E0433.

2. **Non-blocking: Tokio's FX0602 code and help bypass `Rule.code` in two places.**
   - Evidence: `walk/habits.rs:65-78` (`habit_def` builds `Rejection { code: Fx0602, help: .. }`)
     and `walk/authority.rs:366-370` (`Source::TokioSpawn` reports a hard-coded FX0602 message).
   - The help text "spawn into `tasks.scope` instead" appears three times (`policy.rs:189`, `:194`,
     `habits.rs:77`).
   - Under the 2026-11-11 decision ("Uses Rule.code") and AP-OWNERSHIP-001, policy should own the
     code and help.
   - The repository has a precedent: `ambient("thread_rng")` (`policy.rs:154`) is a pseudo-key that
     the walk sends through `denied_key`.
   - Fix (cheap, same files): add `deny_as("tokio::spawn", Code::Fx0602, "spawn into `tasks.scope`
     instead")`, report `Source::TokioSpawn` through `self.denied_key("tokio::spawn".into(), span)`,
     and have `habit_def` look up the same rule. Optionally drop the "(Tide's std does not export
     it)" note for that key.
   - Behavior stays the same, so the author may take this now or rebut it.

3. **Nit: the C10 falsifier's `r.ok();` is not in the corpus.**
   - Evidence: `conformance/cases/errors/ignored_result_accept.rs`. Research/27 §4's C10 row lists
     `r.ok();` as accepted. Probe p07 confirms it is.
   - Fix: add `pub fn ok(result: Result<u8, ()>) { result.ok(); }` to the accept case.

4. **Nit: neighbouring spellings get other codes or only rustc errors.**
   - Unresolved `tokio::task::spawn` / `tokio::spawn_blocking` get only E0433, because
     `imported_member(Source::Tokio, ..)` knows only `spawn` (`authority.rs:428-430`).
   - `futures::select_biased!` gets FX0103.
   - `std::pin::pin!(x)` gets FX0103 + E0433.
   - The reference names only `tokio::spawn`, `select!` and `Pin`, every one of these is still
     rejected, and the README text matches the behavior. No fix needed. Mention them only if the
     reference grows.

5. **Nit: unrelated whitespace churn.** `policy.rs:340` re-aligns the `std::ffi::os_str::*` comment
   (rustfmt). It is harmless, and fx-yg5.5 doesn't touch that line.

## Probes

All probes ran serially through the machine queue against the candidate's prebuilt driver
`target/debug/tide-admit`. It was built 20:51Z, after the last source edit (no file under
`crates/tide-admit/src`, `tide-std` or `tide-macros` is newer). It used production std
`target/tmp/build/std/artifacts/libtide_std.rlib`. The command was:

`(ulimit -c 0; CARGO_BUILD_JOBS=4 timeout 1500 bash /home/lucca/.cache/eon-agents/eon-overseer/heavy.sh nice -n 19 bash <scratchpad>/probe.sh <dir> </dev/null)`

`probe.sh` copies each file to `pkg-<name>/package/src/lib.rs` and runs
`tide-admit --edition=2024 --crate-type=lib --emit=metadata --error-format=short --tide-std <artifacts>/libtide_std.rlib --tide-dep-dir <artifacts> --out-dir <pkg>/out <lib.rs>`
with stdin closed and a 30 s timeout per file. All runs finished by 21:36Z.

- Two early submissions are not results. The first wrapper ran with the source outside a
  `package/src` layout, and every file exited 2 (setup error). A one-file retry timed out after
  120 s while still queued.
- **Batch 1** (21:23Z, 9 files; `L<line> <severity>[E-code] <FX code>`):
  - Box/Rc pin: L1 FX0601, L2 E0433 + FX0103 (`pin!`), L3 FX0601.
  - Future impl via `std::` paths: only E0433/E0425/E0603, with no FX on any line (finding 1).
  - Mutex param and `.lock()`: FX0604 (+FX0520, E0603). `Condvar`: FX0520.
  - Tokio and select: `tokio::task::spawn` / `spawn_blocking` E0433 only; `futures::select_biased!`
    FX0103; `crossbeam::select!` FX0605.
  - Thread: `Builder::new().spawn` and `scope(|s| s.spawn)` both FX0602.
  - Results reject: FX0702 on 9 of 10 statements (lines 5–12, 14); `dbg!(r);` is unresolved in
    Tide's std (FX0103).
  - Results accept: exit 0, rustc warnings and FX0508 only.
  - Async misc: FX0601 ×3.
  - catch: FX0701; `resume_unwind` E0433 only.
- **Batch 2** (21:30Z, 8 files):
  - FX0601: `std::future::ready`, `core::future::ready`, `std::pin::Pin`.
  - `impl std::future::Future<..>` (argument position): E0433 only.
  - `impl core::future::Future for S`: FX0601 at L2 and L4, FX0520 at L4–5.
  - `std::thread::spawn` FX0602; `use std::future::Future` + `impl Future`: E0432 only;
    `std::panic::catch_unwind` FX0701.
- **Batch 3** (21:33Z, 10 files):
  - FX0601: `use std::future::{IntoFuture | Ready | Future as F}`, `use std::pin::Pin`.
  - `use std::task::Wake` FX0520; `use std::sync::Mutex` + `&Mutex` FX0604.
  - `&dyn std::future::Future`, the impl header, and `F: std::future::Future`: E0433 only.
  - Write-trait imports: exit 0.
- **Batch 4** (21:35Z, 4 files):
  - s1 (`std::fs::read` + `F: std::nope::Missing`): E0425/E0433, FX0501 lost.
  - s2 (`async fn` + `F: std::future::Future`): FX0601 kept.
  - s3 (`F: Missing`): FX0501 lost.
  - s4 (unresolved *type* `Missing`): FX0501 kept.
  - Together these show finding 1 is older than this change and lies in the HIR walk.

Not run: full workspace or corpus suites, or a rebuild. No edits, staging, commits or `br` writes.
Nothing under `trials/e1-checked/` was opened or searched. DONE
