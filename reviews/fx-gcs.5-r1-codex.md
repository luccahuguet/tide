Model: `gpt-6.1-sol`. Effort: `xhigh`. Date: 2026-10-21.

## Verdict

**request_changes — high confidence.** The direction is sound; one new case contains unmarked v0 errors.

## Scope

Read-only review of branch `fx-gcs.5`, working-tree diff against `e6e8103`, the requested governance, beads, design sections, corpus, runner, and stub std.

The supplied fingerprint **matched** initially:

`7d0b819efde352e188991ba139c1ff08a7f8e9a634853aec6a89b7a3d4f71db6`

The candidate diff retained that hash throughout. The final full fingerprint became `65e35fe2275a72395707ddc16ccb2bb76b4734cc7d1029796e06b39a275e2cfa` solely because the parallel `reviews/fx-gcs.5-r1-opus.md` appeared as an untracked file. I changed no file.

## Blocking

1. **The float case has unrelated, unmarked FX0301 errors.** [conformance/cases/tasks/par_float_reject.rs:13](/home/lucca/pjs/lab/tide-fx-gcs.5/conformance/cases/tasks/par_float_reject.rs:13)

   `ord as u64` cannot represent `Ordering::Less`’s discriminant, `-1`. `(lo + hi + s) as u64` is a lossy float-to-integer conversion. Both violate D3 and `design/reference.md` §4.2, which assigns FX0301.

   **Risk:** a correct Tide checker produces errors absent from this case’s expected verdict. Today’s runner cannot detect that omission.

   **Smallest fix:** make `observers` return `()` and remove the final summation. This retains every intended PAR-3 rejection without introducing conversion tests.

## Non-blocking

1. **Strengthen PAR-5’s accepting boundary.** [conformance/cases/tasks/par_join_accept.rs:12](/home/lucca/pjs/lab/tide-fx-gcs.5/conformance/cases/tasks/par_join_accept.rs:12)

   Moving a String into the closure does not make it `FnOnce`-only: `label.len()` merely borrows it. An in-memory probe confirmed that the entire accepting case still compiles with stricter `FnMut` bounds. Changing this expression to `label.into_bytes().len()` would protect acceptance of closures that consume captures.

2. **Apply the marking rule consistently when integrating fx-gcs.3.** [conformance/cases/authority/lock_payload_not_send.rs:8](/home/lucca/pjs/lab/tide-fx-gcs.5/conformance/cases/authority/lock_payload_not_send.rs:8)

   The candidate’s dual marks are correct. README’s “a line both report on carries both” means LOCK-1’s non-Send payload also needs `E0277 FX0606`, because LOCK-1 attaches FX0606 to `T: Send + Pure`. This unchanged case belongs to the parallel work.

   Preserve both FX0610 and incoming FX0518 when merging §3.1’s list. The general Pure inventory at `conformance/rules.md:106` also still omits FX0610.

No findings confined to later-tier text warrant “for fx-djp.”

## Reasoning

- **Coverage:** PAR-3 through PAR-7 each have accepting and rejecting cases, subject to the blocking correction. Existing SCOPE-1–4 and SEND-3 cases cover the static task boundaries; `send_local_accept` usefully protects task-local non-Send values. Coverage is not literally complete for SEAL-2/3: those remain uncovered v0 rules, explicitly recorded on fx-gcs. This pass does not establish the parent epic’s completion.
- **Exclusions:** TASK and JOIN prescribe runtime failure, cancellation, and joining behavior. Their relevant static lifetime and transfer restrictions already have SCOPE/SEND owners. SUSP-1/2 classify instances without rejecting programs; SUSP-3 permits a code-generation refinement. Their exclusions are justified. Moving warning-only FX0608 out of an error-mark corpus is also correct.
- **Diagnostics:** FX0610 consistently covers closure admission across 01 D18, 03, reference §8, and the companion. PAR-7 makes shared-capture failures use FX0611 instead. The overlap case correctly records rustc’s E0524 without inventing an FX code for borrow disjointness.
- **Ownership:** The `= PAR-5/6/7` and `= SEND-3` conversions correctly identify restatements and cite their owning rules.
- **Stub:** `par_for` matches the reference signature. `join` matches the documented closure bounds; its additional result Send bounds are explicitly marked GUESS rather than presented as settled design.
- **Verification:** With rustc **1.96.0**, read-only front-end checks using `RUSTC_BOOTSTRAP=1 -Zunpretty=mir`, ordinary and test modes, and the existing stub artifact reproduced **96 PASS, 25 AWAITS CHECKER, zero mismatches**. The current stub source also passed front-end checking. `git diff --check e6e8103` passed.

Not checked: a fresh Cargo rebuild or literal `run.sh` execution, which write artifacts; runtime behavior and Tide FX diagnostics remain unproven.