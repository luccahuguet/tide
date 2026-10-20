# Tide adversarial/consistency review, round 60: the unreviewed-branch guarantee (fx-arx)

## Verdict

**approve_with_alternatives**

Round 59's three blocking findings are closed, and so are N1 to N8. The
answer on N9's core hole-table entry for `global_asm!` holds up: the core's
6.9 lists `core::arch` as `FX0520` (`reference.md:553`), and that is where
`global_asm!` lives. No fix from rounds 54 to 58 was dropped.

I found no escape that RUN-4 leaves out and RUN-1 to RUN-3 fail to close.
I found no forged result outside RUN-6's caveat and no `run: covered` on a
branch that reaches an unreviewed hole. §2.6 reads coherently, and it meets
each of the bead's acceptance criteria (see "Coherence and acceptance").

One alternative is worth taking before merge, though it does not block
(N1). Round 59's N1 fix binds a verdict to a test run by snapshot hash,
but the procedure in §19 compares the hashes only after the run, reads
them from output the child can print into, and shows a 24-bit prefix. An
atomic `--snapshot <hash>` gate on `tide test` and `tide sim`, in the
pattern of D14's write requests, would fix all three. The other notes are
text-only fixes or details for the implementation.

## What was reviewed

The uncommitted working tree on 2026-10-20 against HEAD `87f9cd3`:
`git diff -- design/ README.md`, SHA-256
`7c747137530bcc383ab116de197f6db46ac84fa35d1befe258e28c50f8aeb438`.

The changed text:

- 03 §2.6 at `03:423-628`, ADMIT-1's `global_asm!` entry at `03:410-412`,
  and REPLAY-1 at `03:1477-1482`;
- D7 at `01:425-431`, D11's behavior-diff bullet at `01:719-723`, the D13
  bullet at `01:830-861`, and the tier row at `01:82`;
- companion §19 at `companion:495-521` and its header at `companion:3-16`;
- `reference.md:5-9`, `:951-956`, and `:958-967`;
- `README.md:150-151`.

I checked it against:

- 01 D6 (`01:380`), D14 (`01:873-881`), and open question 5
  (`01:1128-1129`);
- 03 §1.1 to §1.3 (`03:140-213`), ROOT-1 to ROOT-5 (`03:219-245`), ADMIT-1
  and ADMIT-2 (`03:378-421`), FFI-1 (`03:876`), §9.6 (`03:1448-1467`), and
  REPLAY-2 (`03:1483-1486`);
- 04 §11 (`04:1188-1229`);
- `reference.md` 6.9 (`:545-559`) and 12.4 (`:1018-1052`);
- the spike driver (`spikes/d18/admit/scripts/tide-rustc:18`, `:32`);
- rounds 54 to 59.

I did not count tokens.

## Round 59 closure

- **B1 (`tide eval`).** Closed. Companion §19 now ends "This covers `tide
  check`, `tide test`, `tide sim`, and `tide review` only: `tide run`,
  `tide eval`, a built binary, ..." (`companion:518-521`). 03 adds "`tide
  eval`'s root and sandbox are not yet set (D14)" (`03:605-606`), which
  agrees with D14's definition (`01:873-874`) and open question 5
  (`01:1128-1129`).
- **B2 (review starts no process).** Closed in the rule: "starts no process
  from the candidate's build except the behavior diff's RUN-3 children
  below" (`03:556-557`). The conformance row did not take the matching edit
  (N3). It is accurate for v0, since the behavior diff is v1.
- **B3 (reviewed base).** Closed in all three places. RUN-5 says "The base
  must be a reviewed revision, never a commit of the candidate itself"
  (`03:562-563`). Companion §19 says "where `<base>` is a reviewed revision,
  never a commit of the branch itself" (`companion:503-504`). The core says
  "against a reviewed base" (`reference.md:966`).
- **N1 (snapshot binding).** Closed as a rule: "a verdict applies only to
  runs with the same hash" (`03:560-562`), and §19 makes the match a
  condition (`companion:513-514`). The procedure has the gaps in N1 below.
- **N2 (`stat` residual).** Closed: "whether it exists, and its type, mode,
  owner, size, and times" (`03:506-507`).
- **N3 (`execve`).** Closed: "succeeds only for what Landlock lets the child
  execute, its own binary with the loader and libraries it maps"
  (`03:497-499`).
- **N4 (fail closed).** Closed: "If the harness cannot install the level
  `tide review` reported, it fails the run" (`03:512-513`). See N4 below
  for a wording point.
- **N5 (Bend supervisor).** Closed: the server is one "that the harness
  starts, restarts, and stops under a sandbox of its own, connected to the
  child by an inherited descriptor" (`03:536-538`).
- **N6 (replay input, output path).** Closed. The harness reads "the
  contents of any `--replay` manifest or `--faults` plan" and passes them
  in (`03:480-481`), and it "creates a fresh output directory for each run"
  (`03:461-462`). N2 below covers a leftover in the row and in §9.6.
- **N7 (verdict spoofing).** Closed: "Its first line, and fields under
  `--json`" (`03:558`), and §19's "its first line says" (`companion:513`).
- **N8 (traces).** Closed: "forge results and traces, a `#[protected]`
  test's results and v1's behavior diff included" (`03:586-587`).
- **N9 (wording).** Closed. "Sandboxed child processes" replaces
  "in-process" (`03:609`). D13 now says "System V IPC" (`01:840`). Hole 5
  names "the linker" (`03:542`). The core hole table is answered by 6.9's
  `core::arch` (`reference.md:553`). It also gained `#[unsafe(..)]`
  (`reference.md:953`).

## Earlier fixes retained (rounds 54 to 58)

- **R54.** The package-wide clause (`03:567-571`) and the descendant row
  (`03:627`). ADMIT-1 cited in RUN-1 (`03:436`). The build inside the
  session sandbox (`03:598-600`).
- **R55.** Symbolic links (`03:458-463`, row `03:624`). Lockfile identity
  (`03:568`). Operator registries (`03:446`, `03:471`). One code for
  `global_asm!` (`03:410-412`, `03:521`). Unshared artifacts
  (`03:599-600`). The harness and derives in hole 5 (`03:541-546`). Side
  channels (`03:554`). Inline manifests (`03:1480-1482`).
- **R56.** The transitive clause (`03:569`), the Rust crate and Bend clause
  (`03:565-566`), the pipes and new session (`03:476-479`), and
  compile-time-only review (`03:555-557`).
- **R57.** The allowlists, with "Every other call fails" (`03:499`). The
  pre-run sandbox report (`03:509-510`). The RUN-5 covered row (`03:627`).
  The behavior-diff condition (`03:572-574`, `01:721-723`). The exposure
  sorting (`03:592-598`). Core dumps (`03:477`). Tests never started
  (`03:578`). The kernel in hole 5 (`03:547-548`). Regular files and
  directories only, no links (`03:488-489`). "Refused (§9.8)"
  (`03:508-509`).
- **R58.** The Bend descriptor (`03:481-483`, `03:536-538`). Metadata reads
  (`03:494`, `03:502-503`). Printed commands (`03:578-581`). System V IPC
  (`03:501-502`). `clone3` (`03:496-497`). Escaping (`03:582-585`). "Only
  if it is a VM" (`03:598`). The summaries in D13 (`01:850-852`) and
  §19 (`companion:515-517`). The Landlock-less `partial` row (`03:625`).

## Blocking findings

None.

## Non-blocking notes

### N1. The snapshot check comes after the run, reads text the child can print, and compares a 24-bit prefix

**Where.** RUN-5: "`tide test` and `tide sim` report their snapshot hash
too, and a verdict applies only to runs with the same hash"
(`03:560-562`). Companion §19: "Run the branch's tests ... only when its
first line says `run: covered` and `sandbox: full`, and the snapshot hash
it names matches the test run's" (`companion:512-514`). The example prints
`snapshot 9c41e2…` (`companion:507`).

**Problem.** There are three gaps, each small.

1. *Order.* The test run's hash is known only once the run reports it,
   which is after its children may already have run. §19 gates the run on
   a value that exists only after the run, so an agent can check the match
   only after the fact. In a fleet, the orchestrator often tests in the
   delegate's live worktree. A delegate that swaps a granted package's
   file between `tide review` and `tide test`, or during the build, gets
   a not-covered tree run as if covered. The mismatch shows up only
   afterwards. (By inference, the hash also has to be computed over the
   bytes the file loader served. Otherwise a swap during the build passes
   too.)
2. *Source.* §19 does not say where to read the test run's hash. RUN-6
   protects only the result channel (`03:575-576`). A child's standard
   output can print a plain-text line `snapshot 9c41e2…` that contains no
   control character. Whether `tide test` keeps its own lines apart from
   the child's output is not stated.
3. *Width.* If agents compare the hash as printed, a delegate can grind a
   comment until the tree's hash shares a 24-bit prefix with the reviewed
   tree's. That takes about 2^24 tries.

The harm is bounded. A swapped tree still runs under RUN-3's sandbox, so
what is lost is the precision of the `covered` label, not the confinement.

**Alternative (smallest).** Use D14's pattern for write requests: "every
write request carries its base hash and applies atomically or not at all"
(`01:880-881`). Add a sentence to RUN-5: "`tide test --snapshot <hash>` and
`tide sim --snapshot <hash>` refuse to build unless the tree's hash, taken
over the files the build reads, equals `<hash>`". In §19, write "and run
them with `--snapshot` set to the full hash from `--json`". This replaces
the comparison after the fact.

### N2. Output-directory leftovers: the rule and the row differ on a committed file, and §9.6 shows fixed paths

- RUN-2 fails the run only "if a link holds any part of its path"
  (`03:462-463`). Its row expects "a link or file committed at the output
  directory's path" to fail the run (`03:624`). Either the rule says "a
  link or file", or the row drops "or file" and states what the harness
  does with a committed file there.
- §9.6 prints fixed paths, `target/sim/acked_writes_survive-3141.manifest.json`
  and `.ndjson` (`03:1466-1467`). With a fresh directory per run
  (`03:461-462`), either each run gets its own directory, such as
  `target/sim/<run>/`, or a new run removes the last run's replay
  manifests. That includes the one a printed replay command names. Say
  which, or mark the example's paths as illustrative.

### N3. B2's row and D11 lag the rule

- The RUN-5 row still says "An access trace of `tide review` shows no
  process started from the candidate's build" without a condition
  (`03:627`). The rule makes an exception for the behavior diff
  (`03:556-557`). Round 59's suggested text, "for a candidate not covered",
  or "(v0)", fixes it.
- D11 says the behavior diff "runs only for a branch that 03's RUN-5 calls
  covered, unless the operator asks" (`01:721-723`). RUN-5 adds "on a host
  with `sandbox: full`" (`03:573`). 01 is the decision record, so add the
  condition there too.

### N4. "The level `tide review` reported" reads as state carried between commands

RUN-3 fails the run "if the harness cannot install the level `tide review`
reported" (`03:512-513`). An operator may run `tide test` without `tide
review`, or on a different host. The intent, a probe of this host's kernel
that both commands share, is clearer as "the level it reports for this
host (the level `tide review` reports here)".

### N5. Hole 5's list of input bugs names only `tide review`'s parsers

Hole 5 lists bugs that input can exploit "in the compiler, in the harness,
..., or in `tide review`'s parsers of the manifest, lockfile, `tide.seal`,
and approval records" (`03:544-547`). Three are missing:

- `tide check` and `tide test` parse the same manifest and lockfile;
- the harness parses the `--replay` manifest and the `--faults` plan
  (`03:480-481`), which arguably falls under "the harness";
- the package fetcher unpacks the registry archives that the candidate's
  lockfile picks (`03:446-447`). If the operator's registries accept public
  uploads, the attacker writes those archives, and an extraction bug such
  as tar path traversal runs at build time, in the session sandbox.

The operating guidance's list of what "meet[s] only the session's sandbox"
(`03:596-598`) also leaves out the fetcher. Hole 5's heading is "the
toolchain", so this is a gap in the enumeration, not a new escape. Write
"bugs that input can exploit anywhere in the toolchain, among them the
compiler, the package fetcher, the parsers of the manifest, lockfile,
replay manifest, fault plan, `tide.seal`, and approval records, and the
harness, which reads ...".

### N6. Base resolution in a shared repository

RUN-5 now requires a reviewed base (`03:562-563`), but `<base>` is usually
a ref name. Git worktrees share `refs/heads` and the repository config, so
a delegate with commit access in a worktree of the operator's repository
can move `main`. `tide review main` then compares against the delegate's
commit. Round 57 put `.git/config` outside the threat model, since the
candidate is "the branch's tree" (`03:427`). This case is in the same
class. Two text changes would close it. §19 could say "name the base by a
commit you fetched from the protected remote, not a ref the branch's author
can write". RUN-5's first line could print the resolved base commit next
to the snapshot hash, which round 59's B3 also suggested as optional.

### N7. Implementation: what the dynamic loader and std read at startup

Landlock lets the child open "only the binary and the libraries it loads"
(`03:487-488`). At startup, `ld.so` opens `/etc/ld.so.cache`, and glibc's
`pthread_getattr_np` reads `/proc/self/maps` for the main thread. Rust's
std calls it to place the stack guard. `available_parallelism` reads
cgroup files, and the default `par-threads = "cores"` (`03:188`) uses it.
These points come from glibc's and std's behavior and are not in the
design. Each read fails gracefully or can be avoided with a static test
binary. The implementation must choose, and if it admits `/proc/self/maps`
it must not admit `/proc/self/mem`. No text change is needed. This belongs
in the v0 sandbox's design notes.

### N8. Small wording

- RUN-5's "changes neither a source file" (`03:567`) would be exact as
  "adds, removes, or changes no source file". A new file can join a
  granted package's library crate only through an edited `mod` line,
  since `#[path]` is rejected, so this is wording only.
- D13 summarizes the Bend clause as "no ... Bend kernel" (`01:849`). RUN-5
  says "its pin, kernel sources, or law files" (`03:566`).
- Several lines of the diff were not rewrapped: `03:499`, `03:507`,
  `03:513`, `03:538`, `03:587`, `03:598`, `03:610`, `01:852`, and
  `companion:502`.

## Escape attempts that failed

- **A tree swapped in a live worktree between review and test, or during
  the build.** This is N1. The swapped tree still runs under RUN-3, so the
  confinement holds and the `covered` label is wrong.
- **A child printing the reviewed tree's snapshot line or a fake summary.**
  RUN-6 keeps the result channel honest (`03:575-576`). The text-level
  spoof is N1(2).
- **A registry archive that the lockfile picks and that exploits
  extraction.** This is N5: a toolchain bug at build time, inside the
  session sandbox (`03:598-600`).
- **Self-`execve` with a crafted environment, so the re-executed binary
  builds a real `Root`.** Calling `execve` already needs hole 1 or 5. I
  infer that Landlock and seccomp persist across exec, so such a `Root`
  is still confined by RUN-3.
- **`clone` with `CLONE_VM | CLONE_VFORK` but no `CLONE_THREAD`.** "Only by
  `clone` with thread flags, which seccomp can inspect" (`03:495-497`) has
  to mean the filter requires `CLONE_THREAD`, which is an implementation
  point. Reaching `clone` also needs hole 1 or 5.
- **`memfd_create` with `execveat`, `userfaultfd`, `perf_event_open`, or
  `bpf`.** None is on the allowlist (`03:491-499`), so each fails.
- **`mmap(PROT_EXEC)` of a file the child wrote in the output directory.**
  By inference, Landlock's execute right governs only `execve`. Writing
  and mapping that file already needs hole 1 or 5, which give native code
  anyway.
- **Libraries pulled in by `#[link]`, whose constructors run at load.**
  Extern blocks need the `ffi` grant (FFI-1, `03:876`), so this is hole 2.
  Round 57 found the same.
- **Downgrading the edition to make `#[no_mangle]` safe.** D18 fixes Rust
  2024 (`01:972`), RUN-2 lets the tree choose no rustc flags
  (`03:467-468`), and the spike driver forces `--edition 2024` with
  `-F unsafe_code` (`tide-rustc:18`, `:32`).
- **Shadowing `assert!` so that a `#[protected]` test passes without
  checking.** D6 forbids user macros (`01:380`). A change in name
  resolution also changes the test's hash (`04:1194-1203`).
- **A package granting itself `unsafe`** (`[package] grants`,
  `reference.md:1025`). That is "adds or widens no grant" (`03:564`).
- **Removing a grant.** This narrows authority, and any `unsafe` code left
  in the package then fails with `FX0901`.
- **Moving a granted package to a new path under the same grant entry.**
  Its lockfile source changes, or its source files differ from the base's,
  either of which makes the candidate not covered (`03:567-568`). Round 57
  found the same.
- **Test fixtures read from the tree at run time.** Landlock admits no
  tree path (`03:487-489`), and `include_bytes!` is rejected (`03:452-454`).
  No route.
- **Plain-text spoofing of `tide review`'s verdict.** The verdict is the
  first line or the `--json` field (`03:558`). Reason lines that quote the
  branch come after it.

## Coherence and acceptance

§2.6 now reads in order. It defines the terms (`03:427-428`), then gives
the guarantee (RUN-1), its two mechanisms (RUN-2 for the build, RUN-3 for
the run), the named holes with a closure clause (RUN-4, "any other route is
a defect"), the gate (RUN-5), result integrity (RUN-6), what an operator
does, the complement, and a conformance row for each rule. I found no
sentence that contradicts another, apart from N2's row and N3's lag.

Against the bead's acceptance:

- **01 states the guarantee and its holes, and the owning chapter carries
  it as a numbered rule with conformance tests.** Met: D13 (`01:830-849`),
  RUN-1 to RUN-6 (`03:430-588`), and the table (`03:617-628`).
- **01 states the complement.** Met. D13 covers a sandbox per agent
  session, no VM per test run, kernel enforcement for the deployed process,
  and what a microVM does not give (`01:852-857`). 03 covers the same at
  `03:604-615`.
- **The reference says when an unreviewed branch is safe to test without
  a VM.** Met: §19 (`companion:495-521`) and the core pointer
  (`reference.md:965-967`). The core states a necessary condition ("only
  if"), and §19 adds `sandbox: full` and the hash.
- **The bead's scope items.** All are met:
  - the capabilities that tests and the simulator get (RUN-1);
  - the full list of holes, foreign effects without models included (hole
    2);
  - surfacing through the trust diff (RUN-5);
  - kernel enforcement as backing (RUN-3, and D7 for deployment);
  - rustc invocation (RUN-2);
  - the toolchain's proc macros (RUN-1 and hole 5);
  - the build scripts of imported Rust crates as a later hole (hole 3);
  - native test binaries (RUN-3).
- **Verification.** No escape outside the listed holes was found in this
  round.

## Consistency checks that passed

- `global_asm!` reads the same in ADMIT-1 (`03:410-412`), D7
  (`01:430`), hole 1 (`03:521`), the RUN-4 row (`03:626`), D13
  (`01:843`), and the core through `core::arch` (`reference.md:553`).
- The tier row (`01:82`), D13's "(v0)" (`01:830`), and the README
  (`README.md:150-151`) agree. Holes 2 and 3 and the behavior diff are v1,
  hole 4 is later, and both match the tier table (`01:83-84`).
- RUN-3's sandbox clauses match D13 (`01:837-841`) and §19
  (`companion:500-502`).
- The exceptions in §19 (`companion:518-521`) match 03's (`03:604-606`).
- REPLAY-1's "all inline, never as host paths" (`03:1480-1482`) agrees with
  RUN-1 (`03:441-442`) and with RUN-3's harness-read inputs (`03:480-481`).
- The companion header's "17–19" (`companion:10`) and its load list
  (`companion:7-8`) agree with the core pointer (`reference.md:8-9`).
