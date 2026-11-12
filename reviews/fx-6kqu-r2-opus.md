Model: claude-opus-5-5 · Date: 2026-11-12

Verdict: approve

Reviewed frozen candidate `c071d62` (one commit on `f853fa8`) in `.worktrees/fx-6kqu`. The worktree was clean and HEAD unchanged before and after the review. Codex r1's approval was read only after I formed this view; I did not rely on it.

## Assessment

The change does what the binding 2026-11-12 21:35Z decision asks, and nothing more.

- **Correctness.** `crates/tide-admit/src/walk/traits.rs:86-93` peels only `ty::Ref` (both `&` and `&mut`) and rustc's real `Box` (`adt.is_box()`, so a local or aliased `Box` cannot spoof it). It stops at every other constructor.
  - `Vec`, `Option`, `Rc`, slices, arrays, tuples, `dyn`, raw pointers and type parameters still get no type module.
  - Ownership still uses `is_local()` plus the exact `parent_module_from_def_id` comparison (`:94-109`). A wrapper impl is therefore accepted only in `T`'s exact module or the local trait's module. Child, sibling and ancestor placements still get the "move the impl" message.
  - The "no module can hold this impl" branch (`:110-111`) is still chosen only when both owners are absent. It still fires for `impl From<Local> for u16`, for wrappers around non-local types, and for `Box<dyn LocalTrait>`.
- **FX0406 is unaffected.** `heads` still uses the original `self_ty` (`:117-118`). My probe confirms that `impl<T> Root for Box<T> where Box<T>: Clone` still gets FX0406.
- **Inherent impls are unaffected.** `impl Local {}` has no wrapper to peel. Rust itself rejects inherent impls on `&T` or `Box<T>` (E0390/E0116), so the new branch cannot admit one.
- **Spec fit.**
  - D4 (`design/01-design.md:352-354`) and reference §5.2 (`design/reference.md:317-320`) say an impl lives with "the type". Reference §8's "also for &File" and companion §18's `impl Bar for &Feed` (`design/reference-companion.md:510`) already treat a wrapper impl as belonging to the wrapped type.
  - Limiting the look-through to `&`, `&mut` and `Box` matches the decision. `Pin` is right to leave out: it is not admitted (FX0520 today, FX0601 under fx-u65.5), so peeling it would add an unreachable branch.
  - The REF-0401 gloss (`conformance/rules.md:170`) now matches the implementation.
- **Tests.**
  - The accept corpus (`conformance/cases/memory/trait_placement_accept.rs:16-44`) covers all three wrappers plus mixed (`&mut Box<&Bag>`) and repeated (`Box<Box<Bag>>`) nesting. That catches a fix that peels only one layer.
  - The reject corpus covers child and sibling placements for each wrapper, nesting in a child, and `Vec<Local>` in `Local`'s own module, which is the right falsifier for over-peeling.
  - The driver test (`crates/tide-admit/tests/driver.rs:1114-1131`) follows the existing `include_str!` corpus pattern (`:1146`). It adds what the conformance runner cannot check: the move/no-module message split. Together they are strong enough for the risk.
- **Regression risk** is low. The only behavior change turns FX0401 into acceptance for wrapper impls in the wrapped type's exact module. Every original `//~ FX0401` mark in the reject corpus is kept.

## Findings

1. **nit:** the peel loop is now duplicated in a fourth place.
   - Evidence: `crates/tide-admit/src/walk/traits.rs:87-93` repeats the loop in `walk/sinks.rs:81-87`, `walk/surface.rs:304-310` and `walk.rs:819-828`.
   - Those copies serve a different invariant (resolving a callee to a closure), so the author's choice to keep the loop inline is defensible under AP-OWNERSHIP-001.
   - Fix I'd want: none for this bead. A later cleanup could add one `peel_ref_box(ty)` helper for the three identical copies.

## Out-of-scope observation (not a finding against this diff)

The type module still ignores `dyn LocalTrait` self types, and this predates the diff.
- Rust accepts `impl Display for Box<dyn Root>` and inherent `impl dyn Root {}`. Tide reports both as "no module can hold this impl", although D4's "module of the type or the trait" arguably points to `Root`'s module.
- The brief lists `dyn` among the placements that should still reject, so this diff is consistent with it.
- If the owner wants the case settled, it belongs in a separate bead. I filed nothing.

## Probes run

1. Driver probe, run once through the queue on 2026-11-12 at 22:06Z. The source is the scratch package `/tmp/claude-1000/-home-lucca-pjs-lab-tide--worktrees-fx-6kqu/1cc6c74a-6d41-4f43-a35c-58993f915e89/scratchpad/probe/pkg/lib.rs`. It reused the candidate's `target/debug/tide-admit` (built 21:36Z, after the fix).

   `(ulimit -c 0; CARGO_BUILD_JOBS=4 timeout 1800 bash /home/lucca/.cache/eon-agents/eon-overseer/heavy.sh nice -n 19 target/debug/tide-admit --edition 2024 --crate-type lib --emit metadata --error-format=short --out-dir <scratch>/out <scratch>/pkg/lib.rs </dev/null)`

   Result: exit 1, with exactly the expected diagnostics.

   | Line | Probe | Result |
   | --- | --- | --- |
   | 5-6 | `impl Shape for Box<S>` and `impl IntoIterator for &'static S`, in `S`'s module | accepted |
   | 12 | crate-root (ancestor) `impl IntoIterator for RefS`, where `type RefS = &'static mut inner::S` | FX0401, "move the impl" |
   | 16 | `impl inner::Shape for &[inner::S]` | FX0401, "move the impl" (slice not peeled) |
   | 17 | `impl inner::Shape for Box<(inner::S,)>` | FX0401, "move the impl" (tuple not peeled) |
   | 19 | `impl<T> Root for Box<T> where Box<T>: Clone` | FX0406 only, no FX0401 (trait module) |
   | 20 | `impl<T> Root for Option<T> where T: Clone` | accepted |
   | 21 | `impl Display for Box<dyn Root>` | FX0401, "no module can hold this impl" |
   | 24 | `impl Root for *const inner::S` | FX0520 only (raw pointers not admitted), no FX0401 (trait module) |
   | 25 | sibling `impl super::Root for &super::inner::S` | FX0401, "move the impl" |

2. Two earlier attempts produced no result. One timed out at 120 s while waiting in the queue. One exited 2 because the driver refuses an out-dir inside the package directory; that was a probe-harness error, not a candidate defect.

I did not run the full suites, the conformance corpus or E1. I did not open `trials/e1-checked/`, `report.md` or any reference-run directory, and I did not write to the repository or through `br`. DONE
