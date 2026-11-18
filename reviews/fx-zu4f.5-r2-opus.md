Model: Claude Opus 5.5 · Date: 2026-11-18 · Candidate 4289279 (docs)

# fx-zu4f.5 r2: Opus review round (docs, reports only)

## Verdict

**Changes required.** There are two blocking findings. Both have text-only minimum fixes, and neither changes a count or an outcome. The search itself holds: I found no advisory wrongly marked not caught, and no false negative in the screen.

## Scope

- Candidate: branch `fx-zu4f.5`, HEAD `42892798344ce5f9513d325eecab7932b891d92b`, base `cdf47ec`. SHA-256 of `git diff cdf47ec..HEAD`: `67a04be72d5cda0bf5c32f78b1802ca6d0c2c505d6b43e1d42a8d9043c20c179`. One file, `research/37-external-admission-search.md` (+892). `git diff --check` is clean.
- Read: AGENTS.md, the Merge Rule in GOVERNANCE.md, the bead and all its comments (`br --no-db`), the whole record, `target/work/protocol.md`, and the three Codex files (only to avoid repeating them).
- Spot-checks:
  - `protocol.md` hashes to `0e5cebad…24fb0`, and the fenced copy in the record hashes to the same value.
  - The record's 87 table rows are the same set as the 87 `y` rows of `screen-full.tsv`: 75 rows say "no portable source" and 12 do not.
  - `crates/`, `Cargo.*` and `rust-toolchain.toml` are identical at `33085fd`, at the rerun's `5dc6b40` and at HEAD.
  - All 60 P2 items in `search/q1..q6.json` have `state: open`.
- Advisory text read at `233155b4` (prose only): chrono_anchor, rustdecimal, logprinter, evm-units, sha-rust, postgresderive, time_calibrator, rands, replit_ruspty, polymarket-clients-sdk, polymarket-client-sdks, rpc-check, tracing-check, tracing_checks, build_dep, greentic-setup, arrayref, arone, proc-macro1, onering, mysten-metrics, logtrace, pretty-changelog-logger, exploration, jfrog_quotes, tracing-ethers and finch-rst. That is 27 of the 87, 5 of them quoted-code cases. I also read the quoted snippets in `sources/socket-1.txt`, `-2.txt` and `-3.txt`, and compared both chrono_anchor port files with Socket's snippet.
- False negatives among the 1,420 non-qualifying rows:
  - I scanned every one of them for 14 terms (malic, malware, backdoor, exfiltrat, typosquat, steal, trojan, "removed from crates", credential, cryptocurrency, wallet, payload, compromised). About 150 matched. I read each matched row's reason, and the full text of http-types, theshit, wasmtime-0378, tinymember and rustpython-vm.
  - I also scanned for "removed from crates.io", "yanked…malic" and "account banned/locked". Only safe-agent-rs and tinymember match, and the screening notes exclude both correctly.
  - Every exclusion I checked is right.
- Driver code read to judge the claims about what the capability ports' runs show: `crates/tide-admit/src/invocation.rs` (`after_expansion` and `after_analysis`) and `crates/tide-admit/src/walk.rs` (`typeck_of`, `can_select`, `visit_boundary`, `reads_captures_only`).
- Nothing was executed, built or fetched.

## Blocking findings

### B1. "The scope tasks and the clock are admitted" goes beyond the evidence

`research/37-external-admission-search.md:355-356`, and the claims built on it at `:9-10`, `:522` and `:693`. The amaperf version of the same inference is at `:311-312`.

What the record says: the chrono_anchor capability-passing port "fails only with `E0599` … The scope tasks and the clock are admitted". The summary at `:693` repeats this as "chrono_anchor fails only on the second gap".

What the evidence shows: `rerun.log` (lines 128-132) has two `E0599` errors, then "aborting due to 2 previous errors". Both errors sit inside the `bg.spawn` closures of `fetch_ref_background` and `submit_snapshot`. A closure is type-checked with the function that encloses it, so those two functions' typeck results are tainted. The driver then skips its own checks on exactly that code:

- `typeck_of` drops tainted results (`walk.rs:471-474`), and `can_select` is false (`walk.rs:481-483`).
- `reads_captures_only` returns `true` without checking (`walk.rs:1018-1020`).
- The mono-item backstop never runs, because "rustc's analysis stops before this on any error" (`invocation.rs:213-214`).
- rustc's borrow checker also skips tainted bodies.

So the absence of TD codes on `bg.spawn` and `clock.sleep` in this run does not show that they are admitted. A public reader, and the launch line "narrowly missed", would take "fails only on `Proc::command`" to mean that adding `command` is enough. This run does not establish that. The re-screen (fx-zu4f.8) must find out for itself.

amaperf has the same problem in a milder form. "No `TD0501`, so the env and fs reads are admitted through `Env` and `Fs`" (`:311-312`) comes from a run that also failed resolution with `E0433` and `E0425`. That run shows the port names no ambient env or fs item. It does not show that the reads are admitted.

Minimum fix (text only):

- At `:353-356`, say: the run reports only `E0599` (×2) for the missing `command`. That type error stops rustc's later analysis of the two functions, and Tide's checks that need their types, so the run does not show whether the rest of the port would be admitted.
- At `:9-10`, change "cannot be admitted … because" to "is not admitted …: `Proc` has no `command` yet, and the check stops there".
- At `:522`, change "fails only on" to "fails on".
- At `:693`, say chrono_anchor "fails first on the second gap; whether it fails on anything else is untested".
- At `:311-312`, say "no `TD0501`: the port names no ambient env or fs item".

### B2. The conclusion misstates why the quoted-code cases fail

`research/37-external-admission-search.md:684-691`

The text says the caught rule's second leg (admitting the capability-passing version) "fails on one of three gaps in Tide v0" in "every case where the code was quoted". This is false in two ways:

- **finch-rust and uniswap-utils fail the first leg, not the second.** Their buggy ports get no TD code at all, only `E0433` (`:382-383`, `:395-396`). Their only effect is a call into the malicious dependency, so there was nothing ambient for admission to reject, and no capability-passing version was ever relevant. The third bullet calls this a "gap in Tide v0", which it is not: no new Tide API would change either result.
- **evm-units and sha-rust hit several barriers, not one.** evm-units needs an external HTTP client, tokio and `async`, and a process run, all at once (`:377-378`). sha-rust needs `reqwest` and UDP (`:389-390`).

This bears on review point 3. As written, the conclusion implies that every quoted miss is one missing v0 API away from a catch. That pre-empts fx-zu4f.8 and overstates what adding `Proc::command` and `Net` will do.

Minimum fix (text only):

- Say the second leg fails, in each of amaperf, chrono_anchor, evm-units and sha-rust, on at least one of: an external HTTP client (no `Net::get`), a process run (no `Proc` operations), async or tokio, or UDP.
- State finch-rust and uniswap-utils separately: no TD code fires on their buggy ports, because their only effect is a call into the malicious dependency.

## Non-blocking findings

- **N1. Say that P2 could not test admission** (`:6`, `:38-39`, `:569-571`). The tables show that all 60 results were open. Most were opened in the three months before the search, because best-match ranking favours recent items. The text says only that the 20 eligible results were open. Add one sentence, for example: "Every one of the 60 results was open at search time, so P2 as registered contributed no candidate and tests nothing about admission." This underclaims rather than overclaims, but a reader should not count P2 as a second independent test.
- **N2. Drop the argumentative close** (`:693-696`). "It is not evidence that admission misses these effects" argues for something the search did not test. That includes the 75 entries with no portable source, for which nothing is known either way. After B1 and B2, end on the scoped facts: which TD codes fired on which ported calls, and which leg failed for each case.
- **N3. Class labels follow the incident, not the advisory** (`:460-461`, for example `:508`, `:534-539`). polymarket-clients-sdk is labelled "env, fs, net (per advisory)", but its advisory says only "steal credentials from local files". Only rpc-check names an environment variable. The arrayref group's "net, process (per advisory)" comes from proc-macro1's and the blog's "download a malicious payload", not from each advisory. Relabel these entries "per incident", or narrow them to each advisory's own words. No outcome changes.
- **N4. Unclear prose** (`:29-32`). In "Its relay carried tide-desk's conditions" and "ran outside `target/work`, against one of them", the antecedents are unclear. Suggestion: "The relay added tide-desk's conditions. The archive-derived ports broke one of them, because they ran outside `target/work`; see …".
- **N5. Optional: add the malware-proofing caveat.** The record discusses malicious crates and what admission rejects in them. One line from the bead's framing would stop a public reader stretching the TD0501/TD0602 rejections into a malware-defence claim: no malware-proofing claim, because malicious code can still try rustc soundness holes (reviews/2026-11-17-red-team-opus.md). Today the caveat appears only inside the verbatim protocol (`:883-885`).
- **N6. Out of the candidate, for whoever reuses the port files.** The header comment in `target/work/ports/chrono_anchor/buggy.rs` calls the no-network-code rule an "owner condition". It was tide-desk's condition (bead comment, 13:05Z). The record itself gets this right (`:255`, `:262-264`). Correct the comment before the preserved copies feed fx-zu4f.8.

## Reasoning

**Honesty and framing.** The record does what a pre-registered negative should:

- It states the result first.
- It refuses to call the counts a rate (`:59-60`).
- It keeps the bead-named faster_log and async_println out, because they have no advisory, rather than adding them.
- It reproduces the protocol verbatim, and the hash matches.
- It discloses three things plainly and in proportion: the first screen's real scope (`:81-86`), the zz-probe deviation with tide-desk's disposition (`:262-275`), and the rerun that replaces the unretained command lines (`:159-164`).

I see no sign of tuned exclusions. Every protocol narrowing applies before candidates are examined and works against a catch: the star check comes after the ten-result cap, with no refill, and missing code rules out a port. A skeptic's strongest line is the opposite one: the design made a negative cheap. P2 returned only open issues, and 75 of the 87 P1 entries have no quoted code. N1 answers the P2 half. The P1 half the record already states honestly.

The overreach sits in the two places where the record explains why the result came out negative (B1, B2). Both make the result look closer to a catch than the evidence shows. That matters because the launch story ("narrowly missed") and fx-zu4f.8 will lean on it.

**Screening and assessments.**

- chrono_anchor's buggy port follows Socket's snippet structurally: three functions, two detached `thread::spawn`s, one `thread::sleep`, two `Command::new`. Only hosts, paths, the program name and arguments are replaced with placeholders. `TD0602` ×2 and `TD0501` ×3 are what that code should get. The outcome, not caught, is right under the frozen rule, whatever B1's answer turns out to be.
- evm-units' and sha-rust's descriptions match the quoted code: `Runtime::block_on`, `async fn check`, `Client::builder`, `temp_dir`, `File::create`, `Command::new("nohup")` inside `unsafe`, `.spawn()`; and `fs::read`, `env::var`, `UdpSocket::bind`, `reqwest::blocking::Client`.
- rustdecimal, logprinter, postgresderive, time_calibrator, rands and replit_ruspty match their advisories and the stated reasons.
- I found no wrongly marked outcome, and no wrong reason apart from B2's summary and N3's labels.
- I found no false negative in the screen. rustpython-vm (RUSTSEC-2026-0394) is a real ambient-environment bug, but it is accidental, so it is correctly outside P1, and it is not in P2. I note it only as an observation; it needs no change.

**Usefulness and the launch claim.** The record says plainly what v0 lacked: `Proc` "has no operations yet", and `command` is planned (`:359-364`). It neither mentions A1 nor promises a result for fx-zu4f.8. "It would need to be re-checked once `Proc::command` exists" (`:363-364`) is a fair pointer, not a promise. Once B1 and B2 are fixed, the record stands alone as a negative and does not presume what the new re-screen will find.

**Prose.** The prose is clear and mostly plain. The tables are dense but complete. Apart from N4, I have no style concerns.

DONE
