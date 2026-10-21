Model: gpt-6.1-sol · Effort: xhigh · Date: 2026-10-21

## Verdict

**request_changes — high confidence.** The direction is sound, but two stub API mismatches change compile-time verdicts.

## Scope

Reviewed fx-gcs.6 and its parent, the eight-file diff, corpus rules, relevant design sections, and pinned Rust signatures. HEAD and diff SHA-256 match the supplied candidate and remained unchanged. No files were edited.

## Blocking

1. **`Default` and `FromIterator` incorrectly fix the hasher type.**  
   [lib.rs:196](/home/lucca/pjs/lab/tide-fx-gcs.6/conformance/rustc/std/src/lib.rs:196), also lines 200, 280, and 284.

   Rust implements these for generic `S`; the stub implements them only for Tide’s `RandomState`. Read-only compiler probes confirmed both consequences:

   - Unannotated `HashMap::default()` and `HashMap::from_iter(...)` produce E0283 in Rust but succeed against the stub. `HashSet` behaves likewise.
   - Explicit custom-hasher collections support `Default` and `collect()` in Rust but produce E0277 against the stub.

   This makes the oracle report incorrect type verdicts. **Smallest fix:** preserve Rust’s generic impls: `S: Default` for `Default`, and `S: BuildHasher + Default` for `FromIterator`. Give Tide’s `RandomState` a fixed-key `Default` impl. Add a compact inference/custom-hasher regression probe.

2. **`RecvTimeoutError` lacks its specified error traits and conversion.**  
   [lib.rs:614](/home/lucca/pjs/lab/tide-fx-gcs.6/conformance/rustc/std/src/lib.rs:614); required by [03:1375](/home/lucca/pjs/lab/tide-fx-gcs.6/design/03-authority-concurrency.md:1375).

   The new enum implements neither `Display` nor `Error`, and lacks `From<RecvError>`. The chapter explicitly requires all three. A compiler probe confirmed that an `Error` bound fails with E0277, so ordinary propagation of `recv_deadline(...)` into a boxed error also fails.

   **Smallest fix:** add those three impls and a compile probe covering the error bound and conversion. The `.ok()` call in the new case hides this mismatch.

## Non-blocking

- **The hardware case makes the default corpus architecture-dependent.** [admit_hardware.rs:5](/home/lucca/pjs/lab/tide-fx-gcs.6/conformance/cases/authority/admit_hardware.rs:5) is unconditionally compiled; its comment does not restrict execution. Since the brief explicitly discloses this limit, document the x86_64 requirement in the corpus README. Also remove the stale claim that the corpus has no hardware case in [driver.rs:74](/home/lucca/pjs/lab/tide-fx-gcs.6/crates/tide-admit/tests/driver.rs:74).
- Both `with_hasher` constructors lose Rust’s `const` qualification ([lib.rs:144](/home/lucca/pjs/lab/tide-fx-gcs.6/conformance/rustc/std/src/lib.rs:144), line 247). No current case exercises const collections; record this limitation before using the stub for that verdict.

## Reasoning

The CAP-1 reclassification is sound: Tide’s own `Receiver` lacks `recv_timeout`, making this analogous to its missing `Instant::now`. `E0599 FX0501` correctly records the plain-rustc error and intended Tide diagnostic. I found no remaining timed-wait-as-FX0520 text in `design/`, the companion, or `conformance/rules.md`.

The wrappers and inherent methods preserve the relevant Rust ownership and borrowing structure. The accepting rustc probes passed in library and test modes; the hardware driver probe produced FX0520 at the marked line, and timed waits produced E0599 as expected. `git diff --check` passed.

Not rerun: full suites or token counting; compiler probes used existing artifacts, and accepted driver runs hit the read-only artifact limit.  
Remaining risk: FX0501 is still unimplemented, as documented.