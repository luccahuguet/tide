# Rand draws in v0 (fx-qlek.20)

Status: stage 1, the design admission. Stage 2 builds it. This note fixes
which `Rand` operations v0 admits, their contracts, the real backend and
its dependency, the simulated backend and its replay reading, what
`RandomState::from_rand` does under simulation, and the trust-diff
reading, and gives a soundness verdict per operation. Labels follow
research/48: **[repo evidence]**, **[source]**, **[inference]**.

## 1. Need

**[repo evidence]** Owner decision, relayed by tide-desk and recorded on
fx-qlek.20 (2026-11-19): asked whether std needs work before the alpha,
Lucca chose option 1, "Add Rand draws to the alpha, nothing else". No
UDP, no broader std work. The bead's constraints: simulated draws come
from the seed and replay exactly; real draws come from the OS
(`getrandom`), through a dependency that passes AP-DEPENDENCY-001; the
surface stays minimal, each operation admitted on its own; no
cryptographic claim beyond the OS source's.

**[repo evidence]** What v0 has today. `Rand` is a capability identity
with no methods (`crates/tide-std/src/lib.rs`, `pub mod rand`). `Root.rand`
and `sim.rand` exist (`root.rs`, `sim.rs`); its one use is
`RandomState::from_rand`, which takes Rust's OS-seeded keys on the real
backend and panics under simulation ("until its backend exists",
`collections.rs`). Reference §6.1 lists six draws as planned, and §6.4
and §6.8 mark `Rand` partial. Rust's ambient routes are closed: `rand`
is not a v0 dependency (crate imports are v1), `thread_rng` is `TD0501`
(companion 14.1), and Rust's `RandomState::new` and its `Default` are
denied (`policy.rs`, `RANDOM_KEY`). So a v0 program cannot draw a random
number today, except by reading `/dev/urandom` through an `Fs` that
reaches it, which is wider authority and has no simulated model.

**[inference]** What honest programs draw for:

1. **jitter and backoff**: a retry loop sleeps a random fraction of its
   delay (`random_range(0..=max)`), so clients do not retry in lockstep;
2. **sampling**: log or trace sampling at a rate (`random_bool(p)`), and
   picking a replica or a test subset (`random_range(0..n)`);
3. **shuffles**: randomized work order, load spreading, card and puzzle
   programs, test-order randomization (`shuffle`);
4. **identifiers and tokens**: request ids, temporary names, nonces and
   session tokens (`fill_bytes`, `next_u64`);
5. **test data** under `#[sim_test]`, where the seed decides the draws
   and replay reproduces them;
6. **keys**: fx-zu4f.14 (post-quantum crypto) treats "randomness as a
   capability" as its consumer. It needs bytes from the OS source
   (`fill_bytes`) and nothing about their quality beyond the OS's.

## 2. The surface

**[inference]** Five methods on `Rand`, all `&self`, none suspending,
none a checkpoint (03 §1.1's table already says Rand suspends nowhere):

```rust
impl Rand {
    pub fn next_u64(&self) -> u64;
    pub fn fill_bytes(&self, dest: &mut [u8]);
    pub fn random_range<T: SampleUniform, R: SampleRange<T>>(&self, range: R) -> T;
    pub fn random_bool(&self, p: f64) -> bool;
    pub fn shuffle<T>(&self, slice: &mut [T]);
}
```

`SampleUniform` and `SampleRange` are sealed traits in `std::rand`.
`SampleUniform` is implemented for `u8`, `u16`, `u32`, `u64`, `usize`,
`i8`, `i16`, `i32`, `i64` and `isize`. `SampleRange<T>` is implemented for
`Range<T>` (`a..b`) and `RangeInclusive<T>` (`a..=b`). Neither trait has
a method a program can call.

**Names.** The planned names were rand 0.8's `gen_range` and `gen_bool`.
rand 0.9 renamed `Rng::gen_range` to `random_range`, `gen_bool` to
`random_bool` and `thread_rng()` to `rng()`, deprecating the old names,
and rand 0.10 keeps them ([rand 0.9 update guide](https://www.rustmax.net/library/rand-book/update-0.9),
[rand 0.10 source](https://docs.rs/rand/0.10.0/src/rand/rng.rs.html)).
`next_u64` and `fill_bytes` are unchanged `rand_core` names. Tide has not
built these methods yet, so it takes the current upstream names: code
written against today's `rand` ports unchanged, and companion 14.2 maps
the rand 0.8 spellings. One name per operation; no aliases.

### 2.1 What one draw is

Every method is defined over **words**: a word is 64 bits from the
backend's source (§3, §4). The methods consume words as follows, the
same on both backends, so a seed's simulated draws are a function of
the stream position alone.

- **`next_u64()`**: one word.
- **`fill_bytes(dest)`**: the bytes of `dest` come from the source. On
  the simulated backend they are `ceil(len / 8)` words written
  little-endian, the last word's unused high bytes discarded. On the real
  backend they are the OS's bytes (§3). An empty `dest` takes no word.
- **`random_range(range)`**: uniform over the range, with no modulo bias.
  Let `s` be the number of values in the range, computed in `u64` after
  mapping signed values by offset. If the range covers all 2^64 values of
  a 64-bit type, `s` wraps to 0 and the result is one word, offset to the
  type. Otherwise the threshold is `t = (2^64 - s) mod s`
  (`s.wrapping_neg() % s`); draw words until one is `x >= t`, and return
  `low + x % s`. Exactly `2^64 - t` words are accepted, a multiple of `s`,
  so each value has the same probability, and fewer than one word in two
  is rejected even in the worst case. This is the simulator's own `index`
  function (`task/scheduler.rs`). **An empty range panics** before any
  draw: `a..b` with `a >= b`, or `a..=b` with `a > b` (Rust's `rand`
  panics too; an empty range is a caller bug, like an index past the
  end).
- **`random_bool(p)`**: **`p` outside `[0.0, 1.0]`, NaN included, panics**
  before any draw. Otherwise one word `x`; `p == 1.0` returns `true`, and
  any other `p` returns `x < floor(p * 2^64)`. Multiplying by 2^64 is exact
  in `f64`, so the probability is exactly `floor(p * 2^64) / 2^64`, and
  `p == 0.0` is never true. The word is drawn for every `p`, `0.0` and
  `1.0` included, so the stream position does not depend on `p`.
- **`shuffle(slice)`**: Fisher-Yates from the end: for `i` from
  `len - 1` down to `1`, `j = random_range(0..=i)`, then swap `i` and `j`.
  Each of the `len!` orders is equally likely. A slice of length 0 or 1
  takes no word.

Simulated distribution claims describe the sampling algorithm, not
exhaustive or statistically independent seed coverage: "uniform" and
"equally likely" hold for uniform independent words. Under simulation a
run has at most 2^64 seeds, so a shuffle of 21 or more elements cannot
reach every order (21! > 2^64), and two seeds can give the same bool,
bounded integer, byte prefix or map order.

Panics vs errors: no draw returns an error. The argument checks are
caller bugs, so they panic with a message naming the method and the
argument. The OS source cannot fail on the kernels tide-std supports
(§3); if it does, the draw panics with the OS error. A `Result` on every
draw would put a `?` or an `unwrap` at every use for a failure no
supported system produces; Rust's `rand` makes the same choice for its
infallible methods.

## 3. Real backend: the OS source

**[repo evidence, source]** Real draws read Linux's `getrandom(2)` with
flags 0: it blocks only until the kernel's random pool is first
initialized at boot, then never blocks, and returns the kernel CSPRNG's
output ([getrandom(2)](https://man7.org/linux/man-pages/man2/getrandom.2.html)).
A request may return fewer bytes than asked when a signal interrupts it;
the backend loops until `dest` is full and retries `EINTR`. A word is 8
OS bytes read little-endian. A call may request its words in one batch
(for example `shuffle` of a long slice), and no OS bytes outlive the call:
`Rand` holds no buffer, so it stays stateless and `Clone` copies only the
identity.

**[inference]** Draws make no claim beyond the OS source: they are as
unpredictable as the kernel's `getrandom` output, which is what keys,
nonces and tokens need (fx-zu4f.14), and nothing more. Simulated draws
(§4) are predictable from the seed and never secret.

**Failure.** tide-std's real files need `openat2` (Linux 5.6) and its
spawns `close_range` (5.11) (lib.rs); `getrandom(2)` dates from 3.17, so
`ENOSYS` does not occur on a kernel tide-std supports. RUN-3's seccomp allowlist admits `SYS_getrandom`
(`crates/tide-admit/src/invocation/sandbox.rs`). A foreign seccomp filter
that denies it gives `EPERM`; the draw panics with that error rather than
falling back to a file read. This getrandom-only, no-fallback contract
covers the five draw methods; real `RandomState::from_rand` keeps Rust's
inherited path, which can fall back (§5).

### 3.1 The dependency (AP-DEPENDENCY-001)

**Capability needed**: fill a byte buffer from `getrandom(2)` with flags
0, without `unsafe` in tide-std. Candidates:

| Option | Fit | Cost |
| --- | --- | --- |
| **rustix `=1.1.6`, feature `rand`** (chosen) | `rustix::rand::getrandom(buf, GetRandomFlags::empty())`: the raw syscall through the linux_raw backend, no libc, returns the count so the caller loops | already a direct, pinned tide-std dependency (`fs`, `std`, `event`, `net`); `rand = []` adds one module and no crate, so `Cargo.lock` is unchanged; Apache-2.0 WITH LLVM-exception OR Apache-2.0 OR MIT, as already accepted |
| `getrandom =0.2.17` | correct; already locked through `ring` | a second direct wrapper for the syscall that pinned rustix already owns. Its default Linux backend falls back to reading `/dev/urandom` on `ENOSYS` or `EPERM` (`linux_android_with_fallback.rs`); its `linux_disable_fallback` feature removes that path, so the fallback alone does not rule it out |
| std's `std::random::DefaultRandomSource` | `getrandom(0)` with a `/dev/urandom` fallback (`library/std/src/sys/random/linux.rs`) | unstable on the pinned nightly (`feature = "random"`, issue 130703); tide-std enables no features today and would track a moving API at each pin |
| owned `syscall(SYS_getrandom)` | exact | `unsafe` and a libc dependency in tide-std for what rustix already does safely |
| `ring::rand::SystemRandom` | reaches `getrandom` 0.2 | a crypto library's indirection for a plain byte fill |

**Choice**: rustix's `rand` feature. Relied-on behavior: one
`getrandom(2)` call per invocation with the given flags, returning the
count written; proved in stage 2 by a `#[cfg(test)]` seam (§9, RAND-4).
Replacement cost: about ten lines of the loop, against any of the other
rows. The vDSO `getrandom` (Linux 6.11) is not used (rustix's documented
choice); a draw costs one system call, or one per batch. Programs that
need millions of draws per second wait for the generator in §6.

## 4. Simulated backend

**[repo evidence]** The scheduler keeps one SplitMix64 state per
sub-stream, each the seed XOR a tag: `rng` ("tideschd"), `fs_rng`,
`net_rng`, `proc_rng` (`task/scheduler.rs`). 03 §9.9 already keeps the
process stream separate "from task/fs/net randomness" so runs that
spawn nothing keep their draws.

**[inference]** Simulated draws follow the same pattern:

- **One `rand` sub-stream per simulation**, SplitMix64 from
  `seed ^ 0x7469_6465_7261_6e64` ("tiderand"). A word is one SplitMix64
  output. Draw words never come from the scheduler, file, net or process
  streams; the call's scheduling point uses the scheduler stream like any
  capability operation. A run that never draws makes exactly today's
  draws in every stream.
- **Same seed, same draws; another seed, other draws.** The state is a
  function of the seed, and each output is a function of the state. The
  first word differs for any two seeds: the state step adds a constant and
  SplitMix64's output mix is a bijection of `u64`, so distinct states
  give distinct outputs. That no-collision statement is for first raw
  words only; later words, and results derived from words, of two seeds
  may coincide. Long simulated byte strings have at most the root seed's
  64 bits of variation and are never secure keys.
- **Order across tasks.** Every draw is a scheduling point with zero
  latency and no checkpoint, as `Env` reads are (03 §9.2). One task runs
  at a time between points, so the seed's interleaving fixes which task
  takes which word. Clones of `sim.rand` share the stream. Parallel regions
  cannot draw: a `par` closure holds no capability (03 §8.1 rule 3).
- **Trace.** Each call records one `rand` event with its task, operation,
  span, the number of words taken, and the result for `next_u64`,
  `random_range` and `random_bool` (the length for `fill_bytes` and
  `shuffle`). The determinism self-check and `--replay` compare traces
  line by line, so a divergence reports the first differing `rand` event
  with its span (03 §9.7).
- **Replay manifest.** Draws are not injected inputs: they are a function
  of the seed, which the manifest records, and of std, whose content hash
  it records (03 §9.2). They appear in the manifest's recorded trace as
  `rand` events. `VERSIONS` and the manifest format do not change: a
  manifest recorded before draws existed could not have drawn.
- **Identity checks.** A real `Rand` used inside a simulation panics, as a
  real `Fs` does; a simulated one used outside its own simulation panics
  (`Tasks::context`).
- **Not modeled.** OS failure and boot-time blocking (§3) have no
  simulated counterpart; simulated draws never fail or block.
- **Information.** Separate streams isolate consumption, not information.
  A simulated Rand word can reveal the seed and permit prediction of other
  simulator streams. Rand clones share draw position; there is no
  confidentiality boundary between their users. Simulation is for
  reproducible exploration, not hiding test decisions from hostile code.
  (§7.)

**Rejected:** per-task streams (a task's words independent of the
interleaving). They keep a task's draws stable when an edit adds a
scheduling point elsewhere, but need a stable task identity across
nodes and nested scopes for no current test. Drawing from the
scheduler's own stream: a run that draws changes its schedule either way,
through its new scheduling points, but with a shared stream the words a
call consumes (rejections, shuffle length, fill length) would also shift
every later scheduling choice. With a separate stream, word consumption
leaves later scheduling choices unchanged.

## 5. `RandomState::from_rand` under simulation

**[inference]** Decided: the rejection is lifted. Under simulation,
`from_rand(&sim.rand)` takes two words `k0, k1` from the `rand` stream
(one scheduling point and one `rand` event, operation `from_rand`) and
returns a builder whose hasher has absorbed `k0` and `k1` before any key
bytes: `DefaultHasher::new()` (SipHash-1-3, key 0) after `write_u64(k0)`
and `write_u64(k1)`, cloned for each hash. Iteration order then varies by
seed and replays exactly, so a sim test of a server that follows GLOBAL-5
runs, and exposes code that depends on one map order. Hash flooding is not
a simulated threat, so the prefix construction needs to be deterministic
and seed-varying, not a keyed PRF.

**Real backend: unchanged, a stated limit.** Real `from_rand` keeps
Rust's `std::hash::RandomState::new()` (`collections.rs`). On the pinned
nightly that takes a thread-local key pair, initialized once per thread
by `hashmap_random_keys()`, and increments `k0` on each call
(`library/std/src/hash/random.rs`). The initialization calls
`getrandom(2)` with `GRND_INSECURE` (or `GRND_NONBLOCK` where that flag
is missing) and reads `/dev/urandom` instead when the call fails with
`ENOSYS` or `EPERM`, or with `EAGAIN` before the pool is initialized
(`library/std/src/sys/random/linux.rs`). So `from_rand`, unlike the five
draws, can read a file under a foreign seccomp filter, does not make a
system call per call, and gives keys that may come from the pool before
it is initialized. This is acceptable for its purpose, hash-flooding
resistance, and it reaches no resource the process does not already
have. This bead keeps it (tide-lead's decision, round 1): the owner's
decision asks for draws, not a new seeding path. A getrandom-only
contract for the whole grant would need real `from_rand` to key from
`Rand`'s own draws, which needs a keyed hasher of type `DefaultHasher`
that stable std does not offer, and is a separate change.

Kept: `RandomState` stays one type with `Hasher = DefaultHasher`, so no
signature changes. Rejected: the fixed key under simulation (deterministic,
but every seed sees one order, hiding order dependence) and keeping the
panic (a GLOBAL-5 program cannot be sim-tested, which is the gap the
current comment says lasts "until its backend exists").

## 6. `rng() -> Pcg64`: not admitted to the alpha

**[inference]** The sixth planned operation stays planned (bead fx-9bzp,
deferred). Its design is sound as far as it goes: `Pcg64` is a `Pure`
value seeded by one draw, so it cannot break replay (the seeding draw is a
`rand` event; later draws are computation) and it carries no authority, so
a function given `&mut Pcg64` honestly shows none. "Authority is spent at
seeding" (the former 03 §1.1 note) is the correct reading, and
`Pcg64::seed_from_u64(n)`, seeding from a plain value, needs no
capability; fx-9bzp inherits both points. It is deferred because:

- it is a second type with a second copy of every draw method, two ways
  to draw where one suffices for every need in §1;
- a seedable generator's output is a value-stability contract
  (`seed_from_u64(n)` must give the same stream on every toolchain), so its
  algorithm (PCG-XSL-RR-128/64, Xoshiro256++, ChaCha) and its seed
  expansion need their own admission;
- `Pcg64` is `Clone`, and cloning a generator repeats its stream, a known
  porting trap that needs its own teaching;
- it is not a CSPRNG, and a crypto consumer (fx-zu4f.14) must not reach
  for it; the admitted surface has only the OS source;
- the owner's decision limits the alpha to draws; the only cost of
  deferral is a system call per draw (§3.1).

So 03's simulation example and its habit row move from `rng()` to the
admitted draws, and reference §6.1 keeps `rng()` as planned.

## 7. Authority, pre-answering the red team

**[inference]** What a `Rand` grant lets a program do: consume
`getrandom(2)` output through the five draws, and obtain hash keys through
`RandomState::from_rand`, whose real keys come from Rust's inherited
path (§5): cached per thread, and read from `/dev/urandom` when
`getrandom(2)` is denied or the pool is not yet initialized. That file read
is the process's own std, not a path a program can steer. A grant writes
nothing to the world and reaches no other resource. Since Linux 5.6 the kernel's generator neither blocks nor
depletes after initialization, so draws cannot starve other processes. A draw can block only before the
pool is initialized at boot.

What the five OS-source draws let a program learn: bytes computationally
independent of everything else the program or any other process can see,
as far as the kernel's generator is. `getrandom(2)` is not an
observation channel: its output reveals no state of another process or
of the machine. Its latency is a system call's, observable only with a
`Clock` or granted synchronization/concurrency.

Real from_rand builders use correlated thread-local keys. Public hasher
operations, including Debug on the built hasher, expose key-dependent data
and on this pin the keys themselves; they can reveal intervening builder
activity. This path is for map hashing, not cryptographic key generation
or independent random draws.

**Randomness without `Rand`.** Every source of unpredictable values a v0
program could reach, and its status:

| Source | Status |
| --- | --- |
| Rust's `RandomState::new`, its `Default`, Rust's `HashMap` with it | closed: denied (`policy.rs`, `RANDOM_KEY`); Tide's `HashMap::new()` uses a fixed key |
| `thread_rng`, `rand::random`, `getrandom` crate | closed: not dependencies of v0 programs; ambient forms are `TD0501` or unresolved; v1 crate imports map the `getrandom` symbol to `rand` (03 RUST-3) |
| addresses (ASLR): `{:p}`, `addr`, pointer casts, hashing a pointer | closed in safe code (GLOBAL-4); open under the `unsafe` grant, a declared trust hole |
| wall or monotonic time, latency | needs `Clock` |
| task interleaving on the real runtime (who acquires a `Shared` first) | through `tasks`, a declared kind; seeded under simulation |
| `/dev/urandom`, `/proc/sys/kernel/random/uuid` | through `fs.read` on a `Fs` that reaches them; absent from simulated files |
| a remote service, a child process (`head -c8 /dev/urandom`) | through `net` or `proc` |
| uninitialized memory, data races | closed in safe code (02, 03 §3) |
| parallel regions | deterministic by construction (03 §8.1) |

Residuals accepted: the `unsafe` grant, `tasks` interleaving on the real
runtime, and wider `fs`, `net` or `proc` grants. Each is already a
declared kind or hole, wider than `rand`, so none makes `rand` reachable
without a declaration.

**Simulated draws: leak or force?** They are a function of the seed and
the call order. Code under test cannot choose the seed: `Root::sim` and
`#[sim_test]` exist only in test code (ROOT-3), and the runner picks the
seeds. A test author can search for a seed that yields given draws; that
is test code choosing its own inputs. Anyone with a manifest can compute
every simulated draw, so simulated keys are never secret, which is why
§3's claim is only for real draws. A simulated `Rand` cannot reach another
simulation's stream (identity check, §4), and no simulated value reaches a
real run. Separate streams isolate consumption, not information. A
simulated Rand word can reveal the seed and permit prediction of other
simulator streams. Rand clones share draw position; there is no
confidentiality boundary between their users. Simulation is for
reproducible exploration, not hiding test decisions from hostile code.

**`rng() -> Pcg64`** (§6, deferred) would break neither replay nor
accounting.

## 8. Trust diff and accounting

**[repo evidence]** tide-admit already maps `tide_std::rand::Rand` to the
`rand` kind (`pure.rs` `NOT_PURE`; `synthesis/contracts.rs`), so any item
whose parameters, fields or captures reach a `Rand` declares `rand` when
structurally visible, otherwise `opaque(..)` (a generic or `dyn` erasure,
03 §4.1); opaque is not an authority-free verdict. v0
reports declared authority only (03 §4.1).

**[inference]** No new kind and no attenuation. A `rand` line in the
trust diff (03 §4.3) means the item can draw values (unpredictable ones
from `getrandom`; under simulation, the seed's, which are predictable) and
key maps with
`from_rand` (on real runs, Rust's inherited keys, §5). It reaches nothing else,
so `rand` is listed with the ordinary kinds, not first. Stage 2 adds a
fixed-public-items guard for `tide_std::rand`, as NET-HTTP-4 and PROC-3
have, so a later std edit cannot add an unreviewed path to the source;
the sealed traits and the five methods are its list. Generated contract
tests that take a `Rand` parameter already get `sim.rand` from a fresh
simulator; they can now draw.

## 9. Alternatives rejected

- **No draws; use the rand crate.** v0 has no crate imports (v1), and the
  ambient forms are `TD0501`. Programs would read `/dev/urandom` through
  an `Fs`, which declares file authority for randomness and has no
  simulated source.
- **Only `fill_bytes`** (or only `next_u64`). Every program then reduces
  ranges itself, and the obvious code (`x % n`) is biased; shuffles are
  hand-written, and the classic slip (`j` in `0..n` rather than `0..=i`)
  gives a biased order that no test notices. These are the two draws
  agents most often get wrong, so std owns them.
- **Float ranges** (`random_range(0.0..1.0)`). Several non-equivalent
  methods exist (rand has changed its own), and `random_bool(p)` covers
  the probability case. Later, with a caller.
- **128-bit ranges.** Two words and wider arithmetic for no current
  caller; ids use `fill_bytes` or two `next_u64`.
- **`try_` variants returning `Result`.** No supported system fails (§3).
- **`GRND_INSECURE`**, which never blocks: it may return output before the
  pool is initialized. Flags 0 is the OS's default and what keys need.
- **An attenuation** (`Rand::only`, a budget). Nothing to narrow: a grant
  reaches one source and no resource.
- **A `random_ratio(n, d)` or `choose`**: `random_range(0..d) < n` and
  `slice[rand.random_range(0..slice.len())]` say them in one line.

## 10. Admission filter

**[inference]** GOVERNANCE.md's questions:

- *Safer change?* Yes. Randomness becomes declared `rand` authority with
  a seeded, replayable simulated source, where today a program has none
  or reaches for a file. The two error-prone draws (unbiased ranges and
  shuffles) are std's, so agents do not hand-roll them.
- *Overlapping ways?* One way per operation; `Pcg64` is deferred
  precisely to avoid a second (§6). A coin flip can be `random_bool(0.5)`
  or `random_range(0..2) == 0`; both are honest and that overlap is
  Rust's too. `next_u64()` overlaps `random_range(0..=u64::MAX)` and an
  8-byte `fill_bytes`; it stays because it is the word primitive the other
  methods are defined over and `rand_core`'s spelling for a `u64` draw.
- *Local reasoning?* Preserved: a function draws only if its signature
  reaches a `Rand`.
- *Fast feedback?* Simulated draws take no virtual time.
- *One obvious shape?* `rand.random_range(a..b)` and the rest, today's
  `rand` names on a capability.
- *Library instead of core?* Std only; no language change.

**Verdict per operation:**

| Operation | Verdict | Soundness |
| --- | --- | --- |
| `next_u64` | admit | sound: one word from the OS or the seed's stream |
| `fill_bytes` | admit | sound: OS bytes, or words little-endian; the crypto consumer's operation |
| `random_range` | admit | sound: exact rejection method (§2.1), empty range panics before drawing |
| `random_bool` | admit | sound: exact threshold, `p` checked before drawing, NaN panics |
| `shuffle` | admit | sound: Fisher-Yates over `random_range`; cheap (§11) |
| `rng() -> Pcg64` | defer (stays planned) | sound in principle (§6); not admitted for surface and stability reasons |

`RandomState::from_rand` under simulation: lift the rejection (§5).

**Worth its cost?** Yes: it closes v0's one missing nondeterminism
source, which every retry loop, sampler and test-data generator needs,
for about 150 lines over machinery that exists.

## 11. Cost

**[inference]** Implementation: about 80 lines in a new `rand.rs`
(the five methods, the sealed traits, the word source split by backend),
about 25 in the scheduler (the `rand` stream and event), five in
`collections.rs`, one rustix feature, and the guard test. No new crate.
Readers and agents: this stage renames two methods in reference §6.1's
row, points it to companion 22, and replaces the `Pcg64` bullet with the
draws' two panic rules and their sources: 24,999 to 24,960 tokens by
`scripts/count-tokens.sh`, inside D14's 25,000. The draws stay labelled
planned in the reference, 03, 01 and companion 22 until stage 2 builds
them. Stage 2 drops those labels and the reference's two `partial` notes
on `Rand` (§6.4, §6.8), which frees tokens, and keeps `rng() -> Pcg64`
planned. The rules are in companion 22 and 03 RAND-DRAW.

## 12. Stage 2

Stage 2 implements §§2–5 and §8; drops the planned labels of the five
draws from reference §6.1, 03 (RAND-DRAW, §9.3), 01 and companion 22 and
14.2, the reference's two `partial` notes on `Rand`, and 03 §9.2's
modeled list gains random draws; recounts the reference on its rebased
candidate; and drops the `partial`/"no draw methods" notes from `lib.rs`
(the crate doc, `pub mod rand`, the "Not provided yet" list),
`collections.rs` and `sim.rs`'s
`simulation_identities_never_use_host_files_or_randomness` test (which
asserts the panic), and adds cases for `conformance/rules.md`'s RAND-1 to
RAND-4. Their done condition is the bead's Acceptance:

- **RAND-1, simulated replay.** A `#[sim_test]` draws with every method
  (and `from_rand`) from two tasks. Rerunning one seed gives identical
  draws, trace and map order; replaying its manifest is byte-identical;
  seeds 0 and 1 give different first `next_u64` words. A run that never
  draws makes today's draws in every other stream.
- **RAND-2, edge cases, both backends.** The value and panic clauses
  are one suite with shared assertions. The word-count and
  before-drawing clauses ("takes no word", "before drawing") are checked
  on the simulated backend through the `rand` event's word count, and on
  the real backend through RAND-4's seam.
  `random_range` over an empty `a..b` or `a..=b` panics, a one-value range
  returns it, `i64::MIN..=i64::MAX` and `0..=u64::MAX` return values;
  `random_bool` with `-0.1`, `1.1` and NaN panics, `0.0` is false and
  `1.0` true; `shuffle` of an empty or one-element slice leaves it and
  takes no word, a longer one is a permutation; `fill_bytes` of an empty
  slice takes no word.
- **RAND-3, simulated values.** For a fixed seed, words equal SplitMix64
  from `seed ^ "tiderand"`; `fill_bytes(&mut [0; 11])` is two words
  little-endian truncated; `random_range(0..=1u64 << 63)` rejects words
  below its threshold (there `s = 2^63 + 1` and `t = 2^63 - 1`, so about
  half of first words are rejected; a seed that hits one); `shuffle` of
  `[0, 1, 2, 3]` over
  seeds 0..1000 gives every one of the 24 orders.
- **RAND-4, real source.** A `#[cfg(test)]` seam in place of the rustix
  call supplies fixed bytes: `next_u64` returns them little-endian,
  `fill_bytes` copies them, a short count then `EINTR` is retried until
  full, and another error panics naming it. Real draws come only from
  `getrandom(2)`: a real draw in a child under a seccomp filter that
  returns `EPERM` for `getrandom` (RUN-3's sandbox machinery or a
  test-only filter) panics naming `EPERM` and opens no `/dev/urandom`. If
  stage 2 finds that filter infeasible, it records why and falls back to a
  named source check: the single `rustix::rand::getrandom` call site and
  the seam's position. Two real `next_u64` draws differ. A
  `tide_std::rand` public items guard fails on additions.
