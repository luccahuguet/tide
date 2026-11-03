# Tide review, fx-k0z round 2 (Opus): confirmation round, report only

Candidate: commit 9de1696 on branch fx-k0z (worktree `.worktrees/fx-k0z`).
The fix diff since round 1's candidate is
`git diff 0939683 9de1696 -- . ':!reviews'`, SHA-256
`84f7682e8ed6894822254d2e00f573e34fb451582a15bda5c3f2b968341ca33c`. The
whole change is `git diff main...9de1696`, SHA-256
`4663980e2d2d7d7462b820d86d11d506d5897d0ef5fa962e3c66047a1a63fd6e`. This
round confirms the fixes for round 1's findings
(`reviews/fx-k0z-r1-opus.md`, `reviews/fx-k0z-r1-codex.md`). It reports and
does not fix. `git diff --check 0939683 9de1696` passes.

## Verdict

**approve**. Confidence: high (about 0.9).

No blocking findings. Codex's blocking finding is fixed, and nothing in
`design/` or `conformance/rules.md` contradicts the fix. The core measures
24,995 tokens, which I re-ran. Both trims lose nothing an agent needs to
follow a rule. The word diff confirms that the rewraps changed no words.
Three non-blocking notes follow.

## What was reviewed

- Brief: GOVERNANCE.md "Merge Rule" (tier rule), and both round 1 reviews.
- The fix diff (5 files outside `reviews/`), in the context of the whole
  change. Surrounding text: reference 0, 2.1, 6.9, 11, 12.3, 12.4, 13;
  03 §3.1, ADMIT-1, UNSAFE-1, RUN-2, RUN-4; 01 D7 and the generator bullet
  near 01:400; companion 14.1, 14.2, 19; the stub `std::time` module; the
  `unsafe_global_asm.rs` and `unsafe_granted.rs` cases.
- Sweeps over `design/` and `conformance/` for `asm!`, `tide run`, `--bin`,
  `generator`, `12.3`, and `duration_since`.
- `git diff --word-diff=plain 0939683 9de1696 -- design conformance`. The
  only word changes are the ones listed below. Every other changed line
  (01:430-432, 03 GLOBAL-4, reference 6.2 and 6.5) is a rewrap.
- I did not rerun `conformance/run.sh`. The only change under
  `conformance/` is in a doc comment (below).

## Checks

| Check | Result |
| --- | --- |
| Blocking finding (Codex) | Fixed. Reference 2.1:59-60 now says "Under the `unsafe` grant (11), `asm!` and `global_asm!` are admitted (`FX0520` without it)". This sits in the admitted paragraph, before the catch-all. The `FX0103` paragraph no longer names them, so "Any other `!` form" cannot reach them. This agrees with 03 ADMIT-1:557-559, UNSAFE-1:707, RUN-4:821, 01 D7:430-431, `rules.md` UNSAFE-1, `unsafe_granted.rs:31,34`, and `unsafe_global_asm.rs`. A sweep found no remaining text that makes granted `asm!` an error. Reference 6.9's unqualified "hardware intrinsics (`core::arch`)" is still broader than 2.1, but the more specific rule wins (round 1 N3) |
| Section 0 trim | Nothing needed is lost. "Std import paths are in 12.3." was a pointer to a heading, "12.3 Prelude and std paths", which an agent reaches from the outline. 2.2 (reference:74) still cites 12.3 for the prelude |
| 12.4 trim | Nothing needed for the rule is lost. 2.1:66-67 says "run a generator program and commit its `.rs` output". Companion 14.1:66 and 01:400 say the same. `tide run --bin gen` was the only place in the core that named `tide run`, and the only place in `design/` that named a `--bin` flag. That flag was an unbacked guarantee, and dropping it narrows the contract (AP-CONTRACT-001). The core no longer names how to run a program at all; see N1 |
| Budget | `scripts/count-tokens.sh design/reference.md` gives 24,995, at or under 25,000. The companion's "Core count" line says 24,995 |
| 03 §3.1 (round 1 N4) | Fixed. PURE-1 is cited once, at 03:832. The new sentence, "The rule is deep, through every field, private ones included", is accurate against 02 PURE-1. It still ends "§9 has the full table", though §9 is a list (round 1 N7 nit; unchanged, style only) |
| Companion `t.elapsed()` row (round 1 N1) | Correct. In Rust, `Instant::duration_since(earlier)` returns `Duration`, and `SystemTime::duration_since(earlier)` returns `Result<Duration, SystemTimeError>`. That matches what `elapsed()` returns on each type, so each replacement keeps its type. The stub declares `Clock::now`, `Clock::system_time`, and `Instant::duration_since` (the last marked GUESS). See N2 for `SystemTime` |
| Rewraps | Word-preserving. The word diff shows changes only at the stub comment, 03 §3.1, the companion count and row, reference 0, 2.1, and 12.4 |
| Stub change | Comment only. `conformance/rustc/std/src/lib.rs:386`, `///` doc text "no `now()` and no `elapsed()`" became "no `now()` or `elapsed()`". The doc comment adds no item or attribute, so the conformance result on 0939683 (exit 0) still applies |

## Blocking findings

None.

## Non-blocking notes

**N1. The core does not name `tide run`.** Section 13 lists `check`, `fix`,
`test`, `sim`, `eval`, `explain`, `review`, and `fmt`. The trimmed 12.4
clause was the only place in the core that named the command which runs a
program. It still appears in 03:800 and companion 19:527, and a Cargo user
would guess it, so no rule is lost and nothing contradicts it. If a later
trim frees space, add `` `tide run [--] args` runs `main` with real
authority `` to section 13. That fix would edit the core, but the gap is an
omission in a command list, not a defect the change introduced, so I do
not block on it.

**N2. `SystemTime::duration_since` is not shown to be admitted.** The
companion row relies on it. The stub's `SystemTime` has no methods, and
its doc comment names only the two methods it lacks. The allowlist is not
spelled out in the core, and the stub already marks its method choices as
GUESS, so the row is a reasonable reading, not a contradiction. When the
allowlist is written down, it should list `SystemTime::duration_since`
next to `Instant::duration_since`. This goes on `fx-djp` or the allowlist
work, not this change.

**N3. Reference 11's hole table has no `asm!` row.** 2.1 now points to 11
for the grant, and 03 UNSAFE-1 calls `asm!` and `global_asm!` trust holes.
The table's "Unsafe code" row covers `asm!` through its required
`unsafe { .. }` block, but not `global_asm!`. The text is consistent, since
the table does not claim to be complete. This predates the change. Leave
it until there is budget.

## Minimal patch for approval

None required.
