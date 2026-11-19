Model: gpt-6.1-sol (high) · Date: 2026-11-19 · Candidate 08124e0 (design, stage 1)

## Verdict

**changes required** — high confidence. The five-operation admission and sampling mechanisms are sound. Two documentation claims need correction before this candidate is clean. Neither finding requires implementing stage 2 in this review.

## Blocking findings

### B1. The grant-wide getrandom-only claim excludes an existing, explicitly retained backend path

**Files:** `research/53-rand-v0.md:267`–269, with the unchanged-real-backend decision at `research/53-rand-v0.md:231`–232 and the grant-wide reading at `design/03-authority-concurrency.md:1394`–1395.

**Evidence:** The note says a `Rand` grant consumes `getrandom(2)` output and reaches no other resource. However, `crates/tide-std/src/collections.rs:35`–40 implements `RandomState::from_rand` using Rust's `RandomState::new()`, and §5 expressly keeps that real backend unchanged. On the pinned nightly, `library/std/src/hash/random.rs:67`–74 initializes its thread-local keys through `hashmap_random_keys()`. `library/std/src/sys/random/linux.rs:120`–124 falls back on `ENOSYS` or `EPERM`, and lines 156–159 open and read `/dev/urandom`. Thus, on a supported kernel with the foreign seccomp denial the note itself considers at lines 146–148, a fresh hasher-key initialization can read that file instead of producing the new draws' getrandom error panic. It can also reuse cached keys rather than make a syscall per call.

This is an inherited path, not a newly discovered ambient opening introduced by these five methods. The problem is the new absolute description of the whole grant, alongside the decision to retain that path.

**Minimum fix:** Scope the strict getrandom-only/no-file-fallback contract to the five new draw methods. Explicitly document the inherited real `from_rand` behavior, including its cached OS keys and possible file fallback, in §5/§7 and make the trust-diff explanation consistent. If the intended contract is instead getrandom-only for the entire grant, the stage-2 design must also replace real `from_rand` seeding; it cannot promise both that contract and the unchanged backend.

### B2. The companion's mandatory core-count record was not updated

**File:** `design/reference-companion.md:17`–19.

**Evidence:** This candidate records **24,999** tokens, and immediately requires updating that line after every core change. Running `scripts/count-tokens.sh design/reference.md` on 08124e0 returns **24,960**, matching research/53 §11 and the builder's report. The budget passes, but the companion contradicts the current reference and its own recording rule.

**Minimum fix:** Change the companion's recorded count to 24,960, preserving the actual recount date. Recount if the reference changes during fixes.

## Non-blocking findings

### N1. The getrandom alternative can disable the rejected fallback

**File:** `research/53-rand-v0.md:158`.

The default `getrandom` 0.2.17 Linux backend really does fall back on `ENOSYS`/`EPERM` on the relevant x86_64 target. However, that version also provides `linux_disable_fallback` (`Cargo.toml:60`; `src/lib.rs:48`–60 and 261–307). The comparison should acknowledge this syscall-only configuration and reject it on the sufficient remaining ground: adding another direct wrapper when pinned rustix already owns the required behavior. The rustix choice remains sound; fallback is not an unavoidable property of the alternative.

## Reasoning

- **Admission and scope:** Each method has a distinct justification: `next_u64` supplies a word; `fill_bytes` supplies variable-length OS material for the named crypto consumer; `random_range` owns unbiased reduction; `random_bool` owns probability validation and endpoint handling; `shuffle` owns the error-prone Fisher-Yates bounds. The latter three prevent common handwritten mistakes rather than adding general distribution machinery. Deferring `Pcg64` avoids a second generator type, repeated methods, cloning traps and an algorithm/seed-expansion stability commitment. No float/128-bit distributions or broader std work are admitted. This is a defensible minimum useful surface under the relayed owner decision.

- **Range correctness:** Research §2.1 computes cardinality in u64 after an order-preserving signed offset. Empty ranges are rejected first, so wrapped cardinality zero denotes only a full 64-bit inclusive domain. That case draws a word directly. Otherwise `t = 2^64 mod s = s.wrapping_neg() % s`; `[t, 2^64)` has a multiple of `s` elements, so reduction is uniform. Narrow signed domains and ranges crossing zero are covered by the offset, not signed overflow. Stage 2 must preserve that arithmetic and invert the offset correctly. I found no mathematical defect in the admitted type/width coverage.

- **Bool, shuffle and failures:** Scaling a finite f64 in [0,1] by 2^64 is exact; flooring defines the stated discrete probability. Validation must reject NaN through an ordered membership check, not only `p < 0 || p > 1`. The explicit p==1 branch avoids a saturating-u64 threshold error; both endpoints still consume one word. Descending Fisher-Yates with an inclusive 0..=i draw is uniform. Empty/singleton shuffles and empty fills consume no words. Invalid arguments panic before drawing; the proposed rustix loop retries short reads/EINTR and panics on other errors. B1 distinguishes the inherited hasher path from this new draw contract.

- **Dependency and real source:** `crates/tide-std/Cargo.toml:13` already pins rustix =1.1.6; `Cargo.lock:149`–159 locks it. Cached rustix source has `rand = []`, gates its rand module on that feature, and wraps one raw getrandom syscall with no vDSO or file fallback. Enabling it changes compiled feature/artifact inputs, but introduces no dependency edge and does not require a lockfile change. The license matches the research table. The raw libc syscall alternative would require unsafe code and a direct libc dependency; rustix supplies the existing safe owner. RUN-3 really allows `SYS_getrandom` (`sandbox.rs:576`). These conclusions are source inspection, not a feature-enabled build or lockfile regeneration.

- **Simulation, hashing and replay:** A separate tagged SplitMix64 state fits the existing scheduler/fs/net/proc pattern. Its first output is a bijection of the seed, hence distinct seeds have distinct first raw words. Shared stream consumption under the baton scheduler gives deterministic task assignment for the same seed and execution. Per-task streams are unnecessary for the chosen replay contract. The word source remains separate from scheduler randomness, although adding scheduling points can of course consume scheduler draws and change interleavings in runs that draw. No-draw runs retain the existing behavior. Existing manifests already carry root seeds, build/dependency identity and an exact trace (`replay.rs:39`–72; `sim/manifest.rs:205`–230). New deterministic events need no injected-input tape or new manifest identity field; changed std artifacts already invalidate incompatible replay. A fixed-key DefaultHasher with two stream words absorbed as a prefix is deterministic and seed-varying without claiming to be a keyed PRF. This is sufficient for simulated map-order exploration; it does not promise different iteration order for every pair of seeds or every map.

- **Authority and consistency:** Receiver-based `Rand` accounting already exists in `pure.rs:75` and the synthesis mapping at `synthesis/contracts.rs:654`; sealed sampling traits add no user-defined entropy source. The proposed real/sim identity checks follow existing capability boundaries. No new ambient entropy route is specified. 01 owns the admission, 03 elaborates it, the companion explains it, and RAND-1..4 cover the bead's replay/OS-source outcomes plus useful boundaries. Planned labels and the retained partial notes distinguish stage 1 from landed behavior. The reference trims remove no implemented v0 rule; deferred Pcg64 details are shortened along with its explicit deferral. The companion count and grant-wide backend statement are the exceptions identified above.

**Proof scope:** Frozen HEAD verified as `08124e05988cab70fa84cec786b5d0c5f7ce5808`, base `4daf83f`, clean tracked working tree, Linux workspace at `/mnt/build/eon-agents/worktrees/tide-fx-qlek.20`, 2026-11-19. Read the six-file diff, governance, bead/comments, builder report, relevant landed owners and locally cached pinned dependency/toolchain sources. `git diff --check 4daf83f..HEAD` passed. Token count: 24,960. A plain `python3 -I` check passed exhaustive reduced-word-space rejection counts, signed cardinality endpoint checks at 8/16/32/64 bits, representative exact f64 threshold boundaries and rustix's empty rand-feature dependency list. These checks support the design mathematics and source claims; they do not prove the future Rust implementation.

No builds, compiled probes, runtime replay, implementation edits, commits or bead writes were performed. Only this report was written; stage-2 behavior remains unimplemented and unproven.

DONE
