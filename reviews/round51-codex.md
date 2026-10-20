Model: `gpt-6.1-sol`. Effort: `xhigh`. Date: 2026-10-20.

## Verdict

**request_changes — medium-high confidence.**

The named findings are closed or honestly admitted. One applicability gap remains for sealed generic types.

## Scope

Narrow re-check of round 47’s B1 and note, rounds 48–50, and their fixes in §11.1 and companion §18.

- **HEAD:** `5ec5c70d42491bd1f1e5e99ac2a718b926b6f924`
- **SHA-256 of `git diff -- design/ README.md`:** `bf83df00d30d15b9c39b15723c660f61d507daf23a7ff339597807eaa71091c2`

The diff identity remained unchanged. No files were modified.

## Item status

| Item | Status |
| --- | --- |
| Round 47 B1: associated definitions in trait impls | Closed for the named `Feed` example by full impl coverage, excluding method bodies. |
| Round 47 note: recompute every recorded hash | Closed explicitly by SEAL-4. |
| Round 48: inherent constants and refined method returns | Closed by SEAL-1 and SEAL-2. |
| Rounds 48–50: blanket impls and impls for references or `Box` | Named concrete cases closed. Generic applicability remains unclear below. |
| Round 50: re-export captures an unsealed call | Honestly admitted by SEAL-6; sealed calls record their resolution. |
| Supporting notes | Trait-addition exception, private methods, deref dependencies, and companion guidance are addressed. |

## Blocking findings

### B2. The concrete applicability rule does not define coverage for generic sealed types

**Location:** [design/04-contracts.md:1285](/home/lucca/pjs/lab/tide/design/04-contracts.md:1285).

A whole entry can seal a generic type, but applicability now depends on trait selection over “those concrete types.” Consider a library containing:

```rust
pub struct Feed<T>(pub T);

impl<T: Copy> Iterator for Feed<T> {
    type Item = u8;
    fn next(&mut self) -> Option<Self::Item> { None }
}
```

There need be no concrete `Feed<X>` written anywhere. `Feed<T>` is not concrete, and its definition does not require `T: Copy`. Selection under the type definition’s bounds therefore cannot establish this conditional impl.

The rule leaves the driver without a specified answer: reject a valid generic seal, omit the impl, or introduce an unstated symbolic applicability test. Omitting it recreates B1: changing `Item` to `u16` breaks a parallel generic caller returning `Option<u8>`. SEAL-6 admits unsealed resolution changes; it does not admit changing associated definitions of a sealed type.

**Minimal patch:** retain the finite approach, but define generic applicability explicitly:

> Include impls whose resolved self type contains the sealed nominal type, including generic instantiations. For remaining blanket matches involving generic types, use conservative “may apply” matching, counting unresolved applicability as applying rather than requiring concrete instantiations.

This needs no enumeration of future instantiations or language change.

## Non-blocking improvements

None requested.

## Reasoning

For the named concrete cases, full impl coverage closes the associated-item escapes. Resolution hashing closes capture inside hashed callers and tests, while SEAL-6 accurately limits the guarantee for unsealed code. SEAL-4 also clearly separates complete gate verification from local build coverage.

The remaining issue is the new applicability definition. Rust distinguishes concrete selection from checking generic obligations under assumed bounds; the latter requires a parameter environment. That distinction needs an explicit policy here. [Rust Compiler Development Guide](https://rustc-dev-guide.rust-lang.org/typing-parameter-envs.html).

`git diff --check` passed. A Rust 1.96 probe reached MIR generation for the generic baseline; changing `Item` produced the expected `E0308` in its generic caller. Completing compilation was blocked by the read-only filesystem’s rejection of rustc’s metadata temporary directory.

Not checked: the rest of the diff or executable Tide enforcement; no checker exists.  
Risk: generic seal coverage remains unspecified until B2 is resolved.