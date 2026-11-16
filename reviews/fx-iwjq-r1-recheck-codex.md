Model: gpt-6.1-sol (high) · Date: 2026-11-16

Verdict: approve

## Scope and evidence

Finder's narrow re-check of B1, B2, N1 and N2 from r1-codex.md, and every change in `git diff e87ffe0 f093d40`. Candidate: `f093d401748935bca55a1069f05ede5cd6154182`, branch `fx-iwjq`, based on main `3f3d417`. The main-to-candidate diff SHA-256 is `4031551d30e7ab85b5d28fd2001d47a5b9f4186e34939d004cd64d44a0feef56`, matching the brief. The worktree remained clean; `git diff --check e87ffe0 f093d40` passed.

Read AGENTS.md, GOVERNANCE.md, `br --no-db show fx-iwjq`, the prior review, the author's amended report, the complete incremental diff, and the surrounding selection, recovery and backstop code. Checked the relevant compiler implementation in the installed nightly-2026-08-09 source: trait association, assumed well-formed types, parameter environments, predicate elaboration, implied-outlives queries and instance resolution. This is source inspection, not execution proof. No repository files were changed; this requested report is the only file written.

## Findings re-checked

### B1 — Fixed: trait-method impl header

`crates/tide-admit/src/walk/reify.rs:28`, `:74`, `:103`.

The original trait method remains in `items`. Selection uses the actual function-item arguments, normalized in the caller's environment, to identify the impl. A selected override adds that impl's method; its signature and parent impl header are then inspected under identity parameters. Thus the original `<Pair<'static, 'static> as Select>::select` counterexample still exposes the symbolic header relation `'b: 'a` and rejects. An impl method whose signature adds an implied type-parameter bound is checked too, rather than relying only on the trait declaration's signature.

When an impl uses the trait's default method, the already-retained trait method supplies its signature, and the selected impl supplies its header. This is sufficient for that default-method case. Builtin selection has no user impl header to inspect. Local specialization is separately prohibited; this check is not a general replacement for rustc's specialization-aware instance resolver.

Failure is closed for selection: normalization failure, ambiguity, a parameter-bound implementation, object dispatch, and other results not represented by `Selected::Impl` or `Selected::Builtin` produce FX0206. A generic receiver can still select a unique blanket impl; that is correctly inspected rather than rejected merely for being generic. `can_select()` skips unreliable selection only during existing error recovery or tainted type checking; those crates are already rejected.

The new fixture protects both the concrete Pair route and the unresolved `S: Select` route, with a closure control. I found no bypass introduced by this fix through selecting an impl at all-static arguments: the arguments choose the impl but never replace the identity parameters used to inspect its bounds. Nor can the new static exemption suppress a lifetime-to-lifetime bound or an alias bound.

Assessment: sound and complete for the reported B1 defect and the added selection-failure policy. No remaining B1 blocker.

### B2 — Fixed: README safety qualification

`README.md:14`.

The authority claim now immediately states that protection against a malicious dependency rests on rustc's soundness, that open I-unsound issues remain, that FX0206 covers the function-pointer route of #25860, and that RUN-3 is pending. This supplies the qualification required by D7. It does not claim the sandbox has shipped.

Assessment: complete.

### N1 — Fixed: syntactic rule and callback cost

`design/02-memory.md:541`, `conformance/rules.md:58`, `README.md:67`, and `conformance/README.md:118`.

The text removes the claim that every rejected declaration has a breakable instantiation. It identifies the conservative declaration-based check, shows the cost for `first<T>(&[T])`, and gives the closure workaround. The bead's coordinator comment records the widening and its cost. The design and conformance README also describe the static exception and unknown-impl rejection.

Selection necessarily uses the actual receiver to choose an impl; the bound inspection of that chosen definition is what remains independent of its concrete substitution. That is the relevant distinction when reading the declaration-based wording.

Assessment: complete. The additional static exception is assessed separately below.

### N2 — Fixed: route fixtures and stale comment

`conformance/cases/memory/closure_fnptr_routes_reject.rs`, `closure_fnptr_trait_impl_reject.rs`, and `closure_fnptr_reject.rs:5`.

The compact route fixture covers explicit casts, distinct-function if/match common-type conversions, a constructor, and an inherent impl header. The trait fixture covers B1. The if-block diagnostic marks agree with the adjustment locations described by the implementation. The stale re-checking comment now describes conservative rejection.

These cover the distinct compiler paths requested in the finding. They do not add every possible spelling, an external bound-bearing negative case, or an alias-specific fixture; those were non-blocking coverage suggestions, not separate demonstrated defects. The existing driver fixture continues to exercise an external function's reification.

Assessment: sufficient and complete for this finding.

## Other changes since e87ffe0

### Driver fixture replacement — Sound and complete

`crates/tide-admit/tests/driver.rs:65`.

`std::cmp::max::<fn()>` has a by-value generic signature and supplies no implied reference/outlives bound for FX0206 to reject. Its pinned std implementation calls `Ord::max`, preserving the denied `Ord for fn()` dependency that the backstop must discover from the exported library root. The fixture still returns a reified external generic function and still requires FX0520 mentioning `for fn()` in both profiles. Replacing sort therefore preserves this test's contract while removing its now-intentional FX0206 conflict.

### Static type-parameter exemption — Sound; intentionally conservative recognition

`crates/tide-admit/src/walk/reify.rs:132`–`:150`; `conformance/cases/memory/closure_fnptr_accept.rs:8`.

An independently required `T: 'static` makes every implied `T: 'a` hold. Type arguments are fixed for the function item; converting that item to a pointer does not make T a newly quantified type parameter. Skipping this implied bound is therefore sound. The positive `owned::<u8>` control exercises the exception.

The implementation recognizes a `TypeOutlives(T, 'static)` clause in rustc's parameter environment, not just literal inline syntax:

- `T: 'static` inline and `where T: 'static` produce the same relevant clause. Inherited generic predicates are included.
- The pinned parameter-environment builder elaborates predicates. A bound such as `T: Static` with `trait Static: 'static` also supplies `T: 'static`. This is a valid exemption: satisfying the declared trait bound requires its static supertrait. Explicit compound outlives bounds can likewise elaborate to the same clause. The implementation is broader than a literal scan of the item's own where-clause, but the extra recognized cases are sound.
- Bounds inferred only from assuming the function's signature or impl header well formed are obtained later by `implied_bounds_tys`; they are not inserted into this parameter environment by that query. For example, `fn implied<T>(_: &'static T)` without a declared static requirement still yields an unexempted implied `T: 'static` and rejects. Likewise, merely mentioning `&'static T` inside a where-clause type is not itself a declared `T: 'static`: rustc's elaborator does not generally elaborate well-formedness predicates.
- The lookup does not solve every logically equivalent static requirement, such as an arbitrary transitive chain of lifetime bounds. Missing such a proof causes conservative rejection, not admission. Associated types receive no static exemption.

The comparison is against the exact identity parameter of the definition currently being inspected. Trait and impl parameter indices are not mixed between separate inspections. A sufficient static bound for one parameter cannot exempt another, and skipping one bound does not stop the iterator from finding another offending bound.

Assessment: no soundness defect found in the exemption. It is sufficient for the documented direct static case, with safe elaborated extensions and conservative false rejections elsewhere. Where-clause, supertrait and implied-only examples above are inspection-derived, not executed tests.

## New blocking findings

None introduced by the reviewed fixes.

## Reasoning and limits

The repair adds the missing bound owner without replacing the original signature check. It separates actual-argument impl selection from identity-parameter bound inspection, and rejects when selection cannot identify what to inspect. The exception relies on a separate required static predicate, rather than circularly trusting the implied bound being screened. These mechanisms address the reported defects without relaxing lifetime or alias screening.

The prior report's permissive failure behavior inside rustc's implied-outlives query remains unchanged: the helper can return no bounds on query failure. This re-check found no new concrete valid program triggering that failure. Approval is for the narrow reviewed changes; it is not a proof against all rustc soundness defects or a claim that the coordinator's full-suite acceptance has passed. Direct-call/Fn-trait routes were not reopened.

Not checked: cargo, rustc probes, conformance, runtime examples, or the coordinator's full-suite results, as instructed. Risk: execution proof and the subsequent Opus review remain required before landing.

DONE
