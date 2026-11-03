Model: gpt-6.1-sol · Effort: xhigh · Date: 2026-11-03

## Verdict

**approve — high confidence.**

## Scope

Reviewed fx-kw8.13 and its decision comment, the parent epic, required governance and design sections, all candidate changes, related driver and interface code, the earlier B1 review, and the installed pinned compiler sources.

Confirmed unchanged at completion:

- HEAD: `ceb39ded674577187d3de37d1f3d31a1db05d6c8`
- Diff SHA-256: `f5498c9ee87603d60955b43554669fb9c47cea9f72c3b2b2690d78adc6a6648b`
- Candidate tree, including the untracked case and excluding `.beads` changes: `bd42674510daf4f597802196c69b01ee6b4c0c15`

## Blocking

None.

## Non-blocking

A compact regression covering active and inactive `cfg_attr` would strengthen coverage of the expansion boundary at [invocation.rs:561](/home/lucca/pjs/lab/tide/crates/tide-admit/src/invocation.rs:561):

- `#![cfg_attr(all(), no_std)]` must receive FX0520.
- `#![cfg_attr(any(), no_std)]` must remain accepted.

The implementation handles both correctly on the inspected compiler pin.

## Reasoning

The new guard derives its eligibility from the operator’s `--tide-std` and runs before lowering. It therefore remains active when candidate source makes `Crates::tide_std()` return `None`.

I found no remaining admitted candidate-controlled route around it:

- **Conditional attributes:** rustc configures root attributes before injection and expands active, nested `cfg_attr` forms into attributes visible to this check. Inactive forms produce no `no_std`, avoiding false positives. This matches the [Rust Reference’s conditional-attribute semantics](https://doc.rust-lang.org/reference/conditional-compilation.html#the-cfg_attr-attribute).
- **Macro-produced attributes:** injection precedes macro expansion, so a subsequently generated attribute cannot suppress that injection. Root macro attributes capable of rewriting the earlier crate attributes require the unstable `custom_inner_attributes` feature, which ADMIT-2 blocks.
- **`no_core`:** the pinned compiler treats it as unstable. Enabling its feature is rejected by `-Zallow-features=`; omitting the feature also fails.
- **Extern renames:** [injected_std](/home/lucca/pjs/lab/tide/crates/tide-admit/src/crates.rs:162) selects the compiler-generated import by expansion identity and obtains that import’s resolved crate number. Candidate aliases cannot retarget it.
- **Tests and other files:** `--test` does not bypass the guard or remove the relevant root attributes. Module-level attributes do not control crate injection and are correctly excluded.

Rejecting `no_std` is consistent with D18’s injected Tide std and the updated ADMIT-1. Artifact identification alone would also need to ensure Tide std is loaded for a core-only crate. Rejection avoids that additional mechanism. Explicitly importing std under `no_std` remains valid Rust, so rejecting the attribute itself closes the relevant distinction. [Rust Reference](https://doc.rust-lang.org/reference/names/preludes.html#the-no_std-attribute).

The new per-line FX0520 expectation detects removal of this guard even though the static’s path independently fails admission. The driver test additionally covers `no_std` without an explicit std import and checks the diagnostic. Together with the existing static baseline, these satisfy the bead’s chosen acceptance contract.

Not checked: builds or execution, as requested; the passing workspace tests and 194-case corpus run are supplied evidence.  
Residual risk: attribute and injection behavior remains tied to the reviewed compiler pin.