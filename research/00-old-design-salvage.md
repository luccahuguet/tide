# Old Ferox Design: Salvage Review

Date: 2026-10-19

Read in full: `spec.md`, `spec/VISION.md`, `spec/LANGUAGE.md`, all of `spec/language/` and `spec/tooling/`, `spec/TOOLING.md`, `spec/IMPLEMENTATION.md`, `V1_DIRECTION.md`, `ROADMAP.md`, `V2_BACKLOG.md`, `V3_BACKLOG.md`, `FEATURE_REVIEW.md`, `COMPARISON.md`, `MEMORY_MODEL_RESEARCH.md`, `GOVERNANCE.md`, `ZERO_LANGUAGE_FINDINGS.md`. Skimmed: the naming, trademark, and domain reviews.

The new goal this review measures against is a language that is as safe and fast as Rust, much lighter and faster to compile, strongly typed, has optional machine-checked proofs, and is well suited to AI coding agents writing, checking, and maintaining code.

## 1. What the old design was

Old Ferox was a design-first project for a compiled, statically typed, ownership-based language. It targeted backends, infrastructure, CLIs, and long-lived codebases, and its pitch was "Rust-grade safe-code guarantees ... through a simpler and more restrictive model." Its organizing idea was *safe change*. That meant low semantic entropy ("one dominant way, with at most one explicit escape hatch"), costs visible in the source, and tooling treated as part of the language: a canonical formatter, structured JSON diagnostics, and a contract for automated edits. The v1 core looked like Rust: `fn`/`let`/braces/semicolons, move by default with an explicitly declared `copy struct`, and two borrow kinds with strictly lexical scopes (no NLL, no disjoint-field borrows, no partial moves). It also had ADTs with exhaustive `match`, `Option`/`Result`/`?`, small traits without associated types, `[]` generics, and `async fn`/`await`/`async let` structured concurrency. Sharing went through explicit `Shared[T]`/`SyncShared[T]` plus `Cell`/`RefCell`/`Mutex`/`RwLock`, and unsafe code went through explicit `unsafe`. Effects, arenas, typestate, and verification were all deferred: contracts (`requires`/`ensures`/`spec fn`) were a v2 candidate and proofs were v3. The plan was a Rust bootstrap with a query-oriented compiler, a lossless CST, the formatter built before the type checker, Cranelift first and LLVM later, eventually reaching self-hosting. The output is about 38,000 words of spec prose and 727 lines of Rust scaffolding spread across seven crates.

## 2. Ideas worth keeping

Confidence means how likely the idea is to survive the new goal unchanged.

### Design discipline

**K1. Low-entropy rule and admission filter.** Sources: `spec/VISION.md` (Low-Entropy Rule), `GOVERNANCE.md` (Design Admission Rule). Why it matters: whenever two idioms do the same job, an LLM effectively picks one at random, which gives inconsistent code and noisy reviews. For the compiler, every overlapping feature is another path to check and test. The admission questions carry over almost unchanged ("does this reduce or increase the number of overlapping ways to solve the same problem?", "can this live in tooling or libraries instead?"). Add two more: "what does it cost in compile time?" and "can an agent predict the checker's verdict before running it?" Confidence: high.

**K2. Honest kill criterion.** Source: `COMPARISON.md`: "if Ferox drifts toward Rust-level semantic breadth, the project loses most of its reason to exist ... If Ferox fails that test, users should just use Rust." This keeps a restart honest. Confidence: high.

**K3. A narrow, enumerated safety claim that also lists what it does not claim.** Source: `spec/language/ownership.md` (Safety Guarantees, Safety Boundary Non-Goals). It names exactly what safe code rules out: use-after-free, double free, dangling borrows, uninitialized reads, null dereference, data races, and invalid discriminants. It states the conditions under which that holds, and it lists what is *not* promised: deadlock freedom, panic freedom, logical correctness, and leaks through shared cycles. A soundness test suite and a future verifier are both written against a list like this, and agents need to know which bugs the compiler will not catch. Confidence: high.

**K4. Ownership invariants stated independently of syntax.** Source: `ownership.md` (Core Ownership Invariants). The seven invariants are live owner, derived access, unique mutation, lexical borrow, drop-exactly-once, explicit sharing, and concurrency boundary. With them comes the rule that every `unsafe` escape "must document which invariant it temporarily suspends and what contract it must re-establish." Starting from invariants is the right way to specify a memory model, and invariants turn directly into proof obligations. Invariant 4 (lexical borrow) only survives if the memory model does (see D1). The other six fit most candidate models. Confidence: high, except for invariant 4.

### Types and code shape

**K5. Explicit signatures, inference only inside function bodies.** Source: `spec/language/types_and_patterns.md` (Inference Boundaries). Parameter and return types are always written out, and only `let` is inferred. Each function can then be type-checked from signatures alone, which makes checking parallel, incremental, and cacheable. Agents can read an API without reading bodies, and diagnostics stay local. Confidence: high.

**K6. ADTs, exhaustive `match`, `Option`/`Result`/`?`, no null, no exceptions, no implicit conversions.** Sources: `types_and_patterns.md`, `errors.md`. Exhaustiveness turns "the agent forgot a case" into a compile error. Error conversion is deliberately explicit: "`?` does not cross incompatible return types automatically ... conversions between error types are not implicit". That avoids Rust's hidden `From` behavior. Confidence: high.

**K7. No user-defined syntax macros.** Sources: `spec/language/syntax.md`, `FEATURE_REVIEW.md` F48. Macros are a major compile-time cost in Rust, and they hide the code an agent has to reason about. The redesign still needs some answer for `derive`-style boilerplate, either built-in derivations or checked code generation. Confidence: high.

**K8. Public-data modeling doctrine.** Sources: `VISION.md`, `types_and_patterns.md`. No clusters of boolean flags, no sentinel values: meaningful states become enums, `Option`, or `Result`. This can be enforced by a lint, and it pushes agents toward types that both the checker and a verifier can see. Confidence: medium-high.

**K9. A small pattern language with an explicit exclusion list.** Sources: `syntax.md`, `types_and_patterns.md`. There are no guards, or-patterns, ranges, aliases, or rest patterns, and "omitted fields are not implicitly ignored" in struct patterns. Exhaustiveness checking stays cheap and obvious. Because patterns must name every field, adding a field breaks every `match` on that struct, which is exactly the signal an agent needs during a refactor. Or-patterns deserve a second look, because without them arms get duplicated. Confidence: medium.

**K10. Blocks as expressions, `if` as an expression, assignment as a statement.** Source: `syntax.md`. Confidence: medium-high. The rule that the last expression without a semicolon is the block's value is a separate question, covered in §4.

**K11. An explicit cost model.** Sources: `VISION.md` (Performance And Feedback), `ownership.md`. Allocation is never hidden. Duplication is an explicit `clone(a)`. Implicit copying applies only to a declared `copy struct` whose fields are all copyable. Values never become shared implicitly, and single-threaded sharing is a different type from synchronized sharing. An agent can read performance straight off the source, which supports the "as fast as Rust" goal. Confidence: high.

**K12. Unsafe hygiene.** Source: `spec/language/mutation_and_effects.md`. Unsafe code lives in `unsafe { }` blocks and `unsafe fn`. The body of an `unsafe fn` is *not* implicitly unsafe, which matches the direction Rust 2024 took. There is no module-wide unsafe mode, and every unsafe site can be found with a plain text search. That keeps the audit surface small, and each unsafe block can later carry a contract. Confidence: high.

**K13. Concurrency boundaries that follow ownership.** Source: `spec/language/async_and_concurrency.md`. Data crosses between tasks only by move, by copy of a copyable value, or through explicit `SyncShared`. Ordinary borrows never cross task boundaries. Every task belongs to a lexical parent, and detached tasks do not exist. This gives data-race freedom without Rust's inferred Send/Sync. Keep these access modes. The async machinery that carried them is questioned in D3. Confidence: medium-high.

**K14. One module per file, explicit `use`, no wildcard imports, only `pub` and private.** Source: `spec/language/modules.md`. The origin of every name is visible in the file, so agents do not have to guess, and the module graph can be computed before type checking. The old spec only listed "cyclic import support as a design goal" among its non-goals. Make it a rule: imports must be acyclic. That enables parallel compilation. Confidence: high.

**K15. Fully opaque types with private constructors.** Sources: `VISION.md` (Promising Adjacent Idea), `V2_BACKLOG.md`. Opacity is total, with no "can match but cannot construct" middle ground. Outside the defining module, a verifier can then assume the type's invariants hold. Now that proofs are a goal, this belongs in the core rather than in v2. Confidence: medium-high.

### Tooling and compiler

**K16. Structured diagnostics as a contract.** Source: `spec/tooling/machine_contract.md`. Diagnostics carry stable dotted codes (`ownership.use_after_move`), a primary span with byte offsets plus line and column, related spans ("value moved here"), suggested fixes, and a schema version. Each output is tagged `stable`, `provisional`, or `internal`. This is the main interface between the compiler and an agent. Confidence: high.

**K17. Safety tiers and refusal rules for automated edits.** Source: `machine_contract.md`. Automated edits are tagged `safe_apply`, `suggest_only`, or `internal_only`. The tool refuses outright when "applying the rewrite would require inventing missing code". Every batch of edits declares whether it is atomic and whether it must be revalidated. An agent can then apply compiler fixes without review only when the compiler has proven them safe. Confidence: high.

**K18. Lessons from Zero.** Source: `ZERO_LANGUAGE_FINDINGS.md`. The useful parts are checked semantic edits, detecting edits made against a stale view of the program, and the compiler serving language docs that match its own version. This is the strongest outside evidence in the repo for what "agent-ideal" means beyond JSON output. Its recommendation still holds: keep text as the canonical source, expose a semantic query command (`inspect`), reject edits made against a stale content hash, and ship a language guide that matches the compiler version. Confidence: high.

**K19. A lossless CST with a round-trip invariant, plus error-tolerant parsing.** Source: `spec/IMPLEMENTATION.md`: "source text -> tokens plus trivia -> CST -> reproduced source should be identity-preserving." Doc comments live in the CST, not in a side table. Confidence: high.

**K20. A query-oriented compiler keyed by stable IDs, with explicit cache boundaries and a warning against over-engineering.** Source: `IMPLEMENTATION.md`, which defines identity layers (workspace, package, target, file, module, item, symbol), cache boundaries per file and per item, and says plainly: "The goal is a small, explicit, reusable architecture, not an infrastructure research project." Confidence: high for stable identities and cache boundaries; medium for building a general query engine early.

**K21. A canonical formatter, enforced by `fmt --check` rather than by `build`.** Source: `spec/tooling/formatter.md`. No column alignment, one item per line with trailing commas in multiline lists, and a single import block sorted by path. This gives deterministic diffs for agent edits. Confidence: high.

**K22. A semantic corpus with a pass case and a fail case for every invariant, and snapshots that only update on explicit request.** Source: `IMPLEMENTATION.md` (Semantic Corpus, Safety Invariant Validation). This becomes the conformance suite. It doubles as worked examples and evaluation data for agents. Confidence: high.

**K23. The shape of the verification layer.** Sources: `VISION.md` (Verification Direction), `IMPLEMENTATION.md` (Verification-Layer Validation), `V2_BACKLOG.md`. Contracts (`requires`/`ensures`/`old()`/`spec fn`) attach to ordinary code. Ghost code is erased before code generation. Verification failures come out through the same diagnostic pipeline, verified and unverified modules coexist, and everyday code never contains proof terms. This is the right shape for optional proofs as users see them. Its timing and its stance on purity have to change (D4, D5). Confidence: high on the shape.

**K24. Deterministic packages.** Source: `spec/tooling/packages.md`. At most one version of each package in a build, a lockfile, immutable published versions, explicit workspace members, and explicit failure on unsupported modes. This makes builds reproducible, and allowing only one version per package also cuts compile work. Confidence: medium-high.

**K25. Storage reuse only as an optimization.** Source: `IMPLEMENTATION.md`. Perceus-style reuse is allowed, but it must "not change observable ownership, aliasing, drop timing, or synchronization rules." Confidence: high for this rule. The Cranelift-first backend choice is reopened in §4.

## 3. Ideas to drop or rethink

**D1. Strictly lexical borrows, no disjoint-field borrows, no partial moves** (`ownership.md`). This is wrong for agents and wrong for speed. It rejects correct code such as `let w = &mut user; rename(w, "x"); save(user);`, and the spec's "preferred style" is to wrap borrows in artificial blocks. Agents trained on Rust will write NLL-style code, hit errors, and usually work around them with clones, which costs runtime performance. Borrow checking is usually a small share of Rust's compile time; most of it goes to monomorphization, macro expansion, and LLVM. So lexical scoping buys little compile speed. The rules also break the spec's own examples. `display_name` does `some(value) => value.name`, which moves a `String` out of an owned struct; under a strict reading of the no-partial-moves rule, that is rejected. Rethink the memory model from scratch instead of tuning this one.

**D2. The question nobody answered: references in return types and struct fields.** The spec never says whether `fn first(xs: &[T]) -> &T` is legal or how a struct could hold a reference, and it has no lifetime syntax. All it says is that "borrows may be passed to functions." That is the hard part of any Rust-class memory model, and the old design deferred it simply by not mentioning it. The redesign has to choose explicitly. Options include Rust-style lifetimes; second-class references, as in Hylo's mutable value semantics, where references are only parameters and are never stored or returned; regions; or something else.

**D3. `async fn`/`await`/`async let` as the v1 concurrency mechanism.** Async brings function coloring and needs an executor, which contradicts "no hidden global runtime." In Rust it also brings Pin and a lot of user confusion. It is heavy for the compiler and a frequent source of agent mistakes. Start instead with scoped threads or structured tasks that use the same access modes as K13. Decide separately, and later, whether the language needs suspension at all.

**D4. Purity as an unchecked style preference** (`VISION.md`, `V1_DIRECTION.md`: "not a checked effect contract"). This cannot coexist with machine-checked proofs. `spec fn` and contract expressions need purity, or at least freedom from side effects, to be checked, and a verifier needs facts about framing and aliasing. Some form of checked purity or effect marker has to be in the core.

**D5. Verification scheduled for v2 and v3** (`ROADMAP.md`, both backlogs). Proofs are now a top-level goal. They don't have to be built first, but the core has to be designed so a verifier can be added without a retrofit. That means defined integer semantics, checked purity, aliasing guarantees a verifier can trust, and opaque types.

**D6. Self-hosting as a project goal** (`IMPLEMENTATION.md`). It is years of work that helps neither compile speed nor agents. A fast compiler written in a mature language is the better product. At most, keep self-hosting as an optional stress test.

**D7. Map literals written with `{}`** (`syntax.md`). `{}` is both an empty block and an empty map, and `{"host": "localhost"}` competes with block expressions in the grammar. That directly contradicts the stated goal of "low syntax ambiguity." Drop it.

**D8. Bare lowercase enum variants** (`syntax.md`, `types_and_patterns.md`). The pattern grammar allows both "name binding pattern: `name`" and unit variants such as `none`, `read`, and `admin`. Parsing a pattern therefore requires name resolution, and a misspelled variant quietly becomes a catch-all binding, a known footgun in Rust and Swift. Unqualified variants like `active` and `blocked` can also collide across enums. Rethink using qualified or sigil-marked variants (`.none`, `Status.active`) or a case convention the parser enforces.

**D9. Two path separators, and `[]` generics that collide with indexing.** Module paths use `.` (`net.http.Client`), while `Shared::new` and `http.Client::new()` use `::`, and field access also uses `.`. Generics use `identity[Int](1)`, but the contract examples in `VISION.md` index with `xs[index]`. Both choices push semantic information into the parser. The spec also contradicts itself: `modules.md` writes `Result<Value, ParseError>` and `traits_and_generics.md` writes `Result[T, ParseError>`. Re-decide all of this as one decision.

**D10. `Cell`/`RefCell` as the standard interior mutability** (`mutation_and_effects.md`). `RefCell` turns borrow errors into runtime panics, which is the opposite of what an agent needs. Revisit this together with whatever memory model replaces D1 and D2.

**D11. A prose-heavy spec whose examples nobody checked.** Every spec file repeats "Design Intent / Goals / Initial Non-Goals" boilerplate, and 38,000 words yield a few hundred normative rules. The examples contradict the rules in at least three places: `fn render(&self) -> String { self.username }` moves out of a borrow; `impl[T: Display] Display for Box[T] { fn render(self) ...}` breaks the "receiver form must match exactly" rule; and the `display_name` partial move from D1. Agents and implementers need a terse normative reference, an executable grammar, and a corpus that compiles every example in the spec. Keep the rationale in a separate file.

**D12. Process scaffolding built before any compiler existed.** This includes stability tiers for every output category, the three-version ladder, a merge rule that requires one specific vendor's AI supervisor (`GOVERNANCE.md`), "human-written production code is forbidden by default", partial-file formatting in the formatter, corpus tests for FP style preferences, and seven crates for 727 lines of code. All of it is premature: it costs attention without reducing risk. Keep the one-page admission filter (K1) and add the rest once there is something to stabilize.

**D13. Arenas deferred by default** (`ownership.md`). `MEMORY_MODEL_RESEARCH.md` actually recommended "one explicit lexical arena construct in v1", and the narrowing pass cut it. Deferral may still be right, but regions could be the lighter alternative to lifetimes. Compare the options instead of inheriting "deferred."

## 4. Decisions to re-decide consciously

| Topic | Old decision | Source | Why it needs a fresh decision |
| --- | --- | --- | --- |
| Name | Ferox; CLI `ferox` | `spec.md`, `PUBLIC_NAME_REVIEW.md` | It is still the recorded decision, but the old reviews call it "medium-risk" and "not a clean brand": live US `FEROX` trademark filings, `feroxbuster` (a Rust CLI tool), and active ferox.com, ferox.systems, and ferox.net.au sites. It was never legally cleared, and the fallback domain plan was `feroxlang.com`. Nothing in the new design depends on the name, so decide before publishing anything. |
| File extension | `.ferox` | `VISION.md` | Flagged at the time as still needing practical review. It is long; agents don't care, humans might. |
| Implementation language | Rust bootstrap | `IMPLEMENTATION.md` | Probably still right, but choose it with compile speed and agent productivity in mind. Don't inherit the seven-crate scaffolding. |
| Backend | interpreter, then Cranelift, then maybe LLVM | `IMPLEMENTATION.md` | Compile speed is now a headline goal. Options: a custom debug backend (the Go and Zig approach), emitting C, or Cranelift for debug builds and LLVM for release. |
| Syntax family | Rust-like: braces, `fn`, `let`, `->`, `;` | `syntax.md` | For: LLMs have a deep Rust prior and will write it fluently. Against: they will also invent Rust features that don't exist here. Decide how close to Rust to stay, deliberately. |
| Trailing-expression rule | an expression without `;` is the block's value | `syntax.md` | Agents and humans slip on this. Either require explicit `return` or make every slip a type error. |
| Grammar details | `[]` generics, `.` plus `::`, bare lowercase variants, `{}` maps | D7, D8, D9 | Decide together so the parser never needs name resolution. |
| Memory model | ownership plus strictly lexical two-kind borrows | `ownership.md` | D1 and D2. This is the central decision. |
| Generic instantiation | never decided | none | Monomorphization, dictionary passing, or a hybrid is the biggest single lever on compile time, and the old design says nothing about it. |
| Concurrency | `async`/`await` plus `async let` | `async_and_concurrency.md` | D3. |
| Effects and capabilities | none in v1; IO untracked | `mutation_and_effects.md` | Proofs need purity (D4). Capabilities may also matter for sandboxing agent-written code and for supply-chain safety, the direction Zero's standard library takes. |
| Verification layering | v1 none, v2 contracts, v3 proofs | `VISION.md`, `ROADMAP.md` | D5. Also choose the verifier approach, for example SMT-backed in the style of Dafny or Verus, or something else. |
| Machine contract | JSON diagnostics, edit tiers, source-first | `machine_contract.md`, `ZERO_LANGUAGE_FINDINGS.md` | Keep the direction. Decide what the first compiler ships; stable codes and fix suggestions from day one are cheap. |
| Traits and generics | no associated types, blanket impls, `where`, or const generics | `traits_and_generics.md` | Const generics matter for Rust-level performance (fixed-size arrays, SIMD); associated types matter for iterator ergonomics. Weigh both against compile cost. |
| Panic | `panic(...)` exists; unwinding never specified | `errors.md` | Abort-only is lighter (no unwind tables, no unwind safety) and easier to verify. |
| Packages | TOML, one registry, one version per package | `packages.md` | Reasonable defaults. Decide whether the manifest should be written in the language itself. |
| Governance | agent-produced-code rule, AI supervisor pass | `GOVERNANCE.md` | This is process, not language design. Decide it separately. |

### Gaps the old design never closed

A restart could silently inherit these absences:

- No closures or lambdas anywhere in the spec.
- No loops except `while let`: no `for`, `loop`, or plain `while`.
- No integer widths (only `Int`) and no overflow semantics.
- Slices (`&[Int]`) appear in examples but are never defined.
- No destructor or `Drop` mechanism, even though "types with explicit destruction behavior" are mentioned.
- Only `impl Trait for Type` is defined, yet the examples call inherent functions like `Shared::new`.
- No visibility rules for struct fields.
- No FFI or ABI story.
- No compile-time evaluation.
- No model for strings or characters.
- No rule for references in returns or struct fields (D2).
- No strategy for instantiating generics.

## 5. Syntax and semantics worth reusing

These are short quotes. Case conventions and brackets are subject to D8 and D9.

Code shaped as one named step per change of representation (`VISION.md`, `V1_DIRECTION.md`):

```ferox
fn load_profile(id: UserId) -> Result[Profile, LoadError] {
    let raw = read_profile(id)?;
    let parsed = parse_profile(raw)?;
    ok(parsed)
}
```

The contract layer (`VISION.md`). The `binary_search` example also uses an implicit `result` binding inside `ensures`:

```ferox
fn debit(account: &mut Account, amount: Int)
    requires amount >= 0
    requires account.balance >= amount
    ensures account.balance == old(account.balance) - amount
{ ... }
```

Declared copy (`ownership.md`): `copy struct Point { x: Int, y: Int }`, where "a user-defined type may only be marked `copy` if every stored field is copy-safe" and "copy-safety is part of a type's definition, not inferred ad hoc at each use site."

Three variant shapes in one enum form (`types_and_patterns.md`): unit (`end`), tuple (`number(Int)`), and named-field (`connected { user: User, at: Time }`).

Structured-task rules that work with or without async syntax (`async_and_concurrency.md`):

- "capturing a non-copy value ... moves that value unless it is wrapped in an explicit concurrent-sharing type"
- "using the bound value without `await` is rejected"
- "normal successful scope completion with an unfinished `async let` binding is rejected"

Error propagation (`errors.md`): "`?` is shorthand for ordinary early-return control flow, not an exception mechanism," with no implicit error-type conversion.

Comments (`syntax.md`): `//` is an ordinary comment, `///` documents the next declaration, and `////` documents the file. All three are kept in the CST.

Imports (`modules.md`, `formatter.md`): `use net.http.{Client, Request, Response};`. There is one contiguous import block sorted by full path, aliases sort by their underlying path, and wildcard imports are not allowed.

Diagnostic record (`machine_contract.md`, abridged):

```json
{ "schema_version": "1", "tool": "ferox check",
  "diagnostics": [{ "severity": "error", "code": "ownership.use_after_move",
    "message": "value used after move",
    "primary": { "file": "src/main.ferox", "span": { "start_byte": 42, "end_byte": 46 } },
    "related": [{ "message": "value moved here", "...": "..." }],
    "suggested_fixes": [] }] }
```

Edit tiers (`machine_contract.md`): "`safe_apply`: ... preconditions are fully checked and the result is guaranteed to remain within the validated contract"; "`suggest_only`: ... a human should review"; refuse when "applying the rewrite would require inventing missing code rather than transforming known code."

Round-trip invariant (`IMPLEMENTATION.md`): "source text -> tokens plus trivia -> CST -> reproduced source should be identity-preserving when no formatting or rewriting step is applied."
