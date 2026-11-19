Model: Claude Opus 5.5 · Date: 2026-11-19 · Candidate beae640 (research record)

# Opus review round: fx-zu4f.15 record (docs and research)

## Verdict

**Changes required.** There is one blocking finding (B1). It needs a text-only fix and leaves the result unchanged. The result holds as written: the evidence supports leg 1 and leg 2, and the caught rule is met with nothing relaxed. B1 concerns disclosure. The record calls the motivated std fix "an ergonomics fix on its own merits" but leaves out a material fact. That fix re-adds a conversion that an earlier review of Tide had deliberately removed because of its cost. The fix commit, which the record cites, says so.

## Scope

- Candidate: `beae640fd0015bc6f6406520a5312dbd0bb3155a` on branch fx-zu4f.15. It adds one file, `research/NN-fx-zu4f.15-chrono-anchor-retest.md` (986 lines).
- Read: AGENTS.md, the bead fx-zu4f.15 with its comments, the bead fx-qlek.18, the protocol `research/protocols/fx-zu4f.15-protocol.md`, fx-zu4f.8's protocol §4 and §5, `research/47` at `fc43b37`, commit `4c5b8c0`'s message and diff, and `reviews/fx-nkv.3-r1-{codex,opus}.md`.
- Evidence checked, read-only. I ran no builds and no admission runs, and made no network requests.
  - final.log and dev.log;
  - screen.sh and dev.sh, each against PRIOR's copy;
  - the port hashes;
  - `undeleted-artifacts.sha256` and the now-empty output folders;
  - the capability run's diagnostics file;
  - the saved block headers in `ots-check/`.
- Already done by others and not repeated: Codex r1 and its two finder re-checks. I did reproduce the OTS check offline, because it was cheap.

### What I verified, and how

| Claim | Check | Result |
| --- | --- | --- |
| §10 is the protocol verbatim | `diff` of record lines 729 to the end against the protocol file | identical; protocol sha `a15c9236…8065` |
| final.log digest | `sha256sum` | `e0029ced…6c36` ✓; file is read-only |
| Leg 1: exit 1, `TD0602` ×2 (15:5, 29:5), `TD0501` ×3 (17:17, 33:17, 30:9); `E0433` ×3, `E0599` ×2 | final.log lines 52 to 66 | ✓ exactly |
| These are counted codes, on in-class operations | fx-zu4f.8 protocol §4 (lines 199 to 209): `TD0501` and `TD0602` count; R' `conformance/rules.md` CAP-1 and PROC-2 keep `Command::new` at `TD0501`. Spawn, process and clock are in-class. All spans fall in the three affected functions | ✓ |
| Leg 2: exit 0, no diagnostic | final.log lines 71 to 81; the `.diagnostics` file holds only harness lines (load, cwd, times, exit 0) and no driver output | ✓ |
| interface.json `dbc20e74…1f43`; authority bg, clock, proc | final.log line 81; `undeleted-artifacts.sha256`; research/47 §8's variants have the same digest | ✓ |
| Ports are byte-identical to fx-zu4f.8's | sha256 of PRIOR's and WORK's buggy.rs, capability.rs and capability.diff: `3c296fb4…`, `51f34851…`, `c8a002fb…` | ✓ |
| §6 diff recheck | body of `diff -u buggy.rs capability.rs` equals capability.diff; record §8 `-U0` hunks equal both the live `diff -U0` and research/47 §8's chrono_anchor hunks | ✓ |
| Harness changes only as §5.6 allows | `diff` against PRIOR's screen.sh: header comment, WT, TARGET, R, RUNS, labels. dev.sh likewise (sha `d4c8bfd6…b42f`) | ✓ (see N4) |
| Incorporated digests | `br --no-db … --json` hashing of the .8 description `f20b5667`, the rule block `7cb2479c`, comments 1106 `7ad11899` and 1312 `cd8d3ba0`; the RULING files; the .8 protocol and its .ots; research/47 at fc43b37 `6c6111b7` | ✓ all |
| §4 claims at R' | `impl From<Cancelled> for TaskError` at task.rs:1891; the companion row before and after; e0b0d62, 8ac1317, d378a7d and 4c5b8c0 are ancestors of R'; 17 turbofished sites (commit message) | ✓ |
| §3 printed OTS script | the listing equals `ots-verify-short.py` byte for byte; run offline on the saved header (bs and mp headers identical) with the three committed `.ots` files: `hash ok: True`, 05:34:04Z, 976536, `afd6390e…18dd` ×3, `ALL OK`, exit 0 | ✓ |
| Git times and the hash link | commit dates (−03:00) convert to the Z times stated; `442f8a9` resolves uniquely; it is not an ancestor of 4c5b8c0; 4c5b8c0's message names it | ✓ (see N2) |
| Safety | the commit adds one file. No port body appears, only the `-U0` hunks §5.8 allows (placeholders `<program>`, `<arg>`, `.invalid`). The ports hold no IP address. Admitted artifacts were deleted (output folders empty) | ✓ |
| Wording rule | "exposed", "gap", "fixed", "new registration" appear in §1 and §9. "first time" appears only in negations. "0 caught of 87" is kept. "not a rate" appears twice | ✓ |

## Blocking findings

### B1. The motivated std fix leaves out that it reverses an earlier deliberate removal of the same conversion

- **Where**
  - `research/NN-fx-zu4f.15-chrono-anchor-retest.md:38-42` (§1: "fx-qlek.18, an ergonomics fix on its own merits");
  - `:289-291` (§4: the turbofish is stated, but not why it matters);
  - `:704-706` (§9 negatives).
- **Evidence**
  - `git log -1 --format=%B 4c5b8c0`: "This deliberately reverses fx-nkv.3 r1 B1 (reviews/fx-nkv.3-r1-codex.md B1, reviews/fx-nkv.3-r1-opus.md B1), where the conversion was removed for its E0283 inference cost. It returns under the owner's 2026-11-19 outcome."
  - `reviews/fx-nkv.3-r1-opus.md:41-61`: B1 records that "`From<Cancelled> for TaskError` breaks `scope(...)?` inference in spec examples" and prescribes "Delete `impl From<Cancelled> for TaskError`". `reviews/fx-nkv.3-r1-codex.md:7,57` says the same.
  - The record never mentions fx-nkv.3 or the reversal (grep for `nkv`, `revers`, `removed`).
- **Why it blocks**
  - The brief asks whether the motivated fix is understated, and the record goes out as a public positive result. Read alone, "Tide's std had a gap … an ergonomics fix on its own merits" sounds like an oversight was filled in.
  - What happened is different. Tide's reviewers had removed this exact conversion on purpose, because it costs `scope(..)?` inference. The decision was reversed after this screen exposed the row. Every unannotated `TaskError` scope now pays that cost: 17 sites, one of them a normative design example, 02 §10.12.
  - A skeptical outsider finds this one hop away, in the commit the record cites as the fix. Finding it there, rather than in the record, would read as concealment and would undercut the disclosure §1 exists to make.
  - The motive itself is stated plainly. Without the reversal, though, the "on its own merits" clause overstates how independent the fix is.
- **Minimum fix** (text only, record only; the protocol is not touched, since its §1 allows "text with the same meaning" and this sentence only adds):
  - In §1's "Why it was re-tested" bullet, after the fx-qlek.18 clause, add: "The conversion fx-qlek.18 adds had been removed earlier, on purpose, by fx-nkv.3's review (`reviews/fx-nkv.3-r1-codex.md` and `-opus.md`, B1), because it breaks type inference for an unannotated `tasks.scope(..)?` (rustc E0283). The owner's 2026-11-19 decision reversed that removal. The cost now falls on users: 17 scope sites name their error type (design/03 §6.1)."
  - In §9's negatives, add "and the change reverses an earlier removal made for its inference cost (section 1)" to the bullet at `:704-706`.
  - Optionally, cross-reference §4 `:289-291` to §1.

## Non-blocking findings

- **N1. Same-campaign siblings.** research/47 lines 446 to 449 show four siblings of chrono_anchor in the "time-utility .env exfiltration (2026)" campaign: time_calibrator, time_calibrators, dnp3times and time-sync. All four were "not caught (no portable source)", so C1 rightly excludes them. A reader seeing "one catch" may ask about the rest of the campaign. One clause in §6's selection bullet would answer that ahead of time: "including four same-campaign siblings with no portable source".
- **N2. Message line numbers.** `:101` says "lines 41 to 43 of the message body". In `git cat-file -p 4c5b8c0` those lines are 46 to 48. They are lines 41 to 43 of the message only if the subject line counts as line 1. Say "of the commit message, subject line included", or drop the line numbers. The quoted text is correct.
- **N3. The `.diff` file is not in final.log's before/after hashes.** `:425-426` says each port file "was unchanged before and after every run". final.log's `portsums` hashes only the two `.rs` files. `capability.diff`'s hash still matches (`c8a002fb…`), and it was checked when copied and at the diff recheck. Narrow the sentence to match the evidence.
- **N4. Harness edits.** Protocol §5.6 lists only "log path, output root, run list and labels" as allowed changes. The actual edits change WT, TARGET and R, from which the log path and output root are derived; §5 and §5.1 authorize those as name and revision substitutions. Record `:354-362` describes the changes correctly. Adding "(the name and revision substitutions of §5.1 and §4)" would make the authority explicit.
- **N5. The 4c5b8c0 author date is evidence the record doesn't use.** `4c5b8c0` has author date 05:38:16Z and committer date 06:35:00Z. The record's claim "before fx-qlek.18 was implemented" (`:79`) fits this: the registration came at 05:24:05Z, 14 minutes before the fix was first authored. Both dates are self-reported, so this is optional context only.
- **N6. Untracked files in the worktree.** The worktree holds untracked `*.ots.bak` files, the pending proofs before the upgrade, under `research/protocols/`. They are not in the candidate. Keep them with tide-lead's evidence or delete them when the worktree retires, so they are never committed by accident.
- **N7. Style.** §3's ordering bullet runs about 150 lines, so the result's one qualification sits deep inside it. §9 repeats that qualification in short form. That is enough for readers, and Codex's re-checks already approved this text. No change is needed.

## Reasoning

**Result (review item 1).** final.log supports every result claim in §2, §6, §7 and §8, and the claims match the log exactly. The counted codes are the ones fx-zu4f.8's §4 counts, read against R''s rules.md. Leg 2 is a clean admission: the diagnostics file holds only harness lines. The trust artifact's digest and authority match. Port fidelity holds three ways:

- the three files are byte-identical to fx-zu4f.8's PRIOR copies and to research/47 §8;
- the diff body and the `-U0` hunks are unchanged;
- no fix was applied, so §6 has no numbered fix to list.

The harness changed only names, the revision, the run list and the labels. The caught rule is fx-zu4f.8's, and its digest checks out. Nothing in the record reads it more loosely. I tried to see the rule as relaxed and could not: the table form at R' stays `clock.sleep(d)?` because the scope's error is `TaskError` (companion row at R'), and RULING-0039 forbids `let _ =` and the added `.map_err`, and neither appears.

**Honesty (review item 2).**

- Section 1 opens the record, as protocol §1 requires.
- The mandated wording is present in §1 and §9. "First time" appears only in negations. "0 caught of 87" and fx-zu4f.8's outcome for this row are kept. The result is reported as one row, admission only, nothing executed, and not a rate.
- The ordering qualification appears where readers meet it, in §3 and §9, and is not overstated.

The one gap is B1. The record discloses that the screen motivated the fix, but not that the fix reversed a deliberate, review-backed removal and carries a known cost for users. That is the kind of fact a skeptical outsider uses to discount the result, and it is already public in the cited commit. Disclosing it costs one sentence.

**Safety (review item 3).** No port body, payload or archive-derived code is committed beyond the `-U0` hunks §5.8 allows, which hold only placeholders. The admitted runs' artifacts are listed with hashes and are gone from disk. The `/home/lucca` paths in the commands follow research/47's precedent.

**Readability (review item 4).** Terms are defined up front, the two legs are explained, and the limits are stated. §3 is long (N7).

**Consistency (review item 5).** The record agrees with the protocol, with the bead comments (R', digests, the go and the final.log hash), with the commit message, and with `target/work/report.md`, apart from B1's omission.

DONE
