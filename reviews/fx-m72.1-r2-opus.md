Model: claude-opus-5-5 · Effort: low · Date: 2026-11-11

# fx-m72.1 round-2 Opus re-check (read-only; only the fixes and rebuttals for round-1 findings 1, 2, 3, 4, 7, 8 and 9)

Candidate: worktree `.worktrees/fx-m72.1`, HEAD `2c4b844` on base `182a594`. `git diff 182a594 2c4b844 | sha256sum`
= `624ac26eb01e53797b94ec4c7bdba2be6d8dd5fb16c3bc2f6fe3751421936118`, which matches the brief, and the worktree is clean.
I read `fix-r1-report.md` and `fix-r2-report.md`, plus the code and tests named below. I ran no probes and no builds;
the suite results quoted in the brief are the author's proof. I did not open `trials/e1-checked/`.

## Verdict

`approve`

## Re-checked findings

### 1 (blocking): synthesized code could contain newlines that shift later lines. Resolved.

- `crates/tide-admit/src/synthesis.rs:42-43` flattens the pretty-printer's newlines before inserting the code.
- The inline case still inserts ` {code} ` before the module's closing brace (`:44-46`). The EOF case wraps the code
  in `\n…\n` after the last user line, so no user line moves.
- The property-parameter doc attribute (`:139-143`) is inserted inline, with no newline.
- The regression test `crates/tide-admit/tests/build.rs:897-918` falsifies the old behaviour:
  - The field type in an inline module is well over 78 columns.
  - The test requires the later mismatch at `demo.rs:6`.
  - It also requires that no `Arbitrary` error appears, which proves synthesis ran. Synthesis runs in
    `after_expansion`, before typeck (`crates/tide-admit/src/invocation.rs:74-80`), so the type error in `broken`
    does not suppress it.

### 2: an ungeneratable field gave an error inside the driver's inserted text. Resolved.

- `synthesis.rs:298-313` emits a driver diagnostic at `def_span(field)` for a concrete field type with no
  `Arbitrary` impl. It is reached from the non-local branch (`:266-268`), the local branch for unions, empty enums,
  types not declared in a module, and inapplicable user impls (`:271-277`), and every other unsupported shape
  (`:294`).
- `invocation.rs:76` aborts on those errors before any overlay is compiled.
- The visited key includes the field (`synthesis.rs:168`, `:222`), so an earlier direct parameter of the same type
  cannot hide the field's diagnostic.
- `build.rs:736-785` asserts the diagnostic for a `Box<u8>` field at `demo.rs:2:`, with a direct `Box<u8>`
  parameter discovered first, and for an unsized `[u8]` tail.

### 3: a local generic type with a user impl did not have its type arguments followed. Resolved.

- Discovery now follows the impl that selection actually picks, through `select` and `where_clauses` from
  `walk.rs` (`synthesis.rs:225-235`). Local and non-local ADTs are handled the same way.
- The round-2 change follows requirements on traits other than `Arbitrary` transitively (`:236-238` stops only
  after an unmet one). This covers the indirect `Leaf: Generate` → `Leaf: Arbitrary` chain.
- A requirement that is not `Arbitrary` and is unmet is left for rustc to diagnose.
- Auto traits are never followed (`walk.rs` `select` returns `None` for them).
- Exact cycles are cut by the visited set, keyed on the trait ref and the field. Growing chains are cut by the shared
  64-level limit (`synthesis.rs:203-209`).
- Tests:
  - `build.rs:478-487` runs local `Wrap<Leaf>` and requires the shrunk input `Wrap(Leaf(3))`
    (`build.rs:688`).
  - `build.rs:808-858` runs the paired-library `Wrap<T: Generate>` case. The author's red log shows E0277 before
    the fix.

### 4: a `Result` property printed `Error:` on every evaluation and gave no cause. Resolved.

- `synthesis.rs:130-137` counts the resolved `Result` layers after normalization, so aliases are included.
- `crates/tide-macros/src/test.rs:115-125` lowers each layer into a silent `match`. Each `Err` becomes
  `Err(format!("{:?}", e))`, and the binding has mixed-site hygiene, so a user constant named `error` cannot capture
  it (`build.rs:422-423`).
- Only the final success value goes through `Termination::report` (`crates/tide-std/src/test/runner.rs:11-17`).
- `runner.rs:91-93` prints `cause:` for the `Exit` kind.
- `build.rs:651-662` requires no `Error:` line, exactly one `cause:` line and the right message, both for a plain
  result and for a nested aliased one.

### 7: recursion marking missed an ADT reached only through a DFS cross edge. Resolved.

- `synthesis.rs:327-335` adds the `param: Arbitrary` bounds whenever a field is left out of the bounds for mentioning
  a recursive ADT. The condition `has_param() && mentions_recursive` exactly matches the exclusion at `:364`, so no
  field can be dropped without its parameter bounds.
- The fixture `A↔B, A→C→B` with generic `C` (`build.rs:515-520`, compiled and run at `:599`) exercises the cross
  edge. On the walk back from C, B's field `a` is already visited, so the back-edge check never marks C, and only
  the new clause supplies `T: Arbitrary`.

### 8: the harness test could not catch generators stuck at the simplest value. Resolved.

- `build.rs:543-556` adds one property per family that a simplest-only generator would satisfy. The families are
  enum variants, structs, `Option`, `Result`, `bool`, `char`, `String`, arrays, tuples, floats, `HashMap`, `HashSet`,
  `BTreeMap` and `BTreeSet`.
- `build.rs:689-706` requires each of them to fail, with a seed line.
- A misspelled name would not pass silently: `--exact` with no match exits successfully, and `failure()` asserts a
  non-success exit.

### 9: a parameterless `#[test(args)]` produced a malformed attribute. Resolved.

- `test.rs:6-11` rejects any attribute arguments before the parameterless branch at `:12-14`.
- `build.rs:759-764` requires the message at `demo.rs:1:`.

## What the fixes introduced

There is nothing blocking. Two non-blocking text points; neither needs a re-check:

- **Nit, wording.** For a nested type, the field diagnostic names the inner type. With `v: Vec<Box<u8>>`, the
  `Box<u8>` requirement comes from `Vec`'s impl and `cannot_generate` receives `Box<u8>`
  (`synthesis.rs:231-232` → `:298-310`). The message then reads "field `v` of `S` has type `Box<u8>`", although
  the field's type is `Vec<Box<u8>>`. Wording such as "contains `Box<u8>`" or "needs `Box<u8>`", or naming the
  declared field type, would be exact.
- **Negligible.** The newline flattening at `synthesis.rs:43` would also rewrite a literal newline inside a string
  literal, if the field type has a const-argument expression that holds a multi-line string. That is only
  reachable in a generic field type with such a const expression, and the worst case is a compile error. No action
  needed.

DONE
