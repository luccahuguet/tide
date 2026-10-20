# Tide adversarial/consistency review, round 55: the unreviewed-branch guarantee (fx-arx)

## Verdict

**request_changes**

Round 54's findings are closed: RUN-5 now scopes coverage to whole packages
with an `unsafe` or `ffi` grant (`03:533-536`), which over-approximates the
descendant-module and `pub(crate)` cases B1 raised; RUN-1 cites ADMIT-1 for the
minting entry point (`03:436-437`, N1); RUN-3 rests on sandbox inheritance
rather than on forbidding process creation (`03:479-481`, N2/N3); the build's
residual is now confined by the session sandbox, `tide check` included
(`03:538-541`, N4); and the closing paragraph of RUN-4 reads as a list
(`03:521-524`, N5).

Two blocking gaps remain. Both are cases where the stated rule depends on a
property the text never fixes. (B1) RUN-2's confinement is lexical and says
nothing about symbolic links in the candidate's tree. (B2) RUN-5's
"no locked version" check is narrower than what the attacker-controlled lockfile
can change. The rest are non-blocking.

## What was reviewed

The uncommitted working tree on 2026-10-20 against HEAD `87f9cd3`:
`git diff -- design/ README.md`, SHA-256
`d896b3e6215ff02fb06e7dca0375720f4a65913b7e675d8865626a00e3d74506`.

I read the changed text against 01 D7, D11 (`01:704-754`), D12 (`01:756-800`),
D13 (`01:804-853`), D18 (`01:957-1010`); 03 §1 path notes (`03:88-91`),
ROOT-1..5 (`03:219-241`), PANIC-1 (`03:293-308`), SINK-*, ADMIT-1/2
(`03:379-421`), §9.6 REPLAY-1/2 (`03:1384-1428`), §9.8 (`03:1438-1451`),
§10 (`03:1453-1506`), §11 RUST-5 (`03:1539-1550`); 04 §8.4 (`04:1109-1119`);
reference.md §11-§12.4 (`reference.md:950-1052`); and round 54. I did not
recount tokens.

## Blocking findings

### B1. RUN-2 confines rustc's reads lexically; nothing covers symbolic links in the candidate's tree

**Where.** RUN-2 says the loader "serves only `.rs` files under the package's
source directories" and "no byte of an outside file can reach a diagnostic"
(`03:450-456`). RUN-1 says the toolchain "writes its artifact cache and the
run's output directory" (`03:443-445`), and RUN-3 grants the child create and
write access to "files in the run's output directory" (`03:474-477`). The
output directory sits in the tree (`target/sim/...`, `03:1402-1403`).

**Problem.** The attacker controls the whole tree, and a git checkout restores
symbolic links. "Under the source directories" and "the run's output
directory" are path properties. Unless the loader and the harness resolve paths
without following links that leave the tree, a `.rs` path or a `target`
directory in the tree can name a location outside it. Then the RUN-2 title
claim ("the build reads only the candidate's tree") and its no-bytes-in-
diagnostics claim fail, and the harness's writes, and the child's writable
set, can land outside the checkout. `tide review` runs RUN-2 too
(`03:525-526`), so this also applies before any test runs. RUN-4 does not
list it, so by `03:489-490` it is a defect.

**Evidence vs. inference.** Evidence: the design already states
`openat2(RESOLVE_BENEATH)` for `Fs` paths (`03:88-90`), and `reference.md:411`
gives the same rule for Fs. No such rule exists for the driver's loader, the
artifact cache, or the output directory. A grep for "symlink" in `design/`
finds only those two Fs passages. Inference: a loader implemented from RUN-2's
text could follow links. The text does not rule it out.

**Smallest fix.** In RUN-2, state that the loader resolves every path beneath
the package root with `RESOLVE_BENEATH` semantics (`..` and links cannot leave
it). In RUN-3, state that the harness creates the output directory itself and
opens it beneath the run's root without following links, and that the child may
not create symbolic links there (Landlock's make-symlink right is withheld). In
the RUN-2 and RUN-3 conformance rows, add a committed link to a canary file
and a committed `target` link to a canary directory, and require no access to
either.

### B2. RUN-5 checks a granted package's locked *version*, but the lockfile carries more identity than that

**Where.** RUN-5 marks a candidate covered if it "changes no source file and no
locked version of a package with an `unsafe` or `ffi` grant" (`03:533-535`).
The conformance row tests "a new version of a granted dependency"
(`03:564`). D13 says lockfiles carry content hashes (`01:810`). RUN-1 fetches
"from the configured registries", in the plural (`03:445`).

**Problem.** The candidate owns the lockfile. A lockfile entry for a granted
package can change in its content hash, or in which configured registry it
comes from, without changing its version string. RUN-5 as written still prints
`run: covered`, yet the granted package's code is then not the code the base
reviewed. That is the "covered while reaching an unreviewed hole" case fx-arx's
verification rules out.

**Evidence vs. inference.** Evidence: the cited text names only the version.
The design states neither that a (name, version) pair has exactly one content
across the configured registries nor that the lockfile cannot pick among them.
Inference: whether this is exploitable depends on those unstated registry
semantics. The rule should not depend on them.

**Smallest fix.** Replace "no locked version" with "no lockfile entry (version,
source, or content hash)" of a granted package, and widen the conformance row
to match. Separately, see N1 on who configures registries.

## Non-blocking notes

### N1. "Configured registries" has no stated owner

RUN-1 (`03:445`) and RUN-2 (`03:463-464`, and the conformance row at `03:561`)
treat "the configured registries" as trusted, but no document says whether the
operator configures them or the candidate's manifest or lockfile does. Only
the manifest is tested (`03:561`). State that registries come from the
operator's configuration, never from the tree. Extend the conformance test to a
lockfile entry naming an unconfigured source.

### N2. `global_asm!` carries two diagnostic codes

RUN-4 hole 1 lists `global_asm!` among items that "need the package's `unsafe`
grant (`FX0901`)" (`03:493-497`). The RUN-4 conformance row (`03:563`) and
ADMIT-1 (`03:410-412`) make it `FX0520` without a grant. `reference.md:953`
puts it in the `FX0901` row. Pick one code. The raw-pointer row's pattern,
"`unsafe` grant (`FX0520` without it)" (`reference.md:956`), fits ADMIT-1.
Round 54 marked this consistent, but the hole-1 sentence reads otherwise.

### N3. A shared artifact cache crosses the session sandbox that RUN-5 relies on

RUN-5 confines a build-time compiler bug to the session sandbox
(`03:538-541`). The driver's content-addressed cache (`01:986-987`, `01:779`)
has no stated scope. If orchestrators share it across sessions or with release
builds, an unreviewed build's entries outlive that sandbox. 04 §8.4 already
takes this stance for proofs: "a cache an agent can write is advisory only"
(`04:1118-1119`). Say the same for artifacts written by unreviewed builds, or
scope the cache per session.

### N4. Hole 5 omits the harness and the toolchain's own proc macros

Hole 5 names "rustc, LLVM, the driver, Tide's checker, or Tide's std"
(`03:509-511`). The test harness runs outside the child's sandbox and parses
the result channel and the output directory after the child exits. The
toolchain's derives (`Encode`/`Decode`) and contract attributes expand
candidate input inside rustc (RUN-1, `03:432-434`). Neither is named. Add both,
and state that the harness treats the result channel and the output directory
as untrusted input.

### N5. Side channels

RUN-1 claims the candidate "reads ... nothing outside the run" except through
RUN-4 (`03:441-443`), and RUN-4 says any other route is a defect
(`03:489-490`). §9.8 lists timing side channels as not simulated
(`03:1449-1450`). Microarchitectural and timing side channels are not in RUN-4.
Add one line excluding them, or list them under hole 5 or 6.

### N6. Replay manifest paths

REPLAY-1 records "files" and the "fault plan" (`03:1410-1418`). If either is
recorded as a path rather than as contents, `tide sim --replay` on a
candidate-supplied manifest reads a host path outside the sandbox (the harness,
not the child, loads the manifest). Say that the manifest holds contents
inline, or that its paths resolve beneath the tree as in B1.

## Consistency checks that passed

- Round 54 B1 and N1-N5 are closed as described in the verdict. The package
  scope matches D13's bullet (`01:842-843`), companion §19's example
  ("package has an unsafe grant"), and the RUN-5 row (`03:564`).
- RUN-3's sandbox and environment are consistent with PANIC-1's child-process
  model (`03:296-306`) and ROOT-3 (`03:228-233`).
- RUN-1's minting closure (ROOT-1, ROOT-5, ADMIT-1) matches `03:219-241` and
  `03:413`.
- Hole 4's server outside the child's sandbox matches §10's process protocol
  and simulation fallback (`03:1478-1482`, `03:1494-1497`).
- Hole 3 matches RUST-5 (`03:1539-1544`). The `v1`/later tags match the tier
  table (`01:82-84`) and the D13 bullet.
- Approvals and seals: RUN-5 coverage does not depend on approval records or
  `tide.seal`, so neither can make a candidate `covered` (D11, `01:730-745`).
- README, the tier-table line, D13, companion §19, and the reference.md pointer
  restate RUN-1..5 and add no claim the chapter lacks, apart from the B1/B2
  scope gaps they inherit.
