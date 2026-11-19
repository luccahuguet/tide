Model: Claude Opus 5.5 · Date: 2026-11-19 · Candidate 2745bc2 (docs, S2a)

## Verdict

**Changes required.** The paper is in good shape and close to landing. The title and abstract land the thesis, §4.3 is fair about provenance, §2.3 stays within D9, and every major section has a visual or a captioned placeholder. Six blocking findings remain, each fixed by a clause or a sentence: four limits missing where readers meet a result, one framing that contradicts its record, and gaps in the terms pass. None of them repeats a Codex finding.

## Scope

- **Candidate.** Worktree /mnt/build/eon-agents/worktrees/tide-fx-qlek.7, branch fx-qlek.7, HEAD 2745bc2 (parent 90de8d5). It changes paper/tide-v0.typ and adds reviews/fx-qlek.7-paper-terms.md.
- **Read first.** AGENTS.md, GOVERNANCE.md, `br --no-db show fx-qlek.7` and its comments (the wording, tone, terms, title, red-team, D3/visual and research/45-correction rules), plan.md, report.md ("Stage S2a" and "S2a round 1 fixes"), s2a-codex.md and s2a-recheck-codex.md.
- **Read in full.** The whole paper source.
- **Checked against records.** research/49 (§1–3, §9), research/47 (§1 and §6 counts), reviews/2026-11-17-red-team-opus.md, all three case-study READMEs, the reviews/ red-team set, design/01-design.md (D7 sinks) and design/reference.md (stdin).
- **Rendered.** `typst compile --root . paper/tide-v0.typ /mnt/build/tide-agents/fx-qlek.7-target/tide-v0-s2a-opus.pdf` exited 0, 18 pages, with the title metadata correct. I read the extracted text with pdftotext.
- **Not re-checked.** The table counts and the B1–B6 items, which Codex verified. Figures belong to S2b.

## Blocking findings

### B1: The abstract's catch account omits the post-hoc pool

**Where:** paper/tide-v0.typ:141, "Our searches for external bugs are _pre-registered_ … On such a re-test, a port of a malicious crate … meets the test's caught rule …"

**Evidence:**
- research/49 §1 opens its disclosure with "**The pool was chosen after fx-zu4f.8 saw the rows**" and "The outcome at R' was close to predictable."
- research/49 §9 ends: "Any further write-up repeats section 1's disclosure."
- §4.4.1 (:316) repeats that disclosure. The abstract, which is the account most readers see, does not.
- The abstract now frames the catch as one of "our searches … pre-registered … On such a re-test". That framing pulls the catch toward the registered-search programme more strongly than main's abstract did.
- The S1b Opus round (B1) already required this exact clause in docs/tide-or-rust.md.

**Minimum fix:** after "re-tested under a new registration", add: "that re-test's one-row pool was chosen after the screen had seen the row, so its outcome was close to predictable;". Keep the rest.

### B2: "Outside admission's class" contradicts the hyper record

**Where:** :141 ("place them outside admission's class: they are logic and timing errors that admission's rules do not address"), :320 ("Each bug is a logic or timing error that admission's rules do not address, so the work falls to the simulator"), and :328.

**Evidence:**
- The hyper case is titled "hyper's pool reaches for an ambient runtime". Its README (examples/case-studies/hyper-ambient-runtime/README.md:1–27, :146–153) says the root cause is an ambient reach. It says Tide's rules do address it ("Reject the ambient reach at admission, in its std spellings … `TD0602` … `TD0501`"), even though no TD code catches either *reported failure*.
- That README also says: "What Tide gives is the fix's shape." It does not say the simulator does the work.
- The paper's own hyper paragraph (:326) says this correctly. The framing sentences around it do not, and a skeptic who reads the title "reaches for an ambient runtime" next to "outside admission's class" will see the contradiction.
- tide-desk's rewrite required "Every clause must match its record".

**Minimum fix:**
- Abstract: "place their reported failures outside what admission rejects: two are timing errors in code that holds the authority it uses, and hyper's starts from an ambient-runtime reach but fails inside tokio, which Tide does not admit."
- :320: "Admission rejects none of the three reported failures, so the work falls to the simulator and, for hyper, to the fix's shape:".

### B3: The abstract quotes benchmark ratios without the "unstable runs" hedge

**Where:** :141, "Benchmarks against plain Rust price Tide's additions, from a tie for contracts … to cold debug builds 3.8 to 4.6 times slower".

**Evidence:**
- research/36's main hedge, which the paper itself quotes first at :436, says all 60 series are unstable. "The medians and ratios below are descriptive observations from these unstable runs, not settled measurements."
- This candidate newly puts specific numbers into the abstract, including a favourable one ("a tie for contracts"), with no hedge.
- The limit appears in §4.7 and §6 but not where most readers meet the numbers.

**Minimum fix:** "Benchmarks against plain Rust, on runs their tool marks unstable, price Tide's additions …".

### B4: The paper does not say that Eon is the author's own project

**Where:** :141 ("real bugs from Eon, a terminal workspace") and :322 ("Eon [17] is a terminal workspace …").

**Evidence:**
- Both Eon case-study READMEs say: "Eon is the Tide owner's project" (examples/case-studies/eon-discovery-race/README.md:161, eon-slow-close/README.md:140).
- Two of the three "real bugs" come from the author's own project and tracker. That is a selection and independence fact a skeptical reader needs, and the records already disclose it.

**Minimum fix:** at :322, "Eon [17], the author's own terminal workspace, …". Optionally also in the abstract: "from Eon, the author's terminal workspace,".

### B5: The `proc` grant's limit is missing from the catch

**Where:** :310 and :316.

**Evidence:**
- research/49 §9's negatives list: "The child a `Command` starts has ambient authority (companion 20). The `proc` grant says that the code runs processes, not what those processes do."
- The paper says the admitted version "declares the authority it uses: task authority, … clock and process authority". It also says "what changes is that its authority is declared."
- For an exfiltrator of `.env` files, a reader will take "the authority it uses" to cover the file read and the network send. Those happen in the started processes, which the declaration does not describe.
- §2.2's general sentence about an allowed program's full authority sits 10 pages earlier. The limit belongs where the catch is claimed.

**Minimum fix:** at :316, after "… what changes is that its authority is declared", add: "; the `proc` grant says the code starts programs, not what they do, and the programs it starts, which receive the `.env` path, run with their own full authority".

### B6: The terms pass misses five terms, so its "MISSING entries: none" is false

**Where:** the paper's first uses, and reviews/fx-qlek.7-paper-terms.md:12.

**Evidence:** under the binding 2026-11-18 19:51 rule, each of these is undefined at its first use and has no row in the artifact:
- "nightly" (:193, "a pinned, unmodified nightly rustc"; again at :213). The Dialect row quotes it without defining it.
- "Cargo workspace" (:213).
- "I/O" (:294, "an I/O method"). It is written out only later, at :324.
- "glob" (:294, "the allowed glob over `std::path`"; :511, "`core::*` globs").
- "`no_std`" (:494).

**Minimum fix:**
- Define each in a few words where it first appears: "nightly (rustc's unstable daily build)", "Cargo (Rust's build tool) workspace", "input/output (I/O)", "glob (a `::*` import of every item in a module)", "`no_std` (built without Rust's standard library)".
- Add the five rows to the artifact.

## Non-blocking findings

1. **Swallowed semicolons in the PDF.**
   - Typst consumes a `;` that directly follows an embedded call, so 9 `#repo("…");` sites lose their semicolon.
   - The sites are :185 and :575 (×3), :577 (×3), :578 and :583.
   - The PDF then reads "par.rs the core reference §6.8" and "design/01-design.md first principles: …".
   - Fix: write `#repo("…")\;`.
2. **The abstract's definition of authority versus sinks (:139).** "Authority, the power to observe or change the world outside the program, is reachable only through such capability values." `print!` and `eprintln!` change the world outside with no capability (D7, design/01-design.md:539; the red team's SINK-1 route is left open on purpose). Suggest adding "apart from write-only diagnostic output (§2.2)".
3. **"Tools layered on Rust cannot remove that, because the code can always reach `std`" (:155).** This text predates S2a. Tide is itself a tool layered on rustc that removes it by replacing std, so the sentence contradicts §2.5. Suggest "Libraries and lints that keep Rust's std cannot remove that, …".
4. **The abstract's E1 sentence (:141).** Consider adding "on the point estimates, near ceiling". §4.6 states both, and the iteration ratio's interval (1.29–1.64) crosses the 1.5 limit.
5. **The fig-redteams caption (:291).** It says "every blocking find was fixed before its change landed". The 2026-11-17 finds were holes in admission that had already landed, and that red team's own change fixed them. "Every blocking find has been fixed" is exact.
6. **A detail from research/49 §1 the paper does not use (optional).** fx-zu4f.8 also disclosed that two non-registered variants of the same port were admitted. It is a further reason the outcome was predictable.
7. **The terms row for `async`/`await`** reads "named for Rust readers", which is not a plain-words definition. A PL reader outside Rust would welcome a few words ("Rust's syntax for asynchronous functions").

## Reasoning

**What I checked and confirmed.**
- **Title and abstract.** The title and first line follow the 2026-11-18 20:18 decision. "Capability-safe" and "replayable" are defined in the first paragraph. The accident-only qualifier precedes any use of the claim. "Unforgeable" appears only in §4.2's conditional paragraph, which says v0's capabilities are not called unforgeable. There is no "race-free", no rate, and no first-time claim.
- **The trust diff.** It is described only as a summary. "Catch" vocabulary appears only for E3's measured reviewer outcomes.
- **The catch.** Its long account (§4.4.1) carries every element of research/49's binding wording: the gap, screen, fix and new registration; the qualified order with the 61-minute and two-hour figures; admission and type-checking only; 0 of 87 kept; not first-time and not a rate; the pool chosen after; the E0283 reversal with 17 sites; "found nothing unaided".
- **The red teams (§4.3).**
  - They match their records, including the 48 routes, which include the projection form found by the r2 review (record lines 10–16, 76).
  - The section is fair about provenance: it states that the admission red team wrote its own fixes and that the records do not establish independence or blinding.
  - The table covers every red-team record in reviews/.
- **§2.3 and D9.** The parallel-regions paragraph stays within D9: sequential equivalence as a checked contract, explicitly not a confluence theorem, the panic and trust-hole qualifications, and the partial runtime.
- **Rust positioning.** It stays complementary: "not a fork or a replacement".

**Why the six findings block.** The weaknesses are argument-level. The abstract compresses three results (the catch, the case studies, the benchmarks) and drops the one limit each record puts first (B1, B3), or uses a framing its record contradicts (B2). B4 and B5 are disclosures that the source records make and the paper omits, at the place a skeptic will attack. B6 is the owner's binding terms rule, and the artifact asserts that it is complete. Each fix is a clause, so I expect the finder's re-check to be quick.

DONE
