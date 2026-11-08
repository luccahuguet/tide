# Tide review, fx-aw6 round 1 (Opus): review round

Candidate: commit e03b2af on branch fx-aw6 (`git diff d12d7e0 e03b2af`; docs and conformance cases only, no driver change). The worktree's tracked tree matched e03b2af at the start and at the end; the untracked `DECISION.md` and `CODEX-REPORT.md` are scratch and outside the candidate.

Model: claude-opus-5-5 · Date: 2026-11-07

## Verdict

Not clean, by one small text-only blocker. The change implements the Opus decision faithfully and meets the bead's Acceptance:

- The rule is stated in its owner, D3 (`design/01-design.md:318-323`), with its reason. GLOBAL-4 (`design/03-authority-concurrency.md:398-408`) elaborates it without restating the reason.
- Reference 4.2 (`design/reference.md:262-263`), reference 11 (`:958`), and the GLOBAL-4 row of `conformance/rules.md:99` agree with D3 and GLOBAL-4. Every edit matches DECISION.md's instructions word for word, the companion count included.
- The verdict the driver does not emit yet (`FX0301`) is a case that AWAITS CHECKER (`address_casts_lossy.rs`), not a false PASS. The driver still admits that file under the grant. The granted and ungranted cases match what the driver does today.
- The reference is within budget: 24,977 tokens (see Proof).

I think the decision itself is sound (see "Decision soundness"). The blocker is the one spelling GLOBAL-4 gives for a signed integer. It is wrong for most signed widths.

## Findings

B1 (text-only, blocks). The spelling GLOBAL-4 gives for a signed integer-to-pointer cast is wrong for every signed type except `isize`.

- **Where:** `design/03-authority-concurrency.md:400-403`. The text says "`n as *const T` for a `u64`, `u128`, or signed `n` are `FX0301`, spelled with `addr()` and D3's conversions (`u32::try_from(p.addr())`, `n.cast_unsigned() as *const T`)". That presents `n.cast_unsigned() as *const T` as the spelling for any signed `n`.
- **Failure 1, wider types:** for `n: i64` or `i128`, `n.cast_unsigned()` gives a `u64` or `u128`. Casting that to a pointer is `FX0301` again, so the spelling the text recommends is rejected.
- **Failure 2, narrower types:** for `n: i8`, `i16`, or `i32`, it is accepted but means something different. Rust's `n as *const T` sign-extends: `-4096i32 as *const u8` has address `0xFFFF_FFFF_FFFF_F000` on a 64-bit target. `(-4096i32).cast_unsigned() as *const u8` zero-extends instead, to `0x0000_0000_FFFF_F000`. An agent porting Rust code by following the text gets a different address, silently, and the address goes into an unsafe dereference.
- **Correct spelling:** for `isize`, `n.cast_unsigned()` is the same as Rust's cast. For a narrower signed `n`, the spelling that keeps Rust's meaning is `(n as isize).cast_unsigned() as *const T`; `i32 as isize` is lossless under 4.2. For a wider one, `usize::try_from(n)` or a checked conversion is needed.
- The case file (`address_casts_lossy.rs:36`) uses `s: isize` and is correct. The reference does not repeat the spelling.
- **Fix:** make the example `isize`-specific, e.g. "(`u32::try_from(p.addr())`, `n.cast_unsigned() as *const T` for an `isize` `n`)". Optionally add "a narrower signed `n` widens to `isize` first". The change is to 03 only, so it costs no reference tokens.
- **Why it blocks:** GLOBAL-4 is v0 core and this change edits it.

N1 (nit, text-only). The trim deletes a disambiguation, not only a duplicate.

- **Where:** `design/reference.md:443-444`.
- **What was lost:** the deleted sentence was "Creating an `Arc` takes `&Tasks` (7.4); creating an `Rc` or `Cell` does not". Its first half does duplicate section 1 (`:36-37`) and 7.4 (`:723`). Its second half was the one place that says `std::cell::Cell::new` needs no `&Tasks`.
- **The risk:** 7.4 opens "Cells, channel ends, `Arc`, ... every constructor takes `&Tasks`", where "cells" means `Shared`. Section 1's "keep their Rust API" for `Cell` still implies the right answer, so the risk is small: a reader may misread 7.4's "Cells" as `Cell`.
- **Possible fixes:**
  - If a reword stays within the 23-token margin, write "Shared cells" in 7.4.
  - Otherwise accept the trim. It was the decision's deliberate budget call.

N2 (nit, text-only). D3 records the reason but none of the rejected alternatives.

- **Where:** `design/01-design.md:318-323`.
- **What was dropped:** the decision rejected three options:
  - exempting address casts from D3 as trust holes (the Codex memo's recommendation);
  - requiring named conversions only;
  - requiring strict provenance only.
  None of these reached the owning document. They survive only in the bead comment.
- **Why it matters:** the first rejection constrains future work. Whoever implements `FX0301` (fx-u65) may be tempted to exempt pointer casts. The repo-local Decisions rule asks for the decision and its reasons in the owning document.
- **Fix:** add one clause to D3's `Rejected:` line, e.g. "exempting address casts from the lossless rule". That is cheap and is in 01, not the reference.

N3 (nit, text-only). GLOBAL-4's examples of accepted casts leave out two Rust forms that the rule accepts.

- **Where:** `design/03-authority-concurrency.md:398-406`.
- **The forms:**
  - a function pointer cast to a raw pointer (`f as *const ()`, common in FFI);
  - a function *item* cast to an integer (`noop as usize`; `global_fn_pointer.rs:14` gates it, and the driver keys it as `fn() as integer` via `ty::FnDef`).
- **Why it is only a nit:** "an address counts as a `usize`" decides both: the first is accepted and the second follows the `usize` table. The text names only raw-pointer-to-raw-pointer and reference-to-raw-pointer casts.
- **Fix:** none needed. If GLOBAL-4 is touched for B1, "a cast between pointer types (raw or function)" would cover the first form.

N4 (nit, out of scope, report only). DECISION.md's out-of-scope note says `FX0301` "has no bead". fx-u65 ("Enforce every FX rule the reference names in tide-admit") covers it, since reference 4.2 names `FX0301`. No new bead is needed. The fx-kw8.23 budget overage the decision found is real: the base measured 25,019 tokens in my run too.

No fx-djp comments: every finding concerns v0-core text this change edits.

## Checks against the brief

1. **Faithful, and meets Acceptance.** Every edit matches DECISION.md: the D3 sentence, the GLOBAL-4 paragraph, the four reference edits, the companion count, the rules.md row with the `a3b0177` citations kept, and the three cases. The memo's line-24 and 4.2-heading edits were correctly not applied. The Acceptance requires "stated in its owning document with reasons" (D3), "reference and rules.md agree" (yes), and "run.sh exits 0" (see Proof).
2. **Consistency with the driver.** Without the grant, the driver gates raw pointer types wherever a type names them (`walk.rs:472`, `policy.rs:73-75`) and gates function-pointer integer casts (`walk.rs:290-315`, `policy.rs:84`). So `global_pointer_addr.rs`'s new `rebuild` (`n as *const u8`) is truly FX0520. Under the grant the driver admits every form in both new files. The driver implements no `FX0301` (`grep` finds none in `crates/`), and `run.sh:85,134` makes the lossy file AWAITS CHECKER rather than PASS.
3. **Over-promising and trust model.**
   - The new text promises nothing about provenance or dereferenceability (GLOBAL-4's last sentence), and it moves no trust boundary: the grant gating is unchanged.
   - Reference 6.9 and 11 no longer name `addr()`. That loses nothing: `addr` needs a raw pointer, whose type is gated.
   - The 4.2 reading relies on "a wider integer of the same signedness" being non-strict, so `p as usize` is `usize as usize`. The contrast with "strictly wider signed" makes that the intended reading.
   - I confirmed with the stable rustc that an unsuffixed literal in `0x4000_0000 as *mut u32` is typed `usize`, so the common MMIO idiom stays accepted and is not read as a signed `i32`.
   - Leftovers: none found, beyond N1's lost clause.
4. **Budget.** `scripts/count-tokens.sh design/reference.md` gives 24,977, under 25,000 and equal to the companion's line 14. The margin is 23 tokens.
5. **Decision soundness.** I agree with it.
   - Applying D3 by definition ("address = `usize`") adds no exception and reuses 4.2's widths and signedness. Truncating an address is the same silent-loss bug D3 exists to stop.
   - The `unsafe` grant is about observing addresses and running unsafe code, not about intent to truncate.
   - Every rejected form has a spelling that exists today (`addr()`, `TryFrom`, `truncate()`, `cast_unsigned()` for `isize`). The dominant Rust idioms (`p as usize`, `n as *const T` with `n: usize`, unsuffixed literals) are unaffected.
   - The memo's option 1 calls a truncating cast a trust hole. That is a category error: a cast breaks no checker guarantee.
   - The one cost is that 64-bit-only low-level code must spell `u64`-to-pointer as `usize::try_from(n)`. That follows from 4.2's provisional "usize is 32 to 64 bits", which this change does not alter.

## Proof

- Revision e03b2af, x86_64 Linux. I checked a `git archive` copy under the scratchpad, with the worktree's build caches copied in, so the worktree was not touched.
- Command: `flock /home/lucca/.cache/eon-agents/eon-overseer/build.lock nice -n 19 env CARGO_BUILD_JOBS=6 conformance/run.sh </dev/null`. Result: exit 0, with 195 PASS, 25 AWAITS CHECKER, and 0 FAIL (220 cases). `address_casts_granted.rs` is PASS, `address_casts_lossy.rs` is AWAITS CHECKER for FX0301, and `global_pointer_addr.rs` is PASS. These are the counts the decision expected.
- Worktree state: HEAD stayed e03b2af with no tracked changes. The untracked files are `DECISION.md`, `CODEX-REPORT.md`, and, by the end, the parallel Codex pass's `reviews/fx-aw6-r1-codex.md`.
- `scripts/count-tokens.sh design/reference.md`, run in the worktree, gave 24977 (exit 0). The same script on `git show d12d7e0:design/reference.md` gave 25019.
- `git diff --check d12d7e0 e03b2af` exited 0.
- I did not run `cargo test --workspace`: the candidate changes no crate source, and conformance exercises the driver at the highest boundary the change touches.
