Model: claude-opus-5-5 · Date: 2026-11-12

Verdict: approve

This is a focused re-check of round-3 findings 1-8 and nit 10 on `git diff 0a1f130 c722700` (15 files), with the "Fix round 2" section of report.md. HEAD is `c722700` and its only parent is `2da0cf0`. The worktree is clean. Nits 9 and 11 were skipped as the coordinator decided. The LogConfig flag is already on fx-djp (the comment of 2026-11-12, around 12:44-12:45Z).

## Per-finding results

1. **Fixed: `Stdin`/`&Stdin` implement `Read`.**
   - The implementations are at `crates/tide-std/src/io/stdio.rs:89-114`.
     - `&Stdin::read` checks the domain and cancellation at entry. An empty buffer returns `Ok(0)`.
     - The real side reads through Rust's locked stdin and `from_std`.
     - The simulated side uses `input_op("stdin.read")` and drains `min(buf.len(), available)` bytes.
     - Owned `Stdin` forwards to the borrowed impl.
   - The stub gets the same code through `io/mod.rs`, which includes `stdio.rs`.
   - Tests:
     - `crates/tide-admit/tests/build.rs:2287-2330` covers a cancelled read, an empty read, a two-byte prefix read followed by intact `lines()`, and owned-`Stdin` EOF, on both backends.
     - `conformance/cases/authority/process_inputs.rs:35-36` resolves `Read` on owned and borrowed `Stdin`.
   - Docs: `lib.rs:20-22`, README:306-307 and `io/mod.rs:17` name the impl.
   - My probe 4, below, agrees on both backends.

2. **Fixed: the authority-free clause is back in §6.5.** `design/reference.md:452` now reads "no reads, capability, or suspension, so functions stay authority-free; a pure decoder may `warn!(..)`", which restores the meaning. My recount gave **24,998** (exit 0), which matches `design/reference-companion.md:14` and is ≤ 25,000. In this round the reference changes only that line.

3. **Fixed: INPUT-1 now has an owner.**
   - `design/03-authority-concurrency.md:116` tags the note `**INPUT-1. Process inputs (v0).**`.
   - `conformance/rules.md:102` points at "03 INPUT-1; §1.3; ref §6.2".
   - INPUT-1 appears only in those two places and in its citing case.

4. **Fixed: the prefixed log macros are labeled not provided.** "`log::`/`tracing::`-prefixed log macros" appears in the not-provided lists at `crates/tide-std/src/lib.rs:213-214`, `README.md:320` and `conformance/README.md:211-212`.

5. **Fixed: trace spans point at the caller.**
   - What changed:
     - Tracked `write_all` overrides for borrowed and owned handles (`stdio.rs:184-187`, `:203-205`).
     - `#[track_caller]` on `Lines::next` (`shared.rs:1079`).
     - A tracked `StdinReader::read_line` (`stdio.rs:143-149`). It takes the single buffered line, so it reads up to and including the LF, as the default does. It returns `Error::INVALID_UTF8`, which `stdio.rs:222-229` checks against ordinary `BufRead`, and it propagates cancellation through `fill_buf`.
   - Test: `stdio.rs:231-270` asserts exact caller lines for five events.
   - Probe 4's simulated trace also attributes `stdin.read` to the user's file.
   - `#[track_caller]` on the generic `Lines::next` changes no behavior for other readers.

6. **Fixed: rules.md hygiene.**
   - `task_controls.rs:1` cites TASK-6.
   - The SINK rows are in ID order (`rules.md:103-108`), and TASK-6 sits before the LOCK block (`:119`).
   - `process_sinks.rs:1` drops SINK-5.

7. **Fixed: the SINK-2 heading no longer contradicts its text.** `design/03-authority-concurrency.md:563-564` now reads "No admission rule enforces this yet; whether a `LogConfig` may leave `main` is open (fx-djp)". No restriction was added.

8. **Fixed: invalid simulated environment keys panic.**
   - `crates/tide-std/src/sim.rs:83-97` asserts that the key is nonempty and contains no `=`/NUL, before taking the runtime lock.
   - `sim.rs:138-148` checks three invalid keys: each panics, the trace stays empty, there is no poisoning, and a valid key works afterwards.
   - 03 §9.2 (`:1909`) documents the panic.

9. **Nit 10: fixed, a single `read_line` path.** `stdio.rs:49-51` always uses `append_string(buf, self.line_bytes()?)`. The shared exercise shows an invalid line is consumed, the caller's `"kept"` string is unchanged, and the next line is readable, on both backends (`build.rs:2312-2315`).

## New defects (nits only, none blocking)

- **Broken sentence in the io docs.** `crates/tide-std/src/io/mod.rs:16-18`: the inserted line splits a sentence. It reads "…checking cancellation at entry; / owned and borrowed `Stdin` implement `Read`. / a real read already blocked…", so a lowercase "a" follows a period. Fix: move the new clause after the cancellation sentence, or capitalize. Text-only.
- **SINK-5 still cited by a case that doesn't exercise it.** `conformance/cases/authority/process_inputs.rs:1` still cites SINK-5, although that case, like `process_sinks.rs`, doesn't exercise load or the 141 exit; the driver tests do. Optional: drop it for consistency with finding 6.

## Probes

Probe 4 used the same constraints as round 3:
- Gate: `build-gate.py 240`, passed at 13:29:10Z.
- Limits: `ulimit -c 0`, nice 19, `CARGO_BUILD_JOBS=4`, timeout 120, stdin closed.
- Build: the candidate's driver and std, both built after the fix. `target/tmp/build/std/artifacts/libtide_std.rlib` is newer than `stdio.rs`.
- Location: sources only in the scratchpad `/tmp/claude-1000/-home-lucca-pjs-lab-tide--worktrees-fx-5vj/190c4344-6d46-4bff-ad32-0f10b529d6c4/scratchpad/p4`.

The probe runs a generic `R: Read` consumer and `BufReader::new(&Stdin)`:
- **Real side:** `printf 'hello\nworld\n12345' | demo` printed `hello|0`, exit 0. As in Rust, `BufReader`'s read-ahead takes the rest.
- **Simulated side:** the `#[test]` gave the same `hello|0`, its trace contains `stdin.read`, and the event is attributed to `demo.rs`. 1 passed.

Recount: `timeout 400 scripts/count-tokens.sh design/reference.md </dev/null` returned **24998**, exit 0.

## Not checked

- The full suites (the coordinator runs them).
- Anything outside the listed findings.
- `trials/e1-checked/` (held out).

DONE
