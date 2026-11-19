Model: Claude Opus 5.5 · Date: 2026-11-19 · Candidate ad9c03b (docs, 6.1c)

## Verdict

Changes required. One blocking finding, a one-clause text fix.

## Blocking findings

1. **docs/owner-guide/10-v0-and-planned.md:97-98: the bead-title note still says fx-muza uses the old `FX` code.**
   - The sentence this candidate edited ends "and fx-muza's code is spelled in the older `FX` form."
   - The quoted listing four lines above (:87) shows `TD0516`. Commit 0850bc1 fixed the listing.
   - `br --no-db show fx-muza` shows the title `Authority policies (\`[[policy]]\`, \`TD0516\`)`. Its comment of 2026-11-19 11:02 UTC records the rename from FX0516 to TD0516, made so that this listing would match.
   - `br --no-db list --label v0-gap` gives exactly the 11 lines quoted, in the same order.
   - Failure scenario: a reader checks the note against the listing above it. The note says a code is in `FX` form, but the listing shows `TD0516`. The reader cannot tell which one is stale, and the note no longer explains anything.
   - This is a stale claim in docs/, inside the sentence this change rewrote, and the brief's item 4 asks for exactly this check. The Codex pass prescribed the fx-9bzp clause and missed this one.
   - Minimum fix: delete the clause, which makes the sentence:
     "The bead titles are as filed: fx-tl1u's `command` and `only` have landed (fx-ayh2.1), and fx-9bzp's five `Rand` draws have landed (fx-qlek.20) and only `rng() -> Pcg64` remains planned."
   - No `br` write is needed.
   - `code_prefix` still passes after the fix, since the text has no `FX` followed by digits.

## Non-blocking findings

1. docs/driver.md:233-234: the std inventory now reads "`Root` and the capability types," with no mention of `Rand`'s draws or `Proc::command`. That is true, but less informative than it could be. Dropping the stale parenthesis was the right minimum change.
2. paper/tide-v0.typ:131: "draws integers (`next_u64`, `random_range`)" is accurate, because `random_range` is integer-only (reference.md:340 says "(integers)"; design/01-design.md:527 says "integers, unbiased by rejection"). Nothing needs to change.

## Reasoning

### 1. Changed sentences are true and narrow

- **paper/tide-v0.typ:131**
  - It names all five draws, matching rand.rs:127-176 and reference.md:340.
  - "under simulation the seed fixes each draw" matches 01-design.md:529-530 and rand.rs:5-6.
  - "a seeded generator on `Rand`" matches 03-authority-concurrency.md:396-397: `rng() -> Pcg64` is "a `Pure` generator seeded by one draw".
  - It says nothing about crypto, keys or "unforgeable", and keeps the paper's voice.
- **paper/tide-v0.typ:170 and :390** agree with README.md:446 and 10-v0-and-planned.md:27. Fault injection, crashable nodes and `tide sim` stay listed as planned, which README.md:422-423 confirms.
- **docs/driver.md:355-356** ("`Rand` draws and `RandomState::from_rand` keys come from the seed") matches rand.rs:179-189 and 01-design.md:530-531. Dropping "Simulated HTTP" at :368 is backed by README.md:445 (`world.http`).
- **docs/tide-or-rust.md:132 and :156** are accurate and narrow.

### 2. Stale-claim sweep

I searched README.md, docs/, paper/, paper/figures/spec, and every other *.md, *.typ and *.html outside design/, research/, reviews/, archive/ and target/ for: randomness, `Rand`, "identity only" and "no draw".

- Apart from the blocker, nothing stale remains.
- docs/driver.md:352 ("no random draw") is about env operations, and :531 ("`Rand`-seeded hashing") is current.
- Hits in trials/ are historical records.
- There is no site/ directory.

### 3. Recount and figures

- `node paper/figures/build.mjs --check` → "figures match their records (22 files)", exit 0.
- `git hash-object design/reference.md` = a4b5230…, which matches reference-sections.json.
- The `## 6` to `## 13` heading lines (326, 570, 766, 799, 813, 949, 970, 1057) match the new first_line values.
- budget.svg shows 5,959, a whole of 24,932, and a sum of 24,904. The whole equals reference-companion.md:17.
- sim-model.svg's last row reads "Arguments, environment variables, standard input, sink output; `Rand` draws from the seed" against "Fault injection". It wraps to two lines inside the column.
- The figure's caption (:402) makes no area claim that changed.
- No text outside the data and SVG files quotes 24,960, 24,932, 24,904, 5,987 or 5,959. The paper's budget caption cites only `budget.budget`, which is 25,000 and unchanged.
- I did not re-derive the 28-token gap; the Codex pass explains it.

### 4. br listing

The listing matches `br --no-db list --label v0-gap` exactly. fx-9bzp is still deferred and still titled "`Rand` draws and `Pcg64`", and its acceptance still names `rng() -> Pcg64`, so the new fx-9bzp clause is correct. The fx-muza clause is false (blocking 1).

### 5. Leftovers

- The only guard that reads the changed files is build.mjs. diagrams.json's README quotes are checked by it, and it passes.
- The `code_prefix` test (crates/tide-admit/tests/code_prefix.rs) walks the tree for `FX` followed by four digits. The candidate adds none. The author's run recorded 2 passed; I did not rerun it, because it compiles.
- No other test, fixture or script references docs/driver.md, docs/tide-or-rust.md, the owner guide or paper/tide-v0.typ.

DONE
