# Tide or Rust?

Tide is a subset of Rust, built to complement it, not to replace or fork
it. Every Tide program is a valid Rust 2024 crate (a Rust package), compiled
by rustc, the standard Rust compiler, unmodified and pinned to one nightly
build, against Tide's own standard library (std). Before rustc compiles a
crate, `tide-admit`, Tide's admission checker, admits or rejects it. This
page says what that subset gives you, what it costs, who should use plain
Rust instead, and who Tide is for. It describes the v0 alpha, an early test
release; the README's [Status](../README.md#status) and
[Known holes](../README.md#known-holes) are the source for every limit
below.

## What Tide gives you

- **Authority you can read in a signature.** Files, the clock, the
  environment, the network and other programs are reachable only through
  capabilities: values a function is handed, such as `&Fs`, `&Clock`,
  `&Env`, `&Net` and `&Proc`. Tide's std has no free function that reaches
  them, printing aside, so a function's parameters bound what it can do: a
  parser that takes only `&str` cannot open a socket
  ([at a glance](at-a-glance.md), item 3). The precise claim is no
  *accidental* ambient authority for admitted safe code outside the
  `unsafe` grant
  ([owner's guide, answers](owner-guide/answers.md), item 2).
- **Authority you can narrow.** `fs.dir(path, mode)` confines a file
  capability to one directory, and `only([...])` limits a network
  capability's outbound connections to listed hosts, a process capability
  to listed programs and an environment capability to listed variables.
  A network `only` does not stop listening or unlisted incoming peers:
  accepting incoming connections stays possible until `no_bind()` removes
  it. A program a process capability allows runs with its own full
  authority
  ([at a glance](at-a-glance.md), item 4).
- **Concurrency failures that replay.** A `#[sim_test]` runs under a
  simulator of tasks (Tide's units of concurrent work), channels, files,
  local TCP, declared HTTP peers and child processes, the clock and the
  environment, where a seed, one number, decides every scheduling choice.
  A race becomes an ordinary failing test, and `tide-admit` replays a
  recorded failing run on the same machine with identical traces. The
  [demo](../demo/README.md) finds a lost update on 8 of 16 seeds and
  replays it. In the
  [eon-discovery-race](../examples/case-studies/eon-discovery-race/README.md)
  case study, a rewrite of a real bug in the Eon project, a seeded test
  fails on 13 of 100 seeds and each failing seed reruns with an identical
  trace. No external search round has yet caught an external bug this way
  (one row is unresolved; [Evidence](../README.md#evidence),
  [research/44](../research/44-fx-zu4f.10-concurrency-round.md)). Replay reproduces the simulator's model of the program, not
  the machine ([Simulator fidelity](../README.md#simulator-fidelity)).
- **A checker that says why.** Every rejection by an admission rule
  carries a numbered `TD` code, such as `TD0501` for an ambient call, and a
  message, which suits an artificial intelligence (AI) agent fixing its own
  code. Code that rustc itself rejects gets rustc's own errors.
- **Built for agents to learn.** The core reference, the document that
  teaches the language to a model, fits a budget of 25,000 tokens, the
  units a model reads text in. In the
  E1 trial, one model (`claude-opus-5-5`) on 30 small tasks wrote Tide about
  as reliably as Rust: 58 of 60 final passes in each language, first check
  56 of 60 against Rust's 59, with 1.46 times Rust's check runs against a
  1.5 limit (its information-only interval, 1.29 to 1.64, crosses 1.5).
  That is parity within the trial's margins, on small tasks with
  one model, and Tide sessions used about 9 times Rust's cached input
  tokens (the Tide arm's prompt carried the reference)
  ([E1 gate report](../trials/e1-checked/gate-report.md);
  [owner's guide, chapter 9](owner-guide/09-experiments.md)).
- **What you learn stays Rust.** Tide keeps Rust's syntax, ownership and
  compiler. Leaving means replacing calls into Tide's std (capabilities,
  tasks, the simulator) and Tide's toolchain attributes (`#[tide::main]`,
  `#[sim_test]`, `#[requires]`, `#[ensures]`, `#[error]` and the like) with
  Rust's std or crates; the rest is already Rust.
  This is our reading of the design, not a measured migration.
- **Admission on real malicious code, with its record.** On a re-test,
  our port (a rewrite in Tide) of the malicious crate `chrono_anchor`'s
  code is rejected by admission, and a version handed the authority it
  uses is admitted, with its task, clock and process authority now
  declared in its parameters and its trust artifact. Admission does not
  judge intent: the admitted version still starts the same processes. The
  re-test's scope: our screen of all 87 malicious-crate advisories in
  RustSec, Rust's public database of security advisories, caught none of
  them, and it had already rejected this port
  ([research/47](../research/47-malware-rescreen.md)). Tide's std had a
  gap, the screen exposed it, we fixed it, and re-tested under a new
  registration, a method recorded in advance
  ([research/49](../research/49-fx-zu4f.15-chrono-anchor-retest.md)). That
  re-test's pool, one crate the screen had already seen, was chosen
  afterwards, so its outcome was close to predictable. Every check is consistent with the
  registration being anchored, by an OpenTimestamps proof in a public
  blockchain, before the fix landed, and independent proof of that order
  is not established. It was admission and type-checking only, with no
  port executed, and it is not a first-time catch or a rate. The fix also
  reversed an earlier, deliberate removal of the same conversion. The README's
  [Evidence](../README.md#evidence) gives the full account.
- **A trust diff, as a summary only.** `tide-admit --tide-diff`
  summarizes what a change may do: the exported items whose declared
  authority widened or narrowed, and the `unsafe` sites added or removed.
  The E3 pilot, a small trial in which AI models reviewed planted changes
  with and without it, gave no evidence that it helps reviewers
  ([E3 report](../trials/e3-pilot/report.md)). It could not have shown a
  benefit: plain review of its small changes already caught almost every
  planted one. We present the trust diff only as a summary.

Next to Rust tools: cap-std is opt-in, so the rest of a program can still
call `std::fs`; cackle checks which kinds of API each dependency package
uses, where Tide checks each function's signature; and library simulators
such as turmoil work only while all the code uses their APIs, where Tide's
checker rejects the ambient call
([owner's guide, answers](owner-guide/answers.md), items 2 and 4).

## What it costs

- **One crate, no dependencies.** `tide-admit` admits one crate against
  Tide's std and rejects `extern crate`: no crates.io. Packages and
  per-dependency grants are planned. This is the largest limit today.
- **No async.** Async code is left out by design; Tide has structured
  tasks, which cannot outlive their scope, instead. Code built on tokio or
  other async libraries does not fit.
- **No user macros or build scripts.** Admission rejects them, and
  attributes that load files.
- **x86_64 Linux only.** On macOS or Windows, [docs/docker.md](docker.md)
  runs it in a Linux container, a route checked only on a Linux host so
  far. Apple-silicon Macs run it under emulation.
- **A slower edit loop.** [research/36](../research/36-benchmarks.md)
  measured four small programs, each written in Tide and in plain Rust, on
  one machine. Its series are marked unstable, so these are descriptive
  observations, not settled measurements or a general performance result.
  A cold debug build took 3.8 to 4.6 times plain Rust's debug build, and a
  rebuild after a one-line edit 5.2 to 6.3 times incremental Rust's: the
  driver has no incremental build yet. Compiler peak memory was 17% to 39%
  higher, and release binaries loaded 1.4 to 2.0 times the bytes of plain
  Rust built with Tide's profile flags (the compiler flags of Tide's build
  profile).
- **Some run time.** From the same unstable runs, every comparison was a
  tie or a loss. Against release Rust built with Tide's profile flags, the
  `contracts` program, which exercises Tide's executable `#[requires]` and
  `#[ensures]` checks, was a tie with those checks erased in release; file
  I/O was a small loss (Tide slower in 26 of 30 paired rounds, median
  per-round ratio 1.07x); and local TCP took 1.18x. One task-heavy program
  took 2.29x plain release Rust: 1.74x from Tide's profile flags, times
  1.31x between Tide and its plain-Rust twin built with the same flags.
  One seed of that `tasks` program under the simulator took 12.8 times the
  real run, in release.
- **Missing pieces.** No UDP, no `Rand::rng()` generator (`Rand`'s five
  draws are there), no `Proc::exit`, no signals, no FFI (calls into C), no
  fault injection in the simulator, and no `tide` command: `tide-admit` is
  the whole toolchain ([Not in this alpha](../README.md#not-in-this-alpha)).
- **A pinned nightly.** The driver links seventeen of rustc's internal
  libraries (`rustc_private`) on one pinned nightly, and the pin has never
  been bumped; a rehearsal pin bump is the recommended next check
  ([owner's guide, answers](owner-guide/answers.md), item 9).
- **A boundary that rests on rustc.** Admission closes three known routes
  through rustc's soundness bugs, compiler bugs that let safe code break
  Rust's guarantees; the others stay open, and code under an `unsafe`
  grant, the operator's explicit permission for `unsafe` operations, can
  forge a capability. Test runs the driver starts get a kernel sandbox
  (Landlock, which restricts file access, and seccomp, which restricts
  system calls), but enforcement depends on the kernel: the driver prints
  `sandbox: full` or `sandbox: partial, not enforced: …`, and a partial run
  continues under the restrictions that remain
  ([Sandbox](../README.md#sandbox)). The build and directly run binaries
  get no sandbox ([Known holes](../README.md#known-holes)).
- **A design still under test.** Safe code sees one global order of memory
  writes; weaker atomic orderings are planned only behind the `unsafe`
  grant. Experiments with preset thresholds test the design before its
  syntax freezes ([owner's guide, chapter 9](owner-guide/09-experiments.md)).

## Who should use plain Rust

- You need a second crate, from crates.io or your own workspace, an async
  runtime, or C libraries.
- You need anything under Missing pieces above, such as UDP, signals or
  process exit.
- You ship on macOS, Windows or embedded targets, or need fast incremental
  builds today.
- Your code needs custom macros, build scripts, weak atomic orderings or much
  `unsafe`.
- You need a stable toolchain and released ecosystem for production now.
- You need to run untrusted code: use a real isolation boundary, such as a
  VM, with or without Tide. Tide makes no malware-proofing claim.
- You write and review all your own code, and the Rust ecosystem matters
  more to you than authority you can read in a signature.

## Who Tide is for

Tide's best user has AI agents write much of their code and has to trust
what comes back:

- a developer or small team using coding agents for command-line tools,
  automation or small services that touch files, the network or other
  programs;
- someone who wants to know, without reading every body, what kind of
  authority a function can reach: that the config parser takes no `&Net`,
  or that the uploader can reach files. Which directory and mode it gets
  depends on the capability value its caller passes, so check that value:
  a signature shows the kind of authority, not the narrowing;
- someone writing concurrent logic (workers, queues, retries, timeouts) who
  wants a race found by seed and replayed, instead of chased in production;
- someone who can work inside one crate and Tide's std, on x86_64 Linux,
  for now;
- a Rust user, or someone ready to learn Rust, who wants these properties
  without leaving Rust's compiler or syntax.

If that is you, start with [Try it](../README.md#try-it). The README's
[v0 Core and Later Tiers](../README.md#v0-core-and-later-tiers) says what
comes next.
