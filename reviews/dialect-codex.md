# Tide second-model review: Rust dialect study (research/15)

## Verdict

**request_changes**

Reviewed HEAD `0ca13231bc17efd8b00d0b1b46c361ed1ab03374` on 2026-10-19.
The target was the **uncommitted working-tree version** of
`research/15-rust-dialect.md`, SHA-256
`93f5deb6b86e354adc8939d8f655b4874f11939e4fb439c2d806f0c269621b18`.
Read `br show fx-enq` and the authoritative design sources.

**Review inference:** the dialect remains plausible, but the recommendation
does not yet establish D7/D15 or its claimed cost advantage. The experiments
below refute specific proposed mechanisms; they do not prove that every
possible checker on stock rustc must fail. No Tide checker exists to test.
Confidence is high in the reproduced mechanism failures and moderate in the
cost assessment. I found no independent backend obstacle to computing D11;
its authority summaries still depend on closing the admission gaps below
([research/15:188–195](../research/15-rust-dialect.md#L188-L195)).

## Blocking findings

### B1. Safe `core` calls escape the stated D7/D15 argument

The study closes `core` through the fixed macro set and unsafe grants, then
claims all remaining nondeterminism is in the facade
([research/15:171–195](../research/15-rust-dialect.md#L171-L195)).
That misses safe hardware queries.

**Observed experiment:** rustc `1.96.0 (ac68faa20 2026-05-25)`, host
`x86_64-unknown-linux-gnu`. A `tide_std` rlib re-exported Rust's 2024
prelude and `println!`. Compile with:

```sh
rustc --edition 2024 --extern std=libtide_std.rlib cpuid.rs -o cpuid
```

`cpuid.rs` contained:

```rust
#![forbid(unsafe_code)]
fn main() {
    println!("apic={}", core::arch::x86_64::__cpuid(1).ebx >> 24);
}
```

Six executions of the same binary printed `apic=0`, `54`, `40`, `54`, `0`,
`16`. No capability, forbidden macro, or unsafe operation was involved.
This contradicts authority-free determinism
([01:393–402](../design/01-design.md#L393-L402)). Separately, safe
`core::sync::atomic::AtomicU64::new(0)` compiled without `Tasks`, bypassing
the facade constructor policy and its scheduling instrumentation
([03:947–953](../design/03-authority-concurrency.md#L947-L953)).

**Fix:** replace the two-leak argument with an explicit, versioned admission
policy for resolved `core`/`alloc` items, including safe intrinsics,
constructors, inherent methods, and aliases. Add these negative cases to the
feasibility gate. Specify driver restrictions on extra externs, sysroots,
search paths, semantic flags, and cfg inputs. F1 proves name substitution,
not confinement. The existing address restrictions
([03:202–206](../design/03-authority-concurrency.md#L202-L206)) also need to
cover safe `.addr()` observers; my facade experiment compiled one and
returned different addresses across runs.

### B2. The proposed float replacement does not dispatch, and target pinning is insufficient

The float row says std-only math can be replaced by prelude extension traits
([research/15:114](../research/15-rust-dialect.md#L114)); the falsifier section
says remaining float results are deterministic per target
([179–186](../research/15-rust-dialect.md#L179-L186)).

**Observed experiment:** the facade prelude imported a `Math` trait whose
`impl Math for f64` returned `123.0` from `sin`. Safe user code compiled with
the same `--extern std=...` command printed `0` for `0.0_f64.sin()`. Real
std's inherent implementation remains available through the facade's
dependency graph and wins method lookup.

Moreover, Rust documents [NaN arithmetic](https://doc.rust-lang.org/std/primitive.f32.html#nan-bit-patterns)
as potentially producing different bits for repeated operations with the
same inputs. Its [float method documentation](https://doc.rust-lang.org/std/primitive.f64.html#method.sin)
also permits invocation-dependent precision; `min`/`max` permit either
signed zero for equal inputs. These are stronger problems than differences
between targets. This is a specification counterexample, not an observed
float replay failure.

**Fix:** retain deterministic semantics through checker-enforced explicit
helpers, UFCS, or wrapper types, and demonstrate their Rust dispatch. Audit
every float observer, including formatting and byte conversions, against
[D3](../design/01-design.md#L288-L292). Do not weaken D3 to "per target" and
claim that this alone preserves D15. Reclassify the row and update the
decision-change list accordingly.

### B3. `Pure` trait objects need a real v0 spelling

The proposed blanket marker cannot be combined as `dyn Trait + Pure`, and
the study leaves this as possibly unnecessary or solvable by a type alias
([research/15:122](../research/15-rust-dialect.md#L122),
[451–452](../research/15-rust-dialect.md#L451-L452)). The design already
assigns this spelling semantics
([02:457–459](../design/02-memory.md#L457-L459),
[780–809](../design/02-memory.md#L780-L809)); an alias does not legalize an
invalid trait-object type.

**Observed alternative:** with the facade's proposed unsafe marker and
blanket implementation, a safe user crate declaring
`trait PureValue: Value + Pure {}` and
`impl<T: Value + ?Sized> PureValue for T {}` compiled. Both a call through
`&dyn PureValue` and upcasting to `&dyn Value` compiled and ran successfully.

**Fix:** choose a companion-supertrait spelling and specify the checker's
purity enforcement, or explicitly narrow v0 and account for affected APIs.
Classify a supported spelling as L+C. Explain that the blanket marker alone
enforces nothing for unchecked Rust consumers; callbacks crossing that
boundary require a trust policy.

### B4. A loop-body `invariant!` does not preserve loop contracts

The proposed placement is at the top of the loop body
([research/15:131](../research/15-rust-dialect.md#L131)). Tide requires checks
at entry and exit, including an empty range, with the range index equal to
its exit value
([04:399–403](../design/04-contracts.md#L399-L403),
[480–491](../design/04-contracts.md#L480-L491)).

**Observed experiment:** implementing `invariant!(e)` as `assert!(e)` let
`for _i in 0..0 { invariant!(false); }` succeed. Likewise,
`while n < 1 { invariant!(n == 0); n += 1; }` exited with `n=1` without
checking the now-false invariant. A local invocation does not receive the
enclosing loop to transform.

**Fix:** specify an enclosing function attribute or loop-owning macro that
instruments entry, iteration, exit, and control-flow edges. Demonstrate empty
ranges and the next-index rule. Include sampled-clause operation budgets
and transitive calls in the lowering design; an ordinary assertion is not
the CLAUSE-6/7 contract.

### B5. Direct Rust linking is not universal zero-cost interop

The study acknowledges different map and Arc types
([research/15:124–129](../research/15-rust-dialect.md#L124-L129)), but calls
zero-cost interop the deciding difference for every boundary
([345–348](../research/15-rust-dialect.md#L345-L348)).

**Observed experiment:** a facade-owned `collections::HashSet<T>` was used
in a facade-compiled library's public parameter. Compiling an ordinary Rust consumer
that passed `std::collections::HashSet::new()` failed with E0308:
"similar names, but are actually distinct types". Rust consumers see the
facade types. Converting containers, sharing Arc ownership, or supplying
capabilities therefore needs an API-specific answer.

The inbound boundary also still needs authority enforcement. Current
[RUST-2](../design/03-authority-concurrency.md#L1127-L1132) puts capabilities
on wrapper constructors/functions and authority on returned handles.
Direct linking plus a symbol scan
([research/15:152](../research/15-rust-dialect.md#L152)) does not itself add
those requirements to a foreign function's Rust signature.

**Fix:** distinguish ABI compatibility, type identity, adaptation cost, and
capability wrappers. List which types are re-exports versus wrappers and
which public Eon signatures cross them. The leaf-crate proposal may survive:
at Eon `40492dc`, the observed HashSet uses in
`runtime/crates/eon-workspace-protocol/src/v2.rs:460–478` are internal, while
`orbit/crates/protocol/src/lib.rs:265–280` exposes String/Vec. That is
evidence for selected APIs, not every boundary. Preserve the imported-crate
trust and simulation caveats in the recommendation.

### B6. The v0 classification omits constraints and mixes incompatible class definitions

Three concrete omissions affect the promise to cover v0:

- **Opaque auto traits:** Tide forbids Send/Sync leakage from returned
  `impl Trait` ([01:270–274](../design/01-design.md#L270-L274),
  [02:474–481](../design/02-memory.md#L474-L481)). My facade experiment
  successfully passed `fn values() -> impl Iterator<Item=u8>` to
  `fn needs_send<T: Send>(_: T)`, without an explicit Send return bound.
  This needs an explicit C rule, not an assumption that Rust matches.
- **Typed holes:** the agent-toolchain row is C
  ([research/15:140](../research/15-rust-dialect.md#L140)), but D14's
  `_?name` form ([01:692–694](../design/01-design.md#L692-L694)) failed
  parsing in the experiment. Keeping only `todo!()` is a surface change.
- **Const evaluation and eval:** deterministic, fuel-limited `const fn`
  evaluation and interpreter-backed `tide eval` are required
  ([01:348–349](../design/01-design.md#L348-L349),
  [695–696](../design/01-design.md#L695-L696)); their implementation is not
  assigned before making the interpreter conditional on E4
  ([research/15:407–408](../research/15-rust-dialect.md#L407-L408)).

The class definitions also call valid-Rust attributes/library forms L
([research/15:96–101](../research/15-rust-dialect.md#L96-L101)), while main
and contract rows already provide such forms but label them X. The summary
omits module contract modes and says "one rule change" before listing two
([157–163](../research/15-rust-dialect.md#L157-L163)).

**Fix:** separate unchanged spelling, supported respelling, checker rule,
semantic change, and unresolved implementation. Add the missing rows and
their decision impacts. Do not treat valid Rust token syntax as proof that
the complete Tide behavior has been preserved.

### B7. The cost comparison leaves out dialect-specific proof and integration work

The demo table equates front ends and std work, omits a dialect execution
component, and concludes "about the same" minus the interpreter
([research/15:211–227](../research/15-rust-dialect.md#L211-L227)). F1 only
tested a small facade, whereas the new-language interpreter also owns const
evaluation and the semantic oracle
([01:643–655](../design/01-design.md#L643-L655)).

**Inference:** the dialect replaces execution work with a different
obligation: every accepted program's resolved Rust behavior must match the
checker's interfaces and Tide semantics, including facade prelude contents,
trait selection, contracts, cfgs, and dependency identities. Rustc's final
borrow check does not validate that equivalence. B1–B6 show concrete work
missing from the equal-cost cells. The separately implemented checker and
pinned rustc also require compatibility validation on upgrades, including
safe API additions, even without an edition change.

**Fix:** compare bounded prototypes with the same demo acceptance criteria.
Count interpreter/backend work on one side; admission checks, facade and
macro coupling, rustc invocation/cache integration, and compatibility tests
on the other. Label "years of work" as an unmeasured estimate. Keep the
likely backend savings as an inference; do not use an uncosted comparison
as an adoption premise.

## Non-blocking findings

### N1. Tool reuse is conditional integration, not "for free"

The study admits rust-analyzer is unverified but later lists Rust's tools as
retained ([research/15:242–245](../research/15-rust-dialect.md#L242-L245),
[366–371](../research/15-rust-dialect.md#L366-L371)). Its non-Cargo driver
also needs a project description. Rust-analyzer's
[official integration guide](https://rust-analyzer.github.io/book/non_cargo_based_projects.html)
requires crate/dependency/cfg information and explicit checking integration.

**Fix:** describe adapter work and test facade completion, contract macro
expansion, Tide diagnostics, and run/test commands before claiming tool
compatibility. Treat performance parity similarly: mandatory overflow
checks and facade/runtime behavior prevent deriving comparative performance
solely from using LLVM; the study itself leaves overflow cost open
([453–454](../research/15-rust-dialect.md#L453-L454)).

### N2. Learnability and positioning predictions need measurements

"E1 should change little" and "a few hundred tokens" saved are marked
inference, but neither is measured
([research/15:258–272](../research/15-rust-dialect.md#L258-L272)). The design
explicitly makes surface choices evidence-gated
([01:124–153](../design/01-design.md#L124-L153)).

**Fix:** compare actual minimal references and held-out tasks covering
contracts, empty-loop invariants, main wiring, and facade types. Include
Rust-habit failures. Present the product choice explicitly: a distinct
language versus a Rust toolchain profile whose guarantees depend on always
using Tide's checker and driver; syntax familiarity alone does not decide it.

### N3. The v1 reversal condition understates the known obstacle

The study records task isolation as unexpressible in Rust
([research/15:148](../research/15-rust-dialect.md#L148)), but its reversal
condition asks only whether colorless async generation works
([437–439](../research/15-rust-dialect.md#L437-L439)).

**Fix:** require evidence for non-Send locals across migration, scoped child
borrows, non-moving frames, recursive/dynamic suspension, and foreign pinning
([02:1251–1271](../design/02-memory.md#L1251-L1271),
[03:786–805](../design/03-authority-concurrency.md#L786-L805)). State the
cost to v0 users if v1 requires another source or semantic transition.

Validation: document analysis, official Rust/tool documentation, and bounded
scratch experiments only. No Tide runtime, complete facade, or checker
conformance was tested. Only `reviews/dialect-codex.md` was written in this
repository; no staging, commit, or other git mutation was performed.
