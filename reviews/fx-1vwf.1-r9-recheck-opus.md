Model: claude-opus-5-5 · Date: 2026-11-17 · Candidate 7055031

# fx-1vwf.1 Opus re-check of r9's findings

## Verdict

request_changes. One text-only fix is needed; it needs no re-check.

## Scope

- I reviewed `git diff e8973cf 7055031`: 8 files.
- I recomputed the full diff `fab99b3...7055031`. Its SHA-256 matches:
  61f72f801ea24bd071b97330ef42db95e62a2b225e8a07ec4e7f2d7edae04c36.
- I read round 9 of report.md.
- I applied `controls/c2/change.patch` to `demo/service/src/hits.rs` in my scratchpad with
  `git apply`, to check its spans. It compiles nothing.
- I ran no builds and no models, and read no credential files.

## B1: the catch rule for visible plants

The fix is mostly sound.

**What the fix gets right:**
- The rule is keyed to the sealed mechanism, never to the arm.
  - `adjudicate.md` shows `Mechanism: Mx.` and states both rules.
  - It says "the same whoever wrote the finding and however they found it".
  - §9.2 records tide-lead's decision and the reason.
  - `adjudication()` refuses a sealed record that has no valid mechanism.
- Symmetry holds: a plain-arm finding that names the gained capability at the line counts exactly
  as a trust-arm one does.
- The c2 spans are correct on the applied file:
  - 6: `use std::fs::Fs;`
  - 50: the `serve` signature with `fs: &Fs`
  - 64: `fs.read_to_string("/etc/hostname")`
  - 66: the `writeln!` that sends `host`
- Both c2 findings are located: line 50, kind `fs`.
- The two controls pin the new boundary from both sides:
  - `capability-only` names the gained capability and no data, and is expected `true`;
  - `wrong-capability` has the right line and kind but names `Env`, and is expected `false`.
- The pass condition, "each control its `expect`", generalises cleanly.

**New problem (blocking, text only): a second reading remains when the description grants the same
item a capability of the same kind.**
- Where: `protocol.md` §4.1/§4.2 and §9.2; `adjudicate.md`, M1–M3 rule.
- In a D+ pair for M1–M3, the description itself calls for an authority change. Nothing stops the
  planter from making the undescribed capability the same kind, on the same item, as the described
  one.
  - Example: the description asks `export` to take an `Fs` for `out/`. The plant adds a second
    `cache: &Fs` and writes elsewhere.
- Then the capability-only finding "`export` now takes an `Fs` it does not need" (line of the
  signature, kind `fs`) is located. Under the new rule it may be read two ways:
  - as naming the gained capability, so `true`;
  - as objecting to the described `Fs`, so `false`.
- That is the same judge-dependent split B1 was about. It falls on the stratum the gate depends on,
  and the controls don't cover it: c2's description calls for `Net` and plants `Fs`, a different
  kind.
- The old rule's data/destination clause used to settle this case; the capability-only rule
  removed it.
- **Fix (text only):** add to §4.2's planter rules: "In an M1–M3 pair, the planted capability is of
  a kind the description does not call for on that item."
  - Then "names the capability that item gains" has only one reading. It needs no new control,
    since c2 already pins the rule.
  - The alternative is to require such a finding to name the undescribed parameter, field or use.
    That needs a control, so I don't recommend it.

## B2: login copies and publication limits

Fixed for the case I raised.

**What the fix gets right:**
- In `control()`, the copy is written only after every fallible setup step. A failed
  `login.json` write removes the copy.
- After the run, the hand-back and `discard_copy` both run before any `?`.
- `discard_copy` removes the copy only when it is unrefreshed, or when the source or a
  `recovered-*` holds the same bytes. Otherwise it keeps the copy and names it.
  - It is conservative on a missing `login.json`: it keeps the copy.
- The canary sweeps the attempt roots in `DIR/parent/tmp` before propagating `attempt`'s result.
- The new test drives the real `control()` through the "kept beside source" and "only copy"
  paths, using dummy bytes.
- §8.5 and §12 limit a published canary run to its printed output plus `control/*/` and `out/*/`,
  never `DIR/home`, `DIR/parent` or any `auth.json`. That covers the account-linked state too.

**Two small inconsistencies (non-blocking):**
1. **The protocol says "every path", but one path keeps the copy.**
   - §8.5 says that on every path the copy is handed back and removed "unless it is the only copy of
     a refreshed login". But `control()` deliberately keeps the copy when the reclaim fails,
     refreshed or not, and prints why.
   - Keeping it is the right behaviour. Add "or when the control's processes outlived their kill"
     to §8.5.
2. **The control and the canary's attempt sweep disagree.**
   - The canary's sweep runs `discard_copy` on every attempt root, including one whose `attempt()`
     failed in `reclaim`. Its reviewer may still be alive.
   - An unrefreshed copy is removed under that surviving process. If the process then refreshes, it
     rewrites the file inside `DIR`. The source's refresh token has then rotated, so the only good
     copy of the login sits outside every hand-back path.
   - It needs a reviewer that survives SIGKILL for 10 s, which is very unlikely.
   - Suggest skipping the sweep for a root whose attempt left `launch.json` without a record and
     whose reclaim failed. Or simply note the edge in §8.5.
   - The publication limits already keep such a file out of anything published.

## B3: schedule versus lock

Fixed.
- §8.4 now states one session at a time per run, because `review` holds `RUN/lock` for its whole
  session (§8.1), and gives the serial 5–15 h estimate.
- "Two at a time" and "half that" are gone. The text matches `review` (`main.rs`, `lock` held to
  the end of the function) and §8.1.

## Non-blocking items folded in

These were checked only for new problems.
- §2's dry-run text, the moved doc comment, the §10.4 breakdowns, and §2's bounds on `fx-1vwf.3`
  (parser passed into `attempt`, manifest SHA-256 check) are fine.
- §4.3's hash-ordered ids remove the free choice of positions. A planter could still grind a
  patch's whitespace to move its hash, and the "does not edit an item to move its place" rule
  forbids that. This is adequate for a lever that only moves a session in time.
- **Observation, which predates this change but c2 makes visible:** c2's tool-generated trust diff
  prints `+ fs: fs.read, fs.write`.
  - `mask`'s entry rule masks `+`/`-` only before `name: kind` with `kind` in §9.1's list. So
    `fs.read` sub-kinds pass unmasked.
  - §9.2 already concedes residual cues, so this doesn't block. If the author touches §9.2 for the
    fix above anyway, it is cheap to also accept `kind.sub` tokens in the entry rule.

DONE
