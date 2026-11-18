Model: claude-opus-5-5 · Date: 2026-11-18 · Candidate cc2d9f3

Round: fx-kw8.25 r2, the Opus converge round (report only), on the tree that Codex r1 reported clean.
Diff: `git diff main...cc2d9f3`, SHA-256 `66fb67ce538145b1eef9622c329acf2c363f923aa9b6113a29896cf9675226f3`. It is test-only, +199 lines in `crates/tide-admit/tests/driver.rs`. HEAD is `cc2d9f3a72902e0b5cd2555266667f817d889278` and the worktree is clean. Merge base is `50164f9`. `main` has since moved to `cb2d0c0`, but only `.beads/issues.jsonl` changed (ledger commits), so a rebase changes no check input.

I read AGENTS.md, GOVERNANCE.md, `br --no-db show fx-kw8.25` with its comment (FX means TD), the builder report, r1-codex.md, the `admit` helper, `diag.rs` (`Diagnostics::report`/`emit`), and the backstop's witness note in `walk/mono.rs:59–70`. I ran no builds or tests, started no subagents, and did not open `trials/e1-checked/`.

## Verdict

**approve**. Confidence: high.

## Blocking findings

None.

## Non-blocking findings

1. **A spanless TD diagnostic is skipped silently instead of failing** (`driver.rs:2104`). The filter requires `": error: TD"` / `": warning: TD"`, with a `": "` before the level. A TD diagnostic with no span renders in short format as `error: TD0520: …` with no `lib.rs:L:C:` prefix, so the filter misses it, `place` hits `continue`, and the diagnostic never reaches the multiset.
   Concrete case: a variant of the walk case gains one extra spanless TD error. The exit code is still 1 and the multiset is unchanged, so the variant passes. (A base diagnostic that *loses* its span is still caught as missing. The accepted control is still caught by its exit code.)
   Minimum fix: match `format!("{s}: TD")` rather than `": {s}: TD"`. A spanless line then fails `strip_prefix("lib.rs:")` and returns `Err("unparsed")`. This is low risk today, because TD diagnostics always carry a user span (the backstop falls back to the crate span).

2. **"Walk reached" is inferred, not asserted** (`driver.rs:2224`). The backstop case asserts that every diagnostic carries the backstop's `(it runs as `…`)` witness. The walk case does not assert the reverse. Codex's argument that walk errors stop the run before the backstop is about pipeline order. It does not prove that the walk case's diagnostics came from the walk.
   Minimum fix: one assertion for both cases, `base.iter().all(|d| d.5.contains("(it runs as `") == (case == "backstop"))`. The walk baseline (TD0520 atomic/VarError/_print, TD0501, TD0508) has no witness today, so this passes and makes lens 1's "both rejection paths reached" a checked fact.

3. **The self-check can skip itself silently** (`driver.rs:2234`). `if let Some(first)` skips the corruption checks when the baseline has no TD *error* line. A rejecting case whose blocks owned only TD warnings, with exit 1 coming from a non-TD rustc error, would pass without a self-check. Similarly, severity is a compared field only because `print` happens to emit a TD0508 warning, and nothing asserts that a warning is present. If TD0508 stopped being a lint, severity coverage would disappear without a failure.
   Minimum fix: for `exit == 1`, use `.expect("rejecting baseline has a TD error")`. Optionally add `assert!(base.iter().any(|d| d.3 == "warning"))` for the walk case.

4. **Optional deletion: the standalone `comments` and `unrelated` variants** (`driver.rs:2260–2261`). `combined` applies both of them along with reversal, so as falsifiers they duplicate it (AP-TEST-001). Dropping them saves 6 of the 18 driver runs, about 2 s. Keeping them buys failure localization, and the bead lists each transformation separately. Either choice is acceptable, and I would not block on it.

5. **Nits.**
   - The self-check parses `first` a second time with `unwrap`s and a fixed `[..6]` slice for the code (`driver.rs:2236–2241`), duplicating `place`'s parser. It is acceptable, because the corruption has to be applied to the raw stderr.
   - It also calls `lay_out(blocks, "")` again (`:2235`). Returning the `lines` from `check` would avoid that.
   - `lay_out` relies on every gap and block ending in `\n` (`:2089–2092`), and nothing states this. Today's fixed data satisfies it. A one-line doc note or `debug_assert!` would protect future fixture edits.

## Reasoning by lens

**1. Does the test prove the Acceptance?**
- *Same multiset after mapping.* `place` keeps block, block-relative line, column, severity, code and the full message. It sorts without deduplicating, and every variant must equal the baseline exactly, with the same exit code. Reorder, comment gaps, the unrelated item, fresh runs (`repeat`, each a new process via `admit`), and their combination are all covered.
- *Accepted control.* It requires exit 0 and an empty multiset across all six layouts.
- *Self-check.* It applies five corruptions to the real baseline stderr: drop, duplicate, recode, line+1 and column+1. Each corruption still parses and lands in the same block (line 2→3 of a 3-line block), so it differs in a compared field rather than only becoming an `Err`.
- *Vacuity.* If a fixture stops rejecting, the exit assertion and the per-block `flagged` assertion fail. If rustc's short format changes, either the prefix parse fails (`Err("unparsed")`, a panic) or no TD line matches. In the second case flagged blocks own nothing and the test fails. The only silent gaps are findings 1 and 3.
- *Backstop reached.* The `(it runs as` witness is written only at `walk/mono.rs:69`, so the backstop assertion is sound. Finding 2 covers the walk side.

**2. Location mapping.** Source and mapping come from one block list in `lay_out`. `first` is measured after the gap, and the end is exclusive (`first + items.lines().count()`, checked with `(a..b).contains`), so gap lines belong to no block. A diagnostic that starts in a gap is an error, which is exactly what the builder's mutant demonstrated (`mutant.log`: "in no block" in the four gap variants). Only the start line is used, so diagnostics whose spans cover several lines map correctly. TD messages contain no newlines (`synthesis.rs:66` strips them, and no walk message builds one), so short format gives one line per diagnostic. Gaps are whole lines and indentation never changes, so the identity mapping for columns is correct. `Diagnostics::report` deduplicates on `(code, message, file, absolute line)`. Each block shifts as a whole and no two items share a line, so that key groups the same way in every variant.

**3. Fragility.** Every comparison is baseline-against-variant within one run, so changes to diagnostic wording cannot break the test. Only two things are coupled to fixed facts: the backstop witness substring, which `mono.rs` owns and an existing test at `driver.rs:530` already pins, and the fixtures' current verdicts (atomics, `env::var` and `println!` denied; `&dyn PartialOrd<fn()>` and `Debug for fn()` backstop rejections). If a future allowlist change admits one of these, the test fails loudly at the `flagged` assertion, and the fix is a fixture swap. That is the right failure. The regressions this test targets are traversal-order dependence, dedup that depends on visit order, absolute-line leakage into messages, and dropped or duplicated records. All of them reach a compared field.

**4. Cost.** The 18 runs are sequential `--emit metadata` checks of tiny libraries. They take 6.85 s alone, and the driver binary's total moved from about 18 s to 21–25 s in the logs. Next to the 107 s binary already in the workspace suite, this is acceptable. Finding 4 can cut it by a third if wanted.

**5. Simplicity.** Two small local helpers, fixed data, and no parser, dependency, snapshot or production change. Concrete deletions are listed in findings 4 and 5. None is required.

Proof I relied on, which is the builder's and which I did not rerun: `focused1.log` (1 passed, 6.85 s), `driver.log` (42 passed), and `workspace.log` headed `cc2d9f3a72902e0b5cd2555266667f817d889278` (all binaries ok, including this test at line 201).

DONE
