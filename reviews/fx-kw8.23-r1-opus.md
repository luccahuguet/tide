# Tide review, fx-kw8.23 round 1 (Opus): converge review round

Candidate: working tree on base ff9900f (`git diff HEAD -- . ':!.beads'`), diff SHA-256
`e182876d86ad3565457dc620ba1f66d875312dd4bfcbb34d53bc4fc1b606c6c0`. I checked it at the start and the end, and it did not change.

Model: claude-opus-5-5 · Date: 2026-11-03

## Verdict

One finding blocks (B1, a small code fix). The mechanism is otherwise correct and meets the bead's Acceptance:
- `visit_foreign_item` now sends every foreign item through `granted`, so a `safe` fn and a `safe static` get FX0901 without the grant and are recorded as a site hole with it.
- The driver test covers both verdicts and checks for the `lib::abs` hole.
- UNSAFE-1's text, the reference row, and the rules.md row agree with each other.

Edition 2024 rejects a plain `extern` block, so "`unsafe extern` items" covers every foreign item v0 can write. Two nits are text-only. One later-tier note goes on fx-djp, and one out-of-scope concern needs its own bead. Every item in the new-surface report should stay (N3).

## Findings

B1 (bug, not text-only, **blocks**). An `unsafe` foreign fn without the grant now gets two FX0901 errors on one line.
- **Where:** `crates/tide-admit/src/walk.rs:1723` (new) and `walk.rs:1042-1043` (`fn_item`). Both fire for `fn tide_probe_ext();` at `conformance/cases/authority/unsafe_ungranted.rs:25`:
  - `a foreign item needs the package's `unsafe` grant`, at `item.span`;
  - `an `unsafe fn` needs the package's `unsafe` grant`, at `def_span`.
- **Why the dedup misses it:** `Diags::report` (`diag.rs:116-118`) dedups on (code, message, file, line), and the two messages differ.
- **Why nothing caught it:** `conformance/run.sh` folds driver errors with `sort -u`, so the case still passes with one `//~ FX0901`. The new driver test uses only a `safe` fn, so nothing pins the count.
- **Before this diff:** the item got one error.
- **With the grant:** the same item gets two holes ("a foreign item" and "an `unsafe fn`"). That was already true before this diff, and the same fix removes it.
- **Fix:** let the foreign-item check own foreign items (AP-OWNERSHIP-001).
  - At `walk.rs:1042`, change the condition to `if header.is_unsafe() && !tcx.is_foreign_item(def)`. `fn_item` already calls `tcx.is_foreign_item(def)` a few lines below.
  - Say so in its doc comment at `walk.rs:1035`, e.g. "`unsafe fn` needs the grant (UNSAFE-1; a foreign one, as a foreign item)".
  - Add a non-`safe` line such as `fn probe();` to the driver test's source (`driver.rs:732-734`), and expect exactly one `a foreign item` line for each of the two items in `fx_lines`. Only the driver test can pin the count, because the runner dedups.
  - The hole test at `driver.rs:712` (`lib::outer::inner`, not foreign) is not affected.

N1 (nit, text-only, non-blocking). Two texts still name the foreign `unsafe fn` separately, though "every foreign item" now covers it.
- `design/03-authority-concurrency.md:701-703`: "(with a body or not: a required trait method or a foreign function declared `unsafe` too), every item of an `unsafe extern` block (...)". Fix: "(with a body or not: a required trait method too), every item of an `unsafe extern` block (...)".
- `conformance/cases/authority/unsafe_ungranted.rs:2-3`: "an `unsafe fn`, with a body or not (a required trait method, a foreign function), every foreign item, `safe` ones included". After B1's fix this is also wrong: the foreign fn is reported as a foreign item, not as an `unsafe fn`. Fix: "an `unsafe fn`, with a body or not (a required trait method), every foreign item, `safe` ones included".

N2 (later-tier note, text-only, non-blocking, for a comment on fx-djp). FFI-1 at `design/03-authority-concurrency.md:1085-1086` says "Calling a foreign function is `unsafe` and needs a `// SAFETY:` comment".
- This is false for a Rust 2024 `safe` foreign fn, which is exactly what this bead is about.
- FFI-2's "every foreign call is a counted trust hole" (`03:1091`) depends on it.
- What v1 should decide: either FFI-1 forbids `safe` in `extern` blocks, or it says how a call to a `safe` item is counted and reviewed. It should also say whether the `ffi` grant replaces UNSAFE-1's `unsafe` grant for foreign items or comes on top of it.
- Only v1 text changes, so this does not block.

N3 (new-surface report: `pub` items in case and test source). All seven stay.
- `unsafe_granted.rs:20` `pub safe fn abs`: **stay.** It is the admitted half of the Acceptance in the corpus. `pub` matches every other item in the case.
- `unsafe_granted.rs:23` `pub fn magnitude`: **stay.** It shows that a call to a `safe` foreign fn is admitted under the grant with no `unsafe` block. The rustc oracle would give E0133 if the item lost `safe`, and the call goes through the driver's `use_def` check of a foreign fn.
- `unsafe_ungranted.rs:26` `pub safe fn abs`: **stay.** This is the bead's regression.
- `unsafe_ungranted.rs:27` `pub safe static TIDE_PROBE_COUNT`: **stay.** It is the only non-fn foreign item. It takes a different path (no `fn_item`) and shows "any `extern` block item" from the Acceptance.
- `unsafe_ungranted.rs:30` `pub fn magnitude`: **stay.** It shows the verdict falls on the declaration alone, with no second diagnostic at the call. That is the claim on the case's comment line 5.
- `driver.rs:733` `pub safe fn abs`: **stay.** The Acceptance requires a driver test, and the hole assertion names `lib::abs`.
- `driver.rs:735` `pub fn magnitude`: **stay.** The exact one-line `fx_lines` assertion is what shows the call adds no diagnostic. It is cheap.

O1 (concern, out of scope, non-blocking; recommend a bead under fx-kw8, not a fix here). An `unsafe extern` block with no items is still admitted without the grant, and so is any `#[link(...)]` attribute on it.
- `#[link(name = "...", modifiers = "-as-needed")]` (or `+whole-archive` with `kind = "static"`) makes the test binary link a native library that no item references. The library's load-time constructors then run in the candidate's process.
- That would be a RUN-4 route that UNSAFE-1 does not list. Search paths limit it, since the driver passes no user `-L`.
- `unsafe_attrs` (`walk.rs:1119-1131`) handles only `no_mangle`, `naked`, `export_name`, and `link_section`, and nothing checks `ItemKind::ForeignMod`.
- I have not checked link-modifier stability on the pinned toolchain or which link modes the harness uses. The bead should confirm the hole first.
- Likely fix: `granted` on the block's `#[link]` attribute (or on the block itself) as "a native library link".
- Per-item holes stay the better unit for review, so flagging the block should not replace them.

## Checked and acceptable
- The diff hash matches at the start and the end.
- The hole span and dedup key are unchanged: before, `hole(item.span, .., true)`; now `granted(item.span, ..)` takes the same path.
- `granted` still skips code written by another crate and code the toolchain generates.
- `design/reference.md:1073` ("Not in v0: FFI") does not conflict with the new row. FFI's design (`ffi` grant, wrappers, models) is v1 per `01-design.md:83`, and foreign items in v0 are unsafe code under UNSAFE-1, as the bead allows.
- `visit_foreign_item` does not call `static_item` for a foreign static, so GLOBAL checks skip it. That predates this diff, and the item is now a granted hole anyway.

## Verification
I only read: `git diff HEAD -- . ':!.beads' | sha256sum` (twice), `br show --no-db fx-kw8.23`, the relevant parts of `walk.rs`, `diag.rs:107-127`, `conformance/run.sh`, `tests/driver.rs`, both UNSAFE-1 cases, the 03 sections on UNSAFE-1 and FFI, and the FFI lines in 01-design. I ran no builds, as instructed. The recorded `cargo test --workspace` and `conformance/run.sh` results were not rerun. B1 comes from reading the code: the two `granted` calls, the dedup key, and the runner's `sort -u`. No output was observed.
