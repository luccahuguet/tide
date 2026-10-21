Model: `gpt-6.1-sol`. Effort: `xhigh`. Date: 2026-10-21.

## Verdict

**request_changes — high confidence.** Both round-9 reproducers are fixed. The indexing fix introduces a ROOT-4 false acceptance.

## Scope

Read-only narrow review of `ce6d728..5295926 -- crates conformance`, with AGENTS.md, GOVERNANCE.md, round 9, relevant specifications, callers, tests, and pinned rustc implementation.

HEAD remains `52959261bedf017a4569caf374077c699f871696`. No files changed; worktree clean.

## Blocking

1. **P2 — The indexing exemption crosses implicit dereferences that read values.**  
   [walk.rs:540](/home/lucca/pjs/lab/tide-fx-kw8.2/crates/tide-admit/src/walk.rs:540)

   Under the unsafe grant, this produces no FX0517:

   ```rust
   use std::fs::Fs;

   pub fn address(p: *const &[Fs; 1]) -> *const Fs {
       unsafe { &raw const (*p)[0] }
   }
   ```

   Indexing implicitly loads the reference stored at `p`. That is a raw-pointer read producing a non-`Pure` value. Making the load explicit—`let base = *p; &raw const base[0]`—correctly produces FX0517.

   Confirmed variants include `*const Box<[Fs; 1]>` and indexed writes through `*mut &mut [Fs]`. A user type containing `Fs` and implementing `Deref<Target = [u64; 1]>` also bypasses the check: `&raw const (*p)[0]` invokes its safe `deref` method, which can access that capability.

   `!t.is_method_call(parent)` excludes overloaded `Index`/`IndexMut`, but builtin indexing can still follow implicit dereferences of its base. The new traversal then reaches the raw-borrow or assignment exemption and suppresses the required diagnostic.

   **Minimal fix:** follow indexing only when reaching the indexed place requires no intervening reference/owner load or overloaded dereference. Add rejecting controls for a reference-valued raw pointee and a user `Deref` base, while retaining the direct array/slice exemptions.

   **Risk:** this weakens ROOT-4’s specified defense under an unsafe grant; it does not establish an escape from safe code.

## Non-blocking

None.

## Reasoning

- **Callable fix resolved:** all three new regressions receive FX0610 at the call. Deeper reference/Box chains also reject correctly. FX0611 retains capture priority; valid shared-reference adapters and mutable `join` adapters remain clean in admission probes.
- **Original indexing rejection resolved:** direct indexed writes and raw borrows stay clean for arrays, slices, and nested projections. Reads, compound assignments, and index operands remain checked. Direct overloaded indexing remains checked.
- **Verification:** existing driver artifacts matched every FX line/code mark in `par_shared_reject.rs`, `par_capture_reject.rs`, and `root_zeroed.rs`, in both library and test modes. Additional stdin probes reproduced the blocker; an unrelated lifetime error prevented artifact generation.

Not checked: fresh builds, `cargo test --workspace`, or `conformance/run.sh`; both suites require writes. Author-reported results were not independently rerun.  
Risk: the indexing exemption still suppresses FX0517 for implicit-dereference cases.