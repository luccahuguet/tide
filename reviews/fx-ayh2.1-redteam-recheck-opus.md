Model: Claude Opus 5.5 · Date: 2026-11-18 · Red-team re-check 1e48e1d

# Red-team re-check: fx-ayh2.1, findings F1–F9

- Candidate: `1e48e1df013abb990ceeae165ff4e81257665db8`, which amends `fc015c3`. Round diff: `git diff fc015c3 1e48e1d`, SHA-256 `8fa2a6fe2ef21d92531a26b0e6e0286f6ce8bcf03cd3d8836ec0779eac2f8ce3`. It touches 5 files and leaves `design/reference.md` unchanged.
- Scope: only the fixes for F1–F9, and what they touch. I treated the builder's "Round redteam fixes" account as claims and checked each one against the diff.
- Evidence for the glibc list: `sysdeps/generic/unsecvars.h` read at tags `glibc-2.39` and `glibc-2.41`, through the GitHub mirror `bminor/glibc`, because sourceware.org refused the automated fetch. The host's glibc is 2.39 (Ubuntu 2.39-0ubuntu8.12). Admission behavior comes from reading `crates/tide-admit/src/walk.rs:340–350` and `crates.rs:210–237`. Nothing was compiled or run.

## Verdict

**Changes required**, for two small text defects that this round introduced: N1, a misquoted glibc list, and N2, a PROC-3 guard option that misses re-exports. Each needs about one line. All nine findings are otherwise fixed as I proposed, or better. N3 and N4 are non-blocking.

## Finding status

| Finding | Status | Where |
| --- | --- | --- |
| F1: full authority beyond arguments and variables | **Fixed**, with my text: scripts, files read, the return path, setuid, and the "bounded only when" sentence. | 03 "What `only` does not bound"; 03:284 "at least"; research/40 §4; companion 20 `only` bullet; 01:511–518 |
| F2: loader variables | **Fixed in mechanism**, with the recommended option. The cited list is incomplete (N1). The "it knows" framing is in research/40:249. | 03:299–307, research/40:190–201, PROC-2 |
| F3: trust-diff wording | **Fixed.** 03 §4.3 now owns the text and gives an example line; research/40 §8 points to it. | 03 §4.3, after "listed first" |
| F4: authority has no end | **Fixed.** Stated in 03, the companion's Authority bullet, and research/40 §4. | 03 Processes note |
| F5: order on both backends | **Fixed**: program, then refused variables, then NUL (`InvalidInput`), then lookup or model lookup; PROC-2 checks it under simulation. Nit N4. | 03 PROC-ONLY; research/40:202–208 |
| F6: magic links; lookup binds a path | **Fixed as a limit.** The optional `/proc/` skip was also taken, and is lexical (N3). | 03:309–319; research/40:171–180, §5 |
| F7: the sweep fails closed | **Fixed**, after stdio setup, with `EINVAL` before 5.11 named; PROC-1 checks it. | 03 Child environment and descriptors; research/40 §5; PROC-1 |
| F8: Tide std's admit-by-default | **Fixed in intent** with PROC-3 and research/40 §2 and §12. One of its two guard options is insufficient (N2). | rules.md:98; research/40 §12 |
| F9: argv[0] | **Fixed** on both backends; models match the arguments after argv[0]; PROC-1 checks it. | 03 PROC-ONLY; research/40 §3 |

## New defects

### N1. The cited glibc 2.39 `UNSECURE_ENVVARS` list leaves out seven entries (misleading statement, blocking; one-line fix)

**Where.** 03:299–307 and research/40:192–201 both say "glibc's/glibc 2.39's `UNSECURE_ENVVARS` names (`GCONV_PATH`, … `TZDIR`), or that is `GLIBC_TUNABLES`". The builder's report calls `GLIBC_TUNABLES` "my addition".

**Evidence.** At tag `glibc-2.39`, `sysdeps/generic/unsecvars.h` has one macro, `UNSECURE_ENVVARS`, with 34 names. Fourteen are `LD_*`, which the prefix rule covers. The other twenty are:

`GCONV_PATH`, `GETCONF_DIR`, `GLIBC_TUNABLES`, `HOSTALIASES`, `LOCALDOMAIN`, `LOCPATH`, `MALLOC_ARENA_MAX`, `MALLOC_ARENA_TEST`, `MALLOC_MMAP_MAX_`, `MALLOC_MMAP_THRESHOLD_`, `MALLOC_PERTURB_`, `MALLOC_TOP_PAD_`, `MALLOC_TRACE`, `MALLOC_TRIM_THRESHOLD_`, `NIS_PATH`, `NLSPATH`, `RESOLV_HOST_CONF`, `RES_OPTIONS`, `TMPDIR`, `TZDIR`.

So `GLIBC_TUNABLES` is already in glibc's list, not an addition. The candidate's list drops the seven `MALLOC_*` tunable aliases other than `MALLOC_TRACE`. The `glibc-2.41` list is the same except that `LD_HWCAP_MASK` was removed, which the prefix rule covers anyway. The seven missing names tune the allocator and load no code, so the security impact is small. The problem is that the docs present the refused set as glibc's list, and it isn't.

**Minimum fix.** Replace the parenthesized names with a reference to the whole set, in both places: "every name in glibc 2.39's `UNSECURE_ENVVARS` (`sysdeps/generic/unsecvars.h`, 34 names: the `LD_` ones, `GCONV_PATH`, `GETCONF_DIR`, `GLIBC_TUNABLES`, `HOSTALIASES`, `LOCALDOMAIN`, `LOCPATH`, the `MALLOC_*` tunable aliases and `MALLOC_TRACE`, `NIS_PATH`, `NLSPATH`, `RESOLV_HOST_CONF`, `RES_OPTIONS`, `TMPDIR`, `TZDIR`)". Drop "or that is `GLIBC_TUNABLES`" as a separate clause. In 03, add the "what it knows" scope that research/40:249 already carries: "It is glibc's list at that version; another C library's variables (musl's `MUSL_LOCPATH`) and names later glibc adds are not refused." PROC-2's "03's list, `GLIBC_TUNABLES` included" then stays true unchanged.

### N2. PROC-3's allowlist-plus-deny option does not guard re-exports (misleading statement, blocking; one-line fix)

**Where.** rules.md:98 (PROC-3: "…or by a profile allowlist with `deny("tide_std::process::*")`") and research/40 §12, which states the same two options.

**Evidence.** `walk.rs:340–350` (`admit_def`) returns `Admitted::Free` for any item that Tide's std re-exports, and for the items of any trait it re-exports, *before* it looks up a policy rule. `crates.rs:210–237` (`collect_exports`) records every public `pub use` of another crate's item. Keys for a re-exported Rust item print under `std::`, not `tide_std::process::`, so a `tide_std::process::*` deny never matches them, and the TD0501 rule `ambient("std::process::exit")` is bypassed too. Concretely, a stage-2 `pub use std::process::abort;` (or `exit`) in `tide_std::process` would be admitted under the deny option, with no diagnostic. Re-exporting a Rust process trait would admit its methods the same way.

**Minimum fix.** In PROC-3 and research/40 §12, keep the item-list test as the required guard and say what it covers: "a test that lists `tide_std::process`'s public items, `pub use` re-exports included (admission admits a re-exported item before any rule), and fails on additions". A deny glob may be added alongside it, but not instead of it.

### N3. The `/proc/` skip is lexical (limit to state, non-blocking)

**Where.** 03:310 ("skipping elements under `/proc/`") and research/40:171–173 ("elements under `/proc/` are skipped too, since a per-process link such as `/proc/self/cwd` resolves again in the child").

A byte-prefix check misses `//proc/self/cwd/bin` and `/./proc/self/cwd/bin`. It also misses a symlinked element such as `~/bin -> /proc/self/cwd/bin`, and the kernel still resolves all of these in the child after `current_dir`. The general limit ("lookup selects a path … resolves after `current_dir` applies") partly covers this, but research/40's "since" reads as if the skip closes the case.

**Minimum fix**, either of:
- **Text only:** "The skip is lexical: an element that reaches a per-process link another way (a symlink, `//proc`) still resolves after `current_dir`."
- **Mechanism (better, and it also covers absolute `only` entries):** exec the file that lookup found by its canonical path, resolved in the parent. Resolving `/proc/self/cwd` in the parent gives the parent's real directory, so the child's `chdir` no longer matters. argv[0] stays as spelled.

### N4. "Never accept a command the real backend refuses" overclaims (nit)

**Where.** research/40:207–208.

The ordered checks guarantee sim and real agree on the program, refused-variable and NUL refusals. The real backend also fails on host state: a missing directory, `EACCES`, `ETXTBSY`, `ENOEXEC` from musl. Models represent those, and the checks don't. **Fix:** "…never accept a command that these checks refuse on the real backend."

## Checked and held

- F1's 03 text matches my proposal word for word, and the companion and 01 summaries agree with it. reference.md's row still holds and is unchanged at 24,985 tokens, by the builder's count, which I did not rerun.
- F3's §4.3 sentence is in the owner document, and PROC-2's note points to it.
- F5's order puts the program check first, so an unlisted program is never told apart from a refused variable. Unattenuated `Proc` still gets the NUL check. `spawn` observes no cancellation, which is unchanged from Codex's fix.
- F7: "after setting up its standard descriptors" is the right order for Rust's `do_exec`.
- F2's research/40 §4 "it knows" wording removes the closure overclaim. N1 is only about the list's accuracy.
- Taking `/proc/` out of lookup diverges from Rust only for `PATH` elements under `/proc/`. That case is too exotic to need a cost note.

DONE
