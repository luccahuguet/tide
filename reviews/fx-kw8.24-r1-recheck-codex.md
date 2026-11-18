Model: gpt-6.1-sol (high) · Date: 2026-11-18 · Candidate 4903049 (re-check of e9c66af finding)
Verdict: approve

1. **Finding 1 — resolved.**
   `crates/tide-admit/src/pure.rs:357` checks the defining ADT before
   rendering its field. At the first foreign ADT, it prints only that ADT's
   name and breaks the outermost-to-innermost rendering loop. The foreign
   field name, enum variant, subsequent tuple displays, and all descendant
   steps are therefore omitted. Using `adt` rather than `owner` also avoids
   exposing a foreign enum's rejecting variant.

   The boundary marker does not expose instantiated generic arguments:
   `pure.rs:353` calls `tcx.def_path_str(adt)`, and the pinned rustc source
   implements that by calling `def_path_str_with_args(def_id, &[])`. The
   marker does not print the foreign `Ty`, inspect its fields, or normalize
   anything. Tuple steps before the boundary remain visible, as intended;
   tuple steps behind it are never formatted. The existing root and terminal
   message text is unchanged by this fix.

   Acceptance policy is unchanged. At `pure.rs:501`, every field remains in
   the same ordered `try_for_each` traversal, with the same recursive call,
   normalization, depth, visited set, and failure propagation. The only walk
   change adds the ADT DefId to diagnostic data after a failure. The privacy
   stop is exclusively in rendering; it does not skip decision work.

   `crates/tide-admit/tests/build.rs:787` adds the requested paired-library
   regression. A private `secret_inner: Inner` leads through the private
   `hidden_pair: (u8, Net)` to authority. Both direct `Vault` and an outer
   `(u8, Vault)` obligation must still reject with the expected TD0517
   message ending at `Vault`; the outer tuple step must remain. Assertions
   also exclude both private field names, the private helper type, and the
   internal tuple display from all stderr. The old renderer fails these
   checks, and skipping the rejecting fields would fail the expected-message
   checks. The fixture is non-generic; absence of generic arguments in the
   boundary marker was checked in source rather than by executing a probe.

No regression introduced by this fix was found. No further fix requested.
Confidence: high. No override recorded.

Scope: only `git diff e9c66af 4903049`, the original finding, the author's
response, and directly relevant rendering/test/printer context were checked;
no new full review. HEAD was `4903049624e6157417d2ce3e434d7285cd940d9a`,
with a clean tracked working tree. Diff SHA-256:
`800694d32412030c17bd3fd5360e33d278352c9ff04ee6173b6d7614dc85838f`.
The author's focused and workspace logs both record the new regression passing.

Not checked: no builds or tests were run; trials/e1-checked/ was never opened.
Risk: foreign public field detail is also intentionally suppressed by this conservative stop.
DONE
