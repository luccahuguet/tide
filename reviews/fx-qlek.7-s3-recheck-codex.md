Model: gpt-6.1-sol (high) · Date: 2026-11-19 · Candidate 2e7635e (S3 re-check)

## Verdict

approve — B1–B5 and the taken NB1 are resolved. No new blocking defect found in these fixes. Confidence: high for wrapper behavior and documentation alignment; runtime conclusions rely on the recorded verification, not newly executed builds or demo runs. This is the requested finder re-check, not the subsequent Opus review or landing approval.

## Identity and scope

HEAD: 2e7635ea7115d74e80ac609059403f700b900d32, branch fx-qlek.7-s3. Tracked worktree clean before review. Reviewed `git diff da0fcaa 2e7635e`, its four touched files and relevant surrounding implementation, the original findings, fixes brief, and report/logs. `git diff --check da0fcaa 2e7635e` passed. Candidate diff against supplied base 46297dd has SHA-256 `108e05227f50dabfaa554b4e76b72c6ec1d6255d1b82b14a408142d580b768c2`. The local main ref has advanced to be11ccb; this review remains bound to 2e7635e and does not cover a future rebase.

Offline, build-free evaluation of `apps.x86_64-linux.tide-demo.program` selected `/nix/store/cig11s9y83ac0f4bgjc88adw7zw48rbv-tide-demo/bin/tide-demo`. Read that existing launcher: its target handling, environment exports, source directory and Cargo command match the candidate. This agrees with the wrapper identity recorded in report.md, S3 round 1 fixes.

## Findings re-check

### B1 — Resolved

`flake.nix:205–214` obtains the target before changing directories, leaves absolute values unchanged, and prefixes relative values with the invocation `$PWD`. Quoting preserves spaces. An empty override retains the existing default behavior. `CARGO_HOME` and `TMPDIR` are derived after this resolution.

Recorded clean-HOME verification (`b2-write-check.sh`, `s3r1-b2-write-check.log`, plus the retained a.log/b.log under b2check-1795087635) covers the default and relative override. Default: exit 0 in 45 s, Done line, artifacts under home-a/.cache/tide-demo. Relative `CARGO_TARGET_DIR=target` from scratch-b: exit 0 in 50 s, Done line, driver and std under scratch-b/target; scratch-b contains only target. The absolute override `/work` is exercised by the successful read-only-root container check with writable /tmp described below. No path is resolved against the read-only store sources.

### B2 — Resolved, with the documented linker exception

`flake.nix:211–213` exports and creates `$target/tide-demo-home/cargo` as CARGO_HOME and `$target/tide-demo-home/tmp` as TMPDIR before exec. The evaluated launcher contains these exact exports. Both clean-HOME targets retain Cargo's .global-cache, .package-cache, .package-cache-mutate and registry beneath that Cargo home; I inspected these retained entries. The logs and retained directories show no caller ~/.cargo, no entries in either outer TMPDIR, and no scratch-directory output outside target. Outside target under the fresh HOMEs are the .cache parent and Nix's .cache/nix activity, which the check explicitly excludes from application writes. The harness's own logs and directories are also excluded explicitly.

The clean-HOME listings alone do not observe transient or global /tmp writes. The stronger recorded check (`b2-readonly-check.sh`, `s3r1-b2-readonly-check.log`, `s3r1-docker-run-readonly.log`) uses a read-only image root and only the absolute target /work mounted writable: it fails at step 1 while linking, SIGABRT. `crates/tide-admit/src/invocation.rs:1206,1307` confirms the compiler subprocess environment is cleared, so the wrapper's TMPDIR cannot reach that linker. The second check (`b2-readonly-tmpfs-check.sh`, `s3r1-b2-readonly-tmpfs-check.log`, `s3r1-docker-run-readonly-tmpfs.log`) adds only writable /tmp to that setup and completes every demo step, exit 0 in 36 s.

The supported boundary is therefore target plus linker temporary files in /tmp, not strict target-only writes. The flake comment, README.md:329–331, docs/nix.md:94–103 and docs/docker.md:21–27 now state that boundary consistently. Nix's own acquisition/cache writes remain separate. The account also discloses that env -i dropped the Cargo-job cap in the two new clean-HOME probes; those probes are not the original README timing measurements.

### B3 — Resolved

README.md:337–343 and docs/docker.md:62–65 distinguish Nix build caps from runtime caps. They explicitly disclose that the measured devcontainer runtime had no CPU or Cargo-job cap, while retaining the original 360 s / 49 s observations. No stronger capped-runtime comparison is claimed; no rerun was needed for this correction.

### B4 — Resolved

README.md:341–350 says no download volume was measured, changes the column to Size, and labels ~/.rustup (about 1.4 GiB) and ~/.cargo (83 MiB) as disk usage afterwards. The Nix number is store closure size; Docker's 769 MB is explicitly a gzip-compressed docker save measurement. These are no longer presented as crate-download bytes.

### B5 — Resolved

Docs/docker.md:21–27 says the try image starts the demo, has no /bin/sh, and runs its launcher using the closure's own bash. The evaluated launcher's store-Bash shebang and Dockerfile:47–60 support that distinction. The shell-free claim is removed.

### NB1 — Resolved

Docs/docker.md:48–52 distinguishes the flake's pinned runtime closure from the unpinned apt/Nix-installer bootstrap, and explicitly says the recipe does not rebuild either image bit for bit. Dockerfile:18–20,35 still show those inherited moving bootstrap sources; the documentation acknowledges the limitation without expanding implementation scope.

## New blocking findings

None in the fixes reviewed. NB2 was excluded as requested. The recorded code_prefix check still fails on the inherited owner-guide quotation (s3r1-code-prefix-test.log:15–24); this re-check does not claim that check passed or that the anticipated rebase fix has occurred.

No builds, Rust/demo execution, image commands, commits, bead writes, or contact with other sessions performed. Only this report was written. Platform execution and filesystem confinement were assessed from the supplied Linux verification and code; no new runtime trace was taken.

DONE
