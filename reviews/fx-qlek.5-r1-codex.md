Model: gpt-6.1-sol (high) · Date: 2026-11-17

Verdict: request_changes

## Scope

Read-only GOVERNANCE.md Codex supervisor pass for fx-qlek.5. Reviewed `git diff fab99b3...e028284`, all changed files, and the connected admission, policy-key/export, conformance-runner, and sandbox paths. HEAD was `e0282848acb57b05689fceb5fd2846ead057dfab`; the diff SHA-256 was `200f16e1535cccdc2cfe381ae4066553a3418b39fd14c5d0d1aaad5866eaa00b`. The worktree was clean before and after inspection.

Read AGENTS.md, GOVERNANCE.md's Merge Rule, `br --no-db show fx-qlek.5`, `reviews/2026-11-17-red-team-opus.md`, and the external author report. Checked relevant D4/D7, R3, RUN-3/RUN-4, and reference contracts, plus the pinned nightly's installed Rust sources. No repository changes, issue writes, compiler invocations, cargo runs, conformance runs, or sandbox probes. `git diff --check fab99b3...e028284` passed.

What this change does: It rejects a blanket-impl form behind rust#57893, replaces the path-module glob with explicit entries, and adds admission and kernel-sandbox regression cases. The security direction is sound, but the admission changes reject legitimate programs and two sandbox observations are not sufficiently established by their tests.

## Blocking findings

1. **[P2] FX0402 rejects safe blanket extension impls and associated types unavailable to trait objects.** `crates/tide-admit/src/walk/traits.rs:137`–`140`.

   The predicate asks whether any associated type exists anywhere in the supertrait graph. It never checks whether the object can choose a conflicting value for that type. For example, this ordinary extension trait meets the new predicate and receives FX0402:

   ```rust
   pub trait Bytes: Iterator<Item = u8> {}
   impl<T: Iterator<Item = u8> + ?Sized> Bytes for T {}
   pub fn next_byte(it: &mut dyn Bytes<Item = u8>) -> Option<u8> {
       it.next()
   }
   ```

   `Bytes` is dyn-compatible, `T` need not be sized, and its supertrait `Iterator` declares `Item`. But the supertrait equality already fixes every `dyn Bytes`'s `Item` to `u8`; the blanket `Bytes` impl defines no associated type that can disagree with it. This rejects legitimate trait-object use with no transmute route. D4 and reference §5 permit ordinary traits, flat associated types, and structurally valid blanket impls.

   A second concrete false rejection is a trait whose only associated type has `where Self: Sized`, with a blanket impl retaining that condition:

   ```rust
   pub trait Describe {
       type Owned where Self: Sized;
       fn describe(&self) -> usize;
   }
   impl<T: ?Sized> Describe for T {
       type Owned = () where Self: Sized;
       fn describe(&self) -> usize { 0 }
   }
   ```

   That associated type cannot be used on the unsized trait object. The pinned compiler itself recognizes this distinction through `tcx.generics_require_sized_self(item.def_id)` in `rustc_trait_selection/src/traits/dyn_compatibility.rs`.

   **Smallest correction:** distinguish associated types that the object can actually bind incompatibly, honoring fixed supertrait equalities and `Self: Sized` item requirements. Add accepted cases for these two boundaries alongside the existing rejected forge and sized-parameter control. Update the blanket explanatory claims in README/conformance documentation and the red-team record to match the narrowed rule. These examples are static counterexamples; I did not compile them, as instructed.

2. **[P2] The explicit path list drops two stable, authority-free methods at the current pin.** `crates/tide-admit/src/policy.rs:365` and `388`.

   `std::path::Path::is_empty` and `std::path::PathBuf::into_string` have no allow entries. Both are already stable in the pinned nightly: its installed `library/std/src/path.rs:2867` and `1812` mark them stable since 1.98.0, and their bodies only inspect or convert owned path data. Thus `pub fn empty(p: &std::path::Path) -> bool { p.is_empty() }` and a function returning `p.into_string()` become FX0520 rejections after this change. Previously the path glob admitted both.

   Re-exporting the `path` module does not bypass this omission: `collect_exports` records the external module without recursively exporting its inherent methods, and `Walk::admit_def` applies the explicit policy to those methods. The reference §1 keeps Path/PathBuf's data API. The report's statement that verdicts do not change at the pin is therefore incorrect (`reviews/2026-11-17-red-team-opus.md:28`).

   **Smallest correction:** add both exact method entries and regression witnesses against the pinned driver, extending the existing production-std reference check if convenient. The stable 1.96 conformance oracle cannot serve as the positive oracle for these 1.98 methods. Correct the record's completeness statement after checking the actual pinned surface.

3. **[P2] The process-memory probe can report the expected denial without Tide's seccomp rule.** `crates/tide-admit/tests/build.rs:5021`–`5023`, `5049`–`5052`, and `5067`.

   `process_vm_readv` takes six arguments after the syscall number, but `call` supplies only five; its required zero `flags` argument is missing. Furthermore, the target is the external test process, whereas `remote` contains an address from the child. No valid remote buffer or permission to read the ancestor is arranged.

   Even if the missing flags happen to be zero, a normal unprivileged child-to-ancestor read is independently denied by Yama on a host with `ptrace_scope=1`; that is this host's setting. Removing Tide's syscall denial can therefore still produce the asserted `vmread=Err(Some(1))`. This does not establish route 39's stated seccomp attribution. The [Linux syscall documentation](https://man7.org/linux/man-pages/man2/process_vm_readv.2.html) specifies six arguments, zero flags, and a ptrace permission check; the [kernel's Yama documentation](https://docs.kernel.org/admin-guide/LSM/Yama.html) explains the descendant restriction.

   **Smallest correction:** supply an explicit sixth argument and make the underlying read valid, with an unsandboxed success control. A self-read with valid buffers suffices to prove this syscall's seccomp denial; an external-process claim needs a cooperating permitted target and its actual buffer address. Keep the record's route description consistent with what the corrected probe exercises.

4. **[P2] The dynamic-loader route passes when the loader never executes.** `crates/tide-admit/tests/build.rs:5071`–`5074` and `5080`–`5089`.

   The test only checks the earlier `RUN3B` report and the absence of `ESCAPED`. It does not check the loader exec's outcome or its supposed error opening `/bin/sh`. If `/lib64/ld-linux-x86-64.so.2` is absent, or is denied as an ungranted path on a host using a different interpreter, `execve` returns, the helper prints `LOADER Some(2)` or `LOADER Some(13)`, and every assertion still passes. The same assertion accepts an unrelated loader failure. Yet route 41 says the granted loader ran and Landlock prevented it from reading the shell.

   **Smallest correction:** exercise the interpreter actually granted for this binary and assert evidence that it started and failed specifically on the ungranted shell. At minimum, reject the post-exec `LOADER` return marker and require the expected target-specific permission diagnostic and failure outcome, with an unsandboxed control establishing the shell/loader fixture works. This tests the claimed second-stage boundary rather than merely the absence of shell output.

## Non-blocking findings

5. **The admitted crate-type route has no independent accepted witness.** `conformance/cases/authority/escape_crate_attrs.rs:5`–`6` combines the supposedly inert `crate_type` attribute with an intentionally invalid `crate_name`. This checks rejection of the combination, but cannot by itself establish route 24's admitted verdict or the effective output type. The driver's explicit crate-type argument supports the explanation on inspection. An accepted companion or existing invocation assertion would make that observation independently reproducible.

## Reasoning

The new FX0402 check is structural rather than tied to the reproducer's identifiers or its particular forged capability. `supertrait_def_ids` includes the trait itself and its transitive supertraits, and the sized test uses the impl's typing environment. Consequently, it catches the supplied bare-parameter blanket impl, inherited-associated-type variants satisfying that predicate, and direct type aliases that lower to the same parameter. It preserves sized blanket impls, traits without associated types in their supertrait graph, and ordinary concrete impls/objects. I found no demonstrated admission escape through the targeted shape. Finding 1 concerns its overly broad rejection boundary. The upstream [rust#57893 record](https://github.com/rust-lang/rust/issues/57893) supports the overlapping associated-type mechanism.

The path change correctly retains the ten Path I/O denials and `std::path::absolute`, including deref and qualified-method routes through resolved identities. Its new policy unit test would fail with the old glob; the new forge case's required FX0402 mark would fail without the trait fix. The existing negative Path cases appropriately continue to pass with the old policy because those I/O items were already denied. Finding 2 is a present-pin compatibility regression.

The new admission cases generally test the stated boundaries: the stable oracle checks the exact error-line/code set, and the driver must match every enforced FX mark. This prevents an unrelated compile error alone from standing in for an expected FX0402, FX0520, or FX0901 rejection. The function-trait, higher-ranked, and projection cases exercise rustc's lifetime/well-formedness failures; the missing panic/OS/backtrace modules and resolver method are intentionally unavailable surfaces, so their resolution errors are the relevant rejection. Termination::report remains admitted consistently with the stderr sink contract. The positive sized impl and pure path code within rejecting cases also constrain unexpected FX diagnostics, although they do not cover the false rejections above.

Inspection of the unchanged sandbox confirms the claimed fork/clone, clone3, io_uring, ptrace, memfd, namespace, socketpair, and limit rules exist. Landlock is installed before exec, so it covers constructors, and the canary has a valid created file. Exact errno assertions offer useful regression coverage, but an errno alone cannot identify the denying layer when the underlying operation is independently invalid or forbidden; findings 3 and 4 are concrete instances. This review establishes no new open sandbox escape.

README, D7, 02 R3, and 03 RUN-4 correctly retain the limitation that other rustc soundness bugs remain and distinguish admission from the test-child sandbox. Their named #57893 closure is supported for the targeted shape; the blanket overlap explanation needs the precision in finding 1. The red-team record also needs the corrections in findings 2–4 before its completeness and attribution claims can be relied on.

Confidence: high in the path omissions and test-assertion defects; medium-high in the trait-bound counterexamples, based on source tracing without execution. The sequential Opus pass remains required after this pass reports the corrected candidate clean. No override recorded.

Not checked: builds, full suites, or live counterexamples, per the review brief; the coordinator owns those checks. Risk: this candidate rejects valid programs and overstates proof of two RUN-3 routes.

DONE
