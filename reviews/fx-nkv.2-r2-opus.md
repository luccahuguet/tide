Model: claude-opus-5-5 · Effort: medium · Date: 2026-11-09

# fx-nkv.2 round 2: Opus narrow re-check

Candidate `b89b4ec` (fix) on `7c82418` (rebase onto main `ccde679`). Read-only; no cargo, no br.

## Verdict

**approve**

## Scope

- `git diff 7c82418 b89b4ec` (the fix commit).
- `git range-diff 0583ecb..f7f77a2 ccde679..7c82418` and `git diff ccde679 b89b4ec` on the shared files: `README.md`, `crates/tide-std/src/lib.rs`, `conformance/rustc/std/src/lib.rs` and `crates/tide-admit/tests/build.rs`.
- README paragraph at 150–175, compared with `ccde679`'s.
- The stub's `task` module (lines 435–466) and the header of `task.rs`.
- Grep for `task`/`TaskError` in `crates/tide-std/src/io/`.
- The author's report, `fix1-report.md`.

I did not reproduce the proof (`cargo test --workspace` and `conformance/run.sh` on b89b4ec, both exit 0 according to the brief and report); the brief forbids builds.

## Item status

**N1 (restore `TaskError::DeadlineExceeded`): fixed.**
- `task.rs:152-167`: the variant is restored, with the doc line "No operation in this lifecycle slice returns this; deadline support belongs to fx-zc1".
- `Display` now matches both variants: "task cancelled" and "task deadline exceeded".
- The enum's doc no longer says the variant is absent.
- The stub `include!`s this file, so the conformance oracle again has the reference §7.1 shape. Exhaustive matches need both arms again, and naming the variant resolves.
- No exhaustive `match` on `TaskError` exists in `crates/` or `conformance/cases/` (grep), so nothing else needed updating.
- The added unit test `task_errors_have_distinct_display_messages` is small, but it checks the new Display arm. Acceptable.

**N3 (Failures SAFETY wording): fixed.**
- `task.rs:60-62` now separates the two claims:
  - dereferencing the pointer to `&Mutex<Vec<thread::Result<E>>>`, which is `Sync` because `E: Send`;
  - reading and writing the data only under that lock.
- The scope-lifetime justification is kept. This matches my suggested text.

**N2, N4, N5: no action required.**
- N2 was a correction to the report's wording plus a note for fx-nkv.3. No code change was requested, and none was made.
- N4 is a suggestion for fx-nkv.3.
- N5 is resolved by the rebase's rewording. The stub doc now says "The remaining stub methods use `unimplemented!()`", which is true with both `io/shared.rs` and `task.rs` included.

**Rebase: correct.**
- **README.md (conflict, hand-resolved).**
  - The merged paragraph keeps everything from main: the io sharing through `io/shared.rs`, `BufWriter<W: Sized>`, Drop without I/O, and the `Lines`/`Split`/`Bytes` cancellation note.
  - It also keeps "The crate docs list what it does not provide yet" and the stub-std sentence.
  - It adds the partial task lifecycle with its stated gaps.
  - Nothing was dropped or duplicated.
- **Conformance stub crate doc (conflict, hand-resolved).** It names all three shared sources (`root.rs`, `io/shared.rs`, `task.rs`), and the rest of the text is main's.
- **`crates/tide-std/src/lib.rs` (auto-merged).**
  - The bullet list has both the task bullet and fx-ecb's io bullet, with consistent punctuation.
  - "Not provided yet" drops "tasks" and keeps main's narrowing of `io`'s Rust items to `time`'s.
  - `pub mod io;` and `pub mod task;` sit side by side.
- **`crates/tide-admit/tests/build.rs`.** It only adds the task test, which uses the existing `package`/`driver`/`checked` helpers. No io test interacts with it.
- **Code interaction.**
  - `io/` does not reference `task`, and `task.rs` uses only Rust's `std::{marker, panic, sync, thread, fmt, ptr}`. In both crates, `std::` resolves to Rust's std, so the new crate-level `io` module does not shadow anything.
  - The stub's `sync` module still finds `crate::task::Tasks` through the included `capability!`.
  - The two `include!`s are in separate modules, and there are no name collisions.

## New findings

None.

DONE
