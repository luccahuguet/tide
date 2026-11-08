# Tide review, fx-aw6 round 2 (Opus): confirmation round

Candidate: commit 351d9a3 on branch fx-aw6, fix delta `git diff ca01fe0 351d9a3` (one hunk, `design/03-authority-concurrency.md` GLOBAL-4). The rebased change is ca01fe0 on main's 8c9a38a.

Model: claude-opus-5-5 · Date: 2026-11-07

## Verdict

Clean. The fix resolves round 1's blocker B1 (Opus) and Codex's blocking item 1. The rebase carries the same change, and only the companion count differs. The reference counts 24,952 tokens, under 25,000. No blockers. One nit, text-only, which needs no re-check.

## Findings

### The fix resolves B1

- GLOBAL-4 now reads "`u32::try_from(p.addr())`, `usize::try_from(n)? as *const T`, or `n.cast_unsigned() as *const T` for `n: isize`". The general spelling for a rejected `n` (`u64`, `u128`, or any signed width) is now the checked conversion. `cast_unsigned` is limited to `isize`, where it gives a `usize` and matches Rust's `isize as *const T` bit for bit. The two failures round 1 named are gone: `i64` and `i128` no longer go through a rejected `u64` or `u128` cast, and `i8` to `i32` no longer zero-extend silently.
- Precedence: postfix `?` binds tighter than `as`, so `usize::try_from(n)? as *const T` casts a `usize`. Under the lossless rule that is a `usize` to address cast, so it is accepted.
- Checked with stable rustc (edition 2024, scratchpad copy, worktree untouched). `Ok(usize::try_from(n)? as *const u8)` compiles in a `Result<_, TryFromIntError>` function for `n` of type `i8`, `i16`, `i32`, `i64`, `i128`, `isize`, `u8`, `u16`, `u32`, `u64`, `u128`, and `usize`. For `u8` and `u16` the error type is `Infallible`, and `TryFromIntError: From<Infallible>` covers it. Those two are accepted without a conversion anyway. `(-1isize).cast_unsigned() as *const u8` has address `usize::MAX`. `u32::try_from(p.addr())` compiles and gives `Ok(16)` for address 16. `usize::try_from(-1i32)` and `usize::try_from(u128::MAX)` are `Err`. `u32::try_from(p.addr())` is not a cast, so the lossless rule does not apply to it. `p.addr()` is the `usize` that GLOBAL-4 admits under the grant.

### The rebase is faithful

- `git range-diff d12d7e0..e03b2af 8c9a38a..ca01fe0` shows two differences:
  - the companion count line (24,977 becomes 24,952 against main's 24,994);
  - one context line of reference 11's Unsafe code row, which fx-xrx reworded. fx-aw6 does not edit that row.
- The two diffstats are identical (8 files, +83 −11). `git diff --check 8c9a38a 351d9a3` exits 0.

### fx-xrx and fx-aw6 read coherently together

- `reference.md`:
  - 2.1 (fx-xrx) says "The `unsafe` grant (11) admits `asm!`, `global_asm!`, and `naked_asm!`".
  - 4.2 (fx-aw6) says "An address (11) counts as a `usize`".
  - 6.9 (fx-aw6) says "addresses without the `unsafe` grant (11)".
  - Section 11 has fx-xrx's shortened Unsafe code row and fx-aw6's Addresses row.
  - The rows use the same "`unsafe` grant (`FX0520` without it)" phrasing, and every pointer to 11 lands on that table.
- `01-design.md`: fx-aw6's D3 sentence (around line 319) and fx-xrx's admission-list edit (around line 431) are in separate bullets. They do not overlap or conflict.
- `03`: the GLOBAL-4 addition (around line 398) and fx-xrx's ADMIT-1, UNSAFE-1, RUN-2, and RUN-4 edits are independent. ADMIT-1's "every address observer (GLOBAL-4)" still points at the right rule.
- `rules.md`: the GLOBAL-4 row (fx-aw6) and the UNSAFE-1 row (fx-xrx) sit side by side and agree.
- Line citations such as `03:310` and `ref:956` are taken at `a3b0177`, as the file's header says. They are not current line numbers, so the shifts from both changes do not stale them.

### The reference is within budget

`scripts/count-tokens.sh design/reference.md`, run in the worktree at 351d9a3, gives 24952, under 25,000. That equals the companion's line 14. The margin is 48 tokens.

### N1 (nit, text-only, does not block)

`usize::try_from(n)?` rejects a negative narrow signed `n` (`i8` to `i64`) where Rust's `n as *const T` sign-extends. This is a loud failure, not a silent one, so it is safe. Code that relies on sign extension would need `(n as isize).cast_unsigned() as *const T`, and GLOBAL-4 does not say so. That case is rare. The `?` form also assumes the function returns a `Result` whose error converts from `TryFromIntError`. In an `Option` function the spelling is `.ok()?`, which is obvious Rust. Neither needs a fix.

## Proof

- Revision 351d9a3 (fix delta `ca01fe0..351d9a3`), x86_64 Linux, worktree `/home/lucca/pjs/lab/tide/.worktrees/fx-aw6`. No tracked changes. The untracked files are the scratch `DECISION.md`, `CODEX-REPORT.md`, and the two round-1 reviews.
- `scripts/count-tokens.sh design/reference.md` gave 24952.
- `git range-diff d12d7e0..e03b2af 8c9a38a..ca01fe0`: only the differences listed above.
- `git diff --check 8c9a38a 351d9a3` exited 0.
- Spelling check with `rustc +stable --edition 2024` on a scratchpad file: it compiled, and the run printed `true true Ok(16) true`.
- I did not run conformance, since it is running elsewhere. The fix changes no case or crate.
