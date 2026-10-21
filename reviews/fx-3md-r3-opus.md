# fx-3md review round 3 (Opus, fresh pass)

Diff: `git diff 76dfe06 -- . ':!.beads'` plus untracked `scripts/autorun.sh` and `scripts/autorun-test.sh`.

## Changed

- `scripts/autorun-test.sh:42-43,54`: the fixture created `a` (P1) before `b` (P2), so creation order and priority order agreed. Removing `--sort priority` from `autorun.sh:105` (falling back to br's default `hybrid` sort, or `oldest`) still passed all 19 checks, which meant the "top ready bead" rule had no test. The fixture now creates `b` (P3) first, then `a` (P2). Both fall in hybrid's "others" class, where hybrid and oldest pick `b` and priority picks `a`. The same mutation now fails 12 scenarios. Expected picks are unchanged.

## Left alone

- `autorun.sh:149`: in continue mode this repeats the `latest_comment` call that `pick` already made. It costs one extra br call and is harmless.
- `autorun.sh:177`: a session that runs `git fetch` while the remote has moved also moves `@{u}` and stops the run. This errs on the safe side and matches the recorded decision.
- Handoff-skill step 5 can turn the handed-off bead into an epic whose handoff points at a child. The runner would continue the epic and judge the outcome on the epic, which would probably log "failed". This is a speculative edge case and the runner fails safe, so it is reported here and not fixed.
- `autorun.sh:22`: one header line runs past 80 columns. That is style only.

## Mutation probes (scratch copies outside the repository; every one was caught)

- Dropping the `owner` label filter fails 12 scenarios.
- Dropping the `epic` filter fails 12 scenarios.
- Dropping the "new comment since the session started" check fails continue-noop.
- Dropping the `.beads` exclusion from the dirty check fails 7 scenarios.
- Dropping the `blocked` outcome fails the blocked scenario.
- Dropping `--sort priority` fails 12 scenarios after the fixture change (0 before it).

## Check

- `timeout 400 scripts/autorun-test.sh </dev/null` passed all 19 checks with rc=0, before and after the change (about 25s).
- shellcheck is not installed, so it did not run.
