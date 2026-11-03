# Tide review, fx-kw8.14 round 1 (Opus): converge review round

Candidate: working tree on base 5661fa8 (`git diff HEAD` plus the untracked `conformance/cases/authority/root_main_called_library.rs`, `.beads/` ignored), diff SHA-256
`f2fc03b72bde348bfe5e013f585bd09cc70b1120589e4cecdb0f71178273ae09` (I checked it at the start and the end, and it did not change).

Model: claude-opus-5-5 · Date: 2026-11-03

## Verdict

Clean: no finding blocks. The mechanism is correct and meets the bead's Acceptance. `visit_path` sees every resolved path, including each namespace of a `use` path, so these are all reported under FX0506:
- a call of `main`;
- taking `main` as a value (`let _f = main;`, the case the Acceptance requires);
- an import or re-export of `main`, and a call through that alias;
- a test's `super::main()`;
- a library's generated `main` under `--test`.

The generated entry point and the test harness stay admitted. There are three nits, two of them text-only, and one concern that I recommend accepting without a change. On the new-surface item, `crates.rs:129` `pub fn is_main` should stay (see N3).

## Findings

N1 (nit, text-only, non-blocking). Two texts say the library's `main` is the entry point under `--test`, which is wrong.
- `conformance/cases/authority/root_main_called_library.rs:2-3` says "its crate-root `#[tide::main] fn main` is the entry point there".
- The FX0506 message at `crates/tide-admit/src/walk.rs:1325` opens with "`main` is the entry point".

Under `--test` the harness's own `main` is the entry point. rustc strips the user's `main` of entry status, and fx-nkv.1-r2 notes that the harness never calls it. The real risk is different. The crate is executable, so `Crates::is_entry` trusts that `#[tide::main]` expansion as it would a binary's, and its call to `__rt::enter` mints a real `Root` whenever code calls it.

The same message also says "call it or use it as a value", which does not describe the import mark at `root_main_called.rs:4`.

Suggested fix:
- Case comment: "Built as a test, a library is executable, so the driver trusts its crate-root `#[tide::main]` expansion as it does a binary's; calling that `main` mints a real `Root`, so its tests cannot call it."
- Message: "no code may call or name `main`, which mints the process's `Root`".

N2 (nit, text-only, non-blocking). The module doc at `crates/tide-admit/src/walk.rs:1-14` lists the rules the walk enforces (GLOBAL, AFREE, UNSAFE-1, the ROOT-4 reads) but not the new path rule. Fix: add "and names of the binary's `main` (ROOT-5)" to the list.

N3 (new-surface report, `crates/tide-admit/src/crates.rs:129` `pub fn is_main`): stay.
- **Why it stays:** `Crates` owns crate and entry identity (AP-OWNERSHIP-001). `is_main` keeps the `main` field private next to its twin `is_entry`, and its one caller, `main_named`, needs exactly this question.
- **Alternatives are worse:** inlining it into `walk.rs` would mean making the field public. `pub` matches the other `Crates` methods.
- **Optional simplification (not text-only, does not block, not worth a re-review round on its own):** store `main: Option<DefId>` (`crate_main(tcx).map(LocalDefId::to_def_id)`, or have `crate_main` return a `DefId`). The body then becomes `self.main == Some(def_id)` and the `LocalDefId` import at `crates.rs:8` goes. The current `as_local().is_some_and(..)` is correct; a naive `self.main == def_id.as_local()` would wrongly match `None == None`.

C1 (concern, not text-only, non-blocking; I recommend no change). A library whose crate root has a plain `pub fn main()` helper, without `#[tide::main]`, gets a different verdict in each mode.
- **Lib mode:** `crate_main` is `None`, so nothing is reported.
- **`--test` mode:** the crate is executable, so `crate_main` (`crates.rs:180`) returns that helper. Any call of it, from a test or from the library's own code, is reported as FX0506, although calling it mints nothing.

I recommend accepting this rather than narrowing `crate_main`:
- The error is on the safe side.
- ROOT-5's text, "no code calls `main`", covers it literally.
- A Tide library has no reason to define a crate-root `main`, and renaming it is the workaround.
- Narrowing the rule to "the entry fn in bin mode, or Tide std's expansion" would add a mode split for a case nobody writes.

If the owner wants it recorded, a comment on fx-kw8.9 (which will run library tests) is enough.

## Checked and acceptable

- **The `!generated_by_toolchain(span)` guard (`walk.rs:1324`).** Neither `#[tide::main]`'s output nor the harness names the crate-root `main`; the expansion names only `__tide_main`. The one toolchain path that can name it is core's `#[test]` descriptor for a crate-root `#[test] fn main`, so the guard has a real, if rare, purpose. User code inside `#[test]` and `#[tide::main]` keeps its own spans and is still reported, as the cases confirm.
- **`entry` derived from `main` (`crates.rs:61-64`) instead of `entry_expansion`.** This is equivalent. The old code could pick the harness's expansion for a binary with a plain `main` under `--test`. `is_entry` is only asked about an `Attr` expansion of Tide std's `main`, so that never mattered, and excluding the harness explicitly is clearer.
- **Hygiene.** The harness `main` has a def-site context, so `crate::main` and `super::main` in user code resolve to the user's `main`. The `def_kind` check still runs before `item_name` in `crate_main`, so unnamed items cannot panic.
- **Imports reported beyond ROOT-5's wording ("calls or uses it as a value").** The bead's Scope says any reference "including through a path, a re-export", and README.md:49-50 says "calls or names". The import mark comes free from `visit_use` → `visit_path`; skipping it would need extra filtering. Keep it; no design edit needed.
- **FX0506 does not depend on `tide_std`.** That is consistent: ROOT-5 does not depend on the attribute.
- **Other crates cannot name a binary's `main`.** A library's `#[tide::main]` is FX0520 in lib mode, which the union of modes keeps (`root_main_called_library.rs:5`).
- **`extern "C" { fn main(..); }`** needs the `unsafe` grant (UNSAFE-1), and the lockfile marks such a package unverified, so it is out of scope here.
- **The diag.rs entry.** It is in numeric order, and `Code::ALL` and `as_str` agree, so `--tide-print-codes` lists FX0506. README.md:49-51 is accurate.
- **Case headers.** `root_main_called_library.rs`'s `ROOT-1, ROOT-5, ADMIT-1` mirrors `root_main_library.rs`.

Checks: read-only inspection only (`git diff`, `grep`, `sed`, `br show --no-db fx-kw8.14`). Nothing was rebuilt or rerun; I relied on the recorded checks.
