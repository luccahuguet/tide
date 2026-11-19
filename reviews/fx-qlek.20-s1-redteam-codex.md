Model: gpt-6.1-sol (high) · Date: 2026-11-19 · Candidate d92305b72ab3983e35044e71a058c0a702d75fcd (red team, design)

## Verdict

**Sound with stated limits.** No blocking escape added by the five draws was established. Their source selection, range arithmetic and simulated ownership/order contracts hold on paper. The information-independence language needs the qualifications below: a shared simulated PRNG is neither an isolation boundary nor independent entropy, and the inherited real hasher is not an independent draw source. Confidence: high for these paper arguments; the future implementation is unproved.

Frozen branch: `fx-qlek.20`. HEAD: `d92305b72ab3983e35044e71a058c0a702d75fcd`. Base main: `4daf83f1ce7664884e866d33001af20bdf296533`. SHA-256 of the exact output of `git diff 4daf83f..HEAD`:

`04d106f024e922e6a93726623b7505047789ca2c4ee8de80024f9b24258b5765`

Read AGENTS.md, GOVERNANCE.md, `br --no-db show fx-qlek.20`, its comments, the complete six-file candidate diff, research/53, the specified design/RAND surfaces, landed capability construction, collections, simulator context/scheduler/manifest paths, and admission/authority mapping. Read the preceding supervisor report and re-check to avoid repeating their findings, and `reviews/fx-6d50-redteam-codex.md` for review depth. Consulted locally installed pinned Rust and cached rustix sources only. This is a red team of the added draw design, not a repeated supervisor pass or a whole-repository admission audit.

## Breaks (blocking)

None established. In particular, I found no specified route from an admitted program without a capability or declared hole to fresh OS entropy, and no same-identity simulated draw route to the OS. The existing real `from_rand` backend and its file fallback are an explicitly accepted limit, not an unresolved request to change that backend.

## Residual limits to state

### R1 — One simulated word can disclose the seed and predict other streams

**Route.** Give a component the first `next_u64` from a simulation, without its `Clock`, `Fs`, `Net`, `Proc`, world inspector or manifest. Let `M` be SplitMix64's output permutation, `C = 0x9e37_79b9_7f4a_7c15`, and `T = 0x7469_6465_7261_6e64`. The specified first word is

`x = M((seed ^ T) + C)` modulo 2^64.

The odd multiplications and xor-right-shifts are invertible. Ordinary integer computation recovers `seed = (M^-1(x) - C) ^ T`; no raw pointers, unsafe grant or external input is needed. The component can then calculate the initial scheduler/fs/net/proc states from their public XOR tags. Even without knowing the initial draw index, an output exposes the Rand stream state and permits prediction of subsequent Rand words. Another holder's secret-dependent decision to draw also changes this shared stream's position; clones are not isolated streams.

**Evidence.** `research/53-rand-v0.md:183–210` deliberately chooses a shared bijective stream and task-order consumption. Existing stream tags are `crates/tide-std/src/task/scheduler.rs:310–313`, the mixer is `:894–905`, and task selection uses that PRNG at `:654–678`. Research §7 says the code cannot choose the seed (`:323–330`); that does not mean it cannot discover it.

**Boundary.** This exposes simulator decisions, not host entropy or ungranted host files. Predicting a stream does not itself supply world setup, current positions of all other streams, or every future runnable set. An adversarial component can nevertheless recognize a simulated word sequence, predict known future choices, and tailor its behavior to tests. Replay remains exact. Seed secrecy, per-holder secrecy and adversarial test indistinguishability were not promised by the chosen simulator.

**Minimum clarification**, beside research §4/§7 and the §9.3 simulator description:

> Separate streams isolate consumption, not information. A simulated Rand word can reveal the seed and permit prediction of other simulator streams. Rand clones share draw position; there is no confidentiality boundary between their users. Simulation is for reproducible exploration, not hiding test decisions from hostile code.

No new generator, dependency or per-task stream is needed for the authorized contract.

### R2 — The inherited real hasher exposes correlated keys, not just independent random data

**Route.** An admitted caller can evaluate:

```rust
use std::collections::hash_map::RandomState;
use std::hash::BuildHasher;

let state = RandomState::from_rand(rand);
let exposed = format!("{:?}", state.build_hasher());
```

This is more than the prior re-check's `finish()` observation. The returned `DefaultHasher` has derived `Debug`, whose nested SipHasher debug output includes the numeric `k0` and `k1`. Tide's redacted `RandomState::Debug` does not redact the object returned by `build_hasher`. Two snapshots on the same real thread, surrounding another component's work, expose the increment in `k0`: delta 1 with no intervening Rust `RandomState` construction, delta 2 with one. They also reveal the key relation of adjacent maps constructed on that thread. Other intervening constructions make the measurement aggregate, not an identity oracle for a particular map.

**Evidence.** `crates/tide-std/src/collections.rs:28–40,49–52`; `crates/tide-admit/src/policy.rs:297,454–456`; trait-impl admission at `crates/tide-admit/src/walk.rs:391–418`. In the pinned `nightly-2026-08-09` source, `library/std/src/hash/random.rs:67–84,92–94` caches/increments the keys and returns the debug-capable hasher; `library/core/src/hash/sip.rs:19–23,51–59` derives Debug over the actual key fields. This is source inspection, not an executed probe.

**Boundary.** The new five draws do not inherit this TLS key relation. This route already exists, and research §5 now explicitly keeps the cached/incremented backend. It does not obtain a new capability or steer the fallback file path. It does defeat a grant-wide reading of `research/53-rand-v0.md:297–301` as guaranteeing that everything obtainable through Rand is independent of other program activity. Likewise, possession of a randomized builder is not a guarantee that its keys are private from its holder or from holders of neighboring thread keys. Hash-flooding resistance against external input is a weaker claim than isolation from a malicious in-process Rand holder.

**Minimum clarification:** scope §7's independence discussion to the new OS-source draws, and state:

> Real from_rand builders use correlated thread-local keys. Public hasher operations, including Debug on the built hasher, expose key-dependent data and on this pin the keys themselves; they can reveal intervening builder activity. This path is for map hashing, not cryptographic key generation or independent random draws.

Keep the already approved fallback/backend decision. This is a consequence of that retained design, not a repetition of the supervisor's fallback finding.

### R3 — Finite seeds bound coverage; distinct seeds do not imply distinct derived results

**Route.** Shuffle 21 distinct elements under every possible u64 seed, with identical program/setup. There are at most 2^64 resulting permutations, but `21! = 51,090,942,171,709,440,000 > 2^64`. Some permutations can never occur in this experiment. Much smaller derived results collide routinely: two seeds can return the same bool, bounded integer, byte prefix or map order. Rejection over a deterministic stream also does not establish independent ideal samples across seeds or successive calls.

**Evidence.** The seed/stream contract is `research/53-rand-v0.md:183–191`; the distribution language is `:93–115` and `design/reference-companion.md:816–818`. The bijection proves distinct **first raw words** for distinct seeds, not distinct high-level results. RAND-1's specific first-word witness and RAND-3's four-element coverage witness are appropriately narrower (`conformance/rules.md:107–109`).

**Boundary.** This is ordinary finite-state simulation, not modulo bias in the range algorithm or a replay failure. Fisher-Yates is uniform given independent uniform bounded draws; the simulator is a reproducible pseudorandom exploration source, not a proof of exhaustive coverage or exact joint distribution. The real source still claims only the OS generator's properties.

**Minimum clarification:** say that simulated distribution claims describe the sampling algorithm, not exhaustive or statistically independent seed coverage; reserve the no-collision statement for first raw words. Long simulated byte strings have at most the root seed's 64 bits of variation and are never secure keys. No algorithm change is required.

### R4 — Authority may be reported as opaque, and latency is observable through synchronization

**Routes.** A generic `fn apply<F: Fn() -> u64>(f: F)` can invoke a closure capturing Rand; `fn apply(f: &dyn Draw)` can dispatch to a Rand-bearing implementation. Their generic/erased public interfaces need not literally print `rand`: they print `opaque(...)`. Separately, a component with Tasks/synchronization can race a draw against another task and observe progress without reading a Clock.

**Evidence.** `design/03-authority-concurrency.md:1318–1328` explicitly preserves opaque declared authority; `crates/tide-admit/src/interface.rs:17–20,258–265,345–347` implements it. Structural Pure rejects Rand and non-Pure captures (`pure.rs:19–24,75`). Timing through synchronization is already admitted by `design/01-design.md:558–576`, and timing side channels are outside simulation coverage (`03:2450–2454`).

**Boundary/minimum clarification.** Research §8's “any item ... declares rand” (`:338–341`) should read “rand when structurally visible, otherwise opaque”; opaque is not an authority-free verdict. In §7, replace “latency ... observable only with a Clock” (`:300–301`) with “a Clock or granted synchronization/concurrency.” Neither route bypasses the existing declaration boundary; both qualify the new explanatory claims.

## Held attacks

“Held” below means the written design and inspected boundaries close the route; it does not mean stage 2 has passed execution.

- **Ambient randomized maps:** Rust RandomState constructors/default and implicit randomized map constructors are denied (`policy.rs:436–452`); Tide's map defaults use a zero-key DefaultHasher (`collections.rs:43–46,60–69`), so lifting from_rand's sim rejection need not seed default maps from the OS.
- **Ambient thread_rng/getrandom/dependency aliases:** no v0 dependency import supplies them; the symbol allowlist keys actual defining crates and ambient thread_rng is denied (`research/53:27–32,308–309`; `policy.rs:5–14,154–155`).
- **ASLR via raw pointers, Box formatting, address hashes or function-pointer casts:** raw-pointer types/observers and Pointer formatting need unsafe; Box/reference ordinary Debug/Hash follow the referent (`03:680–700`; `policy.rs:156–168`); an unsafe grant remains a declared hole, not closed ambient entropy.
- **TypeId:** admitted, but the pinned core's TypeId is a compile-time type hash at runtime, not an allocation address (`library/core/src/any.rs:724–730,764–768,843–872`); its instability across builds is covered by replay build identity, not promised away.
- **Clock-free ambient now/elapsed, process id and environment:** Tide's Instant has no ambient now/elapsed (`time.rs:83–113`), process exports omit id (`process.rs:14`), Env owns real reads and checks simulation identity (`env.rs:37–49`); new Rand methods add none of these exports.
- **Allocator state, uninitialized bytes, stack-depth and panic-message entropy:** safe construction gives initialized data; raw reads/forges require the declared unsafe boundary and cannot forge Rand (`pure.rs:141–176`; `root.rs:223–232`); exhaustion/stack overflow are explicitly outside exact determinism (`03:729–751`), and panic text is not an ambient memory-reader.
- **Get entropy through Fs, Net or Proc instead:** possible with those grants and already stated (`research/53:313–321`); these are wider authorities, not an undeclared Rand route or a promise that all randomness requires the literal rand kind.
- **Real Rand captured into a simulated task, destructor or nested scope:** research requires the real/sim check on use (`:208–210`); the reusable check examines the current runtime, not creation/capture site (`task.rs:488–524`), so cloning or moving the handle cannot legitimize it.
- **Foreign/expired simulated Rand used outside its own run:** the same context identity check rejects it; root construction retains simulation identity (`sim.rs:47–57`), and Rand's private field has no public constructor/default (`root.rs:223–232`). Pure sampled data may escape as ordinary data; the handle cannot thereby gain real authority.
- **Clone Rand to restart the stream or attenuate/forge it:** clones share one sim stream (`research/53:183–195`); real clones copy only identity (`:133–136`); neither cloning nor sampled bytes mint a grant, and no attenuation is promised.
- **Schedule from parallel branches or authority-free callbacks:** Pure capture/payload rules exclude Rand from parallel regions (`research/53:192–196`; `03:1299–1302`); generic/dyn erasure remains opaque rather than proving Pure.
- **Race draw assignment under sim:** one baton task executes between points, and each outer call owns one point and all its word consumption (`research/53:192–202`; `task.rs:538–549`; scheduler selection `:654–678`); OS thread arrival does not select the draw recipient.
- **Cancel halfway through rejection, shuffle or fill:** Rand calls are non-checkpoints (`03:382–401`); cancellation cannot truncate them. Helpers must consume the shared stream within that one outer operation, not introduce separately scheduled random_range calls per swap.
- **Consume real entropy through simulated from_rand:** its specified branch takes two simulated words and prefixes a zero-key DefaultHasher (`research/53:224–233`); DefaultHasher::new uses key (0,0) in pinned std (`library/std/src/hash/random.rs:104–108`), so no RandomState::new is needed in that branch.
- **Empty/reversed/signed/full-width ranges:** validate ordering before wrapped cardinality; signed offset avoids signed arithmetic overflow, and only the full 64-bit inclusive domain uses wrapped size zero (`research/53:93–105`); a singleton has threshold zero and still consumes a word.
- **Bool NaN, infinities, subnormals, negative zero and endpoints:** ordered membership validation rejects NaN/infinities; -0.0 equals 0.0, subnormals below 2^-64 yield threshold zero, p=1 uses the explicit true branch, and every valid p consumes one word (`research/53:106–111`).
- **Empty/singleton/huge shuffle:** zero/one length consumes no word; descending Fisher-Yates must guard these before computing len-1, and range size for a valid Linux slice index fits u64 (`research/53:112–115`); nonempty work is unbounded computation, not a cancellable service.
- **Empty/huge fill and interrupted OS reads:** empty fill consumes none; simulated ceil(len/8) must be computed without overflowing len+7, while real short reads/EINTR loop until full and other errors panic (`research/53:89–92,127–150`); OS failure can leave a partially filled real destination, with no transactional guarantee.
- **Panic after a draw, or draw from Drop during unwind:** no rewind is specified; consumed words remain consumed. Deterministic Drop calls use the active task's stream/points, including when canceled; a second panic during unwind aborts under PANIC-2 (`01:380–392`; `03:749–751`; `task.rs:1713–1720`).
- **Crypto consumer silently gets a secure simulated stream:** the same Rand type intentionally selects by runtime identity, but the companion explicitly says simulated values are predictable and never secret (`companion:824–831`; `research/53:138–141,323–330`); cryptographic suitability is conditional on the real OS source, not a Rand type-level guarantee.

## Reasoning

The observable contract is a function holding a Rand receiving source-derived values through five non-checkpoint methods; invalid range/probability arguments panic before drawing, real source errors panic, and valid simulated consumption is determined by seed, inputs, execution order and recorded std/build identity. The source split must happen before any real source access, including when a real handle is captured into a sim task. For valid empty operations, the identity rule still applies even though no word is consumed.

The range proof is the right one: for nonzero cardinality s, accepting `[2^64 mod s, 2^64)` gives a multiple of s words, hence modulo reduction adds no bias for uniform source words. Full-width and signed mapping avoid the common overflow/zero-divisor holes. Bool's explicit p=1 branch avoids an unrepresentable 2^64 threshold; scaling a binary f64 by 2^64 does not create rounding bias in this interval. Shuffle needs no user callbacks and swaps do not invoke T's Drop, so no user panic or cancellation can enter between its random choices under the specified one-call scheduling rule. Huge inputs add resource/time risk, already outside bounded replay guarantees, not authority.

Panic behavior needs careful implementation rather than a new policy. Invalid arguments consume zero words in that operation, but unwinding may run a destructor which itself draws; the total stream can therefore advance after an invalid call. A successful call followed by a user panic retains its draws, and cancellation does not suppress destructor draws. One simulated call has no ordinary mid-draw error path: rejection and swaps run under the baton with no internal scheduling point. The existing scheduler can itself issue a deterministic starvation panic at a point (`scheduler.rs:400–435`); a draw's pre/post-point placement must be consistent and must not unwind while holding the runtime lock (`task.rs:543–549`). OOM, native stack exhaustion, and a nested unwind panic keep their existing stated exceptions. No rollback or all-or-nothing real fill promise should be inferred.

The highest useful stage-2 witnesses are therefore the planned cross-task replay plus: cloned/borrowed handles in nested scopes; real/foreign handles including empty operations; rejection and validation draw counts; and a Rand-bearing Drop on ordinary, cancellation and isolated-panic paths. Trace recording must count actual consumed words, with one outer operation event rather than one per helper draw. Fill/shuffle events record lengths rather than full output, so equal traces alone do not prove equal byte/permutation results; RAND-1 must compare those actual results too, as research §12 requires. These are implementations of the admitted contract, not extra feature scope.

Replay uses recorded build/std/target identity and exact traces (`replay.rs:39–72`; `sim/manifest.rs:205–230`), so a source algorithm change or native-endian hasher behavior on another target is not permission to replay under a different identity. Same seed alone is not sufficient across changed code, setup, target or std. Separate streams keep no-draw baselines stable; once a draw call adds a scheduling point, other tasks' assignments and scheduler consumption can change. Empty fills/shuffles still have the call's scheduling point despite consuming no random word. None of these facts contradicts same-input, same-identity replay.

Only this report was written. No builds, tests, scripts/probes executing Tide or sampling algorithms, runtime replay, network requests, third-party interactions, implementation edits, commits or .beads writes were performed. Stage-2 enforcement and actual replay remain unverified; the residual information/coverage limits above must not be mistaken for stronger guarantees.

DONE
