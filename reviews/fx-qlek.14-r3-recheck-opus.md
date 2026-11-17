Model: claude-opus-5-5 · Date: 2026-11-17 · Candidate b2ba030

Scope: a re-check of my new finding N1 only, from r2-recheck-opus.md (on e3a41fb). HEAD is `b2ba030` and the worktree is clean. `git diff 81dbeed...b2ba030 | sha256sum` gives `f2942d6de4f6010de59011cb7176638befc16885610ffd117d2f5eafce256b24`, which matches the brief.

What I read:
- `git diff e3a41fb b2ba030`: 3 files, +15/−8;
- report.md, Round 3;
- `current_artifacts`, `validate_replay` and `run_test_process` in `crates/tide-admit/src/invocation/replay.rs`;
- the `#[path]` module includes in `crates/tide-admit/src/main.rs`;
- `entries`, `shown` and `Manifest::validate` in `crates/tide-std/src/replay.rs`;
- the JSON string encoder and decoder in `codec.rs`;
- the test's `refuse` helper and its component loop in `crates/tide-admit/tests/build.rs`.

I ran nothing, started no subagents, and changed no repository files. The builder reports tide-admit 181/0 and workspace 349/0. I did not rerun them.

## Verdict

approve

## N1: a manifest name reached stderr unescaped (fixed)

**Every manifest-derived name on these paths is escaped.** In `current_artifacts`, a manifest name can reach stderr three ways:
- **The stat error.** `try_exists`'s `map_err`, at `replay.rs:154`.
- **The open or read error.** `file_digest`'s `error` closure, at `replay.rs:487`. It covers both the `File::open` error and the hashing read error.
- **The comparison.** `Build::compare` → `entries`, which handles both keys and values.

All three now go through `wire::shown`. The fixed-text refusals print no manifest text: "not a file name", the `validate` messages (the seed prints only as a canonical decimal), the configuration mismatch, and the test identity. I found no other route from a manifest name to stderr in the driver.

**One owner.** `shown` is defined once, in `crates/tide-std/src/replay.rs:180`, as `text.escape_debug().to_string()`.
- **tide-std.** `entries`, which backs `Build::compare`, calls it for the key and both values. Its output is byte-identical to the old inline `escape_debug`: `shown(&short(v))` gives the same string as `short(v).escape_debug().to_string()`, and `shown(key)` gives the same as `key.escape_debug()`. So the existing unit test's pinned escaping (`lib\u{202e}\n.rlib`) still holds.
- **tide-admit.** It reaches the same source file through `#[path = "../../tide-std/src/replay.rs"] mod replay;` in `main.rs:97-98`. `invocation/replay.rs:3` imports it as `use crate::replay as wire;`. That is one definition compiled into both crates, so there is no second escaper.

The builder left `interface/diff.rs`'s own `shown` alone. It predates this bead and has a different test-pinned contract, so I agree with leaving it.

**The falsifier reaches the `try_exists` error path.** The arm inserts the key `"x\u{1b}\0"` into `dependencies[1]`.
- **The manifest round-trips.** The encoder writes ESC and NUL as `\u001b` and `\u0000`, and the decoder accepts escaped control characters (it rejects only raw ones). `refuse` reseals, so `Manifest::validate` passes; it does not inspect dependency names.
- **The name passes the file-name check.** `Path::new("x\u{1b}\0").file_name()` is the name itself, because it contains no separator.
- **The stat fails.** `dep.join(name)` contains a NUL. `fs::metadata` fails converting it to a CString with InvalidInput, not NotFound, so `try_exists` returns `Err`.
- **Nothing else refuses first.** The real names sort before it, and they exist and hash cleanly.
- **The output.** The driver prints `tide-admit: <dep>/x\u{1b}\0: file name contained an unexpected NUL byte` and exits 2, before any build or child process starts.

The arm's component string is the raw literal `x\u{1b}\0` (backslashes included), which matches the escaped form.

**It would fail on e3a41fb.** That tree printed `path.display()` raw. stderr would hold a raw `x`, ESC, NUL sequence, so `contains(r"x\u{1b}\0")` fails. The new assertion, that no raw `\u{1b}` appears in stderr, fails too. It now runs for every arm, so a regression on any refusal path would also be caught.

A name over 255 bytes (ENAMETOOLONG) takes the same escaped `map_err`, so it needs no separate test.

## New findings

None blocking. Two notes:
- **Not blocking: `file_digest` has no falsifier for a manifest name.** No test covers a manifest name that exists but fails to open or read. Triggering it needs an operator-side entry with a hostile name, which is low risk, and it goes through the same helper.
- **Nit: operator paths change slightly.** `escape_debug` now also applies to operator-chosen paths in `file_digest` errors (std, cc, ld, the driver, the executable). A backslash or quote in such a path now prints escaped. This changes error text only, and no test pins it.

The tide-std change is a behavior-preserving extraction. The test change adds one arm and one assertion. Neither introduces anything new.

DONE
