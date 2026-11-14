Model: claude-opus-5-5 · Date: 2026-11-14

## Verdict

**approve.** No blocking findings. Three non-blocking comments; the author may take or leave them.

## Scope

Read-only GOVERNANCE docs-only review round of branch `fx-5i0o` at `a2ccfe301b221c5ec8780d5730147e16d60beeed`. `git show --format= HEAD | sha256sum` = `db94eef940e90dde920c4bc18c326be87dd4228a429e60482f8f05420c66c20c`, which matches the brief. The diff touches only `research/31-bend-update.md` (new) and one-line pointers in `research/01-bend-hvm.md:17` and `research/11-bend-interop.md:29`. No design, reference, companion, conformance or example file changed.

Read: AGENTS.md (Decisions, AP-SCOPE-001, tier rule), bead fx-5i0o (`br --no-db`), D16 (`design/01-design.md:1008-1028`), companion §10 (`design/03-authority-concurrency.md` around 2395-2455), report 11's claims on stdin, flags and U32 opacity, the Codex review (`fx-5i0o-r1-codex.md`), and `crates/tide-std/src/lib.rs` for `Proc`. I did not redo Codex's link and date sweep. The date nit at line 367 now reads "September–October 2026 runs". I opened nothing under trials/e1-checked/ and ran nothing heavy.

Spot-checked against primary sources, pinned to Bend `f0cf0a7`:
- The compare from f311527 to f0cf0a7 shows 5 commits ahead and 0 behind, and v2.0.40 to f0cf0a7 shows 38 ahead. All five commit subjects match the F1 table. PR #2143 confirms that the bug left a closure out of the GPU program, and #2155 that the fix flushes stdout before helper-thread jobs.
- Releases: v2.0.40 was published 2026-11-06T02:06:29Z and v2.0.38 on 2026-10-20. The 2.0.38 notes confirm `+RTS ... -RTS`, that plain `--threads 4` goes to `IO.args`, and `BENDRTS`. `comp.ts:5885` reads `BENDRTS`.
- #813 is `closed`, `not_planned`, closed 2026-09-19. WONTFIX confirms these lines: 125-126 (the 2^48-1 fail-stop), 118 (unbalanced Array fail-stops), 137-142 (U64 and F64 in SOON), 152-154 (library target "planned, not scheduled") and 54/108 (F32 lane differences).
- In base.bend, line 64 defines `U32{data: Word(32n)}`, line 1539 defines `U32.add` in Bend, and lines 1544-1552 prove `U32.add_comm`. F32 operations are bodiless `law` declarations (e.g. line 1764). `Process.run` (line 235) writes to a child's stdin (GUIDE.md:385).
- `file_open.c` calls POSIX `open(path, flags)`. `file_read.c:18-19` uses `read` for `File.read` and `pread` only for `read_at`.
- Wakamex PERFORMANCE.md: the 2026-09-17 run used Bend 2.0.3 `b9d1352` on a 3950X, with medians of ten runs. The 0.742/0.824, 0.834/0.343, 1.689/0.087 and 7.797/2.263 (7.368 one-thread) figures, the 1.32 median and the 13.58 median all match their stated columns. STARTUP_SCALING.md gives 5.7×, 1.3× and 300×, five alternating rounds, fitted work terms and 2.0.3.
- checker-eval README: "Our result (Oct 10, 2026)", 20/20 for $18.24 and 0/20 for $12.92, a blind arm limited to one reply, and the stated rules on unchanged task text, no redefinition and no unsafe code. The commit is dated 2026-10-22.
- The Taelin posts of 2026-11-04 14:23Z and 2026-11-12 17:31Z, read via the fxtwitter mirror, match F2's wording: "certified, with zero bugs, and already 2x faster", and "the language will remain the same".
- `tide-std` `Proc` is documented as "Authority to run processes and exit. It has no operations yet." This supports R2's prerequisite.

## Blocking

None.

## Non-blocking

1. **The U32 correction leaves out how thin Base's proof coverage is.** `research/31-bend-update.md:126-131`.

   **Evidence:** The correction is right that U32 is proof-visible. At f0cf0a7, base.bend has only five `law U32.*` items: `add_comm` (1544), `to_f32` (1756, bodiless) and three `show`/`read`/`hex` helpers. It has no laws for sub, mul, div or mod. The source report 11 quoted (Akita, `docs/findings.md:82-84`) says "opaque to proofs" while explaining that exact proportional shares "need division lemmas, a multi-day job". Its practical obstacle therefore still stands. The definitions were also already transparent when report 11 was written. base.bend at `e5527d5` (the last base.bend commit before 2026-10-19) already defines `U32{data: Word(32n)}` and `U32.add_comm`. So this is a correction of report 11, not an upstream change. Line 264's wording is consistent with that, but the report never shows it. Report 11's new pointer says report 31 "supersedes … U32 proof opacity", and an agent reading only that pointer could take it to mean U32 contracts are now cheap to prove.

   **Fix:** Add one sentence after line 129, e.g.: "Base proves only `U32.add_comm` among U32 arithmetic laws; sub/mul/div/mod lemmas are absent, so U32 claims are provable in principle but costly in practice (report 11's division-lemma example stands). The definitions were already proof-visible at the 2026-10-19 snapshot (`e5527d5`)." R1's guard at 286-288 ("not silently expanding its accepted fragment") already covers the design side.

2. **The Stage 0 row does not recheck the companion's chosen transport.** `research/31-bend-update.md:250`.

   **Evidence:** The row cites arguments, file effects and stdout. The companion's specified process protocol, however, is a loopback TCP kernel server (`design/03-authority-concurrency.md`, "Process protocol"). The pinned Bend still has it: `TCP.listen`, `TCP.accept` and `TCP.try_accept` are at base.bend:401-410, and `tcp_listen.c`, `tcp_accept.c`, `tcp_recv.c` and `tcp_send.c` are in `bend2/effs/`. The verdict is still right. The cited evidence, though, only supports R2's one-shot variant, so a reader cannot tell whether the specified server transport was rechecked.

   **Fix:** Add "TCP listen/accept effects ([Base][7])" to the row's evidence. Optionally, F3 line 108-109 could say that the loopback-server transport remains available upstream.

3. **The stdin inference could cite the evidence that most supports it.** `research/31-bend-update.md:101-107`. Nit.

   **Evidence:** `file_read.c:4-8` states that `File.read` handles "a pipe, a terminal and a short read". It also says `File.read_line` reads one byte at a time "so nothing past it leaves the file". That matters for a line protocol over an inherited pipe. On the other side, Linux refuses to open `/dev/stdin` when fd 0 is a socket (ENXIO). This fits the report's "depends on the host" caveat.

   **Fix:** Optionally cite `file_read.c`'s comment as support and name the socket case in the caveat. The inference label and the "do not change the transport" recommendation are already right.

Nothing here calls for a comment on fx-djp. Companion §10's "since Bend has no stdin" is still accurate in the sense the report keeps, since there is no documented stdin effect. R1 explicitly recommends keeping the transport.

## Reasoning

**Bead questions.** All five are answered. Q1 is covered by F1 (five post-correction commits, no release after 2.0.40, #813 and WONTFIX unchanged) and F2 (Bend 3 and the AI-written compiler claim, the round, the merger, HVM5 private). Q2 is the 15-row D16 table, with one verdict per claim and a dated source for each. Q3 is F4, which separates Bend's self-reported M4 pin from third-party measured CPU/OpenMP, GPU/CUDA and Koru figures. It also flags the version gap (2.0.3 or 2.0.5 against 2.0.40), the fitted-not-timed GPU ratios, the excluded raw samples and the sequential Rust baseline. Q4 is F5's two adopt and two reject items. Q5 is R2: it gives the contract, a five-step check, a cost of half a day plus a day once a process interface exists, and what success would and would not prove. It concludes the experiment is not an fx-rpsb example. The acceptance condition that every changed D16 claim be listed with its evidence is met by lines 264-268 and F3/F4/F2. No D16 claim changes verdict, and the report says why.

**Conclusions are supported.** The verdicts follow from the explicit definitions of "Holds" and "Unknown" at lines 238-240. They are consistent: rows that depend only on a Tide-side mechanism nobody has demonstrated (pin and cache, crash to `Err`, the law translator) are marked Unknown rather than Holds. The report 11 corrections hold up against the sources:
- The stdin correction is properly scoped. There is no stdin effect, and `Process.run` feeds a child's stdin. `/dev/stdin` via `File.open` is marked untested inference and comes with a test list.
- The runtime-flag correction matches the 2.0.38 notes and the parser.
- The U32 correction is accurate but could say more (comment 1).

The Stage 0 inference is careful. Its one-shot design, with depth as an argument and one versioned line on stdout, avoids the stdin question entirely, and it says it is not the companion's persistent server. Its prerequisite (`Proc` has no operations) is verified. It also refuses to let an unsafe Rust adapter stand in for Tide integration, which is the right call.

**Inference and evidence are kept apart.** "Observed" and "Inference" labels are used throughout. Sources distinguish measurement dates from access dates. Author posts are flagged as claims, and the mirrors as supplying text only.

**The recommendations stay recommendations.** No design file changed. R1 keeps D16 in Later. It proposes qualifying the Stage 0 sentence "at the next authorized design edit" and rejects promotion into the alpha. It keeps the specified transport and the accepted proof fragment unchanged. This matches the bead's constraint and AGENTS.md's role rule that research records options plus a recommendation.

**Clarity.** The report is dense, but not padded. The repeated "Inference, 2026-11-14" tags cost a little flow but serve the bead's marking constraint. F2 (funding and people) is the most tangential section, but Q1 asked for it.

**Tier rule.** The change edits only research reports and touches no design tier. None of the comments would change a tier, so none blocks.

DONE
