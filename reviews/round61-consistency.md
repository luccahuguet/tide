# Tide adversarial/consistency review, round 61: the unreviewed-branch guarantee (fx-arx)

## Verdict

**approve**

This is a narrow confirmation round on the edits that took round 60's N1
to N6 and N8. Each edit closes its finding. None of them contradicts §2.6,
§9.6, D11, D13, D14, or companion §19, and no fix from rounds 54 to 59
was dropped. N7 was left to the implementation, as round 60 itself
suggested.

The notes below are small and none blocks. They are a residual of the
`--snapshot` gate when a file is swapped during the build, a first-line
example that is missing the base commit, two leftovers from the
output-directory move, and wording. Each has a one-line fix.

## What was reviewed

The uncommitted working tree on 2026-10-20 against HEAD `87f9cd3`:
`git diff -- design/ README.md`, SHA-256
`ddaae40c40588b2824accfadf5233a3e788f5ffc1cd3c2babaefc2e6c1553c1f`.

I read the changed text:

- 03 §2.6 (`03:423-635`), and in particular RUN-2's output-directory
  clause (`03:458-464`), RUN-3's level clause (`03:510-514`), hole 5
  (`03:541-552`), RUN-5 (`03:557-579`), the operating guidance
  (`03:595-608`), and the RUN-2 and RUN-5 rows (`03:630`, `03:633`);
- D11's behavior-diff bullet (`01:719-723`) and D13 (`01:830-862`);
- companion §19 (`companion:495-523`).

I checked it against:

- §9.6 and REPLAY-1/2 (`03:1455-1495`);
- the companion's replay paths (`companion:369`, `:430`);
- D14 (`01:874-882`);
- the core's pointer (`reference.md:965-967`) and the core's toolchain
  list (`reference.md:1059-1066`);
- round 60.

I did not count tokens.

## Round 60 closure

- **N1 (snapshot binding).** Closed. RUN-5 says "`tide test --snapshot
  <hash>` and `tide sim --snapshot <hash>` refuse to build unless the full
  hash matches, so a verdict applies to the tree it judged"
  (`03:565-567`). The hash is taken "over the bytes the build reads"
  (`03:562-563`). §19 runs the tests "only as `tide test --snapshot
  <hash>` with the full hash from `tide review --json`"
  (`companion:514-516`). The row adds "`tide test --snapshot` with a hash
  from another tree refuses to build" (`03:633`).
  - Round 60's order gap is closed: the gate now comes before anything
    runs.
  - The source gap is closed: the hash comes from `tide review --json`,
    not from output a child can print into.
  - The width gap is closed: the gate compares the full hash.
  - The gate follows D14's pattern of a base hash on a write request
    (`01:881-882`). A residual is in N1 below.
- **N2 (output directory).** Closed. Each run's output directory "is fresh
  and lies in the toolchain's state directory, outside the tree; the
  harness copies the manifests and traces it reports into `target/sim/`
  (§9.6), replacing, never following, anything there" (`03:461-464`). The
  row now agrees: "one committed at `target/sim/` is replaced, and the
  harness writes nothing through it" (`03:630`). §9.6's paths
  (`03:1472-1473`) and the companion's (`companion:369`, `:430`) are now
  the copies' paths. Their names carry the test and seed, so a later run
  replaces only a manifest with the same name. Two small leftovers are in
  N3.
- **N3 (row and D11).** Closed. The RUN-5 row now says "no process started
  from the candidate's build except behavior-diff children, and none of
  those for a candidate not covered or on a host without `sandbox: full`"
  (`03:633`). D11 now says "It runs only for a branch that 03's RUN-5
  calls covered, on a host whose test sandbox is full, unless the operator
  asks" (`01:721-723`), which matches RUN-5 (`03:576-579`).
- **N4 (level wording).** Closed: "If the harness cannot install the level
  it reports for this host, it fails the run" (`03:513-514`). This agrees
  with RUN-5's "the sandbox level this host's runs get" (`03:561`).
- **N5 (parsers and fetcher).** Closed. Hole 5 now lists "the toolchain's
  parsers of the manifest, lockfile, `tide.seal`, approval records, replay
  manifests, and fault plans; or in the fetcher, which unpacks the
  registry archives the lockfile picks" (`03:546-549`). The guidance lists
  "bugs in the harness, the parsers, the fetcher, or the kernel"
  (`03:601-602`), and these meet only the session's sandbox, which the
  build runs in (`03:603-606`). See N5 for the punctuation.
- **N6 (base resolution).** Closed. RUN-5's first line now gives "the base
  commit it resolved" (`03:561-562`), and the base is "a reviewed
  revision, such as a commit fetched from the protected remote, never a
  commit of the candidate itself" (`03:563-565`). §19 has the same text
  (`companion:503-505`). An agent can now see which commit a moved ref
  resolved to. See N2 for the example line.
- **N7 (loader and std startup reads).** Left to the implementation, as
  round 60 advised. No text was needed.
- **N8 (wording).** Closed for the two points that were taken.
  - RUN-5 now reads "adds, removes, or changes neither a source file nor
    the lockfile entry" (`03:571-572`).
  - D13 now reads "no imported Rust crate or Bend dependency"
    (`01:849`), which matches RUN-5's "Bend dependency (its pin, kernel
    sources, or law files)" (`03:570`).
  - The rewrap point is still open (see N5).

## Earlier fixes retained (rounds 54 to 59)

- **R54.** The package-wide and transitive clause (`03:571-576`) and the
  descendant row (`03:633`). The build inside the session sandbox
  (`03:603-606`).
- **R55.** Symbolic links (`03:458-461`, row `03:630`). Lockfile identity
  (`03:572-573`). Operator registries (`03:446`, `03:470-472`). One code
  for `global_asm!` (`03:410-412`, `03:522`). Unshared artifacts
  (`03:605-606`). The harness and derives in hole 5 (`03:542-547`). Side
  channels (`03:556`). Inline manifests (`03:1486-1488`).
- **R56.** The Rust crate and Bend clause (`03:568-570`), the pipes and new
  session (`03:477-479`), and compile-time-only review (`03:557-559`).
- **R57.** "Every other call fails" (`03:500`). The pre-run sandbox report
  (`03:510-511`). The behavior-diff condition (`03:576-579`,
  `01:721-723`). The exposure sorting (`03:597-603`). Core dumps
  (`03:478`). Tests never started (`03:583`). The kernel in hole 5
  (`03:549-550`). Regular files and directories only, with no links
  (`03:489-490`). "Refused (§9.8)" (`03:509-510`).
- **R58.** The Bend descriptor (`03:479-483`, `03:538-539`). Printed
  commands (`03:583-586`). System V IPC (`03:502-503`, `01:840`). `clone3`
  (`03:497`). Escaping (`03:587-590`). "Only if it is a VM" (`03:603`).
  The Landlock-less `partial` row (`03:631`).
- **R59.**
  - B1: `tide eval` appears in 03 (`03:612`) and in §19
    (`companion:520-523`).
  - B2: the review starts no process from the build (`03:558-559`).
  - B3: the base is reviewed (`03:563-565`, `companion:503-505`,
    `reference.md:966`).
  - N2: `stat` (`03:507-508`).
  - N3: `execve` (`03:498-499`).
  - N5: the Bend supervisor (`03:536-539`).
  - N7: "Its first line, and fields under `--json`" (`03:560`).
  - N8: traces (`03:591-592`).
  - N9: "sandboxed child processes" (`03:614-615`) and `#[unsafe(..)]`
    (`reference.md:953`).

## Blocking findings

None.

## Non-blocking notes

### N1. "Refuse to build" checks before the build, not over what it read

RUN-5 says `--snapshot` makes `tide test` and `tide sim` "refuse to build
unless the full hash matches" (`03:565-567`). Read literally, the check
runs before the build. A file swapped after the check, while rustc is
reading the tree, is then built and run as if covered. This is the
inference round 60 flagged: the hash has to cover the bytes the file
loader served. The harm stays bounded, as round 60 said, because RUN-3
still confines the run and only the `covered` label is wrong.

The smallest fix is "refuse to start any test or simulation unless the
hash of the bytes the build read equals `<hash>`". The loader already
serves every byte the build reads (`03:449-452`), so it can compute that
hash.

§19 names only `tide test --snapshot` (`companion:515`). It could add "or
`tide sim --snapshot`" for `tide sim <bin>`, which §19's last sentence
also covers (`companion:520-521`).

### N2. §19's example first line omits the base commit

RUN-5 puts "the base commit it resolved" on the first line
(`03:561-562`). The example's first line shows only `run: not covered
sandbox: full    snapshot 9c41e2…` (`companion:508`). Adding `base
e9d8ed4` (or similar) would match RUN-5 and show agents what to check.

Separately, §2.6's opening still gives the base as "such as the protected
branch it will merge into" (`03:427-428`). RUN-5's example is "a commit
fetched from the protected remote" (`03:564`). The two do not contradict
each other, but the opening is the reading that N6 warned against.
"Such as the protected remote's head" would align them.

### N3. Two leftovers from moving the output directory

- RUN-1 lists what the toolchain itself writes as "its artifact cache and
  the run's output directory" (`03:445-446`). It leaves out the copies
  into `target/sim/` (`03:463-464`). Add "and the copies in `target/sim/`
  (RUN-2)".
- "Replacing ... anything there" (`03:464`) could be read as clearing
  `target/sim/`, which would delete the manifest that an earlier printed
  replay command names. "Replacing, never following, whatever is at each
  copy's path" says what the row (`03:630`) tests.

### N4. The core's `tide test` synopsis lacks `--snapshot`

`reference.md:1059` lists `tide test [filter] [--seed N] [--trace]`.
The core's pointer sends an agent to companion 19 before it tests another
agent's branch (`reference.md:965-967`), and §19 gives the flag. Under
D14's budget this is acceptable as it stands, and no change is needed.

### N5. Wording

- Hole 5 nests semicolons inside a semicolon list: "bugs that input can
  exploit in the compiler; in the harness, ...; in the toolchain's
  parsers ...; or in the fetcher, ...; and bugs in the operating-system
  kernel" (`03:544-550`). As written, "in the harness" reads as an item of
  its own in the outer list. Use "bugs that input can exploit, in the
  compiler, the harness (which reads ...), the toolchain's parsers of ...,
  or the fetcher (which unpacks ...);".
- These lines are still not rewrapped: `03:500`, `03:508`, `03:514`,
  `03:539`, `03:616`, `01:853`, and `companion:502`.

## Escape attempts that failed

These are limited to what the edits could introduce.

- **A link committed at `target/`, so that the copies land outside the
  tree.** RUN-2 resolves every path in the tree beneath its root, and a
  link that leaves the tree reads as missing (`03:458-461`). The copy
  replaces the link and never follows it (`03:464`).
- **A delegate moving the local `main` before `tide review main`.** The
  base must be reviewed, and the resolved commit is printed (`03:561-565`).
  An agent that follows §19 names a commit fetched from the remote
  (`companion:503-504`).
- **A child printing a fake `snapshot` line to steer the gate.** The hash
  comes from `tide review --json` (`companion:515-516`), which runs no
  candidate process for a branch that is not covered (`03:557-559`). The
  gate compares the full hash inside `tide test` (`03:565-567`).
- **A hand-written replay manifest in `target/sim/` that names a host
  path.** REPLAY-1 holds everything inline (`03:1486-1488`), and
  `--replay` reads no other host path (`03:441-442`). REPLAY-2 refuses a
  source or build mismatch (`03:1490-1491`).
- **A crafted manifest, fault plan, or registry archive aimed at a parser
  or the fetcher.** These are now named in hole 5 (`03:546-549`) and meet
  the session's sandbox (`03:601-603`).

## Consistency checks that passed

- The `--snapshot` gate (`03:565-567`), its row (`03:633`), §19
  (`companion:514-516`), and D14's base-hash pattern (`01:881-882`)
  agree.
- The behavior-diff condition reads the same in D11 (`01:721-723`), RUN-5
  (`03:576-579`), and the RUN-5 row (`03:633`).
- The `target/sim/` paths in §9.6 (`03:1472-1473`) and the companion
  (`companion:369`, `:430`) agree with RUN-2's copy clause (`03:463-464`)
  and RUN-6's "a path it chose" (`03:585`).
- The base-commit wording reads the same in RUN-5 (`03:563-565`) and §19
  (`companion:503-505`).
- D13's summary of the coverage condition (`01:846-850`) matches RUN-5
  (`03:567-576`).
- Hole 5's parsers and fetcher (`03:546-549`) match the guidance
  (`03:601-602`).
