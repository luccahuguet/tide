Model: gpt-6.1-sol (high) · Date: 2026-11-19 · Candidate 5246ee1 (re-check)

## Scope

Finder re-check of B1–B5 and both non-blocking items taken from `target/work/s1b-codex.md`. Read that review first, then `target/work/brief-s1b-fixes.md`, the builder's “Round S1b fixes” account, and `git diff 8b1b2ea 5246ee1`. The fix diff changes only `docs/tide-or-rust.md`. This is not a fresh full review.

## B1 — Resolved

`docs/tide-or-rust.md:25–33` now limits the network claim to outbound connections and explicitly says that `only` does not stop listening or unlisted incoming peers; accepting incoming connections remains possible until `no_bind()`. It retains the allowed program's full-authority qualification. This agrees with `design/reference.md:506–510` (outbound checks, open bind-name resolution and inbound peers) and `docs/at-a-glance.md:24–31`. The original confinement overclaim is removed.

## B2 — Resolved

`docs/tide-or-rust.md:154–158` now says the uploader can reach files, and that its directory and mode depend on the capability value its caller passes. It expressly distinguishes authority kind from narrowing. This matches `docs/owner-guide/01-capabilities.md:72–75` and the same-type attenuation rule in `design/reference.md:353–354`. It no longer implies that a signature reveals directory bounds, or that today's trust diff reveals the attenuation change excluded by `docs/owner-guide/07-trust-diff.md:161–163`.

## B3 — Resolved

`docs/tide-or-rust.md:121–127` now states that sandbox enforcement depends on the kernel, gives the `full` and `partial, not enforced` reports, and says partial runs continue under the restrictions that remain. Its Sandbox link targets the existing `README.md:398` heading. The wording agrees with `README.md:400–414`, `docs/owner-guide/08-sandbox.md:53–58`, and the container limitation in `docs/docker.md:52–56`. Builds and directly run binaries remain explicitly unsandboxed.

## B4 — Resolved

`docs/tide-or-rust.md:97–112` now scopes both cost bullets to descriptive observations from four small programs on one machine, with unstable series, rather than a general performance result. This matches `research/36-benchmarks.md:3–5,19–22,47–52`.

The release contracts and I/O comparisons now name Rust built with Tide's profile flags, and the contracts tie explicitly concerns erased clauses (`research/36-benchmarks.md:70,73–77,162–169`). The simulator result is now one seed of the `tasks` workload in release, consistent with `research/36-benchmarks.md:241–254`. The original numbers are preserved with the requested limits and comparators.

## B5 — Resolved

The required explanations appear at first use on the page:

- AI is expanded to “artificial intelligence” at `docs/tide-or-rust.md:45`.
- Tokens are identified as the units a model reads text in at lines 47–48, matching `docs/at-a-glance.md:43–46`.
- E3 is described as a small trial in which AI models reviewed planted changes with and without the trust diff at lines 80–81. This supplies the explanation in `docs/at-a-glance.md:40–42` and agrees with `trials/e3-pilot/report.md`'s opening and “What was tested” section.
- The `unsafe` grant is defined as the operator's explicit permission for `unsafe` operations at lines 119–120, matching `docs/owner-guide/02-admission.md:43–48`.
- Landlock and seccomp are explained as restricting file access and system calls at lines 122–123, matching `docs/owner-guide/08-sandbox.md:9–17`.

These are inline explanations, not links standing in for definitions.

## Non-blocking 1 — Resolved

`docs/tide-or-rust.md:138` now says “custom macros”. This preserves the distinction between excluded user macros and the retained fixed built-in forms and derives in `design/reference.md:55–78`.

## Non-blocking 2 — Resolved

`docs/tide-or-rust.md:103–105` adds compiler peak memory 17%–39% higher and release binaries loading 1.4–2.0 times the bytes of Rust built with Tide's profile flags. These match `research/36-benchmarks.md:68–69`; the loaded-byte comparison is further explicit at lines 132–143, so it does not misstate that ratio as total file size. The preceding unstable-run qualification covers these observations.

Lines 53–56 add E1's roughly ninefold cached-input-token cost for loading the reference and link chapter 9. This matches `docs/owner-guide/09-experiments.md:39–40` and remains scoped to the one-model trial.

## Verification and limits

HEAD was `5246ee1374324f72fa4e7dd8296500caa1cef446`; the tracked working tree was clean before writing this report. `git diff --exit-code 5246ee1 -- docs/tide-or-rust.md` confirmed that the inspected page matches the candidate. `git diff --check 8b1b2ea 5246ee1` passed. No new problem introduced by the fixes was found.

Only this report was written. No build, runtime test, benchmark rerun, external lookup, contact, commit or tracker write was performed. This re-check verifies the corrected wording against its cited records; it does not independently establish implementation behavior or measurements.

## Verdict

approve — B1–B5 and both non-blocking items taken are resolved. No blocking finding remains within this finder re-check's scope.

DONE
