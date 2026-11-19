Model: gpt-6.1-sol (high) · Date: 2026-11-19 · Candidate 666106f (re-check)

## B1 — Partially resolved; changes required

The requested scope correction is present. `research/53-rand-v0.md:148–150` limits the getrandom-only/no-file-fallback contract to the five draw methods. Its §5 (`:235–252`) explicitly retains the inherited real `from_rand` backend as a stated limit, describes cached thread-local keys and the possible `/dev/urandom` fallback, and records tide-lead's decision. §7 (`:287–295`), §8 (`:344–347`), and `design/03-authority-concurrency.md:1394–1397` now include that inherited source in the grant/trust-diff description. No backend replacement is required by this re-check.

Checked against the original cited sources: `crates/tide-std/src/collections.rs:35–40` still constructs the real hasher through Rust's `RandomState::new().build_hasher()`. The pinned `nightly-2026-08-09` source, `library/std/src/hash/random.rs:67–74`, caches the key pair per thread and increments `k0`; `library/std/src/sys/random/linux.rs:95–124,156–169` confirms the flags, fallback conditions, file read, and hash-key entry point. The added backend description matches those sources.

**Remaining problem introduced by this fix:** `research/53-rand-v0.md:298–299` adds “(for `from_rand`, only through map iteration order).” The unchanged public API does not impose that limit: `crates/tide-std/src/collections.rs:49–52` implements `BuildHasher` and returns a cloned `DefaultHasher`. A caller importing `std::hash::{BuildHasher, Hasher}` can evaluate `RandomState::from_rand(rand).build_hasher().finish()` and observe a key-dependent `u64` without constructing or iterating any map. The pinned `library/std/src/hash/random.rs:124–141` supplies `DefaultHasher`'s public `Hasher::finish`; `crates/tide-admit/src/policy.rs:454–456` allows that hasher. This is an inaccurate new restriction in the authority explanation, not a request to change the inherited backend.

**Minimum correction:** Remove the new parenthetical, or say that observations include hash outputs as well as map iteration order. Keep the five-draw contract and the stated inherited-backend limit.

## B2 — Resolved

Ran `scripts/count-tokens.sh design/reference.md </dev/null` on candidate `666106fcd465da55bb0f4e815138a02fb0561dd9`: exit 0, **24,960** tokens, within the **25,000** budget (40 remaining). `design/reference-companion.md:17–19` records exactly 24,960, the actual recount date 2026-11-19, and the counting command. `git diff --quiet 08124e0 666106f -- design/reference.md` exits 0, confirming the core is unchanged during fixes. The accompanying introduction correction to “17–22” agrees with the existing §22 heading.

## N1 — Resolved

`research/53-rand-v0.md:160` now acknowledges `getrandom` 0.2.17's `linux_disable_fallback` feature and rejects the additional direct wrapper because pinned rustix already owns the required syscall behavior. Checked against the same cached version cited originally: `getrandom-0.2.17/Cargo.toml:60` declares the feature; `src/lib.rs:48–61` documents it, and `:261–307` selects the syscall-only Linux implementation when enabled. `src/linux_android_with_fallback.rs:19–28` confirms the default Linux ENOSYS/EPERM fallback. The revised comparison is accurate.

## Verdict

**changes required** — high confidence. B2 and N1 are resolved. B1's requested backend/scope correction is implemented, but its new map-order-only assertion needs the small documentation correction above.

Proof scope: inspected only the three findings, their fixes, their cited sources, and the fix-introduced assertion. Frozen HEAD verified as `666106fcd465da55bb0f4e815138a02fb0561dd9`; tracked working tree clean before report creation. `git diff --check 08124e0 666106f` passes. Fix-diff SHA-256 (`git diff 08124e0 666106f | sha256sum`): `fa72891473ecadc3e43171989e22ce7174bd3838872e0281366ab25e02aa890f`. Environment: Linux workspace `/mnt/build/eon-agents/worktrees/tide-fx-qlek.20`, 2026-11-19; pinned toolchain/dependency sources read locally.

No build, compiled probe, runtime check, commit, bead write, or source edit was performed. Only this report was written; stage-2 implementation behavior remains unproven.

DONE
