Model: claude-opus-5-5 · Date: 2026-11-17 · Candidate 156fcc7

# Opus r2 re-check: fx-qlek.5

## Verdict

approve

- All r2 findings are fixed.
- The fixes introduce one new text-only nit: a stale line in the record (R1 below), which does
  not block.
- The full diff `fab99b3...156fcc7` hashes to `a4cdd728…7975b6e`, as stated.
- I re-checked only `git diff 6a30b7c 156fcc7` and the builder's round-2 report. I ran nothing
  that compiles.

## B1: FX0402 now covers projection self types (fixed)

The self-type test at `crates/tide-admit/src/walk/traits.rs:138` is now
`ty::Param(_) | ty::Alias(..)`.

- **The route is closed.** The builder ran the review's probe unmarked against 6a30b7c through
  `conformance/run.sh`. It passed, so both rustc and the driver admitted it. The probe is now
  `mod projection` in `memory/escape_dyn_overlap_transmute`, with `//~ FX0402` on the impl, and
  that case passes. Without the widening, the mark would go unmet and the case would fail.

The obvious variants are all covered:

| Variant | How the rule handles it |
| --- | --- |
| Nested projection, `<<P as A>::B as C>::D` | Still `ty::Alias`, so it is caught. |
| Concrete projection that normalizes to `dyn` (the E0371 bypass) | `Alias`. `is_sized` normalizes, so it is caught. |
| Plain `type` alias | rustc expands it to the underlying parameter or projection, so it is caught. |
| Opaque type (`impl Trait`) as the self type | rustc already rejects it in an impl header. |
| Lazy type aliases, inherent associated types | Unstable, so the driver rejects the feature. |

- **No new false rejections that matter.** A projection is flagged only when it may be unsized.
  Two cases stay admitted:
  - a projection whose associated type is implicitly `Sized` (`type Assoc;`);
  - a concrete projection that normalizes to a sized type.

  The existing accept cases (`escape_dyn_overlap_accept`, `pure_*`, `dyn_*_accept`) pass
  focused. Flagging the rare remaining impls is consistent with the rule's fail-closed design.
- **Docs agree with the code.** README:94, conformance/README.md, rules.md REF-0402, 03 RUN-4
  hole 5 and the diagnostic now say "a type parameter or a projection". README:18, 01 D7 and
  02 R3 name the route without describing its shape, so they stay true. RUN-4 says "overlaps"
  while the others say "may overlap". That describes the route, not the rule, and is fine.

## M2–M5 (recorded, each for the right reason)

- **M2, route 42** (`modules/escape_cfg_attr_path`): FX0911, on an out-of-line module through
  `cfg_attr`.
  - The target file `modules/extern_crate_accept.rs` exists and is clean, so no stray
    diagnostic can stand in for the expected code.
- **M3, route 43** (`authority/escape_hooks`): an uncoded rustc `error` on the attribute line.
  - Recorded reason: Tide's prelude does not export `global_allocator`. That matches how
    built-in attribute macros resolve through the prelude.
  - The oracle's exact line/code match keeps the error on that attribute.
- **M4, route 44** (`authority/escape_linkage`): FX0901 for `#[unsafe(no_mangle)]`.
- **M5, route 45**: `vfork` and `execveat` both get `EPERM`.
  - `execveat` is passed AT_FDCWD (-100) and `shell` as argv, and `shell` is in scope in the
    sandboxed branch. A Landlock denial would give `EACCES`, so the asserted 1 does pin seccomp
    as the layer.
  - `vfork` was added to the exit-at-once guard (`56..=58`), so if it ever succeeded, the child
    would exit at once.
  - The focused run passed.

## N1–N5 (folded in)

- **N1:** the message now says "may overlap", and the over-rejection is recorded under "Left for
  others".
- **N2:** `pure` matches every `Component` variant and calls `kind().is_verbatim()`, so the
  variant keys and the `Prefix*` and `PrefixComponent*` globs now go through the driver.
- **N3:** recorded as a hand-off for tide-lead (fx-djp or fx-qlek.7).
- **N4:** README:18 is rewrapped.
- **N5:** route 35 is corrected. `sandbox.rs:571` allows `SYS_tgkill`, which confirms the
  record's note that seccomp lets `tgkill` through.

## R1 (new, text only, non-blocking): the record's "Fixes" bullet is stale

`reviews/2026-11-17-red-team-opus.md:36` still lists the third condition as "its self type is a
type parameter that may be unsized". The r2 revision note, route 6b and the code all say "a type
parameter or a projection". Suggested fix: add "or a projection" there.

This is text only, so it needs no re-check.

## Note for tide-lead

The builder ran only focused checks on 156fcc7: 11 conformance cases, `escape_path_io`, and the
RUN-3 test. Per AP-PROOF-001, the full `cargo test --workspace` and `conformance/run.sh` still
have to run on 156fcc7 before landing. This is not a finding.

DONE
