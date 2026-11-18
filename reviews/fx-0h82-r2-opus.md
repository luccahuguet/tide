Model: claude-opus-5-5 · Date: 2026-11-18 · Candidate f7144b9

# fx-0h82 r2: Opus converge review

## Verdict

approve

## Blocking findings

None.

## Non-blocking findings

1. **`BufRead` for `VecDeque<u8>` is missing, and the docs don't say so** (`crates/tide-std/src/io/mod.rs:5`, `:22`).
   - Rust has implemented `BufRead` for `VecDeque<u8, A>` since 1.75 (`alloc/src/io/impls.rs`, `fill_buf` = front slice, `consume` = drain).
   - Line 5 now says "the traits implemented for … `VecDeque<u8>`", which a reader can take to cover all four traits.
   - Failing case: `q.read_line(&mut s)` or `q.lines()` on a `VecDeque<u8>` compiles against Rust's std and fails (E0599) in Tide.
   - This is outside the bead's scope (Read and Write only). The builder's report already proposes a follow-up.
   - Minimum fix (text only, optional here): add "`BufRead` for `VecDeque<u8>`" to the "Not provided here" list at line 22, so the gap is labelled (AP-CONTRACT-001).
   - The coordinator decides whether to file the implementation follow-up.

2. **The reference build relies on rustup resolving the pinned toolchain** (`crates/tide-admit/tests/support/admitted.rs` `plain_rustc`, used at `crates/tide-admit/tests/build.rs:188`).
   - `plain_rustc` runs `$RUSTC` or the `rustc` on PATH from `crates/tide-admit`, so rustup picks `rust-toolchain.toml` (nightly-2026-08-09) unless `RUSTC` or `RUSTUP_TOOLCHAIN` overrides it.
   - With an older std whose `VecDeque` lacks the `read_exact` override, the default `read_exact` partially fills `buf` on EOF. The reference transcript would then differ and the test would fail.
   - The failure goes in the safe direction: a spurious red, never a false green.
   - The helper predates this change and the benchmarks share it. No change is needed for this bead.

3. **Simplicity: optional deletions. None changes correctness.**
   - Test, `build.rs` `for n in [3, 4]` "flat" `read_exact` loop: n == len and n > len are already covered by the wrapped cases 6 and 7. The flat loop adds only "unwrapped deque", which `read_to_string invalid` also exercises.
   - Test, the `bytes` line: it exercises only the provided `Bytes` over `read`, which the "read wrapped" and "read short" lines already pin.
   - Impl, `shared.rs:802` `write_all` override: it is observably identical to the provided `write_all` loop, because `write` is never short. Keeping it matches both Rust and the existing `Write for Vec<u8>`, so either choice is fine.

4. **`read_to_string` goes through the provided `append_to_string`.**
   - That function reads into a fresh `Vec`, where Rust appends into the `String`'s own buffer after `try_reserve`.
   - The two differ only on the allocation-failure path, which no test can reach and no program can observe in normal use. No change.

## Reasoning

### Lens 1: correctness against Rust's std

I compared each method with `alloc/src/io/impls.rs` in rust-src of the pinned `nightly-2026-08-09`, lines 450–601.

- **`read`** (`shared.rs:765`): identical to Rust's.
  - It reads `&[u8]` from the front slice only, then drains `n`.
  - The temporary borrow from `as_slices()` ends with the `let` statement, before `drain`.
  - It cannot fail.
- **`read_exact`** (`shared.rs:772`): Rust fails exactly when `buf.len() > front.len() + back.len()`.
  - `split_at_mut_checked(front.len())` is `None` only when `front.len() > buf.len()`, and in that case the copy from the front slice succeeds.
  - Tide checks `buf.len() > self.len()` up front. On failure, both clear the deque, leave `buf` untouched and return `UnexpectedEof` "failed to fill whole buffer" (`Error::READ_EXACT_EOF`, the same const message that `&[u8]` uses).
  - On success, `split_at_mut(front.len().min(buf.len()))` gives the same copies. Because `buf.len() ≤ len`, the back length is at most `back.len()`, so neither slice index can panic. Then both drain `..buf.len()`.
  - n = 0 is a no-op `Ok(())` in both.
- **`read_to_end`** (`shared.rs:785`): line for line Rust's.
  - It does `try_reserve(len)?`, so on OOM the deque is left untouched; Tide's `From<TryReserveError>` gives `OutOfMemory`, as Rust's does.
  - It extends with both slices, clears and returns `Ok(len)`.
- **`read_to_string`**: the provided method runs `append_to_string(read_to_end)`.
  - On invalid UTF-8 the deque is drained (`read_to_end` cleared it), `buf` is unchanged and the result is `InvalidData` "stream did not contain valid UTF-8". Rust gives the same, through `ret.and_then(|_| Err(INVALID_UTF8))` after its truncating guard.
  - Tide's `result.and(Err(...))` is equivalent.
- **`Write`**: `write` and `write_all` extend at the back and return the full length or `Ok(())`; `flush` returns `Ok`. All match Rust's.
- **Overrides Rust has that Tide's traits lack**:
  - `read_buf` and `read_buf_exact`: Tide's `Read` has neither. Rust's `read_buf_exact` appends partial bytes on EOF, but nothing in Tide can observe that.
  - `write_vectored`, `is_write_vectored` and `write_all_vectored`: `mod.rs:22` already lists vectored I/O as not provided.
- **Provided methods**: Tide's `&mut R` forwarding, `Take`, `Bytes` and `write_fmt` route through these impls with Rust's results.
- **Non-`Global` allocators**: `VecDeque<u8>` only, not `VecDeque<u8, A>`. Tide programs cannot name another allocator, and this matches `Vec<u8>`.

### Lens 2: test strength

- **Wrong implementations fail.** The test is differential, with the same body built by the driver and by plain rustc, so each plausible wrong implementation changes a line of the transcript:
  - a `read` that takes both slices: "read wrapped" changes `Ok(2)` to `Ok(6)`;
  - the default `read_exact`, which partially fills `buf` and drains on EOF: the 7-byte wrapped and 4-byte flat lines change their buf and rest;
  - a `read_exact` that does not clear on failure: rest changes;
  - a `read_to_string` that keeps the deque on invalid UTF-8: rest changes;
  - a `write` that pushes at the front: the round-trip changes;
  - wrong error kinds or messages: `res` prints `kind()` and Display.
- **The wrapped layout is guaranteed.**
  - `ring()` asserts `as_slices() == ("ab", "cdef")` in both binaries. The asserting statement uses Rust's real `VecDeque` in both, since `tide_std::collections` re-exports alloc's.
  - A layout change would panic, `checked` would fail the test, and it would never silently test an unwrapped deque.
- **The reference uses the pinned toolchain** through rustup, with the caveat in non-blocking finding 2.
- **The test is deterministic**: no randomness, time, or environment input.
- **Failures are readable**:
  - every line is labelled;
  - the `END` sentinel catches a truncated reference run;
  - Tide's stderr must be empty;
  - `assert_eq!` prints both transcripts in full.

### Lens 3: ownership

- The impls exist once, in `crates/tide-std/src/io/shared.rs`.
- `conformance/rustc/std/src/lib.rs:54` includes `crates/tide-std/src/io/mod.rs`, which includes `shared.rs`. There is no copy in the stub.
- `std::collections::VecDeque` names the same alloc type in both crates (the stub crate is `tide_std_stub`, whose `std` is Rust's). `shared.rs` already names `std::collections::TryReserveError` the same way.

### Lens 4: simplicity

- The impl is minimal. Each override except `write_all` is needed for Rust's observable results:
  - without `read`, the trait has no required method;
  - without `read_exact`, the provided loop partially fills `buf`;
  - without `read_to_end`, the deque is not left intact on OOM.
- `write_all` is kept for parity with Rust and `Vec<u8>` (finding 3).
- No admission entries were added, and none are needed.
- The test reuses the existing helpers. Optional trims are in finding 3.

### Lens 5: leftovers

- The `mod.rs:5` listing was updated. Apart from finding 1, no other doc, test, admission table or `design/reference.md` line lists io implementors.
- I found nothing orphaned.
- `main` has moved past the candidate's base (5593f4e → 00d5a80). The coordinator rebases between rounds, as the Parallel work rules say.

I ran no builds or tests. My evidence is the diff, the pinned rust-src, and the surrounding code.

DONE
