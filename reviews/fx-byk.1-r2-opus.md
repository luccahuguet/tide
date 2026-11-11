Model: claude-opus-5-5 · Effort: low · Date: 2026-11-11

Verdict: approve

Scope: candidate 480e9a5 on bac07df. `git diff bac07df 480e9a5 | sha256sum` gives
89011261783a02d40c1c71d799732d024c91a4f9a9ef87c07c8e8e6e1fe795c2, which matches the brief. This was a
read-only re-check: I read `git range-diff 0cfe7b2..803b164 bac07df..480e9a5`, the diff of each
rebase-touched file against bac07df, and the current fs.rs, real.rs and docs. I ran no builds and no
probes, and I did not open trials/e1-checked/.

## Findings 1–4

1. **ELOOP for magic links: resolved.** The flag stays at `crates/tide-std/src/fs/real.rs:72`
   (`BENEATH | NO_MAGICLINKS`). The provisional exception is now stated in
   `crates/tide-std/src/fs.rs:3-5` and `design/03-authority-concurrency.md:109-111`: inside the base
   a magic link fails with ELOOP's raw kind, while `write_synced` gives `PermissionDenied` on the
   same path. That is option (a), text only, and the code did not change.

2. **Hand-written `Debug` for `File`: resolved.** At `crates/tide-std/src/fs.rs:327-334`, `Debug`
   formats only `read` and `write`. It does not delegate, and it makes no readlink or fcntl call.
   `FileBackend` (`fs.rs:323`) no longer derives `Debug`, so nothing else can reach std's `File`
   Debug. The fixture asserts the exact output at
   `conformance/cases/authority/files_executable.rs:69-72`. The other `Debug` derives that remain
   (`ReadDir`, `DirEntry`, and `real::Entry` at `real.rs:339`) print only an `Arc<OwnedFd>`. That
   is the fd number with no syscall, and it was already so in round 1.

3. **Path checks in the shared wrapper: resolved.** `Fs::check_path` (`fs.rs:139-143`) runs
   after `check` (cancellation, then Mode) and before backend dispatch in `dir` (:149),
   `open_with` (:176, before `opts.validate()`), `write_synced` (:212), `metadata` (:222),
   `read_dir` (:241), `rename` (:254-255), `remove_file` (:267), `remove_dir` (:278),
   `remove_dir_all` (:289), `create_dir_all` (:299) and `sync_dir` (:309). `open`, `create`,
   `read`, `read_to_string`, `write` and `exists` all go through `open_with` or `metadata`, so each
   path is checked once. `real::Fs::check_path`, the check in `resolve_from`, and the checks in
   `parent`, `create_dir_all` and `remove_dir_all` are deleted. In real.rs the only remaining call
   is `real.rs:223`, which checks a discovered `write_synced` link target. That covers every path
   `resolve_from` can now receive: caller paths are checked in the shared wrapper, parent/prefix
   paths are derived from them, and a joined link target is checked at :223. readlink cannot
   return NUL. Coverage: `files_executable.rs:16-41` (`invalid_paths`) runs at :112 (NUL →
   InvalidInput), at :113 (confined absolute → PermissionDenied) and at :279 (cancellation first).
   The fixture also checks Mode before path (`ro.write("\0")` and `ro.dir("\0", ReadWrite)` →
   PermissionDenied) and path before options (`open_with("/absolute", &OpenOptions::new())` →
   PermissionDenied).

4. **Non-Linux branches: resolved.** No `target_os` cfg remains in `crates/tide-std/src` or
   `conformance/rustc/std/src`. The `cfg(not(linux))` arms in `resolve_from`/`dir` and
   `lookup_flags()` are gone, and every lookup uses `OFlags::PATH` (real.rs:113, :153 and the
   others listed in the report). Unit tests are `#[cfg(test)]` (real.rs:435). Linux-only v0 is
   stated in `fs.rs:6`, `design/03-authority-concurrency.md:113` and
   `design/reference-companion.md:406-407`. Nothing in design/, crates/ or README still says "`dir`
   is unsupported off Linux". What remains: `research/19-files.md:36` and S11 (:212) still
   recommend `dir()` → `Unsupported` off Linux. The coordinator chose to leave the research body
   unrevised (finding 5), and 03 owns the decision, so this is not blocking.

## Rebases: resolved, both sides kept

- `crates/tide-std/src/lib.rs`: Clock's simulation wording and the `std::time` bullet are
  retained. Fs moves out of the identity list (:11-13). The codec/JSON bullet now ends with `;`
  and the real-files bullet follows it (:46-53). "(files," is removed from the not-provided list
  and the time removal from main is kept (:62-63). The shared-definition paragraph names
  `root.rs` (codec/json), `fs.rs`, `test`, `task.rs`, `sync.rs` and `time.rs` (:72-75), and
  `test` is in fact included by conformance (`conformance/rustc/std/src/lib.rs:59-60`).
  `pub mod test;` and `pub mod fs;` both remain (:83-84).
- `conformance/rustc/std/src/lib.rs`: the module doc keeps codec/JSON, io, task, sync and time,
  and adds fs and test (:7-13). The stub `pub mod fs { … }` is replaced by the production
  include (:94-95), and main's `#[path] pub mod time;` follows it intact (:98-99).
- `crates/tide-std/src/task.rs`: main's diagnostic `"real capability used inside a simulation"`
  is kept (:289). The candidate's only changes against main are the `outside_simulation` doc
  comment (:507) and `pub(crate) observe_cancellation` (:527). The round-1 rewording to "real
  authority" is gone, as it should be.
- `crates/tide-std/src/task/simulation_tests.rs`: main's four real-Clock misuses and the five
  task/cell misuses are kept, the three file misuses are added, the array size is 12 (:403), and
  the separate shielded-close check uses main's message (:424-428).
- `crates/tide-std/src/root.rs`: main's `Decode, Encode` re-exports and
  `Clock::new(Time::real())` are kept, with `Fs::root()` and `pub mod ffi` added.
  `README.md:189` names `Fs` alongside `Tasks` and `Clock`, and main's text is otherwise
  retained. `.beads/` is not in the diff, and `crates/tide-admit/tests/build.rs` only adds 41
  lines.

## Nits 6–8 (glanced at, not re-checked)

`flush` now uses `check(true)` (fs.rs:429-431). The shared `Kind` enum is in (fs.rs:565-573,
mapped at real.rs:327ff). The trailing DONE is removed from research/19-files.md. All look as
the report describes.

## Introduced by the fixes or rebases

No defects. Two cosmetic text points, neither of which blocks:
- `crates/tide-std/src/lib.rs:72-73` wraps unevenly ("`time.rs`" ends a short line).
- research §1.1/S11 is stale on non-Linux behaviour (see finding 4); it is a draft record,
  superseded by 03.

DONE
