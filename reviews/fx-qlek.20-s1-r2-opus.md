Model: Claude Opus 5.5 · Date: 2026-11-19 · Candidate 0683045 (design, stage 1)

## Verdict

**Changes required.** One blocking finding, and it needs only a text fix: the trust-diff reading of a `rand` line calls simulated draws "unpredictable", which contradicts 03 §9.3 and R1. Everything else holds:

- The admission of the five draws and the deferral of `rng() -> Pcg64` are right.
- The range, bool and shuffle arithmetic is exact.
- R1–R4 are present and accurate.
- The budget holds: 24,960 of 25,000.

AP-PROOF-001 says text-only fixes need no re-check, so this one does not need another round.

## Scope

I read:
- AGENTS.md;
- GOVERNANCE.md's Design Admission Rule;
- `br --no-db show fx-qlek.20` and its comments, including the relayed owner decision;
- the whole diff `4daf83f..0683045`, which covers research/53 in full, 01 D7, 03 (§1.1 table, RAND-DRAW, the habit row, §4.3, §9.3 and the §9 example), reference §6.1, companion 14.2 and 22, and the RAND-1..4 rows in rules.md;
- target/work/report.md, s1-r1-codex.md (as summarized in report.md and the bead), and redteam-codex.md.

I checked these against the landed code:
- `crates/tide-std/src/collections.rs`, for `from_rand` and the redacted `Debug`;
- `task/scheduler.rs`, for `pick`, `index` and the stream tags;
- 03 §9.3, which says every capability operation is a scheduling point;
- 03's §1.1 table convention, where unbuilt v0 operations such as `bind_udp` and `fault_point` carry no label.

Checks I ran:
- `scripts/count-tokens.sh design/reference.md` gives **24,960**, which matches companion line 17. This script calls `claude -p`; it was the only network use, and the brief requires the check.
- `git diff --check 4daf83f..HEAD` is clean.

I made no edits other than this file, built nothing, wrote nothing to `.beads/`, and started no subagents.

## Blocking findings

### B1. The `rand` trust-diff line calls simulated draws "unpredictable"

**Where:**
- `design/03-authority-concurrency.md:1394–1395`: "A `rand` line means the item can draw unpredictable values (from `getrandom`, or the seed's under simulation)".
- The same sentence is at `research/53-rand-v0.md:372–373`.

**Evidence:** The same chapter's §9.3 bullet (`03:2282–2286`, R1's text) says a simulated word "can reveal the seed and permit prediction of other simulator streams". Companion 22 says simulated values are "predictable from the seed, so never secret".

The trust diff is the security-facing reading of authority. As written, its parenthetical extends "unpredictable" to the seed's stream. This is the claim R1 was added to rule out, and the two passages in 03 now contradict each other.

**Minimum fix (text only):** scope the adjective to the real source in both places. For example: "can draw values (unpredictable ones from `getrandom`; under simulation, the seed's, which are predictable)". Equivalently, drop "unpredictable" and keep the parenthetical. 03 is not the core, so the reference count is unaffected.

## Non-blocking findings

**N1. Companion 22's Hashing bullet does not carry R2 where agents meet `from_rand`** (`design/reference-companion.md:834`).
- The bullet says `from_rand` "keys a map from the OS". Two bullets above, the companion says "use `fill_bytes` for keys and nonces".
- An agent loading only the companion can take real `from_rand` keys for independent OS draws, private to that map.
- R2 (correlated per-thread keys, which `Debug` on the built hasher exposes, for map hashing only) is stated only in research/53 §7. That is where the red team asked for it, so this does not block.
- Suggested addition, about 15 words: "(real runs: Rust's per-thread keys, correlated and visible through the built hasher; for hash-flooding resistance, not secrets)".

**N2. research/53 §4 says draws "never touch the scheduler" stream** (`research/53-rand-v0.md:192`).
- Each draw is a scheduling point (line 202). `Scheduler::pick` consumes the scheduler stream whenever two or more tasks are runnable (`scheduler.rs:654–678`).
- What is true: draw words never come from the scheduler, file, net or process streams, and a run that never draws is unchanged. 03 §9.3 and RAND-1 say exactly that.
- Suggested wording: "Draw words never come from …; the call's scheduling point uses the scheduler stream like any capability operation."
- Related: §4's rejection of drawing from the scheduler's stream says a drawing run "would change its schedule as well as its values". The chosen design also changes the schedule of a run that draws, through its new points. The real difference is that word consumption (rejections, shuffle length, fill length) does not shift later scheduling choices. Saying so makes the rejection exact.

**N3. research/53 §7 says the five draws give "bytes independent of everything else the program or any other process can see"** (`research/53-rand-v0.md:313–314`).
- The kernel generator gives computational unpredictability, not statistical independence: every process's output comes from one CSPRNG state.
- Because the owner decision rules out any claim beyond the OS source, say "unpredictable from anything else … as far as the kernel's generator is", or "computationally independent".

**N4. RAND-2's word-count clauses cannot be observed on the real backend** (`conformance/rules.md:108`; `research/53:475–481`).
- The row asserts "the same assertions on the real and simulated `Rand`", including "takes no word" and "panics before drawing".
- A conformance program on the real backend has no word counter. The RAND-4 seam is `#[cfg(test)]` inside tide-std, which a conformance case cannot reach.
- Fix: state that the word-count and before-drawing clauses are checked on the simulated backend, through the `rand` event's word count, and on the real backend through RAND-4's seam. Keep only the value and panic clauses shared.

**N5. RAND-3's rejection witness needs a named range** (`conformance/rules.md:109`; `research/53:484–485`).
- The rows say "a seed that hits one" but give no range.
- For a small `s`, the threshold `2^64 mod s` is below `s`, so no feasible seed search finds a rejection.
- Name a range with `s > 2^63`, for example `0..=1u64 << 63`: there `s = 2^63+1` and `t = 2^63−1`, so about half of first words are rejected.

**N6. RAND-4's "the backend reaches only `rustix::rand::getrandom` (the guard test and a source check)" is weak for the bead's Acceptance, "Real draws are proven to come from getrandom"** (`research/53:490–492`).
- The public-items guard does not show which syscall the backend makes, and "a source check" names no observation.
- A runnable falsifier exists in the repo: run a real draw in a child under a seccomp filter that returns `EPERM` for `getrandom`, using RUN-3's sandbox machinery or a test-only filter. Expect a panic that names `EPERM`, and expect no `/dev/urandom` open.
- That proves both the source and the no-fallback rule. If it is not used, name the source check concretely: the only call site and the seam's position.

**N7. research/53 §6 quotes "Authority is spent at seeding" as "(03 §1.1)"** (`research/53-rand-v0.md:282–283`).
- This diff deletes that sentence from 03, so the citation dangles.
- One planned rule also now lives nowhere: `Pcg64::seed_from_u64(n)` needs no capability.
- This is acceptable, because §6 reopens the generator's admission for fx-9bzp. Cite it as "the former 03 §1.1 note" and keep the capability-free seeding point in §6, so fx-9bzp inherits it.

**N8. The §10 admission leaves an overlap unstated (style).**
- `next_u64()` overlaps `random_range(0..=u64::MAX)` and an 8-byte `fill_bytes`. §10 names only the coin-flip overlap.
- `next_u64` is still worth admitting: it is the word primitive the other methods are defined over, it is the obvious spelling for a `u64` id, and it is unchanged from `rand_core`.
- One clause in §10 would make the admission complete.

## Reasoning

**Admission.** The stated needs are jitter, sampling, shuffles, ids and tokens, sim test data, and the fx-zu4f.14 key consumer. They require:
- a raw word or bytes (`next_u64`, `fill_bytes`);
- unbiased bounded integers, because `x % n` is the habitual bug;
- a probability draw that handles `p == 1.0` and NaN, which a hand-rolled `x < (p·2^64) as u64` gets wrong;
- a shuffle, where the classic hand-rolled version is biased.

Each operation removes a known wrong code shape rather than adding a second way to do something, which is the filter's test. Floats, 128-bit ranges, `choose` and `random_ratio` are rightly left out, and each has a one-line spelling.

Deferring `rng() -> Pcg64` is correct:
- it would be a second type with a second copy of every method;
- a seeded stream is a value-stability contract that needs its own algorithm admission;
- `Clone` repeats the stream;
- it is not a CSPRNG for the crypto consumer;
- the owner's decision is draws only.

Nothing admitted goes beyond the stated needs. The sealed `SampleUniform`/`SampleRange` traits are the minimum that lets `random_range` be generic, and they expose no callable method.

**Arithmetic.** I checked each method's arithmetic:
- *Range.* `t = s.wrapping_neg() % s = 2^64 mod s` accepts `2^64 − t` words, which is a multiple of `s`, so the result has no bias. The worst rejection rate, at `s = 2^63+1`, is below 1/2. `s = 0` means the full width and is special-cased, which matters because the scheduler's `index` would panic on a bound of 0. Stage 2 must not call it with a full range; RAND-2 tests the full range.
- *Bool.* `p·2^64` is exact for `p` in `[0, 1)`, and its largest value is `2^64 − 2^11`, so it fits in a `u64`. With the explicit branch for `p == 1.0`, the threshold never overflows. Drawing a word for every `p` keeps the stream position independent of `p`.
- *Shuffle.* Fisher-Yates from the end over `0..=i` is uniform when its words are uniform. R3 correctly limits what "uniform" means under finite seeds.

**Replay.** A draw being a scheduling point follows 03 §9.3's rule that every capability operation is one. Like `Env`, a draw has zero latency and no checkpoint, so cancellation cannot truncate a rejection loop or a shuffle.

The first-word claim is exact. The step adds a constant and SplitMix64's output mix is a bijection, so distinct seeds give distinct first words. R3 confines the no-collision claim to first raw words, and that limit appears in research §4 and in companion 22.

Simulated `from_rand` keeps the existing `RandomState(DefaultHasher)` shape: a zero-key SipHash with a two-word prefix, cloned for each hash. So no signature changes, and map order varies by seed and replays exactly. Research makes no PRF claim, and a PRF is not needed.

**Honesty, and where R1–R4 are placed.**
- R1 appears verbatim at research §4, research §7 and 03 §9.3. The §9.3 copy is where readers meet "clones share the stream".
- R2 appears verbatim at research §7, and §7's independence sentence is scoped to the five draws. Placement in the companion is N1.
- R3 appears in research §2.1, research §4 and companion 22, in the builder's own wording. The wording is accurate: distinct seeds give distinct first raw words only, and a simulated byte string carries at most 64 bits of variation.
- R4 appears in research §8 (structurally visible, else `opaque`, and opaque is not authority-free) and in research §7 (latency through a `Clock` or granted synchronization). Both match the red team's text.

Other honesty checks:
- The inherited real `from_rand` limit (cached thread-local keys, a possible `/dev/urandom` fallback) is stated in research §3 and §5, and in the 03 §4.3 line.
- The getrandom-only, no-fallback contract is scoped to the five draws, at research §3:155–157.
- No sentence claims crypto strength beyond the OS source. Companion 22 says "as unpredictable as the kernel's generator and no more".
- The remaining honesty problems are B1, which calls simulated values unpredictable, and N3, a wording point.

**Stage 2 readiness.** RAND-1..4 and research §12 map to the bead's Acceptance:
- same seed, same draws, and the manifest replays byte-identical: RAND-1;
- a different seed gives a different first word: RAND-1;
- a run that never draws is unchanged: RAND-1;
- identity panics: RAND-1;
- edge cases on both backends: RAND-2;
- the exact simulated method: RAND-3;
- the real source through the rustix seam, `EINTR` and short reads, error panics, and a guard on public items: RAND-4.

Research §12 also lists the leftovers stage 2 must clear: the `lib.rs` docs, `collections.rs`'s comment, the panic assertion in `sim.rs`, the 03 §9.2 modeled list, the labels, and the reference's partial notes. N4–N6 make three rows falsifiable as written. None of them changes the design.

**Consistency and budget.** 01 D7 owns the decision. 03, companion 22 and 14.2, the reference's §6.1 row and bullet, rules.md and research/53 agree on:
- the names;
- the panic rules;
- the sources;
- the stream tag;
- the `from_rand` decision;
- the deferral of `rng()`.

The only disagreement is B1.

Planned labels are in place in 01 ("planned until its stage 2"), 03 RAND-DRAW and §9.3, reference §6.1, companion 22 and 14.2, and the rules rows. The §1.1 table leaves the five unlabelled, which follows that table's convention: `bind_udp` and `fault_point` are unlabelled too, and RAND-DRAW carries the label. The reference's existing partial notes, at §6.4 line 443 and §6.8 line 515, still state today's behavior correctly.

The reference's old `Pcg64` bullet is gone. Its draw rules moved to 03 and companion 22, and `Pcg64 (planned)` stays in the 12.3 table. The one planned rule not carried over is N7, and it belongs to a surface whose admission this bead deliberately reopens. The core is 24,960 tokens, recounted on this candidate, within D14's 25,000, and the companion records that count.

DONE
