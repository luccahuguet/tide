# 15: Tide as a checked dialect of Rust

Research date: 2026-10-19. Scope: whether Tide should ship as a new
language (the current design) or as a checked dialect of Rust, in which every
Tide program is valid Rust that stock rustc compiles with the same meaning,
and Tide adds a capability std, a checker, the simulator, and the trust
diff. Bead `fx-enq`. Measured against `design/01-design.md` draft 3.5.
**Inference** marks reasoning that no source states.

Method: the design chapters were read for every v0-core item; prior art was
read from its repositories, READMEs, and crates.io metadata on the research
date [D1–D24]; the Eon workspace was surveyed at commit `40492dc` [E1];
experiments F1 and F2 ran on the local rustc. A second-model review
(`reviews/dialect-codex.md`) requested changes; its findings are answered
in place and listed under "Review resolution". Figures from projects about
themselves are self-reported.

---

## Question

Two facts reopened the question. Draft 3.5 took Rust 2024's lifetimes whole,
so the translation to Rust (`design/02-memory.md` §11) is the identity on
lifetimes. And lust showed a checked subset of valid Rust reaching edit
checks 60–250x faster than `cargo check` [D1], so fast builds no longer
distinguish Tide (`research/12-newcomer-sweep.md`, open question 7).

The thesis says tools layered on Rust cannot give D7 (no ambient authority),
D15 (deterministic simulation), and D11 (trust diff), "because Rust code can
reach the outside world from anywhere". That is shown for Rust with its own
std. It is not shown for a checked subset compiled against a different std.

**Cheapest falsifier** (from the bead): if D7, D15, or D11 cannot hold for
safe code under the dialect without changing rustc, the dialect cannot carry
the thesis and the new-language design stands.

---

## The two paths

Both paths need the same front end. D12's edit-to-diagnostics target (about
30 ms) rules out `cargo check` as the check loop on either path, and `Pure`,
authority, the power ceiling, and FX diagnostics are Tide's analyses either
way. lust also wrote its own checker rather than drive rustc [D1]. So the
question is narrower than "write a compiler or not". It is two questions:
must every Tide program be valid Rust, and does rustc become the backend?

| | New language (draft 3.5) | Checked dialect |
| --- | --- | --- |
| Source | Tide syntax: Rust's where meanings agree (D1) | valid Rust 2024, pinned rustc version |
| Front end | Tide checker | Tide checker, which must also resolve names, methods, and impls exactly as the pinned rustc does |
| Std | Tide std | Tide std injected as `std` (F1), plus an allowlist over `core` and `alloc` (F2) |
| Const evaluation | interpreter | Tide checker's const evaluator (needed for types on both paths); rustc's for code |
| Dev and test runs | interpreter, then Cranelift-class backend | rustc debug builds; interpreter optional |
| Release | C-emitting backend (v1) | rustc and LLVM |
| Rust interop | C-ABI shims both ways (Positioning, D13) | direct linking; types re-exported from Rust's std are shared, Tide-owned types need adapters |

---

## Experiment F1: a replacement std on stock rustc

Question: can stock rustc compile ordinary-looking Rust against a capability
std, so that the ambient std cannot be named, without a rustc change?

Setup (rustc 1.96.0, edition 2024, stable): a facade crate `tide_std`
re-exports allowed parts of the real std, defines a capability type `Fs`, and
provides a `prelude::rust_2024` module. User crates are compiled with
`--extern std=libtide_std.rlib`.

Results:

- rustc's normal std injection loaded the facade: `Vec`, `vec!`, and
  `println!` resolved through the facade's prelude, and code using the
  capability (`fs.read(..)` on a passed-in `&std::fs::Fs`) built and ran.
- `std::fs::read(..)` failed with E0425, "cannot find function `read` in
  module `std::fs`". `extern crate std as real;` also resolved to the
  facade, so the real std's paths could not be named.
- `core` and `alloc` stayed reachable by path. `core::include_str!` and
  `core::env!` read files and the build environment at compile time.

**Finding.** A stable rustc flag replaces std's *names*, with no boilerplate
in user files. It does not confine the program; F2 shows what still gets
through.

## Experiment F2: what the facade does not confine

The review (B1, B2) found leaks; all four below were reproduced locally with
the same setup, under `#![forbid(unsafe_code)]`:

- **Safe hardware queries.** `core::arch::x86_64::__cpuid(1)` is safe and
  printed a different APIC id on repeated runs (25, 52, 52, 52).
- **Address observation.** `(&x as *const u8).addr()` is safe and printed a
  different value on every run.
- **Synchronization without `Tasks`.** `core::sync::atomic::AtomicU64::new(0)`
  compiled, bypassing the facade rule that handles take `&Tasks`.
- **Inherent methods of the real std.** A facade prelude trait `Math` with a
  `sin` returning 123.0 did not dispatch: `0.0_f64.sin()` printed `0`. Rust's
  float math methods are inherent impls in std, and they win method lookup
  whenever std is in the crate graph, which it is under the facade.

**Finding.** Under the dialect, `core` and `alloc` are always present and
the real std's inherent methods stay callable. D7 and D15 therefore need
**an admission allowlist**. The checker resolves every path, method, and
impl, and admits only listed `core`/`alloc` items and std inherent methods.
This is per-item taming of the host library, the cost Joe-E reported
[D22]. It is smaller than taming all of std, but it is versioned with every
rustc pin. The driver must also own every rustc invocation: no extra
`--extern`, `-L`, `--sysroot`, `--cfg`, or semantic `-C` flags beyond those
it sets.

The new-language path has no counterpart: its std is the whole library, so
the allowlist is its std by construction.

---

## Per-feature classification

Classes:

- **Same**: valid Rust as written, same meaning.
- **Lib**: same spelling; the facade, a toolchain macro, or an attribute
  supplies Tide's meaning.
- **Rule**: valid Rust; the Tide checker adds a rule or analysis.
- **Respell**: today's Tide spelling is not valid Rust; a valid spelling with
  Tide's meaning is proposed.
- **Change**: Tide's meaning must change.
- **Open**: no mechanism demonstrated yet.

Valid Rust token syntax alone does not show that Tide's behavior is kept;
the Lib and Respell rows below name the mechanism that keeps it.

### v0 core (Tiers table)

| Item | Class | Under the dialect |
| --- | --- | --- |
| Rust surface, ownership, moves, NLL, two-phase borrows, drop check | Same | as Rust |
| Lifetimes as origins (Rust 2024 whole) | Same + Rule | identity (02 §11.1); power-ceiling rejections are rules. `T: Owned` stays spelled `T: 'static` |
| Traits with the power ceiling; impl placement; one inherent method per name | Rule | subset rules with Rust's meaning |
| Returned `impl Trait` names its auto traits (01 D3; 02 §5.2) | Rule | rustc leaks `Send` from an opaque return type; the checker rejects relying on an unnamed auto trait. Tide is stricter, so rustc accepts every program the checker accepts |
| `core` and `alloc` admission, std inherent methods | Rule (new) | the F2 allowlist; covers safe intrinsics (`__cpuid`), address observers (`addr`, `{:p}`), atomics constructors, and float methods |
| `Result`, `?`, must-use as error | Same + Rule | `#![deny(unused_must_use)]` or checker |
| Integers trap in every profile | Lib | `-C overflow-checks=on` in every profile; division by zero already panics |
| Lossless-only `as`; `truncate()`, `round_from` | Rule + Lib | subset rule; facade extension traits (names absent from `core`) |
| Floats: canonical NaN at observation points, one libm, `min`/`max` order | Respell | Inherent methods cannot be overridden (F2), and Rust documents NaN bit patterns and math precision as unspecified [D25]. Keep D3's meaning by banning the inherent observers and math methods (`to_bits`, `copysign`, `total_cmp`, `min`, `max`, `sin`, `exp`, …) through the allowlist, and providing facade functions under other names (`math::sin(x)`, `float::bits(x)`) that canonicalize and use Tide's libm. Formatting and byte conversions need the same audit. Not yet demonstrated |
| Built-in `!` forms | Lib | facade `macro_rules!` exported through its prelude; sinks, EPIPE exit 141, and argument evaluation are facade behavior; `dbg!` in release is a rule |
| Derives `PartialEq` … `Default` | Same | as Rust |
| Derives `Error`, `Encode`, `Decode` | Lib | toolchain-owned proc macros in the facade prelude; users still cannot write macros (D6) |
| Modules | Rule | files declared with `mod` (Tide already accepts it), `.rs`; `#[path]`, `extern crate`, cycles, unused imports are checker errors |
| Capabilities, attenuation | Lib | facade types |
| `fn main(root: Root)` | Respell | rustc rejects arguments on `main` (E0580). Either `pub fn main(root: Root)` in a library crate with a generated entry crate, or a toolchain `#[tide::main]` attribute; E1 should choose |
| `Root` only in `main`; no globals, thread-locals, or lazy statics | Rule | ROOT-1..4, GLOBAL-1..6 over valid Rust |
| `Pure` | Lib + Rule + Respell | `pub unsafe trait Pure {}` with a blanket impl, so rustc accepts every bound and the checker owns the structural analysis; coherence then matches D4. `dyn Trait + Pure` is E0225 in rustc: respell as a companion supertrait, `trait PureValue: Value + Pure {}` with a blanket impl, which compiled and upcast to `&dyn Value` in the review's test. `unsafe impl Pure` conflicts with the blanket impl, so it becomes a toolchain attribute such as `#[trusted_pure]`. The blanket impl enforces nothing for unchecked Rust callers (see Ecosystem) |
| Sinks (`print!`, `log`) | Lib | facade |
| Synchronization handles take `&Tasks`; `Arc` without counts | Lib + Rule | facade types; `core` atomics and Rust's `Arc` excluded by the allowlist |
| Structured tasks, scopes, cancellation, precedence rule | Lib | facade over `std::thread::scope` (02 §11.1: exact) |
| Blocking and thread runtimes | Lib | ordinary functions; no state machines in v0 (03 §7) |
| Channels, `Shared::with`, `with2` | Lib | facade over `Mutex` (02 §11.1: audited); `impl FnOnce(&mut T) -> R + Pure` is a valid bound |
| `join`, `par_map`, `par_for` | Lib | facade; `par_*` as prelude extension traits on slices |
| Deterministic `HashMap::new()` | Lib | facade-owned `HashMap`/`HashSet` at `std::collections`; a distinct type from Rust's |
| Contracts: `requires`, `ensures`, `old`, `result`, bounded quantifiers | Respell | `#[requires(..)]` and `#[ensures(..)]` from a toolchain macro that expands to checks in debug and test builds, as Prusti and Creusot spell them [D7, D8]. Sampled clauses' step budgets and the CLAUSE rules are the macro's job and are not yet demonstrated |
| Loop `invariant` | Respell + Open | A statement form at the top of the body is not enough: the review showed it skips the entry check on an empty range and the exit check (04 §3). The loop must be owned by a function-level toolchain attribute that rewrites each loop, checking at entry, after each iteration, and at exit with the next-index rule. Not yet demonstrated |
| Contract mode per module (`#![contracts(..)]`) | Respell | custom inner attributes are unstable; use the manifest and item attributes |
| `#[test]` property tests with parameters | Lib | facade `test` attribute shadowing the built-in one |
| Panics: unwinding, abort on double panic, scope re-raise, no `catch_unwind` | Same + Rule | as Rust; `catch_unwind` is a checker error |
| Stack overflow is a panic that unwinds (D5) | Change | rustc aborts; matching needs codegen. Change D5 to Rust's abort; under the simulator, an overflow fails the seed |
| `const fn` evaluation: authority-free, deterministic, fuel-limited | Rule | Rust's `const fn` already cannot do I/O; the checker needs its own const evaluator for array lengths and const generics on both paths. rustc's evaluator computes the compiled values; they must agree (an obligation, below) |
| The simulator (crash, restart, durability) | Lib | facade backend behind `Root::sim`, on baton-passed threads (03 §9.1). It needs no interception, because the facade is the only std and the allowlist closes `core`. Crash without destructors parks the thread, which is the same on the new-language path's compiled engine |
| Declared authority, trust holes, trust diff | Rule | computed from the checker's interfaces and the lockfile; sound only if the checker's resolution matches rustc's |
| Per-dependency grants; no build scripts; authority policies (D17) | Rule | Tide's driver and checker |
| Agent toolchain (D14) | Rule + Respell | NDJSON, fix plans, snapshot hashes are the checker's. Typed hole `todo!()` stays; `_?name` does not parse in Rust, so it becomes a toolchain form such as `hole!(name)` |
| `tide eval` | Open | a scratch crate built by rustc (seconds), or the interpreter |
| Interpreter, then Cranelift-class backend | replaced or optional | rustc builds run tests and the simulator; an interpreter is built if `tide eval` or E4's edit-to-test target needs it |

### v1 and later

| Item | Class | Under the dialect |
| --- | --- | --- |
| `prove`, `spec fn`, `decreases`, laws | Lib | attributes, as Prusti, Creusot, and Verus show [D7–D9] |
| Evented runtime (D8 state machines) | Open | rustc lowers only `async fn` to state machines. Candidates: generate `async` Rust from Tide source for evented builds, stackful coroutines (rejected in D8), or a rustc change. Any candidate must also show non-`Send` locals across migration (R1, which Rust cannot express), scoped child borrows, non-moving frames, recursive and dynamic suspension, and foreign pinning (02 §8, 03 §7) |
| Field-borrow declarations | Open | not Rust syntax; needs a source transform or Rust's view types |
| C-emitting release backend | dropped | rustc and LLVM; open question 7 (unwinding in C) disappears |
| FFI with `#[trusted_pure]` | Lib + Rule | `extern` blocks under an `ffi` grant |
| Rust crate import | Lib + Rule | direct linking instead of C-ABI shims, but RUST-2 still applies: a Tide wrapper takes the capabilities and the crate's declared authority is checked by a linked-symbol scan, as cackle does [D4]; build scripts sandboxed; effects unmodeled in the simulator |
| `par_reduce`, record and replay, plan roots, kernel enforcement | Lib | facade and runtime |
| Bend tandem (later) | Lib + Rule | `bend` dependency kind in the driver; `Accel` |

**Summary.** For the v0 core:

- **Respelled:** contract clauses, loop invariants, contract modes,
  `main(root: Root)`, `dyn Trait + Pure` and `unsafe impl Pure`, typed holes,
  and float observers and math.
- **Changed:** stack overflow aborts.
- **New checker rules:** the `core`/`alloc` admission allowlist and the
  opaque-auto-trait rule.
- **Open:** loop-invariant lowering, `tide eval`.

At v1, the evented runtime and field-borrow declarations are open.

---

## The falsifier: D7, D15, and D11 under the dialect

**D7 holds if the admission allowlist holds.** F1 removes the ambient std
from every Tide crate with a rustc flag; rustc enforces that, whatever the
checker does. F2 shows that safe items in `core`, and std's inherent
methods, still reach hardware state, addresses, and unsynchronized atomics.
D7 therefore rests on the checker admitting only allowlisted `core`/`alloc`
items, plus the global, `Root`, and grant rules it enforces on both paths.

**D15 holds on the same condition, with floats respelled.** Every source of
nondeterminism in admitted code is a facade capability. Hash seeds belong to
the facade's `HashMap`, and threads come only from `Tasks`. Floats keep D3's
meaning only through the respelling above. Rust leaves NaN bits and math
precision unspecified even on one target [D25], so pinning the target is not
enough.

**D11 does not depend on the backend.** It is computed from the checker's
interfaces and the lockfile. It is correct only if the checker resolves
every accepted program as rustc does.

**Verdict.** The falsifier does not fire as a matter of principle: no
guarantee needs a rustc change. But each rests on Tide-owned enforcement
the first draft did not count. That enforcement is the allowlist, driver
control of rustc flags, and resolution equivalence with rustc. It was shown
for mechanisms, not for a complete checker. The thesis sentence must change:
tools layered on Rust with its ambient std cannot give these guarantees; a
checked subset with its own std and an admission allowlist can.

**What does not carry over.** Someone can build Tide source with stock
`cargo` and the real std, skipping the checker. That build is not a Tide
build, and its guarantees are void, just as they would be for a
new-language program compiled by a modified compiler. CI and `tide review`
run the checker, so approvals (D11) are unaffected.

---

## Cost of the first demo

The demo has two parts: an agent-written concurrent service whose bug is
found by seed and replayed exactly, and a PR that widens network authority
and is flagged by the trust diff.

| Work | New language | Dialect |
| --- | --- | --- |
| Checker: parse, resolve, types, traits, borrows, `Pure`, authority, FX diagnostics | yes | yes; borrow checking may lag, since rustc rejects what the checker wrongly accepts |
| Std with capabilities, tasks, `Shared`, channels, simulator | yes | yes, over the real std |
| Trust diff, grants, lockfile, driver | yes | yes; the driver also owns every rustc invocation and its cache |
| Execution | an interpreter, which also serves as semantic oracle and const evaluator | none for the demo; rustc |
| Const evaluator in the checker | part of the interpreter | separate, and must agree with rustc's |
| Admission allowlist over `core`/`alloc` and std inherent methods | none | yes, reviewed per rustc pin |
| Resolution equivalence: the checker resolves paths, methods, impls, and the facade prelude as rustc does | none | yes, with differential tests against rustc |
| Toolchain macros: contracts, loop invariants, derives, `test` | built into the parser and checker | proc macros, kept in step with the checker |

**Inference, unmeasured.** Each path carries work the other avoids: an
interpreter on one side; the allowlist, equivalence testing, and macro
coupling on the other. Neither column has been sized, so "about the same"
is not established. The difference that is not in doubt comes after the
demo. The new language must build a Cranelift-class backend, a C release
backend, an answer to open question 7, and C-ABI interop. The dialect gets
those from rustc. The size of that difference is an estimate, not a
measurement.

---

## Ecosystem reach

- **Outbound (Tide into Rust).** A Tide library is a Rust crate that any Rust
  program can depend on, with `tide_std` as a dependency.
  - **Re-exported types are shared.** Types the facade re-exports from Rust's
    std are the same types, so APIs built on them interoperate directly:
    `Vec`, `String`, `Box`, `Option`, `Result`, slices, and `&str`.
  - **Facade-owned types are distinct.** Rust consumers see `HashMap`,
    `HashSet`, `Arc`, and the capability types as different types from Rust's
    own. The review's test passed `std::collections::HashSet` to a
    facade-typed parameter and got E0308. APIs that expose these need
    adapters, and a Rust caller must be handed capabilities by a
    Tide-written entry point.
  - **Rust callers are unchecked.** Code Rust passes into Tide is foreign
    code: closures that claim `Pure` and trait impls. The blanket `Pure`
    impl checks nothing for it, so such callbacks are trust holes, as FFI
    callbacks are.
- **Inbound (Rust crates into Tide).** On both paths crates.io crates use the
  ambient std, proc macros, and build scripts, so they sit outside the
  checked set; lust reports that most crates.io crates do not compile under
  its subset [D1]. The dialect links synchronous crates directly, so there
  is no C ABI. Each crate still needs a Tide wrapper taking its
  capabilities (RUST-2), a symbol-scan authority check, and simulator
  models.
- **Tools.** rust-analyzer, rustfmt, debuggers, and profilers apply to valid
  Rust. With a non-Cargo driver, rust-analyzer needs a generated
  `rust-project.json` describing crates, dependencies, and cfgs [D26]. It
  must also be shown to complete facade items, expand the contract macros,
  and surface Tide's diagnostics. This is adapter work, unverified.
- **Build concurrency (`research/13-agent-fleets.md` P5).** Checked builds
  must not go through Cargo. Cargo's build directory lock blocks agents in
  separate worktrees, and fine-grained locking is nightly-only through the
  1.98 changelog [D20]. Tide's driver invokes rustc per crate against a
  content-addressed artifact store, as D12 plans for its own artifacts. A
  generated `Cargo.toml` is an export for Rust consumers.
- **Performance.** LLVM gives Rust's code generation, but Tide's semantics
  cost the same on both paths: overflow checks in every profile, the
  facade's runtime, and canonicalizing float observers. Release parity with
  Rust is therefore not automatic. The overflow-check cost is unmeasured
  [D19].

---

## Learnability, context tax, and positioning

These are predictions; D0 makes surface choices evidence-gated, so they need
E1 data before they count.

- **Surface.** The dialect removes Tide-only syntax: bare contract clauses,
  `fn main(root: Root)`, `_?name`, and undeclared modules. It adds attribute
  contracts, a function-level invariant attribute, a respelled `main`,
  companion `Pure` traits, and facade float functions. Prusti and Creusot
  code is low-resource, so E1 cannot assume models know these forms.
- **Context tax (research/13 criterion 1).** The reference is about 19,000
  tokens [`research/13-agent-fleets.md`]. Most of it covers capabilities,
  tasks, contracts, `Pure`, and the simulator, and those stay on both paths.
  **Inference**: the difference is small in either direction until both
  references are written and counted.
- **Priors.** "This is valid Rust" invites Rust habits such as
  `std::fs::read`, float `.sin()`, and `Arc::new(v)`. Tide's checker runs
  first, so the agent sees an FX diagnostic with a fix rather than rustc's
  E0425. The new-language path has the same habit risk.
- **Positioning.** The choice is between a distinct language and a Rust
  toolchain profile whose guarantees depend on always using Tide's checker
  and driver. lust, capsec, and cap-std sit near the second [D1–D3].
  Familiar syntax alone does not decide it.

To measure it, held-out E1 tasks (`fx-jv9`) should cover the following, and
compare the two references counted in tokens:
- attribute contracts;
- empty-range loop invariants;
- `main` wiring;
- facade types at a Rust boundary;
- Rust-habit failures.

---

## The fleet criteria (bead comments)

| Criterion | New language | Dialect |
| --- | --- | --- |
| Per-spawn context tax | about 19k tokens | unmeasured; likely close |
| Check loop near 30 ms | Tide checker | Tide checker; rustc only for builds |
| Sealed delegation; run an unreviewed branch's tests without a VM | checker gate, no build-time code | same: the checker runs before rustc, the driver passes no user-controlled flags, and rustc runs no user code without proc macros or build scripts |
| Simulation with virtual time | yes | yes, given the allowlist |
| Commands bounded in time and output; failures tied to change and seed | toolchain contract | same contract; rustc's output is wrapped, not shown raw |
| Build cache safe for concurrent agents | Tide driver | Tide driver, not Cargo |

Neither path wins clearly on fleet needs.

---

## Eon as the test case

The Eon workspace has 8 crates and about 64k lines of Rust [E1].

- **Already near the subset.** `orbit-protocol` (4.8k lines) and
  `eon-workspace-protocol` (4.4k lines) have no dependencies, no `unsafe`
  (both `#![forbid(unsafe_code)]`), no user traits, no serde, and no macros.
  They use hand-written binary codecs, borrowing decoders (`Decoder<'a>`),
  one const generic (allowed by the reference), and std `HashSet`.
  - **Dialect.** They can become Tide crates in place, and Venus, Orbit, and
    eon-runtime keep depending on them as Rust crates. The review found
    their public APIs use `String` and `Vec`
    (`orbit/crates/protocol/src/lib.rs:265–280`), while the `HashSet` uses
    are internal (`eon-workspace-protocol/src/v2.rs:460–478`). So no
    adapters are needed for these two crates as they stand. **Inference**:
    that is a property of these APIs, not of every boundary.
  - **New language.** Every type crossing to the Rust consumers needs a
    C-ABI export and a generated binding crate, and borrowing decoders cross
    a C ABI poorly.
- **eon-manifest** (541 lines) uses serde derive with `deny_unknown_fields`
  and an internally tagged enum. Tide's v0 `Decode` has neither key
  (`design/reference.md`: `deny_unknown_fields` is left out, and enums are
  externally tagged). Both paths need those codec keys first.
- **Orbit core** has the following:
  - a single-threaded `libc::poll` loop;
  - PTYs, `setsid`, `ioctl`, and signals through libc `unsafe`;
  - Unix sockets with peer credentials;
  - libghostty-vt, whose build script compiles Ghostty with Zig.

  Both paths need `unsafe` and `ffi` grants. Tide's capability set has no PTY,
  process-group, or peer-credential capability yet. The simulator lists
  signals and real child processes as not simulated (03 §9.8).
  - **Dialect.** libc and libghostty-vt link directly as imported crates,
    with RUST-2 wrappers and the build script sandboxed.
  - **New language.** Both arrive through v1 shims.
- **Simulation gain for Orbit.** Orbit's correctness rests on wall-clock
  deadlines:
  - 1 s client negotiation;
  - 500 ms SIGHUP grace and a 2 s reap;
  - 10 ms PTY retries;
  - its integration tests use real sleeps of 50–1,150 ms, one longer than
    the 1 s timeout it races.

  Virtual time would make attach, detach, and crash interleavings
  reproducible. It needs process, PTY, and signal models first, and that work
  is the same on both paths.
- **Order on either path.**
  1. The two protocol crates.
  2. eon-manifest, after the codec keys.
  3. eon-runtime's pure parts.
  4. Orbit's core, after the process and PTY capabilities and models.

  Venus (24.7k lines, 211 transitive packages on Linux, 16 transitive proc
  macros) stays Rust.

**Inference.** For a Rust workspace adopted crate by crate, the dialect makes
leaf crates whose APIs use shared std types cost almost nothing to adopt.
Crates exposing facade-owned types need adapters on either path. On the
new-language path, every boundary needs a C-ABI shim.

---

## What each path costs

**The new language keeps:**
- freedom of surface and semantics;
- an std that is the whole library, so no allowlist;
- one semantic oracle (the interpreter);
- a positioning as a new language.

**It pays for:**
- an interpreter, a Cranelift-class backend, and a C release backend;
- open question 7;
- C-ABI interop in both directions.

**The dialect keeps:**
- every v0 guarantee, with the respellings above and one change (stack
  overflow);
- LLVM code generation;
- direct linking with Rust;
- rustc as a second, final borrow checker;
- Rust's tools, through adapters.

**It pays for:**
- **A surface bound to Rust's grammar.** Every future feature must fit
  attributes, toolchain macros, or library types.
- **An admission allowlist** over `core`, `alloc`, and std inherent methods,
  re-reviewed with every rustc pin. This is the Joe-E taming cost [D22], at
  a smaller scope.
- **Resolution and const-evaluation equivalence with the pinned rustc.**
  Without it the authority analysis describes a different program.
- **Toolchain macros kept in step with the checker.**
- **v1's evented runtime and field-borrow declarations**, which are open.
- **Positioning nearer existing tools.**

---

## Recommendation

**Adopt the checked dialect as the v0 direction, gated by a bounded
feasibility spike before v0 implementation planning commits to it.** The
falsifier did not fire. The backend, interop, and soundness-checking
savings after the first demo are large even as estimates. The new costs the
review exposed (allowlist, equivalence, macro coupling) are bounded and
testable in a spike. The new-language path stays the fallback, and nothing
in the v0 design is lost if the spike fails.

Spike acceptance criteria. Each item is a check that could fail.

1. **The facade std.** It provides the v0 capability, task, `Shared`, and
   channel surface, injected by `--extern std=`.
2. **An allowlist prototype that rejects every leak found.**
   - F2's four leaks: `__cpuid`, `.addr()`, `core` atomics, and inherent
     float methods.
   - `core::include_str!` and `core::env!`.
   - Any driver flag outside its fixed set.
3. **The contract macros.** `#[requires]` and `#[ensures]`, plus a
   function-level loop-invariant lowering that fails on
   `for _ in 0..0 { invariant!(false) }` and on an invariant false at exit.
4. **The `Pure` companion supertrait.** It compiles and the checker rejects
   an impure implementor.
5. **Facade float functions.** They return canonical NaN bits and Tide's
   libm results.
6. **Eon.** `orbit-protocol` builds as a Tide crate and Orbit builds against
   it unchanged.
7. **Prototype size.** A count of the work in each, to replace the
   unmeasured cost comparison.

Decision changes to `design/01-design.md` if the owner adopts this:

- the Thesis sentence on tools layered on Rust, and Positioning (Rust
  coexistence);
- D1: contract, invariant, and `main` spellings, declared modules, `.rs`;
- D2: drop the `T: Owned` rewrite;
- D3: float observers and math through facade functions;
- D5: stack overflow aborts;
- D7: `unsafe impl Pure` becomes an attribute, `dyn + Pure` becomes companion
  traits, and the admission allowlist;
- D10: the attribute spellings;
- D12: the backends, open question 7 closed, and the origin cross-check
  becoming the build;
- D13: direct Rust import with RUST-2 wrappers;
- D14: the typed-hole spelling.

**Conditions that would reverse it:**

- **The spike fails a criterion** with no workable fix.
- **E1 rejects the attribute spellings** (`fx-jv9`).
- **The v1 evented runtime cannot be built.** It must meet the requirements
  in the v1 table without exposing coloring to users. If it cannot, v0 users
  would face a second source or semantic transition at v1. Decide before v1
  whether that tier leaves the dialect or drops the evented runtime.

---

## Spike results (fx-7qu)

The spike ran on 2026-10-19 in `spikes/d18/` (see its README). Toolchains:
stable rustc 1.96.0 for the facade, macros, and Eon builds, and pinned
`nightly-2026-08-09` (rustc 1.99.0-nightly 771916f90) for the admission
driver. Two agents wrote it. The main session reran both suites:
`scripts/all.sh` exited 0, and `admit/scripts/run-tests --facade` reported
0 failures.

| # | Criterion | Result |
| --- | --- | --- |
| 1 | Facade std | **Pass.** See the facade details below |
| 2 | Allowlist rejects the known leaks | **Pass.** See the allowlist details below |
| 3 | Contract macros and loop invariants | **Pass with gaps.** See the contract details below |
| 4 | `Pure` companion trait | **Pass.** See the `Pure` details below |
| 5 | Float functions | **Pass.** See the float details below |
| 6 | Eon | **Pass for the build; about 21 edits for admission.** See the Eon details below |
| 7 | Sizes | **Measured for the dialect side only.** See the sizes below |

**1. Facade std.**
- It provides `Root { fs, clock, tasks }`, `Fs` with `dir()` attenuation,
  and `Clock`.
- `Tasks::scope`, `spawn` and `join` run over `std::thread::scope`, and
  panics propagate.
- It has `Shared::with` and `with2` (id-ordered, `Err(Aliased)`),
  `channel(&tasks)`, a fixed-key `HashMap`, and `Pure`.
- These all failed to compile: `std::fs::read`, `std::thread::spawn`,
  `Instant::now`, `std::env::var`, bare `env!`, forging `Root`, and
  `fn main(root: Root)` (E0580).
- Both `main` respellings work. `#[tide::main]` is preferred because it
  keeps one crate and works with cargo and rust-analyzer. Its cost: a
  checker rule must stop tests from calling `main`, because doing so mints a
  real `Root`.

**2. Allowlist.**
- `tide-admit` walks HIR with typeck results and checks every resolved path,
  method, operator, deref, impl selected for an obligation, and macro
  expansion against `policy.txt`. Rejections print `FX0520`.
- It rejected `__cpuid`, `.addr()`, `{:p}`, `core` atomics, inherent
  `sin`/`to_bits`/`min`/`sqrt`/`exp`, `include_str!`/`env!`/`option_env!`
  (including inside a doc attribute), and the real `std::fs::read`. That held
  both against the real std and with the facade injected.
- An ordinary program using 178 distinct items was admitted. The policy has
  66 directives.
- The flag gate rejected `-L`, `--cfg`, `--sysroot`, `-C opt-level`, an
  undeclared `--extern`, `--extern std=`, `-Z`, `--edition 2021`, and
  `@args` files.

**3. Contract macros and loop invariants.**
- `#[requires]` and `#[ensures]` support `result`, `old`, `==>`, and bounded
  quantifiers. Checks are erased in release.
- `#[invariants]` rewrites loops. `for _ in 0..0 { invariant!(false) }` and
  the false-at-exit `while` case both fail, and the next-index rule holds.
  12 of 12 tests pass.
- Gaps: a panic inside a clause is not yet reported as a violation
  (CLAUSE-7); sampled-clause budgets are missing; `a..=b` and general
  iterators get no next-index binding.

**4. `Pure`.**
- `trait PureValue: Value + Pure {}` with a blanket impl compiles,
  `&dyn PureValue` works, and it upcasts.
- The check must run where a type meets a `Pure`-implying obligation or is
  coerced to such a `dyn`, not at the blanket impl. That is because the
  blanket impl makes every `Value` a `PureValue`.
- `FX0510` caught a capability field, a `Shared` reached through tuple, `Vec`
  and `Box`, a closure capture, an atomic, and an unbounded type parameter.

**5. Float functions.**
- `float::bits` maps four different NaNs to `0x7ff8000000000000`.
- `math::sin` and `math::exp` equal libm on 200,000 inputs.
- `float::min` and `float::max` order the signed zeros.
- The inherent `x.sin()` and `x.exp()` differed from libm on 560 of those
  inputs, which is direct evidence for banning them.

**6. Eon.**
- orbit-protocol, at Eon `2332d4b`, compiled against the facade through a
  `RUSTC_WRAPPER` with no source edits.
- yazelix-orbit built unchanged against it.
- Tests: orbit-protocol 21 unit and 3 integration (also under the facade);
  Orbit 36 passed, 2 ignored.
- Admission is another matter. Against the real std it uses 239 distinct
  items, none from `std::` itself. Two things are rejected: `f32::to_bits`
  and `from_bits` at 20 sites in its codec, which D3 bans, and one
  `debug_assert_eq!`, which is outside the fixed `!` set. So it builds
  unchanged but needs about 21 edits to be admitted.

**7. Sizes** (code lines):

| Part | Lines |
| --- | --- |
| Facade std | 320 |
| Contract macros | 326 |
| Admission driver | 486 |
| Scripts | about 210 |
| Policy | 66 directives |

The admission check took 0.44 s on orbit-protocol, against 0.32 s for
rustc's metadata build alone. The new-language side had no prototype, so the
cost comparison stays an estimate at both ends. The spike shows only that
the dialect-specific parts are small at prototype scale.

**New findings:**

- **Admission on rustc's own resolution needs a nightly `rustc_private`
  driver.** Prusti and Creusot are built the same way, and the spike hit
  about eight API changes against expectations on one nightly. This gives a
  choice of checker architecture:
  - **A. The pinned nightly driver is the admission gate.** Builds use the
    same nightly. This removes the resolution-equivalence obligation (B7),
    because the gate checks rustc's own resolution. D18's "stock rustc"
    becomes a pinned, unmodified nightly. `-Zallow-features=` must be passed
    so user code cannot enable unstable features.
  - **B. Tide's own front end is the gate**, as D12 plans, with differential
    tests against rustc.

  Either way, D12's 30 ms edit loop needs Tide's own incremental front end,
  because the driver costs a full rustc analysis per check. **Inference**:
  start with A as the authoritative gate and the oracle, and build B for the
  edit loop.
- **More leaks for the allowlist.** I/O methods on re-exported std types
  (`Path::exists`, `metadata`, `read_dir`, `canonicalize`),
  `io::Error::last_os_error`, `mpsc::Receiver::recv_timeout` (time without a
  `Clock`), and the facade's internal `__rt::root` are all nameable. Raw
  pointer types must be inadmissible as a whole, because their `Debug`,
  `Hash` and `Ord` impls observe addresses.
- **Allowlist keys churn.** Keys are defining paths, not the paths users
  write. On this nightly, `AtomicU64` is `Atomic<u64>` and `io::Error` lives
  in `core`. Trust should key on crate identity, not crate name.
- **Dependents need the facade's path.** Crates that depend on a Tide crate
  need its dependency path (`-L dependency=…`). Cargo injects the facade
  with no wrapper when a manifest says
  `std = { package = "tide_std", path = … }`. That lowers outbound interop to
  a one-line manifest change.
- **Cargo does not track wrapper decisions.** Its fingerprints ignore a
  `RUSTC_WRAPPER`'s choices, which is further support for Tide's driver
  owning every invocation.

**Reading.** No criterion failed, so the gate's condition is met at
prototype scale. The gaps in criteria 3 and 6 are implementation work, not
blockers. The checker-architecture choice and the added leaks change D18's
text before its listed decision changes are applied.

**Owner decisions (2026-10-19).** The gate is passed, and the checker
architecture is A then B. Both are recorded in D18.

---

## Review resolution

`reviews/dialect-codex.md` (request_changes). Each finding and where it is
answered:

| Finding | Resolution |
| --- | --- |
| B1 safe `core` leaks | Accepted and reproduced (F2). The admission allowlist and driver flag control were added; D7 and D15 are now conditional on them |
| B2 float dispatch and NaN | Accepted and reproduced (F2). The float row became Respell through facade functions; dropped the "per target" narrowing of D3 |
| B3 `dyn + Pure` | Accepted: companion supertrait spelling; unchecked Rust callers are trust holes |
| B4 loop invariants | Accepted: function-level lowering required; it is spike criterion 3 |
| B5 interop types | Accepted: re-exported versus facade-owned types; RUST-2 wrappers kept; Eon claim narrowed to its current APIs |
| B6 classification gaps | Accepted: new classes; rows added for opaque auto traits, the allowlist, typed holes, const evaluation, and `tide eval`; summary corrected |
| B7 cost comparison | Accepted: cost table lists each path's obligations and is marked unmeasured; spike criterion 7 sizes it |
| N1 tools | Accepted: rust-analyzer adapter work; performance parity not automatic |
| N2 learnability | Accepted: predictions marked as needing E1; positioning stated as a choice |
| N3 v1 | Accepted: reversal condition lists the evented runtime's requirements |

---

## Open questions

1. The `main` respelling: a library crate with a generated entry, or
   `#[tide::main]`.
2. The size of the admission allowlist, and how it is reviewed when the
   rustc pin moves.
3. The real cost of `overflow-checks` in rustc release builds [D19].
4. The evented runtime's lowering under the dialect (v1).
5. Whether rust-analyzer can show Tide's diagnostics alongside its own.

---

## Sources

- [D1] [t3dotgg/lust-lang README](https://github.com/t3dotgg/lust-lang), v0.2.0, 2026-10-17; all figures self-reported.
- [D2] [bytecodealliance/cap-std](https://github.com/bytecodealliance/cap-std), 4.0.3, 2026-08-20: "not a sandbox for untrusted Rust code".
- [D3] [auths-dev/capsec](https://github.com/auths-dev/capsec), 0.2.2: source audit, `CapProvider` bounds, `cargo capsec diff`; documents its own bypasses (unsafe, direct std calls, FFI).
- [D4] [cackle-rs/cackle, HOW_IT_WORKS](https://github.com/cackle-rs/cackle/blob/main/HOW_IT_WORKS.md), cargo-acl 0.9.0: rustc and linker wrapping, relocation-based API attribution, bubblewrap sandboxing; notes that `-Funsafe-code` is insufficient.
- [D5] [google/capslock caveats](https://github.com/google/capslock): call-graph capability analysis for Go; data races and `go:linkname` escape it.
- [D6] [Rust Reference, preludes](https://doc.rust-lang.org/reference/names/preludes.html): `no_std` does not prevent linking std.
- [D7] [prusti/prusti](https://github.com/prusti/prusti), pins `nightly-2026-08-15`; `#[requires]`, `#[ensures]`, `#[pure]`.
- [D8] [creusot-rs/creusot](https://github.com/creusot-rs/creusot), v0.14.0, 2026-10-12, pins `nightly-2026-08-03`.
- [D9] [verus-lang/verus](https://github.com/verus-lang/verus): `verus!{}` syntax, ghost code erased.
- [D10] [model-checking/kani](https://github.com/model-checking/kani), v0.68.0; [model-checking/verify-rust-std](https://github.com/model-checking/verify-rust-std).
- [D11] [tokio-rs/loom](https://github.com/tokio-rs/loom), 0.7.2, 2024-04-23: `cfg(loom)` swaps sync primitives.
- [D12] [awslabs/shuttle](https://github.com/awslabs/shuttle), 0.9.7: dependency wrappers by package renaming.
- [D13] [tokio-rs/turmoil](https://github.com/tokio-rs/turmoil), 0.7.2: drop-in net and fs crates.
- [D14] [madsim-rs/madsim](https://github.com/madsim-rs/madsim), 0.2.34: libc symbol interposition for time, entropy, and thread creation; `[patch]` forks of dependencies.
- [D15] [S2, deterministic simulation testing](https://s2.dev/blog/dst), 2025-04-02.
- [D16] [Polar Signals, DST in Rust](https://blog.polarsignals.com/blog/posts/2025/07/08/dst-rust), 2025-07-08.
- [D17] [Rust 2024 unsafe attributes](https://doc.rust-lang.org/edition-guide/rust-2024/unsafe-attributes.html) and [newly unsafe functions](https://doc.rust-lang.org/edition-guide/rust-2024/newly-unsafe-functions.html).
- [D18] [Rust project goal 2026: build-std](https://goals.rust-lang.org/2026/build-std.html); [rustc_public goal](https://goals.rust-lang.org/2026/rustc-public.html).
- [D19] [Cargo profiles](https://doc.rust-lang.org/cargo/reference/profiles.html); C overflow-checking cost as a proxy: [LWN 960822](https://lwn.net/Articles/960822/).
- [D20] [Cargo changelog](https://doc.rust-lang.org/nightly/cargo/CHANGELOG.html): `build-dir` stable in 1.91, fine-grained locking nightly through 1.98.
- [D21] [mozilla/sccache Rust notes](https://github.com/mozilla/sccache/blob/main/docs/Rust.md).
- [D22] [Mettler, Wagner, Close, Joe-E, NDSS 2010](https://www.ndss-symposium.org/wp-content/uploads/2017/09/met.pdf): taming the class library by whitelist was "a substantial weakness".
- [D23] [endojs SES](https://github.com/endojs/endo/tree/master/packages/ses): compartments disable `Math.random` and `Date.now` unless endowed; [googlearchive/caja](https://github.com/googlearchive/caja), archived 2021.
- [D24] [Ferrocene 26.02.0](https://ferrous-systems.com/blog/ferrocene-26-02-0/): a certified subset of `core`.
- [D25] Rust std docs, [f32 NaN bit patterns](https://doc.rust-lang.org/std/primitive.f32.html#nan-bit-patterns) and [f64::sin precision](https://doc.rust-lang.org/std/primitive.f64.html#method.sin), as cited by the review.
- [D26] [rust-analyzer, non-Cargo projects](https://rust-analyzer.github.io/book/non_cargo_based_projects.html), as cited by the review.
- [E1] Eon workspace survey, `~/pjs/eon/eon` at `40492dc`, tokei and grep counts; not a public source.
- F1, F2: local experiments, rustc 1.96.0 (ac68faa20 2026-05-25), stable, edition 2024, x86_64 Linux; scripts not kept.
