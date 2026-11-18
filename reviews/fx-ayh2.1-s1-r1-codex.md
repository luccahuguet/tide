Model: gpt-6.1-sol (high) · Date: 2026-11-18 · Candidate 998a0cb (design, docs)

## Verdict

**changes required** (`request_changes` under GOVERNANCE.md). Confidence: high. The bounded profile fits A1, but three design statements need correction before this candidate is Codex-clean. No override.

## Scope

- Bead: fx-ayh2.1, stage 1 only; read `br --no-db show fx-ayh2.1`, `br --no-db comments fx-ayh2.1`, and `br --no-db show fx-ayh2`.
- Worktree: `/mnt/build/eon-agents/worktrees/tide-fx-ayh2.1`, branch `fx-ayh2.1`.
- Candidate HEAD: `998a0cb9c0affd1389c73aba00928abb36810855`.
- Base main: `91a2f36633e1400db52c7cc347f72d78ceadbd21`.
- SHA-256 of the exact output of `git diff 91a2f36..HEAD`: `10186e5f49d0188164a7bb2448158f2c1cd21920fc7b18e4c11c5e559c6a2146`.
- Reviewed all seven changed documents and relevant surrounding capability, cancellation, lifecycle, replay, admission, governance and implementation-status owners. `target/work/report.md` supplied claims, not proof. Checked the installed pinned nightly's Rust source and current primary Rust documentation.
- Read-only review: no source edits, commits or Beads writes. This requested report is the sole write.

## Blocking findings

### 1. PROC-ONLY makes spawn observe cancellation, contrary to the retained lifecycle contract

**Locations:** `design/03-authority-concurrency.md:290`, `research/40-proc-command-v0.md:155`; conflicting owner text at `design/03-authority-concurrency.md:2401` and accepted mechanics at `research/30-sim-world-proc.md:380`.

The new rule says that `spawn`, `output` and `status` check the allowlist **after the cancellation check**. The retained rules explicitly say that spawn is not a cancellation checkpoint and process creation adds no checkpoint. A permitted `cmd.spawn()` in an already-cancelled scope therefore has two incompatible outcomes: create the owned child, or observe cancellation and start nothing. This also changes which error an unlisted spawn returns and whether model lookup occurs.

**Minimum fix:** apply the cancellation-before-allowlist ordering only to `output` and `status`; state that `spawn` checks the allowlist before resolution/model lookup without observing cancellation. Preserve §9.9's existing scheduling point. Correct both new formulations. The cheapest falsifier is the named observation of permitted and refused `spawn` in an already-cancelled scope, compared with pre-cancelled `output`/`status`.

### 2. Immediate simulated kill removes a real lifecycle race and contradicts the accepted mechanics

**Locations:** `research/40-proc-command-v0.md:246`, mirrored at `design/03-authority-concurrency.md:2248`; compare `design/03-authority-concurrency.md:2271`, `:2349`, `:2377` and `research/30-sim-world-proc.md:395`.

The profile says `kill` on a running child “ends it at once.” Rust's Unix implementation submits SIGKILL and returns without waiting; successful submission does not establish termination or waitability. The accepted model distinguishes submission, delivery/termination, and reap, with ready model actions competing through the existing scheduler. Making kill synchronously publish a zombie silently strengthens the v0 model: `child.kill()?; child.try_wait()?` can still return `None` on Linux, while this simulator would always return a status. It can also hide races between a normal exit and kill. Moving general signals and fault controls to Later does not make this ordinary lifecycle race disappear. See [Rust's Unix process implementation](https://doc.rust-lang.org/src/std/sys/process/unix/unix.rs.html) and [kill(2)](https://man7.org/linux/man-pages/man2/kill.2.html).

**Minimum fix:** retain kill as direct-child SIGKILL submission, with termination a ready action on the existing scheduler; no new public signal or fault API is needed. Do not guarantee that kill's return makes the child waitable. Preserve immutable status when normal exit wins, and the no-code status when kill wins. Align the subset summary and research note. The cheapest falsifier is a seed witness in which a successful kill is followed by `try_wait == None` before termination, then a successful wait, alongside the termination-first order.

### 3. The cost case understates the lookup divergence, and its POSIX explanation is false

**Locations:** `research/40-proc-command-v0.md:162`, `:311`; operative rule at `design/03-authority-concurrency.md:294`.

The admission cost says lookup changes which binary runs **only** when a command sets its own PATH. Tide also skips inherited empty and relative PATH elements for every Proc. For example, with start-time `PATH=tools:/usr/bin`, Rust can run `./tools/tool`, while Tide skips it even for an unattenuated command that never calls `env`. The selected binary can change or lookup can fail. The description that POSIX reads both empty and relative elements “as the current directory” is also wrong: `tools` means the `tools` directory relative to the working directory, not the working directory itself. [environ(7)](https://man7.org/linux/man-pages/man7/environ.7.html) describes prefix concatenation and the special empty-prefix case. [Rust's Command documentation](https://doc.rust-lang.org/std/process/struct.Command.html) describes its child-environment lookup.

**Minimum fix:** correct the cost paragraph to include absolute-only start-time lookup and the resulting inherited-PATH/current_dir differences, rather than claiming only command-level PATH overrides differ. Distinguish empty elements from directories resolved relative to cwd in §3. Keep the selected Tide rule; this finding does not ask to loosen it. The smallest check is the `PATH=tools:/usr/bin` example with no command environment override.

## Non-blocking findings

### 4. Make the simulator's default-stdin limitation explicit at the first example

**Locations:** `research/40-proc-command-v0.md:254`, `design/reference-companion.md:649`.

The unsupported inherited-stdin mode is stated, so it is a documented limit rather than an authority hole. Its common consequence is easy to miss: Rust-shaped `proc.command("tool").status()` and `.spawn()` inherit stdin by default and therefore fail simulator coverage unless callers configure a supported stdin mode. This affects ordinary ports, not just advanced routing.

**Minimum improvement:** add one sentence or a `status` example using `.stdin(Stdio::null())`, applied identically on both backends. No simulator expansion is requested.

## Reasoning

- **Ownership, tiers and status:** 01's tier table and D7/D15 record A1 and point to the bounded profile. The reference retains `planned`, companion 20 is planned, §9.9 explicitly says unimplemented, and PROC-1/2 are planned stage-2 obligations. Keeping those labels is correct for a docs-only stage. Proc::signals/Signals, PTYs and the remaining signal APIs are Later in the active tier owners; Proc::exit retains its existing planned v0 specification. Research/30's dated decision history is qualified by its new tier-update pointer.
- **Authority:** exact byte spelling rejects alternate absolute and relative spellings before lookup; list intersection preserves attenuation. Tide-owned lookup prevents a command PATH or current_dir from selecting the initial bare-name executable. The design explicitly accepts writable startup-PATH directories and replaceable files/symlinks as limits (`research/40-proc-command-v0.md:211`, `:215`). It refuses command-set LD_*/DYLD_* variables, explicitly passes inherited ones, and admits argument/non-loader-variable execution as a limit. The fd sweep excludes inherited/FFI-opened descriptors above stderr; deliberate Stdio transfers remain explicit. These are design obligations, not runtime proof.
- **Child authority:** the owning documents plainly say children are not capability-checked and allowed shells/interpreters or programs such as git with command-bearing arguments/variables can exercise full authority. Thus `only(["git"])` is intentionally not a sandbox. No new silent authority escape was established by this review of the stated design.
- **Rust:** the candidate is correct that Unix Command searches a command-set PATH and that kill on an exited child returns Ok. The latter is stated in [Rust's Child documentation](https://doc.rust-lang.org/std/process/struct.Child.html#method.kill) and the pinned nightly's source caches collected status before any PID signal. The wrong rule is immediate completion of a kill on a running child, finding 2. CAP-1/TD0501 remains expressly required despite Tide Command lacking `new`; stage 2 must implement that diagnostic path rather than relying on rustc's E0599.
- **Simulator and replay:** apart from findings 1–2, the subset keeps per-Child status ownership, retained stdin rules, drop semantics, ordered bounded pipes, coverage failures without host fallback, whole-node crash semantics and existing timer/scheduler ownership. `ProcessModel::missing()` is a provisional declarative spawn-failure model, consistent with research/30's explicit distinction between configured spawn failure and missing coverage; there is no existing implemented builder API to contradict. Profile/schema, inline model data, capacity and sub-stream seed, together with §9.9's bounds and D15/REPLAY-1/2 source/build/configuration identities, are sufficient design identities. Actual byte-identical replay and mismatch refusal remain stage-2 proof.
- **Admission:** one capability constructor and Rust-shaped handles avoid a second process API or new grammar. The note names the real lookup/fd work, model interpreter, scheduler integration and manifest cost. This supports admission once the behavioral contradictions and understated compatibility cost are corrected; no alternative redesign is needed.
- **Checks on this unchanged candidate:** `scripts/count-tokens.sh design/reference.md` returned **24985**, within D14's 25000 budget with **15 tokens remaining**. `git diff --check 91a2f36..HEAD` passed. Initial tracked working tree was clean. No Rust compilation or runtime suites were run: this pass reviews a docs-only design, whose implementation does not yet exist. It does not replace the separate authority red team or the Opus review.

DONE
