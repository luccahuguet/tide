Model: claude-opus-5-5 · Date: 2026-11-18 · Candidate ca070c3

# fx-v0gc r2: Opus converge round (report only)

## Verdict

**Approve.** No blocking findings. Four non-blocking findings follow, all text or test coverage. None needs a re-check.

Scope: `git diff main...ca070c3` (merge base d6e5e27), worktree HEAD ca070c3c886764220d93e2252a2f6235de92527d, clean. `git diff d0fa3b3 ca070c3` touches only README.md, design/03 and docs/debugging.md, so the builder's runs on d0fa3b3 still cover the code. I read AGENTS.md, GOVERNANCE.md (Merge Rule), the bead and its comments, the builder's report, notes and logs, r1-codex.md, the touched files, and the code that consumes the argument vector. I also read the remap-scope definition in the pinned toolchain's rustc source (`lib/rustlib/rustc-src`). I inspected the existing `/mnt/build/tide-agents/fx-v0gc-target/tide-demo/out/record/hits_sim-test` with readelf. I ran no builds, tests, debugger sessions or subagents, and changed no file except this report.

## Blocking findings

None.

## Non-blocking findings

1. **The `substitute-path` example assumes the default target directory, and its relative form is untested.** At `docs/debugging.md:135`, the example `set substitute-path tide-library target/tide-demo/revisions/buggy/src` hard-codes `target/`. The by-hand block twelve lines above uses `${CARGO_TARGET_DIR:-target}`, and this host's agents set `CARGO_TARGET_DIR`. The builder verified only an absolute target (`try3.gdb:38`, `try4.gdb:39`: `/mnt/build/tide-agents/fx-v0gc-target/...`). The relative form probably resolves through gdb's `$cdir:$cwd` search. `$cdir` is the driver's working directory, the repository (`DW_AT_comp_dir /mnt/build/eon-agents/worktrees/tide-fx-v0gc`), so that works only with the default target directory. **Failing case:** a reader with `CARGO_TARGET_DIR=/mnt/build/...` pastes the line, and gdb still shows no source text. **Minimum fix:** say "`<target dir>/tide-demo/revisions/buggy/src`, an absolute path", or name `$CARGO_TARGET_DIR` as the block above does. The builder's report already lists this as an open item.

2. **No gdb-free check that debug-profile builds carry line information.** Only demo step 5 can fail if `-Cdebuginfo=2` is dropped or stops reaching a build path, and it skips when there is no gdb. A host or CI without gdb would let a regression land green. The builder correctly rejected a unit test on `rustc_args` as implementation trivia, and a `.debug_line` section scan as non-discriminating, since std carries line tables. A test that discriminates could look for a DW_AT_name compilation unit that starts with `tide-source/` or `tide-library/` (readelf output above), but it needs a DWARF reader. That is acceptable before the alpha. Record it as a follow-up only if tide-lead wants it.

3. **Host-path disclosure for debug-profile binaries is documented only indirectly.** Debug-profile executables now carry DWARF with `DW_AT_comp_dir`, the driver's working directory. In this host's demo binary that is `/mnt/build/eon-agents/worktrees/tide-fx-v0gc`; under a home directory it would include the username. Non-test builds also carry absolute source paths, because they install no remap. Panic `Location`s already embedded the absolute source path of non-test builds, and the linked std already carried its own DWARF paths. The new exposure is therefore incremental: the compilation directory, and the source paths now present in DWARF. README.md:452-459 and design/03:1993-2000 state it correctly ("the compilation directory ... may remain"; non-test builds keep their real spelling). docs/debugging.md, however, mentions physical paths only for release-linked libraries (`:136-138`). A reader of the debugging page alone could take its role-named sources to mean that debug binaries hold no host paths. **Optional fix:** one clause at `docs/debugging.md:132-135`: "debug binaries still hold the build's working directory, and non-test builds their real source paths."

4. **Nit: `demo/README.md` (step 5 text) says "the write on the line after it".** The code finds the write line by its text (`crates/tide-demo/src/lib.rs:1006`) and does not require adjacency. The sentence is true for today's patch. "the write on `stats.with(|s| s.hits = seen + 1)`" would match the check exactly. It needs no re-check.

## Reasoning

### 1. Correctness

- **Every debug-profile build path gets debuginfo.** `rustc_args` (`invocation.rs:342`) is the only argument builder. `compile` passes its vector to both `run_compiler` calls, including the synthesis rerun (`:448`, `:467`, `:474`). It serves metadata, bin, lib, test and paired-rlib builds. `compile_with_library` copies the profile into the library invocation (`:572`). `parse` defaults to `debug` and accepts only `debug|release` (`:206`, `:261`). Every `Invocation` literal sets a profile explicitly: `:564`, and `replay.rs:59`. A `Default` `""` therefore never reaches `rustc_args`, so `!= "release"` and `== "debug"` behave the same. Metadata-only builds also get the flag. It is inert there apart from a one-time cache-key change, which is harmless.
- **Release is unchanged.** Nothing is pushed for release, not even `-Cdebuginfo=0`, so release vectors, and with them release cache keys and replay `options`, are byte-identical. Release identities still change through the driver digest, as with any driver change.
- **Replay identity.** `semantic_options` builds a normalized `Invocation` with the same profile and calls `rustc_args` (`replay.rs:57-97`), so the suite's options and the `paired-library:` options both carry `-Cdebuginfo=2` in the debug profile. That change is deliberate and commented at `invocation.rs:378-379`. The comparison is unchanged (`tide-std/src/replay.rs:113-138`): `profile` is compared as a scalar before `options`, so a recording from one profile cannot replay under the other. Options are compared as exact, ordered vectors. Nothing is weakened. An old-driver debug manifest now reports `options` with `only current: "-Cdebuginfo=2"` before reaching the driver comparison. That is a clear refusal, not a bypass. No committed fixture stores driver options: a grep of `*.json`, `*.rs` and `*.md` for `Cdebug-assertions`/`debuginfo` finds only the code and docs.
- **Other consumers of rustc args.** The build cache key records every `rustc_args` element (`invocation.rs:1184`). Receipts hold the per-build executable digest, re-checked against the staged binary at replay (`replay.rs:261-269`). DWARF is not in the manifest, so build-location-dependent DWARF bytes do not perturb replay. Conformance passes only `--tide-profile=release` as a directive (`tide-conformance/src/main.rs:553`) and does not compare argument vectors. The `raw_compiler_flags_and_response_files_are_rejected` test (`tests/invocation.rs:760`) still rejects a user-supplied `-C`, so the flag stays driver-owned. Trust artifacts and diffs do not hash arguments. Conformance's inputs did not change, but its debug-profile driver arguments did, so tide-lead's landing run of conformance remains the right check, as planned.
- **Step 5 cannot pass vacuously.**
  - A stop with no symtab makes `sal.symtab.filename` raise in `$here()`. The dprintf then errors instead of printing a `gdb: thread` line, so `entries == 0` fails the step, or the run stops and the `replay reproduced` marker is missing.
  - Every printed stop must parse (`Access::parse` now requires ` at FILE:LINE`) and match: basename `hits.rs`, and the computed line, so a stop in `hits_sim.rs` fails.
  - The counts must agree and be nonzero, and two or more threads must show `over > seen` (`lib.rs:1048-1053`). The replay marker must appear (`:1060`).
- **Moving lines in hits.rs.** `read_line` and `write_line` come from the patched buggy file the demo built (`fs::read_to_string(buggy)`, where `buggy` is `revisions/buggy/src/hits.rs`, `lib.rs:456-481`), matched by trimmed exact text. A shift above `record_hit` moves both sides together. A reformatted line becomes a loud setup error. Each text occurs once in the patched file (`Ok(stats.with(|s| s.hits)?)` at :52 differs). The entry stop is expected on the read line, which holds at opt-level 0 because the breakpoint lands after the prologue. A toolchain that reports line 56 would fail loudly, not pass silently.
- **The per-thread read/write check is intact.** The `last_read` map and the `seen == last read` comparison (`:1036-1043`) are unchanged.
- **Register-reading removal is complete.** No `$rdi`/`$rsi` or register reading remains in the crate or the docs (grep). The `set language c` line is kept, and the comment above `GDB_SCRIPT` now gives its real reason.

### 2. Privacy and reproducibility

- **Host paths in debug binaries.** Paths that match the remapped roots are role-named in linked test builds (CU names `tide-source/hits_sim.rs/@/…`, `tide-library/hits.rs/@/…`). `DW_AT_comp_dir` stays physical. Tide std's compilation units are named `crates/tide-std/src/lib.rs/@/…` under the same comp_dir, and dependencies carry `~/.cargo/registry` paths (per r1, not re-dumped). Non-test builds have no remap at all (`:401`). See non-blocking finding 3.
- **Replay records.** `Build.options` holds only normalized locators (`source-root-N`, `pinned-sysroot`, `output`, `std`, `dependency-N`). Adding `-Cdebuginfo=2` adds no host path. r1 found neither `/mnt/build` nor `/home/lucca` in the demo manifests.
- **Published artifacts.** No release artifact ships from this bead. The README release clause correctly says that linked libraries may keep their debuginfo.
- **Remap text against rustc.** The pinned rustc defines `OBJECT = MACRO | DEBUGINFO | COVERAGE` (`rustc-src/rust/compiler/rustc_span/src/lib.rs:251-253`; parser at `rustc_session/src/config.rs:1343-1365`). `macro,object` therefore remaps panic and `file!()` paths, debuginfo and coverage, and leaves diagnostics alone. The new README and design/03 text says exactly that. The existing test `replay_remapping_keeps_diagnostics_and_non_test_locations_real` (`tests/build.rs:4539`) pins the diagnostics and non-test halves. "The compilation directory ... may remain" matches the readelf output above. The `main.rs:39-40` doc comment ("diagnostics and non-test builds keep real paths") was already consistent.

### 3. Design text

- **The design/03 edit is a faithful correction.** The scope has been `macro,object` since it was introduced in 1c1654a (fx-nkv.7). That commit's own review suggestion asked for an embedded-path scope that keeps diagnostics real, and the claim that debug-info and coverage paths "keep real spellings" contradicted the flag's definition from the start. It went unnoticed because Tide crates had no DWARF until now. The edit changes no flag, scope, role name or identity rule: it states what the existing mechanism does.
- **Contract.** The promised outcomes (role paths for panic locations and trace origins, real diagnostics, no promise of removing every physical path, same-machine replay) are unchanged.
- **Tier.** The edit stays inside the v0 core the change already edits, and design/03 is not a single-holder file.
- **Reference files.** `design/reference.md` and `design/reference-companion.md` do not state the old rule: a grep for `remap`, `real spelling`, `debug-info`, `debuginfo`, `debug info` and `tide-library` over `design/` finds only design/03's paragraph. `design/01-design.md`, `research/32-replay-manifests.md:88` and `conformance/` do not repeat it either. Nothing to report there.

### 4. Leftovers and claims

- **docs/debugging.md claims match the builder's observations** (`notes.md`, report "Acceptance"):
  - level 2 is Cargo dev's `debug = true`;
  - `info line tide_candidate::record_hit` gives line 56 of `hits.rs` (56 is `fn record_hit` in both the service and the patched file);
  - `break hits.rs:57` resolves to two locations, so a line with a closure stops in each function;
  - `bt` shows files and lines, and `print seen` works;
  - the scripted output block equals `demo2.log`'s step 5.
- **Release claims.** README.md:176-179 and debugging.md:7-9 and :136-138 now carry r1's correction, which is accurate.
- **rust-lldb and RustRover** remain marked untried (`docs/debugging.md:141-143`, `:151`).
- **Other docs.** The owner guide never stated the gap (`06-replay.md` only says step 5 replays under gdb). `demo/agent-card.md:37`'s `prints gdb: thread N reads|writes hits = …` still holds, since the ellipsis covers the new ` at FILE:LINE`.
- **Builder's proof** (on d0fa3b3, code-identical to ca070c3): `workspace-test.log` has every `test result: ok`, tide-admit build 107/107; `demo-test.log` 7 passed; `demo2.log` ends "every step showed what it should" with step 5 run, not skipped. This is the builder's evidence, not independent proof. Conformance is still to run at landing.

DONE
