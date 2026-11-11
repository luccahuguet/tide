Model: claude-opus-5-5 · Date: 2026-11-11

# fx-mhd round 4: Opus review (read-only)

Candidate: `8f1dc7c` on base `cd9d47e`. I verified the diff SHA-256
`e4f9205eb94d9ce04509b9e05153264721bdf32632da2301a38440acc3b12023` and found the worktree clean.

## Verdict: request_changes

The design follows §10.1, 04 §1.8 and the recorded decisions. The single syn-AST pipeline is a
sound shape, and the std runtime is the one owner of evaluation, blame, budget and records.
The Codex rounds tested the lowering against counterexamples, but three defects remain. All three
are in code this change adds, and probes against the candidate's production driver confirm each
one. Finding 1 also breaks the interface fx-m72.2 relies on.

Probes: three runs of the candidate's `target/debug/tide-admit` against the candidate's
`target/tmp/build/std/artifacts`, plus two plain-rustc mechanism checks. All ran in my scratchpad
under nice 19, with load about 2–3, 14 GiB available, and no quiet window. I made no repository
edits, ran no full-workspace build, and did not open `trials/e1-checked/`.

## Findings

### 1. `#[track_caller]` on the user function misplaces blame. (blocking)

Evidence: `crates/tide-macros/src/contract.rs:244-248` adds `#[track_caller]` to every function that
has a `requires`. For a requires-only function, `:344-345` places the body directly inside that
function. `crates/tide-std/src/contract.rs:114-121` then blames `Location::caller()`.

`#[track_caller]` applies to the whole function, not only to the checks. That has two effects:

- **Wrong caller for nested requires.** Location propagates through every `#[track_caller]` frame.
  When a requires-bearing function `g` calls a requires-bearing `f` with bad arguments, `f`'s
  violation is blamed on the line that called `g`, not on the line in `g` that called `f`.
  Probe through the driver:
  - Source: `#[requires(x > 0)] fn f(x: i32)`, `#[requires(y >= 0)] fn g(y: i32) -> i32 { f(y) }`,
    and a test whose line 14 calls `g(0)`.
  - Output: `requires violated: x > 0 (in `f` at probe.rs:14:25)`, which is the test's line,
    not g's line 9.

  §10.1 says "a failure blames the caller", and here the blamed line is the caller's caller.
  fx-m72.2's decision 3 says that a requires violation "blamed on a call site in the test's own
  body discards the case; deeper is a failure". With this lowering, a bug in `g` is discarded as a
  rejected input, so the generated test can never find it.
- **Wrong location for body panics.** Every panic in the body of a requires-only function is
  reported at the caller's line. This covers bounds checks, overflow, `unwrap` and `panic!`.
  Probe: in `#[requires(true)] fn h(xs: &[i32], i: usize) -> i32 { xs[i] }`, the index-out-of-bounds
  panic is reported at the test's line 15, not at `h`'s body on line 11.

Fix: confine caller tracking to the checks. Run the body of every requires-bearing runtime
function behind the same non-`track_caller` closure boundary the ensures path already uses
(`__contract::call`), so only the requires helpers see `Location::caller()`. This revises
decision 1's "requires-only prepends checks" for the reason above, so record the change on the
bead and in 04. Add two regressions:

- a requires-bearing caller that violates its callee's requires, with the blame on the inner call
  line;
- a body panic in a requires-only function, reported at the body's own line.

Const functions are not affected, because their path adds no `track_caller`.

### 2. Struct-literal shorthand drops entry-value replacement. (blocking)

Evidence: `Replace::visit_expr_mut` (`crates/tide-macros/src/contract.rs:556-567`) replaces the
`expr` of a shorthand `FieldValue`. But syn prints a shorthand field from its `member` alone
whenever `colon_token` is `None` (syn 2.0.119, `expr.rs:4119-4127`), so the replacement is
silently lost. The const path uses the same visitor (`const_fn.rs:82-86`).

Probe through the driver:

```rust
#[ensures(result == W { n })]
fn shorthand(mut n: u32) -> W { n += 1; W { n: n - 1 } }
```

`shorthand(0)` panics with `ensures violated: result == W { n }` in debug. The clause reads the
mutated live `n` instead of its entry value, so a correct function fails its contract. This is the
same class as round-2 N1 (implicit references to an input that the AST visitor misses), and
CLAUSE-4 makes entry values the meaning.

Fix: when a replacement hits a shorthand field, give the field an explicit form
(`colon_token = Some(..)`, so `n: <entry expr>`). Add the example as a regression.

### 3. Const violation messages are used as format strings. (blocking)

Evidence: `crates/tide-macros/src/contract/const_fn.rs:130-139` emits
`::core::panic!(#message)`, and `message` contains the clause's source text. Any `{` or `}` in a
clause on a const fn turns the expansion into an invalid format string. Examples are a struct
literal or pattern (`matches!(p, P { a: 1 })`) and a block.

A plain-rustc probe of this exact expansion fails with "invalid format string: expected `}`,
found `a`". This contradicts round 1's recorded rule that an expansion is never ill-formed.

Fix: emit `::core::panic!("{}", #message)`, which is const-compatible for a `&str`, or escape
the braces when building the literal. Add a const clause containing braces to the
`review_regressions.rs` const coverage.

### 4. Requires-only single-expression bodies trigger `unused_braces`. (non-blocking)

Evidence: `contract.rs:345` nests the original block as `{ checks… #body }`. The probe build warns
"unnecessary braces around block return value" at each of `fn f(..) -> i32 { x }`, `g` and `h`.
Every short function with a `requires` gets a rustc warning at the user's own span.

Fix: the closure boundary from finding 1 removes the warning. Otherwise, splice `body.stmts`
instead of nesting the block.

### 5. The ensures marker shows up in rendered documentation. (non-blocking)

Evidence: `contract.rs:249-252` and `const_fn.rs:7-10` push `#[doc = "__tide_has_ensures"]`. It
appears as text in rustdoc and in `tide doc` output. When the function has doc comments, the
marker joins their last paragraph, and the leading `__` turns into Markdown emphasis.

Fix: keep the marker that decision 1 of the interface comment asks for, but carry it where docs do
not render it. One option is a body-level marker item, or a call to a definition-recognized
`__contract` marker the driver reads from HIR. Another is the existing `__contract::ensures`
calls, which decision 9 already makes definition-identifiable.

### 6. Unchecked records are deduplicated by a linear scan with string comparisons. (non-blocking)

Evidence: `crates/tide-std/src/contract.rs:104-112`, called from `contract.rs:87-91`. In a debug
build outside tests, each call of a function with a sampled clause, and each loop head with a
sampled invariant, scans every record and compares `Clause` by content: text, function and file
strings. A hot loop with a sampled invariant pays that cost on every head.

Fix: deduplicate by identity. Options are a per-call-site `static` flag the macro emits, or a
hash set keyed by `(file, line, column, index, conjunct)`.

### 7. Two weak points in the lexical adapter. (nit)

My view on the adapter: tagging `==>` as an assignment so that syn owns precedence and
boundaries is a reasonable trade against writing a custom Pratt parser, and it is commented.
The fragile part is how a tag is identified: by source position
(`crates/tide-macros/src/contract/expr.rs:15-20`, `:96`, `:181`).

- **Position collisions.** If tokens share a span, any real `=` at the same position is read as an
  implication. This happens when another proc macro generates a clause with all-`call_site` spans,
  as fx-m72 or future std code might.
- **Keyword before `!(`.** `lex` skips any group that follows `ident !` (`:61-65`). That includes a
  keyword followed by negation: `if !(a ==> b) { .. }`, or `match`, `while` and `return` before
  `!(`. Such a clause then fails to parse with a confusing error.

Fix: exclude Rust keywords from the `ident !` macro test. When a real `=` collides with an
arrow's position, reject the clause with a diagnostic instead of guessing.

### 8. Implicit format captures are not exposed for `todo!`, `unimplemented!` and `unreachable!`. (nit)

Evidence: `expr.rs:333-340` gives these macros a format index of `None`, so `expose_captures`
skips them. Example: `unreachable!("{self:?}")` in the body of a `&mut self` method with an
`ensures` still captures the outer `self` next to the `__tide_self` reborrow, which is a
borrow-conflict error on valid code. This is the same N1 class.

Fix: give these macros index `Some(0)`.

### 9. Const `requires` blames the clause, not the caller. (non-blocking)

Evidence: `const_fn.rs:125-139` writes the clause's own location into the literal message, and
the const path adds no `track_caller`. That differs from §10.1's "blames the caller". 04
(`design/04-contracts.md:547-548`) says "the clause's location", but the README (`:175-182`) and
the std crate docs mention only the missing structured metadata.

Fix: in the README and the crate docs, say that a const requires violation names the clause's
location rather than the caller's.

### 10. The design chapter carries implementation notes and round labels, and the limits are stated four times. (non-blocking)

Evidence: `design/04-contracts.md:502-559` adds about 60 lines inside CLAUSE-7, a normative rule.
The text includes "Current implementation limits (fx-mhd round 1)", pipeline mechanics (tagged
nodes, labeled blocks) and other fix history. The same limits appear again in `README.md:166-182`,
in the `crates/tide-std/src/lib.rs` crate docs and in `conformance/README.md:169-179`. The bead's
decision asked for one sentence per research gap in 04.

Fix:
- Keep the ten gap decisions in 04 as one sentence each.
- Give the current limits a single owner. The tide-std crate docs fit, with the README pointing
  to them.
- Drop the round labels and the mechanism narrative from 04.

The limits themselves are stated truthfully: I checked the alias, const-Copy-entry,
identifier-parameter, receiver, old, sampled-in-test and thread-local claims against the code.

### 11. Duplicated test drivers and fixtures named after review rounds. (nit)

Evidence: `crates/tide-admit/tests/build.rs:13`, `:39` and `:65` are three copies of the same 2×2
profile matrix, differing only in fixture and expected count. The fixtures
`conformance/cases/contracts/review_regressions.rs` and `pipeline_regressions.rs` are named after
review rounds rather than rules.

Fix: use one test that loops over the fixtures. Name each fixture, or merge it into
`executable.rs`, by the rule it covers.

## Checked and fine

- **Interface for fx-m72.2 and fx-m72.3.**
  - One helper call per top-level requires conjunct (`contract.rs:230-243`), with conjunct
    source text preserved.
  - A structured `Violation` and `Unchecked` record with `take_*` accessors. The current-thread
    limit is documented at the accessors and in 04.
  - Contract attributes re-emit below `test` and `sim_test` (`contract.rs:161-170`), and the
    fixture's `contract_above_test` and `contract_above_sim_test` exercise this.
  - Blame correctness, which the callee rule needs, is finding 1.
- **FX0804–0807 registration.** It fits fx-u65.6's ownership. fx-u65.6 scopes only
  FX0802/0803/09xx. `--tide-print-codes` lists the codes the toolchain run decides, and the macros
  run inside that run. No overlap.
- **Single ownership.** Std owns evaluation, catch-unwind wrapping, blame, budget, schedule and
  records. The macro owns syntax, classification and enabling. The only duplicate is the const
  path's literal copy of the message format, which is unavoidable without const string
  formatting.
- **Both fallbacks.**
  - The alias diagnostic is type-directed through `SupportedEntry`, and the
    `MutableAlias::deref` `unreachable!` cannot run.
  - The const limits match their diagnostics and the build tests.
- **Leftovers.** The conformance stub drops `spike_macros`, the `Cargo.lock` is pruned, and
  `rules.md` rows are updated. I found no live references to the spike lowering outside
  `trials/e1-dialect` (frozen record) and `spikes/`.

DONE
