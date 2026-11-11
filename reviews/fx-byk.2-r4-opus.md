Model: claude-opus-5-5 · Date: 2026-11-11

# fx-byk.2 Opus review, round 4 (read-only converge round)

**Verdict: `request_changes`**

Frozen candidate: HEAD `5cb8dd548a0a7dd01f43a7635437c58b6a701507` on base `8a3b945`, diff SHA-256
`10ae822189c205667de06b761ff599fc42052bcc686877c0a2bc26a53ad39b31`. I checked the hash before and after the
review; the worktree stayed clean.

The design fits the 2026-11-10 decisions and §6.8. The inode store's lifetimes hold, and no lock is held
across a scheduling point. Each call takes one draw and one point, cancellation is ordered correctly, and
the trace is deterministic. One real/sim divergence remains, in the area the brief asked about
(pinned directory capabilities after removal), and one agent-loadable doc still says simulated files are
planned. Both fixes are small.

## Findings

### 1. Blocking: a removed directory that a capability pins lists as empty in simulation, while real Fs returns `NotFound`

**Evidence.** `crates/tide-std/src/fs/sim.rs:482-492` (`Fs::read_dir`) and `:407-411` (the `.`/`..`
branch of `remove_dir_all`) resolve the unlinked directory and read its (empty) entries. Linux refuses
to iterate a dead directory: `iterate_dir` returns `ENOENT` when `IS_DEADDIR`. That check is VFS-level,
so it does not depend on the filesystem. The simulator already knows this state: `Store::unlink` marks
the directory by setting its parent to itself (`:90-93`), and `Store::parent` rejects new children in it
(`:145-148`). The listing paths do not check that mark.

Probe `r4-opus-probe2.rs` (this directory) ran a confined real directory on ext4 against `/t` in
`Root::sim(5)`, linking the candidate's `libtide_std.rlib` (newer than every source file):

| Case | Real | Sim |
| --- | --- | --- |
| `a = fs.dir("a")`; `fs.remove_dir("a")`; `a.read_dir(".")` | `NotFound` | `Ok`, 0 entries |
| same, `a.remove_dir_all(".")` | `NotFound` | `InvalidInput` |
| `c = fs.dir("b/c")`; `fs.remove_dir_all("b")`; `c.read_dir(".")` | `NotFound` | `Ok`, 0 entries |
| `t.remove_dir_all("./")` after `remove_dir("t")` | `NotFound` | `InvalidInput` |

`r4-opus-probe.rs` shows the same result when the pinned directory is replaced by `rename("src", "e")`.
The other operations on a dead pinned directory agree on both backends: `metadata(".")`, `exists(".")`,
`dir(".")`, `sync_dir(".")`, `create_dir_all(".")` (ok), `create_dir_all("x")`, `write`, `create_new`,
`write_synced` (`NotFound`), and `read("..")` (`PermissionDenied`).

So simulation succeeds where the same real program fails, with no symlink, fault or documented
exclusion involved. This is the same class as Codex r1 finding 4, and the bound decision makes it part
of equivalence. The shared case misses it because its only probe of a removed pinned directory is
`child.create_dir_all("later")` (`conformance/cases/authority/simulated_files.rs:227`), which goes
through the guarded `Store::parent`.

**Fix.**
- Give the "unlinked directory" predicate one owner: a `Store::unlinked(id)` helper (`parent == id &&
  id != ROOT`).
- Use it in three places: `Store::parent` (replacing the inline check at `:145`), `Fs::read_dir` (return
  `NotFound` after resolving), and the `.`/`..` branch of `remove_dir_all` (return `NotFound` before
  `empty`, as real `names()` fails before removing anything).
- Add shared regressions after `fs.remove_dir_all("moved")` (`simulated_files.rs:213`):
  `child.read_dir(".")` gives `NotFound` and `child.remove_dir_all(".")` gives `NotFound`, beside
  line 227.
- Rerun both probes; they should report 0 mismatches.

This changes only sim.rs and the shared case, both of which this change edits (v0).

### 2. Blocking (text-only, no re-check needed): the reference companion still says simulated files are planned

**Evidence.** `design/reference-companion.md:391-418`, "File backend behaviour (6.3; fx-byk.1 and
fx-byk.2)", says:
- "Simulation is planned." (`:394`)
- "Planned simulated file calls take one scheduling point…" (`:410`)
- "Planned `world.file` returns `()`…" (`:414`)

This change makes all three available. The section is the companion's list of "recorded std choices",
but it lacks the new real-backend choice that directory `Metadata::len` is 0 on both backends. That
choice departs from Rust's `std::fs`, which reference §6.3 cites ("metadata … as std::fs"). Agents load
the companion, so an agent reading it would avoid `sim.fs` or expect it to panic (AP-CONTRACT-001:
availability must match).

The companion's token-count line is single-holder; this section is not, and the edit does not touch
`design/reference.md`.

**Fix.** Make three edits to the section:
- Drop "Simulation is planned" and the two "Planned" qualifiers.
- Add one bullet: directory length is 0 on both backends, because the host's directory representation
  is not modeled.
- Optionally add that the simulator applies Linux's 4096-byte pathname and 255-byte name limits to each
  syscall-shaped argument.

I agree with the normalization itself: an unspecified length would let simulated tests pass on a value
production does not return. Record the decision on the bead as well.

### 3. Non-blocking: a dead branch in `Store::open`

**Evidence.** `crates/tide-std/src/fs/sim.rs:218-220` (`if needs_directory(p) { return
Err(IsADirectory) }`) cannot run:
- That arm is reached only with `create || create_new`.
- Every such path ending in `/` already returned at `:198-209`.
- A path ending in `/.` fails `normal_name` and returns at `:214-216` (the shared case expects
  `NotFound` for `absent/.`).

**Fix.** Delete the three lines.

### 4. Non-blocking: the shared case does not test a handle on a file that a rename replaced

**Evidence.** The brief names rename replacement as a lifetime risk. At
`simulated_files.rs:176-177`, `fs.rename("source", "value")` replaces `value` while `ro_file` (`:111`)
and `beyond` (`:140`) are open on it, but neither handle is read afterwards. `r4-opus-probe.rs` cases 1-4
show both backends agree here: the old handle reads `old`, and writes through a replaced read/write
handle do not reach the new `value`. So this is coverage, not a defect.

**Fix.** After `:178`, add `ro_file.rewind()` and `read_to_string`, and push the result, which is the
pre-rename contents. That is two lines in the existing program.

### 5. Non-blocking: pure path parsing has no single owner

**Evidence.**
- `real::split` is a pure path helper that both backends now use (`sim.rs:139`, `:362`, `:520` call
  `super::real::split`), so the simulator depends on the real-backend module for parsing.
- The "strip final slashes" idiom appears five times: `fs.rs:120`, `real.rs:86`, `real.rs:399`,
  `sim.rs:150`, `sim.rs:415`. They mix `unwrap()`, `map_or(0, …)` and `let…else` for the all-slash case.
  The r2 regression (validating before stripping) came from exactly this kind of per-site normalization.

**Fix.** Move `split` next to `normal_name` and `check_path` in `fs.rs`, and add one `trim_final_slashes`
helper that all five sites call. This behavior-preserving cleanup can land now or in a follow-up.

### 6. Non-blocking: `File::write_fmt` copies `File::run` by hand

**Evidence.** `crates/tide-std/src/fs.rs:519-544`. The real arm re-implements `run`
(`checkpoint()`, then the `self.write` check), and `bytes` is declared outside the match although only
the sim arm uses it. The sim arm then re-matches `self.backend` with `unreachable!()`.

**Fix.** Use one `self.run("file.write_fmt", self.write, || match &self.backend { Real(_) =>
io::write_fmt(&mut &**self, args), Sim(file) => { let mut bytes = Vec::new(); bytes.write_fmt(args)?;
file.write(&bytes).map(|_| ()) } })`. Behavior is the same, and formatting still happens outside the
store lock.

### 7. Nits

- `crates/tide-std/src/sim.rs:133`: the test is named `unmodeled_identities_never_use_host_files_or_randomness`,
  but its only `fs` assertion now checks the ended-simulation identity of a modeled `Fs`. Rename it or
  add a one-line comment.
- `crates/tide-std/src/fs/sim.rs:34`: the root inode starts with `refs: 2` without saying why. Add a
  comment: one ref for the root's permanent name, one for the base handle.
- `README.md:265-266`: an odd line break in "path reads / and writes, `OpenOptions`,".

## Alternative (non-blocking): parity by construction

All five Codex findings, and finding 1 above, have one cause: the simulator re-implements three real
algorithms (`create_dir_all`, `remove_dir_all`, `write_synced`) and the precedence of open/create
against the store. They agree only through case-by-case mirroring, which the 4,585 probe cases now pin
down. A structural alternative would put the algorithms in `fs.rs` once, over a small set of
syscall-shaped primitives:
- `openat` (`open_beneath`), `mkdirat`, `unlinkat`, `renameat`, `fstatat`, and `getdents`;
- real implements them with rustix, and sim implements them over `Store` with the kernel's per-call
  rules (`IS_DEADDIR`, `PATH_MAX`/`NAME_MAX`, and trailing-slash handling).

Precedence would then match by construction, and each backend would be reviewed only at the syscall
boundary. I do not ask for it here: the current shape is correct apart from finding 1, its size is
modest (the r1/r2 parity fixes added about 60 lines), and the refactor would touch the reviewed real
backend. It is worth taking if a later bead (faults, companion 17) has to hook per-syscall effects, since
it would need exactly that seam.

## Checked and found correct

- **Ownership split.** The wrappers own Mode, `check_path`, `normal_name` for entry-changing calls,
  `OpenOptions::validate`, access flags and cancellation dispatch. The backends own resolution and
  storage. `Fs::dir` and `read_only` keep `Mode` attenuation. A sim `Fs`, `File`, `ReadDir`, `DirEntry`
  or `World` used outside its own simulation panics through `Context::simulated`; this includes a second
  simulation with the same seed (`sim.rs` test at `:727`).
- **Scheduling.** `Tasks::file_op` (`task.rs:544-591`) does the following:
  - checks cancellation once and, if cancelled, records no draw, takes no point and leaves no effect;
  - takes exactly one `fs_rng` draw on its own `tidefile` stream;
  - takes one point (`point` at zero latency, else an uncancellable timer `wait`);
  - applies `f()`, then records one `fs` event.
  `close` is shielded but takes its point and draw. Provided Read/Write/Seek methods either have
  overrides or reduce to one primitive call each. `ReadDir` takes its point until fused.
- **Locks.** The store mutex is taken only inside `f()`, after the point, and is released before the
  event. The runtime lock is never taken while the store lock is held. `Handle` clone and drop lock only
  the store.
- **Determinism.** The inode table, entries and listings are `BTreeMap`s; no addresses or host time are
  involved; spans come from `Location`; paths are written with `to_string_lossy`.
- **Inode lifetimes.** Unlinking with handles open, rename replacement, rename of a directory with a
  pinned capability and an open file inside, rename into a capability's own subtree, external truncation
  under an open cursor, gaps, `set_len`, and recreating a removed name with a live ghost handle: 32 of
  `r4-opus-probe.rs`'s 34 real/sim cases match, and the two mismatches are finding 1.
- **`sim.world.file`.** It checks identity before any store change, rejects `..` above `/`, creates
  parents through `mkdir_all`, and replaces bytes in place. It records one `world` event even on failure
  and panics after the lock is released. It takes no point, time, draw or Mode check.
- **Docs.** `fs.rs`, `lib.rs`, the README and 03 §1.1/§9.2/§9.3/§9.8 state the model's limits accurately:
  no links, crashes or faults; compound calls apply whole; directory length is 0; equivalence is for
  Linux with case-sensitive filesystems. The leftover is the companion (finding 2).

## Proof

All checks ran on Linux `7.2.6-76070206-generic` with an ext4 root, the 1-minute load at about 1.8,
14 GB available, and no active quiet window. Every command used `nice -n 19`, `CARGO_BUILD_JOBS=4` and
`timeout 120`.

- The probe sources are `r4-opus-probe.rs` and `r4-opus-probe2.rs` in this directory. Each was compiled
  with the pinned `rustc --edition=2024 … --extern tide_std=target/tmp/build/std/artifacts/libtide_std.rlib`
  (rlib mtime 19:45:03 -03, newer than `sim.rs`, `fs.rs` and `task.rs`). Both compiled and ran with
  exit 0:
  - probe 1: 34 cases, 2 mismatches;
  - probe 2: 5 cases, 4 mismatches.
  All the mismatches are listed in finding 1.
- I ran no full-workspace build or suite. I did not open `trials/e1-checked/` and wrote nothing to the
  candidate, Git or Beads.

DONE
