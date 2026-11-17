Model: claude-opus-5-5 · Date: 2026-11-17 · Candidate 15bf6fe (re-check of 20f26c7 findings)

## Verdict

**approve.** Every finding from my review of 20f26c7 is resolved: B1, B2 and non-blocking 1–6. The fixes broke nothing that blocks.

What I checked:
- `git diff 20f26c7 15bf6fe` changes only research/35-trademark-check.md.
- `git diff --check` is clean.
- research/28's pointer is unchanged.

This is a re-check of my own findings only, not a new full review.

## Findings

**B1, the D18 mechanism: resolved.**
- §1 now quotes D18 word for word, and the quote matches design/01-design.md lines 1065–1068.
- Point 2 uses D18's wording: "a pinned, unmodified nightly rustc compiles against Tide's std".
- Point 3 now names the std swap through `--extern std=`. It calls this the strongest reading against the tagline and answers it: the modified-version rules restrict calling a modified Rust "Rust", and Tide calls nothing "Rust".
- "Never describing Tide's std as Rust's std" is added as a keep-it-low example, with the count updated to "Three things".
- The accuracy condition now reads "not valid Rust 2024 for an unmodified rustc", which matches D18.
- Keeping the verdict at low (preliminary) is consistent with the corrected reasoning.

**B2, same-field unregistered uses: resolved.**
- http-rs/tide (marked the closest), IlanCosman/tide, ananthakumaran/tide and Stanislas7/tide now appear in five places:
  - "Other sources", cited to research/28 §4 and the bead's Why;
  - the unregistered-rights follow-ups, with http-rs/tide first;
  - the summary's "Not assessed";
  - §2's "Not assessed";
  - Sources.
- The descriptions match research/28 §4: about 3.7M downloads, a fish prompt with a `tide` command, `tide-mode`, a language written in C.
- All four GitHub URLs answer 200.
- research/28's disambiguation is given as the answer to these uses, which matches the owner's decision to keep "Tide".

**NB1, Tide Platform first: resolved.**
- The EUIPO and UK queries now start with owner Tide Platform, and they check whether its specifications carry the US filing's field limit.
- The cancelled Madrid designation 79193510 (class 36) matches `dl/uspto.json`.
- The new company number 09595646 checks out on Companies House as TIDE PLATFORM LTD.
- Nit, not blocking: the note gives no source for the number.

**NB2, the Nix default runs on its own: resolved.**
- §4 now says the default must work with nothing else installed, and recommends wiring `tidelang-std` into the driver's runtime closure.
- It rejects the documentation-only option and leaves the mechanism to fx-1n5n.
- The names are unchanged.

**NB3, "change" with no flake yet: resolved.** The text now reads "are set to `tidelang-*`. The repository has no flake yet." That is true: there is no flake.nix.

**NB4, a section referring to itself: resolved.** The text now reads "(the registration findings below)", and those findings do follow, under §2's verdict.

**NB5, stites.com: resolved.**
- The claim is labelled a secondary opinion from a law-firm blog, could not be fetched (403), and is unverified background.
- The Sources entry carries the same label.

**NB6, what "low" means: resolved.** The definition ("no evidence was found of a conflict likely to force a rename; it is not a clearance") now appears under the summary table and at §2's verdict.

**Did the fixes break anything?** No. The new text adds no claim stronger than its evidence, and contradicts neither research/28 nor D18. Its repetition is modest and in keeping with the rest of the note.

DONE
