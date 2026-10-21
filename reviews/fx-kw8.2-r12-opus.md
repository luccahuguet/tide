Model: Opus 5.5. Date: 2026-10-21. Round 12 (narrow), candidate 2a82549.

## Verdict

**Clean.** The fix closes the r10/r11 finding. I found no new findings and no nits worth blocking on.

## Scope

Narrow converge round, report only, over `git diff 5295926 2a82549 -- crates conformance`:

- `crates/tide-admit/src/walk.rs`: `raw_place` steps up through a `Field` or a builtin `Index` only when the stepped-from expression has no `Adjust::Deref` adjustment, builtin or overloaded. The doc comment covers this, and the FX0610 text now says "at most `FnMut`".
- `conformance/cases/authority/root_zeroed.rs`: three new FX0517 marks (`indexed_through_reference`, `indexed_through_deref`, `field_through_reference`) and a header sentence.

I read AGENTS.md, GOVERNANCE.md, r10-opus, r11-codex, the full `raw_place` function and its caller (`visit_expr`, walk.rs:1645). No tracked file was edited. The worktree stayed clean throughout.

## Findings

None.

## Checked

**Mechanism.** After writeback's `fix_index_builtin_expr` strips the autoref and unsize adjustments, a builtin index base keeps only its autoderef steps. So `Adjust::Deref(_)` on the base marks exactly the implicit loads: `Deref(Builtin)` for `&`, `&mut` and `Box`, and `Deref(Overloaded)` for user `Deref` and `DerefMut`. A direct array or slice base keeps no adjustments. The probes below confirm both. Field autoderef records its steps on the base expression in the same way. Every `id` the loop checks is in the same body as `t`, as with the existing `is_method_call` call, so the lookup cannot cross owners. When the climb stops, the report still names the outermost raw pointee (`made`), which is the right type: the implicit load reads it or a field of it.

**Reproducers now rejected.** The probe `k2r12/rej.rs` runs through `conformance/run.sh` with marks, under the unsafe grant. All 15 marked lines get FX0517, and the case gives PASS: rustc and the driver agree on every mark in both lib and test modes. It covers:
- `*const &[Fs; 1]`, `*const &&[Fs; 1]`, `*const Box<[Fs; 1]>`, and a generic `*const &[X; 1]` indexed under `&raw const`
- `(*p)[0] = f` and `&raw mut (*p)[0]` with `p: *mut &mut [Fs]`
- a user `Deref` base, both indexed (`W`) and by field (`V`), plus the explicit `(**p)[0]` control
- `&raw const (*p).0` through `*const Box<(u64, Fs)>`, and `(*p).0 = 1` through `*mut &mut (u64, Fs)`
- nested `(*p).a[0]` with `a: &[Fs; 1]`, and `(*p)[1].0` over `[&(u64, Fs); 2]`
- `(*p).0[0]` through `ManuallyDrop<[Fs; 1]>`, and a union field `u.a[0]` through `ManuallyDrop`

r10's `k2r10/deref.rs` through `k2r10/drv.sh` now reports lines 19 (`by_index`), 27 (`by_field`), 31 (`through_ref`) and 35 (`explicit`). Line 23, the plain `*mut [W; 1]` index control, stays clean.

**Stays clean.** The probe `k2r12/ok.rs` gives PASS with no marks. It covers:
- direct `&raw const` and `&raw mut` over arrays, slices, fields, and the nested `(*p).a[1].0` and `(*p).a[0].1`
- stores into arrays, slices, nested projections, a `Fs` element, and a tuple field
- `&raw const *p`, and a raw-pointer-typed field
- the `ManuallyDrop` field itself (no deref)
- union `&raw const u.a` and `u.b.1`
- a `Pure` read `(*p)[1]`, and `&raw const (*p)[0]` with `p: *const &[u64; 2]` (an implicit load of a `Pure` reference, correctly accepted)
- a raw deref as an index operand
- safe code indexing and projecting through `&`, `Box`, `Vec` and `&mut [u64]`, including `&raw const r[0]` and `&raw const b.0`

**No ICE.** No probe, conformance case or test produced a panic or an internal compiler error. `conf.log` holds zero ICE or panic lines.

**Marks fail at 5295926.** I built a scratch copy at `k2r12/base` (`git archive 2a82549` with `5295926:crates/tide-admit/src/walk.rs`, its own `target/`). Its `conformance/run.sh authority/root_zeroed.rs` gives FAIL. The driver reports the 20 old marks and misses exactly lines 89, 106 and 110, the three new marks. On the same base, `rej.rs` catches only line 39 (the explicit `**p` control) and misses the other 14, while `ok.rs` still passes.

**Suites at 2a82549** (load about 1.9, 10 GB free before each):
- `CARGO_BUILD_JOBS=4 cargo test --workspace`: exit 0. tide-admit unit tests passed 2/2 and `tests/driver.rs` passed 5/5.
- `conformance/run.sh`: exit 0, with 180 PASS, 28 AWAITS CHECKER and 0 FAIL.

**Message change.** The text "at most `FnMut`" appears only at walk.rs:700, next to the matching code comment at :696. No case or test matches on that text.
