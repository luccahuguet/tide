# fx-gcs.2 review, round 2 (Opus)

- Reviewer: Opus, round 2, converge round (fresh eyes and simplification pass
  that fixes what it finds)
- Base: `e2a7063`; candidate is the worktree `tide-fx-gcs.2` diff plus
  intent-to-add files
- Date: 2026-10-21

## Verdict

Clean: nothing changed. Every round 1 blocking finding is fixed in the tree:
`drop_may_dangle.rs` carries `E0658 FX0901` with a `// SAFETY:` line,
BORROW-3 names `spawn_isolated` closures instead of the passed-down scope
(02 §3.7 and §5.3, rules.md, and the `scope_passed_*` rule lines), ORIGIN-6
excepts CLOSURE-3, and CLOSURE-2 is narrowed to elided parameter annotations
with the named-lifetime and inferred forms in `closure_hr_accept.rs`.

## Changed

Nothing.

## Non-blocking items left alone

- `reviews/fx-gcs.2-r1-codex.md:27` and `:43` end in two spaces (Markdown
  hard breaks), which `git diff --check` reports. The file is an existing
  review, which this round does not edit; the author should strip the
  trailing spaces before commit.
- `conformance/README.md:101-102` ("In `rules.md`'s first column ...")
  restates rules.md's own column description (`rules.md:10-12`). Harmless
  duplication, a nit.
- `dyn_default_reject.rs:8` boxes a `&'a dyn Fn()` into `Box<dyn Fn()>` and
  marks only rustc's `error`. Whether FX0203 (reference 3.5, worded for
  closures) covers a boxed reference to a closure is unclear, so the mark is
  left as is.
- Round 1's N3 (weak `life_const_reject` and `loan_kill_reject`), N4
  (elision rule 2's wrong-input rejection), N5 (higher-ranked
  instantiation in VAR-1), N7 (ID order), and N9 (rewrap) remain as round 1
  recorded them.

## New `pub` surface

- Cases: `pub` items are how a case keeps its items used, as every existing
  case does.
- `Tasks::spawn_isolated`: used by the SCOPE-3/BORROW-3 cases; its signature
  matches 02:689 and 03:1101.
- `Panicked`: `spawn_isolated`'s error type (03:1116). Its `Debug`,
  `Display`, and `Error` impls match every other stub error type.
- `Task`'s `*const ()` marker and `unsafe impl Send`/`Sync` for
  `U, E: Send`: required by 02 §8.1's `Task` row (`Sync` when `U, E: Send`,
  not when `Sync`), which `send_table_accept.rs:39-42` and
  `send_table_reject.rs:27-30` check. No marker type gives that row while
  keeping `U` and `E` covariant.

## Checks

- `/proc/loadavg` 16.2 before the run.
- `conformance/run.sh </dev/null`: exit 0; 79 PASS, 12 AWAITS CHECKER, no
  FAIL.
- `conformance/run.sh --trace`: each of the 46 02 IDs has an accepting and a
  rejecting case, except SEND-2's accepting case (known, awaits manifests).
- Every `**ID` lead in `design/02-memory.md` has a rules.md row; no ID is
  defined twice.
- The new rules.md rows cite the right lines at the snapshot `a3b0177`
  (spot-checked MOVE-4, LOAN-1/3/4, LIFE-5, VAR-1/2/3, SEND-2, SCOPE-2/3/4,
  SHARED-1).
- `git diff --check e2a7063`: only the two trailing-space lines above.

## Fingerprint

`fb3fbbab2055c06327642b237925212dfea403a2862467b248079d87e170f24d`, computed
before this file was written (the tree this round reviewed, which it left
unchanged).
