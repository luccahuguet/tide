# Tide

**A capability-safe, replayable subset of Rust.** Capability-safe the way
Rust is memory-safe: up to the compiler's soundness bugs (item 8 below).

<!-- at-a-glance: docs/at-a-glance.md, copied verbatim; the site's generator checks the copy -->
## Tide at a glance

Tide is a subset of Rust, a systems programming language, for code that
artificial intelligence (AI) agents write and people have to trust. Code in
it reaches the outside world only through capabilities, values a function is
handed, and its tests replay a failing run exactly.

1. **It's Rust.** Every Tide program is a valid Rust 2024 crate (a Rust
   package written to the language's 2024 edition), compiled by rustc, the
   standard Rust compiler, unmodified and pinned to one nightly version, a
   dated build. Tide keeps Rust's syntax and compiler, with no fork.
2. **A checker runs first.** `tide-admit`, Tide's admission checker, checks
   each crate before rustc compiles it and admits or rejects it. Every
   rejection by one of its rules carries a numbered diagnostic, a `TD` code
   such as `TD0501`, whose message says why. Code that rustc itself
   rejects, such as a call to a function Tide's std lacks, gets rustc's own
   errors.
3. **Authority shows in the signature.** Files, the clock, the environment
   (the variables and arguments a program starts with), the network, other
   programs and randomness are reachable only through capability values a
   function is handed: `&Fs`, `&Clock`, `&Env`, `&Net`, `&Proc` and `&Rand`. Tide's standard
   library, the library every program builds on, has no free function, one
   called without such a value, that reaches them, printing aside, so a
   function's parameters bound what it can do, up to rustc's soundness
   bugs (item 8).
4. **Authority can be narrowed.** `fs.dir(path, mode)` confines a file
   capability to one directory, and `only([...])` keeps a network capability
   to listed hosts (named machines), a process capability to listed programs
   and an environment capability to listed variables. A network `only`
   checks the machines a program connects to, and accepting incoming
   connections stays possible until `no_bind()` removes it. An allowed
   program runs with its own full authority, so allowing a shell, a program
   that runs other programs, allows anything.
5. **Tests replay.** A `#[sim_test]` runs under a simulator of tasks (Tide's
   units of concurrent work), files, the network, the clock and the
   environment, where a seed, one number, decides every scheduling choice.
   `tide-admit` records a failing seed's run and replays it on the same
   machine with identical event and output traces, the logs of what the run
   did.
6. **Any change can get a trust diff.** `tide-admit --tide-diff` summarizes
   what a change may do: the public functions and types whose authority
   widened or narrowed, and the `unsafe` sites, code the Rust compiler
   cannot check, added or removed. Our E3 pilot, a small trial with AI
   reviewers, tested it as a review aid and found no evidence that it helps,
   and we published that result.
7. **Built for agents.** The core reference, the document that teaches the
   language to an AI model, fits a budget of
   [25,000 tokens](/design/01-design.md#d14-toolchain-contract-for-agents),
   the units a model reads text in.
8. **Known limits.** This first release, an alpha (an early test version),
   runs on Linux on 64-bit x86 processors (x86_64). Async code, functions
   that pause and resume later, is left out by design. Threads are parts of
   a program that run at the same time. Safe code, everything outside
   `unsafe`, sees their memory writes in one global order, and weaker
   orderings, which let threads see each other's writes out of order, are
   planned only behind the `unsafe` grant, an explicit opt-in passed to
   `tide-admit`. Replay reproduces the simulator's model of a program, not
   the machine. Admission closes
   [three known routes](/README.md#forge-routes) through rustc's soundness
   bugs, compiler bugs that let safe code break Rust's guarantees, and the
   others stay open.
9. **Evidence, negatives included.** Our pilot and our searches for real bugs
   recorded their methods in advance and published every count. How far
   that order is proven varies by round, and a possible security defect is
   held privately and appears publicly only as a count. Our screen of all 87
   malicious-crate advisories in RustSec, Rust's public database of security
   advisories (a malicious crate is a package published to attack its
   users), caught none of them. Tide's std had a gap, the screen exposed it,
   we fixed it, and re-tested under a new registration, a method recorded in
   advance ([research/49](/research/49-fx-zu4f.15-chrono-anchor-retest.md)).
   That re-test's pool, one crate the screen had already seen, was chosen
   afterwards, so its outcome was close to predictable: it is not a
   first-time catch and not a rate. It counts one catch: our port (a rewrite
   in Tide) of that crate is rejected by admission, as it was in the screen,
   and a version handed the authority it uses is now admitted. Admission
   does not judge intent. It makes the authority visible. Every check is
   consistent with the re-test's registration being anchored, by an
   OpenTimestamps proof (a timestamp in a public blockchain), before the fix
   landed, and independent proof of that order is not established. The
   re-test ran admission and type-checking only. No port was executed. For
   the pilot and four of the searches, the order of method and results
   rests on our own records (self-attested). The other searches' method
   files carry OpenTimestamps proofs, which show only that each file existed
   by its attestation time, and when those searches examined their
   candidates rests on our records. The [status page](https://tidelang.com/status/)
   lists every round with its counts and its qualifier.
<!-- /at-a-glance -->

In code, from [examples/capabilities/](examples/capabilities/README.md): the
Rust habit is rejected, and the Tide version takes the file capability it
uses.

```rust
// Rejected: error: TD0501: `std::fs::read_to_string` is not admitted
pub fn load(path: &str) -> Result<Config, ConfigError> {
    parse(&std::fs::read_to_string(path)?)
}

// Admitted: the file capability is a parameter
pub fn load(fs: &Fs, path: &str) -> Result<Config, ConfigError> {
    parse(&fs.read_to_string(path)?)
}
```

**Tide v0 alpha, x86_64 Linux only.** A Tide
program is an ordinary Rust crate that a pinned, unmodified nightly rustc
compiles against Tide's std, after Tide's admission driver, `tide-admit`, has
checked it. Tide complements Rust: it keeps Rust's syntax, ownership and
compiler, and narrows what a program may do so that its authority shows in
its signatures and its tests replay. The public home is
[tidelang.com](https://tidelang.com).

The alpha leads with replay and admission:

- **Tests replay.** A test that takes a simulated root (`#[sim_test]`,
  `Root::sim(seed)`) runs under a seeded simulator of tasks, channels,
  `Shared`, files, TCP loopback, HTTP peers (`world.http`), declared child
  processes (`sim.world.proc`), the clock, the environment and stdin. A run
  through `tide-admit --tide-run-test` records a manifest, and `--replay`
  reruns it on the same machine and checks that the event and output traces
  are identical, so a concurrency bug in Tide-written logic becomes an
  ordinary failing test. The [demo](demo/README.md) finds a lost update in a
  small concurrent service on 8 of 16 seeds and replays the first failing
  seed twice with identical traces. Replay reproduces the simulator's model of
  the program, not everything the program does on a real machine
  ([Simulator fidelity](#simulator-fidelity)).
- <a id="forge-routes"></a>**Capabilities that admitted safe code cannot forge by any route tried
  so far.** Tide's std has no free function that reads files, the clock, the
  environment, or the network: code reaches the outside world, printing
  aside, only through capability values it is handed (`&Fs`, `&Net`,
  `&Clock`, ...), so a function's signature bounds what it can do. Safe code
  could still forge a capability through a rustc soundness bug. The driver
  closes three such forge routes through two open rustc issues:
  the `fn`-pointer route of
  [rust-lang/rust#25860](https://github.com/rust-lang/rust/issues/25860)
  (`TD0206`) and two forms of the trait-object route of
  [rust-lang/rust#57893](https://github.com/rust-lang/rust/issues/57893), an
  impl for a type parameter and one for a projection (`TD0402`). Both issues
  were public upstream long before Tide (#25860 since 2015, #57893 since
  2019, both forms on record there); neither is a Tide discovery. A red team of
  admission and the test sandbox, run by Claude Opus (an AI red team, not a
  human audit;
  [reviews/2026-11-17-red-team-opus.md](reviews/2026-11-17-red-team-opus.md)),
  tried the escape routes its record numbers 1 to 45, plus 6b, the
  projection form of route 6 (48 with its two controls): the two #57893
  forms were admitted and are rejected after its fixes, and every other
  route is rejected or denied, except two left admitted on purpose
  ([Known holes](#known-holes)). rustc's other open soundness bugs stay
  open.

Each crate admitted against Tide's std also gets a trust artifact, and
`tide-admit --tide-diff` summarizes what a change may do: the exported items
whose declared authority widened or narrowed, and the `unsafe` sites added or
removed. It shows declared authority only, not what code exercises. The E3
pilot below tested it as a review aid and gives no evidence that it helps.

## Evidence

No experiment yet shows that Tide makes reviewers or models better. What v0
offers is a property and a mechanism. The property: no accidental ambient
authority in admitted safe code, apart from printing, outside the `unsafe`
grant (`--tide-grant=unsafe`) and up to rustc's soundness bugs. The
mechanism: seeded replay. The sections below are the tests we ran,
negatives included.

### A malicious crate: rejected as written, admitted only with its authority declared

[research/49](research/49-fx-zu4f.15-chrono-anchor-retest.md) records a
catch on a separate re-test, not in the screen it re-tests. Our
pre-registered screen of all 87 malicious-crate advisories in RustSec,
Rust's public security-advisory database, caught 0 of 87 (tracker issue
fx-zu4f.8; IDs of this form name issues in the project's tracker;
[research/47](research/47-malware-rescreen.md)). On the later re-test,
RUSTSEC-2026-0039, the advisory for `chrono_anchor`, a crate that tried to
send a project's `.env` files to a server impersonating `timeapi.io`, meets
the screen's unchanged caught rule. A port is the crate's code rewritten in
Tide. The rule has two legs:

- **Leg 1: the faithful port is rejected**, as it already was in the
  screen. A port of the crate's malicious code draws `TD0602` twice, on two `thread::spawn` calls (a Rust thread in
  place of a task scope), and `TD0501` three times, on two ambient
  `Command::new` calls and an ambient `thread::sleep`.
- **Leg 2: the capability-passing port is now admitted.** A port that
  receives the authority it uses as parameters is admitted with no
  diagnostic, and its trust artifact declares task authority (`bg`, the
  task scope it is handed), clock and process authority. This leg is what
  changed. It is the success leg because admission does not block the
  behaviour; it makes it visible: a time crate that starts processes has to
  take a `&Proc` to do so.

How it came about: Tide's std had a gap, the fx-zu4f.8 screen exposed it, we
fixed it, and re-tested under a new registration. In the screen, the second
leg failed because the documented idiom `clock.sleep(d)?` did not compile in
a task body. A std fix (issue fx-qlek.18) added the missing conversion, and
a re-test (issue fx-zu4f.15) checked the row at the fixed revision with the
same port files, byte for byte. The fix reverses an earlier removal of the
same conversion: a review (issue fx-nkv.3) had removed it on purpose,
because it breaks type inference for an unannotated `tasks.scope(..)?`
(rustc's `E0283`). The owner's decision of 2026-11-19 reversed that
removal so that the documented idiom compiles in a task body; the cost is
that 17 scope sites now name their error type. Every check is
consistent with the re-test's registration being anchored, by an
OpenTimestamps proof in a Bitcoin block, before the fix landed, and
independent proof of that order is not established (research/49 §3).

What it does not show:

- **Not a first-time catch, and not a rate.** The one-row pool was chosen
  after the screen had seen its rows, the outcome was close to predictable,
  and the screen's result stays 0 caught of 87.
- **Admission and type-checking only.** No port was executed.
- **No judgment of intent.** The admitted port still starts the same
  processes with the same arguments; what changes is that its authority is
  declared. The port was written from a public quote of code already known
  to be malicious.

Admission also rejects package build scripts (`TD0915`) and user macros
(`TD0101`) outright, and several of the advisories place their payload in
a build script ([research/47](research/47-malware-rescreen.md)).

The other external search rounds, issues fx-zu4f.5 to .7 and .9 to .12,
caught nothing; each is a documented negative, except that fx-zu4f.10 keeps
one unresolved row whose required evidence, an upstream pull-request thread,
was unavailable (B23.128,
[research/44](research/44-fx-zu4f.10-concurrency-round.md)). fx-zu4f.13, a
re-screen of four earlier rows once processes and HTTP landed, counts this
same catch once. The
[status page](https://tidelang.com/status/) lists every round with its
counts and its qualifier.

### E3: does the trust diff help reviewers? (pre-registered pilot, negative)

The E3 pilot ran under a protocol frozen before the run
([trials/e3-pilot/report.md](trials/e3-pilot/report.md)). Its first two
paragraphs, as published:

> In the E3 pilot, Opus and Sol reviewed 12 planted authority-widening
> changes repeatedly, 6 times per change in each arm. With the trust diff, 66
> of 72 review sessions caught the planted widening; with the plain diff, 67
> of 72 (ratio 0.99, 95% interval [0.95, 1.00]). This does not meet D0's 1.5x
> threshold on the point estimate. The interval does not rule out no benefit.
>
> The plain arm caught more than 2/3 of its plant sessions (67 of 72), so,
> as `protocol.md` §10.5 states, the gate could not be met whatever the
> trust arm did: 1.5 × 2S/3 = S, and no arm can catch more than S. The
> interval is narrow because 11 of the 12 pairs tie at 6 of 6 in both arms
> (see Pairs); its upper end of 1.00 reflects that ceiling, not evidence
> about a benefit on harder changes.

In plain terms: no benefit was seen, and with plain review this strong, the
setup could not have shown one.

The reviewers were models only (`claude-opus-5-5` and `gpt-6.1-sol`), on
small single-file changes. The pilot did not test changes too large to read
in full or human reviewers.

### E1: the learning cost

The E1 gate ([trials/e1-checked/gate-report.md](trials/e1-checked/gate-report.md))
gave one writer model, `claude-opus-5-5`, the same 30 tasks twice in Tide
and twice in Rust, 60 trials per language. Tide's reference, which the
writer learns from, was itself written by AI agents, Claude among them.
The gate shows parity within the protocol's margins, not a benefit: in the
report's words, "Tide stayed within the protocol's margins rather than
beating Rust".

- **First check.** Tide passed its first check in 56 of 60 trials and Rust in
  59 of 60, a ratio of 0.95 against a floor of 0.8.
- **Final pass.** Both passed 58 of 60, a ratio of 1.
- **Iterations.** Passing Tide trials took 86 check or test runs and passing
  Rust trials 59 (58 trials each), a ratio of 1.46 against a limit of
  1.5; the information-only 95% interval, 1.29 to 1.64, crosses 1.5.
- **Token cost.** Tide sessions read about 9 times Rust's cached input
  tokens (5,156,350 against 569,596, an informational observation in the
  report).

### Benchmarks against plain Rust

[research/36-benchmarks.md](research/36-benchmarks.md) measures four small
programs, each written in Tide and in plain Rust, on one machine; it is not
D0's E4/E5 gate run. In its words:

> **Stability: almost no series met the tool's criterion.** All 60 series
> of the main run ran to the 30-round cap with a CV above 1% (from 1.04% to
> 46.7%), so the run marks every one unstable (§1.6). In the test-build run,
> one of four series met it (`tide-debug-test`, CV 0.81%); the other three
> did not (3.2% to 5.9%). The medians and ratios below are descriptive
> observations from these unstable runs, not settled measurements.

and "Every comparison above is a loss for Tide or a tie. None is a win."
Among its observations:

- a cold debug build takes 3.8-4.6x plain Rust's, and a rebuild after a
  one-line edit 5.2-6.3x incremental Rust's, since the driver has no
  incremental build;
- in release, with contracts erased, Tide's `contracts` binary ran in
  0.901 s against 0.906 s for plain Rust built with Tide's profile flags, a
  tie (Tide's arm slower in 11 of 30 rounds);
- in debug, Tide's `contracts` binary runs in 1.05x the time of plain
  Rust's debug binary at 1,200,000, a tie (slower in 16 of 30 rounds), and
  1.19x at 200,000 (in a run outside a quiet window), slower in all 30
  rounds, a loss;
- in release, against plain Rust built with Tide's profile flags, capability
  file I/O (`files`) is a small loss, slower in 26 of 30 rounds (paired
  median 1.07x), and capability TCP (`tcp`) takes 1.18x as long, slower in
  all 30 rounds;
- the `tasks` program runs 2.29x plain Rust's release build: 1.74x from
  Tide's profile flags, then 1.31x between Tide's arm and its twin with the
  same flags.

Checking an edit in tens of milliseconds is a **target** of Tide's
incremental front end (D12), which is not built; today each check is a rustc
run plus admission.

### Case studies

[examples/case-studies/](examples/case-studies/) rewrites real projects' bugs
in Tide. Admission catches none of these three bugs.

- [eon-discovery-race](examples/case-studies/eon-discovery-race/README.md):
  Tide's admission would not have rejected Eon's bug, a discovery that fails
  when a host exits between its record read and its socket check. A
  `#[sim_test]` running discovery against the exit fails on 13 of 100 seeds
  and passes on the fix; the simulator reruns each failing seed and checks
  that its trace matches. Eon's fix landed with a deterministic test; what
  Tide adds is that the seed finds the interleaving, with no hook written into
  discovery to place it.
- [eon-slow-close](examples/case-studies/eon-slow-close/README.md): Tide's
  admission would not have rejected Eon's bug, a request loop blocked behind a
  slow close. A `#[sim_test]` catches it: it fails on the buggy loop and
  passes on the fix. Rust can write the same test; what Tide adds is that it
  runs in virtual time, deterministically, with no fake clock written for it.
- [hyper-ambient-runtime](examples/case-studies/hyper-ambient-runtime/README.md):
  no Tide admission code catches either of hyper's reported failures. What
  Tide gives is the fix's shape, a pool that spawns only in a scope and reads
  time only from a `Clock` it is handed, and a `#[sim_test]` shows it closing
  idle connections on 20 seeds.

[examples/](examples/) holds small Tide programs, each with a README and a
declared outcome (it runs with given output, its tests pass, a test fails,
or admission rejects it with a named code), which `cargo test --workspace`
checks. `examples/capabilities/ambient_read.rs`, for instance, is rejected
with `TD0501` for reaching for the file system without an `Fs`.

## Try it

Each path runs the [demo](demo/README.md): a seeded simulation finds a lost
update in a small concurrent service and replays it twice with identical
traces, the fixed revision passes the same seeds, and the trust diff shows a
patch widening the service's interface. The alpha runs on x86_64 Linux; the
browser and Docker paths provide one.

1. **In the browser, with nothing installed.**
   [![Open in GitHub Codespaces](https://github.com/codespaces/badge.svg)](https://codespaces.new/luccahuguet/tide)
   The codespace builds the repository's
   [dev container](.devcontainer/devcontainer.json), the `Dockerfile`'s dev
   image. In its terminal, run `nix run .#tide-demo`. The image has no gdb,
   so step 5, the replay under a debugger, prints `skipped: no gdb`; the
   rustup path, on a machine with a gdb built with Python on `PATH`, runs it
   ([demo/README.md](demo/README.md)).
2. **With rustup and no Nix.** On x86_64 Linux with rustup 1.28 or later,
   git, and a C toolchain at `/usr/bin/cc` and `/usr/bin/ld`, which Cargo and
   the driver link with (on Debian or Ubuntu, `build-essential`):

   ```sh
   git clone https://github.com/luccahuguet/tide && cd tide
   # The nightly rust-toolchain.toml pins, with rustc-dev and rust-src. A rustup
   # older than 1.28 installs it at the first cargo run, so skip this line there.
   rustup toolchain install
   cargo run -p tide-demo
   ```

3. **With Nix and no clone:** `nix run github:luccahuguet/tide#tide-demo`
   (add `--extra-experimental-features 'nix-command flakes'` unless both are
   enabled). It needs no rustup and no system C compiler. The demo writes
   under its own cache directory, or `$CARGO_TARGET_DIR` when set
   ([docs/nix.md](docs/nix.md#demo) says where and gives the exceptions).
4. **With Docker, on macOS or Windows:** in a clone,
   `docker build --platform linux/amd64 --target try -t tide-try .`, then
   `docker run --rm --platform linux/amd64 tide-try`
   ([docs/docker.md](docs/docker.md)). The image holds no `.git`, so the
   demo's step 0 prints `revision:  unknown`.

Measured on 2026-11-19 on an x86_64 Linux host with 16 CPUs, over its own
connection, whose bandwidth was not measured;
[reviews/fx-qlek.7-s3-measurements.md](reviews/fx-qlek.7-s3-measurements.md)
gives the conditions and the log lines behind each number. Nix builds were
capped at 4 cores and 4 jobs. The rustup, `nix run` and Docker runs used 4
Cargo jobs, and their containers 4 CPUs; the devcontainer's runtime had no
CPU or Cargo-job cap. The rustup row used rustup's minimal profile; the
default profile also installs rust-docs, clippy and rustfmt, which the row
does not count. The owner measured the Codespaces row's codespace times
under the conditions the row gives; of them, only the time to a ready
codespace is approximate, the owner's estimate. Its devcontainer CLI times
are from the host above. Download times depend on
the connection, and no download volume was measured, so the table gives
disk and image sizes:

| Path | Size | Measured time |
| --- | --- | --- |
| Codespaces | nothing on your machine; the codespace builds the 6.09 GB dev image, and the demo's build output, 1.3 GiB under `nix run`, goes to the demo's cache directory in the codespace | in a codespace from the dev container configuration "Tide", on GitHub's default 2-core machine in US East, at revision `d48b1aa`: about 10 minutes (an estimate) from Create to a ready codespace, then `nix run .#tide-demo` timed at 2m13.715s real (about 2 min 14 s), measured by the owner; on the host above, the devcontainer CLI built the container from a clean clone in 6 minutes, then `nix run .#tide-demo` took 49 s |
| rustup | on disk afterwards: about 1.4 GiB in `~/.rustup` (the pinned nightly with `rustc-dev` and `rust-src`), 83 MiB in `~/.cargo`, and 1.3 GiB of build output in `target/` | 19 s to install the toolchain, then 51 s for the first `cargo run` (16 s once built) |
| `nix run` | a 2.1 GiB closure in the Nix store, plus 1.3 GiB of build output in the demo's cache | 48 s with the closure already in the store; fetching and building it in an empty store took 80 s |
| Docker | the try image, 2.42 GB on disk, 769 MB as a gzip-compressed `docker save`; built from a clone, with its build stages left in Docker's build cache ([docs/docker.md](docs/docker.md#measured-times)); the run's build output goes with the container | about 3 minutes to build the image, then 36 s to run |

### Install the driver

On x86_64 Linux, [docs/install.md](docs/install.md) installs the driver
through the repository's Nix flake, with no rustup and no system C compiler,
and explains each step. It needs `curl`, `xz` and `git`, about 10 GB of
free disk space, and bash or zsh; the Nix installer asks for `sudo` only to
create `/nix`. These are its commands, which
`scripts/clean-install-check.sh` runs in a clean container:

```sh
sh <(curl --proto '=https' --tlsv1.2 -L https://nixos.org/nix/install) --no-daemon
. ~/.nix-profile/etc/profile.d/nix.sh
git clone https://github.com/luccahuguet/tide tide
cd tide
nix --extra-experimental-features 'nix-command flakes' build .#tidelang-admit
result/bin/tide-admit --crate-type=bin --emit=link --crate-name=sorted_insert \
  --out-dir target/quickstart examples/contracts/sorted_insert.rs
target/quickstart/sorted_insert
result/bin/tide-admit --test --emit=link --crate-name=sorted_insert \
  --out-dir target/quickstart-tests examples/contracts/sorted_insert.rs \
  --tide-run-test __tide_contract_tests::insert_sorted_3
```

The example prints `[3, 7, 9, 14, 20]`, and its test, generated from the
function's contract, passes in the kernel sandbox. On macOS or Windows,
[docs/docker.md](docs/docker.md)'s dev image runs the same route in a Linux
container, a route checked only on a Linux host so far.

[docs/debugging.md](docs/debugging.md) covers replays under a debugger, and
the [owner's guide](docs/owner-guide/README.md) walks through each feature
with runnable commands.

## Status

Tide v0 alpha, x86_64 Linux only. The alpha includes:

- **The admission driver**, `tide-admit`: a rustc driver that admits a single
  crate against Tide's std. It enforces the admission allowlist, the
  authority checks (no ambient I/O, `Root` only where allowed, simulation
  only in tests), the trait power ceiling, the `unsafe` grant, the
  concurrency and error rules, and rejects user macros, build scripts and
  file-loading attributes, each with a named `TD` code. It builds binaries
  and test harnesses, writes the trust artifact, compares two artifacts
  (`--tide-diff`), and runs one test in the sandbox (`--tide-run-test`) or
  replays a recording (`--replay`).
- **Tide's std**: capabilities (`Fs`, `Net` for TCP, one HTTP/1.1 `get` and
  `resolve`, `Clock`, `Proc` for child processes (`command`, `only`),
  `Rand` draws, environment, stdin, output and log sinks), structured tasks
  with cancellation and deadlines, `Shared`, channels, `join`/`par_map`/`par_for`, executable
  contracts, property tests, `#[sim_test]` and the simulator, JSON encoding,
  and deterministic float math.
- **Runnable evidence**: the demo, the examples, the case studies, four
  benchmark pairs, and the conformance corpus
  ([conformance/](conformance/README.md)): cases with expected verdicts per
  rule ID, which stable rustc and the driver check.

[docs/driver.md](docs/driver.md) states in detail what the driver checks,
its interface, and what each std module provides and leaves out.

### Not in this alpha

- **Packages and dependencies** (`tide.toml`, `tide.lock`, per-dependency
  grants; fx-x2oo): the driver admits one crate with one `.rs` entry against
  Tide's std, and rejects `extern crate`.
- **The `tide` command** (`tide check`, `tide test`, `tide sim`,
  `tide review`; fx-a9oh): `tide-admit` is the whole toolchain.
- **Simulator faults and crashes** (fault injection, crashable nodes;
  fx-djwo).
- **UDP, `Rand::rng() -> Pcg64` and `Proc::exit`**: `Rand` has its five
  draws (`next_u64`, `fill_bytes`, `random_range`, `random_bool`,
  `shuffle`) but no generator yet; `Proc` starts processes (`command`,
  `only`) but has no `exit` yet, and signals and PTYs are Later.
- **The incremental front end** (D12), contracts in the trust diff, and the
  sandbox level in `tide check` and `tide review`.

Planned v1 work includes SMT proofs of contracts, exercised-authority
tracking, FFI, Rust crate import, and production record and replay
([v0 Core and Later Tiers](#v0-core-and-later-tiers)).

### Simulator fidelity

| The simulator models | It does not model |
| --- | --- |
| Scheduling of structured tasks, `Shared` and channels, one task at a time between scheduling points, chosen by the seed | CPU time: virtual time advances only when every task is blocked, so a slow computation never fires a deadline |
| Virtual time for the clock, sleeps and deadlines | Memory exhaustion, stack depth, and parallel speed-up (`par_*` work runs in order on the calling task) |
| Files (`sim.fs`), each call with a seed-drawn latency | Disk faults, crashes, links, and partial effects within a compound file operation |
| TCP loopback on `127.0.0.1`, with bounded streams and half-close | Other hosts, DNS names, UDP, and real kernel behaviour such as RST timing |
| `Net::get` against `world.http` peers: replies, statuses, delays, resets, TLS failures; `Net::resolve` (synthetic `198.18.0.n` addresses for `world.http` domains) | TLS certificate validation and HTTP wire parsing (a peer models the application response) |
| Arguments, environment variables, stdin, and sink output; child processes as declared `sim.world.proc` models (lifecycle, pipes, `kill`); `Rand` draws from the seed | Signals, real child processes (only declared models run), and fault injection |

The real and simulated backends are separate implementations, so a bug that
lives only in the real backend or in the kernel is invisible to the
simulator. Replay reproduces the model, not the program, and only on the same
machine with identical pinned artifacts.

### Sandbox

`--tide-run-test` and `--replay` start the test binary under Landlock and
seccomp allowlists (RUN-3) and print the level before it starts:
`sandbox: full`, or `sandbox: partial, not enforced: …` naming what the
kernel lacks, in which case the run continues under what remains.

- **Observed `full` on Linux only**: on an x86_64 host (Linux 7.2.6, Landlock
  ABI 10) and in the clean container of the install check (fx-qlek.1,
  [docs/install.md](docs/install.md)), and in the Docker dev image under Docker's
  default seccomp profile on that host (fx-qlek.2,
  [docs/docker.md](docs/docker.md)). No macOS or Windows run has been
  observed; there the level depends on the kernel of Docker's Linux VM.
- **Builds are not sandboxed** (RUN-2; fx-wktt, deferred): the compiler,
  linker and toolchain run as the user.
- A test binary run directly, outside `--tide-run-test` and `--replay`, gets
  no sandbox.

### Known holes

- **rustc soundness.** Admission rests on rustc's soundness. The driver
  closes the three forge routes above; rustc's other open soundness bugs stay
  open, and safe code that reaches one can forge a capability
  (`design/03-authority-concurrency.md`, RUN-4, hole 5).
- **Two routes admitted on purpose** (red-team routes 22 and 24):
  `Termination::report` writes an `Err` to standard error, which every
  program holds as a sink (SINK-1); and `#![crate_type = "cdylib"]` is
  admitted but inert, since the driver's `--crate-type` wins.
- **Later rustc items.** `std::path` is listed item by item, but `core::*`
  globs such as `core::mem::*` and `core::cell::*` admit whatever a later
  rustc adds under them, until the pin-bump audit lists it.
- **What the sandbox leaves open.** Landlock and seccomp do not confine
  reading a host path's metadata (whether it exists, its type, mode, owner,
  size and times); a child can run the granted loader on a granted library;
  the child's output is not escaped (RUN-6); and builds are unsandboxed
  (RUN-2).
- **`unsafe` grants** are trust holes by design: code under the grant can
  forge a capability, and the trust artifact lists every site the grant
  admits.
- **Availability and side channels.** A run can spin, allocate or write
  output until a limit the operator sets stops it, and timing is not
  confined (RUN-4, hole 6).
- **What the trust diff cannot show.** It lists declared authority of
  exported items only: a capability narrowed less than it should be, a held
  capability put to a new use, or data sent out through standard output does
  not appear in it (E3, mechanisms M4 to M6).

### Limitations

- Whether a user destructor may block or reach a cancellation checkpoint, and
  when a child's panic counts as cancelling its scope, is unspecified
  (fx-0514, deferred).
- `TD0402` fails closed: it also rejects some impls no `dyn` type can
  overlap, such as `impl<T: ?Sized + Hash> Key for T`.
- Replay checks the bytes of every library the build loaded. A plain
  `cargo build` of the whole workspace after `cargo run -p tide-demo`
  rebuilds Tide's std into different bytes, so replaying a recording made
  before it refuses with `dependency artifact mismatch`; rerun the demo to
  record again.

## How Tide is built

Tide is AI-welcome because it is checkable. It is AI-led and
human-steered: AI agents design, specify, write, review and validate its
production code and its design text, and the owner sets scope, priorities
and direction and makes the owner calls. A
change normally lands on `main` only after two reviews of a frozen
candidate by fresh agents that did not write it, one from each of two model
families: a pass by GPT Sol through OpenAI's Codex command-line tool
(the Codex pass), then a Claude Opus review, with the author fixing or
rebutting each finding
([GOVERNANCE.md](GOVERNANCE.md) states the exceptions). The checks run from
the repository:

```
cargo test --workspace          # the driver's, Tide's std's, the demo's, and the runner's tests
cargo run -p tide-conformance   # the conformance corpus, through stable rustc and the driver
cargo run -p tide-demo          # the demo: a lost update found by seed, replayed, and a widened interface
```

A whole-workspace `cargo build` after the demo rebuilds Tide's std, so
replaying the demo's recordings then refuses ([Limitations](#limitations)).

These run in the root Cargo workspace with the nightly that
`rust-toolchain.toml` pins, with `rustc-dev`; `cargo test --workspace` and
the corpus also need the `stable` toolchain and `/usr/bin/python3` with MPFR
(`libmpfr.so.6`), which the float oracle uses. The flake's dev shell
(`nix develop`, [docs/nix.md](docs/nix.md)) provides the pinned toolchain.
[CONTRIBUTING.md](CONTRIBUTING.md) says how to contribute.

## Reading Guide

[docs/tide-or-rust.md](docs/tide-or-rust.md): what Tide gives and costs, who
should use plain Rust instead, and who Tide is for.

### Design

- [design/00-first-principles.md](design/00-first-principles.md): the problem
  statement, hard constraints, measurable targets, and hypotheses H1–H7 with
  their verdicts.
- [design/01-design.md](design/01-design.md): the decision record (D0–D18)
  and the tier table; it owns every decision.
- [design/reference.md](design/reference.md): the agent-facing reference for
  the v0 core, written as a delta from Rust, within a 25,000-token budget,
  labelling what is not built yet;
  [design/reference-companion.md](design/reference-companion.md) holds its
  on-demand tables, examples, and later tiers.
- [design/02-memory.md](design/02-memory.md): ownership, origins, variance,
  drop check, `Send`, `Sync`, and `Pure`; the definition the checker
  implements.
- [design/03-authority-concurrency.md](design/03-authority-concurrency.md):
  capabilities, structured tasks, parallelism, the simulator, the Bend
  boundary, and Rust import.
- [design/04-contracts.md](design/04-contracts.md): contracts, proof modes,
  and the trust surface.
- [REFERENCES.md](REFERENCES.md): the languages, projects, papers, and data
  Tide draws on.

Chapters 02–04 elaborate draft 3.7 and label which sections are v0 core and
which are later tiers; on any conflict, `01-design.md` and its tier table
win.

### Research

Cited evidence behind the decisions, one numbered note per question, in
[research/](research/): memory models, compile speed, verification, agent
tooling, effects and capabilities, concurrency and deterministic simulation,
the Rust-dialect decision (`15-rust-dialect.md`), replay manifests
(`32-replay-manifests.md`), the case-study survey
(`33-case-study-candidates.md`), the trademark check
(`35-trademark-check.md`), and the benchmarks (`36-benchmarks.md`), among
others.

### Reviews and Trials

- [reviews/](reviews/): adversarial reviews, one file per reviewer and
  round, named `<bead>-r<N>-opus.md` or `<bead>-r<N>-codex.md`; the
  `round<N>-*.md` files are the earlier, global numbering. Round 1
  (agent usability, performance, soundness, strategy, verification, and a
  second-model review) examined draft 1;
  `round1-resolution.md` audits draft 2 against it. Round 2 (consistency,
  simplicity, and a second-model review) examined draft 2.1 and led to the
  tier split. Rounds 3–7 (consistency and second-model reviews) checked
  drafts 3 to 3.3 until no v0-blocking objection remained. Round 8 reviewed
  the switch to Rust lifetime syntax and led D2 to take Rust 2024's rules
  whole. `dialect-codex.md` is the second-model review of
  `research/15-rust-dialect.md`.
- [trials/](trials/): the learnability trials (E1) and the trust-diff pilot
  (E3). In the early E1 rounds, a fresh agent that saw only a reference
  wrote the programs, and `grading.md` grades them against the normative
  chapters.
  - `e1/`: the pilot. It estimated a 6/10 first-check pass rate against
    about 9/10 for Rust, with no leaked Rust habits; the misses trace to
    gaps in the reference and one design gap.
  - `e1-round2/`: the same tasks after the reference fixes, 9/10. The
    reference mirrors these tasks, so later rounds use held-out tasks.
  - `e1-origins/`: held-out, borrow-heavy tasks written once with `from`
    clauses and once with Rust lifetime syntax. Both arms passed 10/10 with
    no borrow errors, and Rust syntax needed about 3.5 times the
    annotations; the tie decided D2's Rust spelling.
  - `e1-dialect/`: the first round under D18, on held-out tasks covering
    outlives bounds, `use<..>`, scopes, capabilities, the simulator, and
    attribute contracts. One writer passed 11/11 with no definite errors; a
    rustc oracle against a stub std (`rustc/run.sh`) confirmed every type
    and borrow verdict. It records the writer's token cost.
  - `e1-checked/`: the E1 gate, 30 tasks in Tide and in Rust on the checked
    toolchain ([Evidence](#e1-the-learning-cost)).
  - `e3-pilot/`: the E3 pilot of the trust diff as a review aid
    ([Evidence](#e3-does-the-trust-diff-help-reviewers-pre-registered-pilot-negative)).

## v0 Core and Later Tiers

- **v0 core**, the design's first tier, which the v0 alpha implements in
  part (the largest parts not built are listed at the end of this item):
  Rust surface and ownership, with lifetimes checked as origins; traits with a power
  ceiling; `Result` and `?`; built-in `!` forms and derives; capabilities and
  `Root`; the `Pure` marker trait; structured tasks on blocking and thread
  runtimes; channels, `Shared::with`, `join`, `par_map`, and `par_for`;
  executable contracts as runtime checks and property-test drivers; the
  simulator with crashes and durability; the declared-authority trust diff;
  sealed delegation boundaries; unreviewed branches tested without a VM
  per run, in a fixed OS sandbox on Linux; per-dependency grants; the admission
  allowlist; the agent toolchain on rustc builds gated by the admission
  driver, then Tide's incremental front end (D18). The largest parts not
  built yet: the `tide` commands and the incremental front end; packages
  (`tide.toml`, `tide.lock`, per-dependency grants, authority policies)
  and sealed interfaces; UDP, `Rand::rng() -> Pcg64`, and `Proc::exit`; the
  simulator's faults, crashes, and nodes; the rest of the test sandbox and
  the unreviewed-branch guarantee (RUN-3, RUN-5); and contracts in the trust
  diff. `design/reference.md` labels each one **(planned)**.
- **v1**: `prove` mode with SMT proofs, exercised-authority tracking, the
  evented runtime, FFI, direct Rust crate import and Rust consumers of Tide
  crates,
  `par_reduce`, field-borrow declarations, and production record and replay.
- **Later**: the Bend tandem, deferred overflow traps, automatic
  parallelization, `Shape` reflection, Polonius-precision borrows, and
  interactive-proof export.

## Project Process

- [CONTRIBUTING.md](CONTRIBUTING.md): how to contribute, the bar every
  change meets, AI-written or not, and the project's roles.
- [AGENTS.md](AGENTS.md): rules for agents working here, generated from
  pinned protocol profiles plus `.agent-protocols.local.md`.
- [GOVERNANCE.md](GOVERNANCE.md): Tide is AI-led and human-steered (AI
  agents design, specify, write, review and validate; the owner sets scope
  and direction); every change to
  `main` except a generated Starcompass import update (no review) or a
  planning-only one (no Codex pass) gets a Codex supervisor pass and,
  once Codex reports it clean, an Opus review; each reviewer re-checks
  only the fixes to its own findings, and only findings in the tiers a
  change edits, or earlier ones, block it.
- [BR_WORKFLOW.md](BR_WORKFLOW.md): how `br` tracks work and which `.beads/`
  files are committed.
- Open work lives in beads (`.beads/issues.jsonl`), managed with `br`.
- `scripts/autorun.sh` runs fresh headless sessions back to back: it
  continues a handed-off bead, else works the top ready one, judges each
  outcome from `br` and `git`, logs it to `.autorun/log`, stops on no work,
  repeated handoffs, repeated failures, or its caps, and never pushes. After
  every 10th session of each run, and when it stops with no work left, it
  runs the `correct` skill in a session of its own and logs the beads it
  filed and its cost; those beads stay uncommitted in `.beads/`.
  It does not run while a coordinator has beads in flight (AGENTS.md,
  Parallel work).
  `scripts/autorun-test.sh` tests it offline with a stub agent.
- `scripts/handoff-guard.sh` is a Claude Code Stop hook for attended
  sessions: under 350k tokens of context, with a pickable bead ready, it
  blocks a stop that proposes a handoff or asks whether to go on, once.
  It runs once the owner adds its Stop entry to `.claude/settings.json`,
  which agents may not edit. `scripts/handoff-guard-test.sh` tests it
  offline.

## History

- [archive/v0-design/](archive/v0-design/): the first design, superseded by
  `design/`; its name predates the v0 core tier and is unrelated to it.
  `research/00-old-design-salvage.md` records what carried over.
- Naming: the language is named Tide (tidelang). Ferox was its working name;
  documents in `research/`, `reviews/`, `trials/`, `TASKS.md`, `tasks/`, and `archive/`
  keep that name as written, and older records spell diagnostic codes
  `FX####`, with the same digits as today's `TD####`.
  [archive/v0-design/PUBLIC_NAME_REVIEW.md](archive/v0-design/PUBLIC_NAME_REVIEW.md)
  and the `FEROX_*` and `ORBIS_*` documents beside it record earlier naming
  reviews.
- [TASKS.md](TASKS.md) and `tasks/`: tasks from the first design's bootstrap
  work.
- `spikes/d18/` holds the throwaway feasibility spike for D18, and
  `archive/v0-design/scaffold/` the early Rust scaffold from the first
  design, which does not implement Tide.

## License and Name

Tide's code is licensed under Apache-2.0 ([LICENSE](LICENSE)), and
[NOTICE](NOTICE) lists the third-party material it includes. The license
covers the code, not the name: "Tide" as the name of this language, and
"tidelang", belong to the owner, and Apache-2.0 grants no rights to them
(section 6). A fork or modified version that is distributed uses another
name, though it may say it derives from Tide. This is the owner's naming
policy, not a registered trademark claim. To contribute, see
[CONTRIBUTING.md](CONTRIBUTING.md).
