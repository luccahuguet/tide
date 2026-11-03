# Tide review, fx-kw8.13 round 2 (Opus): confirmation round

Candidate: base ceb39de, fixes bd42674..aa4106c (git trees)

Model: claude-opus-5-5 · Date: 2026-11-03

## Verdict

**Clean.** Every round-1 finding is resolved, and nothing blocks. The two notes below are nits.

**B1 is resolved.**
- `interface.rs:4-5` (module doc) and `README.md:64-65` no longer name `#![no_std]` as the way to reach the writer's failure.
- `write`'s doc (`interface.rs:63-67`) now names the operator's `--tide-std` being `core` or `alloc` as the cause. That matches `Crates::new` (`crates.rs:66-70`): an injected std that is `core`, `alloc`, or Rust's std is cleared to `None`.
- "The check rejects `#![no_std]` before this runs" is true. `after_analysis` calls `write` only when `self.tide_std` is set and `has_errors()` is none (`invocation.rs:63-65`), and `check_expanded`'s FX0520 has the same `tide_std` gate.
- `crate_pure` also returns `None` when `--tide-std` is some crate without `tide_std::marker::Pure` (`walk.rs:157-162`). The doc says "as when", so it reads as an example and is still accurate.

**B2 is resolved.**
- `pub static CELL` and `GLOBAL-1` are gone from `admit_no_std.rs`. GLOBAL-1 still has its own cases (`global_static_{accept,cell,mut,shared}.rs`).
- The case is now `#![cfg_attr(all(), no_std)] //~ FX0520` plus `extern crate std;`. It pins only the rejection, with no hole symptoms.
- The plain `#![no_std]` + `extern crate std;` form, which the Acceptance names, is still in the corpus at `root_main_no_std.rs:5-6`, marked FX0520.
- The new comment is accurate. Under `no_std`, `injected_std` finds no injected `extern crate std`, so `tide_std` is `None`.

**N2, N3 and N4 are resolved.**
- `invocation.rs:512-515` now wraps at 96 columns or less, and `README.md:56-59` is reflowed.
- `invocation.rs:557-560`, `03:561-564` and the case comment now give the fuller reason: rustc injects `core`, so the `Pure` rules are off, and `extern crate std` loads Tide's std only as an ordinary crate.

**N1: the rebuttal's outcome holds, but its stated reason does not fully hold.** Round 1 named rewording as the minimum fix, and the new comment (`driver.rs:644-645`) is accurate. The step is also now worth keeping for a new reason: the inactive `#![cfg_attr(any(), no_std)]` check (`driver.rs:650-653`) is the only proof that the guard has no false positive. Two parts of the rebuttal's reasoning are wrong, though:
- **"Only coverage of a core-only `#![no_std]`" is not right.** `admit_crate_identity.rs:5-6` is already a core-only `#![no_std]` (with `extern crate core as std;`, and no Tide std loaded), marked FX0520 on the attribute. The guard reads only `krate.attrs`, so having no `extern crate` at all does not reach a different path.
- **"A failed check leaves no artifact for this cause" is not distinct coverage.** The artifact gate is a single `has_errors().is_none()` check that does not depend on the cause, and the `build.rs` step (`driver.rs:636-642`) already proves it.

Nothing needs to change. See N-b.

**No fix adds a defect, and none changes behaviour beyond its finding or crosses a trust boundary.**
- The production changes (`invocation.rs`, `interface.rs`) are comments only. The guard code, the gate and the message are unchanged.
- The test and corpus changes only remove the pinned symptom and add the active and inactive `cfg_attr` coverage that Codex suggested. That coverage exercises the existing guard and does not widen it.

Checks: read-only inspection only (`br show --no-db`, `git diff bd42674 aa4106c`, `git show`/`git grep` on aa4106c, and line-length counts). Nothing was built or run. I relied on the recorded `cargo test --workspace` exit 0 and `conformance/run.sh` exit 0 (194 PASS) on aa4106c.

## Findings

**N-a. Uneven reflow left by the B1 edits.** Text-only; does not block.
- **What is wrong:** removing the B1 clauses left short lines in the middle of paragraphs:
  - `README.md:60` is `` `<crate>.interface.json` `` on its own (24 columns).
  - `README.md:65` is "is neither Tide's nor Rust's fails the check; one" (49 columns).
  - `crates/tide-admit/src/interface.rs:5-6` are 61 and 51 columns, between lines of 94.
- **Fix:** reflow `README.md:58-67` to the paragraph's width of about 78 columns, and `interface.rs:1-6` to about 94.

**N-b. The N1 rebuttal's premise is partly wrong (record only).** Text-only; does not block.
- **Where:** the author's note on N1.
- **What is wrong:** the note gives the driver step at `crates/tide-admit/tests/driver.rs:644-649` as "the only coverage of a core-only `#![no_std]` with no `extern crate std`". `conformance/cases/authority/admit_crate_identity.rs:5` already covers that route. The step's real distinct value is the message text and the inactive-`cfg_attr` admission it now hosts.
- **Fix:** no code change. If the rebuttal is recorded on the bead, give that reason for keeping the step instead.
