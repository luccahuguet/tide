Model: claude-opus-5-5 · Date: 2026-11-12

Verdict: approve

This round re-checks only my round-2 findings 1, 3, 4, 5 and 6, against `git diff 240aea6..edd9c69` and the "Fix round 1" section of report.md. The frozen HEAD is `edd9c69`, one commit on `42dd813`, and the worktree is clean. Findings 2 (now bead fx-vnb4, which blocks fx-ic2) and 7 (noted on fx-djp) belong to the coordinator and are not re-checked here.

## Outcomes

1. **Fixed (was blocking).** `allow("core::ascii::*")` now sits in the core block (`crates/tide-admit/src/policy.rs:232`). No earlier rule matches `core::ascii`, so first-match order is safe. A `first_matching_rule_decides` assertion covers `core::ascii::EscapeDefault` (`policy.rs:476`). The new shared row `section_1_u8_ascii` (`tests/support/reference_allowlist.rs:9-13`) checks real output: `b'\n'.escape_ascii().to_string() == "\\n"`. That row runs at all three boundaries.
   - The author reports that the row failed at the exact key before the allow was added and passes after it.
   - Probe 1 independently shows `u8::escape_ascii().to_string()` and `std::ascii::escape_default` admitted with no FX line.

2. **Fixed (was non-blocking).** `conformance/cases/authority/admit_reference_sync.rs` is gone, and `global_arc_accept.rs` keeps the Arc proof. The commit no longer touches `conformance/`.

3. **Fixed (was a nit).** A comment at `policy.rs:222` now says the Cursor/SeekFrom and panic keys serve the Rust-std checks.
   - A wording nit remains, which needs no re-check. The comment sits above all four io lines, so it reads as covering the `Cursor::*` and `SeekFrom::*` method globs too. Those globs do matter in programs, because inherent methods get no re-export exemption.
   - Better wording would be "the two type keys and `std::macros::panic` serve Rust-std checks". Removing a method glob by mistake would still fail the production build test, so this does not block.

4. **Fixed (was a nit).** The `pointer-address` row now uses the function item `<*const u8>::addr` and requires `core::ptr::const_ptr::<impl *const T>::addr` (`tests/driver.rs:134-138`).
   - Probe 1 shows the method key reported at `2:13`, separately from the `*const T` type at `2:14`.
   - The row therefore proves the address observer's own rejection, not just the cast's.

5. **Fixed (was a nit).** The Arena, `truncate`, `round_from` and `try_from_float` lines are removed from the production program, leaving that proof with fx-yg5.3 and fx-yg5.4. The §1 collections, io and clock checks stay.
   - The `last_os_error` denial now has its own test, `reference_io_error_last_os_error_stays_denied` (`tests/build.rs:56`).
   - Keeping the witness keys is acceptable, as round 2 said.

None of these fixes changes behavior beyond its finding or crosses a trust boundary. No full review rerun is needed. The fixes also leave my round-2 assessment of conflicts with fx-u65.5 unchanged: the new `core::ascii` line sits in the core block, away from u65.5's hunks.

## Probes

The probe used the candidate's `target/debug/tide-admit`, built at 20:59:25Z, a minute before the 21:00:26Z commit; the report says no source changed after its passing run. It ran against Rust's std in lib mode, through the queue, at 21:02Z, outside the quiet window and far from 02:55Z:

```
(ulimit -c 0; CARGO_BUILD_JOBS=4 timeout 900 bash /home/lucca/.cache/eon-agents/eon-overseer/heavy.sh nice -n 19 ./target/debug/tide-admit --edition 2024 --crate-type lib --emit metadata --error-format=short --out-dir <scratch>/probe3/out <scratch>/probe3/package/lib.rs </dev/null)
```

Source:

```rust
pub fn run() {
    let _ = <*const u8>::addr;
}
pub fn ascii() -> String {
    let s = b'\n'.escape_ascii().to_string();
    let _ = std::ascii::escape_default(b'a');
    s
}
```

Result, exit 1:

```
lib.rs:2:13: error: FX0520: `core::ptr::const_ptr::<impl *const T>::addr` is not admitted
lib.rs:2:14: error: FX0520: `*const T` is not admitted
```

Lines 5-6 produced no FX lines, so the ASCII family is admitted, and the addr method key is reported on its own.

I ran no cargo tests or suites. The author's focused run passed 7 tests; I did not rerun it.

DONE
