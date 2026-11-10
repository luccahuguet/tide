Model: claude-opus-5-5 · Effort: low · Date: 2026-11-09

# fx-ecb round 2: Opus narrow re-check of `db09517..9c5a597`

## Verdict

**approve**

Each of my taken findings (N1, N3–N8) is resolved, and so is the coordinator's reference trim. In the changed lines I found two new nits, both non-blocking: one is code and one is text.

## Scope and checks

- **Read:**
  - the brief, fix1-brief.md and fix1-report.md;
  - `git diff db09517 9c5a597` for every file;
  - the stub's crate doc and conformance/README.md's verdict wording;
  - reference.md §1, lines 36–47.
- **Ran:** `scripts/count-tokens.sh design/reference.md`. It returned 24998, which is within D14's 25,000 and matches design/reference-companion.md:14.
- **Probe:** I built `9c5a597`'s `shared.rs` alone as an rlib with `rustc +stable --edition 2024`, under `nice -n 19`, after checking that no quiet window was active. It used a private `git archive` copy in my scratchpad. The build exited 0 with no warnings, and in particular no `unconditional_recursion` warning on the `Seek::seek_relative` delegation.
- **Not rerun:** cargo test and conformance. The author's suites cover these sources. The coordinator's trim after those suites touched only design/reference.md and design/reference-companion.md, so it needs no rerun.

## Per-finding status

**N1 (cancellation-fused `Lines`/`Split`/`Bytes`): resolved.**

- **Code.** `Bytes`, `Split` and `Lines` each have a `done` flag:
  - `Bytes` at shared.rs:933–959, `Split` at shared.rs:1076–1103, `Lines` at shared.rs:1109–1139.
  - The flag is set to `e.is_cancelled()` only on a yielded error, and `next` returns `None` after it is set.
  - `Bytes` still retries `Interrupted` first, and any other error keeps Rust's continue-after-error behaviour.
- **Tests.**
  - io/mod.rs:105 now asserts `None` twice after the cancelled item. It also shows that the borrowed reader can still `read_line` the data, which is the shielded-caller case.
  - io/mod.rs:137 covers all three adapters for both `Cancelled` and `Other`.
- **Design.**
  - 03:193–194 states the rule.
  - 03:152 refers to it ("Like `io::Lines`…") and keeps the `Stdin::lines()` promise unchanged.
  - The README, conformance/README, mod.rs and shared.rs headers match it.
- **Coordinator's reference trim (reference.md:43–45).**
  - "Dropping a `BufWriter` discards unflushed bytes; `Lines` ends after a cancellation error (03 §1.1)" is accurate. It names `Lines` only, and 03 §1.1 carries `Split` and `Bytes`.
  - The count is within budget.
  - It does not mention TASK-6's `warn!`. That is fine: "discards" is true either way, and the warning is deferred to fx-5vj.

**N3 (`io::prelude`, `From<TryReserveError>`): resolved.**

- `pub mod prelude` is at shared.rs:18–20, with the four traits.
- `From<TryReserveError>` is at shared.rs:315–319 and returns `ErrorKind::OutOfMemory.into()`, which is Rust 1.96's body.
- tests/io.rs:15 exercises the prelude and `try_reserve(usize::MAX)?` into `io::Result`, and checks the kind, the "out of memory" message and that there is no payload.
- The stub gets both through the shared `include!`.

**N4 (Cursor `read_exact` past the end): resolved.**

- shared.rs:834–841 now has Rust's exact shape: `pos += buf.len()` on `Ok`, and `pos = len` on `Err`.
- The `Ok` arm cannot overflow, because success implies the bytes were within the data.
- io/mod.rs:238 covers positions 1 and 10 and asserts that the position ends at 3.

**N5 (`BufReader::stream_position`): resolved.**

- shared.rs:1257–1264 matches Rust's override, including the checked subtraction and its panic text.
- io/mod.rs:322 asserts that the buffer is kept after `stream_position`, and io/mod.rs:345 covers the panic.
- The same arm brings the inherent `seek_relative` (shared.rs:1226–1235). Using `checked_add_signed` handles `i64::MIN` without Rust's negate edge, and the fallback goes through `seek(Current)`.
- The trait override delegates to the inherent method. Inherent methods take precedence, so this is not recursion: rustc gave no lint, and the test's `Seek::seek_relative(&mut r, -1)` restores `abcd`.

**N6 (`ErrorKind::Uncategorized`): resolved.** The doc line is at shared.rs:79.

**N7 (panic inside `flush_buf`): resolved.**

- shared.rs:1337–1372 has a `BufGuard` holding `&mut self.buf` and `written`. Its Drop drains the acknowledged prefix on `Ok`, on an `Err` return and on unwind.
- The borrows are disjoint from `self.inner` and `self.panicked`, and it compiles.
- The `WriteZero` message is Rust's.
- io/mod.rs:487 runs this case: `ab` is accepted, then the writer panics, `cd` stays buffered, the retry writes `cd`, and the output is `abcd` with nothing duplicated.

**N8 (stub crate-doc line): resolved, with a wording nit (R2-2).**

- conformance/rustc/std/src/lib.rs:14 is rewrapped. The remaining lines over 100 columns in that file were there before this change.
- "The remaining stub methods use `unimplemented!()`" fixes the old "its own bodies" wording.

## New findings (non-blocking)

**R2-1. Nit, code: `BufWriter::panicked` no longer has a production reader.**

- It is written at shared.rs:1352, 1358, 1386 and 1388, and read only by the test at io/mod.rs:509 and 513.
- After the TASK-6 change, Drop (shared.rs:1430) does no I/O, and nothing like Rust's `into_parts`/`WriterPanicked` reads the flag. It is now state that only a test can observe.
- rustc does not warn about it: the stable build is clean.
- The coordinator chose to keep it ("keeping `panicked`"), so this does not block. Either:
  - delete the field and the two test asserts (AP-MINIMAL-001: delete what the change orphans); or
  - add a one-line note naming the future reader, if `into_parts` is planned.

**R2-2. Nit, text: "The corpus decides type and borrow verdicts only" is too narrow.**

- Where: conformance/rustc/std/src/lib.rs:16–17.
- The subject changed from the stub ("it") to the corpus. The corpus also decides admission (FX) verdicts, and conformance/README.md:134 says "compile-time verdicts only".
- Suggested text: "The corpus decides compile-time verdicts only."
- Text-only, so it needs no re-check.

DONE
