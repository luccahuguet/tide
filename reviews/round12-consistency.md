# Tide consistency review, round 12: the D18 dialect pass

## Verdict

**request_changes**

Reviewed commit `fc4cb4ccc6862bd671910ee639215d2447685eb9` (`fc4cb4c`,
"Apply D18's dialect changes to the design (draft 3.7)"), dated 2026-10-19
17:48 -0300, diffed against `c8859b8`. The review covers
`design/01-design.md` (D1–D18, the Tiers table, the Feel), `design/02-memory.md`,
`design/03-authority-concurrency.md`, `design/04-contracts.md`,
`design/reference.md`, and `README.md`. The sources were
`research/15-rust-dialect.md` (classification tables, F2, Spike results) and
`spikes/d18/`.

What holds:

- **Every item in D18's "changes applied" list landed in its decision**
  (table below). Stack overflow aborts in D5, 02 §8.3, 03 PANIC-1, 04 §3,
  and reference §9. `T: 'static` is spelled as in Rust everywhere, and 02
  §3.7 calls `Owned` "the design's name for it, not source syntax".
- **The leftovers are history.** No `tide` fence remains in 02 or the
  reference. Interpreter, Cranelift, and C-backend text appears only as
  history (01:749–754), as a conditional ("an interpreter … if one is
  built", 01:735, 03:998), or as the lapsed open question 7. `T: Owned`,
  `unsafe impl Pure`, `dyn Trait + Pure`, and `_?name` appear only as
  rejected or historical spellings (01:209–212, 01:921–925, 02:462,
  03:381, reference:661, :1297).
- **Codes and rule IDs agree.**
  - `FX0520`: 01:413, 03:323, and reference §4.3, §6.9, §11, and §14.
  - `FX0506` and ROOT-5: 03:192 and reference:501.
  - ADMIT-1 and ADMIT-2: 03 §2.5. Their item lists match D7 and D18.
  - RUST-1 to RUST-6: 03 §11, with 01 D13 citing RUST-2.
- **Clause meaning did not move.** Loop invariants are checked at entry and
  exit in debug builds, and after every iteration under `tide test`, in all
  four places: 01 D10 (595–597), 04 CLAUSE-6, 04 §2.3–§2.4, and reference
  10.1. `old`, `result`, and the `checked`/`sampled` rule are unchanged.
- **v1 proof forms are labeled consistently** as draft 3.5 grammar pending
  respelling: 01 D10 last bullet (674–678), the 04 Spelling paragraph
  (26–36), 04 §1.1's table, the 04 §1.2 grammar comments, and reference
  Appendix A (1690–1691).
- **The new spellings compile.** On rustc 1.96, edition 2024, these
  compile:
  - `trait PureFn: Fn() -> u32 + Pure {}` and
    `trait PureValue: Value + Pure {}`, each with a blanket impl over
    `unsafe impl<T: ?Sized> Pure for T {}`. `&dyn PureFn` is callable and
    upcasts to `&dyn Fn() -> u32`.
  - `-> impl Iterator<Item = u8> + Pure`.
  - A proc-macro attribute on a bodiless trait method.
  - 03 §10's bodiless `#[bend] #[ensures(.., by = ..)] pub fn ..;` under an
    expanding attribute.
  - The `#[tide::main]` path, through the spike's prelude.

Six new problems:

- **B1 (v0).** D13, RUST-6, and the reference tell a Rust consumer to put
  `std = { package = "tide_std" }` in its own manifest. That line compiles
  the consumer itself against Tide's std.
- **B2 (v0).** `#[trusted_pure]` replaced v0's `unsafe impl Pure`. The
  reference and 02 now teach it in v0, but 01's Tiers table and 03's tier
  line still put it in v1.
- **B3 (v0).** The reference teaches trait-method contracts in v0. 04 tiers
  them v1, and a contract proc macro cannot carry a trait's clauses to its
  impls.
- **B4 (v0).** CLAUSE-4 makes non-`&mut` parameters in `ensures` denote
  their entry values. The D18 lowering reads them at return, and a moved
  non-`Clone` parameter has no defined meaning.
- **B5 (v0).** The Thesis still promises "much faster builds", and D12
  keeps its old build and runtime targets. Under D18, rustc debug builds run
  in every development profile, and 01's own evidence measures that path at
  53 s clean.
- **B6 (v1).** The clause forms `#[requires]`/`requires!`,
  `#[ensures]`/`ensures!`, `#[decreases]`/`decreases!`, and
  `#[invariant]`/`invariant!` cannot all be macros exported from one
  prelude. Rust gives attribute and `!` macros one namespace.

## D18 change-list status

| D18 item (01:918–930) | Status | Evidence |
| --- | --- | --- |
| D1 spellings, modules, `.rs` | Applied | 01:163–212; reference §1, §2.1, §12.1 |
| D2 no `T: Owned` rewrite | Applied | 01:230; 02 §2.3, §3.7; reference §3.4 and §14 (row removed) |
| D3 float functions | Applied | 01:316–327; 03 PAR-3; reference §4.3. Gaps in the function set are N4 |
| D5 stack overflow aborts | Applied | 01:360–361; 02:727–731; 03 PANIC-1; reference §9 |
| D7 allowlist, `#[trusted_pure]`, companion traits | Applied | 01:409–420, :445–468; 03 §2.5, §3.1. The tier of `#[trusted_pure]` is B2 |
| D10 attribute contracts | Applied | 01:589–603, :638–640, :674–678; 04 Spelling, §1.1, §1.2. See B3, B4, and B6 |
| D12 backends, cross-check is the build | Applied | 01:726–754; 02 §2.8, §11. The targets are B5 |
| D13 direct Rust import | Applied | 01:768–777; 03 §11. The consumer manifest is B1 |
| D14 typed-hole spelling | Applied | 01:786–788; reference §13 |
| Tiers table | Applied | 01:82–83; README:135–145 |

## New blocking problems

### B1. A Rust consumer told to inject `tide_std` as its own `std` stops being a Rust crate

**Location.**
- `design/01-design.md:774–777` (D13): "Their manifest injects Tide's std
  with `std = { package = "tide_std" }`".
- `design/03-authority-concurrency.md:1237–1244` (RUST-6): "The consumer's
  manifest injects Tide's std".
- `design/reference.md:1400–1403`: "A Rust package using a Tide crate adds
  `std = { package = "tide_std" }` to its dependencies".

Tier: v0. The reference teaches it, and 01's Positioning (61–63) promises
the outbound direction without a tier.

**Problem.** A dependency renamed `std` makes Cargo pass
`--extern std=…tide_std…` for the crate whose manifest declares it, so that
crate is built against Tide's std. The spike's probe shows this.
`spikes/d18/examples/cargo-rename/Cargo.toml` declares exactly this
dependency. Its `src/lib.rs` then resolves `std::fs::Fs`, and
`scripts/c1-facade.sh:60–63` expects `std::fs::read` to fail under
`--cfg probe_bad`.

A Rust consumer that follows D13 loses its own `std::fs`, `std::thread`,
and `std::env`. It becomes a facade-compiled crate that Tide's checker
never ran on. That contradicts RUST-6's "Rust code depends on it directly",
D7's "Rust code that calls a Tide crate is unchecked", and the Eon plan in
which Venus "stays Rust".

The spike's evidence for the consumer direction is the opposite setup:

- orbit-protocol, the Tide crate, was compiled against the facade;
- yazelix-orbit "built unchanged against it" (`research/15-rust-dialect.md:587–589`);
- research/15:640–644 puts the rename in the manifest of the crate that is
  built against the facade.

**Proposed fix.**

- In `01-design.md:774–777`, replace the second sentence with:

  > The Tide crate's exported `Cargo.toml` (D18) declares
  > `std = { package = "tide_std" }`, so Cargo builds that crate, and only
  > it, against Tide's std; the consumer's manifest and its own `std` are
  > unchanged.

- In RUST-6, replace "The consumer's manifest injects Tide's std with
  `std = { package = "tide_std" }`" with:

  > The Tide crate's exported manifest injects Tide's std with
  > `std = { package = "tide_std" }`; the consumer depends on the Tide crate
  > as on any crate.

- In `reference.md:1401–1402`, replace "A Rust package using a Tide crate
  adds `std = { package = "tide_std" }` to its dependencies" with:

  > A Rust package depends on a Tide crate as on any crate; the Tide crate's
  > generated `Cargo.toml` injects `tide_std` for that crate alone

### B2. `#[trusted_pure]` is v0 in the reference and 02 but v1 in 01 and 03

**Location.**
- Teaching it as v0:
  - `design/reference.md:1297` (§11 trust holes, v0): "A type the checker
    finds not `Pure` but that is authority-free | `#[trusted_pure]` …".
  - `reference.md:175–177` lists it among v0 attributes.
  - The commit removed it from Appendix A's v1 list (`reference.md:1692–1695`).
  - `design/02-memory.md:788–792` (§9, v0 core per its tier line) makes it
    the only way outside std to declare a rejected type `Pure`.
- Placing it in v1:
  - `design/01-design.md:83`: the Tiers table's v1 row reads "FFI with
    `#[trusted_pure]`".
  - `design/03-authority-concurrency.md:14`: "v1: … FFI, `#[trusted_pure]`,
    and `ThreadBound` (§5)". Its own §3.1 (355–358, v0) describes
    `#[trusted_pure(reason = "...")]` on types.

Tier: v0.

**Problem.** Draft 3.5's v0 had an escape for a type that the structural
rule rejects: `unsafe impl Pure` under an `unsafe` grant. An example is a
data structure that uses raw pointers internally. D18 replaced it with
`#[trusted_pure]` (01:462–465, 924), but 01's Tiers table and 03's tier
line still put that attribute in v1. A checker built from 01 has no v0
spelling for the case. A checker built from the reference accepts an
attribute that 01 defers. Under 01's "the tier table wins" (77–78), v0
loses a capability that it had in 3.5.

**Proposed fix.**
- In `01-design.md:82`, add to the v0 row after "`Pure`":
  "(`#[trusted_pure]` on types, as a trust hole, under an `unsafe` grant)".
- In `:83`, keep "FFI with `#[trusted_pure]` wrappers".
- In `03-authority-concurrency.md:14`, replace "FFI, `#[trusted_pure]`, and
  `ThreadBound` (§5)" with "FFI, `#[trusted_pure]` on foreign functions,
  and `ThreadBound` (§5)".

### B3. Trait-method contracts are v0 in the reference, v1 in 04, and have no D18 mechanism

**Location.**
- `design/reference.md:1202` (§10.1, v0): "Trait methods may carry
  contracts; every impl is checked against them."
- `design/04-contracts.md:19`: v1 includes "laws and trait contracts (§5)".
- `04-contracts.md:68`: the v0 row of §1.1's table allows the attributes on
  a "trait method, or impl method" with no tier mark.
- `04-contracts.md:193–195`: rule 6, "An impl method inherits its trait
  method's contract".
- `design/01-design.md:656–657`: untiered.

Tier: v0. The reference line predates this commit, but D18 changes what
it would take to implement.

**Problem.**
- **Tier.** The reference teaches a v0 feature that 04 places in v1.
- **Mechanism.** Under D18 the contract attributes are toolchain proc
  macros (01:384–386; 04:29). A proc macro sees only the item it is attached
  to. `#[ensures(..)]` on `fn k(&self) -> u32;` in `trait K` cannot add
  checks to a plain `impl K for X { fn k(&self) -> u32 { .. } }` in another
  module or crate. It also cannot cover calls through `dyn K`. Proc-macro
  contract crates need a second attribute on every impl, which then renames
  the method behind a checked provided method. The spike's macro parses only
  an `ItemFn` with a body (`spikes/d18/macros/src/lib.rs:60`), so it has no
  trait form, and research/15's criterion 3 did not test one.

**Proposed fix.** Choose one.
- **(a) v1.** Move the reference line to Appendix A ("trait-method
  contracts and their inheritance by impls"). In `04-contracts.md:68`,
  change "a function, trait method, or impl method" to "a function or
  inherent method; v1: a trait method or impl method".
- **(b) v0.** Name the spelling. For example, the impl block carries a
  toolchain attribute (`#[contracts]`), which the checker requires with a
  safe fix wherever the trait has contracts. List the mechanism as D18
  implementation work.

(a) is the smaller change.

### B4. CLAUSE-4's entry values for by-value parameters do not match the D18 lowering

**Location.** `design/04-contracts.md:416–419` (CLAUSE-4): "In `ensures`, …
other parameters denote their entry values." Against `04-contracts.md:441–444`
and `design/01-design.md:914–917`. These list the spike's contract gaps as
only a panic inside a clause, sampled-clause budgets, and `a..=b`. Tier: v0.

**Problem.** The spike lowers `#[ensures]` by running the body as an
immediately called closure and evaluating the clause afterwards
(`spikes/d18/macros/src/lib.rs:99–112`). A parameter is therefore read at
return, not at entry. With the spike's macros built unchanged on rustc 1.96:

- `#[ensures(result == x + 1)] fn bump(mut x: u32) -> u32 { x += 5; x - 4 }`
  panics with "ensures violated" on `bump(1)`. Under CLAUSE-4 the clause
  holds: `x` is 1 at entry and the result is 2.
- `#[ensures(result == v.len())] fn count(v: Vec<u8>) -> usize { let w = v; w.len() }`
  fails to build: E0382, borrow of moved value `v`.

The second case is a hole in the spec, not only in the spike. The entry
value of a moved by-value parameter whose type is not `Clone` cannot be
computed without a snapshot. CLAUSE-4 makes such snapshots an error for
`old`, but it does not say what a bare parameter means here. 04 lists other
spike gaps, so readers will take this one as working.

**Proposed fix.** Append to CLAUSE-4 (`04-contracts.md:419`):

> A by-value parameter that the body assigns or moves is read at entry
> through an implicit `old(..)`: it must be `Copy` or `Clone`, and otherwise
> naming it in an `ensures` is an error whose fix is `old(x.len())`-style
> narrowing.

Add "parameters' entry values" to the gap list at `04:441–444` and
`01:915–917`.

### B5. The Thesis and D12 still promise fast builds that D18's backend and 01's own evidence contradict

**Location.**
- `design/01-design.md:23–24`: "It removes … slow builds".
- `01-design.md:54`: "with Rust's guarantees and much faster builds".
- `01-design.md:738–741` (D12 targets): "clean debug build a few seconds;
  development-build runtime within 3x of release".

These are set against `01:731–737`, which makes rustc with LLVM the backend
"in every profile: debug builds for development, tests, and the simulator",
and against 01's Evidence (114–117). Tier: v0. These targets are E4's gate
(D0) and `00-first-principles.md:126`.

**Problem.** 01 cites `reviews/round1-performance.md` (M1, M3) for "emitted
Rust measured 53 s clean and 17 s incremental on a 100k-line crate".
round1-performance.md:142–147 records 53 s for a clean *debug* build, and
:147 says "native unoptimized code is 20–106x slower than release". D18
makes exactly that path the only build in every profile.

D12 answers the edit-to-test target with a possible interpreter (735–737).
It leaves "clean debug build a few seconds", "within 3x of release", and the
Thesis claim unchanged. research/15:22–26 itself says fast builds "no longer
distinguish Tide". Only the edit-loop check (Tide's front end, B) is
claimed fast, so the Thesis and D12 now claim what D18's own sources
measure as false.

**Proposed fix.**
- `01:23–24`: "and slow edit loops".
- `01:54`: "with Rust's guarantees and a much faster edit loop".
- `01:740–741`: replace "clean debug build a few seconds;
  development-build runtime within 3x of release;" with:

  > clean and incremental builds and development-build runtime set by E4
  > against rustc's debug profile (drafts up to 3.5 targeted a few seconds
  > and 3x of release with their own backends);

- Mark the matching row of `00-first-principles.md:126` the same way.

### B6. Attribute and `!` clause forms that share a name cannot all be exported macros

**Location.**
- `design/04-contracts.md:26–30`: "`#[requires(..)]` and `#[ensures(..)]`
  … `#[decreases(..)]` … `decreases!(..)` … `#[invariant(..)]` …
  `requires!(..)` and `ensures!(..)` … They are proc macros owned by the
  toolchain and exported through Tide's std prelude (D6)".
- `design/01-design.md:589–599` (D10).
- `design/reference.md:97`: `invariant!` is a std-supplied form in v0.

Tier: v1. v0 exports only `#[requires]`, `#[ensures]`, `#[invariants]`, and
`invariant!`, which do not collide. v1 adds `#[invariant(..)]` beside v0's
`invariant!`.

**Problem.** Rust gives attribute macros and function-like macros one
macro namespace. Re-exporting a `#[proc_macro_attribute] fn requires` and a
`#[proc_macro] fn requires` from one prelude module is E0252: "`requires`
must be defined only once in the macro namespace of this module" (rustc
1.96). So the four pairs D10 promises cannot all be exported as written.

The design still works if the `!` clause forms are tokens that the
enclosing function attribute rewrites before name resolution, which an
outer attribute macro can do. They then need not resolve to a macro at all.
That is a different claim, though, and v0's exported `invariant!` fallback
(the spike's `compile_error`, `macros/src/lib.rs:35–41`) would have to give
way to `#[invariant]` in v1.

**Proposed fix.**
- In `04-contracts.md:29–30`, replace "They are proc macros owned by the
  toolchain and exported through Tide's std prelude (D6)" with:

  > The attributes are proc macros owned by the toolchain and exported
  > through Tide's std prelude (D6); the `!` clause forms are recognized and
  > removed by the enclosing function attribute and are not exported, since
  > an attribute and a `!` macro cannot share a name; an unrewritten one is
  > a checker error with §1.2 rule 10's fix

- Make reference §2.1's `invariant!` row say the same.
- Otherwise, rename the v1 type attribute (`#[type_invariant(..)]`). That
  also removes the near-collision of `#[invariants]` and `#[invariant]`.

## Non-blocking notes

### N1. README is stale

`README.md:35` says "draft 3.6", and `:38–39` says D18's "changes to the
other decisions are not yet applied". `:68` says "Chapters 02–04 elaborate
draft 3.5". This commit edited the README's tier bullets, so it should also
change these lines:

- "draft 3.7";
- "and draft 3.7 applies its changes to the other decisions";
- "Chapters 02–04 elaborate draft 3.7".

### N2. The Feel does not build on a v0 toolchain

`design/01-design.md:984` writes `decreases!(xs.len() - i);` in a
`rust`-fenced example. CLAUSE-1 (`04-contracts.md:379`) and §1.2's grammar
(`:129`) make `decreases!` v1, and the reference does not teach it. A v0
`#[requires]` that leaves it in place fails with an unresolved macro.

**Fix.** Either drop the line from the Feel, as reference §10.1 does with
the same `max_index`, or state in 04 §1.2 rule 10 that v0's rewrite accepts
and ignores `decreases!`.

### N3. Channel ends must be Tide-owned, but RUST-6 lists only some owned types

Tide's channels have checkpoints, `Err(Cancelled)`, and `recv_deadline(t)`
taking Tide's `Instant` (03:113, :771; reference:962). Rust's
`mpsc::Receiver` cannot provide them, and its `recv_deadline` is unstable,
which `-Zallow-features=` forbids. The spike re-exported Rust's `mpsc`
(`facade/src/caps.rs`, `pub use std::sync::mpsc::…`), and that is where the
`recv_timeout` leak came from.

**Fix.** In RUST-6 (03:1240–1242) and D13 (01:776–777), replace the list
with "every type Tide's std defines (`HashMap`, `HashSet`, `Arc`, `Shared`,
channel ends, `Instant`, the capability types)".

### N4. The float respelling leaves some spellings unnamed

D3 (01:323–326) provides `float::bits`, `min`, `max`, `math::sin`, and
`math::exp`. Several cases have no stated spelling or admission status:

- **`from_bits`.** It is not an observer, but research/15:593–596 counts
  Eon's `from_bits` among the rejected sites. A binary codec needs it.
- **`f32` forms.** Eon's codec uses `f32::to_bits`. The spike's functions
  are `f64`-only.
- **`mul_add`.** 03 PAR-3 names it as the way to write FMA, while ADMIT-1
  bans "math methods … and the rest".
- **`sqrt`.** The reference (4.3) bans it, while PAR-3 treats IEEE's basic
  operations as native.

**Fix.** State the admitted set in reference §4.3 and D3:

- `from_bits` stays admitted;
- `float::bits32` or generic forms cover `f32`;
- `mul_add` and `sqrt` are either admitted, as correctly rounded, or moved
  to `math::`.

### N5. Some cross-references still assume the old backends

- **04 §2.6 (575–578).** Check deletion is still "decided per
  instantiation … the build cache (D12) keys each instance". 01 D10
  (664–666) says how a build deletes a check under D18 "is open until v1".
  Add that sentence to §2.6.
- **Counterexample replay.** D12 (735–737) lists it among the reasons for an
  interpreter. 04 §8 (1011–1012) and §11 (1145) replay counterexamples as a
  debug-build run. Pick one.
- **D1 (174–175) cites D12** for "`#[path]` and `extern crate` are errors", but
  D12 does not say so. Cite reference §12.1 (`FX0911`, `FX0912`), or add
  the sentence to D12.
- **D18 (906–909) says the driver "runs no build scripts or proc macros
  except the toolchain's".** D13 (758–761) and RUST-5 run an imported
  crate's build script in a sandbox. Qualify D18 with "for Tide crates".

### N6. A `main` without `#[tide::main]` is unspecified

Reference 6.2 says `main` "takes `root: Root` or nothing". 03 §1.3 makes
the generated entry start the runtime and sinks, and `tide sim <bin>` runs
`main` under seeds. A plain `fn main()` is valid Rust and would bypass all
three.

- **Required or optional.** State whether every binary's `main` needs the
  attribute (an FX code with a safe fix), or whether a plain `fn main()`
  gets a default entry.
- **Single-file roots.** Reference 12.1's "a binary root if it defines
  `main`" should follow the same rule.
- **The habit row.** The row at reference §14 (1440) cites "rustc E0580".
  D14 promises stable FX codes, so give it one.

### N7. Property tests rely on replacing Rust's `#[test]`, which no design text states

Rust's built-in `#[test]` rejects parameters. Reference §10.2's
`#[test] fn midpoint_stays_in_range(lo: u32, hi: u32)` therefore needs
Tide's std prelude to export its own `test` attribute in place of the
built-in macro, as research/15 says ("facade `test` attribute shadowing the
built-in one"). That works: on rustc 1.96 an imported proc-macro `test`
attribute replaced the built-in one on a function with parameters.

Separately, reference 2.3 (175–177) calls `#[error]`, `#[from]`,
`#[source]`, and `#[codec]` "toolchain proc macros". They are derive helper
attributes.

**Fix.** Add one sentence to D6 or reference 2.3 saying that Tide's std
supplies `test`. Reword the 2.3 list to "toolchain proc macros and their
helper attributes".

### N8. The Bend example breaks 04's one-expression rule

03 §10 (1147–1148) writes
`#[ensures(result.is_ok() ==> …, by = bend_law(..))]`. 04 §1.2 rule 1 makes
a top-level comma in a clause an error. The example is Later-tier.

**Fix.** Spell the claim as its own attribute,
`#[bend_law("kernels/LAWS.bend", "steps_keep_size")]`, attached to the
`ensures`, or exempt `by = ..` from rule 1.

### N9. "Under the simulator it fails the seed" needs a mechanism

D5 (360–361), 02:729, 03:246–247, and reference:1139 say this, but v0's
simulator runs on baton-passed threads in one process (03 §9.1). An abort
ends that process, and under `tide test` it ends every other test in the
binary too. State how the seed is still reported:

- one process per seed; or
- an abort hook that writes the seed and replay command before exiting.

### N10. D18's list of spike gaps is incomplete

The spike rewrites only `for` and `while` loops
(`macros/src/lib.rs:322`, `:363`). So `invariant!` at the top of a
`loop { }` reaches the fallback `compile_error`, although 04 §1.2's
`loop_body` covers every loop. The spike also checks invariants after every
iteration in all debug builds, more than CLAUSE-6 asks. Add `loop` (and
B4) to the gap list.

### N11. Small leftovers

- **`Pure` is still called an auto trait** in 01 D4 (343), "a missing auto
  trait (`Send`, `Sync`, `Pure`)", and in 02 §2.7 (267). Elsewhere it is a
  marker trait.
- **03 still has two `tide` fences** (657, 776) around Rust signature
  sketches. 02 and the reference fence the same kind of sketch as `rust`.
- **Open question 7's continuation line** (01:1040) lost its indentation.
- **The spike's `tide-admit` reports the `Pure` check as `FX0510`**
  (research/15:576). The design uses `FX0510` for the least-authority lint
  (03 §4.2). Renumber the spike, or note the difference, before the
  conformance corpus reuses its fixtures.

### N12. The reference budget is unmeasured after growth

`reference.md` grew from 85,203 to 87,334 characters. At the ratio that
trials/e1-round2/grading.md:617 implies (76 KB, about 19,000–21,000
tokens), that is about 21,800–24,100 tokens against D14's 25,000. It is
probably under budget, but with little room left. No tokenizer was
available here, so count it before E1.

## Ready to implement v0

**Not yet.** B1–B5 are v0 and each is a one-to-three-sentence edit:

- **B1** corrects how outbound interop is described.
- **B2** fixes a tier row.
- **B3** needs a choice. Moving trait-method contracts to v1 is the
  smaller one.
- **B4** adds one rule to CLAUSE-4.
- **B5** rewords the Thesis and D12 targets so they claim only what D18
  delivers.

After those, the dialect is described consistently enough to start the
checker. Rust spellings, rule IDs, codes, and clause semantics already
agree across 01, 02–04, and the reference.

- **B6** is v1. Settle it before any v1 clause form is added to the
  prelude.
- **N2, N4, N6, and N7** touch v0 text. Settle them before the checker's
  test corpus is written.

Validation: this was a read-only review of `fc4cb4c` (diffed against
`c8859b8`) on 2026-10-19. Rust programs were compiled with rustc 1.96.0
(ac68faa20 2026-05-25), edition 2024, with stdin closed, in a scratch
directory outside the repository (`/tmp/claude-1000/r12`):

- **B6.** Re-exporting a `proc_macro_attribute` named `requires` and a
  `proc_macro` named `requires` from one module: E0252.
- **What holds.** These compiled and ran (`5 2 3`, `7`):
  - companion traits `PureFn` and `PureValue` with blanket impls over a
    blanket `Pure`, with `&dyn PureFn` called and upcast;
  - `-> impl Iterator<Item = u8> + Pure`;
  - an attribute macro on a bodiless trait method;
  - 03 §10's bodiless `#[bend] #[ensures(.., by = ..)] pub fn` with an
    expanding `#[bend]`.
- **B4.** A copy of `spikes/d18/macros` was built offline with cargo 1.96:
  - `bump(1)` panicked with "ensures violated: result == x + 1 (in
    `bump`)";
  - `count` failed with E0382.
- **N7.** An imported proc-macro attribute named `test` replaced the
  built-in `#[test]` on a function with parameters (printed `42`).
- **B1** rests on the spike's committed rename probe
  (`examples/cargo-rename`, `scripts/c1-facade.sh:60–63`), which was not
  rerun.

No Tide checker exists, so every Tide verdict above is an inference from the
text. Only `reviews/round12-consistency.md` was written; nothing was staged
or committed.

## Resolution

Answered in the commit that adds this review.

- **B1:** fixed as proposed in D13, RUST-6, and reference §12.4: the Tide
  crate's exported `Cargo.toml` injects `tide_std` for that crate alone.
- **B2:** the Tiers table's v0 row lists `#[trusted_pure]` on types under an
  `unsafe` grant; 03's tier line keeps only foreign functions in v1.
- **B3:** option (a). Trait-method contracts and their inheritance are v1 in
  01 D10, 04 §1.1 and rule 6, and reference Appendix A.
- **B4:** CLAUSE-4 reads by-value parameters at entry through an implicit
  `old(..)`; the spike-gap lists in 04 and D18 name it.
- **B5:** the Thesis promises a faster edit loop; D12's build and debug
  runtime targets are set by E4 against rustc's debug profile, as is
  00's table row.
- **B6:** the `!` clause forms are rewritten by the enclosing function
  attribute and not exported (01 D10, 04 §0, reference §2.1).
- **N1, N2, N3, N11:** fixed (README draft 3.7; the Feel drops v1's
  `decreases!`; channel ends listed as Tide-owned; "auto or marker trait";
  03's sketches fenced `rust`; open question 7 reindented).
- **N5:** fixed: 04 §2.6 states check deletion is open under D18;
  counterexamples replay as debug-build runs; D1 cites reference §12.1;
  D18's build-script clause is qualified for Tide crates. D18's gap list
  also names `loop {}` rewriting (N10).
- **Left open:** N4 (float coverage), N6 (`main` without the attribute),
  N7 (property-test `#[test]`, derive helper wording), N8 (Bend `by =`),
  N9 (stack overflow under the simulator), the spike's `FX0510` numbering
  (N11), and N12 (token count, to be measured before E1).
