# fx-3md r8: Opus confirmation of the live_agent fix for Codex r7 B1

Reviewer: fresh Opus agent, read-only on the repository. Date: 2026-10-21.
Candidate: uncommitted working tree on 76dfe06 (HEAD is now 0082d96, an unrelated fx-gcs.1 commit; the fx-3md delta is `.gitignore`, `README.md`, and the untracked `scripts/autorun.sh` and `scripts/autorun-test.sh`).

## Verdict

**revise**: 1 blocking, 5 non-blocking.

B1 from Codex r7 is closed, and `live_agent` is correct on every point I was asked to check. One of the two new scenarios, `app-server-ignored`, proves nothing. It passes even with the app-server filter deleted. The fix to the test is one token.

## Blocking findings

1. **`app-server-ignored` is vacuous: it passes with the filter removed.**
   `scripts/autorun-test.sh:145` starts the stand-in as `"$TMP/fake/codex" -c "sleep 60" app-server`. Bash runs a lone `-c` command with exec. The process then turns into `sleep` (comm `sleep`, cmdline `sleep 60`) before the runner scans, so no process named `codex` exists. I checked this directly: comm=`sleep`, cmd=`sleep 60`. A mutation run confirms it. With line 110 of `scripts/autorun.sh` (the `app-server` skip) deleted, the scenario still prints PASS.
   This filter carries real weight. On this machine a `codex app-server --listen unix://` (pid 1931920) has its cwd in the Tide checkout right now. Without the filter, the runner would never start here.
   **Fix:** stop bash from exec'ing its last command, e.g. `-c "sleep 60; true" app-server`. With that change, the scenario passes against the candidate and fails against the filter-deleted mutant (picks '' instead of 'a b').

## Non-blocking notes

1. **The app-server match is a substring match on the whole cmdline, and it applies to claude as well as codex.** A `codex exec` or `claude -p` whose argv prompt happens to contain "app-server" is skipped as idle. A Codex review of this very bead is one plausible case. Matching argv[1] exactly would be tighter and also simpler, e.g. `[[ "$(tr '\0' '\n' <"$d/cmdline" 2>/dev/null | sed -n 2p)" == app-server ]]`. That also avoids a theoretical pipefail/SIGPIPE false negative from `tr | grep -q` on long cmdlines.
2. **Sessions hosted by an app-server are invisible by design.** That covers the Codex app, the IDE extension, and the plugin broker. Such a session is seen only while it runs a sandboxed command, through its `codex-linux-sandbox` child (comm `codex`). One clause in the header would say so, next to the existing "other worktrees" caveat.
3. **A runner launched from inside a claude session can stop on that session's helpers.** Claude Code re-execs itself as `ugrep` for its Grep tool. Those helpers keep comm `.claude-wrapped` (one is visible in `/proc` now) and are children of the ancestor, not ancestors themselves. If the parent session greps the checkout at the moment of `pick`, the runner stops with "process N (.claude-wrapped) is working". The failure is fail-closed and rare.
4. **`check_extra` kills the fake only when the scenario's other checks pass.** `scripts/autorun-test.sh:74-76` short-circuits with `&&`. In the failing case the stand-in lingers for up to 60 s after the suite ends. That is harmless, since its directory is in the removed `$TMP`.
5. **The header has wrapping nits.** Lines 18 and 23 of `scripts/autorun.sh` run long against the reflowed paragraph.

## Check results

All checks were read-only on the repository. Mutations and probes ran on copies in the scratchpad.

- **Suite:** `timeout 500 scripts/autorun-test.sh </dev/null` on the candidate. 22/22 PASS, including `manual-continue` and `app-server-ignored`. rc=0, 4m12s on the loaded machine.
- **Blocker closed (mutation):** I made the runner copy skip `live_agent` (`false && stop`). `manual-continue` then FAILs with picks 'b b' and outcomes 'failed,failed', so the runner relaunches the handed-off bead the manual session holds. The candidate stops with "is working in this checkout" and picks nothing. The scenario proves B1's fix.
- **app-server scenario (mutation):** with the filter deleted, `app-server-ignored` still PASSes (vacuous). With the filter deleted and the test using `sleep 60; true`, it FAILs. With the candidate runner and the fixed test, it PASSes.
- **Sibling-worktree prefix:** I probed the extracted `live_agent` with ROOT=`.../probe/tide`. A `.claude-wrapped` stand-in with cwd `.../probe/tide-fx-gcs.1` was not matched. One with cwd `.../probe/tide/sub` was matched. `"$ROOT"/*` requires the slash, and the quoted ROOT is literal.
- **Ancestor exclusion and launch from inside claude:** I ran the probe from a Bash tool inside a claude session (pid 608990, cwd in the Tide checkout) with ROOT=`/home/lucca/pjs/lab/tide`. 608990 was in `ancestors` and excluded. The two app-servers in the checkout were skipped. The other two `.claude-wrapped` sessions in the checkout (4179717, 582191) were reported, which is correct fail-closed behavior.
- **Processes vanishing mid-scan:** in code, an unreadable `comm` or `cwd` means `continue`. A `cmdline` that vanishes leaves `grep -q` false, and the following `readlink` then fails, so the result is still `continue`. Another user's processes fail at `readlink` and are skipped. A churn probe ran 30 scans against 400 short-lived stand-ins: no errors, no stderr output. The ancestor walk ends cleanly if a `status` read fails (empty `p`).
- **comm truncation:** `.claude-wrapped` is exactly 15 characters, and `/proc` shows it whole on this machine. `codex`, `.codex-wrapped`, and the plain `claude` symlink name all fit. On this machine, `codex exec` and `codex-linux-sandbox` both show comm `codex`.
- **Regressions:** no other scenario changed outcome. Every check that ran before `live_agent` existed still runs after it. The only behavior change is the intended one: the runner refuses to start beside any non-ancestor claude or codex process in the checkout, and the header documents that. `.gitignore` and `README.md` are unchanged from earlier rounds.
