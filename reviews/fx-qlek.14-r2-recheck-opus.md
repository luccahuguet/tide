Model: claude-opus-5-5 · Date: 2026-11-17 · Candidate e3a41fb

Scope: a re-check of my round-2 findings 1 and 3 only (r2-opus.md, on ef2cc49). HEAD is `e3a41fbfcf3e18f633d3d4a83f3f38080fced1df` and the worktree is clean. `git diff 81dbeed...e3a41fb | sha256sum` gives `1ba83ecbc9f142b80bd935b6ec07b64e38e871d38c69c97b3e7e1bc2cd7325c5`, which matches the brief. I read `git diff ef2cc49 e3a41fb` (2 files), the test's `refuse` helper and its loop, `current_artifacts`, `validate_replay`, `run_test_process`, `Build::compare` and `Manifest::validate` in tide-std, the child's use of the replay context (`sim/manifest.rs`), and the driver's `main`. I ran nothing, started no subagents, and changed no repository files. Findings 2 and 4 are out of scope.

## Verdict

request_changes

Finding 1 is fixed. Finding 3 is fixed as asked, but the fix opens a small new gap (N1 below): a name taken from the manifest can now reach stderr without escaping. The fix is one line.

## Finding 1: no test for the post-build loaded-set refusal (fixed)

The new arm, `"dependency artifact" => bad.build.dependencies[1].pop_last()`, does reach the post-build compare and only that compare:

- **Slot.** With std present, `dependencies[0]` is std's map and `dependencies[1]` is the first `--tide-dep-dir` (`first_dependency`). The same test later asserts that `dependencies[1]` is non-empty and holds only `.rlib`/`.so` names, so `pop_last().unwrap()` drops one real loaded library.
- **The pre-build check passes.** `refuse` reseals the manifest, so `validate()` passes. `validate_replay` then compares `manifest.build` with `current_artifacts(build, &manifest.build)`. That function hashes only the names the manifest lists, so the dropped library is never looked at. Both sides hold the same remaining names with unchanged digests, and every other component equals `build_identity`'s value.
- **The receipt check passes.** In `run_test_process`, `receipt.build.compare(current_artifacts(current, &receipt.build))` re-hashes the receipt's full loaded set, which is unchanged.
- **The post-build compare fails.** `manifest.build.compare(&receipt.build)` then sees the receipt's extra entry, and `Build::compare` reports it as `replay dependency artifact mismatch … <name>: recorded missing, current <digest>`. The driver exits 2 before the context is written or the child spawned, so stdout is empty.

`refuse` asserts three things: exit code 2, stderr contains the component string, and stdout is empty. Here the component string is `"dependency artifact"`, a stronger match than the existing `"dependency"` arm's.

**If the compare line were deleted, the test would fail.** The child does not compare `manifest.build` with the receipt: `sim/manifest.rs` checks only the test name and config, and copies `receipt.build` into its own new manifest. So the candidate would start, and libtest's stdout would be forwarded, which breaks `run.stdout.is_empty()`. The exit code would also be wrong in either outcome:
- The run reproduces. The driver exits 101, not 2.
- The run does not reproduce. The driver exits 2, but stderr says "did not reproduce", without "dependency artifact".

The new comment ("The recorded names all match, but the build loads one more: the post-build check.") is accurate.

## Finding 3: error paths (fixed)

- **`try_exists`.** It returns `Ok(false)` only for an absent entry, a dangling symlink included, which is still reported as missing. That is reasonable. Any other stat error, such as EACCES on an unsearchable directory, is now reported as itself with the path. That is what I asked for.
- **`file_digest`.** Both the `File::open` error and the hashing read error now carry `"{path}: {e}"`. Opening a directory succeeds on Linux and the read fails with EISDIR, so the read-side `map_err` is the one that covers the "is now a directory" case. Both cases are handled.
- **Other callers.** The remaining `file_digest` callers (std, `/usr/bin/cc`, `/usr/bin/ld`, the driver, the staged executable, `loaded_identity`) just gain a path prefix on failure. No test asserts that text. The builder reports tide-admit 181/0 and workspace 349/0. I did not rerun them.

## New findings

**N1. A name from the manifest reaches stderr unescaped** (`crates/tide-admit/src/invocation/replay.rs:154`; blocking). In `current_artifacts`, `name` is a key of the replay manifest's `build.dependencies`. The only check on it before the join is that it is a single file name. The new `map_err` prints `path.display()` raw, and `main` passes the error straight to `eprintln!`.

A hostile manifest can make `try_exists` fail on purpose:
- a name with a NUL byte (JSON `\u0000`) gives InvalidInput;
- a name over 255 bytes gives ENAMETOOLONG.

In both cases the attacker's bytes reach the terminal verbatim, ESC/OSC sequences and bidi overrides included. Before this fix, `exists()` returned false for both, so the name went through `Build::compare`, which escapes it. The unit test with `lib\u{202e}\n.rlib` pins that escaping, and RUN-6 (design/03) requires the toolchain to escape control characters and bidi overrides in everything it prints. So the fix regresses a guarded contract on the manifest input boundary.

Fix: print the path escaped, e.g. `format!("{}: {e}", path.display().to_string().escape_debug())`. Apply the same escaping in `file_digest`'s `error` closure, because in `current_artifacts` the name it prints also comes from the manifest. That case only applies when an entry of that name exists in the operator's directory, so it is lower risk, but one escaped helper covers both. A cheap falsifier: run the `refuse` loop on a manifest whose `dependencies[1]` gets the key `"x\u{1b}\0"`, and assert exit 2 with no raw `\u{1b}` in stderr.

No other new issues: the test change is confined to one arm, and the error-path change alters only failure text.

DONE
