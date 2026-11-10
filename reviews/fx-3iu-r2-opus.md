Model: claude-opus-5-5 · Effort: low · Date: 2026-11-09

# fx-3iu round 2: Opus narrow re-check

Candidate: `dffb442` on branch fx-3iu, reviewed as `git diff cadea79 fx-3iu`.
The committed content of the six changed files matches the author's proven
digest `0d9ef365…4bc5`; I recomputed it from `git show fx-3iu:<path>`. So the
reported final `cargo test --workspace` (79 passed) and `conformance/run.sh`
(198 PASS, 0 FAIL) results apply to this commit. Those runs are the author's
claims; I did not repeat them.

Scope: only my taken findings N1, N3, N4, N5, N6 and N7, plus the changed lines
that implement them. I compared against the cached thiserror-impl 2.0.21 source
(`fmt.rs::expand_shorthand`, `expand.rs::type_parameter_of_option`). I ran no
probes, because static reading and the author's executed mutation settle each
item.

## Verdict

**approve**

## Per-finding status

**N1: resolved.** `crates/tide-admit/tests/build.rs:631`
`generic_error_patterns_keep_distinct_type_arguments`
- The test uses `Bounded<P: Pure, T>` with `P = u8` and `T = Net`.
- Its only `Self::Value {..}` patterns are the derive's, inside
  `impl<P: Pure, T>`. Typeck gives the node identity args `[P, T]`.
- If `path_args` returned any arguments that put a non-Pure type in `P`'s slot,
  it would raise FX0517 there. The swap mutation is one example.
- No other site rejects: the user constructor is a resolved path and is valid.
- The author's executed swap mutation failed with exactly one FX0517 at the
  derive (report item 6), and the normal driver accepts and runs the program.
  This is the falsifiable test I asked for.
- The old test is honestly renamed `generic_error_types_still_require_pure`
  (`:613`) and simplified to a constructor rejection, which is what it shows.

**N3: resolved.**
- Positional ambiguity, `crates/tide-macros/src/error.rs:337-350`: any numeric
  `{N}` in a tuple container with an extra unnamed argument is now rejected.
  This matches thiserror's `extra_positional_arguments_allowed` /
  `first_unnamed` rule. The negative is at `build.rs:540` (`"{} {1}", .0, 7`).
  One residual difference is listed under nits below.
- Option arity, `error.rs:233-246`: an `Option` source now needs exactly one
  type argument, as in `type_parameter_of_option`. The negative at `build.rs:558`
  uses a two-parameter alias, so it reaches source detection rather than rustc's
  arity error.
- Empty-enum attribute, `error.rs:419`: `message(&input.attrs)` now runs once,
  before any variant is handled. The negatives at `build.rs:516-539` check the
  line.

**N4: resolved.** `README.md:162-164` and `crates/tide-std/src/lib.rs:17-19`
- Both exclude `error(fmt = ..)`, backtrace detection and `#[backtrace]`, and
  `Error::provide`.
- Both label the difference for mixed transparent and unannotated variants with
  a manual Display.
- The two texts agree.

**N5: resolved.**
- `build.rs:26-43` and `crates/tide-std/tests/entry.rs:38-62` use `env_clear()`.
  They keep only `HOME` and `CARGO_HOME`, and set a fixed `PATH`, `RUSTC` and
  `CARGO_BUILD_JOBS`.
- Both use `--frozen`.
- Each computes one `DLL_PREFIX`/`DLL_SUFFIX` file name and uses it for both
  `--emit=link=` and `--extern`.
- In build.rs, `.env("RUSTC", ..)` comes after `env_clear()`, so it survives.
- The fixed `PATH=/usr/bin:/bin` is no regression: base `07e104b` already
  hard-coded `-Clinker=/usr/bin/cc`.
- Residual: Cargo still reads `$CARGO_HOME/config.toml` and ancestor
  `.cargo/config.toml`, since `env_clear` cannot prevent config discovery. The
  checkout has no ancestor config. This is acceptable for the supported host,
  not a finding.

**N6: resolved.** `entry.rs:176`: `git diff 07e104b fx-3iu --
crates/tide-std/tests/entry.rs` has no hunk at that assertion any more.

**N7: resolved.** `build.rs:591` `error_derive_formats_the_original_pointer_with_an_unsafe_grant`
- The test admits a `{address:p}` message on a `&'a u8` field under
  `--tide-grant=unsafe` and runs it.
- It compares the message against `format!("{address:p}")` on the original
  reference. That comparison catches the extra-borrow address bug that the
  `Pointer` wrapper exists to prevent.
- The run is executed, not just admitted.

## New findings in the changed lines

None blocking.

- Nit, positional rule on an empty tuple struct (`error.rs:343`). The check keys
  on `Fields::Unnamed(_)`, while thiserror keys on "some field is unnamed". So
  `#[error("{0}", 5)] struct S();` is rejected here but accepted by thiserror,
  which leaves it to rustc and prints `5`. It is vanishingly rare and errs toward
  rejection. Fix only if you touch the line again, with
  `matches!(fields, Fields::Unnamed(u) if !u.unnamed.is_empty())`. No re-check
  needed.

DONE
