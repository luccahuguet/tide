Model: Claude Opus 5.5 · Date: 2026-11-18 · Red-team re-check 2, bb81188

# Red-team re-check 2: fx-ayh2.1, items N1–N4

- **Candidate:** `bb81188ab2926c1a65aa6608869c84bd20684d18`, which amends `1e48e1d`.
- **Round diff:** `git diff 1e48e1d bb81188`, SHA-256 `59b4a3a8dd7c33fd477a44968480ab6721e94774b0970939e4c7af2bc30ded0a`. It touches `conformance/rules.md`, `design/03-authority-concurrency.md` and `research/40-proc-command-v0.md`; `design/reference.md` is unchanged.
- **Scope:** N1–N4, plus whatever their fixes touch. I treated the builder's "Round redteam-b" account as claims and checked each against the diff.
- **Method:** I read the docs and reasoned about them; nothing was compiled or run. Kernel and glibc behavior below comes from my reading of their documented semantics.

## Verdict

**Sound with stated limits.** N1–N4 are fixed. Canonical exec closes every route I named under N3 and does not widen `only`. It opens no new hole. One table row in research/40 still describes the old mechanism (R1), and two edge cases are unstated (R2, R3). All three are non-blocking text items.

## Item status

| Item | Status | Notes |
| --- | --- | --- |
| **N1:** glibc list as cited | **Fixed** | 03:300–308 and research/40:205–210 now refuse "any name in glibc 2.39's `UNSECURE_ENVVARS`". The summary of the 34 names matches the source I read at tag `glibc-2.39`. The separate `GLIBC_TUNABLES` clause is gone. 03 now scopes the list to glibc at that version, naming musl's `MUSL_LOCPATH` and later glibc names as not refused. PROC-2's "03's list, `GLIBC_TUNABLES` included" stays true. |
| **N2:** PROC-3 re-export guard | **Fixed** | PROC-3 (rules.md) and research/40 §12 require the item-list test to cover `pub use` re-exports, and give the reason (`admit_def` admits re-exported items before any rule). A deny glob may only sit beside that test. |
| **N3:** canonical exec (tide-lead's mechanism) | **Fixed: closes the routes, adds no hole** | See below. |
| **N4:** sim parity overclaim | **Fixed** | research/40:220–223 now says "that these checks refuse on the real backend", and lists host-state failures as what models declare. §12 matches. |

### N3 in detail

**Routes now closed.** Std resolves the program to its canonical path in the parent, before the child's `chdir`, and the child execs that path (03:312–321, research/40:173–192). That covers every route I named:
- `/proc/self/cwd` reached directly, or as `//proc/..` or `/./proc/..`;
- a symlinked `PATH` element such as `~/bin -> /proc/self/cwd/bin`;
- absolute `only` entries through any of these.

`/proc/self/cwd` resolves in the parent to the parent's own directory, so the child's `chdir` can no longer pick the binary. Dropping the lexical `/proc/` skip is right, because it was a weaker duplicate of this rule. PROC-2 and research/40 §12 carry the falsifying case: a symlink to `/proc/self/cwd/bin`, plus a `current_dir` holding a different program of the same name.

**`only` is not widened.** Matching still runs on the program as spelled, before resolution, and the canonical path is never matched against the list. Resolution can only change which file runs for a spelling that is already allowed.

**No new window between check and exec.** Before, a symlink component could change between lookup and exec. Now only the file at the canonical path can be replaced, and the docs already state that ("whatever is at the canonical path when the child execs runs").

**Behavior changes, checked against the docs:**
- **argv[0] is unchanged.** It is still the program as spelled. Multicall binaries that dispatch on argv[0] keep working: `ls -> busybox` and rustup's `cargo -> rustup` still run as `ls` and `cargo`.
- **Scripts now see their canonical path.** The kernel hands the interpreter the exec path and drops the original argv[0], so a script reached through a symlink sees its target in `$0` or `__file__`. research/40:186–190 states this. R2 adds the one consequence it doesn't state.
- **`$ORIGIN` and `/proc/self/exe` are unaffected.** The loader already derives `$ORIGIN` from the canonical executable. That is consistent with research/40's remark about `/proc/self/exe`.
- **The extra path walk** per spawn is stated.
- **Under simulation nothing is resolved**, as before, so this adds no divergence between real and simulated runs.

## Remaining items (non-blocking, text)

- **R1. research/40:279 (§5 Symlinks row) still describes the old mechanism.** It says "Followed by the kernel; entries name paths, not file contents." The kernel no longer follows symlinks on the program path; std does, in the parent. Suggested text: "Resolved by std in the parent to a canonical path, which the child execs (§3); entries name paths, not file contents." The residual column ("whoever can replace the file or a link on the path chooses the program") stays true, since links are resolved at spawn time.
- **R2. Scripts that dispatch on their own name now run as their target.** research/40's stated cost names `$0` but not this consequence: a script installed under several names via symlinks, which picks its behavior from `$0`, now behaves as its target for every name. Suggested addition after "names the target, not the link": "so a script that dispatches on its own name through symlinks runs as its target."
- **R3. A failure of the canonical resolution is unspecified.** The file lookup found can vanish before it is resolved, or the resolution can hit `ELOOP`. A deleted file or a memfd reached through `/proc/self/fd/N` has no resolvable path either. Rust's exec would run that memfd; Tide's spawn now fails. One sentence in 03 and research/40 closes it, and it fails closed: "If canonical resolution fails, `spawn`, `output`, and `status` return its error and start nothing."

DONE
