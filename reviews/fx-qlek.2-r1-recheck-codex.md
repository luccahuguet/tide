Model: gpt-6.1-sol (high) · Date: 2026-11-18 · Candidate b919d06 (re-check of 0d6baa3 findings)

Verdict: approve

Confidence: high. Finder re-check limited to the three original findings and regressions caused by their fixes in `git diff 0d6baa3 b919d06`. This is not a new full review. No override.

1. **Finding 1 — Resolved: private checkout state in the build context.** `.dockerignore:5`, `.dockerignore:21`, `Dockerfile:37`, and `docs/docker.md:20`.

   The default exclusion now admits only named source/documentation directories and root build/license files. `.git`, `.beads/`, `.worktrees/`, root local state, nested dotfiles and the named build-output patterns are excluded. Removing `.git` also removes local Git configuration before context transfer, rather than hiding it in a later layer.

   I independently compared the candidate's tracked files under the allowed paths with both recorded context listings and both actual exported context directories. Each contains exactly 549 files, with no extra or missing file. Every exported file is byte-identical to the corresponding file in `clone-r1`; that clone is at `b919d06d3800f6527a7daef760cb50f9b469bd59` and has no diff from the candidate in those paths. Thus the tested clean-clone context and the context containing the planted private files both match the intended public-source subset. The context-export logs show successful exports. The updated container environment log also shows no `.git` or `.beads`.

   The rebuilt image and demo still succeed without Git repository metadata: Nix reads the directory flake, and the demo prints `revision: unknown`, as the updated guide discloses. No blocking regression found from the context change.

2. **Finding 2 — Resolved: exact pinned rustc and Cargo identity.** `crates/tide-demo/src/lib.rs:264`, `crates/tide-demo/src/lib.rs:320`, `crates/tide-demo/tests/demo.rs:77`, and `crates/tide-demo/tests/demo.rs:109`.

   The calendar-date heuristic is removed. Before building, the fallback requires the channel to match its recorded pin and compares both rustc and Cargo's `release` and full `commit-hash` fields by exact equality. Missing fields, mismatches, unsuccessful version commands and spawn failures return setup errors. Mismatch diagnostics identify the tool, field, reported value and required value, with a recovery route.

   The subprocess regression test actually starts the demo with a cleared environment and a PATH containing only fake rustc/Cargo tools, so rustup cannot select the other branch. It separately changes rustc's hash and Cargo's hash while preserving their releases, requires exit 2 with the appropriate mismatch diagnostic, and checks that neither mismatch reaches a Cargo build. The positive control reaches the inner build. A separate test compares the recorded identities against both tools from `rustup run <pinned channel>` and checks the channel against rust-toolchain.toml, guarding the additional identity record against pin drift.

   `demo-test-r1.log` records all nine tests passing, including both new tests. The updated container demo also exits 0 through the real no-rustup path. No remaining exact-identity blocker found.

3. **Finding 3 — Resolved: bind Cargo to the compiler checked.** `crates/tide-demo/src/lib.rs:292`, `crates/tide-demo/src/lib.rs:305`, `crates/tide-demo/src/lib.rs:521`, and `crates/tide-demo/tests/demo.rs:158`.

   The fallback resolves and retains absolute paths for both validated tools. Inner builds invoke that Cargo path and explicitly set `RUSTC` to the validated compiler path. Both inner package builds use this shared command constructor. The subprocess test supplies a conflicting `CARGO_BUILD_RUSTC`, then checks the fake Cargo's build log for the exact checked compiler in RUSTC. Removing the binding would fail that assertion. The author also records real pinned-Cargo precedence checks for both CARGO_BUILD_RUSTC and `[build] rustc`, plus mutation checks for the binding and Cargo identity gate. The passing recorded suite and container run support the implementation.

   The rustup branch still removes RUSTUP_TOOLCHAIN and calls `rustup run <channel> <program>` unchanged; the new identity gate and RUSTC binding apply only to the fallback. The find_gdb refactor preserves its previous relative-path/bare-name resolution. The recorded host suite passes with rustup present. No regression caused by this fix found. Deliberately configured compiler wrappers remain operator-controlled, as the author explicitly records; these fixes do not claim to validate arbitrary executable contents.

Proof checked: `target/work/report.md`, `demo-test-r1.log`, the clean/planted context exports and their logs/listings, `build-r1.log`, `run-r1.log`, and the relevant `accept-r1` logs. The rebuilt image is recorded as `sha256:8e96c5d4ebe825340d7d7ca0fc3d3ceed38c474bdea6cffb6fcff4d2f134a648`. All four container check statuses are 0, including the no-rustup demo and real production-std confinement test. The author discloses that the container acceptance run started above the repository load limit; that operational breach does not change the observed contract results or reopen these three code findings.

Review identity: HEAD `b919d06d3800f6527a7daef760cb50f9b469bd59`; SHA-256 of `git diff 0d6baa3 b919d06`: `5d654c229fa826297e1d9d712b202edd96faa1e3829f69950cb2ce96ddee2bc8`. `git diff 0d6baa3 b919d06 --check` passes. No remaining blocking finding or required patch from this re-check.

Not checked: no builds, Docker/Nix commands or tests rerun; runtime conclusions rely on the supplied proof. No macOS/Windows execution verified, no subagents started, and trials/e1-checked/ was not opened.

DONE
