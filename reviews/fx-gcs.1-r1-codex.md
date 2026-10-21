Model: gpt-6.1-sol. Effort: xhigh. Date: 2026-10-21.

## Verdict

**request_changes — high confidence (95%).**

The direction is sound, but the runner can overlook unexpected errors, and the format cannot express some rules already in its inventory.

## Scope

- HEAD: `901c512a05218690ca75413ca9b1f6848dd4df36`, plus the staged change.
- Before SHA: `f0354b1a49ba26ef145c143101855d3e0c579645f8ce61e156cc67a914350d3e`
- After SHA: `f0354b1a49ba26ef145c143101855d3e0c579645f8ce61e156cc67a914350d3e`
- Read `AGENTS.md`, `GOVERNANCE.md`, both beads, the candidate files, the trial runner, and relevant sections of design chapters 01–04.
- `bash -n`, `git diff --check`, and `conformance/run.sh --trace` succeeded.
- `conformance/run.sh` exited 2: the read-only sandbox prevented opening Cargo’s build lock.
- Direct rustc probes using existing artifacts confirmed both rejecting seeds’ `E0277` locations. Additional probes checked uncoded lifetime diagnostics, binary versus test compilation, and diagnostic parsing.
- No files edited. Stub-copy differences are comments, package descriptions, and the adjusted Cargo dependency path; implementation bodies and lockfile match.

## Blocking

1. **Unexpected errors in paths containing spaces disappear.** [conformance/run.sh:63](/home/lucca/pjs/lab/tide-fx-gcs.1/conformance/run.sh:63)

   The diagnostic regex excludes spaces from paths, and unmatched located errors have no fallback. Supplying the parser with an expected `/source/case.rs:8:29: error[E0277]` and an unexpected `/source/helper file.rs:1:20: error[E0308]` produces only `8:E0277`. The comparison therefore permits PASS—or AWAITS CHECKER with FX marks—despite the unmarked error. Actual rustc output preserves spaces in paths.

   **Smallest fix:** accept spaces in diagnostic paths and retain unmatched error diagnostics as failures. Add one regression check for an expected error accompanied by an unexpected error in another file.

2. **The mark grammar cannot represent ordinary lifetime failures.** [conformance/run.sh:53](/home/lucca/pjs/lab/tide-fx-gcs.1/conformance/run.sh:53), [conformance/README.md:55](/home/lucca/pjs/lab/tide-fx-gcs.1/conformance/README.md:55)

   The inventory includes “lifetime may not live long enough” at [conformance/rules.md:36](/home/lucca/pjs/lab/tide-fx-gcs.1/conformance/rules.md:36). Rustc 1.96 emits that error without an `E####` code for:
   ```rust
   pub fn bad<'a, 'b>(s: &'a str) -> &'b str { s }
   ```
   The runner records `line:error`, but no permitted mark can match it. Correct negative cases for this rule—and several other inventoried lifetime rules—must always FAIL.

   **Smallest fix:** extend the grammar and comparison to explicitly expect uncoded rustc errors, while keeping unexpected uncoded errors fatal. Document it and add a representative check.

3. **Test-only compilation cannot exercise ordinary binary and non-test verdicts.** [conformance/run.sh:58](/home/lucca/pjs/lab/tide-fx-gcs.1/conformance/run.sh:58)

   Every case receives `--test`. This removes `#[cfg(not(test))]` code and suppresses binary entry-point checks. For example, `fn main(_n: u8) {}` produces E0580 as a binary but not under `--test`; the inventory explicitly includes that rustc behavior under ROOT-1. The format currently cannot express its binary case.

   **Smallest fix:** support a per-case compilation kind—binary, library, or test—with the current test mode as the default. Document it and check the E0580 distinction.

4. **CLAUSE-6 has a compile-time restriction that the inventory excludes.** [conformance/rules.md:168](/home/lucca/pjs/lab/tide-fx-gcs.1/conformance/rules.md:168)

   [design/04-contracts.md:468](/home/lucca/pjs/lab/tide-fx-gcs.1/design/04-contracts.md:468) forbids an invariant over a non-range iterator from naming the loop pattern’s bindings. That affects name resolution and acceptance, rather than only runtime scheduling. Excluding CLAUSE-6 entirely leaves this v0 restriction outside the promised inventory.

   **Smallest fix:** inventory its compile-time portion under CLAUSE-6 and retain the runtime scheduling portion in the exclusions, following the existing PAR-3 split.

## Non-blocking

- **FX expectations remain assertions awaiting validation.** Changing `FX0606` to another syntactically valid FX code still yields AWAITS CHECKER; removing it can yield PASS. This follows the current implementation, but the bead’s broad “wrong expected verdict exits nonzero” acceptance needs qualification: only rustc expectations are enforced today. State that limitation explicitly.
- [conformance/README.md:37](/home/lucca/pjs/lab/tide-fx-gcs.1/conformance/README.md:37) places `Shared::with` within §1–§5, although its rules are in §6.5. Correct the area description.

## Reasoning

The five seeds align with LOCK-1, LOCK-4, LOCK-5, ADMIT-1, and PAR-4: accepted locking, non-Send payload rejection, non-Pure payload rejection, forbidden core atomics, and non-Sync parallel elements. They cover accepting, rustc-rejecting, and checker-pending expectations. A FAIL requires a mutation or malformed case.

The ID scheme preserves existing IDs, assigns structural `Pure` to chapter 02, and gives future rule groups identifiable owners. Spot-checked citations match the chapters. Runtime and later-tier exclusions are generally appropriate, apart from CLAUSE-6 above. All blocking fixes concern the current v0 corpus; none requires changing only v1 or Later text for fx-djp.

Not checked: a successful clean build, complete seed run, or end-to-end mutation test; filesystem restrictions prevented these.
Risk: FX codes and locations remain unverified until a Tide checker consumes the corpus.