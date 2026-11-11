Model: claude-opus-5-5 · Date: 2026-11-11

Verdict: request_changes

Candidate: `b0c3854` on base `10991bd`. The SHA-256 of `git diff 10991bd b0c3854` is `0cd6e041…45d1dc` and matches the brief. The tree is clean. The review was read-only. It ran four focused probes with the candidate's `target/debug/tide-admit`, under `nice -n 19` and `CARGO_BUILD_JOBS=4`, with load about 6, 12 GB available and no quiet window. The probe sources are in this session's scratchpad. `trials/e1-checked/` was not opened.

Overall: the architecture is right. rustc keeps every borrow verdict. The walk records sites and evidence (`walk/surface.rs`), one emitter wrapper matches and names them (`diag/labels.rs`), and a label never decides acceptance. The E0373 split and the certain-chain rule for FX0203 hold up: I found no false FX0203. FX0102, FX0103, FX0201, FX0202 (unresolved form) and FX0301 match the reference's meaning and severity. I checked the §4.2 cast table row by row against `lossless_cast`, including addresses, `usize`/`isize`, `char`, the enum discriminant ranges and the anonymous-const positions. One blocking problem remains: the other user of the label layer, FX0210, breaks the policy this change states for the layer, and the corpus now asserts those false labels.

## Findings

1. **Blocking: FX0210 attaches self-borrow labels to errors that are not self-borrows, which contradicts the never-false policy stated in the same paragraph.**

   Evidence:
   - Matching: `crates/tide-admit/src/diag/labels.rs:117-125` labels any E0505, E0515 or E0597 whose primary span lies inside a struct literal of a local type with lifetime parameters. Recording is at `walk/surface.rs:176-190`.
   - Policy: `conformance/README.md:80-84` says "The label layer prefers precision over recall: every label … must never be false". `labels.rs:22` says "supplementary explanations must never be false".
   - Reference meaning: §3.6 (`design/reference.md:214`) defines FX0210 as "A struct cannot borrow from its own field". The label's advice is "keep the owner and view apart or store indices".
   - The corpus asserts FX0210 labels that are false:
     - `conformance/cases/memory/drop_dc_reject.rs:16-17`: a DROP-1 drop-check failure, now marked `E0597 FX0210` and tagged `REF-0210`. The comment says the heuristic "also labels" it.
     - `conformance/cases/memory/provenance_matrix_0597_reject.rs:13-16`, the four `field_*` rows: the cause is a closure capture escaping into `spawn_isolated`. FX0203 is withheld as uncertain, but FX0210 fires. The writer gets the wrong explanation instead of none.
   - Probes with the candidate binary:
     - `pub struct View<'a>{pub s:&'a str} pub fn make<'a>()->View<'a>{let t=String::from("x"); View{s:&t}}` gives `E0515` plus `FX0210`. This is a return of a view into a local, not a self-borrow.
     - `pub fn keep(v:&mut Vec<View<'static>>){let t=String::from("x"); v.push(View{s:&t});}` gives `E0597` plus `FX0210`. The `'static` requirement comes from the Vec. "Keep the owner and view apart" does not apply.

   Why it blocks: this change edits all of the text a fix touches (labels.rs, surface.rs, conformance/README, the two fixtures). Leaving it as is also lands fixtures that would make a later precise rule look like a regression. The fx-u65.2 decision that adopted C4 ("a heuristic; say so") came before the r2 decision that made the label layer precision over recall. The current text claims both.

   Fix wanted (recommended): make FX0210 certain-only, in the same way FX0203 now is:
   - Record a literal only when it moves a whole local `x` into one field and also borrows `x` in another field of the same literal. Reuse the certain steps the FX0203 chain already uses: `&x` or `&mut x`, or a receiver-tied borrow method on `x`; a projection's root can count here because the root itself is what gets moved.
   - Label only E0505, E0515 or E0597 inside that literal.
   - Remove `FX0210` and the `REF-0210` tag from `drop_dc_reject.rs:17`, and remove `FX0210` from the four matrix `field_*` rows.
   - Add the two probes above as no-label regressions. rustc's error stays; there is no FX mark.
   - `self_borrow_reject.rs` (`Doc { word: &text, text }`) still gets FX0210, so Acceptance still holds.

   Alternative if the coordinator keeps the heuristic: scope the policy sentence in `conformance/README.md` and `labels.rs` to FX0203. State there that FX0210 also labels drop-check, returned-view and other non-self-borrow errors inside such literals. Also suppress FX0210 at least on errors where an FX0203 owned-closure site matches the same diagnostic. I don't recommend this alternative: it leaves a code whose `tide explain` text describes a different problem.

2. **Non-blocking: research/27 does not record the label-layer decisions that later slices will reuse. It also has two leftovers.**

   Evidence:
   - `research/27-fx-rule-enforcement.md:110` (C1) still describes FX0203 as "region and borrow errors … only inside recorded candidate sites", which is containment. fx-u65.3 will reuse this layer for FX0402 (E0119, no site test) and FX0407 (recursion limit), and its bead says it cites research/27. Nothing there names the precision-over-recall rule, the E0373 rustc-span evidence, or the certain-chain requirement.
   - Leftovers: the file's last line is a stray `DONE` (`:186`), the agent's own sign-off. Line 2 says "Drafted … on 2026-11-11", but line 4 says "Research report, 2026-11-10".

   Fix: add a two- or three-line note under C1, "Adopted with changes (fx-u65.2 r2/r3)", that states the policy and points to `conformance/README.md`. Delete the trailing `DONE` and reconcile the dates. All of this is in text this change adds.

3. **Non-blocking (seam for fx-u65.3): the emitter's per-code rules are spread across the code and not table-driven.**

   Evidence:
   - `RustcLabels::record(code, span)` (`labels.rs:82`) accepts any `Code`, but the emitter honours only `Code::Fx0210` (`labels.rs:118`). Recording another code does nothing, silently.
   - The rustc-code mapping for FX0210 is inlined in `emit_diagnostic`.
   - Messages come from a `match` that ends in `_ => unreachable!()` inside an `Emitter` (`labels.rs:139-147`). A missed arm panics during diagnostic emission, which is an ICE.

   Fix: when .3 adds FX0402 and FX0407, keep one small table in labels.rs, one row per label: code, rustc codes or wording, whether it needs a site, and message. Have `record` take only codes that have a row. This change does not need it, since it has one literal-site code, but it would keep .3 from re-deriving the shape.

4. **Non-blocking (pre-existing, outside this diff): the reference says more about FX0202 than its owner chapter and the driver do.**

   Evidence:
   - `design/reference.md:154-156` says elision "that ties to the wrong input, is `FX0202`".
   - 02 LIFE-4 (`design/02-memory.md:235`) has only the unresolved case, and research/27 C3 recorded the gap.
   - The driver correctly gives no FX0202 for wrong-input elision (`tests/invocation.rs`, `wrong-input-elision`). `conformance/README.md` says so.

   E1 writers read the reference, so they are told to expect a code they won't get. rustc still rejects the code, so this is a teaching gap, not a soundness one. Fix: the holder of `reference.md` aligns §3.2 with LIFE-4 ("…cannot be resolved is `FX0202`; elision that ties to the wrong input is rustc's lifetime error"). Record it on fx-djp or the next reference bead. It does not block this change, which doesn't edit the reference.

5. **Nit: the label layer is coupled to the pinned rustc, and the pin note doesn't say so.**

   Evidence:
   - `labels.rs:58-66,114` match on the exact message "lifetime may not live long enough" and on a `'static` substring in labels and child notes.
   - `labels.rs:92-96` builds rustc's emitter itself: `AnnotateSnippetEmitter`, `ColorConfig::Never`, short. That fits the pin, which has no HumanEmitter and no `wrap_emitter`; I checked the `rustc_errors` rmeta. It is safe only because the driver fixes `--error-format=short` and accepts no colour flags. On a compiler bump, drift loses labels and never adds false ones, and the FX0203 conformance marks would FAIL, so the corpus is the tripwire.

   Fix: extend the `rust-toolchain.toml` pin comment ("The allowlist … is reviewed whenever this pin moves") to cover the label layer's wording and span matching.

6. **Nit: `Code::ALL`'s doc comment is no longer accurate.** `crates/tide-admit/src/diag.rs:74` says "Every code this driver decides", but FX0203 and FX0210 are labels on verdicts rustc decides. Suggested wording: "Every code this driver reports (its own verdicts and its labels on rustc's)".

7. **Nit: FX0201 points at the whole trait item.** `walk/surface.rs:129-131` reports at `item.span`. The lifetime parameter's span, `generics.params[i].span`, would point the short-format column at `'a`, the thing to remove.

8. **Note for fx-u65.5 (no change here):** FX0103's carve-out list matches by name only (`walk/surface.rs:73-87`), so a `select!` from any crate gets no FX0103. fx-u65.5's FX0605 must therefore catch every user-written `select!`, not just std's. The same holds for `thread_local`/`lazy_static` with FX0502, which already has an owner.

Not checked: full suites (the author's `workspace-r4.log` and `conformance-r4.log` stand for `b0c3854`), and parts of the driver this change doesn't touch.

DONE
