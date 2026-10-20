# Tide adversarial/consistency review, round 54: the unreviewed-branch guarantee (fx-arx)

## Verdict

**request_changes**

The text is careful and closes most of the surface a reviewer would try:
compile-time evaluation does no I/O, the file loader confines rustc's reads,
the sandbox is installed between fork and exec so static constructors are
covered, results travel on a channel no sink writes, `tide sim --replay` is
named as running under the same two places, the artifact cache is kept out of
the test child's writable set, `global_asm!` is added as a hole, and the
build/run-time split of residual exposure is stated honestly. I could not
construct a host-escape that `sandbox: full` does not confine and that RUN-4
fails to list.

One blocking defect remains: RUN-5's `run: covered` rule scopes `unsafe`
soundness to "a module that holds one," but in Rust an `unsafe` block's
invariant can be broken by safe code in any *descendant* module, because
private items are visible to the defining module and its descendants. A
candidate that edits only a pre-existing descendant module — holding no hole
itself — is marked `covered`, yet can break a reviewed `unsafe` invariant the
base relied on and, through it, forge a passing test result (RUN-4 hole 1).
That is exactly the "covered while reaching a hole the base did not review"
case fx-arx's acceptance asks the reviewer to rule out. It is a one-line scope
widening (B1).

The rest are non-blocking: an incomplete rule citation in RUN-1 (N1), imprecise
enforcement wording in RUN-3 that the sandbox-inheritance clause already saves
(N2, N3), and a disclosure wrinkle in RUN-5's shared-CI-runner advice (N4).

## What was reviewed

The uncommitted working tree on 2026-10-20 against HEAD `87f9cd3` ("Design
sealed delegation boundaries for agent fleets (fx-1am)"):
`git diff -- design/ README.md`, SHA-256
`274651473cc3ed5f8c3dbde4d2972f3eb74b2f44fc048c4e8df85a3f1669a222`.

The diff adds 03 §2.6 (RUN-1..RUN-5, the complement paragraph, and the
conformance-test table) and a `global_asm!` entry in ADMIT-1; D13's
"Unreviewed branches" bullet, a D7 allowlist entry, and a tier-table entry in
01; §19 and pointer/trim edits in the companion and `reference.md`; and a
README tier line.

I read the changed text against 01 D7 (`01:407-507`), D11 (`01:704-754`), D13
(`01:804-851`), D15, D18 (`01:955-1010`); 03 ROOT-1..5 (`03:217-241`),
GLOBAL-1..6, PANIC-1 (`03:287-308`), SINK-1..10, ADMIT-1..2 (`03:379-421`),
§9 (`03:1263-1448`, incl. §9.8 and REPLAY-1/2), §10, §11 RUST-5 (`03:1534-1546`);
`spikes/d18/examples/leaks.rs`; `research/10` §F6 (seccomp/io_uring) and
`research/13` X3/X8/E6. I did not recount the core token budget (D14 forbids
it here); I take the companion's 24,995 figure as recorded. I tried the
vectors fx-arx lists: rustc reads and compile-time behaviour, the toolchain's
derives, std-facade leaks, the test child's inheritance, Landlock/seccomp
limits (clone3, io_uring, `/proc`, abstract sockets, signals, ptrace, memfd,
execveat, inherited fds), the output dir and artifact cache, the replay
manifest, `tide sim --replay`, dependency resolution/fetch, and seal files.

## Blocking findings

### B1. RUN-5 scopes `unsafe` soundness to one module, but a descendant module can break it while staying `covered`

**Where.** `03:527-528`: a candidate is covered when it "changes nothing in a
module that holds one [a trust hole], since the soundness of `unsafe` code
rests on its whole module." The conformance test repeats the scope: not-covered
is triggered by "an edit to a safe function in a module that holds an `unsafe`
block," and covered is granted to "a candidate that edits only modules holding
no hole" (`03:560`). D13's one-liner and companion §19 inherit this.

**The escape.** In Rust a private item is accessible from the module that
defines it *and from that module's descendant modules* — privacy is not
module-local. So the safety of an `unsafe` block that relies on a private-field
invariant can be violated by safe code in a descendant module, which holds no
`unsafe` of its own. Concretely, suppose the reviewed base contains:

```text
src/ring.rs          mod ring { pub struct S { buf: *mut u8, len: usize }  // private fields
                                 impl S { pub fn get(&self, i: usize) -> u8 {
                                     // SAFETY: callers in this module keep i < len
                                     unsafe { *self.buf.add(i) } } }
                                 mod helper; }                              // pre-existing descendant
src/ring/helper.rs   (benign safe code)
```

The candidate edits **only** `src/ring/helper.rs` to add a safe function that
sets `s.len` to a large value (legal: `ring::helper` is a descendant of `ring`
and can touch `S`'s private fields), and adds a test (also covered) that calls
it and then `S::get`. The candidate adds no grant, no `unsafe` block, no trust
hole, no new dependency; it edits no module that itself holds a hole. By RUN-5
it is `run: covered`, so an orchestrator runs its tests without review. The
test then drives an out-of-bounds access through the base's reviewed `unsafe` —
reaching RUN-4 hole 1 (memory corruption → forge a capability, and "write the
result channel, so it can forge results too"). RUN-4 hole 1 explicitly scopes
the safe-code path to "an unsound `unsafe` abstraction the base already holds"
(`03:541-542`); here the candidate *newly* weaponises a sound base abstraction,
which is the review RUN-5 was supposed to force.

The external effects of the corruption are confined by RUN-3 under
`sandbox: full` (no syscalls, no sockets), but forging a passing test result
is a listed in-scope harm (fx-arx acceptance; RUN-3's own "printing cannot
forge a pass" shows result integrity is a guarantee here), and it reaches the
operator because RUN-5 told them not to review a `covered` branch. So this is a
real "covered while reaching an unreviewed hole" case, not one RUN-4 lists.

**Evidence vs. inference.** Evidence: Rust's visibility rule (a private item is
reachable from its module and descendants) is unchanged by the dialect (D18
keeps Rust's resolution; nothing in 02/03/04 restricts child-module access to
ancestor privates — the only module-scoping text in the design is this new
`03:528` line). Inference: that a descendant edit can therefore break an
ancestor's `unsafe` invariant and forge a result; this follows directly, and
the RUN-3 conformance test itself assumes safe code under a held abstraction
can forge the channel.

**Smallest fix.** Widen RUN-5's scope from "a module that holds one" to the
privacy scope of the hole: a candidate is not covered if it changes the module
that holds a hole **or any descendant of it** (and, for a hole relying on a
`pub(crate)` item, any module in the crate). Mirror the change in the RUN-5
conformance row (`03:560`) and in companion §19's example caption. The simplest
defensible over-approximation, if pinning the exact relied-on items is deemed
too much for v0, is "any module in a package that holds a hole," stated as
conservative. The accompanying rationale clause "rests on its whole module"
should read "rests on its whole module and that module's descendants."

## Non-blocking notes

### N1. RUN-1 cites ROOT-1/ROOT-5 for "test code cannot name a real `Root`," but the actual closure also needs ADMIT-1

`03:430-434` (RUN-1): "test code cannot name a real `Root` (ROOT-1, ROOT-5)."
ROOT-1/ROOT-5 (`03:219-241`) bar naming the *type* `Root` as a parameter and
calling `main`. They do not bar the toolchain's internal minting entry point,
which `spikes/d18/examples/leaks.rs:8` shows is nameable as `std::__rt::root()`
and which is closed only by ADMIT-1's "internal runtime entry points of Tide's
std" (`03:413`). No escape — ADMIT-1 rejects it — but RUN-1 should cite ADMIT-1
alongside ROOT-1/ROOT-5 so the guarantee's basis is complete.

### N2. "may execute only the binary" is imprecise against memfd/execveat, but sandbox inheritance saves it

RUN-3 (`03:541-544`) says the child "may execute only the binary." The initial
exec of the test binary happens under the already-installed sandbox, so
`execve`/`execveat` must be permitted for it; seccomp cannot match the path (or
a memfd) argument, so the child could in principle `execveat` a `memfd`. This is
not an escape because seccomp and Landlock are inherited across `execve` and
cannot be dropped, so any re-exec'd image stays confined — which RUN-3's
"covers everything the binary runs" implies. Consider rewording to "may execute
only images confined by this same sandbox" so the claim is literally true.

### N3. "may not ... start processes" is enforced by confinement, not by preventing process creation (clone3)

RUN-3 (`03:544-547`) lists "start processes" among what the child may not do,
alongside "threads ... remain." Thread creation uses `clone`/`clone3`, whose
flags live behind a pointer seccomp cannot inspect, so a filter that allows
thread creation cannot reliably forbid forking a process. The guarantee in fact
rests on the forked child inheriting the sandbox and Landlock's execute
restriction (it can run nothing outside the allowlist), not on blocking process
creation. research/10:104 is cited correctly for io_uring (seccomp blocks the
`io_uring_setup` syscall; it is the submitted operations it cannot inspect).
A half-sentence noting that started processes inherit the sandbox would make
the mechanism exact.

### N4. RUN-5's "shared CI runner, without a VM" understates the build-time residual it discloses

RUN-5 (`03:549-556`) permits running a `covered`, `sandbox: full` candidate "on
its own host or a shared CI runner" without a VM, and discloses the residual as
"compiler bugs at build time and resource use." RUN-4's closing paragraph
(`03:519-521`) states these "meet only the sandbox around the agent's session,"
i.e. *not* RUN-3's sandbox, since the build (rustc on attacker source) is not
run under RUN-3. On a shared CI runner that is not itself an agent-session
sandbox, that build-time exposure is unconfined. The disclosure is honest, but
the "shared CI runner, without a VM" recommendation reads as stronger than the
residual allows; recommend adding that the build step still wants the session's
confinement on a shared runner.

### N5. Minor wording

- `03:517` "In a test or simulation process, holes 1, 2, 3, and 5 meet RUN-3's
  sandbox ... Hole 4, a compiler bug exploited at build time, and resource use
  meet only the sandbox around the agent's session." The comma after "Hole 4"
  reads as an appositive (Hole 4 *is* a build-time compiler bug) when it is a
  list; a semicolon or "; a compiler bug" would disambiguate.
- "fixed deny-all OS sandbox" (`03:436`, `01:836`) is a deny-all with the
  explicit allowances RUN-3 enumerates (memory, threads, futexes, clock reads,
  output-dir writes, lib reads); accurate as shorthand, worth no change, noted
  only for precision.

## Consistency checks that passed

- The `global_asm!` addition is consistent across ADMIT-1 (`03:410-412`), D7
  (`01:430`), RUN-4 hole 1 (`03:537-539`), the companion hole table
  (`reference.md:953`), and the RUN-4/RUN-5 conformance rows (`FX0520` without a
  grant, a hole under one). No code mismatch.
- RUN-2's claims track D18/ADMIT-2: no user-controlled `--extern`/`-L`/
  `--sysroot`/`--cfg`/`-C`, `-Zallow-features=`, driver-owned rustc, own cache
  (`01:982-987`, `03:417-421`). `#[debugger_visualizer]` and the ignored
  `rust-toolchain.toml`/`.cargo/config.toml`/`RUSTFLAGS`/`RUSTC_WRAPPER` are the
  right set for "the branch cannot choose the toolchain."
- RUST-5's build-script sandbox (`03:1534-1546`) is correctly cited as a
  distinct, v1 hole for holes 3; the Bend server-outside-the-child case (hole 4)
  matches §10's process-mode protocol.
- REPLAY-2's identity check (`03:1415-1421`) plus RUN-1 listing `tide sim
  --replay` means an attacker manifest only configures a *simulated* world and
  still runs sandboxed; no host read via the manifest.
- The tier-table, D13 bullet, companion §19, and README line restate RUN-1..5
  without adding a guarantee the chapter does not carry; "v1" tagging of
  `ffi`/imported-crate holes and kernel enforcement is consistent with D7/D11.
- reference.md's trim drops the enumerating sentence but its "companion N"
  citations (17, 18, and now 19) still resolve; no dangling pointer introduced.
