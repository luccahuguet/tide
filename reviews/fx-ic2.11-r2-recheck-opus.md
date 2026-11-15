Model: claude-opus-5-5 · Date: 2026-11-15

# fx-ic2.11 Opus re-check of b50fdaa..c7f7950

Scope: only the new content. `git diff b50fdaa c7f7950` touches only
`trials/e1-checked/gate-report.md`; `gate_reproduce.py` is unchanged.

## Verdict

**approve**. All three items check out. Two nits are text-only and need no
re-check.

## 1. Sampled-regrade result block (gate-report.md:574-587)

**The block was run verbatim.**

- `ic2.11-regrade-block.sh` is byte-identical (`diff` empty) to the report's
  command block at b50fdaa.
- The runner `ic2.11-regrade-run.sh` adds three things around it:
  - a check that quiet-until is the 16:45Z grant;
  - a check that the start falls in 16:30:00-16:31:00Z;
  - an outer `timeout` that ends 30 s before the grant.
- It runs from the fx-ic2.11 worktree with `QUIET_LOG=$T/ic2.11-quiet.log`.

**Run log (`ic2.11-regrade.log`):**

- start 16:30:05Z;
- every preflight check passes, including:
  - load 2.89;
  - the E1 grant at 16:45:00Z;
  - QUIET OVER at 15:07:49Z (the quiet log holds exactly that line);
  - the gate worktree at R and clean;
  - both canaries and the probe current;
  - store scan exit 0;
- `PREFLIGHT PASS`;
- heavy.sh grants `build.lock`;
- all 12 slots print "first and final match";
- end 16:33:33Z, rc=0.

**heavy.log** agrees for pid 2087433: queued and granted `build.lock` at
16:31:06Z, `exit:0` at 16:33:33Z.

**matches.json:**

- SHA-256 is `3355d86fa9c31ef2eb20d85e8ea5e9826bbbaef3f76459c2efb28bf2e0eeff79`,
  as the report states.
- It has 12 entries, [5, 9, 26, 28, 30, 31, 39, 48, 60, 65, 93, 97], each with
  `first: true` and `final: true`.
- The triples match the report's sample table.

**Independent re-check.**

- I ran `signature()` on all 24 regraded grade files against a fresh unpack
  of the archive. All 24 match.
- All 24 differ byte-wise from the stored files. The harness rewrote them, so
  none is a stale copy.
- G is still clean at R after the run.

**Nit (text only).** "Wrapped only in an outer timeout" understates the
runner, which also aborted on a wrong quiet-until or a start outside the
window. Those checks are guards that change nothing in the command, so the
claim's substance holds.

## 2. N1, the summary (gate-report.md:8-16)

Resolved and accurate:

- first check 59/60 vs 56/60;
- final pass tied at 58/60;
- 1.46 times the iterations against a 1.5 limit;
- near ceiling;
- the interval caveat is kept;
- the regrade sentence is true.

## 3. N2, the resemblance read-in (gate-report.md:386-391)

The numbers match my own computation from `per-trial.json`:

| Task group | F (Tide vs Rust) | P | I (Tide vs Rust) | I ratio |
| --- | --- | --- | --- | --- |
| Non-resembling | 37/40 vs 40/40 (0.925) | 38/40 each | 56/38 vs 38/38 | 1.474 |
| Resembling | 19/20 each | 20/20 each | 30/20 vs 21/20 | 1.429 |

**Nit (text only).** "With the same narrow iteration margin" is slightly
loose. 1.474 against 1.5 is narrower than the overall 1.458. "An even
narrower iteration margin" would be exact. The long source line at :391 could
also be rewrapped.

## Rulings noted

U1-U4 were handled before the run, and the log confirms it. N3-N5 are
deferred, and that is acceptable:

- the fresh output directory plus the exit-code check guard against stale
  grades, and my rewrite check above confirms no stale grade was used;
- the helper never writes `run/`;
- the regrade passed under 3.12.3, as the report now records.

DONE
