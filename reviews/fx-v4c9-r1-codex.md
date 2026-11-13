Model/Effort/Date: GPT-6.1 Sol / high / 2026-11-13

Verdict: approve. Confidence: high for this design-only candidate. No blocking findings, non-blocking improvements, or required patch. Proceed to the required Opus review round; this verdict does not approve landing by itself.

Findings: none.

Reviewed candidate: `1732aca4993e69c4e8e671563d13dee102a8fe6c`, `git diff HEAD^..HEAD`, diff SHA-256 `fc0e1f42a9d960bc383be9244fa1d0bf9409d387b4f38f055274b100f7aa2ecf`. The worktree was clean before and after inspection, and the candidate identity remained unchanged. The author's report describes an earlier proposal revision; this verdict covers the actual three-file candidate and the coordinator's decision in research/29 §10 and `br --no-db show fx-v4c9`.

Contract assessment:

- CODEC-1/2 preserve the v0 owner and entry points. Strictness is local to each object type. For external tags, it governs named-field payloads while retaining the existing single-key enum wrapper and other variant rules. Maps remain data, and nested types retain their own policy.
- Defaults still cover absence, not invalid present values. Skipped fields add no permitted payload key under strictness; the discriminator independently owns its key. Existing field/container default precedence and loose skipped-value behavior remain governed by the current codec rules.
- Variant `rename` overrides enum case conversion for discriminator values. Neither changes the literal discriminator key or named-variant field names. Field renaming still applies. Missing, repeated, non-string, and unknown tags fail independently of strictness; decoded-string comparisons cover escaped-equivalent duplicates. Payload order is unrestricted, known payload duplicates remain errors, and no fallback variant is allowed.
- Placement, malformed/repeated keys, unsupported tagged shapes, and wire-name collisions are compile-time checks shared by both derives, including single-trait derives. The limits, absolute error positions, raw numbers, whole-input consumption, and ROOT-4 boundary are retained requirements rather than claimed implementation results.

Evidence assessment:

- Inspected Eon sources through `git show 7d1f921427a6c094c85f4a4dd9688f92b2356db4:<path>`. The dependency-free EONW crate implements binary v42 framing, numeric action dispatch, checked payload offsets, and strict frame/payload consumption. The manifest's `Source` is exactly the strict, lowercase, internally tagged `kind` enum described; its parser validates a JSON value and then deserializes the original input. The cited session-registry and TOML settings examples also match. The note correctly distinguishes these workloads and claims no complete Eon migration.
- Spot-checked all §4 documentation links. The [Serde container rules](https://serde.rs/container-attrs.html), [enum shapes](https://serde.rs/enum-representations.html), and [borrowing explanation](https://serde.rs/lifetimes.html) support the comparison. The [miniserde 0.1.46 documentation](https://docs.rs/miniserde/0.1.46/miniserde/) supports its stated derive, customization, trait-object, and error limitations.
- The [nanoserde trait](https://docs.rs/nanoserde/0.2.1/nanoserde/trait.DeJson.html), [crate description](https://docs.rs/nanoserde/0.2.1/nanoserde/), and [feature matrix](https://github.com/not-fl3/nanoserde#features-support-matrix) support the bounded claims. The note explicitly avoids treating matrix omissions as proof of impossibility. The [Go documentation](https://pkg.go.dev/encoding/json) supports strict-field decoding, stream decoding, and the stated v1 case/duplicate behavior.
- Proposed scanning, costs, and admission conclusions are labeled inference. No measured performance, adoption, or learnability result is claimed.

Placement and admission: D6, the tier table, and the companion agree on v1; the v0 core reference is unchanged. No contradictory codec rule was found in the design text. Option A passes the admission filter for a bounded extension of existing toolchain derives and std parsing, with schema policy owned by types and no second runtime API or dependency. No alternative is required for approval. The implementation follow-ups exist as fx-1p2i and fx-tf7d.

Verification: read-only source/document review on the supplied Linux workspace; `git diff --check HEAD^..HEAD` passed. Only this requested report was written. No builds or tests ran. Future tag-scanning behavior and performance remain unproven and belong to the implementation follow-ups.

Held-out exclusion: `trials/e1-checked/` was never opened or searched.

DONE
