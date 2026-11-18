Model: claude-opus-5-5 · Date: 2026-11-18 · Candidate 4903049

Bead fx-kw8.24, "Explain structural Pure failures with one witness path". This is the Opus converge round, which reports and does not fix. It ran after Codex's round 1 (request_changes on e9c66af) and Codex's re-check (approve on 4903049).

Scope read: `git diff main...4903049` (`crates/tide-admit/src/pure.rs`, `tests/driver.rs`, `tests/build.rs`), the whole of `pure.rs` at 4903049, every `NotPure` consumer in `walk.rs` and `interface.rs`, `Crates::is_std`, the bead, AGENTS.md, GOVERNANCE.md, the builder's report, both Codex files, the builder's round-1 logs, and grep sweeps of `design/`, `docs/`, `conformance/`, `trials/` (excluding `trials/e1-checked/`, which was never opened), and the tests for pinned `Pure` message text. No builds or tests were run.

## Verdict

**approve** (confidence: high)

## Blocking findings

None.

## Non-blocking findings

1. **The foreign stop is redundant when it is the only step** (`pure.rs:357-360`).
   - Case: the message reads "`Vault` is not `Pure`: it reaches `Net`, which is never `Pure`, via `Vault`". The obligation's type is the foreign type, so the suffix repeats the root.
   - It is harmless, deterministic and pinned by `tests/build.rs:816`.
   - Optional fix: emit no suffix when the rendered path is only a foreign stop on the root. This is a nit.

2. **Two properties are checked only by reading the source.** The paired-library regression (`tests/build.rs:787`) uses a non-generic struct, so no test exercises either of these:
   - (a) the foreign stop prints the ADT and not the enum variant (`adt` against `owner`);
   - (b) the foreign stop prints no generic arguments.

   `def_path_str` prints no arguments, which Codex confirmed in the pinned rustc source and I agree with. A variant name of a public foreign enum is public anyway, so (a) does not leak anything. The risk is low. A one-line `pub enum Sealed<T> { Hidden(T, Net) }` case in the same fixture would pin both.

3. **Out of scope, and older than this change: the terminal `part` can name a foreign crate's private type.**
   - Case: a paired library has `pub struct Vault { p: *const Inner }` with a private `Inner`.
   - The message reads "... it reaches `*const Inner`, which is a raw pointer, via `Vault`".
   - On main the same text appears without the suffix. This change neither adds nor widens it: the new suffix stops at `Vault`.
   - I report it only. I filed no bead, because builders and reviewers file none here. The coordinator can decide whether it needs one.

4. **Simplicity nits** (`pure.rs:273-284`):
   - `#[derive(Clone, Copy)]` on `Step` is unneeded. `match *step` binds only `Copy` fields, which compiles without the derive.
   - `Step::Field.adt` could go: test `owner.is_local()` and print `name(owner)` at the foreign stop. That would show a public enum's variant, which is public.

   Both are optional. The current form is the conservative one that tide-lead chose and Codex re-checked.

5. **Proof record.** `target/work/r1-workspace.log` and `r1-focused.log` do not record their revision; only report.md ties them to 4903049. The worktree's HEAD is 4903049 and the tree is clean, so I accept the attribution. Future logs should print `git rev-parse HEAD`. This is a nit.

## Reasoning

### Lens 1: policy is unchanged

- **ADT walk order.** The ADT arm now iterates `def.variants()` and then each `variant.fields`. That is exactly `AdtDef::all_fields`'s `flat_map`, as Codex verified in the pinned rustc. `try_for_each` still exits at the first failure, so the field order and the early exit are unchanged.
- **Tuple walk order.** The tuple arm adds `enumerate()` and changes nothing else.
- **What `map_err` touches.** It runs only on `Err` and changes only the error payload.
- **The `assumed` set.** It has the same insert point and lifetime. It is never iterated, so it cannot make the witness order unstable.
- **Unchanged parts of the walk.** Normalization (the alias arm and `field_ty`), the trusted and std arms, `MAX_TYPE_DEPTH`, and `TooDeep` are untouched.
- **`TooDeep` rendering.** `TooDeep` still returns its fixed text before any path is rendered, so it never claims a witness. `too_deep()` is unchanged, so `interface.rs:258-261` behaves the same.
- **Codes and locations.** No new code exists. Every caller (`walk.rs:873, 959, 990, 1094, 1185, 1267`) passes `e.message(..)` into its unchanged code, span and prefix. The new driver lines keep `TD0517` at the obligation spans, and the builder's red run showed the same lines without the suffix.
- **Other consumers.** `seen_pure` (`walk.rs:868`) dedupes before the message is built, and exported interface JSON never reads `message`.

### Lens 2: information exposure

- **Opaque types.** The walk still runs in the non-revealing environment. A tuple step prints the tuple `Ty`, which shows an opaque type as `impl Bounds`, not its hidden type. The `Wrap<impl Sized>` fixture would stop rejecting if the hidden `u8` were revealed.
- **Foreign types.** Rendering stops at the first `Field` step whose ADT is not local. It prints `def_path_str(adt)` under forced trimming, with no arguments and no field names, and drops every later step, tuple displays included. The paired-library test asserts that neither `secret_inner`, `Inner`, `hidden_pair` nor `(u8, Net)` appears.
- **Steps before the stop.** Tuple steps before the stop render types that are parts of the obligation's own type after substituting local field types. The root display (unchanged) already shows the same information.
- **Private fields of local types across modules.** This is not a concern within one crate. The diagnostic goes to the crate's own author, who owns every module. PURE-1 (`pure.rs` module doc) is defined over private fields "private ones included", so the private field is exactly the fact the author needs to act on. rustc also names same-crate private fields in its own errors.
- **Paired libraries and dependencies.** These are separate crates (`is_local()` is false), so the foreign stop covers them.
- **`ty::tls::with` with `def_path_str`.** It reads the same compiler context that `Ty`'s `Display` already uses in `show`. It neither normalizes nor reveals anything. Under forced trimming it prints only the item name, or `Enum::Variant` for a variant.

### Lens 3: witness correctness

- **It is a real path.** Each step is pushed while the failure unwinds through a frame that actually descended into that field or component, then reversed once for display. The coinductive `Ok` on a revisit can never be the failing branch, so the path is the actual DFS stack and is finite.
- **Recursion and generics.**
  - `Ring` gives `Ring.tasks`, and `List` is accepted.
  - Deep generic recursion still gives `TooDeep` with no path.
  - Generic owners print without arguments (`W.0`).
  - A projection field normalizes before descending, and the next owner is in the normalized type, so the chain stays valid.
- **Unnamed hops.** References, arrays, std containers, `#[trusted_pure]` arguments and closure captures add no step, as the type doc comment says. A closure root therefore starts its path at the captured type's fields. The bead puts closure-capture naming out of scope.
- **Determinism.** The path comes only from declaration-order iteration and `DefId` and `Symbol` values.
- **Readability.** `Payload.client -> Client.socket`, `Msg::B.1` and `(Client, u32).0` match the bead's example form. Nested tuples repeat their inner tuple types, a cost the builder accepted.

### Lens 4: consumers

- **Tests.** The only pinned full message that changes is the `W<Clock>` line in `driver.rs:713`, and it is updated. The other pinned texts are root failures (`` `Clock` is never `Pure` ``), whose path is empty or not rendered.
- **Owner guide.** The TD0603 quote in `docs/owner-guide/04-tasks-and-shared.md:99` comes from `examples/habits/src/lib.rs:18-22`. That closure captures `fs: &Fs` directly (capture, then `&`, then `Fs`), which adds no step, so the quote stays accurate.
- **Other sources.** `conformance/` pins codes only. `design/` and `README.md` do not quote the full text.
- **Workspace run.** The builder's 4903049 workspace run passed 16 binaries with 0 failures, which would catch any other pin in the workspace.

### Lens 5: tests

`pure_failures_name_one_witness_path` compares the full ordered TD lines with `assert_eq!`. A missing, extra, reordered or misnamed step fails it, as would printing the ADT instead of the variant (`Msg.1`) or a wrong tuple index. It also fails on a changed span or code, or on a spurious `List` rejection.

It covers:
- a struct field, two levels deep;
- an enum variant, with a preceding variant that is `Pure`;
- tuple components, nested;
- the opaque stop;
- recursion, accepted and rejected.

`pure_witness_stops_at_another_crates_type` covers the foreign stop, both bare and behind a local tuple step. It anchors on line ends and checks that the private names are absent.

I found no wrong path that both tests would accept. The gaps are in finding 2.

### Lens 6: simplicity

The change uses one payload field, one two-case enum and a `map_err` at the two recursive sites. It needs no second traversal, no extractor reuse, no `diag.rs` change and no dependency, and a successful walk allocates nothing. The only deletions I can name are the optional ones in finding 4. I found no leftover callers, fixtures or exports.

DONE
