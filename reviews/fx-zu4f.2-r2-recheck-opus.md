Model: claude-opus-5-5 · Date: 2026-11-17 · Candidate 7a44e54 (re-check of 7632f00 findings)

# fx-zu4f.2: finder re-check of round 2 (Opus)

Scope: only my round-2 findings, checked in `git diff 7632f00 7a44e54`. I ran no builds and no new
full review. The diff touches only `examples/case-studies/eon-slow-close/README.md` and comment
lines in `open_start.rs`. No `//@` declaration, code line or runner line changed, so the earlier
test proof still covers the behaviour.

## Verdict

**approve**. B1 is resolved. Non-blocking items 1–4 are resolved. Items 5–8 were optional and were
not taken, which is acceptable. The edits introduce no new overclaim and break nothing.

## Findings

### Blocking

- **B1 (15 s start timeout cited as Eon's, but on `edge` only): resolved.**
  - README.md:38-41 now cites the 15 s `TERMINAL_START_TIMEOUT` to Eon's `edge` branch (commit
    `283bf76a`; eon-pimct cites it at `dcaef1c`). It also says the v0.1.16 release on `main` waits
    up to 5 s.
  - README.md:71-76 says the test uses `edge`'s bound. It adds that under `main`'s 5 s bound one
    held start stays under Venus's 6 s, and that this is why eon-pimct names several queued starts.
  - The source paragraph (README.md:144-145) lists the commits by branch. The `open_start.rs`
    header and the `START_TIMEOUT` doc comment say `edge` and `283bf76a`, with `main` at 5 s.
  - I checked each fact against Eon on the first pass:
    - `283bf76a` and `dcaef1c` are on `origin/edge` and not on `main`;
    - `main` is `d41f99ea` ("Release v0.1.16") with `SESSION_START_TIMEOUT = 5 s`;
    - eon-pimct says the original bound was 5 s and that "several queued starts can stack up past
      Venus's 6 s timeout".
  - The "which is why" link is a fair reading of that sentence.

### Non-blocking

1. **The opening paragraph's "What Tide adds", and tokio: resolved.** It now reads "is
   deterministic, and needs no fake clock written for it". It adds that async Rust gets the same
   from tokio's paused clock, and that Eon's thread-based supervisor would need a clock it is
   handed. This matches research/33 §5's pairing, and the "every seed" wording is gone. README.md:115
   ("deterministically") matches too.
2. **"A second wait cannot start a fresh five seconds": resolved.** It is now limited to "while it
   sits inside that scope". The README adds that a second scope opened afterwards gets its own five
   seconds, which matches reference §7.2, where `set_deadline` lowers and nested scopes take the
   earliest. Record 04 is now "a defect of this kind", so the sentence no longer implies Tide would
   have prevented it.
3. **"Keep the unended by construction": resolved.** It is now "Deliver cancellation as an ordinary
   error", and it credits "the fix's filter" with keeping the terminals. Cancellation reaching every
   wait as an `Err` is a Tide property (§7.2).
4. **"Waits the full 15 s": resolved.** It now reads "which would hold the loop up to 15 s",
   describing the input, not the run.
5. **Unused close path in `open_start.rs`: not taken.** This was optional. The README's framing as
   "fixed.rs plus starts" justifies keeping it.
6. **Dead keep branch in `buggy.rs`: not taken.** This was optional. It mirrors Eon's "keeps any it
   could not confirm".
7. **Runner test lacks an "`examples/` holds no example" case: not taken.** This was optional,
   since the rule is equivalent to main's.
8. **Scope note: no file change was expected.** The landing record should cite tide-lead's
   2026-11-17 21:08 comment for the runner change.

## Newly introduced issues

None that blocks.
- **Nit:** README.md:115-117 now has a short ragged line ("…fails at" / "3.78 s on the serial
  loop."). This is formatting only.
- **Wording, checked and acceptable:** "Eon's thread-based supervisor would need a clock it is
  handed" sits next to a Tide model whose `serve` also takes a `&Clock`. The preceding "needs no
  fake clock written for it" makes the intended difference clear: Tide's std supplies the simulated
  clock, while Rust needs one written.

DONE
