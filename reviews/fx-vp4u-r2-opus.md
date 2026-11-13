Model: Claude Opus 5.5 (claude-opus-5-5) · Date: 2026-11-13

Verdict: **approve.** No blocking findings. Three nits follow; none blocks, and all are text-only, so none needs a re-check.

Candidate: worktree `.worktrees/fx-vp4u`, HEAD `fac7793`, `git diff HEAD^..HEAD` (README.md, design/01-design.md, research/28-positioning-and-name.md). Read-only; no builds. Nothing under `trials/e1-checked/` was opened; every search passed `-g '!trials/e1-checked/**'` after its other globs (these searches used only exclusion globs).

What I checked against the block list:

- **Claims the design does not support: none.** "Subset of Rust" and "a checked subset of valid Rust 2024" state D18 (design/01-design.md:993, "Every Tide program is a valid Rust 2024 crate that a pinned, unmodified nightly rustc compiles … against Tide's std") and reference §0 ("A Tide program is a valid Rust 2024 crate"). Reference §0 also says a Rust habit that would mean something different fails with a code rather than being reinterpreted, so "subset" (containment) holds; it is not merely "dialect" (divergence). Granted `unsafe` is still Rust, so the subset claim holds with it. "Programs replay bit for bit" in README:6 keeps its condition in the next sentence (README:8–9) and in the "Every program replays" bullet, matching Thesis item 2.
- **Misreadable wording: none that blocks.** "Subset" in the headline and "checked dialect of Rust" in README Status (:41) are consistent: D18 is titled "A checked dialect of Rust" and defines the dialect as a subset compiled against Tide's std. Status names that std, so a reader who wonders whether Tide code builds against Rust's std finds the answer there.
- **Leftover adopted uses of "Rust" alone as Tide's descriptor: none.** `rg -i "replayable Rust|Rust that can|capability-safe|Tide is Rust|is Rust that|safer Rust|…"` outside `archive/`, `reviews/`, `.beads/` and `trials/e1-checked/` finds the old forms only in research/28:62–67, where they are rejected or quoted historically. The two trial origins' "Tide is Rust's surface with a few changes" (trials/e1-origins/arm-{a,b}/reference.md:17) describes the surface, not the product. Out of scope and not a descriptor: research/12:54 (dated 2026-10-19, draft 3.1, before D18) says "Tide is not valid Rust". It is a superseded dated research claim; D18 and open question 7 at research/12:444 record what replaced it.
- **research/28 vs README: they match.** §3's adopted plain form (research/28:60) is README:5–6's first sentence word for word, and the headline matches README:3 and design/01-design.md:53. The only drift is the compressed table row (nit 1).

Findings:

1. **Nit (non-blocking): the §2 table row makes "code" the thing that replays.** research/28-positioning-and-name.md:24 reads "a Rust subset in which code can't overreach and replays bit for bit". The README and §3 say "programs replay", and replay is whole-program (Thesis item 2). The table's own "Deterministic replay" row carries the scope and the condition, so no reader is misled about the guarantee. **Fix:** "a Rust subset in which code can't overreach and programs replay bit for bit".

2. **Nit (non-blocking): one reason for rejecting the old headline applies to the adopted one too.** research/28-positioning-and-name.md:66 rejects "Capability-safe, replayable Rust" partly because "Tide has its own std". Read as "programs that build against Rust's std", "subset of Rust" has the same gap. What the new form fixes is that it names Tide's relation to Rust instead of calling Tide "Rust", and "subset" covers "admits only part of Rust". README Status and reference §0 already cover the std. **Fix (optional):** "it calls Tide 'Rust' rather than stating Tide's relation to Rust (Tide admits only part of Rust, compiled against its own std), and that is the use the trademark policy restricts."

3. **Nit (non-blocking): line width.** design/01-design.md:53 is 83 columns, while the rest of the paragraph and the Thesis wrap at 80 or less. **Fix:** rewrap lines 53–55.

Legal sufficiency is fx-gbq4's and I have not judged it. research/28:67 still routes the tagline through that check.

DONE
