Model: gpt-6.1-sol (high) · Date: 2026-11-19 · Candidate 97006fd (code, stage 2)

## Verdict

**changes required** — high confidence. One range-validation defect remains. The source split, sampling arithmetic for non-exhausted ranges, scheduler integration, simulated hashing, and authority accounting otherwise match the landed contract. No alternative architecture is needed.

## Blocking findings

### B1. Exhausted inclusive ranges can draw instead of panicking

**File:** `crates/tide-std/src/rand.rs:70–72`.

`RangeInclusive::span` checks only `start <= end`, ignoring the range's exhausted state. That is insufficient for the supported Rust `RangeInclusive<T>` input. A concrete caller on the pinned nightly is:

```rust
let mut range = u8::MAX..=u8::MAX;
assert_eq!(range.next(), Some(u8::MAX));
assert!(range.is_empty());
rand.random_range(range); // must panic before drawing
```

The pinned core's `Iterator::next` (`library/core/src/iter/range.rs:1530–1538`, integer step at `:348–352`) wraps `start` to 0 and sets `exhausted = true`. `RangeInclusive::is_empty` explicitly includes that flag (`library/core/src/ops/range.rs:571–575`); its endpoint documentation warns against determining emptiness from endpoints alone (`:399–404`). The candidate instead accepts `0 <= 255`, computes cardinality 256, consumes a word, and returns an arbitrary u8. Both backends take this path; simulation additionally adds a scheduling point and a one-word event for a call that should reject before drawing.

This violates RAND-2 and the empty-range panic contract in research/53 §2.1 and 03 RAND-DRAW. The surface accepts `RangeInclusive<T>` without restricting callers to fresh range literals. Existing empty-range tests construct fresh reversed ranges, so they miss this state.

**Minimum fix:** validate `!self.is_empty()` in the inclusive implementation before calculating the span. Keep the method/argument panic message. Add an exhausted-at-MAX regression to the real/simulated panic assertions and verify zero real source requests and no simulated trace change, using the existing seam and trace helpers. This is a source-proven finding; no Rust probe was run during review.

## Non-blocking findings

### N1. Public status documentation still says the delivered feature is absent

`README.md:372–374` calls Rand identity-only, `README.md:391` lists randomness as unmodeled, and `README.md:578` lists draws as unbuilt. The same stale availability claims remain at `docs/owner-guide/01-capabilities.md:23`, `docs/owner-guide/05-seeds-and-simulator.md:145–150`, `docs/owner-guide/10-v0-and-planned.md:23–27,44–46`, and `docs/owner-guide/answers.md:113–118`.

Update those current-status passages to include the five draws and seeded simulated randomness, keeping `rng() -> Pcg64` planned. This is a documentation leftover from implementing the feature; it does not change runtime behavior. The numbered design owners and RAND rows have had their planned labels updated consistently. Historical records and the owner guide's quoted deferred-bead listing need not be rewritten as current descriptions.

## Reasoning

**Contract and arithmetic.** For fresh/non-exhausted ranges, the integer macro maps all ten supported types to ordered u64 offsets via i128, avoiding signed overflow. Half-open cardinality is an ordinary positive subtraction; inclusive cardinality wraps to zero only for the full 64-bit domain. `below(0)` consumes one raw word. Otherwise `s.wrapping_neg() % s` rejects exactly the excess prefix, and accepted modulo residues have equal multiplicities. Singletons still draw one word. The inverse signed offset stays within the selected domain. B1 is the omitted Rust range-state validation, not a rejection-sampling defect.

`random_bool` uses ordered membership to reject NaN, infinities and out-of-range probabilities before dispatch. It always evaluates `next()` for valid p, including 0, negative zero and 1; the explicit p==1 branch handles the unrepresentable 2^64 threshold. The conversion truncates nonnegative values as specified. Descending Fisher-Yates uses `below(i + 1)` without introducing extra scheduling points. Empty/singleton slices consume no word; iterating `1..len` avoids an underflowing `len - 1`. For Linux's usize widths, the maximum loop index plus one fits u64, including huge zero-sized-element slices. Simulated fills iterate chunks of eight, so ceil(len/8) needs no overflowing len+7 calculation. Empty fills skip source reads. Huge real fills can complete through repeated short reads rather than assuming one syscall fills the destination. Huge inputs remain unbounded computation under the existing resource limits; I did not allocate them.

**Real backend and dependency.** There is one production source call, `rustix::rand::getrandom(dest, GetRandomFlags::empty())` at `rand.rs:226`. Every word uses eight such source bytes little-endian; real fills request the remaining destination. The loop preserves progress on short reads, retries EINTR, and panics on other errors with the stated `Rand: getrandom(2) failed with ...` message. The test seam is cfg(test). No file fallback or new production unsafe appears. Both std manifests retain the exact rustix =1.1.6 pin and add only its rand feature; neither lockfile changes. Cached pinned rustix source confirms `rand = []`, the accepted license, and a raw getrandom syscall without a vDSO/file fallback. Research/53 §3.1 records capability, alternatives, costs and replacement scope, satisfying AP-DEPENDENCY-001.

RAND-4 uses the feasible seccomp-EPERM child, not the conditional source-only fallback. Its BPF program denies getrandom and allows every other syscall. The unfiltered program draws; the filtered program must exit 101, name EPERM and emit no successful draw output. `/dev/urandom` is openable in the test environment and the filter leaves file reads allowed, so an answering file fallback would fail the assertion. This is a behavioral no-fallback witness supplemented by source inspection; it is not an openat syscall trace proving that no file was ever opened. The distinction is reflected in the updated row. The seam independently checks exact bytes, short reads/EINTR, error handling and zero-read cases.

**Simulation, hashing and replay.** The single scheduler-owned stream starts at seed XOR `0x7469_6465_7261_6e64`; the extracted SplitMix64 preserves the old streams' exact arithmetic. `Tasks::rand_op` takes one zero-latency, non-checkpoint scheduling point, then consumes all words and records one rand event while holding the runtime lock/baton. The event carries task, operation, span, actual word count, and the specified value/length. Clones share stream position. Invalid argument checks precede dispatch. Real handles invoke `outside_simulation`; simulated handles resolve through `Context::simulated` and its runtime-identity check before source use.

Simulated `RandomState::from_rand` takes two words through that same mechanism and prefixes `DefaultHasher::new()` with them. The pinned hasher constructor uses fixed zero keys, so this path does not call real RandomState or reach OS entropy. The real branch still calls Rust's inherited RandomState exactly as §5 requires. No manifest/input format change is needed: the existing replay identity includes seed and trusted artifacts, and rand events enter the exact trace comparison. The independent word oracle, rejection witness, two-task repeatability/map-order checks, stream-consumption isolation check and admitted manifest replay are meaningful falsifiers. R1–R4 remain stated limits: predictable/shared simulated words, inherited correlated real hasher keys, finite-seed coverage, and opaque authority/timing observations.

**Authority and surface.** The five public methods require `&Rand`; its private capability field supplies no public constructor/default. The new sealed traits are the specified range surface and cannot supply entropy. Their supertrait methods require a private-module token that safe downstream code cannot construct. Existing `pure.rs:75` and `synthesis/contracts.rs:654` own the rand classification. The new admitted draw test checks the exact trust diff, with rand and no extra kind; admission source still denies ambient RandomState construction and direct unrelated-crate routes. Wider Fs/Net/Proc/unsafe grants retain their already documented meanings. I found no new route to a draw or OS entropy without the required capability or declared wider authority/hole.

**Proof and hygiene.** Reviewed frozen HEAD `97006fd5b313b0ae2a99a701032cd7f551488312`, base `e2f2d26`, clean tracked tree, and the full 19-file diff. Diff SHA-256: `b32c72280f5d052c4bd3e9a7af78e459ac4b46fb52bbd33e2d7ed7e089f8fd79`. Stage-1 review files are absent from this candidate checkout but were read from review commit `2e6f95a` using git show, including R1–R4. No unexplained production API, dependency, dead helper or unsafe addition was found; the removed `Tasks::unmodeled` had lost its only caller. The one-line TD0516 correction is a justified suite repair.

The suite identity file records `86131daaa1d4d724203987519ffe92eee222f498`. `conformance.exit` is 0; its log ends with 393 cases, 390 PASS, 3 AWAITS CHECKER, 0 FAIL, including authority/rand. `test.exit` is 101: the log records all 122 build tests passing (all four new RAND tests included), then the sole observed failure at code_prefix for the owner-guide spelling. Cargo stopped there, so that log is not evidence that later workspace suites ran. The Suite fix section records the focused code_prefix rerun on 97006fd with two passes; the candidate diff makes exactly that correction. Per the review brief, the pending full-suite rerun on the final candidate is not itself a blocker here.

`git diff --check e2f2d26 97006fd` passes. A read-only `python3 -I` check verified equal rejection-residue multiplicities exhaustively for reduced 2–10-bit word spaces and signed cardinality endpoints at 8/16/32/64 bits; a source-derived state calculation confirms B1. These mathematical/source checks are not Rust runtime proof. The builder records a reference recount of 24,932 tokens and the companion matches it, below 25,000. I did not independently rerun count-tokens.sh: it invokes an external Claude session, outside this review's permitted commands/no-contact scope.

Only this report was written. No Rust builds/runs, commits, bead writes, other contacts or implementation edits were performed. Final-candidate full suites and an independent token recount were not performed in this pass.

DONE
