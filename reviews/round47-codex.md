Model: `gpt-6.1-sol`. Effort: `xhigh`. Date: 2026-10-20.

## Verdict

**request_changes — high confidence.**

The direction is sound, but whole-type seals leave one concrete interface escape. The independent safety review confirmed it. A small extension to hash coverage should resolve it.

## scope

Reviewed `git diff -- design/ README.md` against HEAD:

- **HEAD:** `5ec5c70d42491bd1f1e5e99ac2a718b926b6f924`
- **Diff SHA-256:** `762462696391f86634488b5116b63190df855eb6e25681dc74dc7e7c9ba38b24`
- **Files:** README, design/01, design/04, reference, and reference-companion.

Read AGENTS.md, GOVERNANCE.md, README.md, `fx-1am` through `br --no-db show`, research/13, rounds 43–46, and the surrounding design sections specified in the request. The diff hash remained unchanged throughout the review. `git diff --check` passed.

## Blocking findings

### B1. Whole-type seals omit associated definitions inside existing trait impls

**Locations:** [design/04-contracts.md:1276](/home/lucca/pjs/lab/tide/design/04-contracts.md:1276), together with module expansion at [line 1245](/home/lucca/pjs/lab/tide/design/04-contracts.md:1245).

A whole-type entry covers the type definition and the **set of trait impl headers**. It does not cover associated type or constant definitions inside those impls. Module sealing includes inherent methods, but does not require separate entries for these trait impl items. D4 explicitly permits associated types and constants.

For example, seal a module containing:

```rust
pub struct Feed;

impl Iterator for Feed {
    type Item = u8;
    fn next(&mut self) -> Option<Self::Item> { None }
}

pub fn new_feed() -> Feed { Feed }
```

A delegate changes only `type Item = u8` to `type Item = u16`. The struct, impl header, written method definition, and `new_feed` signature remain unchanged. SEAL-2 expressly excludes named types’ trait impls from interface hashes except where clauses call them.

The sealed API has changed: a parallel caller expecting `Iterator<Item = u8>` breaks. No seal entry changed, so there is no `FX0921`; the enumerated hash coverage provides no `FX0920`. No contract weakening, authority widening, trust hole, or protected-test change independently requires D11 approval.

**Minimal patch:** extend the whole-type rule with:

> A type’s whole entry also fixes the associated type and constant definitions in its trait impls, including their transitive hash dependencies.

This closes the gap while preserving the existing method-body policy.

## Non-blocking improvements

At [design/04-contracts.md:1311](/home/lucca/pjs/lab/tide/design/04-contracts.md:1311), explicitly state that review and CI recompute **every target/profile hash recorded in the base’s seal**, regardless of configurations built by the local command.

Checking against the complete base seal already implies this. Making it explicit prevents an implementation from accidentally inheriting SEAL-3’s narrower “configurations it builds” scope.

## Alternatives

Keep the toolchain lock design and extend its existing hash coverage as B1 describes. Freezing complete impl bodies would unnecessarily restrict delegated implementation. No broader redesign is warranted.

## Reasoning

- **D11’s approval model remains intact.** The same human-held keys, external key allowlist, signatures, and CI verification govern unsealing. Adding the `unseal` change class does not trigger the bead’s stop condition.
- **The direct escapes are covered:** deleting or rewriting the candidate seal, shadowing clause helpers, changing resolved assertion impls, widening sealed grants, and adding targets cannot replace the inherited boundary without detection.
- **The admission filter supports this approach.** It adds tooling over existing IDs and hashes, avoids syntax and solver requirements, and gives delegates fast failure. The separate diagnostics distinguish modifying protected content from modifying its lock.
- **Acceptance is otherwise substantially met:** the decision, tier, rejected alternatives, approval path, diagnostics, reference pointer, and worked example are present. B1 leaves “what is sealed” incomplete.
- The documented limits around unsealed callers, interface-sealed oracle helpers, and test-dependent bodies appropriately bound the guarantee.

Not checked: runtime enforcement—no checker exists—or an independent token recount; 24,987 is the recorded count. No files changed.