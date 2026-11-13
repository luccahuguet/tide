Model: claude-opus-5-5 · Date: 2026-11-13

Verdict: request_changes

Scope: frozen candidate `9ccb2dc` on `71127b1`, the whole diff `git diff 71127b1..9ccb2dc`, read in context: recovery.rs in full; the walk.rs, authority.rs, surface.rs, traits.rs, habits.rs, diag.rs and labels.rs hunks; the driver tests; report.md (both fix rounds); the three Codex files; the bead and its coordinator decisions; AGENTS.md. I also read the pinned rustc-src (nightly-2026-08-09) at the query providers cited below. The candidate binary `target/debug/tide-admit` has SHA-256 `dd537186…6eb6`, the same as the author's final build. The worktree is clean at HEAD.

**Every finding comes from reading the source. No probe has run, because the Eon quiet window is still closed.** Each finding below is marked "probe pending" and has a confidence. Its staged probe is listed at the end. A "## Probe results" section will be appended after the release. It will confirm, drop or downgrade each finding.

## Summary

The direction is right. FX diagnostics are kept by not entering queries that can reach rustc's fatal trait lowering, and unresolved-path recovery stays in `walk/authority.rs`. The C flush is correctly limited to crates where rustc has reported an error. I found no invented diagnostic and no clean-crate output change.

But (A)/(B) model which queries a body can reach, and rustc has crate-global queries that lower *every* impl of a trait, or every inherent impl. A trait-keyed reference closure narrowed by impl head does not see those queries. Findings 1–3 are three instances. Each erases diagnostics for the rest of the walk. C limits the loss to diagnostics that were still buffered. This is the third round in which a closure over rustc internals has missed a path, so I recommend a different mechanism (finding 4) rather than more edges.

## Findings

### 1. Blocking — (B)'s head narrowing ignores that `trait_impls_of` lowers the self type of every local impl. Confidence: high on the mechanism, about 75% that `u8 + 1` reaches it. Probe pending.

Evidence: `recovery.rs:190-201` (`head`) reads only the path's `Res` and ignores generic arguments. So `impl Add for W<dyn Missing>` gets `head = Some(W)`. Then `recovery.rs:405-410` adds only the edge `(W, Add)`.

The pinned `trait_impls_of_provider` (`rustc_middle/src/ty/trait_def.rs:328-331`) calls `tcx.type_of(impl)` for **every** local impl of the trait. It does this to build the simplified-type index, before any head filtering happens. `for_each_relevant_impl` (`trait_def.rs:145`) forces that query on every impl-candidate assembly. Lowering `W<dyn Missing>` reaches `lower_poly_trait_ref`'s `FatalError.raise()` through `dyn_trait.rs:67` and `hir_ty_lowering/mod.rs:973`.

Operator typeck always looks up the trait method (`rustc_hir_typeck/src/op.rs:233`, `lookup_op_method`), even for builtin primitive operands. So I expect `pub fn c(x: u8) -> u8 { x + 1 }` to abort in `typeck_body(c)`. Nothing marks `c` unavailable: it names neither `W` nor `Add`, and it has no method call or type-relative path. C has already flushed the FX0501 of `a`, but every diagnostic after `c` is lost. The same holds for `==` with `PartialEq`, and for any implicit trait whose impl has a bad self type.

(B)'s narrowing to the head is only sound when the impl's **predicates** are bad (bounds, where-clauses, associated-item signatures), since those are evaluated only for candidates whose head matches. It is not sound when the self type is bad. The control `add-pred-control` (a bad predicate only, head `W`) should pass, which isolates the cause.

Fix I'd want: see finding 4. If the closure is kept, `head` must return `None` whenever the impl's HIR self type contains an unresolved trait reference. That covers `dyn`, `impl` and nested arguments, and it makes every body depend on the trait. Add a regression with an operator on a primitive beside `impl Add for W<dyn Missing>`, with a sentinel cast and Result after it.

### 2. Blocking — inherent impls whose self type contains an unresolved trait poison every inherent method lookup, and no edge models this. Confidence: about 80%. Probe pending.

Evidence: `crate_inherent_impls` (`rustc_hir_analysis/src/coherence/inherent_impls.rs:22-30`, with `check_item` at line 175) calls `type_of` on **every** inherent impl in the crate. Two lookups force it:

- method probing on a primitive receiver, through `incoherent_impls`. The local crate is always consulted, at `rustc_middle/src/ty/trait_def.rs:346-357`, `inherent_impls.rs:43-48` and `rustc_hir_typeck/src/method/probe.rs:958`;
- probing on a local ADT, through `inherent_impls(def)` at `probe.rs:964`.

Type-relative paths probe the same way, and so does the walk itself, at `walk/authority.rs:417` for local structs and enums that are not untyped.

In recovery, `impl dyn Missing {}` makes only the impl bad (`visit_trait_ref`). No trait is tainted, `head` is `None`, and the receiver-type edge (`recovery.rs:226-232`) never fires. So `pub fn c(x: i32) -> i32 { x.abs() }` is available and its typeck should abort. `impl W<dyn Missing> {}` behaves the same: the head edge marks `W`, but `c` never names `W`.

Fix I'd want: finding 4. Within the closure model, any inherent impl whose self type is bad must make every body with a method call or type-relative path unavailable, along with the walk's `inherent_impls` callers. Add a regression with `impl dyn Missing {}` and an independent `x.abs()` body, with sentinels.

### 3. Blocking — the walk's own `adt_destructor` forces coherence of `Drop` over every local `Drop` impl, for any local ADT. Confidence: about 65%. Probe pending.

Evidence: `use_generic` calls `local_drop_impl` (`walk.rs:615`), which calls `tcx.adt_destructor(adt)` (`walk.rs:1571`). In the pin, `calculate_dtor` (`rustc_middle/src/ty/util.rs:382-387`) first runs `coherent_trait(Drop)`. That calls `impl_trait_header` for every local `Drop` impl (`coherence/mod.rs:175-176`). It then calls `type_of` on each `Drop` impl to find the one for this ADT.

With `impl Drop for W<dyn Missing>`, (B) adds only `(W, Drop)`. An unrelated body `pub fn c() { let _ = T; }` with a local `struct T` passes `can_select`, and its walk should abort *after* `typeck_body(c)`. C flushed before typeck, so `c`'s own diagnostics and everything after it are lost.

This shows that C limits the loss only to "since the last typeck". The walk's own queries after typeck are not covered. Fixing finding 1 by treating a bad self type as headless also closes this case. Finding 4 closes it without depending on that.

### 4. Blocking (mechanism) — replace the reference closure with one catch of rustc's fatal error per body, on crates that already have rustc errors. Confidence that this design is sound: medium-high. It needs a spike, see below.

Findings 1–3, and finding 5 of the earlier re-check, are all the same defect class: the preflight has to predict which rustc queries a body will reach. rustc has crate-global indexes (`trait_impls_of`, `crate_inherent_impls`, `coherent_trait`, `specialization_graph_of`), diagnostic paths (`all_traits`) and lang-item machinery, so that prediction cannot be made complete with a 440-line model. It is also too coarse where it does fire: see finding 5.

The author rejected catching the unwind because of poisoned queries. The pin answers that objection. When a query unwinds, its key is marked `Poisoned` (`rustc_query_impl/src/execution.rs:183-200`). Any later access to that key raises `FatalError` again (`execution.rs:364`; also `:267` for waiters). That is the same catchable `FatalErrorMarker` unwind, not an ICE. The `panic!()` at `execution.rs:206` fires only when a job completes for a key that is already poisoned, which ordinary single-threaded use never does.

**Contract I would want** (owner: `walk.rs`, `visit_nested_body`, plus the item-level visit for queries outside bodies):

- Consumer: a writer whose crate fails rustc trait resolution.
- Trigger: `tcx.dcx().has_errors()` is set, and a `FatalErrorMarker` unwind is raised while Tide calls `typeck_body(B)` or walks body B, or while it queries an item's signature, fields or impl header.
- Result:
  - The unwind is caught at that body or item boundary with `catch_unwind(AssertUnwindSafe(..))`.
  - The walk's saved state (`typeck`, `owner`, `item`, `recovering`, and any other per-visit stack field) is restored, because the visitor's restore code did not run.
  - Diagnostics already queued for B are kept. They were computed from known types.
  - B is then revisited once in HIR-only `recovering` mode, the existing mode, under the same catch. Unresolved-path FX0501 in B stays with `authority.rs`.
  - The walk continues with the next body.
  - Later bodies that touch a poisoned query raise again and are caught the same way. Independent bodies keep every typed check, including those lost in finding 5.
- Failures:
  - Any other panic payload resumes unwinding, so real ICEs stay visible.
  - With no rustc error, nothing is caught and the clean-crate path is byte-for-byte the current one.
  - The crate stays rejected, because rustc's error count is non-zero.
  - Holes are not serialized once a catch has fired.
- Falsifiers: the existing 33 driver tests. All Codex probes and this file's probes, each with sentinels *after* the hazard. A clean-crate comparison against the full corpus.

What this deletes or keeps:

- It deletes `recovery.rs`'s closure, B's implicit-trait list, the all-traits edges and the `untyped` set.
- C becomes redundant, since nothing is lost at an unwind. Keep it only if the coordinator wants defense in depth.
- It keeps the round-1 taint-gate work for findings 3 and 4 (`recovered_typeck_of`, the guarded cast branch, `can_select` on tainted results), which is independent.

Risks to check in the spike:

- No Tide lock or `RefCell` borrow may live across the catch boundary. The labels mutex is taken only briefly, and `Diagnostics` is a plain Vec.
- rustc's own analysis later re-hits a poisoned key and aborts with fatal. That happens after Tide's emit, as it does today.
- The pinned nightly runs the single-threaded query engine by default.

If the spike shows the catch is unsound on the pin, keep A/B/C and apply the narrower fixes in findings 1–3. Then record that the class remains open.

### 5. Non-blocking — precision: one tainted std trait disables typed checks in every body with a method call. Confidence: about 80%. Probe pending.

Evidence: `method_scope` (`recovery.rs:41-52`) makes every method-call and type-relative body depend on every trait, including std's, through `all_traits_including_private`. Rustc reaches that path only when building diagnostics for a method it could not find. `#[derive(Debug)] pub struct S<T: Missing>(T);` gives a derived `impl<T: Debug + Missing> Debug for S<T>`, which taints `Debug`. So an unrelated `fn e(x: u16) -> u8 { let _ = "a".len(); x as u8 }` loses its FX0301. This is under-reporting, not invention, and the crate is rejected. But it works against the contract the coordinator set on 01:07Z, to keep diagnostics on other items and lines, in a very common shape (a derive on a struct with a misspelled bound). Finding 4 removes it.

### 6. Non-blocking — clean-crate behavior relies on reasoning plus three controls. Confidence that output is unchanged: high. Needs the full suites.

`visit_expr` (`walk.rs:1944-1970`) now runs `lint_sink_expr`, `fn_pointer_address` and `use_ty` before `ceiling_call` and `use_generic`. On untainted results the same calls run with the same `e.span`. Final emission sorts and deduplicates by (lo, code), so I see no output change. `seen_pure` and `seen_obligations` record the first span, but every reordered call uses the same `e.span`.

`authority.rs` gained short-circuits for local definitions that are not gated on errors (`authority_use`, `root_name`, `denied_key`, `rust_counterpart`). They change behavior only when the admitted crate itself is named `tide_std`, `std` or `tokio`, because `Role::Local` keys are never remapped (`crates.rs:87-90,153-167`).

Fix: none. The coordinator's planned `cargo test --workspace` and `conformance/run.sh` on the rebased tree are the proof. Note both points in the landing record.

### 7. Nit — C changes FX ordering and deduplication on every crate with a rustc error, not only on crates with unresolved traits.

The flush runs before each typeck once `has_rustc_errors()` is set (`walk.rs:1874-1885`). So on an ordinary E0308 crate, FX output now interleaves with rustc's errors instead of all coming at the end. Across a flush boundary, the (lo, code) duplicate that wins can be the earlier-flushed one rather than the one with the smallest `hi` or message. Codes, lines and severities are unchanged, and output order is not part of the contract. Finding 4 would make C unnecessary.

### 8. Non-blocking — the tests are too weak for the risk that remains.

The 33 driver tests and the 88 replayed probes vary the *consumer*: coercions, constructors, anonymous constants, aliases. None varies where the badness sits in the *impl*: a self type rather than a predicate, an inherent impl rather than a trait impl, or `Drop`. Whichever mechanism lands, add regressions for findings 1–3 with sentinel FX0301 and FX0702 lines after the hazard.

## Checked and fine

- **Invented diagnostics:** none found.
  - HIR-only recovery in `recovering` bodies reads the `Res` of written paths. Its inherent-name lookup for known external self types (`authority.rs` around 313-345, 413-417) matches rustc's rule that inherent items take priority.
  - Tainted results run only checks over node types known not to contain errors. Casts (`surface.rs:168-172`) need both types known and keep enum and fn-item casts guarded. `use_ty` filters `references_error()`.
  - The emitter path is unchanged, so codes and severities are unchanged. Rejection is preserved.
- **Clean crates:** `untyped_items` returns empty without walking when `has_errors()` is unset. A crate with only Tide FX errors runs the HIR preflight but gets an empty set. The flush needs a rustc error, recognized as not starting with a code from `Code::ALL` (`labels.rs:190-201`). `can_select` is true on untainted results.
- **Ownership:** unresolved-path recovery stays in `authority.rs`. `recovery.rs` holds no FX rule table, only availability bookkeeping.
- **Complexity:** about 440 lines of `recovery.rs` model rustc's query graph, and findings 1–3 show that model is still incomplete. A design that cannot be checked against rustc's own dependency structure costs every later reader. Finding 4 is the proportionate alternative.

## Probes

Not yet run (quiet window). Staged in `/home/lucca/.cache/eon-agents/tide-fx-u65.7-work/opus-r2-probes/`, one `package/lib.rs` and one `expect.txt` per case. The runner is `run.py`, copied unchanged from `codex-r2-recheck-probes/`. For each case it requires:

- error FX0301 exactly once on the expect line and FX0702 on the next line;
- FX0501 present when the case calls `std::fs::read`;
- exit status 1;
- no panic or ICE.

It uses the reused candidate binary (SHA-256 above), with stdin closed and a 20-second timeout per case.

| Case | Tests | Expected on the candidate |
| --- | --- | --- |
| `add-selfty` | Finding 1: `impl core::ops::Add for W<dyn Missing>` plus an independent `x + 1` on `u8` | fails: FX0301 and FX0702 lost |
| `eq-selfty` | Finding 1: `impl PartialEq for W<dyn Missing>` plus `x == 1` | fails |
| `add-pred-control` | Finding 1 control: bad predicate only, head `W`, plus `x + 1` | passes |
| `inherent-dyn` | Finding 2: `impl dyn Missing {}` plus `x.abs()` on `i32` | fails |
| `inherent-adt-arg` | Finding 2: `impl W<dyn Missing> {}` plus `x.abs()` | fails |
| `inherent-dyn-nomethod-control` | Finding 2 control: same impl, a body with no method call | passes |
| `drop-selfty` | Finding 3: `impl Drop for W<dyn Missing>` plus `let _ = T;` for an unrelated local unit struct | fails |
| `derive-precision` | Finding 5: `#[derive(Debug)]` on `S<T: Missing>`, sentinel bodies that make a method call | fails (precision) |
| `derive-precision-control` | Finding 5 control: same struct without the derive | passes |

Command to run after the release, as one queued job:

```sh
(ulimit -c 0; CARGO_BUILD_JOBS=4 timeout 600 bash /home/lucca/.cache/eon-agents/eon-overseer/heavy.sh nice -n 19 python3 /home/lucca/.cache/eon-agents/tide-fx-u65.7-work/opus-r2-probes/run.py </dev/null)
```

If findings 1–2 are confirmed, I will also add a spike for finding 4 when the coordinator asks for one. It was not run in this review.

No candidate edits, staging, commits or br writes were made, and no held-out files were accessed. The probe inputs were written only under the work dir.

DONE

## Round 3 re-check

Model: claude-opus-5-5 · Date: 2026-11-13

Verdict: approve

Scope: only my findings 1–4, against candidate `c6d5fdf0d8793c7a93db7635c703b5e8e2a9332f`, which is one commit on `71127b1`. I read `git diff 9ccb2dc..c6d5fdf` (walk.rs, authority.rs, driver.rs; recovery.rs deleted; diag.rs and labels.rs restored) and the report's "## Fix round 3". I also checked the pinned `rustc_span/src/fatal_error.rs:33-41`.

**Finding 4 (mechanism) — resolved.** `walk.rs:200-242` (`visit_boundary`) is the contract I asked for:

- It wraps every item, impl item, trait item and foreign item visit (`walk.rs:1795-1893`), every body visit (`walk.rs:1917-1935`) and the post-walk entry-point check (`walk.rs:93`).
- It uses rustc's own `catch_fatal_errors`. That helper catches only a `FatalErrorMarker` and resumes every other payload, so real ICEs stay visible.
- It catches only when `dcx().has_errors()` is set. Otherwise it calls the visit directly, so the clean-crate path is unchanged.
- On a catch it:
  - restores `typeck`, `owner`, `item`, `recovering` and `item_env`;
  - restores `seen_pure`, which is the only other state taken and returned mid-visit (`walk.rs:1384-1399`);
  - keeps the diagnostics already queued;
  - retries the visit once in the existing HIR-only mode, under the same catch;
  - sets `caught_fatal`, which stops trust-hole serialization (`walk.rs:102`).
- Nested independent items start a fresh typed attempt.

The rest of the round:

- `recovery.rs`, the `untyped` set and the C flush are gone. `diag.rs` and `labels.rs` are byte-identical to base, so findings 7 and 5 disappear with them.
- Unresolved-path FX0501 recovery stays in `authority.rs`. Its two former `untyped` guards are now `recovering && def.is_local()` (`authority.rs:345, 411`).
- No FX rule table was added.

**Findings 1, 2, 3 — resolved, confirmed by probe.**

- The author's baseline run on the r2 production code is preserved as `round3-opus-baseline-results.json`. It lost both later sentinels in exactly the six cases I predicted: `add-selfty`, `eq-selfty`, `inherent-dyn`, `inherent-adt-arg`, `drop-selfty` and `derive-precision`. All three controls passed. So each finding was real.
- My rerun on `c6d5fdf` passes 9 of 9. In every case:
  - FX0301 and FX0702 each appear exactly once on their sentinel lines, which come after the hazard;
  - FX0501 is present on line 1;
  - E0405 rejects the crate, with exit status 1;
  - there is no panic or ICE.
- No other FX code appears. The FX0520 on line 1 beside FX0501 also appears in the earlier rounds' probes and comes from running without Tide's std.
- The new driver test `fatal_global_queries_keep_later_walk_diagnostics` covers all three hazards and the derive case with sentinels after the hazard. For Drop, it also checks diagnostics queued in the same body before the fatal.

**Remaining findings:** none blocking.

- Nit, no action needed: `ceiling_cycles`, `check_expansions` and the `trusted_pure` loop (`walk.rs:92-100`) still run outside the boundary. They read recorded call edges and attributes, and I found no query that lowers a trait in them. A fatal there would lose the buffered diagnostics, as it would have before this bead.
- My finding 6 (full workspace suite and full conformance corpus on the rebased tree) remains the coordinator's landing proof. It is outside this re-check.

Probe run:

- Binary: `target/debug/tide-admit`, SHA-256 `71adc9fe21cb0b8f0e60a983c5ca179bea8a55967776442aa92064674243116d`. This is the author's final driver hash. The worktree is at HEAD `c6d5fdf` and clean.
- I waited until the 1-minute load was below 12; it was 11.17 at start. The quiet-until marker read 18:13:00Z, which had already passed.
- Command:

  ```sh
  (ulimit -c 0; bash /home/lucca/.cache/eon-agents/eon-overseer/heavy.sh env CARGO_BUILD_JOBS=4 timeout 300 nice -n 19 python3 /home/lucca/.cache/eon-agents/tide-fx-u65.7-work/opus-r2-probes/run.py </dev/null)
  ```

- The run went 18:40:10Z–18:40:12Z and exited 0 with `TOTAL 9 FAILED 0`. Results and per-case stderr are in `opus-r2-probes/`; `results.json` was overwritten by this run.
- There was one run, no build, and nothing is left queued.

No candidate edits, staging, commits or br writes were made, and no held-out files were accessed.

DONE

## Round 4 re-check

Model: claude-opus-5-5 · Date: 2026-11-13

Verdict: request_changes

Scope: a converge round over `git diff 7961b3e..13eba5b` only (walk.rs and tests/driver.rs). The candidate is `13eba5bcaf6211dd7e04b377010402e8e5ecfd93`, one commit on `b11438e`. The worktree is clean. I read the report's "## Fix round 4" and checked the pinned rustc-src for the capture-map change.

**The fix is correct for what it targets.**

- `use_generic` (`walk.rs:640-666`) handles partial results with a strict boundary:
  - Only Pure obligations run.
  - Argument count must equal `generics_of(def_id).count()`, which rules out an out-of-range `instantiate` ICE. When `node_args` is missing it returns empty args, which also fail this check.
  - Admission selection (`obligation`), `named` and Drop lookup stay behind the complete-result gate.
  - Every `def_id` that reaches this path (ADTs, fn items and constructors, method callees, value paths with recorded args) has `generics_of`.
- Pure inputs are validated before any normalization or selection: error, inference and escaping-bound-variable flags are checked in `pure_ty` (`walk.rs:814-820`), `pure_obligation` (`:844`), the parallel `Fn` bound (`:977-980`), each capture place (`:937-942`) and `io_error_payload` (`:1042`). So unknown types cannot produce a verdict. Rustc's writeback turns unresolved inference variables into error types, so partial args that pass these checks are types rustc actually recorded.
- Captures now come from `tcx.typeck(local).closure_min_captures_flattened(local)` (`walk.rs:931`) instead of `closure_captures`. In the pin, `closure_typeinfo` (`rustc_middle/src/ty/closure.rs:201-209`) indexes `user_provided_sigs[&def]`, which panics when error recovery left no entry. It builds its captures from that same `closure_min_captures_flattened` iterator, which returns nothing for a missing entry (`typeck_results.rs:540-549`). So the change removes an ICE path while keeping clean-crate capture order and output identical.
- Fatal queries reachable from the new paths (`predicates_of` of a callee with a bad bound, normalization in the Pure environment) are still bounded by round 3's per-body catch.
- I found no ICE path and no over-report in this diff.

**1. Blocking — the fix restores only call-site Pure obligations. Other Pure checks are still skipped in any body with a rustc error, and that is a regression against `main`.** Confirmed by probe.

The cause the coordinator named is round 1's whole-body taint gate. Round 4 lifted it only for `use_generic` at the three call sites. These checks are still behind it:

- `visit_expr` returns early when the results are tainted (`walk.rs:2029-2036`). That skips `raw_place` (`:2043`), and with it ROOT-4's `forged` FX0517 for raw-pointer derefs and union-field reads. It also skips the adjustment loop's `use_ty`/`follow` checks on coercion targets.
- `visit_pat` still reads only complete results (`walk.rs:2079`, `typeck_of`). It therefore skips `use_ty` on pattern types and the union-pattern `forged` check.
- `named` is still gated on complete results (`walk.rs:1983`, `:2002`). Its only effect is Pure obligations, through `use_generic(admit = false)`, which is exactly what this round made safe on partial results.

On base `71127b1`, `typeck_of` did not filter tainted results, and `visit_expr` called `raw_place` unconditionally (base `walk.rs:408-410`, `:1888`). So these diagnostics are produced on `main` and lost by this bead.

Probe `opus-r4-probes/forged_tainted.rs`: `deref_tainted` and `punned_tainted` each have an unrelated `let _: u8 = "x";` (E0308) before a raw read of `Fs`, and `deref_clean` is the same read with no error. The driver reported FX0517 only on line 22 (the clean body) and lost it on lines 8 and 18. That is the same class of loss as the four landing failures. The corpus has no ROOT-4 case with a rustc error in the same body, which is why the 336-PASS run did not catch it.

**Fix I'd want:** delete the taint early return in `visit_expr` (`walk.rs:2029-2036`), not add more per-call branches. Every typed call after it already guards itself:

- `ceiling_call` returns unless `can_select()` (`walk/traits.rs:177`);
- `use_generic` falls back to Pure-only with the arity check (`walk.rs:641-648`);
- `obligation` returns unless `can_select()` (`walk.rs:1434`), which also covers `follow`;
- `pure_obligation` validates its trait ref (`walk.rs:844`), which covers `forged`/`raw_place`;
- `raw_place` itself only reads recorded types.

Then:

- switch `visit_pat` to `recovered_typeck_of`;
- drop the complete-result gates on `named` at `walk.rs:1983` and `:2002`;
- add a regression with this probe's shape (raw deref, union field and union pattern, each in a body with an unrelated E0308) beside the existing call-site case.

**2. Non-blocking — leftovers.** None in this diff. The doc comment on `recovered_typeck_of` (`walk.rs:469`) matches its new role. No dead code or orphaned helper remains. All five corpus cases are unchanged.

Probe run:

- Probe file: `/home/lucca/.cache/eon-agents/tide-fx-u65.7-work/opus-r4-probes/forged_tainted.rs`. It is a corpus-format case outside `cases/`, with `rules: ROOT-4` and `grants: unsafe`.
- Load was 2.88 and the quiet-until marker had passed. The run went 20:07:32Z–20:07:34Z with no rebuild (HEAD `13eba5b`).
- Command:

  ```sh
  bash /home/lucca/.cache/eon-agents/eon-overseer/heavy.sh env CARGO_BUILD_JOBS=4 timeout 600 bash conformance/run.sh /home/lucca/.cache/eon-agents/tide-fx-u65.7-work/opus-r4-probes/forged_tainted.rs </dev/null
  ```

- Result: exit 1, `FAIL … driver diagnostics [22:error:FX0517], marked [18:error:FX0517 22:error:FX0517 8:error:FX0517]`. E0308 was reported on lines 7 and 17 as marked.
- I did not rerun the full corpus or the driver tests. The author's 336-PASS run and 35-pass driver run are cited, not reproduced.

No candidate edits, staging, commits or br writes were made, and no held-out files were accessed. Nothing is left queued.

DONE

## Round 5 re-check

Model: claude-opus-5-5 · Date: 2026-11-13

Verdict: approve

Scope: only round-4 finding 1, against `git diff 13eba5b..acce435`. The candidate is `acce43562b1609c437dc6e93f1cf2e035823de52`, one commit on `b11438e`, and the worktree is clean.

**Finding 1 — resolved.** The diff makes exactly the fix I asked for:

- `visit_expr`'s early return for tainted results is deleted (`walk.rs:2022-2030` now flows straight on). Every typed call after it guards itself on partial results:
  - `ceiling_call` returns unless `can_select()` (`walk/traits.rs:177`);
  - `use_generic` runs only Pure obligations, after the exact-arity check, with Drop lookup and admission still behind the complete-result gate (`walk.rs:640-666`);
  - `obligation`, and therefore `follow`'s admission part, returns unless `can_select()` (`walk.rs:1434`);
  - `pure_obligation` rejects trait refs with errors, inference variables or escaping bound variables (`walk.rs:844`), and `forged`, `raw_place` and `follow`'s Pure part all go through it;
  - `raw_place` and `unsized_trait_ref` (`walk.rs:1593`) only read recorded types.
- `visit_pat` uses `recovered_typeck_of` (`walk.rs:2067`), so union-pattern `forged` checks run again.
- Both `named` gates are removed (`walk.rs:1983`, `:2000`). `named` reaches only `use_generic(admit = false)`, which is the guarded Pure-only path.
- The diff adds no new query path, rule or state.

The new driver test `partial_typeck_keeps_raw_read_pure_obligations` (`tests/driver.rs:1874`) covers raw deref, union field and union pattern, each beside an unrelated E0308. It requires FX0517 exactly once on each raw-read line, and no panic or ICE.

Probe run:

- HEAD was `acce435` and the driver hash was `36c2aafb…a51722`, which matches the author's.
- Load was 5.16. The quiet-until marker read 20:40:37Z, which had already passed. The run went 20:40:46Z–20:40:58Z.
- Command:

  ```sh
  bash /home/lucca/.cache/eon-agents/eon-overseer/heavy.sh env CARGO_BUILD_JOBS=4 timeout 600 bash conformance/run.sh /home/lucca/.cache/eon-agents/tide-fx-u65.7-work/opus-r4-probes/forged_tainted.rs </dev/null
  ```

- Result: `PASS`, exit 0. FX0517 on lines 8 and 18 (the bodies with E0308) is restored beside line 22's clean control, with the E0308 marks on lines 7 and 17.
- One run only. The author's driver tests (36 passed), probe sets and full-corpus run (336 PASS, 0 FAIL) are cited, not reproduced.

No candidate edits, staging, commits or br writes were made, and no held-out files were accessed. Nothing is left queued.

DONE

## Rebase re-check

Model: claude-opus-5-5 · Date: 2026-11-13

Verdict: approve

Scope: only what the rebase changed. Candidate `4c3d5af` (tree `6422d0e7`), one commit on `4dadc7d`, which contains fx-u65.6 (`55f3ecb`). The worktree is clean. I read `git range-diff 55f3ecb~1..acce435 4dadc7d..4c3d5af`, the report's "## Rebase onto 55f3ecb", `walk/contracts.rs` and `walk/modules.rs` in full, and fx-u65.6's own diff to `walk.rs` and `invocation.rs`. Read-only; no builds or probes.

**Conflict resolutions — correct.**

- `walk.rs:98`: `module_rules` sits after `ceiling_cycles` and before the `entry_point` boundary, as on `main`. It runs outside any boundary, but its only typed queries (`used_trait_imports` over every body) are behind `err_count() == surface_errors` (`modules.rs:96`). An unresolved trait path always emits E0405, so that branch never runs in the crates this bead targets. The rest of it is HIR and resolver data.
- `walk.rs:1880`, `:1898`: `contract_placement` moved inside each impl-item and trait-item boundary closure, at the same position as on `main` (after `declared`/`fn_item`, before `unsafe_attrs`). `contract_placement` reads only `def_kind`, `local_parent` and expansion data, so it is safe in HIR-only retries.
- `visit_fn` → `contract_function` (`walk.rs:1939`) and `visit_block` → `safety_comment` merged without conflict. Both are reached only from inside an item or body boundary.
- `tests/driver.rs:1313`: the three new module/import tests are kept ahead of this bead's regression tail. The range-diff shows only a hunk-anchor change there.

**New production guard in `walk/contracts.rs` — correct and minimal.**

- `:93`: in recovering mode, the macro-callee scan skips `Crates::key` for local callees. `key` runs `def_path_str`. For a fn nested in an unresolved impl, that prints the impl's path and lowers its trait, which re-raises FatalError inside the HIR-only retry. The rule matches only `tide_std::__contract::*` helpers, which are never local in a user crate. So the skip cannot drop an FX0802 there, and the clean-crate path is unchanged. Unmatched local calls still fall through to `walk_expr`, so nested contract calls are still found. This is the same `recovering && is_local()` pattern round 3 adopted in `authority.rs`.
- `:148`: the `fn_sig`/`typeck`/`clauses_of`/outlives analysis for FX0803 returns in recovering mode. The HIR clause scan runs first, so FX0802 placement findings survive. In recovering mode, losing FX0803 matches every other typed check in a body that has already failed its typed attempt. The crate is rejected anyway, and the rejected alternative (skip the whole scan) would have lost FX0802.
- Neither guard adds a query path, rule or state. A retry cannot double-report FX0902 or FX0101/0103, because `diag.rs:317` dedups by `(span.lo, code)`.

**Evidence the guard is needed, not speculative.** The author's merged tree `15f9ef3`, without the guard, lost FX0501 in the macro-recovery probe, and the same probe passes on `4c3d5af` (report; logs `rebase-55f3ecb-final-checks.log` and `rebase-55f3ecb-macro-recovery.log`, cited and not reproduced). The regression `contract_scan_keeps_recovery_paths_inside_an_unresolved_impl` (`tests/driver.rs:1910`) has that shape. It requires exactly one FX0101 at 5, FX0103 at 6 and FX0501 at 7, plus E0405 and no panic or ICE.

**Expectation changes — legitimate.** None changes this bead's production diagnostics:

- `forged_tainted` gains FX0902 at lines 8, 18 and 22 because its unsafe blocks have no `// SAFETY:` comment, which is fx-u65.6's rule. FX0517 is still present on all three lines, and the original source was left unchanged. The new copy adds only marks.
- The `future-glob` probe record now carries fx-u65.6's FX0913 on an unused glob import.
- No in-repo test expectation was weakened. Besides the placement moves, the range-diff touches `driver.rs` only to add the new test.

**Findings:** none.

- Nit, no action needed: when Tide's std is checked itself with rustc errors, `:93` would skip its own local `__contract` helpers in the HIR-only retry. Such a crate is already rejected.

Proof is cited, not reproduced: driver 40 passed; opus-r2 probes 9/9, codex-r2 probes 36/36; fix-round3 probes 87 unchanged plus the explained FX0913 record; full corpus 361 PASS / 4 AWAITS / 0 FAIL on `4c3d5af`.

No candidate edits, staging, commits or br writes were made, and no held-out files were accessed. Nothing is queued.

DONE
