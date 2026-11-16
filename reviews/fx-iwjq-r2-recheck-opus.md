Model: claude-opus-5-5 · Date: 2026-11-16 · Candidate f0d63d0

# fx-iwjq Opus r2 finder re-check (read-only)

I checked that `git diff 3f3d417...f0d63d0` has SHA-256 6ee2233a7bc659a3bf82b19d76ff0747249eab7cd3169dfa4bd275cc11445837. I re-checked only the fixes in `git diff f093d40 f0d63d0`, against the rustc sources of the pinned nightly-2026-08-09. I ran no cargo, rustc or conformance command.

## Verdict

approve

## Findings

**N1, the cost was understated: fixed.**
- The new cost wording appears in 02 §5.4, the rules.md CLOSURE-3 row, README.md and conformance/README.md. They now name four cases:
  - generic functions over references;
  - trait methods with a reference receiver (`Clone::clone`, and `Ord::cmp` in 02, with "`&self` implies `Self: 'a`");
  - `&self` and `&mut self` methods of types that have a lifetime parameter;
  - `fn f(v: &mut Vec<&str>)`.
- Each one matches what `implied_bound` does:
  - The trait item's `&'s Self` gives `RegionSubParam(Self)`.
  - `&'s Lexer<'a>` gives `'a: 's`.
  - `&'1 mut Vec<&'2 str>` gives `'2: '1`.

**N2, "rustc checks the bounds of a call" overclaimed: fixed.**
- reify.rs's `HELP` now says: "a call checks its argument types against the signature, though not the bounds an impl header implies". The module doc and 02 §5.4 now say:
  - a call checks argument types, so the bounds the signature implies hold;
  - a direct call does not check the bounds an impl header implies (the rust#100051 family), and the rule leaves that open.
- This is accurate. It is honest for every FX0206 path, including the "impl not known" message, which shares the same help.

**N3, the closure route was undocumented and unpinned: fixed.**
- The new 02 §5.4 sentence is accurate:
  - The closure takes its signature from the higher-ranked expected type.
  - Borrowck assumes that signature's implied bounds for the closure body, because `DefiningTy::Closure` inputs feed `free_region_relations::create`.
  - rustc's #129021 WF check covers only `ReifyFnPointer`.
  - The later fn-pointer subtyping checks nothing more.
  - Elision cannot share one bound origin across two parameters, so the type needs an explicit `for<'a, 'b>`, which FX0404 (LIFE-3) rejects.
- The rules.md row adds "the closure route stays closed through FX0404".
- The new case `closure_fnptr_closure_binder_reject.rs` has these properties:
  - It carries the rules `CLOSURE-3, LIFE-3` and a single `//~ FX0404` on line 9, where both binder parameters are.
  - It has no rustc mark. A PASS therefore also confirms that rustc accepts the closure exploit, which makes the case a real pin.
  - No FX0206 can fire on it: the closure goes through `ClosureFnPointer`, not a reify, and `f` to `g` is plain subtyping.
  - If FX0404 stopped rejecting `for<>` on fn-pointer types, the case would FAIL on the missing mark.
- One unproven point remains: whether the driver's two same-line FX0404 diagnostics match the single mark. They have identical messages and dedupe per line, as the builder's report notes. The coordinator's full conformance run settles it.

**N4, no case covered the alias branch: fixed.**
- What `bar<'a, 'c, T: Item>`'s identity signature implies:
  - `&'a <T as Item>::Out` stays a rigid projection under `bar`'s param env.
  - It has no escaping bound vars, so `compute_components` pushes only `Component::Alias` (rustc_type_ir/src/outlives.rs:155-163).
  - `implied_bounds_from_components` maps that to `RegionSubAlias` alone (implied_outlives_bounds.rs:214-217). No `RegionSubParam(T)` is produced.
  - The projection's own well-formedness adds only `T: Item`.
  - `v: T::Out` and `Box<dyn Debug + 'a>` imply nothing, `&'c ()` implies nothing, and `bar` has no impl parent.
- So the `alias` coercion is rejected only through the `RegionSubAlias` arm (reify.rs:151-153). If that arm returned `None`, the `//~ FX0206` on `alias` would go unmet and the case would FAIL.
- The exploit shape is right:
  - `'a` is late-bound, since it is constrained by `&'a …` outside the projection.
  - `T = Hold<'x>` normalizes `Out` to `&'x u32`.
  - The target `fn(_, &'x u32, &())` has a bound region, so rustc's #129021 check is skipped, as in the type-parameter route.

**N5, the "two different origins" text excluded `'static: 'a`: fixed.**
- 02 now says a bound holds trivially when:
  - the outliving side is `'static` or the origin itself;
  - the subject is a concrete type;
  - the subject is a type parameter declared `'static`.
- rules.md and conformance/README match. All three agree with `short != long && !long.is_static()` and `is_static(param)`.
- Nit, not blocking: "a concrete type such as `u32`" means a type with no origins. A concrete type that carries one, such as `Foo<'b>: 'a`, reduces to `'b: 'a` and is rejected. The wording predates this round.
- Nit, not blocking: conformance/README's "one origin outliving another other than `'static` or itself" reads awkwardly but is correct.

**N6, README line length: fixed.** The FX0206 paragraph in README.md is rewrapped. I left N6's other two items as stated.

## New blocking findings

None. The fixes change text, add two conformance cases, and change only the `HELP` string and module doc in reify.rs. No checker logic changed, so no rule outside the findings is affected. CLOSURE-3 still starts at 02:541, so the rules.md line reference holds.

DONE
