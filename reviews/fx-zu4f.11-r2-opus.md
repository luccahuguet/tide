Model: Claude Opus 5.5 · Date: 2026-11-18 · Candidate cfc5c26 (research record)

# fx-zu4f.11 record: Opus review round (docs and research)

Candidate: worktree `/mnt/build/eon-agents/worktrees/tide-fx-zu4f.11`, branch `fx-zu4f.11`,
HEAD `cfc5c2656c591e0a384ea1c731f7e8ff1ad08292`. Files under review:
`research/46-fx-zu4f.11-agent-marker-round.md` and
`research/protocols/fx-zu4f.11-record/b-candidates.tsv` (SHA-256 `505b9442…056d7`, which matches
the hash the record cites). Read-only. No edits except this file, no builds, no network. The only
upstream data read was commit messages from the frozen clones, read as commit objects so no blobs
were fetched.

## Verdict

**Changes required.** There is one blocking finding, and it is small. The judgments I sampled
hold, and both NOT CAUGHT results are supported. The blocking finding is that one sentence
claims more about evidence coverage than the screening records support. Fixing it changes no
judgment: I read every source it names, and none of them changes an outcome.

## Scope

- Read: AGENTS.md, GOVERNANCE.md, `br --no-db show/comments fx-zu4f.11`, `br --no-db show fx-zu4f.13`,
  the protocol with A1/A2 in full, the record in full, and all 42 TSV rows. I also read the Codex r1
  findings, the fixes and the re-check, but only to avoid repeating them.
- Mechanical checks, all passing:
  - The record's 42-row table matches the TSV on rank, first parent, first failure, duplicate
    link and capability tag.
  - In every TSV row, the criteria before the first failure pass, the first failure says FAIL,
    and later criteria say "not assessed".
  - Every row that passes E2 has a quote and a source, and every E4 row has an inventory.
  - Per-repository candidate counts are 4/13/1/11/3/0/10, total 42.
  - Raw and deduplicated first-failure totals are 16/4/16/6 and 16/2/16/5.
  - Every column total in the admission table matches the sum of its rows.
  - The protocol, pool, pool-amended and TSV SHA-256s match the cited values.
- Judgments sampled against the private screening rows and evidence: all 16 E3 rows, all 6 E4
  rows, all 4 E2 rows, and the E1 rows 5bbb9f6, 80e2d19, 859f691, a6eee82, 7c93f61, bc52224 and
  d59e329. For d59e329 I also read the commit message itself.
- I did not redo Codex's recount of the registration selection counts or its B1–B7 dispositions.

## Blocking findings

### O1. "Every inventoried source was read" is not supported for five rows, and two rows recorded an associated PR as absent

- **Where.**
  - `research/46-fx-zu4f.11-agent-marker-round.md:221-222`: "Every registered object was
    acquired and every inventoried source was read."
  - `:228` says the appendix's sources are "the PRs/issues read".
- **Evidence.** I compared each candidate's evidence inventory, its list of acquired GitHub
  threads, with the builder's list of sources read and with the TSV `sources` cell.
  - **2d3f2f8bc8d4 (E3) and 2e1479b0534f (E1), rank 5.**
    - GitHub's associated-PR lookup, kept in the evidence, returns **PR #62**. That PR is the
      open plugin PR whose branch holds both commits. The evidence also acquired issue #54,
      which links to it.
    - The screening record for each row says there was no associated PR, and lists only the
      commit message and diff stat as read.
    - The TSV `sources` cell for each row lists only the commit and the tree.
    - Protocol §3 requires reading the associated PR(s) and "read the inventoried sources in
      full, so a failure is never explained by a skipped thread". 2d3f2f8's failure is E3, so
      that rule applies to it.
  - **476247f7028b (E3).** PR #982 was acquired, and its body states the bug ("`tick` never
    returned"). It is not in the read list, which names only #1024 and #1021–#1023.
  - **553d9bc121f6 (E1).** #781 was acquired but is not in the read list.
  - **Other rows.** For 204906a, 80e2d19, 859f691 and e2d89d9, the root-body links the builder
    did read are missing from the TSV `sources`. So the appendix does not show "the PRs/issues
    read", as line 228 says it does.
- **Impact.** None on outcomes. I read #62 (body and all three comments), #54, #982 and #781.
  - No thread names a race, a hang or shared-state blocking for these commits.
  - #982 confirms 476247f's E3 reasoning: the hang comes from reading history from before the
    compaction frontier, a deterministic fault.
  - Every judgment stands.
  - Still, a skeptical outsider who opens 2d3f2f8 on GitHub sees PR #62 attached to it, while
    the record claims full reading.
- **Minimum fix.**
  1. The author reads #62/#54 for rows 2d3f2f8 and 2e1479b, #982 for 476247f and #781 for
     553d9bc, and adds them to those rows' `sources`.
  2. Add the root-body links already read to the `sources` of 204906a, 80e2d19, 859f691,
     e2d89d9 and 8a76bbf (#801).
  3. Correct the private read records to match.

  Keep line 222 only if all of this is done. Otherwise soften it to say which threads were read
  per row. The only file that changes is the TSV (and its hash at line 225), and no judgment
  changes.

## Non-blocking findings

1. **E3 standard is applied inconsistently to waits on an external party.**
   - fbcbbb8 passes E3 tentatively: a parent blocked on a hung child process counts as
     "blocking between concurrent activities".
   - df7e31e fails E3: a tunnel that hung 60 s waiting on a server that had already sent
     close_notify is called a "single-threaded TLS-protocol handling fault".
   - The two can be told apart: in fbcbbb8 the other party hangs, while in df7e31e the program
     ignores the end signal. But neither the TSV nor the record says so.
   - Neither outcome changes. If df7e31e passed E3 it would fail E4 on network and rustls, and
     would not be capability-only.
   - Suggest one clause in df7e31e's reason that states the distinction.
2. **One sentence calls the capability rows capability-only without the hedge.** Line 281 says
   "two are capability-only (below)", while lines 339-341 call both tags provisional.
   - fx-zu4f.13 takes its pool from committed records only, so this sentence could be read as
     a firm tag. Suggest "two are tagged capability-only, provisionally".
   - The cap:net row (12a36c9) needs UDP DNS queries. fx-zu4f.13 names `Net::get` (HTTP) as the
     landing network capability.
   - A short note in the capability table would stop fx-zu4f.13 from overreading the tag: it
     should check that the landed capability covers UDP DNS.
   - Otherwise the rows are correctly labelled for fx-zu4f.13 and not relaxed. E3 is marked
     tentative, the union counts are marked INCOMPLETE, and fx-zu4f.13 is told to count the
     union before relying on the tag.
3. **One candidate came in only through the widened keywords.** Line 146 already says the
   whole-word regex is not a strict subset. One of the 42 candidates, **db8db55**, matched only
   through an added inflection (`racing`). I checked this on its commit message.
   - It fails E1, so nothing changes.
   - The bead says the protocol may narrow its criteria but never widen them. The coordinator's
     19:44Z directive sets this exact regex, so the contract holds.
   - A skeptical reader still benefits from one sentence: "one candidate (db8db55) matched only
     an added inflection; it fails E1."
4. **Readability for a cold reader.**
   - The record never says what the arms do or what "caught" means before its headline "not
     caught". Two sentences after the E1–E5 definitions would fix that:
     - B screens past keyword-matched fixes.
     - A ports the pinned concurrent core and tests documented properties.
     - A catch needs the buggy model to fail, two byte-identical replays, a passing
       fixed/control model, and, for A, a reproduction on the real code.
   - "Marker" and "T" are never defined before the "Marker T" table. Give the trailer list
     briefly (`Co-Authored-By: Claude`, `Generated with Claude Code`, Copilot/Codex/…) and say
     that T is the share of default-branch window commits carrying a marker.
   - Add each pool repository's marker share from `activity-markers.tsv` to the pool table:
     RedisJSON 22%, differential-dataflow 64%, insta 64%, claudes-c-compiler 65%, chimera 91%,
     gameboy 23%, Antigravity 74%. Two repositories sit just above the 20% floor, and their
     maintainers would reasonably want readers to see that "agent-marker" means at least one
     commit in five, not agent-written.
   - Also carry over the protocol's note that name substrings (for example `Amp`) can match
     by accident.
   - Say in one clause what fx-zu4f.13 is (a re-screen of rows that failed only for a since-landed
     Proc/Net capability).
5. **Line 36 can be misread.** "No property was supplied" can read as a property of Tide. The
   limit protocol §8 means is that the method supplies properties from upstream text. Suggest:
   "The method supplies each property from upstream text; this round froze none and sampled no
   seed."
6. **Line 273-274 drops a hedge.** It calls both differential-dataflow sites "races and hangs",
   but 59dfd3d's E3 is a tentative pass. Suggest "races and hangs (#801's E3 tentative)".
7. **Line 25-26 says "nothing in this round is undisclosed".** Private screening notes do exist,
   even though none is a finding. Suggest "no finding is withheld".
8. **Housekeeping.** The worktree holds 24 untracked `*.ots.bak` files under
   `research/protocols/`. They are not in the candidate. Keep them out of the landing commit or
   delete them after landing.

## Reasoning

- **Judgments.**
  - **E3 rows.** All 16 E3 failures hold under §4:
    - eight claudes-c-compiler rows are compiler or miscompiled-kernel faults, or a GCC flag
      name, with no concurrency in the compiler;
    - two differential-dataflow tick hangs are deterministic non-convergence, as the PRs'
      own explanations confirm;
    - two chimera rows are UI or state logic;
    - four Antigravity rows are configuration, socket option, state classification and TLS
      handling.
  - **5ab40fa.** Its timing lies in the Windows resolver choosing between remote servers, not in
    the program's shared state. E3 is defensible, and E4 would also fail (PowerShell, network).
  - **E4 rows.**
    - In all six, the deciding operation (timely, network, subprocess, kernel RST behavior)
      is on the path as the protocol registers it.
    - The size claims were correctly withdrawn: the private draft once said ">400" for #801,
      and the public record does not claim it.
  - **E2 rows.** The RedisJSON E2 failures apply "a failure attributed to another defect does
    not pass" correctly. d59e329's folded encoding fixes state no observed failure, which I
    confirmed on the commit message.
  - **E1 rows.** The sampled rows are features, refactors, or fixes to code new in the same
    commit, with no buggy first parent.
  - **Hidden candidates.** No judgment I checked hides an eligible candidate. Every row that
    could plausibly pass E3 fails E4 on an operation the protocol bans.
- **Honesty.**
  - The results lead with negatives, and give no discovery rate and no "Tide found" claim.
  - The record says plainly that marker share is not a measured fraction of agent-written code.
  - It makes no judgment of authors or projects. The framing section and the A2 exclusion
    wording are neutral.
  - The only overstatement is O1.
- **Privacy and disclosure.**
  - 0 private candidates, and no arm-A candidate exists.
  - Every committed quote is from a public commit, PR or issue, with an attribution.
  - The RedisJSON response-formatter problem the record mentions is the maintainers' own public
    statement in #1437, with nothing added.
  - The db8db55 paragraph describes a module that public source references only under a cfg
    flag. That is visible upstream and neutral.
  - I found no sentence a maintainer would reasonably object to, beyond the marker-share
    context in item 4.
- **Consistency.**
  - The record agrees with the protocol's §8 checklist: pool, candidates, first parents,
    duplicate links, E2 quotes, E4 inventories, counts by unit, admission kept separate, and
    the AI-report policies with revision URLs.
  - It also agrees with the bead's comments, with the 23:36Z correction of the protocol hash,
    and with the commit message, which states both tags as provisional.

DONE
