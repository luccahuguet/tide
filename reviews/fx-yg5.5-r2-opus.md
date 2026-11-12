Model: claude-opus-5-5 · Date: 2026-11-12

Verdict: request_changes

Reviewed the frozen candidate `240aea6` on `42dd813` (`git diff 42dd813..240aea6`, 5 files, +652). I read AGENTS.md, both beads with their binding comments, research/25 §3, reference §1, §2.1, §3–§5 and §6.9, the touched files, `walk::admit_def`/`admit_impl`/`check_expansions`, `walk/surface.rs`, tide-std's `root.rs`/`lib.rs`, the conformance fixture std, and fx-u65.5's candidate `b3eda12`. I did not open `trials/e1-checked/`.

The candidate is sound and close to done. Its table, its deny rows, and its three allows are correct. One finding blocks: a family §1 keeps, the scalar API, still gets FX0520 through a return type, and the fix is a one-line allow plus one table row in files this change edits.

## Findings

1. **Blocking (small): `u8::escape_ascii` is still rejected.** §1 says scalars "keep their Rust API" (`design/reference.md:35`). `b'a'.escape_ascii()` passes its method key (`core::num::<impl u8>::escape_ascii` matches `core::num::*`). Its return type `core::ascii::EscapeDefault` matches no rule, so it gets `FX0520` (probe 1, `lib.rs:2:13`). Its `Display` impl would fail the same way, because `admit_impl` needs the head type admitted. Production std behaves the same: `root.rs:18-21` re-exports no `ascii` module, so nothing exempts the type. Of the ~25 kept-API forms I probed, this was the only one rejected. It is rare in practice, but the bead's Acceptance says every reference-kept item is admitted, and its scope is to add exactly this kind of allow.
   Fix: add `allow("core::ascii::*")` to the `// ---- core` block of `crates/tide-admit/src/policy.rs` (around line 230). No earlier deny matches `core::ascii`. Also add a row to `tests/support/reference_allowlist.rs`, e.g. `("section_1_u8", &["core::ascii::EscapeDefault"], "assert_eq!(b'\\n'.escape_ascii().to_string(), \"\\\\n\");")`. This is the cheapest falsifier and runs at all three boundaries.

2. **Non-blocking (high for E1, separate bead): production std has no `Arc` or `sync::Weak`, and no `dbg!`.** §1 promises `Arc` (created with `&Tasks`, minus count observers) and `Weak` (`design/reference.md:36-37`). §2.1 promises `dbg!` (`design/reference.md:58-59`). Production `crates/tide-std/src/sync.rs` has neither `Arc` nor `Weak`. `crates/tide-std/src/lib.rs:219-222` lists "the `dbg` and `trace` sinks" and "other `sync` items" as not provided yet. Only the conformance fixture declares them, with `unimplemented!()` bodies (`conformance/rustc/std/src/lib.rs:134-160`, and `dbg!` at `:32-42`).
   - **E1 severity:** a Tide-arm writer who follows §1 to `use std::sync::Arc`, or who reaches for `dbg!`, gets E0432 or "cannot find macro" against production std. That reads as a language failure, not a slip. This bead's Acceptance won't catch it: "every … solution that compiles reports no FX0520" excludes non-compiling solutions, and the E1 harness only greps for FX0520. So this is a real threat to E1's validity, of the same class as the fx-yg5.1–.4 std-surface beads.
   - **Where it belongs:** not here. It is new std semantics (`&Tasks` creation, GLOBAL-6's count-observer removal, the dbg sink), which this bead's "no new language semantics" scope rules out. Admitting `alloc::sync` instead would break the retained restriction.
   - **Fix wanted:** a separate implementation bead, e.g. "Production std provides `Arc`/`sync::Weak` (GLOBAL-6) and `dbg!` (SINK-8)", that blocks fx-ic2. Filing it is the coordinator's call under the standing no-new-beads limit; until it exists, record it on fx-ic2 as an E1 blocker.

3. **Non-blocking: `conformance/cases/authority/admit_reference_sync.rs` duplicates `global_arc_accept.rs`.** The existing `global_arc_accept.rs:1-11` already admits `Arc::new(_, tasks)`, `make_mut`, `downgrade(_, tasks)` and `upgrade()` against the same fixture. The new case repeats those four operations (`admit_reference_sync.rs:7-13`). Its line-2 claim that `alloc::sync` stays denied is not something this case tests; `global_arc_count.rs`/`_payload.rs` and the policy deny do. AP-TEST-001 says to delete duplicate proof.
   Fix: drop the new case and leave the Arc proof to the existing cases and to finding 2's bead. If finding 1 opens another round anyway, include this.

4. **Nit: the three new allows are reached only when Rust's std is the injected std (the driver tests).** In programs and in conformance, Tide's std re-exports `Cursor` and `SeekFrom` (`crates/tide-std/src/io/mod.rs:4-5`), so `walk::admit_def` exempts their type DefIds (`crates/tide-admit/src/walk.rs:277-282`). Tide's `panic!` is `tide_macros::panic` (`root.rs:34`), which expands to `::core::panic!` (`crates/tide-macros/src/lib.rs:176`) and is admitted by `core::macros::*`. So the re-export exemption stays the one owner in programs, and the rules add no program-visible admission. That is fine. The new comment at `policy.rs:277` ("Rust's `panic!` entry expands to core's panicking support") reads as if programs needed it, and the type allows at `:222` and `:224` sit under a comment saying std re-exports them, which makes them look redundant.
   Fix: one comment line saying these keys serve the Rust-std check (`tests/driver.rs`), since Tide's std re-exports or replaces them in programs. The same mode still rejects `println!` and `dbg!` (`std::io::stdio::_print`/`_eprint`, probe 1). Sinks are Tide-owned, so that is correct, but the comment could say it.

5. **Nit: the `pointer-address` deny row does not isolate `addr()`.** In `tests/driver.rs:135-138`, `(&0u8 as *const u8).addr()` is satisfied by the `*const T` cast on the same line. The row therefore re-proves the `raw-pointer` row and never checks `core::ptr::const_ptr::<impl *const T>::addr`. The deny holds anyway, because `core::ptr::*` is `granted` and unlisted core is rejected. The row is just weaker than its name.
   Fix: expect the `addr` method key, isolated as decision 4 isolated the Path methods. Or drop the row as a duplicate.

6. **Nit: some proof in `reference_kept_families_build_and_execute_against_production_std` is duplicated or bundled.** Its Arena/`truncate`/`round_from`/`try_from_float` lines (`tests/build.rs:44-49`) re-prove what its own comment says fx-yg5.3/.4 already own and test. The HashMap/HashSet/io/clock lines are §1 families and belong here.
   - The test also carries an unrelated denial (`last_os_error`) under a name that says "build and execute".
   - The hand-typed witness keys and the `reference_kept_families_have_allow_rules` unit test (`policy.rs:352-369`) never check against the keys rustc actually prints. They prove a string is allowed, not that a use is. The Rust-std driver run is the real proof, and the pin-bump detector is that run together with the production build.
   - Keeping the witnesses as fast documentation is acceptable. The smaller sufficient proof is the driver and build runs alone.

7. **Non-blocking, report only (fx-djp):** probe 1 shows these reference-silent forms rejected with FX0520 and no help:
   - the deprecated module constants `std::f64::MAX` → `core::f64::MAX`, and `std::u32::MAX` → `core::legacy_int_modules::u32::MAX`. The latter is unresolvable in production anyway, since Tide re-exports no `std::u32`;
   - `core::hint::black_box`;
   - `f64::is_infinite`, `clamp` and `powi`, as decision 3 already records.

   A help that points the legacy constants to `f64::MAX` would save a Rust writer a round trip. The reference is silent on all of these, so per decision 3 they stay as they are.

## Answers to the brief

- **Completeness:** my own list, from §1, §2.1, §3–§5 and §6.9's allowed side, matches the table's 42 families. The exceptions are Tide-owned items, covered in build.rs (HashMap/HashSet, io traits and types, Instant/SystemTime), and Arc/Weak (finding 2). I probed the core/alloc forms a Rust writer reaches most often:
  - `parse::<f64>`, `successors`, `Reverse`, `ControlFlow`, `NonZeroU32`, `char::from_u32`, `array::from_fn`;
  - `step_by`/`peekable`/`map_while`/`sum`, `type_name`, `from_utf8_lossy`, `rem_euclid`/`pow`/`abs_diff`, `mem::take`;
  - `Box<dyn Error>::from`, `Rc::ptr_eq`, `from_str_radix`, `join`, `to_string`.

  All are admitted. Only `core::ascii` (finding 1) is a kept family still on FX0520.
- **Breadth:** the three allows are exact keys. No earlier rule matches them and no later deny is shadowed. `core::io::*` and `std::macros::*` are not opened wholesale. Each has a `first_matching_rule_decides` assertion (`policy.rs:471-473`).
- **Denials:** covered with the right code: every §6.9 category, the §4.3 observers and math at both widths, and §2.1's `include_str!`/`env!`/`option_env!`, plus `asm!` without the grant. `include!`, `include_bytes!`, `global_asm!`, `naked_asm!` and `path::absolute` are covered by existing tests (`tests/invocation.rs:230-233,372`, `conformance/cases/authority/unsafe_global_asm.rs`, `admit_std_io_method.rs`). The `last_os_error` row is correctly backed by build.rs's exact Tide key. Finding 5 is the only weak row.
- **Ownership:** `core::task::*` stays FX0520, and the Waker row asserts it. No `core::future`/`core::pin` rule or row is touched. Tide's own re-exports still go through `walk::admit_def` (finding 4).
- **Test strength:** removing any needed allow fails the policy test for that family's witness and the Rust-std driver run. The production build run catches path or resolution gaps through Tide std. The author's `core::num::*` removal (23 diagnostics) shows the table is not vacuous. Finding 6 notes the duplicate proof.
- **Decisions:** the author's decisions 1–7 are sound. Decision 4's function-item isolation of the Path methods is correct. Decision 5's exact keys are the right breadth. Decision 3's refusal to admit `alloc::sync` is right.
- **Docs:** no README or rules.md change is needed, since ADMIT-1 and the commands are unchanged.
- **Leftovers:** the duplicate case (finding 3) and the misleading comment (finding 4).
- **Conflict with fx-u65.5 (`b3eda12`):** none beyond textual rebasing, and the hunks don't even overlap. In policy.rs, u65.5 edits the async block (~L175) and reformats the `os_str` comment (~L337), while this candidate edits L222–224 and L277. In build.rs and driver.rs, u65.5 appends at the file ends (L3662, L1016) and this candidate inserts near the tops. On semantics:
  - u65.5's `admit_impl` change propagates the inner code and help, but no row here asserts an impl key.
  - Its FX0601 globs (`core::future::*`, `core::pin::*`) and FX0602/0604/0701 denies match no key the table or the deny rows use.
  - `habit_def` runs before lookup, but only for spawn, tokio and similar items.
  - After the rebase, rerun the three table tests and `rejects_each_use_outside_the_allowlist_at_its_span`.

## Probes

The probe used the candidate's prebuilt `target/debug/tide-admit` (built 2026-11-12 20:27:53Z, two minutes before the commit; the report says that window changed whitespace only). It ran against Rust's std in lib mode, matching `tests/driver.rs`, through the queue:

```
(ulimit -c 0; CARGO_BUILD_JOBS=4 timeout 900 bash /home/lucca/.cache/eon-agents/eon-overseer/heavy.sh nice -n 19 ./target/debug/tide-admit --edition 2024 --crate-type lib --emit metadata --error-format=short --out-dir <scratch>/out <scratch>/package/lib.rs </dev/null)
```

A first attempt, with the output directory inside the package, was refused by the driver: "output and cache must be outside the candidate package". It was rerun with that fixed; the second run started at 20:52Z, after the quiet window ended at 20:41Z.

Probe 1 had 26 statements, one per line. It exited 1 with these FX lines:

```
lib.rs:2:13: error: FX0520: `core::ascii::EscapeDefault` is not admitted        (b'a'.escape_ascii().to_string())
lib.rs:3:13: error: FX0520: `core::legacy_int_modules::u32::MAX` is not admitted (std::u32::MAX)
lib.rs:5:13: error: FX0520: `core::f64::<impl f64>::clamp` is not admitted
lib.rs:6:13: error: FX0520: `std::f64::<impl f64>::powi` is not admitted
lib.rs:15:13: error: FX0520: `core::hint::black_box` is not admitted
lib.rs:21:13: error: FX0520: `core::f64::<impl f64>::is_infinite` is not admitted
lib.rs:22:13: error: FX0520: `core::f64::MAX` is not admitted                     (std::f64::MAX)
lib.rs:23:13: error: FX0520: `std::io::stdio::_eprint` is not admitted            (dbg!, Rust-std mode)
lib.rs:24:5: error: FX0520: `std::io::stdio::_print` is not admitted              (println!, Rust-std mode)
```

The other 17 statements were admitted with no FX line. These were `parse::<f64>`, `iter::successors`, `cmp::Reverse`, `ops::ControlFlow`, `NonZeroU32::new`, `char::from_u32`, `array::from_fn`, the `rev/step_by/peekable/map_while/sum` chain, `any::type_name`, `String::from_utf8_lossy`, `rem_euclid/pow/abs_diff`, `mem::take`, `Box<dyn Error>::from`, `Rc::ptr_eq`, `i32::from_str_radix`, `split/map/collect/join`, and `u64::to_string`.

No full suites, cargo builds or production-std builds were run. `git diff --check` was not rerun; Codex r1 recorded it passing.

DONE
