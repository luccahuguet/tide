Model: gpt-6.1-sol · Effort: high · Date: 2026-11-13

Verdict: request_changes

1. **blocking — FX0803 rejects a scalar copied out by a parameter pattern.** Evidence: `crates/tide-admit/src/walk/contracts.rs:161` and `crates/tide-admit/src/walk/contracts.rs:168`. The check takes the outer argument's `&mut` type and associates every binding beneath its pattern with that reference. A reference pattern can instead copy an independent value:

   ```rust
   #[ensures(x == x)]
   pub fn f<'a>(&mut x: &'a mut u32, _y: &'a u32) { let _: u32 = x; }
   ```

   Here `x` is a `u32`, not the mutable reference or a borrow of its referent. Its use cannot observe a prophetic borrow. Reference §1 preserves Rust parameter patterns, and §10.1's restriction on naming a mutable reference does not prohibit this copied scalar. The pinned raw rustc accepts the production macro's lowering (exit 0); this candidate rejects it with FX0803 on line 1 (exit 1). The equivalent shared-reference pattern also passes the driver. **Fix wanted:** associate postcondition uses with what the parameter pattern actually binds; exclude bindings copied out of the referent while retaining checks for bindings that retain the mutable borrow. Add this accepted case alongside the existing rejection for an ordinary `x: &'a mut u32` binding.

2. **blocking — FX0803 misses mutable-reference aliases named outside `old`.** Evidence: `crates/tide-admit/src/walk/contracts.rs:94`–`115`, with the existing lowering at `crates/tide-macros/src/contract.rs:307`–`364`. Runtime postcondition entry snapshots use `__contract::entry`, which this visitor does not recognize. For a type alias, the macro moves the parameter use into that entry helper and replaces the postcondition use with a generated snapshot binding:

   ```rust
   type M<'a> = &'a mut u32;
   #[ensures(*x == *y)]
   pub fn shared<'a>(x: M<'a>, y: &'a u32) { *x = *y; }
   ```

   This is the same forbidden shared-origin postcondition as the explicit-reference fixture. The candidate emits only the existing unsupported-alias E0277, with no FX0803. A second probe mixing `old(*x)` and bare `*x` has the same omission; the alias case using only `old(*x)` is accepted. The documented runtime alias limitation is legitimate, but it does not supply the FX code this bead must report. **Fix wanted:** recognize the parameter uses moved into runtime postcondition entry snapshots, using their resolved mutable-reference type, as the implementation already does for const entry snapshots. Preserve the existing unsupported-lowering diagnostic; this finding does not require adding runtime alias support. Add a violating alias case requiring FX0803 and an `old`-only accepted twin.

3. **blocking — an early FX0913 suppresses independent late FX0913 diagnostics.** Evidence: `crates/tide-admit/src/walk/modules.rs:58`, `crates/tide-admit/src/invocation.rs:69`, and `crates/tide-admit/src/invocation.rs:936`. `check_expanded` emits buffered unused-import errors before `module_rules` tests the compiler's global error state. Consequently that test also sees the driver's own earlier errors, even when all method resolution succeeds:

   ```rust
   use std::collections::BTreeMap;
   mod extension {
       pub trait Size { fn size(&self) -> u32; }
       pub struct S;
       impl S { pub fn size(&self) -> u32 { 1 } }
   }
   use extension::Size;
   pub fn measure(x: extension::S) -> u32 { x.size() }
   ```

   Both imports are unused. The candidate reports FX0913 only at line 1. Removing that import makes it report the trait import, confirming the missing diagnostic is caused by the early error, not by usage. The author's decision to avoid incomplete trait-usage results after a genuine rustc failure is sound and matches the research; the global guard implements a broader restriction. **Fix wanted:** preserve that recovery boundary while preventing the driver's early diagnostics from suppressing complete trait-usage results, for example by collecting the two halves before emitting their errors. Add a combined case requiring both lines, plus a control retaining silence for a trait actually used in method resolution.

Review basis: frozen HEAD `55af8203892c1dcbd2060043e70b6a3d83923a0a`, base `b3b19dc3ec521f25f4125cb7a9a223b6bdef21b9`, tree `4d07d4826f79072bb43dcf31f7f1f2ef3741bc93`. SHA-256 of `git diff b3b19dc..55af820`: `2f040dc55f95544385272e385e2ac48a19dd3e9b4ba877481e36080b326dfe51`. Confidence is high in the reproduced findings. The direction and ownership are appropriate; no alternative architecture or dependency is needed for approval. Fix the three findings with focused regression cases, then re-check those fixes before the Opus round.

Other review conclusions:

- All nine assigned codes use error severity. FX0910 reuses the existing rustc-label owner; FX0911 and FX0915 keep their existing implementations. The new checks use the existing diagnostics/provenance mechanisms and family modules, without a parallel rule table.
- FX0914 follows the recorded use-item graph: local resolved definitions determine named-import targets, imported modules/glob sources are module targets, public imports count, and declarations, body paths and external targets add no edges. Re-export and enum-variant probes and function-local use cycles support this. The parent/child glob cycle in the corpus matches the binding decision. No cycle finding is raised.
- The FX0915 invocation test at `crates/tide-admit/tests/invocation.rs:392` meets the explicit package-level alternative: accepted exit 0, rejected exit 1, unlocated `error: FX0915:`, and a build-script canary; it retains the `../` spelling check. Independent package probes confirmed the accepted/rejected code, severity and verdict.
- `tasks/channel_deadlock.rs` keeps its test signature and both blocked channel operations; gating its imports preserves what the `--test` build exercises. `limit_deadlock.rs` and `tcp_deadlock.rs` merely move imports into their test bodies. Removing the unused `Tasks` imports preserves the provenance cases. The `build.rs` alias change still exercises the same candidate trait through an admitted Rust-2024 import; the other fixture edits document existing unsafe blocks or mark existing violations. I found no lost test purpose in these changes.
- The native outlives environment, opaque variance filtering, resolved binding identities, and insertion provenance are justified uses of existing compiler/toolchain owners. The unsupported-alias and const-lowering errors remain existing limitations, rather than reasons to omit an assigned FX label. Existing single-purpose cases miss the interactions in findings 1–3.

Probes run:

Environment: Linux x86_64, pinned `nightly-2026-08-09` (rustc 1.99.0-nightly, `771916f90`); candidate's existing `target/debug/tide-admit` and existing pinned conformance stub/dependency artifacts. Probe sources and output directories are under `/home/lucca/.cache/eon-agents/tide-fx-u65.6-work/r1-probes/`, outside the worktree. No build or rebuild of the workspace was needed.

Every compilation ran sequentially under this queue wrapper, with a 30-second subprocess timeout per compiler invocation:

```sh
(ulimit -c 0; CARGO_BUILD_JOBS=4 timeout <120|180|300> bash /home/lucca/.cache/eon-agents/eon-overseer/heavy.sh nice -n 19 python3 <probe-harness> </dev/null)
```

The Python harness invoked the absolute candidate driver with `--tide-std <worktree>/conformance/target/admit/debug/libtide_std_stub.rlib`, one `--tide-dep-dir` for each existing `.rlib`/`.so` parent under `conformance/target/admit/debug/build`, and scratch `--out-dir` and source paths. Library metadata is the default; the first batches also supplied explicit edition/crate-type/emit/error-format flags. SAFETY tests with a grant used `--tide-grant=unsafe`; the release probe used `--tide-profile=release`. The two raw-rustc controls used `rustc +nightly-2026-08-09 --edition=2024 --crate-type=lib --emit=metadata --error-format=short --extern std=<same stub>` and corresponding `-L dependency=<dir>` arguments.

| Scratch probe(s) | Observation |
| --- | --- |
| `pattern_copy_tautology`, `pattern_copy_typed` | Raw rustc exits 0; driver exits 1 with FX0803. The body asserts that the copied binding is `u32`. Finding 1. |
| `destructured_mut`, `pattern_copy_mut_body`, `pattern_copy_control` | Mutable-reference copy patterns get FX0803; shared-reference copy control exits 0. |
| `contract_alias`, `alias_mixed_old_bare`, `alias_old_accepted` | Bare alias uses produce E0277 without FX0803; `old`-only alias exits 0. Finding 2. |
| `unused_mixed`, `unused_trait_only` | Combined case reports only the ordinary import; isolated late trait case reports its FX0913. Finding 3. |
| `unused_actual_type_error` | Actual failed method lookup produces E0599 without a spurious unused-trait diagnostic, supporting the recovery rationale. |
| `contract_nested`, `bare_required` | Nested trait required/default methods and trait-impl contracts reject with FX0802; inherent method remains silent. Bodiless-method primaries are on their declaration lines; body-bearing methods point to the macro call site. |
| `contract_nested_accepted` | Despite the scratch name, its inner function deliberately has a shared-lifetime postcondition: FX0803 is reported inside the outer invariants function. |
| `inherited_bound`, `own_bound`, `contract_self_alias` | Inherited/own signature constraints and an inherent static method produce FX0803 as applicable. |
| `contract_old_result` | `old(result)` is rejected by the existing macro owner. |
| `prophetic_release` | FX0803 still reports as an error with release clause execution disabled. |
| `cycle_reexport_defining` | Defining-module re-exports through a facade are accepted. |
| `cycle_enum`, `local_use_cycle` | Both cyclic use lines reject with FX0914, including enum-variant imports and use items inside function bodies. |
| `cycle_glob_facade`, `cycle_glob_alias` | Glob module dependencies form the expected graph cycle; the former also has an unused glob. |
| `cfg_surface` | Inactive extern-crate, use and missing-mod items are silent; exit 0. |
| `missing_nested` | Missing nested source file gets E0583 and FX0910 on its mod line; exit 1. |
| `extern_forms` | `extern crate self as me` and public renamed extern-crate forms each get FX0912; exit 1. |
| `safety_comments` | Plain SAFETY line accepted; doc comment does not qualify; undocumented user unsafe block passed through `assert!` gets FX0902; documented macro argument is accepted. |
| `safety_no_grant` | Undocumented block gets FX0901 and FX0902; documented block gets FX0901 alone. |
| `pattern_mut_ref_binding` | Additional ref-binding experiment hits existing closure-lowering E0631/E0308 plus FX0803; it is not used as an accepted-form claim. |
| `package_build_script` | Without script: exit 0. With script: exit 1 and exactly the unlocated `error: FX0915: package build scripts are not admitted`; no canary panic. |

An initial setup attempt used the nonexistent conventional `debug/deps` directory: all nine invocations exited 2 before compilation. It supplied no verdict evidence. The corrected batches used Cargo's actual artifact directories above. All probes finished before 02:30Z; none entered a prohibited window. The worktree remained clean and on the frozen candidate.

Not checked: full workspace/conformance suites (reserved for the coordinator), runtime execution of these metadata probes, and the separately assigned fx-u65.7 integration. No held-out source was opened, and no repository or Beads state was changed. The candidate must not advance as Codex-clean until findings 1–3 are resolved. DONE
