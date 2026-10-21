# Tide review, fx-e3t round 4 (Opus): rebase and stub, narrow confirmation

Reviewer: Opus, round 4 (narrow confirmation), read-only, candidate commit
`ac29d69` on main `0082d96` (branch `fx-e3t`), `git diff main..HEAD`
SHA-256 `5b7cd232ad6efd821803c50bf531433a784cf27bcef7cf060171ff955a1a849a`,
2026-10-21. `git diff main..HEAD --check` passes. `scripts/count-tokens.sh
design/reference.md` gives 24,996, which matches the companion's count line
and is under D14's 25,000. `conformance/run.sh </dev/null` exits 0: 3 PASS,
2 AWAITS CHECKER (`admit_atomic`: FX0520, `lock_payload_shared`: FX0606).

Scope: the two changes since round 3, the rebase from `76dfe06` onto
`0082d96` (fx-t99, fx-gcs.1) and the one stub line in
`conformance/rustc/std/src/lib.rs`.

## Verdict

**clean.** The stub change is correct and complete, no other stub signature
contradicts fx-e3t, and the rebase leaves no stale text in `design/` or
`conformance/`. No finding blocks.

## Blocking findings

None.

## Non-blocking findings

None.

## Reasoning

- **The stub line.** `Fs::exists(&self, p: impl AsRef<Path>) ->
  io::Result<bool>` matches 03 §1.1's new `exists` note
  (`fs.exists(p) -> io::Result<bool>`, Rust's `std::fs::exists`), the
  reference's `Fs` block, which now lists `exists` among the methods "as
  std::fs", and the companion's `fs.exists(path)?` replacement for
  `path.exists()`. No case calls `exists`, so no case changes, and the stub's
  header ("the signatures `design/reference.md` and 02/03 give") stays true.
  The diff against `trials/e1-dialect/rustc/std` in `pub mod fs` is that one
  line, so the frozen trial copy, and the trial program `backup.rs` that
  calls `exists` as a `bool` against it, stay the trial's record, as the
  conformance README says.
- **Other stub signatures.** The stub defines no `read_owned`,
  `write_owned`, `BufError`, `File`, sockets, `Command`, `output`, `status`,
  or `Fs::write`, so none can contradict fx-e3t; missing items are expected
  (the stub covers only what cases use). `write_synced` returns
  `io::Result<()>` as the reference gives. `sim::Reply::status(code: u16)`
  is the simulator's HTTP reply constructor, unrelated to
  `Command::status`. `io` re-exports Rust's `Error`, `ErrorKind`, and
  `Result`; when a case needs `BufError` or `ErrorKind::Cancelled`, the case
  bead adds them per the README's rule.
- **Rebase interaction.** fx-t99 edits 02 §9's not-`Pure` list (simulated
  roots, simulator handles, `LogConfig`) and companion §17's `NodeRoot`
  note. fx-e3t's `BufError` is `Send + Sync + Pure`; it holds an `io::Error`
  and a `Vec<u8>` and acts on nothing, so it falls in none of 02 §9's
  not-`Pure` classes, and fx-t99 adds none that would catch it. 03's §9.5
  example now calls `backup.fs().exists(key).unwrap()`; `SimNode::fs()`
  returns `Fs` in the stub and the companion, so that compiles in shape. The
  grep for `exists`, `output`, `status`, `BufError`, `read_owned`,
  `write_owned`, and "never interrupted" over `design/` and `conformance/`
  finds only fx-e3t's own text, the 03 capability table rows (which already
  list `exists`, `read_owned`, `write_owned`, `output`, `status`), and
  unrelated uses (04's `exists` quantifier, "exists" as prose). The one
  "never interrupted" (reference 7.2) is fx-e3t's line and agrees with
  TASK-5.
- **`rules.md` snapshot.** Its header says the inventory was "taken at
  `a3b0177`", and its 03 line citations (TASK-3 at 03:1177, TASK-5 at
  03:1223, TASK-6 at 03:1231) are true at that commit. It claims nothing
  about the current tree's lines, so the agreed policy holds. fx-e3t adds no
  compile-time rule and no rule ID, so no row is missing: TASK-3, -5, and -6
  are listed as run-time rules outside the corpus, which fx-e3t's changes
  keep them.
