Model: Opus 5.5. Date: 2026-10-21. Round 10 (narrow), candidate 5295926.

## Verdict

**Findings.** One blocking finding. Both r9 Codex findings are resolved, but the ROOT-4 index fix makes a new false accept reachable: an index whose base has an overloaded `Deref` autoderef adjustment.

## Scope

`git diff ce6d728 5295926 -- crates conformance`. The diff changes `parallel_closure`'s `&mut` tracking and `raw_place`'s parent walk in `crates/tide-admit/src/walk.rs`, and adds new cases to `conformance/cases/tasks/par_shared_reject.rs` and `conformance/cases/authority/root_zeroed.rs`. I read AGENTS.md, GOVERNANCE.md, `reviews/fx-kw8.2-r9-codex.md`, ROOT-4 (design/03-authority-concurrency.md:345-369), and the surrounding walk.rs code (`visit_expr`'s adjustment loop, `pure_ty`, `reads_captures_only`). The worktree stayed clean. Probes are in `scratchpad/k2r10/`.

## Findings

### 1. ROOT-4: an index through an overloaded `Deref` on the raw place hands safe code a forged `&T` (blocking; a regression for `Index`, pre-existing for `Field`)

`crates/tide-admit/src/walk.rs:539-544`. The new `Index` arm checks only `!t.is_method_call(parent)`, so it does not catch an autoderef adjustment on the base. When `*p` has type `W: Deref<Target = [u8; 1]>`, `(*p)[0]` is a builtin array index, but its base `*p` carries an `Adjust::Deref(DerefAdjustKind::Overloaded(_))`, which calls `W::deref(&*p)`. That gives a user `Deref` impl a `&W` that no constructor made: an implicit `&*p`, which ROOT-4 lists as a raw-pointer read. Because the walk climbs to `&raw const` and returns, nothing reports it. At ce6d728 the old loop stopped at `Index` and reported FX0517 for `W`, so the `Index` route is a regression from this change. The `Field` arm has the same gap and predates the change: `&raw const (*p).f` with `V: Deref<Target = T>` goes through `V::deref`.

Probe `k2r10/deref.rs` under `--tide-grant=unsafe`. rustc accepts it, and the candidate driver reports only line 35, the explicit `**p` control:

```rust
pub struct W(Fs);
impl Deref for W {
    type Target = [u8; 1];
    fn deref(&self) -> &[u8; 1] { let _ = self.0.write_synced("x", b"y"); &BYTES }
}
pub fn by_index(p: *const W) -> *const u8 { unsafe { &raw const (*p)[0] } }    // no FX0517
pub fn by_field(p: *const V) -> *const u8 { unsafe { &raw const (*p).f } }     // no FX0517 (pre-existing)
pub fn explicit(p: *const W) -> *const u8 { unsafe { &raw const (**p)[0] } }   // FX0517
```

The new doc comment states the principle this violates: "an overloaded `Index` borrows its base, so it reads". An overloaded `Deref` on the base borrows it the same way.

**Minimal fix.** Do not step up through a base that has an overloaded deref adjustment. Then extend the doc comment ("…; an overloaded `Index` or `Deref` borrows its base, so it reads") and add `by_index` (with `//~ FX0517`) to `root_zeroed.rs`:

```rust
let overloaded = |e: &hir::Expr<'tcx>| {
    t.expr_adjustments(e)
        .iter()
        .any(|a| matches!(a.kind, Adjust::Deref(DerefAdjustKind::Overloaded(_))))
};
...
hir::ExprKind::Field(base, _) if !overloaded(base) => id = parent.hir_id,
hir::ExprKind::Index(base, ..)
    if base.hir_id == id && !overloaded(base) && !t.is_method_call(parent) => ...
```

I applied this patch to a scratch copy (`k2r10/ws`, separate target dir) and checked it. It builds on the pinned nightly. With it, `by_index` and `by_field` get FX0517, `root_zeroed.rs` keeps the same 20 diagnostics, and the full conformance run gives 180 PASS / 28 AWAITS / 0 FAIL. The fix leaves builtin derefs alone. For example, `&raw const (*p)[0]` with `p: *const &[Fs; 1]` stays clean: it reads a reference no code sees and yields only a raw pointer, and that pointer still needs a flagged deref before anyone can use it.

No other findings.

## Checked

- **r9 finding 1 is resolved.** These calls get FX0610 ("a callable through a `&mut` is only `FnMut`"): `par_for(Box::new(&mut f))`, `par_for(Box::new(Box::new(&mut f)))`, `par_for(&boxed_mut)`, `par_for(&&r)` with `r: &mut F`, and `par_for(Box::new(f))` with `f: &mut dyn Fn`. Peeling `&`, `&mut`, and `Box` is correct: `&F: Fn` iff `F: Fn`, `Box<F>: Fn` iff `F: Fn`, and `&mut F` is never `Fn`, so any `&mut` layer rules out `Fn`.
- **FX0611 keeps priority.** A closure that captures a `Cell` and is passed as `&mut f` or `Box::new(&mut f)` gets FX0611, not FX0610, because the capture loop runs before the `through_mut` check.
- **No false rejection.** `k2r10/par_ok.rs` is clean: `par_for(&f)`, `Box::new(f)`, `Box::new(&f)`, `&Box::new(f)`, `&&f` with a reading capture, `par_map(Box::new(&f))`, and `join(&mut a, &mut b)` (JoinClosure is not `shared`, so `&mut` is fine there). Ordinary safe indexing (`xs[0] = v; &xs[1]`) and indexing through a raw pointer to `u64` data are clean.
- **r9 finding 2 is resolved.** These are clean under the grant: `&raw const (*p)[0]` on arrays and slices, `(*p)[1][0] = v`, `&raw const (*p).0[i]`, and `&raw const (*p).a[1].1`.
- **Overloaded `Index`/`IndexMut` and index operands are still checked** (`k2r10/root.rs`). These get FX0517: `&raw const (*p)[0]` and `(*p)[0] = v` on `*const/*mut Vec<Fs>`, `&(*p)[0]`, `ptr::read(&raw const (*p)[0])` (FX0517 on `Fs`), a raw-read index operand `(*p)[(*q).0]` (FX0517 on `(usize, Fs)`), range indexing `(*p)[0..1]`, and `&mut (*p)[1]`.
- **No ICE on ill-typed input** (`k2r10/ill.rs`): unknown names, a wrong closure arity, a non-callable `&mut u8`, a `&str` index, indexing a `*const u8` pointee, and a `u8` index into `[Fs; 1]`. There are no panics. The only extra output is FX0517/FX0610 alongside rustc's own errors, which is harmless.
- **Comments are accurate,** apart from the `raw_place` doc gap in Finding 1. One nit, not blocking: the FX0610 message says "only `FnMut`", while the code comment's "at most `FnMut`" is the precise wording, since `&mut F` with `F: FnOnce` is not even `FnMut`.
- **Runs at 5295926.** Before each run, load was 1.8 and 13 GiB was available.
  - `CARGO_BUILD_JOBS=4 cargo test --workspace`: 2 + 5 passed.
  - `conformance/run.sh`: exit 0, 180 PASS / 28 AWAITS CHECKER / 0 FAIL. Both new cases pass.
