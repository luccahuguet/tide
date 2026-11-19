Model: Claude Opus 5.5 · Date: 2026-11-19 · Candidate 5246ee1 (docs, S1b)

## Verdict

Changes required. There are two blocking findings, and each needs one clause to fix. Otherwise the page is accurate and answers all three of Lucca's questions directly. Its "use Rust instead" list is concrete enough to act on, and its tone stays complementary to Rust.

## Scope

- **Files:** `git diff c82030d..5246ee1`, which covers `docs/tide-or-rust.md` (new, 168 lines, about 1,500 words) and the README Reading Guide line.
- **Rules and records read:** AGENTS.md, GOVERNANCE.md's Merge Rule (through AGENTS.md), and the bead with its comments: the A1 framing, the "never unforgeable" relays, research/49's wording rule (05:14Z and 07:22Z), the terms and tone rules (2026-11-18 19:51 and 20:02), and the S1b request (07:57Z). Both Codex reports (`s1b-codex.md` and `s1b-recheck-codex.md`) are not repeated here.
- **Sources checked against the page:**
  - README: at a glance, Evidence, Benchmarks, Case studies, Try it, Status, Not in this alpha, Simulator fidelity, Sandbox and Known holes
  - `docs/owner-guide/answers.md` item 2 and `docs/owner-guide/09-experiments.md`
  - `docs/docker.md`
  - `research/36` §1.1, `research/44`'s head, and `research/49` §1, §2, §9 and §10 (the protocol's §1)
  - `design/reference.md` §2.3 and §12.1
- **Mechanical checks:** every relative link target exists. No "unforgeable" appears. The only "catch" is the malware re-test's "not a first-time catch".
- **Not done:** no builds and no network use.

## Blocking findings

### B1: the catch account leaves out that the re-test's pool was chosen after the screen

- **Where:** `docs/tide-or-rust.md:64-68` ("…re-tested under a new registration, a method recorded in advance… On that separate re-test, our port…").
- **Evidence:**
  - research/49 §10 (the registered protocol, §1) says: "This text, or text with the same meaning, opens the record and any write-up". Its first disclosure is "**The pool was chosen after fx-zu4f.8 saw the rows.**" Its third adds that "the outcome … is close to predictable".
  - Every reviewed short form keeps the selection clause:
    - at a glance item 9 (README.md:71-72): "That separate re-test, chosen after the screen had seen its one row, found a catch"
    - the README Evidence section (README.md:182-183): "The one-row pool was chosen after the screen had seen its rows, the outcome was close to predictable"
  - This page keeps the gap, the fix, the new registration, the order qualifier, "admission and type-checking only" and "not a first-time catch or a rate". It drops the post-hoc selection.
  - A skeptical reader is left thinking a method recorded in advance picked the row. It did not. The registration came after the row had been seen.
- **Why it blocks:** it breaks a binding disclosure rule and drops a material limit of the one admission result the page lists as a benefit.
- **Minimum fix:** at line 67, write "On that separate re-test, chosen after the screen had seen its one row, our port…". This matches at a glance. Adding "and its outcome was close to predictable" is recommended but optional.

### B2: macOS and Windows support is stated as working, but no run there has been observed

- **Where:** `docs/tide-or-rust.md:95-96` ("On macOS or Windows, Tide runs in a Linux container").
- **Evidence:**
  - `docs/docker.md:58-64` says: "The image was checked on Linux only … No macOS or Windows run has been observed."
  - README.md:406-410 (Sandbox) says the same, and adds that the sandbox level there "depends on the kernel of Docker's Linux VM".
  - `docs/docker.md:41-42` says Apple-silicon Macs run the image under emulation, more slowly.
- **Why it blocks:** the page states as fact a result nobody has observed. It is also the one platform line that a macOS or Windows reader uses to decide whether Tide fits them.
- **Minimum fix:** "On macOS or Windows, [docs/docker.md](docker.md) runs it in a Linux container, a route checked only on a Linux host so far." Adding "(under emulation on Apple silicon)" is optional.

## Non-blocking findings

1. **The re-test bullet doesn't say what the reader gains** (lines 61-76). "A version handed the authority it uses is now admitted" invites the question "so malicious code gets admitted?" The README's own limit (README.md:186-189, "No judgment of intent … what changes is that its authority is declared") answers it. One clause would fix it: "admitted, with its process and clock authority now declared in its signature". "Tide makes no malware-proofing claim" (line 142) sits in a different section.
2. **The tracker ID is used cold** (line 65). "The fx-zu4f.8 screen" is never defined on this page. The bead's wording allows the ID, but at a glance uses "the screen exposed it", which suits a cold reader better.
3. **The benefit bullet opens with a negative** (lines 61-64): "…caught none of them". The tone rule formally covers only the paper, at a glance and the status page. Still, a list of benefits reads better when it leads with what the re-test shows and states the 0-of-87 screen as its scope.
4. **E1's iteration ratio drops its interval** (lines 50-52). "1.46 … against a 1.5 limit" reads as comfortably inside the limit. Chapter 9 (lines ~33-35) adds that the interval, 1.29-1.64, crosses 1.5. Add it in parentheses.
5. **E3 drops the ceiling** (lines 80-83). "Gave no evidence that it helps reviewers" can be read as evidence that it doesn't help. Chapter 9 (lines 306-309) says the pilot could not have shown a benefit, because plain review already caught almost every plant. Half a sentence would carry that.
6. **The replay bullet cites only the in-house demo** (lines 34-42).
   - The eon-discovery-race case study is stronger: a real Eon bug, failing on 13 of 100 seeds and replayed (README.md:276-283).
   - A skeptic's next question is whether replay has found an external bug. The answer is no external catch yet: the README's Evidence section says the other rounds caught nothing, and research/44 has one unresolved row.
   - One clause on each would make this bullet complete for that reader.
7. **Terms with no definition or with ambiguous wording:**
   - "contracts" (line 108): Tide's executable contracts and the `contracts` benchmark program are both unintroduced.
   - "Tide's profile flags", "paired" (line 109) and "the driver" (line 38, first used before it is tied to `tide-admit`).
   - "1.74x of it" (line 110) reads as a share of the 2.29x. research/36 gives a product: 1.74x from the flags, then 1.31x between Tide and its twin built with the same flags.
8. **"Leaving" undercounts the work** (lines 57-60). Tide's attributes would also need replacing: `#[sim_test]`, `#[requires]`/`#[ensures]`, `#[tide::main]` and `#[error]`, which are toolchain proc macros (reference §2.3). "Calls into Tide's std" doesn't cover them. The bullet is labelled as a reading of the design, which helps.
9. **The "use Rust" list could be sharper** (lines 135-144).
   - The limit is any second crate, including the reader's own multi-crate workspace, not only crates.io (README.md:365-367).
   - Readers who need UDP, randomness, signals or process exit should also be pointed to Rust. Saying "…or anything in Missing pieces above" would cover them.
10. **Facts that may change before the RC:**
    - "no randomness draws" (line 113) goes stale if fx-qlek.20 lands before the RC. S6's claims audit should recheck it.
    - Out of scope, for S5/S6: README.md:330 ("docs/docker.md runs the same route in a Linux container") has a milder form of B2. It sits next to the Sandbox section's caveat, so it isn't a blocker there.

## Reasoning

**Accuracy.** Every number on the page matches its source:

- **E1:** 58/60 and 58/60 final passes; 56 against 59 on first check; 1.457627 ≈ 1.46 against 1.5; about 9 times the cached input (gate report, chapter 9).
- **research/36 §1.1:** 3.8-4.6x, 5.2-6.3x, +17% to +39%, 1.4-2.0x loaded bytes, the erased-contracts tie, 1.07x paired, 1.18x, 2.29x, 1.74x, and 12.8x for one seed of `tasks` in release.
- **Demo:** 8 of 16 seeds.
- **Reference budget:** 25,000 tokens.
- **Simulator surfaces:** as listed in README.md:93-100.

The limits also match:

- one crate with `extern crate` rejected; no async; user macros, build scripts and file-loading attributes rejected (README.md:345, reference §2.3)
- the "Missing pieces" list (README.md:363-376)
- three forge routes closed and the others open, and the `unsafe` grant forging capabilities (Known holes)
- the kernel-dependent sandbox and unsandboxed builds (README.md:398-414)
- `Net::only` limiting outbound connections only, with `no_bind()`
- allowed programs running with their own full authority
- signatures showing the kind of authority, not how far it was narrowed

Codex's B1-B5 fixes hold. Three things not covered there fall short of their sources: the selection disclosure (B1), the macOS/Windows observation status (B2), and the smaller omissions in N1, N4 and N5.

**Wording rules.**

- No "unforgeable". The precise claim is given as "no accidental ambient authority" (line 23), as relayed.
- The trust diff is presented only as a summary, with no "catch" and no claim that it helps.
- No rate. "Not a first-time catch" appears.
- The qualified order wording matches research/49 §3.
- The std-gap sentence is the bead's binding sentence, word for word.
- The one gap is the selection disclosure (B1).

**The three questions.**

- **Pros and cons:** two balanced sections, with the costs stated plainly. The page calls the dependency limit "the largest limit today", which is honest and useful.
- **Use Rust instead:** six specific, checkable conditions, including the right advice on running untrusted code (use a VM).
- **The ideal user:** agent-heavy developers of CLIs, automation or small services that touch files, the network or processes. They want authority they can read and races they can replay, can live inside one crate on x86_64 Linux, and know Rust. This fits what v0 actually ships.

All three are answered directly.

**Tone.** The opening says the subset complements Rust and does not replace or fork it. The Rust list respects Rust's ecosystem ("the Rust ecosystem matters more to you"), and nothing in the page reads as a rebuke.

**Cold reader.** Most terms are defined inline: crate, rustc, std, capability, task, seed, AI, tokens, port, RustSec, E3, the `unsafe` grant, Landlock, seccomp and FFI. N2 and N7 list the gaps. At about 1,500 words, the page takes 6-7 minutes to read, which fits "a few minutes" for this audience. The README Reading Guide line is placed well and describes the page accurately.

DONE
