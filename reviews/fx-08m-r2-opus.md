Model: claude-opus-5-5 · Effort: default · Date: 2026-11-11

Verdict: approve

Candidate: `65d5a17` on base `3729cb8`, branch `fx-08m`, worktree clean. `git diff 3729cb8 65d5a17 | sha256sum` is `393d7342351406ba785b0d0b5186fd792df3c56874f9e8fbd889836ec2891342`, which matches. This was a read-only re-check: no build, no probe, no writes outside this file. I did not open `trials/e1-checked/`.

## Finding 2: the call budget is released. Resolved.

- The test is in `conformance/cases/codec/json.rs:659-668`, inside `std_types()`, which `main` calls at `:899`. In one `json::decode` call each, it round-trips:
  - `vec![7u8; 1000]`;
  - a 300-element `Vec<Named>`.
- Each element goes through `input.decode()` (`crates/tide-std/src/codec.rs:811-812`), so each counts against `DECODE_LIMIT`.
- The budget is released at `codec.rs:164-172`: `self.calls -= 1` runs on the `Result` before returning, on both the success and error paths.
- Removing that decrement would trip the limit at the 257th element, so either assertion would fail. The fixture runs against production through the driver and also compiles against the stub, so the test has teeth.

## Finding 5: spanned error for container `default` on unit and newtype structs. Resolved.

- `crates/tide-macros/src/codec.rs:100-109` adds `Place::UnitStruct` and `Place::NewtypeStruct`.
- `:271-276` picks the place from `Fields::Unit` or from `Fields::Unnamed` with `unnamed.len() == 1`.
- That test matches the newtype branch in the expansion, `Fields::Unnamed(_) if fields.len() == 1` at `:521` (`fields` counts every tuple field, since tuple fields cannot be skipped). So no newtype representation can slip past with `default`.
- `default` stays allowed only on `Field` and `Struct` (`:148-152`).
- `rename_all` is still accepted on all three struct places (`:139-147`), so nothing that was accepted before is now rejected, except the intended case.
- The error comes from `meta.error`, so it is spanned at the key (`:166-168`).
- Negative cases are in `crates/tide-admit/tests/build.rs`:
  - `unit_default` (Decode) at `:1430-1435`;
  - `newtype_default` (Encode) at `:1436-1441`.

  Both expect the diagnostic on line 2.
- The existing `enum_default` expectation at `:1418-1423`, "…it goes on a field or a struct", is still a prefix of the longer message, so it keeps passing.
- The `Self: Default` bound at `:370-373` can no longer arise for a newtype, because the attribute is now rejected first.

## Finding 6: `Box`'s `__missing` delegates. Resolved.

- `crates/tide-std/src/codec.rs:775-777` now reads `T::__missing(input, field).map(Box::new)`.
- For a `Box` of a non-`Option` type, `T::__missing` is the trait default (`:42`), so the "missing field" error is the same as before.
- Field and container `default` still take precedence in the derive (`crates/tide-macros/src/codec.rs:584-591`), so the precedence is unchanged.
- The test adds `boxed: Box<Option<u8>>` to `Profile` (`conformance/cases/codec/json.rs:121`):
  - the absent-key assertion at `:134` requires `Box::new(None)`;
  - the round trip at `:135-138` covers explicit `null`.

## Rebases: both sides kept, codec behavior unchanged. Resolved.

`git range-diff 8df2ee5..8f06e9a 3729cb8..65d5a17` shows exactly two kinds of interdiff.

**The intended fix hunks** (findings 2, 4, 5, 6 and the doc fixes) and nothing else in code. The codec engine, json.rs code, root.rs and the derive expansion are otherwise identical.

**Conflict and context changes, each keeping both sides:**
- `crates/tide-macros/Cargo.toml:4` keeps fx-m72.1's "test" and adds the codec derives.
- `crates/tide-macros/src/lib.rs`:
  - `:8-10` has `mod codec; mod error; mod test;`;
  - `:24-29` keeps the `#[test]` entry point;
  - `:32-48` has both codec derives;
  - `:1-3` has both doc additions.
- `crates/tide-std/src/lib.rs`:
  - `:13-18` keeps fx-o8j's `std::time` bullet;
  - `:37-39` keeps fx-m72.1's `#[test]` bullet;
  - `:45-48` adds the codec bullet;
  - `:60-62` drops both "property tests" and "the `Encode` and `Decode` derives" from the not-provided list. The remaining fx-m72.1 items, `#[sim_test]`, requires-guided generation and generated contract tests, are still listed.
  - `:67-69` names `time.rs` beside `codec.rs` and `json.rs`.
- `conformance/rustc/std/src/lib.rs:8-11` names `codec.rs`/`json.rs` and `time.rs`.
- `conformance/README.md:136-141` keeps the time paragraph and adds the codec one.
- The auto-merged `README.md:194-196` and `crates/tide-std/src/root.rs:23-31, 96-99` contain only this bead's additions over current main, with nothing of main lost.
- The `build.rs` changes are only the two new negative cases.

## Text-only fixes 1, 3, 4, 7 (glanced, not re-checked)

These are present at:
- `crates/tide-std/src/json.rs:45-46` (finding 1);
- `json.rs:30-32` (finding 3);
- `codec.rs:584-592` (finding 4, the lazy `expected` closure, called only in the three error branches);
- `json.rs:14-15` (finding 7).

## Anything introduced

Nothing that blocks. There are three nits, none needing a re-check:
- `crates/tide-std/src/json.rs:12-15`: "Decoding:" is followed by a paragraph that starts "Decoding is stricter…" before the bullets, which reads a little doubled. Folding the sentence into the heading line would fix it.
- Two doc-comment lines from the conflict resolutions run past 100 columns, and rustfmt does not reflow comments:
  - `crates/tide-macros/src/lib.rs:2`;
  - `crates/tide-std/src/lib.rs:67`.

  Both are cosmetic.

DONE
