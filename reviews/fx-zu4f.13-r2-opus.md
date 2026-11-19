Model: Claude Opus 5.5 · Date: 2026-11-19 · Candidate 31da950 (research record)

## Verdict

**changes required.** Two blocking findings, both text-only (wording or citation; neither changes a row's outcome, a count or a rule). Under GOVERNANCE.md's Merge Rule a text-only fix needs no re-check. High confidence that every row's outcome follows from the registered rules as written. P4's deciding criterion is fair, and the P1 non-independence account is sound and uses the mandated wording.

## Scope

Read first: AGENTS.md, GOVERNANCE.md, `br --no-db show fx-zu4f.13` and `br --no-db comments fx-zu4f.13`. Then I read the candidate record (research/50, 432 lines) against the registered protocol `research/protocols/fx-zu4f.13-protocol.md`, along with fx-zu4f.11's protocol (E3/E4 text, lines 425-505), fx-zu4f.9 §4 ("an external crate's operation is never mapped", line 298), research/37's chrono_anchor port account, research/47's table-form history, research/49, and the chrono_anchor texts at R (README.md:68-182, paper/tide-v0.typ:210-212).

For evidence I read `target/work/property-log.md`, `report.md`, `answer-0842.md`, `ports/final.log` (diagnostic lines only) and `/home/lucca/.cache/eon-agents/tide-lead/builder-rules.md` (the reading-observation rule). I also read Codex r1, the r1 re-check and tide-lead's selector replay. I did not repeat their identity, hash, OTS, selector or P2/P3 source checks.

I independently re-read P4's first-parent source `db8db55` in fx-zu4f.11's frozen clone (`git show`, read only): `utils.rs:15-19` (Windows `no_window` with `creation_flags`), `:22-24` (identity), `:28-32` (`powershell`); `dns.rs` `setup_dns_nrpt_with` (`egress::detect`, `probe_if`, `nameservers_for` before `powershell`); and `dns_client.rs` `query_raw_via` (`UdpSocket::bind`, then `sys::bind_socket_to_interface` when `if_index != 0`, Windows FFI at `:27-62`). The record's P4 citations match.

Not done: no build, admission, test, network access, commit or `.beads/` write. Only this file was written. No port body or archive-derived code is quoted here.

## Blocking findings

### B1: an absence claim about private material in the public record

**research/50-fx-zu4f.13-landed-capability-rescreen.md:342-343.** "**Private finds by the method:** 0. No row reached a model, so the method found nothing new on P2-P4's paths."

Evidence: the count is correctly scoped to the method. The second sentence goes further: it asserts that nothing new was found on P2-P4's paths. Protocol §6 defines the private directory as holding "anything new found on P2-P4's paths" (protocol:358-363), and it requires the public record to give "a count only" for private finds (:373). Arm A's method includes locating and reading the core, not only modelling it, so "no row reached a model" does not establish that nothing was found on those paths. Whatever the private directory holds, a public absence claim about material that §6 keeps private can be neither verified nor kept honest later. If anything private on those paths is ever disclosed, the sentence reads as false. This is an overstatement, and it bears on question 3 (launch honesty). I do not say here whether the private directory holds anything. tide-lead can check this sentence against the run's private-handling notes in WORK.

Minimum fix (text-only): delete the second sentence. Keep the scoped count, for example: "**Private finds by the method:** 0 (no row reached a model). Anything noticed while reading, outside the method, is held privately under protocol §6 and is neither counted nor described here." That sentence holds whether the number of such items is zero or more, so it announces nothing. If tide-lead reads §6's "count only" as covering items noticed while reading, that is tide-lead's call. Either way the absence claim must go.

### B2: the per-row landing-commit citation that protocol §4.1 requires is missing, and §5's substitute statement is loose

**research/50-fx-zu4f.13-landed-capability-rescreen.md:88-90, 124-125, 163-164, 302-306** (the row entries) and **:333-335**.

Evidence: protocol §4.1 (:247-249) says: "'The capability's landing commit named' means the record cites the landing commit (§3.2) in each row's E4 (or leg-2) entry." None of P2's E4 entry, P3's portability entry, P4's E4 entry or P1's leg-2 entry cites `e0b0d62`. P4's header (:163-164) does not even say that the subprocess ban was lifted. The record instead aggregates the commits in §5 (:333-335) and says that "P2-P4 failed before any capability mapping was relied on". That is not accurate as stated: each of P2-P4 got past the source round's subprocess ban only because Proc::command's operations were lifted. P4's path, for example, builds the child with the mapped `Command::new` and `.args` before `no_window`. The registered requirement exists so that a reader sees, in each row, which landing each judgment depends on.

Minimum fix (text-only): add "(Proc::command, fx-ayh2.1, `e0b0d62`)" to each of the four entries named above. In P4's header, say that the subprocess ban is lifted for §5.1's operations, as P2 and P3 already say. Reword :333-335 to something like: "Proc::command (`e0b0d62`) lifted the subprocess ban for every pool row. P1's leg 2 relies on its mappings, and P2-P4 fail E4 or portability on a non-capability operation. Net::get and Net::resolve map no pool row (protocol §5.2)."

## Non-blocking findings

1. **P4: show that the negative does not hinge on an incidental flag (question 1).** `research/50…:194-200`. The deciding operation, `creation_flags(CREATE_NO_WINDOW)`, only suppresses a console window and plays no part in the hang. A fair reader accepts it as the decider, because §5.1 named `creation_flags` as "no mapping" in the registration, before go. Still, the record is more convincing if it says plainly that (a) the flag is incidental to the bug and the row fails on the registered rule, not on anything about the hang, and (b) the E4 negative also stands without it. The same path makes UDP DNS probes before PowerShell (`dns.rs` `setup_dns_nrpt_with` → `nameservers_for` → … → `dns_client::query_raw_via`), and those are a network operation outside §5, which §4.1 keeps banned. On Windows, when `egress::detect` finds an interface, that path also calls the `unsafe` `ws2_32` `setsockopt` FFI (`dns_client.rs:27-62`, `query_raw_via` binds when `if_index != 0`). That FFI is an E4 exclusion apart from UDP. The record names it only for the UDP-only row (:234-239), but it applies to P4's path too, and it would strengthen "not UDP-only" (:208-210) beyond the flag.

2. **P4: the OS-trigger sentence is stated flatly (question 1).** `:211-212`: "The hang's cause, WMI stopping, is an OS service, which the OS-trigger exclusion covers." No source protocol defines an "OS-trigger exclusion" under that name. fx-zu4f.11's E4 bans "futex/OS wait queues/kernel mechanisms" (protocol :490-491), and this round's §5.3 explicitly allows child-model data such as "never exits". A child that stops answering could be modelled that way. The bullet is labelled not counted, so it decides nothing, but as written it is the one place that leans negative. Hedge it ("may also fall under …; §5.3's 'never exits' data could instead model it; not decided here"), or drop it.

3. **P4: disclose the revision history in one line.** The run first recorded P4 as failing E3 (property log, 08:15:11Z). Codex r1 B1 overturned that, and the E4 decider was identified afterwards (correction at 08:54:51Z). This will be visible in `reviews/`, but a methods reader of the record alone would want one sentence saying so, under "E3, pass, as written". The current outcome follows the registered rules; the point is transparency about a negative that was found again.

4. **UDP trigger wording (question 3).** `:232`: "not shown: the row is not ported (§4.3), so the UDP trigger does not fire from this round". The reason is the right one, and the record honestly adds the possible FFI blocker. But "does not fire" can be read as "UDP turned out not to be needed". Suggested wording: "not evaluated: the protocol lists this row without porting it (§4.3), so this round neither fires nor rules out the UDP trigger". The bead's 08:45 comment uses the same shorthand.

5. **P1's `?` and §4.2.** `:279-282`. §4.2 bars "an added `?` conversion" (protocol :284-285), and the port adds `?`. The record explains removing `let _ =` but not why the `?` is allowed. Add one clause: the `?` is the table form `clock.sleep(d)?` (companion §14.2; research/47:121), and §4.2's own last sentence (protocol :285-287) treats the table form's `?` as the form to judge, not as an added conversion. Without the clause, a skeptical reader sees an apparent §4.2 breach.

6. **Basis for "counted once, under research/49".** `:70-71, :257-261`. Protocol §4.4 leaves the choice to "whichever registration the fx-qlek.7 reader accepts". The record grounds its choice in an unpublished coordinator message ("tide-lead, 08:44Z"). The committed fx-qlek.7 texts at R already cite research/49 for this catch (README.md:71 and :143; paper/tide-v0.typ:210). Cite those, so a cold reader can check the claim.

7. **One quotable line without its qualifier (question 2).** `:307-308`: "**Result.** Both parts of fx-zu4f.5's caught rule hold at R." Quoted alone, this could be read as an independent catch. Append "(not independent of fx-zu4f.15; counted once under research/49)". Every other mention of the catch already carries the qualifier or sits in a paragraph that states it. I found no sentence that presents P1 as a second independent catch or as a first-time catch. The mandated wording appears verbatim at :65-66, :248-249 and :268-270.

8. **Undefined terms for a cold reader (question 4).** "leg 2" (:334; legs are defined in research/49, not here), "amendments A1-A2" (:163), "Socket archive copy socket-3" (:347), and "WORK", "TARGET" and "tide-lead" (:418). One glossary line each, or rewording, would fix them.

## Reasoning

**Question 1: row outcomes follow the registered rules.**

- **P2** (E4: external crate `clang_sys` on the affected path): follows fx-zu4f.10's E4 and fx-zu4f.9 §4. The call is the function's first statement and it selects the child program, so the starting-state exception cannot apply.
- **P3** (arm-A portability: the non-optional `proc_macro2` on the core path): follows fx-zu4f.10 §6. The 164-line count is correctly labelled a lower bound and does not decide the row.
- **P4**:
  - E3 now applies fx-zu4f.11's disjunction as written ("deadlock or hang … in shared state/blocking between concurrent activities"). The child-hang ambiguity is disclosed. That fixes r1's unregistered exclusion.
  - The E4 decider is on the waited-on call's own path. §5.1 names it verbatim, and the protocol was registered before go. The cfg reading (the Windows variant is the affected path because the reported hang involves CIM/WMI/NRPT) is disclosed. It is also consistent with fx-zu4f.11's own rule to "retain production platform/feature branches" (protocol :207-208), which the record could cite.
  - My concerns are about presentation (non-blocking 1-3), not the outcome. The row is robustly not caught, and UDP-only status is ruled out on more than the flag.
- **UDP-only row and resolve-only list:** both match the registered selection.

**Question 2: P1.**

- Protocol §4.4's non-independence, the fx-qlek.18 ancestry, the counting-once rule and the three limits (four previously screened rows chosen after they were seen; admission and type-checking only; nothing archive-derived executed) all appear beside the first catch (:59-82).
- They appear again in §4 (:248-271) and in "What it shows" (:310-313), which honestly says the outcome was predictable.
- The final.log diagnostic lines match the stated TD codes and rustc codes.
- Only non-blocking 5-7 remain.

**Question 3: launch honesty.**

- The UDP trigger's stated reason is the real one: §4.3 does not port the row. Non-blocking 4 asks for wording that cannot be misread.
- I found no port body, archive-derived code or private finding in the record. Ports stay uncommitted under the archive condition.
- The one honesty defect is B1's absence claim. It reveals nothing, but it asserts more than a public record can about §6-private material.

**Question 4: cold reader.**

- Negatives come first (§2.1, then §2.2, then rows P2-P4 before P1). Counts are stated as "not a rate". The glossary covers R, RC, the arms, E1-E5 and the TD codes.
- At 432 lines, the record is auditable. Most of its length is §7's proof list, which a reader can skip.
- Non-blocking 8 lists the remaining undefined terms.

DONE
