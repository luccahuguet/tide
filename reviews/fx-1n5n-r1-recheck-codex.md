Model: gpt-6.1-sol (high) · Date: 2026-11-17 · Candidate cf3a8fd (re-check of 6529c31 findings)
Verdict: approve

Confidence: high for this bounded finder re-check. All four original findings are resolved under the replacement mechanisms chosen by tide-lead. No new blocking regression was found in those fixes. This is not a new full review or an Opus verdict.

Reviewed the clean worktree at cf3a8fdad5e53722052a9c9a44e446ce7f836d5a, both requested diffs, the Round 1 finding-to-fix map, the coordinator's fix instructions, connected build/sysroot/sandbox code, revised documentation, and supplied proof logs. Merge base with main: c3fe99919a6af2528780519ff53a263c3ff533e5. Diff SHA-256: 6529c31..cf3a8fd = df046e5bcdd1dd6b64457a22dbe4dd5ec9f5e59a537657583aee00286b8251a2; main...cf3a8fd = ea9c0e2f0184b3d040caec3c81312c45b99037a29c09b61cd2ae1779494a3b27. No override.

1. **Finding 1 — Resolved.** `crates/tide-admit/src/invocation.rs:172`; `flake.nix:37` and `:103`.

   after_analysis has returned exactly to main's canonical-containment logic. A symlink spelled beneath the sysroot whose target resolves outside it is no longer silently discarded by the added spelling test. The previous operator-dependency identity regression is removed at its cause.

   The Nix accommodation is confined to packaging: rustc and lib are copied with symlinks dereferenced; copied ELF search paths referring to the joined toolchain are redirected to the copy; the derivation rejects remaining symlinks/references and verifies that its rustc reports the copied sysroot. Both package builds set RUSTC to that compiler. build.rs consequently derives the driver's baked sysroot and runtime library path from the copy. These are immutable, pinned store inputs. The log records a fresh sysroot build followed by successful std, driver and link-check builds with the restored identity code. The extra retained rustc-component closure is documented; reducing it is not required for this finding.

2. **Finding 2 — Resolved by removing candidate RUNPATH resolution and basename reuse.** `crates/tide-admit/src/invocation/sandbox.rs:427`; `crates/tide-admit/src/invocation.rs:81`; `flake.nix:76`.

   The ELF parser is back to main's PT_INTERP/DT_NEEDED parser: DT_RUNPATH, DT_RPATH and DT_SONAME do not supply search directories or a reuse policy. Bare names are looked up against a fixed compile-time list followed by the existing system directories. The Nix list contains only the selected C toolchain's absolute glibc/libgcc store directories. A candidate's RUNPATH=/private cannot add /private to that list, so the earlier unused-private-object grant route is removed. The directory list is not read from the runtime environment, source, Cargo configuration or driver arguments. Landlock grants remain per-file, not directory-wide.

   The focused test at sandbox.rs:664 checks that the test executable's RUNPATH-only librustc_driver cannot be resolved into a grant; the supplied cargo-test-r1.log records it passing. The Nix link check exercises the nonempty trusted list successfully. README and docs/nix.md describe the fixed-list policy rather than claiming to reproduce the dynamic loader's search algorithm.

   Precise boundary: this does not mean an arbitrary ELF binary has no influence whatsoever on grants. As on main, it supplies PT_INTERP and DT_NEEDED names, and a dependency name containing '/' follows the unchanged path-as-written branch. This fix adds no new candidate-selected search-directory route; it does not redesign those existing main behaviors. Packagers must keep the compile-time directories trusted, as the Nix flake does.

3. **Finding 3 — Resolved by making lookup independent of traversal history.** `crates/tide-admit/src/invocation/sandbox.rs:429–448`.

   With per-object search paths and basename reuse removed, each bare name resolves against the same ordered trusted/system list regardless of which object was visited first. The LIFO traversal can no longer choose /b/libX.so for one requester and force another requester to reuse that choice instead of its /a/libX.so. Retaining main's traversal is sufficient for the replacement policy; FIFO/SONAME-loader emulation would restore complexity that this fix intentionally removes.

   Empty-list equivalence is exact at the lookup boundary: "".split(':') yields only an empty entry, which the filter removes, leaving SYSTEM_DIRS in main's original order. The dependency parser, path-containing-name branch, duplicate-path check, traversal and ruleset construction otherwise match main. Non-Nix forced linker arguments and ordered cc/ld fingerprints also retain their old defaults. Rebuilt binary digests naturally change whole cache/receipt identity; this is not a new compatibility promise of identical executable bytes.

4. **Finding 4 — Resolved.** `docs/nix.md:100`; `target/work/report.md`, Round 1; `target/work/nix-build-r1.log`.

   The author records a trusted daemon connection and fresh builds with --option sandbox true --option sandbox-fallback false. The supplied build log shows all four derivations actually being built, including the materialized sysroot, followed by the store-interpreter check and a passing 100-case generated contract test. The user also explicitly confirms the fallback-disabled build in this re-check request. No restricted-setting rejection appears in the log.

   Documentation now states that the /usr/bin/cc absence assertion alone is host-specific evidence and that acceptance builds enforce sandboxing without fallback. It no longer presents absence on every host as proof of isolation. The log does not echo the invocation's flags, so the exact daemon options are accepted as the recorded author/coordinator observation, not something independently reconstructed by this reviewer. The RUN-3 “sandbox: full” output remains separate evidence about the child, not the Nix build sandbox.

Proof scope: reviewed the supplied successful Nix build/link-run log and workspace-test log (370 passing tests, including the new negative test). No builds, Nix commands, tests, subagents or exploit probes were run during this re-check. Only this report was written. The required subsequent Opus review remains outstanding; no unresolved blocking finding remains from this pass.
DONE
