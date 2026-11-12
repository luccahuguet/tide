Model: claude-opus-5-5 · Date: 2026-11-12

Verdict: request_changes

Candidate: branch `fx-yg5.4`, HEAD `29316c6` (one commit on `b7a994a`), worktree clean. I reviewed `git diff b7a994a..29316c6` and read AGENTS.md, the bead and its decisions comment, the research report (§2, §4, §6), reference §3.6, 02 §9 and §10.9, 03 §3.2, `pure.rs`, `walk.rs` (`std_attr_made`, `fn_item`), `interface.rs`'s use of `Pure::check`, `crates.rs` (`Crates::key`), `rules.md` and `run.sh`, the conformance stub and macros crate, syn 2.0.119's fn parsers, and fx-yg5.3's candidate diff (for conflicts). I did not open `trials/e1-checked/`.

The code is sound, and the one blocking item is a text-only contradiction in design/03. Its fix needs no re-check beyond reading the sentence (AP-PROOF-001).

## Findings

1. **Blocking (text-only).** The AFREE-1 addition contradicts the paragraph directly above it.
   - Evidence: `design/03-authority-concurrency.md:1026-1029` says the prelude attribute "accepts functions and methods, including required trait methods and foreign declarations". §3.2 just above (`:1015-1016`) says "Foreign functions, declared in `extern` blocks, are never authority-free (`#[pure]` on one is `FX0511`)". `walk.rs` `fn_item` does reject them with FX0511. The macro passes foreign declarations through only so the driver can reject them.
   - Read as language text, "accepts … foreign declarations" says the opposite. The sentence also adds implementation detail (token preservation, which component owns what), and AP-CONTRACT-001 keeps that out of a contract.
   - This change edits the sentence, and AFREE-1 is v0-core text, so the finding blocks.
   - Fix: replace the added sentences with the contract only, for example: "`#[pure]` takes no arguments and goes only on a function or method, a required trait method included; on anything else, or with arguments, it is a compile error." The foreign case is already covered above, and the token-preservation detail belongs in the `tide-macros` rustdoc, where it already is.

2. **Non-blocking (landing order).** The research link in design/02 dangles unless fx-yg5.3 lands first.
   - Evidence: `design/02-memory.md:1256` links to `../research/25-casts-arena-allowlist.md#2-fx-yg54-stdarena-and-pure`. That file exists only on `fx-yg5.3`, where its heading `## 2. fx-yg5.4: \`std::arena\` and \`#[pure]\`` (line 206) yields that anchor.
   - The decisions comment already plans for fx-yg5.3 to land first.
   - Fix: land fx-yg5.3 before fx-yg5.4, as planned. If the order flips, the link stays broken until fx-yg5.3 lands. No edit is needed under the planned order.

3. **Non-blocking (rebase note, no change needed).** Conflicts with fx-yg5.3 are add/add at shared anchors, and none of them is needless.
   - Both candidates insert at the same anchors:
     - `crates/tide-std/src/root.rs`: a module after `pub mod collections;`, and a prelude line after `pub use crate::marker::Pure;`.
     - `README.md:387` and `conformance/README.md:263`: a paragraph after the float/math sentences.
     - `crates/tide-std/src/lib.rs`: the "`root.rs` (with …)" sentence near line 226, which fx-yg5.3 rewrites and this change extends one line later.
     - `crates/tide-admit/tests/build.rs`: both append at EOF.
   - Every resolution is "keep both". In `lib.rs`, fold both edits into one sentence: "`root.rs` (with `convert.rs`, `codec.rs` and `json.rs`) … `arena.rs` is also shared through `root.rs`".
   - Neither bead edits a single-holder file in a way that collides: this bead leaves `design/reference.md` untouched. The local exception covers the rest. Its condition requires re-checking what the rebase changed.

4. **Nit.** The `rules.md` ARENA-1 row breaks the table's conventions.
   - Evidence: `conformance/rules.md:84` has the section `§10.9; ref §3.6` and is decided by `runtime / checker`.
   - The column header asks for the definition's location. Other runtime rows write it as `§2.4, 03 SINK-1`.
   - The legend defines only `rustc`, `checker`, `both` and `runtime`, so `runtime / checker` is not a legend value.
   - Fix: write `§10.9, 02 ARENA-1; ref §3.6`. For the decider, either extend the legend with a runtime-plus-checker value or write `runtime` and let the codes column carry FX0517/FX0511. PURE-1 and AFREE-1 are already listed on the cases.

5. **Nit.** `#[pure]` gives the wrong message on a Rust 2024 `safe fn` foreign declaration.
   - Evidence: `crates/tide-macros/src/lib.rs:439-441` tries `ForeignItemFn`. syn 2.0.119's `ForeignItemFn::parse` (`item.rs:1973-1977`) parses through `Signature::parse`, which calls `parse_signature` with `allow_safe = false` (`item.rs:1513-1527`).
   - So `#[pure] safe fn f();` inside `unsafe extern { … }` fails every parse. The macro then reports "`#[pure]` goes on a function" instead of the driver's FX0511 foreign-function message.
   - The verdict is still a rejection, and such code needs an `unsafe` grant anyway. I found this by reading the source, not by building.
   - Fix (optional): also accept `syn::parse::<syn::ForeignItem>` returning `Verbatim` whose leading tokens are `safe fn`. Or leave it as is and note it in the rustdoc.

6. **Nit (agrees with Codex r1).** The package description is stale.
   - Evidence: `conformance/rustc/macros/Cargo.toml:6` still says it provides "#[pure] and #[trusted_pure]".
   - Fix: describe only the `#[trusted_pure]` identity shim.

## Checked and found sound (no finding)

- **Generations.** `remove` advances the generation before the slot can be reused. A slot reaching `u32::MAX` keeps `Vacant(None)` and is never pushed onto the free list, so `+= 1` cannot overflow: a generation-MAX slot is never occupied. A handle carrying generation MAX matches only a vacant slot, which yields `None`. The unit test hits the MAX-1 boundary with a non-empty free list.
- **Slot ceiling.** `insert` checks `slots.len() < u32::MAX` before narrowing, so the index is at most `u32::MAX - 1`. Retired slots count toward the limit, as documented. `insert` panics before mutating anything.
- **Stale access.** `Index` and `IndexMut` use `#[track_caller]` plus `expect`, so the panic names the caller's location. `get`, `get_mut` and `remove` return `None` for out-of-range, mismatched or vacant handles. `contains` follows `get`.
- **Disjoint access.** `get_disjoint_mut` first requires every handle to be live, which also rules out out-of-range indices. It then relies on `slice::get_disjoint_mut`, so a repeated handle gives `Err` and then `None`, and `[]` succeeds. The `unreachable!` on `Vacant` is truly unreachable after the liveness check. No `unsafe` is added.
- **Iteration order** is slot order and is documented as provisional in the rustdoc, README and 02.
- **Cross-arena handles.** Leaving them unchecked is sound. `Id`'s fields are private, it has no public constructor and no `Default`, and it cannot be retyped. So a foreign handle can only select an element of an arena the caller already holds. That gives no new authority, and borrowck-checked API access cannot alias. The documentation in `arena.rs:1-5`, 02 and README matches the binding decision.
- **The `Pure` exception.**
  - `ALWAYS_PURE` is keyed through `Crates::key`, which normalizes the stub's crate name to `tide_std`, so production and the stub agree.
  - It applies only to std-role crates and sits after the `#[trusted_pure]` arm and before `carries`, so `Arena<T>` keeps the type-argument rule.
  - `interface.rs` calls `Pure::check` before it uses `carries`, so it does not rebuild authority from `Id`'s phantom parameter.
  - The exception agrees with 02 §9's structural field rule: `u32`, `u32` and `PhantomData<fn() -> T>` are `Pure` for every `T`, so no 02 §9 edit is needed.
  - FX0511 still has one owner (`walk.rs` `fn_item`), whose logic is unchanged, and the existing `afree_pure_*` cases are re-run against production.
- **`#[pure]` macro.** It returns the original tokens, so `std_attr_made` provenance and spans survive. It tries the full-function forms (`ItemFn` with a receiver, `TraitItemFn`, `ForeignItemFn`, each needed for a different form). The prelude exports it once from shared `root.rs`; the stub now re-exports only `trusted_pure`, which is unchanged and out of scope as decided.
- **Equivalence.** The stub's separate `arena` module and `pure` shim are deleted. Both surfaces include the same `arena.rs` and the same prelude macro. The fixtures' `//@ rules:` IDs (ARENA-1, PURE-1, AFREE-1) each appear once in `rules.md`.
- **Reference §3.6** still describes what is built:
  - the API names and `get_disjoint_mut`'s shape;
  - `Id`'s traits for every `T`, `'static` when `T` is, with `Ord` and `Debug` provisional;
  - `iter` is provisional;
  - `Arena` is `Pure` when `T` is.

  `is_empty` is an unlisted Rust-convention extra. **No reference change is needed**, so nothing is requested of fx-yg5.3's hold.

## Probes

No builds ran:

- `python3 /home/lucca/.cache/eon-agents/tide-fx-iob-work/build-gate.py 600` at 14:03:07Z, `… 300` at 14:04:02Z, and `… 240` at 14:05:13Z all failed with `AssertionError: load >= 12`. `uptime` showed a load average of 19.49.
- Eon's hold starts at 14:20Z, so the review relies on reading. Codex r1's driver probes (pure method, foreign declaration, defined foreign-ABI callback, generic `Id<T>` at a `Pure` bound, `Arena<Id<Fs>>` against `Arena<Fs>`, `#[pure]` combined with `#[requires]`) already covered the runtime edges I would have tried.

Read-only checks:

- `git diff --stat b7a994a..29316c6`: 16 files, matching the report. `git status --short` is clean, and `git rev-parse HEAD` gives `29316c6244f3e3d012e0c17786c5ed5f77bd4b06`.
- `grep -c "^| <ID> " conformance/rules.md` gives 1 each for ARENA-1, PURE-1 and AFREE-1.
- syn 2.0.119 source (`~/.cargo/registry/src/*/syn-2.0.119/src/item.rs:1498-1560, 1973-1986`): `Signature::parse` and `ForeignItemFn::parse` reject `safe fn` (finding 5).
- `grep -rn "carries(\|pure.check" crates/tide-admit/src`: `interface.rs:258` checks `Pure::check` before `carries` at `:288`; no other consumer walks std arguments around `ALWAYS_PURE`.
- `git -C ../fx-yg5.3 diff --stat b7a994a..HEAD` and its hunks for the shared files (finding 3). The research heading anchor is at `fx-yg5.3:research/25-casts-arena-allowlist.md:206` (finding 2).
- `grep -n "^#" …/tide-research-yg5/report.md`: confirms that the §2 heading text gives the cited anchor.

DONE
