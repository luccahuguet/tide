Model: claude-opus-5-5 (Opus 5.5) · Date: 2026-11-13

## Verdict

**Changes requested. The only blocking fixes are to planning state; no design text has to change.**

Candidate: `1732aca`, `git diff HEAD^..HEAD` (three files). The worktree was clean, and the review was read-only with no builds.

- **CODEC-1/2 contracts are sound.** I checked them against the v0 engine (`crates/tide-macros/src/codec.rs` `attrs`, `fields`, `input`, `decode_fields`; `crates/tide-std/src/codec.rs` `next_field`, `field`, `skip_value`, `variant_begin`, `variant_end`). They don't conflict with v0:
  - Enum `rename_all` already renames only variants, since variant fields get `fields(.., None)`, so CODEC-2's "does not rename the variant's fields" matches.
  - Skipped keys already count as unknown, so CODEC-1's "a skipped payload field adds no allowed key" matches.
  - The external wrapper already rejects a second key (`variant_end`). So "other variant shapes keep their existing decoding rules" leaves no strictness gap at the external wrapper.
  - Tag-anywhere, duplicate-tag rejection, loose-unit acceptance of extras and strict-unit closure all match serde's internally tagged behavior where serde defines it. I found no clause pair that an implementer could both satisfy and still produce surprising behavior, except the small unspecified edges in finding 6.
- **Tiering is correct.** v1 is listed in the tier row (01:84), D6's subsection and the companion's v1 list. `reference.md` is unchanged, so no v1 rule leaks into the v0 teaching surface. The companion's "not yet tiered" sentence now agrees with D6.
- **Eon evidence is confirmed.** I checked it at `7d1f921` with `git show`:
  - `eon-manifest/src/lib.rs:70-75` has the `tag = "kind"` Source enum, and `parse` is at `:205-209`.
  - It is the **only** `serde(tag ...)` use in Eon at that revision, so §10's "the one internally tagged JSON shape Eon uses" holds.
  - `deny_unknown_fields` is widespread there, including the container `default` combination in `slots.rs:49,55`.
  - The web claims are labeled, and Codex spot-checked them. Inference is labeled as inference.
- **Admission passes.** A is a bounded extension of an existing owner, with no new derive, dependency or public API. See finding 5 for one under-stated answer.
- **Follow-up beads block.** fx-1p2i's Acceptance would not falsify part of CODEC-1, and its prerequisites don't match its stated order. See findings 1 and 2. Both fixes are planning-only `br` edits by the coordinator. I re-check only those two.

## Findings

1. **Blocking (follow-up acceptance does not falsify the contract). Bead fx-1p2i, Acceptance; also fx-tf7d, Acceptance.**
   - fx-1p2i's Acceptance covers only decode behavior. It omits CODEC-1's compile-time clauses at `design/01-design.md:459-464`:
     - invalid placement: unit, tuple or newtype struct, variant, field;
     - a repeated key, or one written with a value;
     - acceptance when only `Encode` or only `Decode` is derived;
     - "the key has no effect on `Encode`" (01:444).
   - So an implementation that silently accepts `#[codec(deny_unknown_fields)]` on a field, rejects it on an `Encode`-only type, or alters encoding still meets Acceptance.
   - "Errors are precise" names no observation.
   - Neither bead requires D0's trace: CODEC-1/CODEC-2 rows in `conformance/rules.md`, and the `CODEC` prefix registered in `conformance/README.md`'s prefix table (README:369-371 says each prefix belongs to an owner that defines it).
   - fx-tf7d says "Each falsifier in research/29's §6 table passes". The §6 table lists *forbidden* observations, so "passes" reads backwards.

   **Fix:**
   - Add to fx-1p2i's Acceptance: derive-time rejection of each invalid placement and of a repeated or valued key, in both derives; acceptance on `Encode`-only and `Decode`-only types; byte-identical `Encode` output with and without the key.
   - Replace "Errors are precise" with "each rejection is a `json::Error` at the offending key's absolute line and column".
   - Add "CODEC-1 is traced in `conformance/rules.md` and `CODEC` is registered in the README prefix table" to fx-1p2i, and the CODEC-2 row to fx-tf7d.
   - Reword fx-tf7d to "none of the forbidden observations in research/29 §6 occurs, and the first row's encodings hold".

2. **Blocking (prerequisites not honest, AP-PLAN-001). Bead fx-1p2i, Dependencies; `research/29-strict-tagged-codecs.md:366-367`.**
   - The bead's Order section and §10 say it starts after E1 (fx-ic2) *and* the first demo (fx-nkv).
   - Its dependencies list only fx-ic2 and fx-v4c9, and fx-ic2 does not depend on fx-nkv.
   - So once fx-ic2 closes, `br ready` would offer fx-1p2i while fx-nkv is still open. That is what fx-v4c9's own bead says must not happen ("Must not displace E1 or the first demo").

   **Fix:** `br dep add fx-1p2i fx-nkv`. fx-tf7d inherits the order through fx-1p2i.

3. **Low, non-blocking, text-only (rule-ID format and prefix owner). `design/01-design.md:435`, `:446`.**
   - The conformance ID scheme (`conformance/README.md:369-371`; 03 uses it, as in `**ROOT-4.**` and `**LOCK-1. Payloads.**`) expects a bold `**PREFIX-N.**` lead, optionally with a title.
   - The new leads are `**CODEC-1 (v1): strict objects.**` and `**CODEC-2 (v1): internal tags.**`.
   - These are also the first rule IDs defined in 01 rather than in a chapter. That is reasonable, since no chapter owns codecs, but nothing records 01 D6 as the owner of `CODEC`.

   **Fix:**
   - Respell the leads as `**CODEC-1. Strict objects.**` (v1) …, and likewise for CODEC-2.
   - Have fx-1p2i register `CODEC` with owner 01 D6 (finding 1).

4. **Low, non-blocking, text-only (stale or miscited evidence text). `research/29-strict-tagged-codecs.md`:**
   - **:359:** §10 cites "(§4)" for the Eon tagged shape, but the Eon evidence is in §3.
   - **:302, :315-316, :6:** The §8 heading "(not filed)", "Neither single-holder file was edited", and "no Beads state was written" are author-session statements. §10 of the same landed note contradicts them: the beads were filed, and the companion was edited under the single-holder slot.

   **Fix:**
   - Change the citation to §3.
   - Mark §8 "(author proposal; §10 records what was done)", or amend those sentences to past-proposal tense.

5. **Low, non-blocking, text-only (admission answer under-stated). `research/29-strict-tagged-codecs.md:288`.**
   - The "Overlapping mechanisms?" answer says "two knobs … no second derive".
   - It does not acknowledge that `tag` adds a second enum wire representation beside external tagging, which is exactly the overlap the filter asks about.
   - The justification exists: the representation is fixed by a peer-defined wire format, not chosen by taste. Leaving it unstated makes the answer read as "no overlap".

   **Fix:** Add one sentence: "Yes, one: a second enum representation, chosen by the peer's wire schema rather than by the author; external tagging remains the default."

6. **Nit (unspecified edges). `design/01-design.md:446-457`, `:459-462`, `:470-472`.**
   - It is unspecified whether `tag = ""` (empty key) is malformed.
   - `deny_unknown_fields` on an enum with no named-field variants and no `tag` is a valid but silent no-op. Authors may read it as making newtype payloads strict.
   - D6's "Deferred" items (tagged newtype/map payloads, adjacent/untagged, catch-all) are not in the companion's "not yet tiered" sentence (`design/reference-companion.md:660-662`).

   **Fix (optional):**
   - State that the tag key is any non-empty string.
   - Either reject the no-op placement or let fx-1p2i's docs say so.
   - Add the deferred representations to the companion sentence, or leave them as they are. This should not block.

Held-out exclusion: `trials/e1-checked/` was never opened or searched.

DONE

## Re-check

Model: claude-opus-5-5 (Opus 5.5) · Date: 2026-11-13. Scope: only findings 1 and 2, checked with `br --no-db show fx-1p2i` and `br --no-db show fx-tf7d`. I also confirmed that the amended D6 (candidate `e005dac`, `design/01-design.md:446-468`) states the two new compile errors the beads now test: the no-op enum placement and the empty tag key. The beads therefore do not outrun the contract.

**Verdict: approve. Findings 1 and 2 are resolved, and no blocking findings remain.**

1. **Resolved.**
   - fx-1p2i's Acceptance now covers CODEC-1's derive-time clauses in both derives:
     - each invalid placement, including the no-op enum;
     - repeated or valued keys;
     - acceptance on `Encode`-only and `Decode`-only types;
     - byte-identical `Encode` output.
   - "Errors are precise" is replaced by a falsifiable absolute line and column at the offending key.
   - CODEC-1 is traced in `rules.md`, and `CODEC` (owner 01 D6) is registered in the README prefix table.
   - fx-tf7d now reads "none of the forbidden observations … occurs" plus the first-row encodings. It traces CODEC-2 and adds the empty-tag compile error.
2. **Resolved.** fx-1p2i now depends on fx-ic2, fx-nkv and fx-v4c9, which matches its Order section and research/29 §10. fx-tf7d inherits the order through fx-1p2i.

Findings 3-6 were text-only and are not re-checked, per AP-PROOF-001.

DONE
