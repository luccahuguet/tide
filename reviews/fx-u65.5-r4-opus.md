Model: claude-opus-5-5 · Date: 2026-11-12

Verdict: approve

This is the round-4 re-check of the alternatives and nits I raised in r3, plus the rebase delta. It is
not a new full review. The candidate is HEAD `6c26363`, one commit on main `f853fa8`, with a clean
worktree. I read the "Fix round 2" section of `report.md`. Finding 1 is now bead fx-u65.7 and finding
4 is on fx-djp, so neither is re-checked here.

## Findings

None. Each re-checked item is resolved:

1. **(a) r3 finding 2: resolved.**
   - Code: `policy.rs:188` adds `deny_as("tokio::spawn", Code::Fx0602, SPAWN)`, and the help string
     is now one `SPAWN` const that the two thread rules share.
     - `walk/authority.rs` sends `Source::TokioSpawn` through
       `denied_key("tokio::spawn".into(), span)`. `denied_key` leaves off the std-only "(Tide's std
       does not export it)" note for that key, as it already did for FX0501.
     - In `walk/habits.rs`, `habit_def` keeps its resolved-Tokio predicate unchanged. It now takes
       the code and help from `policy::lookup("tokio::spawn")`, filtered to `Deny`, so policy owns
       both.
   - Safety of the pseudo-key:
     - No earlier rule matches `tokio::spawn`, so first-match order is safe.
     - No real definition's key can equal it: local items return `Free` before lookup, and a
       resolved Tokio item's key is its defining path.
   - Behavior is unchanged. The verdict, code, severity and span are the same, and the span still
     goes through `source_callsite`, as `report` did before. The only visible difference is the
     message shape: before, the help sat inline in the message ("…; spawn into `tasks.scope`
     instead"); now the message is "`tokio::spawn` is not admitted" and the help comes from the rule.
   - The probe confirms this (below).

2. **(b) r3 nit 3: resolved.** `ignored_result_accept.rs` adds
   `pub fn ok(result: Result<u8, ()>) { result.ok(); }`. The probe accepts the file (exit 0).

3. **(b) r3 nit 5: resolved.** In the new change, the `std::ffi::os_str::*` row has no hunk at all.
   The parent's spacing is restored.

4. **(c) Rebase delta: clean.**
   - Between `42dd813` and `f853fa8`, main changed only fx-yg5.5's four tide-admit files:
     `policy.rs`, `tests/build.rs`, `tests/driver.rs` and `tests/support/reference_allowlist.rs`.
     fx-6kqu changed no crate or conformance file.
   - Diffing `git diff 42dd813 fa660dc` against `git diff f853fa8 6c26363`, after dropping `index`
     and `@@` lines, leaves exactly the fix-round-2 edits listed in items 1–3: the `ok` function,
     the `SPAWN` const and Tokio row, the authority/`denied_key`/`habit_def` routing, and the removed
     os_str hunk.
   - Every other hunk is textually identical, including the test appends at the ends of `build.rs`
     and `driver.rs`. The local exception's rebase condition is therefore met: what the rebase
     changed is only context.
   - Semantic interplay with fx-yg5.5:
     - Its allows (`core::ascii::*` at `policy.rs:284`; `core::io::cursor::Cursor`, `Cursor::*`,
       `SeekFrom` and `SeekFrom::*` at `:275-278`; `std::macros::panic` at `:332`) share no prefix
       with this change's patterns (`tokio::spawn`, `std::thread::{functions::spawn, builder::*,
       scoped::*}`, `std::sync::poison::{mutex,rwlock}::*`, `std::panic::catch_unwind`,
       `core::future::*`, `core::pin::*`). Neither change can shadow the other.
     - `std::macros::panic` and `std::panic::catch_unwind` are distinct keys.
     - fx-yg5.5's witnesses include `Unpin`, which is `core::marker`, not `core::pin`. Its snippets
       consume every `Result` (`.unwrap()`, `?`, `let _`), so FX0702 does not trip them.
     - The author's focused run on this tree passes `reference_kept_families` (policy, driver and
       build), `first_matching_rule_decides`, and `rejects_each_use_outside_the_allowlist_at_its_span`.
     - My probe t5 adds direct evidence: `Cursor`, `SeekFrom`, ASCII methods and
       `core::ascii::escape_default` are admitted next to this change's rules, and a bare `r;` in the
       same file still gets FX0702.

## Probes

One serial batch ran through the machine queue, against the candidate's prebuilt driver and
production std.
- The driver was built at 21:38Z for this tree; no file under `crates/tide-admit/src`, `tide-std` or
  `tide-macros` is newer.
- Command: `(ulimit -c 0; CARGO_BUILD_JOBS=4 timeout 1500 bash /home/lucca/.cache/eon-agents/eon-overseer/heavy.sh nice -n 19 bash <scratchpad>/probe.sh <scratchpad>/probes5 </dev/null)`.
- `probe.sh` runs each file as `package/src/lib.rs`:
  `target/debug/tide-admit --edition=2024 --crate-type=lib --emit=metadata --error-format=short --tide-std target/tmp/build/std/artifacts/libtide_std.rlib --tide-dep-dir target/tmp/build/std/artifacts --out-dir <pkg>/out <lib.rs>`,
  with stdin closed and a 30 s timeout per file.
- The slot was granted as build2.lock (pid 739936). The batch finished at 21:42Z, well before the
  02:55Z window.

Results:
- **t1** (a copy of `spawn_reject.rs`): FX0602 errors on lines 3, 4, 5, 6, 7, 8, 12, 17, 19, 21 and
  25, and rustc errors on lines 3, 4, 6, 7, 8 (E0432/E0433) and 11, 15, 23 (E0432). This matches the
  case's marks exactly.
  - Tokio lines print "FX0602: `tokio::spawn` is not admitted", without the std-only note.
  - std lines keep "(Tide's std does not export it)".
- **t2** (a copy of `spawn_accept.rs`): exit 0. Local same-spelled `tokio` modules, aliases and
  re-exports are still accepted.
- **t3** (a copy of the updated `ignored_result_accept.rs`, with `result.ok();`): exit 0, with rustc
  warnings only on the `Notice;` and `local::Result;` lines.
- **t4** (`use tokio as runtime; runtime::spawn(..)` plus `std::thread::spawn`): E0432 on line 1;
  FX0602 on line 2 (`tokio::spawn`) and on line 3 (std).
- **t5** (fx-yg5.5 interplay): exit 1, with a single FX0702 on line 2 (`r;`). The `Cursor`,
  `SeekFrom`, ASCII and `escape_default` uses got no diagnostic.
- Help text doesn't print in `--error-format=short`. That the help comes from the rule is shown by
  the code path (`denied_key` → `reject` → `diags.report(.., rejection.help)`, shared with every
  deny rule), not by this probe.

Not run: full workspace or corpus suites, or a rebuild. No edits, staging, commits or `br` writes.
Nothing under `trials/e1-checked/` was opened or searched. DONE
