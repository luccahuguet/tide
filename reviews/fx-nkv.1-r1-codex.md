Model: gpt-6.1-sol / Effort: xhigh / Date: 2026-10-31  
Verdict: **request_changes** (high confidence)

Scope: Reviewed `fx-nkv.1`, parent `fx-nkv`, repository governance, the complete candidate diff, touched files, surrounding admission/Pure logic, relevant specs, and tests. HEAD and candidate SHA-256 match the supplied identities.

Blocking:

1. **The entry attribute permits minting outside the binary entry.**  
   [tide-macros/src/lib.rs:25](/home/lucca/.cache/eon-agents/tide-nkv1/crates/tide-macros/src/lib.rs:25) checks only that the function is named `main`. This library function therefore generates a callable minting entry:

   ```rust
   pub fn forge() {
       #[tide::main]
       fn main(root: Root) {
           let _net = root.net;
       }
       main();
   }
   ```

   A read-only MIR check confirmed the generated call to `std::__rt::enter`. The admission exception at [walk.rs:284](/home/lucca/.cache/eon-agents/tide-nkv1/crates/tide-admit/src/walk.rs:284) trusts macro provenance without checking entry placement. The guard at [root.rs:141](/home/lucca/.cache/eon-agents/tide-nkv1/crates/tide-std/src/root.rs:141) permits the first invocation, including from test code.

   **Risk:** library or test-local code can obtain a real `Root`, contrary to ROOT-1 and this bead’s trusted-entry scope.

   **Smallest fix:** restrict the trusted attribute to the crate-root binary entry, preserving its unused test-build form; reject nested, module-local, and library applications. Add a negative conformance case for this route.

2. **Destination failures can suppress required formatting effects and panics.**  
   [root.rs:45](/home/lucca/.cache/eon-agents/tide-nkv1/crates/tide-std/src/root.rs:45) formats directly into the output stream. The pinned Rust implementation stops formatting when a write fails.

   For example, `eprint!("prefix {}", Bomb)` with a closed stderr can fail while writing `"prefix "` and never invoke `Bomb::fmt`. If that formatter mutates a `Cell` or panics, the destination’s state changes program behavior. This violates SINK-3’s requirement that formatting effects occur and formatting panics propagate.

   **Smallest fix:** format the complete record into an owned string before attempting destination I/O, then write and flush it under the stream lock. Add a deterministic failing-writer test with an observable formatter.

Non-blocking: No additional improvements or alternative architecture are needed for this slice. Sharing definitions with the conformance fixture is a reasonable way to prevent API drift.

Reasoning: The direction and bounded API are sound, but the trusted-entry boundary and sink semantics need correction. The recorded workspace tests pass, and the conformance log contains 209 passing or awaiting-checker verdicts. Those checks do not cover either counterexample above.

Not checked: I did not rerun Cargo/conformance or execute the counterexamples; conclusions use source tracing and read-only MIR inspection. No files were edited.  
Risk: the candidate currently permits off-entry minting and destination-dependent formatting behavior.