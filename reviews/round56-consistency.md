# Tide adversarial/consistency review, round 56: the unreviewed-branch guarantee (fx-arx)

## Verdict

**request_changes**

Every round-55 finding is closed, and none of the fixes contradicts the rest
of the design (see "Round 55 closure"). Four blocking findings remain. Three
are cases where RUN-5 prints `run: covered` while the candidate changes code
behind a hole: the dependencies of a granted package (B1), and an existing
imported Rust crate or Bend dependency (B2). B3 is an unlisted route: the
inherited standard streams. B4 is a contradiction about what `tide review`
runs. Each fix is a sentence or a clause plus a conformance row. The rest are
non-blocking.

## What was reviewed

The uncommitted working tree on 2026-10-20 against HEAD `87f9cd3`:
`git diff -- design/ README.md`, SHA-256
`54bed208047ef217adcf54476a959849d7d9f4c7a7e3a25b5e85985aaaf7752e`.

I read the changed text (03 §2.6 at `03:423-577`, the ADMIT-1 entry at
`03:410-412`, D7 at `01:430`, the D13 bullet at `01:828-853`, the tier row at
`01:82`, companion §19 at `companion:495-517`, `reference.md:5-10`, `:953`,
`:959-967`, and the README line) against 01 D6 (`01:380-405`), D7, D11
(`01:704-754`), D12 (`01:756-800`), D13, D15 (`01:887-916`), D17
(`01:938-955`), D18 (`01:957-1010`), and the tier table; 03 §1.1 path notes
(`03:88-91`), §1.3 (`03:175-213`), ROOT-1..5 (`03:219-241`), PANIC-1
(`03:293-308`), SINK-1..10 (`03:313-371`), ADMIT-1/2 (`03:379-421`),
§9.2-§9.8 (`03:1293-1464`), §10 (`03:1466-1519`), §11 (`03:1521-1573`); 04
§11 and §11.1 (`04:1187-1365`); `reference.md` §12.4 (`reference.md:1018-1052`);
research/13 X3, X8, E6; and round 55. I did not recount tokens.

## Round 55 closure

- **B1 (symbolic links).** Closed. RUN-2's last bullet resolves every path
  in the tree and the output directory beneath its root, the loader's and
  the harness's included (`03:468-471`); RUN-3 withholds link creation
  (`03:481-482`); the RUN-2 and RUN-3 rows test committed links and a link
  attempt (`03:574-575`). Consistent with §1.1's `Fs` rule (`03:88-90`).
- **B2 (lockfile identity).** Closed for the granted package itself: RUN-5
  names "version, source, or content hash" (`03:545-546`), and the row
  matches (`03:577`). See B1 below for the dependencies of that package.
- **N1 (registries).** Closed: RUN-1 says the registries come from "the
  operator's configuration ..., never the tree" (`03:446-448`); RUN-2's
  third bullet and the row reject a manifest or lockfile source outside
  them (`03:462-465`, `03:574`).
- **N2 (`global_asm!` code).** Closed: hole 1 (`03:501-502`), ADMIT-1
  (`03:410-412`), D7 (`01:430`), and the RUN-4 row (`03:576`) all give
  `FX0520` without a grant and an `unsafe` hole under it. The `reference.md`
  hole table no longer lists it (N7).
- **N3 (shared cache).** Closed: RUN-5 bars sharing artifacts built from an
  unreviewed candidate outside the session sandbox (`03:555-557`).
- **N4 (harness, toolchain macros).** Closed: hole 5 names the toolchain's
  derives and contract macros and the harness, which reads the result
  channel and output directory (`03:518-522`). See N2 below for the parsers
  `tide review` runs.
- **N5 (side channels).** Closed: hole 6 excludes them (`03:526-528`),
  consistent with §9.8 (`03:1462`).
- **N6 (replay paths).** Closed in RUN-1 (`03:431-433`), but the owning
  rule, REPLAY-1, does not say it (N4).

## Blocking findings

### B1. RUN-5 protects a granted package but not the dependencies its soundness rests on

**Where.** RUN-5 marks a candidate not covered if it changes "a source file
[or] the lockfile entry ... of a package with an `unsafe` or `ffi` grant,
since the soundness of `unsafe` code can rest on any item its crate can see"
(`03:544-548`). The RUN-5 row tests only "a granted dependency" (`03:577`).

**Problem.** A granted package's crate can see the items of its own
dependencies, and `unsafe` code may legitimately rely on a specific
dependency's behavior. If the root lockfile, which the candidate owns, fixes
those dependencies' versions, the candidate can change the code a reviewed
`unsafe` abstraction relies on, without touching the granted package's entry
or any grant, and still get `run: covered`. RUN-5's own rationale then
covers more than its rule does, and "A covered candidate reaches only holes
reviewed in the base" (`03:548-549`) overclaims: the hole was reviewed
against different dependencies.

**Evidence vs. inference.** Evidence: the quoted rule and rationale; D13
puts content hashes in "lockfiles" (`01:810`) with no statement about whose
lockfile pins a granted package's dependencies. Inference: as in Cargo, the
root lockfile resolves the whole graph, so the candidate chooses them.

**Smallest fix.** Extend the clause to "the lockfile entry ... of a package
with an `unsafe` or `ffi` grant, or of any package it depends on,
transitively", and add that case to the RUN-5 row.

### B2. Code behind holes 3 and 4 can change without "adding" anything

**Where.** RUN-5 marks not covered a candidate that "adds ... imported Rust
crate, build script, or Bend dependency" (`03:543-544`) and protects the
lockfile entries only of packages "with an `unsafe` or `ffi` grant"
(`03:545-547`). Imported Rust crates carry `authority = [..]` and
`build = true`, not grants (`03:1526-1528`). Hole 3 runs their build scripts
and proc macros at build time (`03:510-512`); hole 4 runs the pinned `bend`
on "the candidate's kernels" and, without a `reference`, a kernel server
outside the child's sandbox (`03:513-517`, `03:1505-1507`).

**Problem.** Against a base that already imports a Rust crate or has a Bend
dependency, a candidate can change the crate's lockfile entry (v1), or edit
the Bend dependency's kernel sources and law files in the tree (later),
while adding nothing. RUN-5 still prints `run: covered`. Then unreviewed
code reaches holes 3 and 4, and RUN-5's list of what remains for a covered
candidate (`03:551-553`, "an unsound reviewed abstraction and toolchain
bugs ... and resource use") leaves both out. Both tiers are later than v0,
but RUN-5 states the rule without a tier, and it is cheaper to fix now than
to remember at v1.

**Smallest fix.** Add to RUN-5's change clause: "the lockfile entry of an
imported Rust crate, or the pin or sources of a Bend dependency", and add
both cases to the RUN-5 row.

### B3. The inherited standard streams are an unstated path out of the sandbox

**Where.** RUN-3 lets the child inherit "its standard streams"
(`03:476-477`) without saying what they are. RUN-1 says "the sinks go to the
harness (§2.4)" (`03:442-443`), but §2.4 sends `out` to stdout and `err` and
`log` to stderr (SINK-2, `03:328-332`); only SINK-6 routes records to the
trace, and only under simulation (`03:353-354`). RUN-3's seccomp list names
sockets, io_uring, tracing, and signals (`03:481-484`), not terminal
operations. The closing paragraph of RUN-4 says holes 1, 2, 3, and 5 "meet
RUN-3's sandbox" (`03:530-531`).

**Problem.** If the operator runs `tide test` in a terminal and the child's
streams are that terminal, two routes open that RUN-4 does not list:

1. Safe test code can write any bytes, terminal control sequences included,
   to the operator's terminal. Terminal emulators have a history of
   vulnerabilities reachable through such sequences, so this is output to
   the host, not to the run.
2. A process that holds a terminal descriptor can use terminal operations
   that Landlock's path rules do not govern. On kernels that still permit
   it, one of them queues input to the terminal, which then reaches the
   operator's shell, outside every sandbox (the class of CVE-2017-5226 in
   bubblewrap). Code that reaches hole 1 or hole 5 — and hole 5 is open to a
   covered candidate's safe code (`03:522-525`) — would have that route. So
   the claim that holes 1 and 5 meet RUN-3's sandbox, and the RUN-3 row's
   "each fails" (`03:575`), overclaim.

**Evidence vs. inference.** Evidence: the cited text gives no stream types,
no session or terminal rule, and no ioctl restriction. Inference: that an
implementer would pass the caller's terminal through, as `cargo test` does by
default. The vulnerability class is public knowledge, not something I
checked against a current kernel.

**Smallest fix.** In RUN-3: the harness gives the child pipes it reads (and
an empty stdin), never a terminal; the child runs in a new session with no
controlling terminal; and the harness writes child output to a terminal only
with control characters escaped (NDJSON escapes them already). Correct
RUN-1's "(§2.4)" to say that the harness owns the child's stdout and stderr.
Add to the RUN-3 row: run `tide test` on a pseudo-terminal, check that the
child holds no terminal descriptor and that control bytes a test prints
reach the terminal escaped.

### B4. "`tide review` runs no candidate code" contradicts RUN-1 and D11

**Where.** RUN-5: "`tide review <base>` runs no candidate code (RUN-2
alone)" (`03:535-536`). RUN-1 counts "compile-time evaluation inside rustc
(`const fn`, and the toolchain's own derives and attributes expanding the
candidate's source)" as one of the two places where the candidate's code
runs (`03:431-436`). D11's v1 behavior diff has `tide review` rerun "the same
simulation seeds on base and head" (`01:719-721`).

**Problem.** The trust diff needs the checker's interface artifacts, which
depend on `const fn` evaluation (`01:765-766`), so `tide review` runs the
candidate's code at compile time by RUN-1's own definition. In v1 it also
runs head's simulation seeds. The sentence contradicts both. The RUN-5 row
already tests the narrower true claim: "no process started from the
candidate's build" (`03:577`).

**Smallest fix.** "`tide review <base>` runs the candidate's code only in
compile-time evaluation (RUN-1, RUN-2) and starts no process from its build;
v1's behavior diff (D11) runs its seeds under RUN-3 and does not affect the
`run:` line."

## Non-blocking notes

### N1. A child that exits without a result record

RUN-3 says printing cannot forge a pass (`03:478-479`); PANIC-1 covers aborts
(`03:299-307`). Neither says what happens when the child exits normally, at
any status (0, 1, 101, or SINK-5's 141, `03:349-350`), before writing a
started test's result record. State that the test and the run then fail, and
add it to the RUN-3 row.

### N2. `tide review`'s own parsers

Before any child runs, `tide review` and the driver parse the candidate's
manifest, lockfile, `tide.seal`, and approval records outside every child
sandbox. Hole 5 names input bugs "in the compiler, or in the harness" only
(`03:520-522`), and the closing paragraph's list of what only the session
sandbox confines (`03:532-533`) omits them. Add "or in the toolchain's
parsers of the candidate's manifest, lockfile, seal, and approval records".

### N3. Grants in a new dependency's own manifest

RUN-5's "adds or widens no grant" (`03:542-543`) does not say whether a new,
ungranted dependency whose own manifest grants `unsafe` to its dependency
counts. D11's transitive hole count (`01:709-711`) makes it "adds a trust
hole", so the candidate is not covered, but the RUN-5 row should test that
case so the implementation does not count only the root manifest.

### N4. REPLAY-1 should own the inline-manifest rule

RUN-1's parenthetical (`03:431-433`) is the only place that says a manifest
holds its files and fault plan inline. REPLAY-1 (`03:1423-1431`) and D15
(`01:899-903`) still say "files" and "fault plan" without it. Under
AP-OWNERSHIP-001 the format belongs to REPLAY-1; state it there and let RUN-1
cite it. Since the simulator in the child writes the manifest (PANIC-1,
`03:297-299`), REPLAY-2's identity check (`03:1432-1436`) is what bounds a
forged one; that already holds.

### N5. Summaries are broader than RUN-3 and RUN-1

D13 (`01:836-837`) and companion §19 (`companion:501-502`) say the sandbox
denies every file open outside the output directory; RUN-3 also lets the
child open the binary and its libraries to read and execute (`03:479-480`).
Companion §19 says "the build reads only the branch's `.rs` sources"
(`companion:499-500`); RUN-1 says the toolchain also reads the tree's
manifest and lockfile and fetches registry packages (`03:445-448`). Say
"outside its binary, libraries, and output directory" and "the branch's
tree".

### N6. A committed link where the output directory goes

RUN-2 makes a committed link that leaves the root "read as missing"
(`03:470-471`), and the row tests one from the output directory (`03:574`).
The text does not say what the harness then does: fail the run, or replace
the entry. Say which, so an implementation does not follow the link while
"creating" the directory.

### N7. `global_asm!` is not in the agent reference

`reference.md`'s hole table (`reference.md:951-956`) gained `#[unsafe(..)]`
but no longer mentions `global_asm!` anywhere. Optional, given the budget;
the checker's `FX0520` explains it.

### N8. Hole 6's heading

"Availability" now also holds side channels (`03:526-528`). Retitle it
"Availability and side channels".

## Escape attempts that failed

- **Ungranted code calling granted code.** Listed: hole 1's last sentence
  (`03:505-506`) and RUN-5's remaining exposure (`03:551-552`). Only B1's
  dependency change is new.
- **`tide.seal` and approval records.** Coverage reads neither (RUN-5,
  `03:540-548`); SEAL-1 checks only the crate's own seal (`04:1256-1257`),
  and approvals count only in CI against the base (`04:1222-1229`). The
  parser risk is N2.
- **`#[protected]` tests.** They run like any test, under RUN-3. CI runs one
  only if its hash matches an approved one (`04:1229-1230`). No route.
- **Authority policies** (D17, `01:948-952`; `reference.md:1034-1045`).
  These are checker rules over declared authority. Removing one is a trust
  diff and approval matter, not a run hole.
- **Per-test timeouts and virtual time.** Hole 6 and companion §19's
  limits cover them; virtual time advances without host time (`03:1323-1324`).
- **Exit codes.** `main`'s statuses (`03:193-194`) are visible to the
  harness. The gap left is N1.
- **Replay.** Inline contents plus REPLAY-2's identity check close host
  reads (N4 is about ownership only).
- **The test-only simulated root.** ROOT-3's `FX0504` (`03:228-233`), ROOT-5,
  and ADMIT-1's runtime-entry ban (`03:413`) leave no safe path to a real
  `Root`; D7 (`01:415-418`) matches.

## Consistency checks that passed

- D7's allowlist entry (`01:430`) matches ADMIT-1 (`03:410-412`) and RUN-4.
- The tier row (`01:82`) puts the guarantee and the Linux sandbox in v0,
  consistent with the D13 bullet's "(v0)" and RUN-3's "On Linux (v0)";
  holes 2-4 carry v1/later tags that match the tier table (`01:83-84`).
- RUN-3's sandbox is distinct from v1 kernel enforcement (D7, `01:498-501`),
  as RUN-3 and the complement paragraph say (`03:494-495`, `03:561-563`).
- RUN-3's research/10 citation for io_uring holds (`research/10:103`), and
  research/13's X3, X8, and E6 exist (`research/13:78`, `:126`, `:302`).
- §9.8's `EPERM` for unmodeled file and socket calls (`03:1455-1457`) agrees
  with RUN-3 (`03:491-492`).
- README, the D13 bullet, companion §19, and the reference.md pointer and
  hole row restate RUN-1..5. They add no claim the chapter lacks, apart from
  N5's broader wording, and they inherit B1-B4.
- The reference.md pointer (`reference.md:7-9`) still reaches every
  companion section: 14 and 17-19 are cited by number, Appendix A at
  `reference.md:1073`, and 15-16 by name.
