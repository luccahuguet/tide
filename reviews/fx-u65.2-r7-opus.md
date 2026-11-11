Opus 5.5 (`claude-opus-5-5`) · 2026-11-11

**Verdict: approve.** The round-6 note is fixed and no finding blocks landing.

**Status:** I re-checked candidate `9548501` on base `10991bd`. The diff hash matches the brief (`5d026761…ee8b65`), and the worktree is clean. I read the whole fix delta (`git diff 04f6378 9548501`) and worked read-only. I ran the probes directly with the candidate's `target/debug/tide-admit`, under `nice -n 19`. Load was about 1.1 with 16 GB available, and the quiet window had ended. No file under `crates/` is newer than the binary. I did not rerun the suites. I did not open `trials/e1-checked/`.

**Check 1: probes**
- **Push** (`v.push(Doc{word:&text, other:&y, text})`):
  - `text`'s E0597 (`:2:117`) and its E0505 (`:2:134`) both have FX0210.
  - `y`'s E0597 (`:2:130`) no longer has it.
- **Return:**
  - `y`'s E0515 has no FX0210.
  - The owner's E0505 (`:2:115`) keeps it.
  - The owner's E0515 also loses the label, as the note below explains.
- **`self_borrow_reject.rs`:** still gets FX0210 on line 6, through the E0505 at `6:24`, so its line marks hold.

**Check 2: no new wrong or missing label in what the fix touched**
- **Method-receiver borrow:** in `Doc{word:text.as_str(), other:y.as_str(), text: text}`, only `text`'s E0597 and E0505 are labelled.
- **Two owners:** in `Two{ra:&a, rb:&b, a, b}`, each owner's E0597 and E0505 is labelled.
- **Moved field that nothing borrows:** in `Two{ra:&b, rb:&b, a, b}`, only `b`'s errors are labelled, and `b`'s E0505 is labelled once even though two fields borrow it.
- **Matcher code:**
  - The removed `!labels.contains` guard is not needed. `.any()` pushes FX0210 at most once, and the FX0203 path pushes a different code.
  - `direct_borrow` only accepts method calls with no arguments. So a borrow field cannot contain another local's `&y` argument that would wrongly match "inside the borrow field".
- **Docs:** `conformance/README.md:84-86` and the REF-0210 row at `conformance/rules.md:154` describe the new rule accurately.

**Note (minor, non-blocking, works as the decision intended):** at `crates/tide-admit/src/diag/labels.rs:22`, a returned self-borrow literal no longer labels the owner's own E0515.
- **Why:** rustc puts that error's primary span on the whole literal (`ret.rs:2:89`, and `self_borrow_reject.rs:6:5`). The rule only accepts an E0515 whose primary span is inside the borrow field.
- **Effect:** in round 6 this error had FX0210. The same literal's E0505 still has it, so the writer still sees the explanation, and nothing is labelled wrongly.
- **Possible later improvement:** the E0515's secondary label "`text` is borrowed here" sits on `&text`. Matching that label against the paired borrow field would bring the owner's label back without any guessing.
- **Where:** if you want it, add it to fx-u65.3 with the table-driven label seam. It is not needed for fx-u65.2.

No new findings.
