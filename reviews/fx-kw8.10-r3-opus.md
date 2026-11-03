# Tide review, fx-kw8.10 round 3 (Opus): converge confirmation round

Candidate: the staged diff on base 0141db4, SHA-256
`5dd005cbec422d0780473ed41a82312e7617b003f32259873a2ff92c8d6a5179`.

Model: claude-opus-5-5 · Date: 2026-11-03

## Verdict

Findings: 1 blocking, 2 non-blocking, 1 nit. Round 2's B1, B3, N2, and nit
are fixed; B2 is fixed for local items only; N1 is fixed for the case named.
`cargo test --workspace` and `conformance/run.sh` exit 0.

## New findings

- **B1, blocking** (`interface.rs:103`): re-exports are keyed
  `(path, "re-export")`, so a type and a value re-exported under one name
  overwrite each other: `pub use std::net::Net as X; pub use std::mem::drop as
  X;` lists only the function, and `Net`'s `net` disappears. Fix: put the
  target's kind in the key.
- **N1, non-blocking** (`walk/mono.rs:69`): `requires_monomorphization` is
  true for every closure, so the backstop drops holes only it finds inside
  closures of non-generic functions (`sort` on `Vec<fn()>` loses
  `impl PartialOrd for fn()`). Fix: test the closure's typeck root.
- **N2, non-blocking** (`interface.rs:306`): finite recursion that is not
  polymorphic is `opaque` (`T2<u8>` inside `T2<Fs>`, `Swap<u8, Net>` inside
  `Swap<Net, u8>`, `A<A<Net>>` at its own path). Fix: keep every held
  instance, skip an equal one, and report `opaque` only when the type
  arguments grow.
- **Nit**: rustfmt diffs at interface.rs:80, :265, :283, invocation.rs:71,
  walk.rs:72, :376-379; two doc lines over 100 columns.

## Checked

B1's ordering and every later failure path (nothing written in the package);
the child compile run and the cache-hit copy; JSON validity and key sets;
generic and non-generic hole lists; holders after the walk; `holder_at`
timings now linear (6,000 functions: 13 s, was 18 s); docs against the code.
