Model: gpt-6.1-sol · Effort: xhigh · Date: 2026-11-03

## Verdict

**request_changes — high confidence.** The direction is sound, but the artifact can omit reachable authority and granted holes, misattribute holes, and survive an admission failure.

## Scope

Read-only review of bead `fx-kw8.10`, consumer `fx-kw8.11`, the required governance/design sections, all seven staged files, and surrounding admission code.

Verified base `0141db4` and staged SHA-256 `f768ba29969675d1d7bf651028464f00a7902f11d24ea18e95fca955a3de19d0`. The hash remained unchanged. No files edited.

## Blocking

1. **[P1] Synchronization wrappers discard their payload’s authority.**  
   [interface.rs:215](/home/lucca/pjs/lab/tide/crates/tide-admit/src/interface.rs:215)

   `Carries::Kinds` reports the wrapper’s kinds and stops. With the existing conformance std, `Arc<Net>`, `Receiver<Net>`, or `Task<'_, Net, E>` therefore reports `sync` while omitting reachable `net`. Changing an `Arc<u8>` parameter to `Arc<Net>` can leave its declared authority unchanged.

   **Minimal fix:** report the named kinds and traverse the relevant type arguments at the holder’s path. Add an `Arc<Net>` regression assertion.

2. **[P1] Granted trait impls are admitted without being recorded.**  
   [walk.rs:228](/home/lucca/pjs/lab/tide/crates/tide-admit/src/walk.rs:228), [walk.rs:1229](/home/lucca/pjs/lab/tide/crates/tide-admit/src/walk.rs:1229)

   `decide` now returns `Admitted::Granted`, but `obligation` handles only `Err`. For example, `format!("{f:?}")` for `f: fn()` requires the grant through `impl Debug for fn()`, yet contributes no hole. The mono backstop likewise discards successful granted decisions and returns no holes.

   **Minimal fix:** record granted impl selections at their local holding item, including selections caused by formatting expansions, and merge backstop findings into the artifact. Preserve the existing rejection behavior under toolchain expansions; routing everything through the ordinary suppression path would weaken it. Test formatting without an explicit unsafe block.

3. **[P2] Deep analysis can become empty authority without reporting its limit.**  
   [interface.rs:189](/home/lucca/pjs/lab/tide/crates/tide-admit/src/interface.rs:189), [interface.rs:236](/home/lucca/pjs/lab/tide/crates/tide-admit/src/interface.rs:236)

   For `struct W<T>(T)`, a parameter containing 65 nested `W`s around `u8` fails `Pure::check` at its depth limit. Extraction descends once, checks the remaining 64 wrappers afresh, succeeds, and emits `{}`. It never reaches its own opaque fallback. This contradicts the recorded rule that undecidable deep types report `opaque`, and breaks the advertised relationship between empty authority and `Pure`.

   **Minimal fix:** preserve the depth-limit failure and emit `opaque` at that path, or explicitly report unsupported analysis. Test both sides of the boundary.

4. **[P2] Nested declaration holes belong to the enclosing function.**  
   [walk.rs:373](/home/lucca/pjs/lab/tide/crates/tide-admit/src/walk.rs:373)

   In `pub fn outer() { unsafe fn inner() {} }`, visiting `inner` updates `self.item`, but `self.owner` still identifies `outer` until `inner`’s body starts. Because `hole` prefers `owner`, the unsafe-function declaration is recorded against `outer`. Nested unsafe impls and declaration attributes have the same problem. Moving a hole between nested items can become invisible to the consumer.

   **Minimal fix:** establish declaration ownership at item boundaries while retaining closure-to-function collapsing. Assert that the nested unsafe-function hole belongs to `inner`.

5. **[P2] Granted macro uses lose their holding item and collapse together.**  
   [walk.rs:1287](/home/lucca/pjs/lab/tide/crates/tide-admit/src/walk.rs:1287)

   `check_expansions` runs after traversal, with neither body nor item ownership set. Inline `asm!` uses consequently belong to the crate root and deduplicate across functions. The existing generated `unsafe_granted.interface.json` corroborates this: its `asm` use belongs to `unsafe_granted`, rather than `unsafe_granted::barrier`.

   **Minimal fix:** record granted expansion uses with their actual HIR holder. Test two functions containing assembly and verify separate entries, each counted once.

6. **[P1] Early admission failures leave the previous artifact intact.**  
   [invocation.rs:653](/home/lucca/pjs/lab/tide/crates/tide-admit/src/invocation.rs:653)

   Cleanup occurs after source resolution and the `build.rs` rejection. After a successful check, adding `build.rs` causes `FX0915` and returns before cleanup, leaving the previous interface artifact beside the metadata. Removing the source also leaves it behind.

   **Minimal fix:** invalidate the named artifact after safely validating the output destination but before candidate rejection paths. Add a success-then-`build.rs` rejection test.

7. **[P2] Unsupported std discovery silently succeeds without an artifact.**  
   [interface.rs:50](/home/lucca/pjs/lab/tide/crates/tide-admit/src/interface.rs:50)

   `crate_pure == None` returns success. This conflates invocation without `--tide-std` with failed discovery despite that flag. A library containing `#![no_std] pub fn f() {}` can pass admission with `--tide-std`, but std injection supplies no Tide std identity, so no artifact is produced and no unsupported-analysis result is reported.

   **Minimal fix:** use the invocation’s `tide_std` flag to distinguish the intentional no-artifact mode; require successful discovery otherwise, or fail explicitly as unsupported.

8. **[P2] Public aliases and external reexports disappear silently.**  
   [interface.rs:60](/home/lucca/pjs/lab/tide/crates/tide-admit/src/interface.rs:60)

   The loop examines local definitions and skips `Use` and `TyAlias`. Consequently, `pub use std::net::Net as Authority;` or `pub type Authority = std::net::Net;` exposes authority through the package interface but produces no corresponding entry or unsupported indication.

   **Minimal fix:** handle these exported forms, or explicitly reject/report them as unsupported. Supporting their full extraction is unnecessary to satisfy the completeness requirement.

## Non-blocking

Extend the integration test with a cache-hit round trip and an ordinary body-only edit that preserves the artifact. The cache key includes source, driver, grants, and trusted inputs; I found no separate ordinary cache-key staleness defect.

## Reasoning

Compiler-derived extraction, deterministic ordering, shared capability classification, and publication through the existing staging cache are appropriate choices. No new dependency or architectural layer is needed.

The current test covers nested local fields, a pure function, one unsafe block, and a compiler rejection. It does not protect the classification, ownership, and early-failure boundaries above. Existing conformance checks primarily validate admission verdicts, so passing them does not establish artifact completeness.

Static comparison found no concrete regression in the refactored direct-site `FX0901` branches. The granted-impl fix must preserve rejection of user-selected impls even beneath toolchain expansions.

`git diff --cached --check` passed. I attempted `cargo test --workspace --locked --offline`; Cargo exited before compilation because opening `target/debug/.cargo-build-lock` failed with `Read-only file system`.

Not checked: executable tests and `conformance/run.sh`; the read-only sandbox prevented build execution.  
Risk: the current artifact is incomplete in cases its downstream comparison would treat as authoritative.