Model: Claude Opus 5.5 · Date: 2026-11-19 · Candidate b586c19 (code, stage 2)

## Verdict

**Changes required.** There is one blocking finding, and it is small: the proof of RAND-4's "flags 0" clause cannot fail. The fix is one guard line. The rest of the change matches the landed contract on both backends: arithmetic, word counts, the order of panics, replay and stream isolation, the real source, authority, and the docs. Everything else below is non-blocking.

## Scope

I reviewed the whole diff `git diff e2f2d26 b586c19` (23 files). I read it against these sources:
- research/53 §§2–5, 8 and 12;
- 03 RAND-DRAW, the Rand rows and §9.3;
- companion §22;
- rules.md RAND-1..4;
- the stage-1 red team R1–R4, read from `2e6f95a`;
- the builder's report;
- the Codex pass (`s2-codex.md`) and its re-check (`s2-recheck-codex.md`).

Codex's B1 (exhausted `RangeInclusive`) and N1 (stale README and owner-guide status) are fixed in this tree, and I don't repeat them. I worked read-only: no builds, no runs, and the only file written is this report.

## Blocking findings

### B1. RAND-4's "flags 0" clause has no check that can fail

- **Where:** `crates/tide-std/src/rand.rs:228`, `crates/tide-admit/tests/build.rs:4293`, `conformance/rules.md:110`.
- **What the contract says:** research/53 §3 makes real draws `getrandom(2)` with flags 0, which blocks until the pool is initialized and then gives CSPRNG output. That is the property that justifies keys and nonces (fx-zu4f.14). RAND-4's row says the seam test checks the draws "all through `rustix::rand::getrandom` with flags 0".
- **Why no check covers it:**
  - The `#[cfg(test)]` seam replaces the rustix call, so it never sees the flags.
  - The seccomp child denies `getrandom` whatever its flags are.
  - The guard counts only the substring `rustix::rand::getrandom(`.
- **Failure scenario:** a later edit changes line 228 to `GetRandomFlags::INSECURE`, which can return output from an uninitialized pool, or to `NONBLOCK`, which surfaces `EAGAIN` as a panic. All four RAND tests still pass, and the conformance row still claims the clause is checked.
- **Minimum fix:** in `std_rand_public_items_are_fixed`, assert that the one call site is exactly `rustix::rand::getrandom(dest, rustix::rand::GetRandomFlags::empty())`. That could be a `contains` on the full call next to the count of 1. Alternatively, let the seam's `call` take and record the flags and assert they are empty. Either way, the row's clause then names a check that can fail.

## Non-blocking findings

- **N1. RAND-3 pins neither the shuffle method nor the signed offset.** `rand/tests.rs:94` checks only that all 24 orders occur.
  - A forward Fisher-Yates, or any shuffle that reaches every order, would pass. The stated method is from-end, with `random_range(0..=i)` per step, and that method is what fixes which words a seed's shuffle takes.
  - Signed `random_range` is checked only for a singleton (`-3i8..=-3`) and for bounds in the conformance case. No case pins `low + x % s` through the i128 offset mapping against the reference words.
  - The RAND-3 row as written is met. Two assertions against `reference()` would make the "follows the stated methods" title fully true: one `shuffle` of `[0,1,2,3]` and one `random_range(-5i8..5)`.
- **N2. Nothing checks that `random_bool(0.0)` and `random_bool(1.0)` take one word.**
  - research/53 §2.1 draws a word for every `p`, so that the stream position does not depend on `p`.
  - `rand.rs:167` does this correctly: `next() < threshold` is evaluated before `|| p == 1.0`.
  - If someone reorders it to `p == 1.0 || next() < …`, no test fails: the conformance case asserts only the values, and the determinism self-check reruns the same code.
  - Fix: add both calls to `simulated_edge_cases_take_no_word`'s expected word counts, each with 1.
- **N3. The public-items guard reads only `rand.rs`.**
  - It doesn't see `pub struct Rand`, which `capability!` generates.
  - It doesn't see inherent `impl Rand { pub fn … }` blocks elsewhere in the crate, which Rust allows in any module.
  - This is the same file-scoped limit as the existing NET-HTTP-4 and PROC-3 guards, so it's acceptable. The row could say so.
- **N4. The trust-diff test checks the parameter, not the draws.**
  - `rand_draws_are_admitted_with_rand_in_the_trust_diff` shows that a function taking `&Rand` is admitted and declares `rand`.
  - v0 reports declared authority only (§8). So the same diff appears whether or not `deal` draws.
  - That matches the contract. The test's value is admitting all five draws through the driver, and its name and doc say as much.
- **N5. The build test duplicates the expected string.** `build.rs:4180-4184` repeats the literal `EXPECTED` that the case already asserts internally. A mismatch already makes the case panic and `checked` fail. The build test could compare the simulated values to `real` and drop the copy. This is cosmetic.
- **N6. The owner-guide quote now differs from its source.**
  - `docs/owner-guide/10-v0-and-planned.md:87` presents a quoted `br` listing of fx-muza with `TD0516`.
  - The bead's actual title still says `FX0516`.
  - The code_prefix fix is right for live text. Retitling fx-muza through `br` would make the quote true again. That is a coordinator action, outside this diff.
- **N7. Nits:**
  - `root.rs`'s `capability!` comment ("distinguishes simulated identities before their operations are modeled") and its `#[allow(dead_code)]` now apply only to `Stdin`.
  - 03 §4.3's `rand` line (`design/03-authority-concurrency.md:1396`) describes `from_rand` keys as "Rust's cached OS keys" without saying that this is the real backend. That text is from stage 1, not this diff, and it is later-tier, so it goes as a comment on fx-djp if anywhere.

## Reasoning

### Arithmetic

- **Offsets:** the i128 mapping sends each of the ten types to ordered `u64` offsets, and back, without overflow.
- **Range sizes:**
  - `a..b` gives a positive difference.
  - `a..=b` wraps to 0 only for the full 64-bit domain, and `below(0)` then takes exactly one word.
  - Full `u8`/`u16`/`u32` ranges get threshold 0, so they never reject.
- **Rejection:** it matches the scheduler's `index`, now sharing one `splitmix`, with old streams bit-identical.
- **Emptiness:** `is_empty()` covers both reversed and exhausted inclusive ranges, and `start < end` covers `a..b`.
- **`random_bool`:**
  - `contains` rejects NaN, ±inf, `-0.1` and `1.1` before dispatch.
  - `-0.0` is accepted and yields false.
  - `(p·2^64) as u64` is exact `floor` for `p < 1`.
- **`shuffle`:** it iterates `(1..len).rev()`, so length 0 and length 1 take no word, and there is no underflow.
- **`fill_bytes`:** simulated, it uses `chunks_mut(8)` with the last chunk truncated little-endian; an empty slice takes zero words.

### Panic order and word counts

On both backends, argument checks run before the identity check, and the identity check runs before any word or scheduling point.
- **Simulated:** `rand_op` takes the point, then draws all the call's words under the runtime lock while the task holds the baton, then records one `rand` event with the actual count. Nothing run under the lock can call user code: only arithmetic, `slice::swap`, and `Display` of the sealed integer types.
- **Real:** each word is one 8-byte request. `fill` loops on short reads, retries `EINTR`, and panics naming any other errno.

### Replay

- **Stream:** the `rand` stream is `seed ^ "tiderand"` and is touched only by `rand_word`, so a run that never draws is unchanged.
- **Event contents:** each event carries words and the result or the length, so a divergence shows at the first differing event.
- **What RAND-1 checks:**
  - The unit test checks:
    - identical draws, trace and map order per seed;
    - different first words for seeds 0 and 1;
    - that 1 word versus 100 words leaves the non-`rand` lines identical, with `"latency"` and `sched` lines present, so the check is not vacuous.
  - The build test replays a manifest byte for byte, with `rand` events and the `from_rand` event in it.
- **Simulated `from_rand`:** it uses `DefaultHasher::new()`, which has a fixed key, with a two-word prefix. No OS entropy is reachable under simulation.

### Real source

The only production OS call is `rustix::rand::getrandom` at `rand.rs:228`: no file path and no fallback.
- **The seccomp child:** RAND-4's filter returns `EPERM` for `getrandom` only. The parent confirms that `/dev/urandom` opens, and the filter leaves file reads allowed. So any file fallback would answer and fail the exit-101, `EPERM` and empty-stdout assertions. This proves "the real source is `getrandom` and nothing answers in its place". It is a behavioral check, not an `openat` trace; the row's reworded clause says exactly that.
- **The seam:** it proves the loop's handling of short reads, `EINTR` and errors, and that empty or rejected calls make zero requests. It proves nothing about flags (B1).
- **Dependency:** `rustix =1.1.6` stays pinned in both std manifests. Only the `rand = []` feature is added, the lockfile is unchanged, and research/53 §3.1 holds the AP-DEPENDENCY-001 record. No production `unsafe` was added.

### Authority

- **Who can draw:** every draw requires `&Rand`, and the `Option<Tasks>` field is `pub(crate)`. `pure.rs:75` maps `Rand` to `rand`.
- **Sealed traits:** their supertraits sit in a private module, and their methods take a `Token` that only `rand` can build. So the traits add no callable surface and can't be implemented outside.
- **Surface:** exactly five public draws and two sealed traits; `rng()` stays planned everywhere.

### Leftovers and docs

- `Tasks::unmodeled` is removed along with its only caller.
- The sim.rs test is adjusted correctly: its stale-handle `from_rand` assertion still holds through `Context::simulated`.
- The `#[cfg(test)] mod tests;` under a `#[path]` include is harmless, because the fixture std is never built with `cfg(test)`.
- The status text in README, the owner guide, lib.rs, reference and companion all match what ships, with `rng() -> Pcg64` labeled planned.
- The builder recorded a reference recount of 24,932 tokens, which is within D14's budget; I did not rerun the recount.

DONE
