Model: gpt-6.1-sol · Effort: xhigh · Date: 2026-11-01

## Verdict

**request_changes — high confidence.** The approach is appropriate, but three paths bypass the entry requirement.

## Scope

Reviewed fx-kw8.8 and parent fx-kw8 against GOVERNANCE.md, ROOT-1, D18, the reference, driver code, macros, and conformance tests.

Confirmed HEAD `1e0cbac` and candidate diff SHA-256 `b53d820afbf8052fb2a5965e66d1d216ba81cdcc1593cfc33610e770cbb5a3ec`. No files changed.

## Blocking

1. **Candidate source can disable FX0512 through `#![no_std]`.**  
   [crates/tide-admit/src/walk.rs:1194](/home/lucca/pjs/lab/tide/crates/tide-admit/src/walk.rs:1194)

   With the trusted Tide std supplied, this binary escapes the check:

   ```rust
   #![no_std]
   extern crate std;
   fn main() {}
   ```

   `Crates::injected_std` recognizes only rustc’s generated std import. Here rustc injects `core`, while the explicit import links std and its runtime support. `tide_std()` consequently returns `None`, disabling FX0512. No existing guard rejects these source constructs.

   **Risk:** candidate-controlled source bypasses ROOT-1 despite the operator supplying Tide’s std.

2. **Imported entry functions return before enforcement.**  
   [crates/tide-admit/src/walk.rs:1190](/home/lucca/pjs/lab/tide/crates/tide-admit/src/walk.rs:1190)

   A trusted std fixture exporting `pub fn entry() {}` permits this binary:

   ```rust
   use std::entry as main;
   ```

   The function is admitted as a Tide-std item, and rustc supports a nonlocal resolved entry. `main.as_local()` then fails, returning before FX0512. Trusting a callable std function does not establish that it starts Tide’s runtime.

   **Risk:** an admitted imported function becomes the binary entry without the required attribute.

3. **`#![no_main]` suppresses the check entirely.**  
   [crates/tide-admit/src/walk.rs:1187](/home/lucca/pjs/lab/tide/crates/tide-admit/src/walk.rs:1187)

   With the supported `unsafe` grant, this source supplies a C entry while bypassing Tide’s runtime:

   ```rust
   #![no_main]
   #[unsafe(no_mangle)]
   pub extern "C" fn main() -> i32 { 0 }
   ```

   The pinned compiler’s `entry_fn` explicitly returns `None` for `no_main`. The driver therefore skips FX0512; its unsafe-attribute check permits `no_mangle` under the grant. ROOT-1 provides no corresponding exemption.

   **Risk:** a binary can receive admission without any Tide-generated entry.

## Non-blocking

The decoy regression usefully tests macro identity. Positive cases for a renamed genuine attribute and `cfg_attr` would strengthen coverage of the same mechanism.

The diagnostic span and help are appropriate for ordinary source-defined `fn main`. An imported-entry diagnostic should point at the local import and explain how to provide a marked local entry.

## Alternatives

Keep enforcement in `entry_point`; no new subsystem is needed. If Rust-std development fixtures require an exemption, derive it from explicit trusted invocation state, rather than candidate-controlled std injection.

## Minimal patch for approval

- Remove the source-dependent std-presence exemption.
- Apply ROOT-1 before the nonlocal-entry return; retain locality requirements only where subsequent obligation analysis needs them.
- Reject source `no_main` on binary targets while preserving rustc’s diagnostics for genuinely missing entries.
- Add regressions for these three paths. Keep the valid marked-entry and compiler-generated test-harness cases passing.

## Reasoning

Using rustc’s resolved entry avoids spelling-based searches, and the existing expansion check recognizes genuine attribute aliases by macro identity. The diagnostic code registration and README update are consistent. The problem is that eligibility shortcuts run before enforcement and are broader than ROOT-1’s exemptions.

The supplied successful corpus and workspace runs cover the existing fixtures, but none exercises these bypasses. The conclusions above follow from the driver and installed pinned compiler sources.

Not checked: builds or execution in this read-only sandbox; the examples were traced statically.  
Risk: unmarked binary entries can still pass admission.
## Resolution (author)

- B1: the entry rule's switch is now the operator's `--tide-std` (`Admission::tide_std`),
  not the candidate's std injection, so `#![no_std]` cannot turn it off
  (`conformance/cases/authority/root_main_no_std.rs`). The broader effect of `#![no_std]`
  on `Crates::tide_std` (the `Pure` decision and exports) is outside this bead and is filed
  separately.
- B2: FX0512 is reported before the locality return, at the import's span
  (driver test `an_imported_main_is_unmarked`).
- B3: `#![no_main]` on a binary is FX0512 when Tide's std is supplied
  (`conformance/cases/authority/root_no_main.rs`).
