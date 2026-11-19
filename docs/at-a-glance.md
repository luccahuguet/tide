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
   rejection carries a numbered diagnostic, a `TD` code such as `TD0501`,
   whose message says why.
3. **Authority shows in the signature.** Files, the clock, the environment
   (the variables and arguments a program starts with), the network and
   other programs are reachable only through capability values a function
   is handed: `&Fs`, `&Clock`, `&Env`, `&Net` and `&Proc`. Tide's standard
   library, the library every program builds on, has no free function, one
   called without such a value, that reaches them, printing aside, so a
   function's parameters bound what it can do.
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
6. **Every change gets a trust diff.** `tide-admit --tide-diff` summarizes
   what a change may do: the public functions and types whose authority
   widened or narrowed, and the `unsafe` sites, code the Rust compiler
   cannot check, added or removed. Our E3 pilot, a small trial with AI
   reviewers, tested it as a review aid and found no evidence that it helps,
   and we published that result.
7. **Built for agents.** The core reference, the document that teaches the
   language to an AI model, fits a budget of
   [25,000 tokens](/design/01-design.md#d14-toolchain-contract-for-agents),
   the units a model reads text in.
8. **Honest limits.** This first release, an alpha (an early test version),
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
   fixed their methods before looking at results, and published whatever
   they found. Our screen of all 87 malicious-crate advisories in RustSec,
   Rust's public database of security advisories (a malicious crate is a
   package published to attack its users), caught none of them. Tide's std
   had a gap, the screen exposed it, we fixed it, and re-tested under a new
   registration: a method recorded in advance, before the re-test ran
   ([research/49](/research/49-fx-zu4f.15-chrono-anchor-retest.md)). That
   separate re-test, chosen after the screen had seen its one row, found a
   catch. Our port (a rewrite in Tide) of that crate is rejected by
   admission, as it already was in the screen, and a version handed the
   authority it uses is now admitted. The registration carries an
   OpenTimestamps proof, a timestamp anchored in a public blockchain, which
   shows that a file existed by a given time. Every check is consistent
   with the registration being anchored before the fix landed, and
   independent proof of that order is not established. The re-test ran
   admission and type-checking only; no port was executed. For the pilot
   and four of the searches, the claim that methods were fixed before
   results were examined rests on our own records (self-attested). The
   other searches' method files carry an OpenTimestamps proof too, which
   shows only that each file existed by its attestation time; when those
   searches examined their candidates still rests on our records. The
   [status page](https://tidelang.com/status/) lists every round with its
   counts and its qualifier.
