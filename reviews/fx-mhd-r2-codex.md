Model: gpt-6.1-sol / Effort: high / Date: 2026-11-11

Verdict: **request_changes** — high confidence. The original reproducers are repaired, but the replacement pipeline still has three semantic gaps and one unsupported-const diagnostic gap.

Reviewed `23c407f`; HEAD, diff hash, and clean worktree remained unchanged.

| Finding | Re-check result | Defect in touched behavior |
|---|---|---|
| 1 | Resolved. Endpoint borrow survives the loop in debug and release. | None observed. |
| 2 | Original macro entry, classification, loop, and receiver probes resolved. | Residual implicit-capture gap N1; loop-scope regression N2. |
| 3 | Destructured inputs and struct field labels resolved. | Loop bindings remain incorrectly replaced: N2. |
| 4 | Argument and initializer reproducers resolved; closures and nested blocks also pass. | Format arguments can silently lose implication: N3. |
| 5 | Resolved through the authorized fallback. Both profiles emit the explicit alias diagnostic without E0500. | None observed; parenthesized references and old-only alias use pass. |
| 6 | Resolved. Non-Clone shared referents are rejected in both profiles; Clone positives pass. | None observed. |
| 7 | Original probe resolved. Checked const functions work at compile time, runtime, in array lengths, and in function-body const generics. | Unsupported enclosing generics produce an ill-formed expansion: N4. |
| 8 | Resolved. Parenthesized ranges compile and preserve the tested schedule. | None observed. |
| 9 | Documentation resolution accepted. Rerun still prints `RECORD None`; transport remains explicitly deferred. | None introduced. |

**N1 — Blocking: implicit format captures bypass entry snapshots.**  
[contract.rs:615](/home/lucca/pjs/lab/tide/.worktrees/fx-mhd/crates/tide-macros/src/contract.rs:615), [expr.rs:266](/home/lucca/pjs/lab/tide/.worktrees/fx-mhd/crates/tide-macros/src/contract/expr.rs:266)

```rust
#[ensures(result == format!("{n}"))]
fn f(mut n: u8) -> String {
    let r = format!("{n}"); n += 1; r
}
```

A debug test calling `f(0)` incorrectly panics: the clause captures exit `n`. Explicit `n = n` passes. This is a residual finding-2 gap. Expose implicit captures to parameter discovery and replacement.

**N2 — Blocking: builtin traversal does not preserve loop binding scopes.**  
[contract.rs:557](/home/lucca/pjs/lab/tide/.worktrees/fx-mhd/crates/tide-macros/src/contract.rs:557), [contract.rs:671](/home/lucca/pjs/lab/tide/.worktrees/fx-mhd/crates/tide-macros/src/contract.rs:671)

```rust
#[ensures(matches!({
    let mut s = 0;
    for n in 0..2 { s += n; }
    s
}, 1))]
fn f(n: u8) {}
```

`f(42)` incorrectly panics in ordinary debug execution. A `while let Some(n)` variant also fails. Renaming the loop binding passes. Both replacement and old extraction lack these scopes: `old(n)` inside these loops incorrectly snapshots the parameter instead of rejecting the bound variable.

**N3 — Blocking: normalized implication is mistaken for a named format argument.**  
[expr.rs:270](/home/lucca/pjs/lab/tide/.worktrees/fx-mhd/crates/tide-macros/src/contract/expr.rs:270)

```rust
#[ensures(format!("{}", p ==> result) == "true")]
fn f(p: bool) -> bool { false }
```

A debug test calling `f(false)` incorrectly panics. The tagged assignment representing `==>` enters the named-argument branch, which visits only its RHS; it becomes `p = result`. Parenthesizing the implication passes. Preserve the distinction between tagged implication and genuine named arguments.

**N4 — Blocking diagnostic gap: const helper loses enclosing generics.**  
[const_fn.rs:49](/home/lucca/pjs/lab/tide/.worktrees/fx-mhd/crates/tide-macros/src/contract/const_fn.rs:49)

```rust
struct S<T>(T);
impl<T: Copy> S<T> {
    #[ensures(result == 1)]
    const fn f() -> usize {
        let _ = core::mem::size_of::<T>();
        1
    }
}
```

Both profiles fail with E0401 because the generated nested function cannot capture `T`. The unannotated baseline and equivalent free generic function pass. The documented free-function limit is acceptable; this unsupported combination needs a clear contract diagnostic instead of an ill-formed expansion.

The fallback mechanisms themselves are justified for this slice: rerunning the author’s experiments reproduces E0507, E0502, and E0015. Alias rejection is correctly bounded on the tested paths; const rejection needs N4 closed.

Validation used candidate artifacts: 72 serial driver probes plus three experiments, nice 19, four Cargo jobs, closed stdin, and timeouts. Maximum observed load was 9.03; minimum available memory was 11.77 GiB; no quiet window was active. The author’s fixture passes all four profile/test combinations. Quantifiers in `matches!` guards and nested-old rejection inside builtins behave correctly. [Commands and results](/home/lucca/.cache/eon-agents/tide-fx-mhd-work/r2-supervisor-probes/results.jsonl).

Skipped full-suite reruns and held-out files. A const-item generic ICE also reproduces without contracts, so it is outside this narrow verdict. No repository edits or git/`br` writes occurred.