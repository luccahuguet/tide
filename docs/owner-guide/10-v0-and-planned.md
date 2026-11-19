# 10. What v0 does, and what's planned

## The idea

Tide's design is much larger than what's built. That's normal for a project
at this stage, and it's also the easiest thing for a critic to use against
you. The defense is simple: **never let a planned feature be heard as a built
one.** This chapter is the map.

Three words to keep straight:

- **v0** is the tier the first release commits to. Parts of it are built
  and parts are planned.
- **v1** and **later** are the tiers after it. Nothing in them is built.
- **The alpha** (bead fx-qlek) is Tide's first public announcement: v0, Linux
  only, x86_64.

### Runs today, on `main`

| Area | What works | Chapter |
| --- | --- | --- |
| Admission | `tide-admit` checks one crate (or a test suite paired with one library) against Tide's std: 57 `TD` codes, the allowlist, the backstop | 2 |
| Capabilities | `Root`, `Fs`, `Net` (TCP, one HTTP GET, name lookup), `Proc` (`command`), `Clock`, `Rand` (five draws), `Env`, `Stdin`, `Tasks`, attenuation; `Pure` | 1 |
| Soundness routes | `TD0206` closes rust#25860's function-pointer route; `TD0402` closes rust#57893 (red team fx-qlek.5) | 3 |
| Concurrency | scopes, cancellation, deadlines, channels, `Shared`, `join`, `par_map`, `par_for` | 4 |
| Contracts | `#[requires]`, `#[ensures]`, `invariant!` as runtime checks and property tests | — |
| Simulation | `#[sim_test]`, `Root::sim(seed)`: tasks, channels, `Shared`, files, local TCP, HTTP peers, declared child processes, clock, env, stdin, random draws | 5 |
| Replay | `--tide-run-test` with `--seed`, and `--replay` with the identity check; tests, same machine | 6 |
| Trust diff | `<crate>.interface.json` and `--tide-diff`: declared authority and `unsafe` holes | 7 |
| Sandbox | RUN-3 for the driver's test runs, with the level reported | 8 |
| Evidence | the demo (`cargo run -p tide-demo`), E1 passed, E3 published (negative), the red team (fx-qlek.5), the chrono_anchor re-test (research/49), 392 conformance cases | 2, 9 |

### Planned for v0, not built

The biggest gaps (the repository `README.md`'s tier list names them too):

- **The `tide` command**: `tide check`, `tide test`, `tide sim`,
  `tide review`, `tide fix`. Today `tide-admit` is the only command, and its
  run and diff modes are their partial precursors.
- **Tide's incremental front end**, for edit checks in tens of milliseconds
  (D12). Today each check is a full rustc run.
- **Packages**: `tide.toml`, `tide.lock`, dependencies, per-dependency grants,
  authority policies, sealed interfaces (`tide.seal`).
- **More of the world**: UDP, `Rand::rng() -> Pcg64` (the five draws
  exist), and `Proc::exit`. HTTP is one GET, and processes start through
  `Proc::command`.
- **More of the simulator**: fault injection, crashes and restarts, nodes, and
  `tide sim` for whole programs.
- **More of the sandbox and review**: the build sandbox (RUN-2), escaping test
  output (RUN-6), the unreviewed-branch guarantee (RUN-5), and contracts and
  grants in the trust diff.

### v1 and later

**v1:** proofs of contracts (`prove` mode with an SMT solver),
**exercised** authority in the trust diff, the evented runtime (many tasks on
few threads), importing Rust crates as listed trust holes, kernel
enforcement for deployed programs, production record and replay, behavior
diffs in `tide review`. **Later:** the Bend tandem (handing pure parallel
kernels to Bend), Polonius-precision borrows, and more.

## Run it

Count how much of the agent-facing reference is labelled as not built:

```sh
bash -c 'grep -o "(planned)" design/reference.md | wc -l'
```

Output: `10`. Each is a place where the reference teaches agents a surface and
labels it **(planned)**, so they don't assume it exists.

List the v0 gaps that the claims audit (fx-p44q) filed as beads:

```sh
br list --label v0-gap
```

Output (11 beads; `❄` means deferred, scheduled after the alpha):

```text
❄ fx-2hr5 [● P3] [task] - The blocking runtime and `[bin] runtime`
❄ fx-vozv [● P3] [task] - `#[trusted_pure]` and `#[protected]`
❄ fx-m6mk [● P3] [task] - `Secret<T>`, `std::trace::span`, and the `log::`/`tracing::` prefixed log macros
❄ fx-8cpg [● P3] [task] - Atomics through `&Tasks`, and `std::sync::raw`
❄ fx-x78i [● P3] [task] - Contracts and grants in the trust artifact and `--tide-diff`
❄ fx-muza [● P3] [task] - Authority policies (`[[policy]]`, `TD0516`)
❄ fx-x2oo [● P3] [task] - Packages: Tide dependencies, per-dependency grants, and `tide.lock`
❄ fx-a9oh [● P3] [task] - The `tide` CLI over the driver: `check`, `test`, `sim`, with NDJSON diagnostics and fix plans
❄ fx-djwo [● P3] [task] - Simulator faults and crashable nodes: `sim.faults`, `sim.node`, crash and restart, `Tasks::fault_point`
❄ fx-tl1u [● P3] [task] - Real `Proc` backend: `command`, `exit`, `only`, process handles (companion 20)
❄ fx-9bzp [● P3] [task] - `Rand` draws and `Pcg64`
```

The bead titles are as filed: fx-tl1u's `command` and `only` have landed
(fx-ayh2.1), and fx-muza's code is spelled in the older `FX` form.

And check what commands exist:

```sh
ls target/debug
```

There is `tide-admit` and no `tide`.

## Read the code

**`reviews/fx-p44q-audit.md`**, its header and first table (the first 60 of
197 lines, 10 minutes). Not
code, but the most useful file for this chapter: on 2026-11-16 an agent
checked all 110 surfaces the docs teach against the source code and marked
each **implemented** (54), **partial** (14), **planned** (34), or
**reworded** (8, claims restated so they hold for v0). Each row cites the
file and line that proves it.

```sh
bash -c 'sed -n "1,60p" reviews/fx-p44q-audit.md'
```

- The header says what was audited, against which commit, and that nothing
  was built: every row comes from reading source.
- Each table row: the surface, where the docs teach it, its status, and the
  evidence (`A/` is `crates/tide-admit/src`, `S/` is `crates/tide-std/src`).
- Row 14 (the sandbox) says "planned": that was true on 2026-11-16. fx-ip4x
  built it the next day. An audit is a snapshot; check the date.

## What to look at

- The planned count: the reference teaches the whole v0 design, and labels
  what isn't built.
- The v0-gap beads: each is a gap written down as work, not hidden.

## The hardest question

> "Your guarantees hold because your world is tiny: one crate, no
> dependencies, no TLS, no HTTP, no processes. Real infrastructure lives
> outside it. Either you import Rust crates and the guarantees weaken, or you
> rebuild the world by hand and never finish."

This is the adversarial review's top finding (Achilles W1). It is the strongest
critique of Tide. Concede what's true, then answer.

**True today:** the driver admits one crate with no dependencies. The std
has grown: one HTTP GET with TLS, name lookup, and starting processes landed
for the alpha, but there is no UDP and no Unix sockets. "In any dependency"
holds only because there are none. The README was reworded to say so (fx-p44q).

**The answer:**

1. **Graded claims, not absolute ones.** Tide's claim is authority and replay
   you can read *for the Tide-written part, with every hole listed*. When Rust
   crates are imported (v1), each is a listed trust hole behind a capability
   wrapper, and runs that touch unmodeled effects are marked not replayable.
   As designed, the guarantee doesn't silently weaken; it says where it
   stops. (Crate import is v1 and not built.)
2. **The kernel sandbox doesn't care what language a crate is in.** Once
   crates can be imported (v1), RUN-3 bounds them in the test runs the driver
   starts, and deployed-program enforcement (v1) in production. Today there
   is no imported code.
3. **Tide's bet**, a hypothesis: much of the risk in agent-written code is in
   its own logic, the Tide-written part. Achilles W1's Eon example shows
   that's not yet true for process-heavy programs (Eon's lifecycle races need
   processes and terminals; v0 now starts processes but has no terminals or
   signals), which is why point 4 sets the priority.
4. **What the critic gets right** sets the priorities: the effect surface
   (processes and HTTP landed for the alpha; UDP and Unix sockets remain)
   should come ahead of later-tier language
   features, and a coverage audit of three real programs should decide it.
   That's an owner's scope call.

## Going deeper

- `design/01-design.md`, the Tiers table.
- The repository `README.md`, "v0 Core and Later Tiers" and Status.
- `reviews/fx-p44q-audit.md` (the claims audit).
- `reviews/2026-11-15-achilles-opus.md` W1 (closed world) and its suggested
  next steps.
- `br show fx-qlek` (the alpha and what blocks it).
