Model: Claude Opus 5.5 · Date: 2026-11-19 · Candidates Tide 7eb5585, site 555cc9c (code)

## Verdict

**Changes required.** There are five blocking findings, each with a small fix. Two are claims that say more than their record (B1, B2). One is a wrong first-use definition (B3). Two are readers that can fail open (B4, B5). Codex's r1 fixes B1 to B6 and N1 to N2 hold.

## Scope

- Both candidates match the brief. Tide `git diff 04dde61..7eb5585 | sha256sum` = 8d6a87da…4b3e and site `git diff 9aefc6e..555cc9c | sha256sum` = 1b8427d8…6493. Both trees are clean.
- I read the bead, its comments, AGENTS.md (Tide and site), r1-codex.md, both re-checks, report.md ("The pin" and the r1 fix rounds), terms-pass.md and r1-checks3.log.
- I read all of the site diff: status-data.ts, status-parse.ts and its tests, status-falsifier.ts, status.astro, index.astro, Inline.astro, the generated status.json and at-a-glance.md, the e2e diff, README, site.ts, CSS and package.json. I also read all of the Tide diff.
- I checked each tile value against its source at 7eb5585: research/37, 41, 46 and 47 (including §6), conformance/README.md, trials/e3-pilot/report.md, the reference's §6.1 table, the atomics bullet and the §11 holes table, and the README's forge-routes and replay text.
- I ran these checks myself:
  - `scripts/count-tokens.sh design/reference.md` gives **24999**, exit 0. This matches the companion line and status.json.
  - The README's glance block equals docs/at-a-glance.md byte for byte (Python).
  - The run record's lines 8 onward equal WORK/conformance-a4ab41c.stdout (`diff`, no output).
  - `git diff --name-only a4ab41c 7eb5585` touches none of the record's inputs. Only .beads, README, the record itself, the reference, the companion and the glance changed.
  - The conformance runner reads only paths under `conformance/` and `crates/`, plus rust-toolchain.toml (main.rs:34-36, 170-172, 415), so the record's declared inputs cover what it reads.
- I ran no build, `bun`, browser or Cargo command. Site runtime results (astro check, 10 parser tests, the falsifier, `--check` match, 30/30 e2e) are builder-reported in r1-checks3.log.

## Blocking findings

### B1. The conformance headline counts AWAITS CHECKER cases as giving their expected verdicts

- **Where:** site `src/pages/status.astro:109-110`. The headline renders "`{pass + awaits}` of `{total}` conformance cases give their expected verdicts", which is "392 of 392".
- **Record:** conformance/README.md:61-62 and 90-93 say AWAITS CHECKER means "`TD` marks in other codes remain, which only later slices of the checker produce", and "The `TD` marks are the design's expected verdicts".
- **Failure:** contracts/clause_param_reject.rs, contracts/clause_pure_reject.rs and memory/opaque_auto_reject.rs do not yet give TD0809, TD0808 and TD0205. A reader of the first status tile is told that all 392 cases give their expected verdicts. The page's own AWAITS bullet (lines 122-123) then contradicts the headline. This is the page's lead number, and it claims more than the record.
- **Minimum fix:** lead with PASS. For example: "`{pass}` of `{total}` conformance cases give every expected verdict. The other `{awaits}` match every mark today's checker decides, and `{fail}` fail."
  - Update the e2e assertion at tests/site-smoke.spec.ts (`${c.pass + c.awaits} of ${c.total}`) to match.
  - The falsifier still works unchanged, because it moves one PASS to FAIL.

### B2. E3's frozen protocol is stated without its qualifier on /status/

- **Where:** site `src/pages/status.astro:249`, "E3 ran under a protocol frozen before the run."
- **Record:** E3's freeze rests on git and bead records (trials/e3-pilot/report.md §2 and §13). The glance itself says "For the pilot and four of the searches, that order rests on our own records (self-attested)" (docs/at-a-glance.md:61-62). The brief's rule is "self-attested otherwise".
- **Failure:** a reader who opens /status/ directly gets an unqualified pre-registration claim for E3, while the same claim for each search carries a qualifier. The two surfaces disagree.
- **Minimum fix:** for example, "E3 ran under a protocol frozen before the run, an order that rests on Tide's own records (self-attested)." "Self-attested" is already defined in the glossary.

### B3. The E4 gloss reads E4 as a bead's name, but E4 is an eligibility criterion

- **Where:** site `scripts/status-data.ts:191`. The gloss is rendered at status.astro:232 as "The experiment's frozen rules, set in its bead E4, barred outside libraries, the network and other programs".
- **Record:**
  - research/41:150 says "come from frozen bead E4", and line 192 says "The bead bans external-crate operations…". There, E4 is the bead's frozen criterion E4, "not portable to Tide v0" (research/41:97 and 136).
  - research/38:14 uses the same criterion.
  - The same tile's "Bugs that failed first on portability" list (fx-zu4f.6 15, .7 26, .9 68) counts exactly the rows labelled "Not eligible, E4 (not portable to Tide v0)".
- **Failure:** the glossary defines a bead as "An entry in Tide's issue tracker, named like `fx-zu4f.6`". A cold reader therefore reads "its bead E4" as a bead named E4, which does not exist. The term-definition rule is met with a wrong definition. The r1 re-check accepted this text, but it does not match research/41.
- **Minimum fix:** gloss it as "The search's frozen portability criterion, E4 in its bead, barred outside libraries, the network and other programs". Better, add a glossary entry "E4: the searches' eligibility criterion that a bug can be ported to Tide v0". That entry also explains the "failed first on portability" counts.

### B4. Two reference readers can show wrong text instead of failing

- **Where:** site `scripts/status-data.ts:181` (atomics) and `:182` (weaker orderings).
- **Atomics reader:**
  - `^- (Atomics … only[^\n]*\n[^\n]*\n[^\n]*)$` captures exactly three physical lines. The bullet at reference.md:741-743 is three lines today.
  - design/reference.md is a single-holder file that is rewrapped often under the 25,000-token budget. If the bullet grows to four lines, the tile quotes a cut sentence (for example "…are in `std::sync::raw` (11).") and does not throw.
  - If the bullet shrinks to two lines with a following line, that line is swallowed into the quote.
- **Weaker-orderings reader:**
  - `^\| (`std::sync::raw` locks, weaker atomic orderings \(planned\)) \|[^|]*\| ([^|]+) \|$` matches the row without checking its table's header (`Hole | Spelling | Rule`, reference.md:953).
  - If the last two columns are reordered, the page shows "…: —" as the rule.
  - This is the failure class r1 B3 fixed for the other tables, and these two readers missed that fix.
- **Minimum fix:**
  - Atomics: capture the bullet to its end, its first line plus every following line indented by two spaces, for example `(?:\n {2}[^\n]+)*`.
  - Weaker orderings: read the row through `table(reference, <§11 heading>, ['Hole', 'Spelling', 'Rule'], …)` and select it by its first cell.

### B5. The rounds list is hard-coded, so a newly registered round never reaches the page, while the glance says the page lists every round

- **Where:** site `scripts/status-data.ts:338-388`. The seven rows are fixed, and nothing compares them with the rounds registered at `tideRevision`. Tide docs/at-a-glance.md:65-66 says "The status page lists every round with its counts and its qualifier". The tile's h2 says "Each registered search is published…".
- **Failure:**
  - fx-zu4f.10 and .12 are in progress, and .13, .14 and .15 are filed. Each one registers a protocol in research/protocols/ and a record under research/.
  - At the landing re-pin, or any later re-pin (report.md "The pin", step 2), the generator succeeds, `--check` matches, and the page silently omits the new round.
  - Meanwhile the README's first screen says the page lists every round. A "new round" is one of the source changes the brief asks about, and here it fails open.
  - Already at 7eb5585, research/protocols/ holds `fx-qlek.17-protocol.md` with a complete proof (a pre-registered post-mortem check, research/43). The page neither lists it nor records why it is left out.
- **Minimum fix:**
  - In status-data.ts, list `research/protocols/*-protocol.md` at `tideRevision` with `git ls-tree`.
  - Require each bead to be either a `round` in `rounds` or a key of an explicit `notRounds` map whose value gives the reason (fx-qlek.17: a paper comparison of post-mortems, with no screened, eligible or caught stages). Throw on any other bead.
  - tide-lead decides whether fx-qlek.17 is a row or an exclusion, and the glance's "every round" stays true either way.
  - The self-attested rounds fx-zu4f.5 to .7 have no protocol file, so they stay in the hard-coded list. This check covers every round registered since the protocols directory began.

## Non-blocking findings

- **N1. The glance's anchored/self-attested contrast implies a stronger proof of order.** docs/at-a-glance.md:61-64 says "For the pilot and four of the searches, that order rests on our own records". This implies that for the other two, the order rests on more.
  - research/46:48-51 says "The examination times below are self-attested… [the proofs] do not prove that no screening took place before".
  - research/47 §2 says the proof "proves only that the file existed by that block's time".
  - The literal text is true. A sentence such as "When they examined their candidates still rests on our records" would close the gap. The status glossary's two qualifier entries (status.astro:67-68) have the same implicature. Leave this to the fx-qlek.7 cold reader if not fixed now.
- **N2. The fx-zu4f.8 Eligible count is derived and does not match fx-zu4f.5.** "16 with a portable source" (status-data.ts:385) is 87 − 71. The number 16 and its label appear nowhere in research/47, yet the page says "Each count carries its record's own label" (status.astro:259). For the same 87 advisories, fx-zu4f.5-p1 shows "87 qualify". Either show "87 advisories" as eligible and list research/47 §6's outcome lines, or say in the paragraph that this one count is derived.
- **N3. The boundary table caption claims more than two of its rows.**
  - status.astro:228 says "Why searched bugs did not port". research/41:197 says the size cap "decides no first failure", and :191 says "atomic API absence alone decides no first failure".
  - Suggested caption: "The limits research/41 maps".
  - Suggested Planned gloss (status-data.ts:190): "The bug needs raw locks, which are planned".
- **N4. "Three searches for real concurrency bugs" (status.astro:221).** This count comes from the hard-coded `unportable` list. fx-zu4f.11 also hunted concurrency bugs and stopped at portability failures (research/46 Arm A). Suggested wording: "Three of the searches recorded, per bug, the first criterion it failed".
- **N5. The run record declares its own inputs.** `field('inputs')` (status-data.ts:95) is trusted as written. A record whose header names fewer paths would pass a stale run. Requiring the declared set to contain `conformance crates Cargo.toml Cargo.lock rust-toolchain.toml` costs one line. Today's header is complete (verified above).
- **N6. Nothing checks the glance's hard-coded "four of the searches" (glance:61) against `rounds`.** One assertion in status-data.ts would catch the drift that B5's new rounds will cause.
- **N7. The glance copy check runs only in the site.** A Tide edit to the README's glance block is caught only at the next site re-pin. A Tide-side check is optional, since the bead's "a check that the two match" is met.
- **N8. The `.ots` check is structural only, as its comment says.**
  - What it checks: the header magic, version 1, the SHA-256 file op, the digest equal to the protocol's SHA-256 at the pin, op tags (0x02, 0x03, 0x08, 0x67, 0xf0 to 0xf3, with varbytes for 0xf0 and 0xf1), attestations as tag, 8 bytes and varbytes, the Bitcoin tag `0588960d73d71901` with a payload of exactly one complete LEB128 block height, the pending tag `83dfe30d2ef90c8e`, and the 0xff forks. All of this matches the OpenTimestamps serialization.
  - Malformed, truncated, padded, unknown-tag, pending-only and empty-height proofs throw.
  - What it does not check: it runs no ops, so it does not prove that the Bitcoin leaf commits to the digest. The page's wording ("has a complete OpenTimestamps proof") stays within that.
  - The records name the blocks (976452 and 976487). Linking `ots verify` as the reader's own check would be a cheap addition.
- **N9. The fidelity note reader can match outside its section.** `fidelityNote` (status-data.ts:168) is a lazy `[\s\S]*?` from `### Simulator fidelity`. If the note left that section, a later paragraph starting "The real and simulated backends" anywhere in the README would be quoted. Bounding the match at the next heading would fix this.
- **N10. /status/ opens with 42 definitions before the tile index.** That suits the terms rule but not a grand first impression. One option is to move the six-link tile index above the glossary and wrap the glossary in an open `<details>` or a two-column grid. Both keep the definitions before the tiles in reading order.
- **N11. Many links are bare numbers.** Most `data-number` links read "389", "3", "0" and so on, all to the same file. Their sentences give the context (WCAG 2.4.4 "in context"), so this is acceptable. An `aria-describedby` or a visually hidden "(run record)" would help screen-reader link lists.
- **N12. The hero caption's "whose tests pass" (index.astro) is sourced by examples/capabilities/README.md:13 and :48-52, but not linked.** Linking that README's table row would close it.

## Reasoning

**1. Claims.**
- Tile 1 values (392, 389, 3, 0, exit 0, a4ab41c, 2026-11-19T05:13:49Z) match the record, and the record matches the run's stdout. Only the headline's wording is wrong (B1).
- Tile 2 is the §6.1 table, split correctly at "planned:":
  - Net ships get and resolve, with bind_udp planned.
  - Proc ships `command`, with `exit` planned.
  - Tasks' fault_point is now planned.
  - Rand is planned only.
  - The Later text matches the companion.
- Tile 3 is the README fidelity table, verbatim apart from Inline's semicolon-to-period rewrite.
- Tile 4's counts (15, 26, 68) match research/38, 39 and 41. The kind mapping fails closed on a new boundary name. Glosses: B3, N3.
- Tile 5's counts match:
  - research/37: 1,507 screened, 87 qualifying, 12 assessed, 0 caught. P2: 60 unique, 60 excluded, 0 examined.
  - research/38 and 39: 32/0/0 and 119/0/0. research/41: 100/0/0. research/46: 7 repositories, 42 rows, 0/0.
  - research/47: 87 in the pool, summing to 87. "16" is derived (N2).
  - E3: 66 and 67 of 72, ratio 0.99, 95% [0.95, 1.00], "not met".
  - The qualifiers match the bead: self-attested for .5 to .7 and .9, anchored for .11 and .8, and both proofs are complete at the pin according to the Codex re-check.
- Tile 6 (24,999 of 25,000) I recounted myself.
- Glance items 1 to 9 each trace to the README, the reference or a record:
  - The forge routes: three routes through #25860 and #57893, with TD0206 and TD0402.
  - `--replay` on the same machine with identical event and output traces.
  - `only` and `no_bind` (reference:338, 354 and 509).
  - "An allowed shell is full authority".
  - The `--tide-diff` description.
  - E3 "no evidence".
  - x86_64 Linux, async out, SC atomics only with weaker orderings behind the grant.
  - The bead's conditions hold: &Proc and &Net are named only after fx-ayh2.1 and fx-ff13 landed, the no_bind point is stated, and "pre-registered" and "verifiable" never appear. The owner's 'race-free by construction' wording rule is honored and e2e-asserted.

**2. Fail closed.**
- `table()` checks the heading, that no heading comes between it and the table, the exact header, the separator width and every row's cell count. `splitPlanned()` accepts exactly three forms.
- The run record must cover the whole corpus at the pin, its summary must equal its tally, its exit status must agree with its FAIL lines, and its inputs must be unchanged since the run.
- research/37's pool sums, the screening funnels of research/38, 39 and 41, research/46's identities and research/47's §6 sum all throw on drift.
- `--check` is a byte equality between the regenerated strings and the committed files at `tideRevision`. Its passing line is in r1-checks3.log:72. The e2e test asserts that status.json, charts.json and site.ts carry the same revision.
- The falsifier edits the export, rebuilds in a scratch copy under $TMPDIR and requires the area row to change, which is what the Acceptance asks.
- The gaps are B4, B5, N5 and N9.

**3. The .ots parser.** It is correct for what it relies on (N8). No path returns `anchored` without a parsed proof whose file digest equals the protocol's SHA-256 at the pin and which carries a well-formed Bitcoin attestation. A proof with only pending attestations throws. A missing proof requires the record to say "self-attested" and never to name OpenTimestamps.

**4. Readers.**
- The glossary precedes the tiles (asserted by e2e). The hero and glance define their terms inline. B3 is the one wrong definition.
- x86_64 Linux appears right after "Install the alpha" and again in glance item 8.
- Headings are h1 then h2. Tables have captions and scoped headers. Colours use the existing tokens. Axe and the 320px no-overflow check cover /status/ (builder-reported).
- A semicolon e2e test covers /status/ main and the home hero, and the glance has no semicolon outside code.

**5. Tide side.**
- reference.md changes two lines: the Budget line and the Tasks row. The Tasks row keeps every item (scope, spawn_isolated, shield, checkpoint, yield_now (7.1), synchronization handles (7.4), fault_point with its note and companion 17). The count is 24,999, which the companion records and I reproduced.
- The README glance is byte-identical to docs/at-a-glance.md, and `#forge-routes` exists.
- The run record equals the run's stdout, and its header matches WORK's .meta and .exit files.

**6. Tests and leftovers.**
- The parser tests are few and strong: header reorder, missing cell, separator, the planned-marker variants, and the OTS complete, pending, empty, truncated, extra-byte, wrong-digest, padded, unknown-tag and empty cases.
- pending-check.ts, ots_probe.py and make-record.py stay in WORK. Neither candidate commits them, and Tide's target/ is ignored.
- I found no dead exports apart from `cells`, which is exported but used only internally. That is a nit.
- The cheapest missing check is B5's protocols enumeration.

**Pin procedure.** Steps 1, 2 and 5 regenerate from LANDED and re-assert equality, so the data cannot drift from its source in the procedure itself. The drift it allows is B5 (new rounds omitted) and N6 (the glance's "four"). The procedure's step 2 note, "only tideRevision and input SHA-256s should change", is a human check, and a reviewer of the re-pin diff should actually run it.

DONE
