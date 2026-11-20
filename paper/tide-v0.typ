// Tide v0 paper. Typst source; build with typst 0.15.0 (3ae52774) from the
// repository flake's nixpkgs pin 567a49d1913ce81ac6e9582e3553dd90a955875f:
// nix shell github:NixOS/nixpkgs/567a49d1913ce81ac6e9582e3553dd90a955875f#typst \
//   -c typst compile --root . paper/tide-v0.typ <out>.pdf
// Run it from the repository root: `--root .` lets the paper read the committed
// records its captions cite (paths starting with `/` are relative to that root).
// The figures are committed SVG files and excerpts that paper/figures/build.mjs
// generates from committed records (paper/figures/README.md); the paper only
// embeds them, so the build needs no package download and no network.
// Every link to a file in the repository goes through #repo below, which pins it
// to one release tag. A build can pass another repository or tag with
// `--input tide-repository=https://github.com/<owner>/<repo> --input tide-tag=<tag>`.

// date: none keeps the PDF byte-identical across builds (no build timestamp).
#set document(title: "Tide: A Capability-Safe, Replayable Subset of Rust for Agent-Written Code", author: "Lucca Huguet", date: none)
#set page(paper: "a4", margin: (x: 2.4cm, y: 2.4cm), numbering: "1")
#set text(size: 10.5pt, lang: "en")
#set par(justify: true)
#set heading(numbering: "1.1")
#show link: set text(fill: rgb("#1a4f8a"))
#show heading: set block(above: 1.4em, below: 0.8em)
#show raw.where(block: false): set text(size: 0.95em)
#show table: set par(justify: false)
#show table: set text(size: 8.5pt)
#show quote.where(block: true): set block(inset: (left: 1em))
#show figure.caption: set align(start)

// ---- Repository links ------------------------------------------------------
// The one place the repository URL and the release tag are set.
#let tide-repository = sys.inputs.at("tide-repository", default: "https://github.com/luccahuguet/tide")
#let tide-tag = sys.inputs.at("tide-tag", default: "v0-alpha.1")
// #repo("path") links a repository file at the tag; a path ending in `/` is a directory.
// The link text is the path, or the optional second argument.
#let repo(path, ..text) = {
  let kind = if path.ends-with("/") { "tree" } else { "blob" }
  let label = if text.pos().len() > 0 { text.pos().first() } else { raw(path) }
  link(tide-repository + "/" + kind + "/" + tide-tag + "/" + path, label)
}

// ---- Figures -------------------------------------------------------------
// paper/figures/build.mjs draws every figure from committed records, and its
// --check fails when a committed figure differs from what the records give.
// Captions read the same records for the numbers they quote.
#let e3 = json("/trials/e3-pilot/result.json")
#let bench-path = "/research/36-benchmarks/2026-11-18-bf6b958/records.jsonl"
#let bench = json("/paper/figures/data/bench.json")
#let fmt2(x) = str(calc.round(x, digits: 2))
#let budget = json("/paper/figures/data/budget.json")
#let replay = json("/paper/figures/data/replay.json")
#let search = json("/paper/figures/data/search.json")
#let search-round(r) = search.find(x => x.round == r)
#let redteams = json("/paper/figures/data/redteams.json").teams
#let thousands(n) = {
  let s = str(n)
  s.clusters().enumerate().map(((i, c)) => if i > 0 and calc.rem(s.len() - i, 3) == 0 { "," + c } else { c }).join()
}
#let count-word(n) = if n == 0 [none] else [#n]
// Tables 1 and 2 type their counts in prose; these asserts fail the compile when the figures'
// data no longer give the same numbers. Each tuple: pool, screened, eligible, caught (none: —).
#for (r, want) in (
  "fx-zu4f.15": (1, 1, 1, 1), "fx-zu4f.8": (87, 87, none, 0), "fx-zu4f.5": (1567, 1567, none, 0),
  "fx-zu4f.6": (160, 32, 0, 0), "fx-zu4f.7": (162, 119, 0, 0), "fx-zu4f.9": (100, 100, 0, 0),
  "fx-zu4f.10": (24, 5076, 0, 0), "fx-zu4f.11": (7, 42, 0, 0), "fx-zu4f.12": (30, 9, 0, 0), "fx-zu4f.13": (4, 4, none, 1),
) {
  let got = ("pool", "screened", "eligible", "caught").map(k => { let v = search-round(r).at(k); if v == none { none } else { v.n } })
  assert(got == want, message: "Table 2 row " + r + " disagrees with data/search.json: " + repr(got))
}
// Table 1: routes or attacks held, blocking finds, non-blocking or open, controls (data/redteams.json).
#for (i, want) in ((42, 2, 2, 2), (27, 5, 11, 0), (12, 1, 3, 0), (15, 0, 2, 0)).enumerate() {
  let t = redteams.at(i)
  assert((t.held, t.blocking, t.nonblocking, t.controls) == want, message: "Table 1 row " + t.name + " disagrees with data/redteams.json")
}
#let fig(name) = image("figures/svg/" + name + ".svg", width: 100%)
// Code and tool output, as recorded: one style for every excerpt.
#let excerpt(name, lang: none) = block(width: 100%, fill: luma(96%), inset: 6pt, radius: 2pt,
  align(left, text(size: 7.5pt, raw(read("figures/excerpts/" + name), block: true, lang: lang))))
#let excerpt-label(body) = align(left, text(size: 8pt, body))

#align(center)[#text(size: 17pt, weight: "bold")[Tide: A Capability-Safe, Replayable Subset of Rust for Agent-Written Code]]

#align(center)[#text(size: 12pt)[Lucca Huguet]]

#heading(level: 1, numbering: none)[Abstract]

*Authority Is an Argument.* A Tide function that reads files is handed a file capability, a value of type `&Fs`, as a parameter, and a function handed none cannot reach files, the network, the clock, the environment, other programs or randomness. Tide is a checked subset of valid Rust, in its 2024 edition, for code that artificial-intelligence (AI) models write and people review. It is _capability-safe_: _authority_, the power to observe or change the world outside the program, is reachable only through such capability values, apart from write-only diagnostic output (@sec-capabilities), so a function's signature bounds what it may do. That guards against accident. A deliberate exploit of the Rust compiler's open soundness bugs, cases where the compiler accepts code that breaks its own type rules, can still forge a capability, much as Rust's memory safety holds only up to those bugs. Tide is also _replayable_: because every source of nondeterminism is a capability, a test can run under a simulator in which one number, the _seed_, decides every scheduling choice, and a failing run can be rerun exactly from its recorded _manifest_, a file that records the run's seed, sources and inputs. This paper describes Tide v0, the alpha release. It is not a new compiler: an _admission_ driver built on a pinned version of rustc, the Rust compiler, checks each crate, Rust's unit of compilation, against Tide's rules and compiles it against Tide's own standard library (std) before rustc writes any artifact.

In v0, a seeded test finds a lost update, one concurrent write silently overwriting another, in a small concurrent service, and its replay reproduces the recorded traces, the logs of what the run did, byte for byte. Admission has closed three routes that two open rustc soundness bugs gave admitted code, and AI _red teams_, reviews that try to break a security claim, have attacked admission, the kernel sandbox that confines test runs, and the designs of process, network and randomness access; other rustc soundness bugs remain open. Our searches for external bugs are _pre-registered_: their method and counting rule are recorded before the search, and @sec-search states what our records prove about that order. On such a re-test, a port of a malicious crate from RustSec, Rust's public security-advisory database, meets the test's caught rule: admission rejects the faithful rewrite of the malicious code into Tide, its _port_, and admits a version handed the authority it uses. Tide's std had a gap, an earlier screen of all 87 malicious-crate advisories exposed it, we fixed it, and re-tested under a new registration; that re-test's one-row pool was chosen after the screen had seen the row, so its outcome was close to predictable; that screen's own result stays 0 caught of 87. Every check is consistent with the registration being anchored before the fix landed, and independent proof of that order is not established. The re-test ran admission and type-checking only; no port was executed. (A registration is _anchored_ when its hash is timestamped in the Bitcoin blockchain; @sec-catch explains.)

A pilot of the _trust diff_, a summary of the authority each exported item gains or loses between two revisions, whose protocol our own records show frozen before the run, did not meet its threshold as a review aid: plain review already caught almost every planted change, which left no room to show a benefit. A paired learnability trial, the same 30 small tasks written in each language, shows one model writing Tide about as well as Rust on the point estimates (the measured values, not their intervals), near ceiling (both arms passed almost every task), within the margins that its protocol, the plan fixed before the trial, set for parity. Benchmarks against plain Rust, on runs their tool marks unstable, price Tide's additions, from a tie for contracts (checked preconditions and postconditions) in release builds to cold debug builds 3.8 to 4.6 times slower and rebuilds after a one-line edit 5.2 to 6.3 times slower, with more for a test build that checks sampled contracts; every comparison is a tie or a cost to Tide. Three case studies that rewrite real bugs from Eon, the author's terminal workspace, and hyper, a library for HTTP, the web's request protocol, place their reported failures outside what admission rejects: two are timing errors in code that holds the authority it uses, and hyper's starts from an ambient-runtime reach but fails inside tokio, Rust's most used runtime for asynchronous code, which Tide does not admit. There Tide's simulator does the work. For Eon's discovery race, once someone states the property, the condition the code must keep, a seeded test finds the failing interleaving, the order in which concurrent steps run, over a simulated file system the project did not have to write, and replays it exactly; for the other two, a simulated test checks the fix deterministically.

_Disclosure._ Tide's code, its design text, and this paper were produced by AI agents (Claude and GPT models: mainly Claude Opus and GPT Sol, with some earlier production code by Claude Fable). The project is AI-led and human-steered: the agents made the other technical choices and reviewed and validated each change, and the author set scope, priorities and direction and made the owner calls. Human-written production code is allowed only as an explicitly marked exception. Under the project's merge rule, a change lands only after an AI supervisor pass, normally a pass by GPT Sol through OpenAI's Codex command-line tool (CLI) and then a Claude Opus review; @sec-built gives the rule and its exceptions (#repo("GOVERNANCE.md")).

_Status._ This paper describes the v0 alpha. It states each claim fully enough to follow on its own; repository paths name the raw evidence (logs, protocols and run records), linked at the release tag #raw(tide-tag). Design decisions are cited by their IDs in the project's decision record (#repo("design/01-design.md")), D0 to D18; #link(<sec-decisions>)[a list at the end] gives the ones cited with a line each. Every measurement in this paper comes from a report in the repository and is cited where it appears. Features of v1 and later tiers are labelled planned.

= Motivation <sec-motivation>

When code-writing models produce changes faster than people can read them, the bottleneck moves from writing to checking: deciding whether a change does what its description says and nothing more, and reproducing a failure well enough to fix it. A language cannot make review free, but it can decide how much of a change a reviewer must read to answer those two questions.

Tide starts from four properties of the agent's edit loop, the cycle of editing code and checking it (#repo("design/00-first-principles.md")): latency per check, the signal quality of each diagnostic, locality (a function is understood from its body and the signatures it uses), and convergence (one obvious way to write a common thing, so independent attempts produce similar code). It adds a learnability constraint. A new language has no training data, so its reference must fit in an agent's context, and its surface should borrow what models already know.

Prior evidence shaped how close to Rust Tide stays (summarized in the decision record's "Evidence That Shaped The Design"). In RustAssistant, a tool in which models fix Rust compile errors, giving GPT-3.5 a localized error with a fix description instead of a changelog-only prompt raised its repair rate for Rust compile errors from 10.7% to 73.7% (#repo("research/05-llm-code-empirics.md")). And Tide's first draft would have rejected or respelled constructs, despite their identical meaning, in 76.8% of files of a 6.08M-line Rust sample (#repo("reviews/round1-agent-usability.md")). Tide therefore keeps Rust's spelling wherever the meaning is the same (D1), and under D18 every Tide program is a valid Rust crate.

What Rust does not give is a bound on authority. Any Rust function, in any dependency, can open a file, read the clock or connect to the network through the standard library: Rust's authority is _ambient_, reachable by any code without being handed to it. Libraries and lints (extra compiler checks) that keep Rust's std cannot remove that, because the code can always reach `std`. A reviewer of a Rust change must read every body to learn what the change may touch, and a test of concurrent Rust code cannot in general be rerun deterministically from a seed, because its dependencies reach real time, real threads and real randomness. Record-and-replay tools such as rr can record one real run and replay it; rr's chaos mode randomizes schedules, but no seed reproduces a run. Tide's thesis (decision record, "Thesis") is that a language can close both gaps by construction and keep Rust's guarantees: memory safety and freedom from data races (two threads touching the same memory without synchronization, at least one writing) without a garbage collector. @fig-code shows the difference in one signature.

#figure(
  grid(columns: (1fr, 1fr), column-gutter: 10pt, row-gutter: 4pt,
    excerpt-label[*Plain Rust* (`examples/capabilities/ambient_read.rs`)],
    excerpt-label[*Tide* (`examples/capabilities/config.rs`)],
    excerpt("code-rust.rs", lang: "rust"), excerpt("code-tide.rs", lang: "rust"),
    excerpt-label[Admission: rejected with `TD0501`, the admission driver's error code for a call to an ambient standard-library function (@fig-diagnostic).], excerpt-label[Admission: admitted, no diagnostic.],
  ),
  kind: image,
  caption: [Authority is an argument. Left: in plain Rust, a function's signature says nothing about the files it may open, because any code can reach the standard library's file system, which is ambient authority. Right: in Tide, the same function must be handed a file capability, `&Fs`, so its parameters bound what it can touch, and admission rejects the version on the left.],
) <fig-code>

= Design principles <sec-design>

== Low entropy <sec-entropy>

Tide removes what costs agents the most and adds as little as it can. It has no user macros (D6), no `async`/`await` (Rust's syntax for asynchronous functions, D8), and a trait system with a hard power ceiling (D4). D14 specifies a deterministic formatter with no options; it is planned. The project's governance applies an admission filter to substantial language changes: "A feature should be rejected by default if it increases semantic entropy more than it improves safe change." The core reference, #repo("design/reference.md"), is written for agents to load whole and has a budget of 25,000 tokens, the units a language model reads text in (D14).

#figure(
  fig("budget"),
  kind: image,
  caption: [The core reference fits in an agent's context: its token count, by section, against the #(thousands(budget.budget))-token budget that D14 sets. A token is the unit a language model reads text in. The sections are the reference's own. Its preamble is the text before §0; `!` forms are macro calls such as `println!`; derives and attributes are `#[..]` annotations that generate or configure code; lifetimes say how long a borrowed reference stays valid; traits are shared interfaces, and generics are code written once for many types; a panic is Rust's stop on an unrecoverable error; modules and packages organize code into files and crates. `unsafe` code is code whose memory safety the programmer, not the compiler, must uphold, and trust holes are the places where admitted code may do what admission cannot check (@sec-trust). The toolchain loop is the planned `tide` commands an agent runs on each edit.],
) <fig-budget>

== Capabilities bound authority <sec-capabilities>

Code touches the world only through _capability_ values it is handed (D7): `Fs` for files, `Net` for the network, `Clock`, `Env` for the environment (the variables and arguments a program starts with), `Proc` for other programs, `Tasks` for concurrency, and others. A program's `main` receives a `Root`, the capability that holds all the others, and passes narrower capabilities down; a test can take a simulated root instead. A function that takes no capability cannot read the clock, the network, files, the environment or randomness, or start a program, in any dependency, by design. In v0, `Proc` starts other programs (`proc.command("git")`), and `Net` covers TCP connections (the internet's reliable byte streams), one HTTP/1.1 `get` over TLS (which encrypts the connection), and name lookup (`resolve`). `Rand`, the randomness capability, draws integers (`next_u64`, `random_range`), bytes (`fill_bytes`) and booleans (`random_bool`), and shuffles slices; under simulation the seed fixes each draw. UDP (the internet's connectionless datagrams), a seeded generator on `Rand`, and `Proc::exit`, which ends the program, are planned.

Capabilities can be narrowed. A directory capability can be cut down to a subdirectory or made read-only, and `only([...])` keeps a network capability to listed hosts, a process capability to listed programs and an environment capability to listed variables. Two limits of narrowing are part of the contract. A network `only` checks the hosts a program connects to, and accepting incoming connections stays possible until `no_bind()` removes it. And an allowed program runs with its own full authority, so allowing a shell, a program that runs other programs, allows anything. Write-only diagnostic output (`print!`, `eprintln!`, logging) goes to _sinks_, which are not capabilities and cannot read anything back (D7). The design rejected effect rows and effect variables, type-level lists of the effects a function may perform, and implicit capability resolution in favor of plain values (D7, "Rejected").

== Deterministic concurrency through the simulator <sec-concurrency>

Concurrency is structured (D8). _Tasks_, Tide's units of concurrent work, are spawned into scopes, blocks that wait for every task spawned in them, from the `Tasks` capability and cannot outlive them; the first failure cancels the siblings; cancellation is level-triggered, so every later wait in a cancelled scope sees it, with deadlines that every wait in a scope shares. Because time, scheduling, files and the network are all capabilities, the same program can run against a simulated root, `Root::sim(seed)`, in which the seed decides the schedule and virtual time, the simulator's own clock, advances only when every task is blocked (D15). A simulated test run started through the driver writes a replay manifest that records the seed, the content hashes of the sources, a semantic build identity and the injected inputs; replay refuses to run against a mismatched identity (D15). Running the test binary directly records no manifest.

D9’s parallel regions give schedule-independent successful results without exploring task schedules. The library functions `join`, `par_map` and `par_for` accept only closures and elements that are `Pure`, a marker for types from which no capability, synchronization handle or foreign handle is reachable, together with Rust's `Send` and `Sync` thread-safety markers, and they reach mutable data only through disjoint splits; admission and rustc's trait and borrow checks enforce this. D9 then gives such regions schedule-independent successful results by admission: when no branch panics, a region returns what its sequential version returns (D9’s sequential equivalence). This is a checked contract, not a formal confluence theorem, a proof that every schedule gives the same result. D9’s qualifications apply: when a branch panics, data the region could write has unspecified, memory-safe contents that only destructors can observe, and code behind a _trust hole_, a place where admitted code may do what admission cannot check, such as `unsafe` code admitted under an explicit grant (@sec-trust), is outside what admission checks. The parallel runtime is partial: it forks operating-system (OS) threads where a pool is planned and records its limits on panics and divergence (#repo("crates/tide-std/src/par.rs")\; the core reference §6.8). Under the simulator, parallel regions run sequentially, in sequential order.

== Contracts <sec-contracts>

Contracts use toolchain attributes, `#[requires(..)]` and `#[ensures(..)]`, plus loop `invariant!(..)` (D10). In v0, debug and test builds evaluate _checked_ clauses, those built from operators and the standard library's accessors. _Sampled_ clauses, such as one with a quantifier or a call to a user function, run only in test builds, each under a step budget (D10; the core reference). Contracts also drive property tests, which check a stated property on generated inputs, and release builds erase them. Proof of contracts is planned for v1 (the decision record's tier table); v0 claims no proofs.

== Admission: a checked dialect of Rust <sec-admission-design>

Tide v0 is a dialect, not a separate compiler (D18). Every Tide program is a Rust 2024 crate that a pinned, unmodified rustc nightly (rustc's unstable daily build) compiles against Tide's standard library, which the driver passes as `std`, so the real standard library's paths cannot be named. Tide's checker runs inside rustc as a driver and gates every crate before rustc writes any artifact for it. It owns the subset, authority, the `Pure` trait, contracts, the _TD codes_ (the driver's own error codes, such as `TD0206`, each with a message that says why) and the trust diff, and it enforces an admission allowlist over Rust's lower-level libraries `core` and `alloc` and std's inherent methods. Because the checker is rustc itself, with rustc's own name resolution, no separate argument is needed that Tide and rustc read a program the same way.

#figure(
  excerpt("diagnostic.txt"),
  kind: image,
  caption: [What a rejection looks like: a TD code is rustc-style output from the admission driver, here `TD0501` on @fig-code’s Rust version: a call to an ambient standard-library function, which only a capability may reach, with its file, line and column. rustc also reports `E0425` at the call, because Tide's std has no such function.],
) <fig-diagnostic>

== A reviewable trust surface <sec-trust>

The compiler computes each public item's signature and declared authority and lists _trust holes_, the places where admitted code may do what admission cannot check, such as every site an `unsafe` grant admits (D11). The trust diff compares this surface between two revisions. In v0 it shows the declared authority of exported items only: which exported functions, methods and types gained or lost a capability. It does not show exercised authority, effects, or narrowing inside a body. We describe it as a summary of what a change may do. @sec-e3 reports a pilot that found no evidence it helps reviewers catch more.

#figure(
  excerpt("trust-diff.txt"),
  kind: image,
  caption: [The trust diff summarizes what a change may do: between two revisions, the exported items whose declared authority grew or shrank. Here the demonstration's patch widens `hits::serve`, which gains network authority (`+ metrics: net`). The `-` and `+` lines at the top quote the patch's old and new signature of `serve`. Under _declared authority_, `+` marks an item or capability gained and `~` an item whose authority changed. The added associated function, a function defined in a type's `impl` block, is the patch's conversion from an input/output error into the service's error type.],
) <fig-trust-diff>

= Implementation <sec-implementation>

The v0 production code is one Cargo (Rust's build tool) workspace, built with the nightly rustc that the repository pins (#repo("AGENTS.md"), "Verification"). @fig-pipeline shows the path a simulated test run takes through its parts.

- *`tide-admit`*, the admission driver, is a `rustc_private` driver, one that links rustc's internal crates, on the pinned nightly. It owns every rustc invocation and adds no user-controlled `--extern`, `-L`, `--sysroot`, `--cfg` or semantic `-C` flags (D18). It checks resolved uses before rustc's analysis, then a backstop checks the instances that can run. Today it admits a single crate, or one test suite paired with one library, against Tide's std (#repo("demo/README.md"), "What runs today, and what is planned"). It also runs tests (`--tide-run-test`), replays manifests (`--replay`) and prints the trust diff (`--tide-diff`). The planned `tide` command (`tide test`, `tide sim`, `tide review`) does not exist yet; the driver's modes are its partial precursors.
- *`tide-std`* is Tide's standard library, with its macros in `tide-macros`. It provides the capabilities, among them `Proc::command` for child processes and `Net`’s TCP, HTTP `get` and `resolve`; structured tasks, channels, `Shared` cells, and parallel regions; and the simulator of tasks, channels, `Shared`, files, TCP loopback, HTTP peers and name lookups, child processes as declared models, the clock, randomness draws, the environment and standard input.
- *The simulator and replay.* In v0 the simulator runs each task on its own OS thread and passes a baton so that one runs at a time (#repo("design/03-authority-concurrency.md") §9.1). A `#[sim_test]` runs a test over a fixed set of seeds; a run through the driver records a manifest and a trace, and `--replay` checks the identity and reproduces the run.
- *The RUN-3 test sandbox.* The type system alone cannot bound what admitted code reaches, because rustc has open soundness bugs that safe code can exploit to forge a capability (D7; @sec-admission). Test and simulation runs that the driver starts therefore run as a child process inside a kernel sandbox, which the design rule RUN-3 specifies: Landlock, which limits the files a process can open, and seccomp, which limits its system calls, on Linux, installed between fork and exec (#repo("design/03-authority-concurrency.md"), RUN-3). The driver prints the level before the child starts: `full`, or `partial, not enforced`, naming what the kernel lacks. A test binary run directly gets no sandbox, and the build itself (rustc and the linker) has none yet, the planned rule RUN-2 (#repo("docs/owner-guide/08-sandbox.md")).
- *`tide-conformance`* checks a _conformance corpus_: every normative rule in the design chapters carries an ID and is traced to cases, small programs with expected verdicts, that an implementation must pass (D0).

#figure(
  fig("pipeline"),
  kind: image,
  caption: [The path of a simulated test run started through the driver (`--tide-run-test`): the admitted crate's test runs in the sandboxed child under the simulator, and the run's manifest is what replay checks. A test binary run directly gets no sandbox and records no manifest, and the build itself has no sandbox yet (RUN-2).],
) <fig-pipeline>

= Evidence <sec-evidence>

We report the evidence in the order chosen for the alpha: replay and admission first, with the red teams, the external catch and the case studies, then the pre-registered trust-diff pilot (E3), then the learning-cost trial (E1), then benchmarks. Each subsection states its scope once, where it applies.

== Replay <sec-replay>

The demonstration (#repo("demo/README.md")) builds a small concurrent service through the admission driver, with a patch that introduces a lost update. Its suite's `#[sim_test(seeds = 16)]` runs the same code on the same input under a seeded scheduler for seeds 0 to 15. On the buggy revision 8 seeds pass and 8 fail (seeds 0, 2, 4, 8, 9, 10, 12 and 14), each failing on the `lost hits` assertion; only the schedule differs between seeds. The demo records seed 0 and replays the recording twice: the recording and both replays have 40 events and identical hashes of the event trace, the run's whole trace, and the output trace, its `sink` events, which hold what the test printed. The fixed revision passes all 16 seeds. A further step runs the replay under `rust-gdb`, the Rust wrapper of the GNU debugger, with breakpoints in the service, when gdb is available (demo README, "Run it" and "What the demo does"). `cargo run -p tide-demo` runs it, and the workspace test suite runs it too, with two runs that must fail.

#figure(
  fig("replay"),
  kind: image,
  caption: [A seed finds the lost update and replay reproduces it. Each seed is one simulated schedule of the same test: on the buggy revision #(replay.buggy_failing.len()) of #(replay.seeds) seeds fail, on the fixed one #count-word(replay.fixed_failing.len()). The recorded run of seed #(replay.recorded_seed) and both replays have identical event and output trace hashes.],
) <fig-replay>

#figure(
  excerpt("manifest.txt"),
  kind: image,
  caption: [The replay manifest records a run: the seed, the content hashes of the sources, the build identity and the injected inputs. Replay checks source and build identity before running, uses the recorded seed and choices, and fails if the run's inputs, trace or outcome differ from the recording. The manifest is JSON, a text format of named fields. Each long hexadecimal string is a SHA-256 hash, a 64-digit fingerprint of content, cut here to 16 digits. `roots` lists each simulated root the test created, by its `ordinal`, its creation order, with its seed and the inputs it read (none here). `profile` names the build profile, here the unoptimized debug build; `outcome` is the run's result, here a panic on the failed assertion. `trace_hash` is the hash of the event trace, and `input_hash` is the hash of the whole manifest with that field left empty, so replay notices an edited field.],
) <fig-manifest>

The same demo shows the trust diff on a patch that gives the service's public function a `Net` parameter: `hits::serve` is listed as widened, with `+ metrics: net` (@fig-trust-diff). This is the trust diff working as a summary; @sec-e3 reports what a pilot measured about reviewers using it.

Replay is bit for bit on the same machine, for the model the simulator has. It reproduces what the simulator models, not every behavior of the real program (@sec-limitations).

== Admission <sec-admission>

On 2026-11-15 a probe (`fx-6xi2`; IDs of this form name issues in the project's tracker) showed that the driver admitted rustc's own test case for rust-lang/rust\#25860, an issue open since 2015, and a variant that turns a borrowed `&'a Fs` into a `&'static Fs` with no `unsafe` and no grant. Two things followed: `TD0206` now rejects the function-pointer route of \#25860, and the kernel sandbox for test runs moved into v0 (D7) (#repo("docs/owner-guide/03-soundness-bugs.md")).

A red team on 2026-11-17 then found that admission let through one more known route: the trait-object overlap of rust-lang/rust\#57893, open upstream since 2019, which gives safe code a generic `transmute`, a function that reinterprets one type's bytes as another's, through a blanket impl, one that implements a trait for every type. That blanket form is the issue's original report, and a `transmute` through the same overlap was already posted upstream as rust\#152783 (theemathas, 2026), closed as a duplicate of \#57893. The review of its fix found a second, projection form, an impl for another trait's associated type, which was also already known upstream (Niko Matsakis named it on \#57893 in 2019). Neither form is a Tide discovery. `TD0402` now rejects both (@sec-redteams).

In all, admission has closed three admitted routes from two rustc soundness bugs: the function-pointer route of \#25860 and both forms of \#57893. The red team also checked that the known reproductions of other routes (\#25860 through closures and trait objects, \#84591, \#100051) are rejected; that covers those reproductions, not every variant, and the \#100051 family stays open in the design (#repo("docs/owner-guide/03-soundness-bugs.md"), "The hardest question"). rustc has other open issues labelled `I-unsound`, its label for soundness bugs. Tide's claim is therefore narrow: no _accidental_ ambient authority, where accidental means code that does not exploit a compiler soundness bug, in admitted safe code outside declared trust holes such as code under the `unsafe` grant (`--tide-grant=unsafe`). Against a malicious dependency, safety rests on rustc's soundness, on the routes closed so far, and, for test runs the driver starts, on the kernel sandbox.

*When capabilities would be unforgeable.* Admitted safe code could forge no capability only once every soundness hole that admitted code can reach is closed. Two routes lead there. First, rustc could fix those holes, with each new toolchain pin audited against admission's allowlist, but fixing the known issues cannot show that no unknown one remains: this route only shrinks the known holes. Second, a machine-checked proof over the admitted subset could show that no admitted program reaches one, and only such a proof could support the word unforgeable. Either way, Tide's own trusted code must also be correct: `tide-std`’s `unsafe` runtime, and the completeness of admission's rules, whose gaps over rustc's holes are what the red team found. A proof of a model covers rustc and that code only as far as the model matches them. That proof is deferred work after the alpha (`fx-ip8d`) and has not started. Until it exists, v0’s claim stays no accidental ambient authority, and we do not call v0’s capabilities unforgeable.

== Red teams <sec-redteams>

A _red team_ here is an adversarial review: an AI model, given a security claim of Tide's, tries to break it and records every route it tried with the result. Five red teams have run, on admission and the RUN-3 sandbox, on the design of `Proc::command`, on the designs of `Net`’s HTTP `get` and name lookup, and on the design of `Rand`’s draws. Each design red team was read-only: it read the design, the code it builds on and the pinned sources, and compiled and ran nothing. The committed records do not establish whether each red team was independent of the change's author or was briefed with only the attack goal. The implementation of each design was then reviewed by the merge rule's Codex pass and Opus review (@sec-built), and the defects they found that had to be fixed before landing, their blocking finds, are counted in the table's last row. @tab-redteams summarizes them; the records are linked below.

#figure(
  table(columns: (2.5cm, 2.2cm, 1.9cm, 1fr, 1.7cm, 1fr), align: left, inset: 4pt, stroke: 0.4pt,
    table.header([*Red team*], [*Surface*], [*Tried*], [*Found*], [*Fixed*], [*Left open on purpose*]),
    [2026-11-17, Claude Opus 5.5], [Admission rules; RUN-3 sandbox], [48 routes, controls included], [2 admitted forms of one route (\#57893)], [2, by `TD0402`], [2: a write to standard error, a sink every program holds; an inert crate attribute],
    [`Proc::command` design, 3 rounds, Claude Opus 5.5], [Process authority, `only`, the child's environment and descriptors], [27 attacks held, 9 findings], [No route to a process without `Proc`; 3 blocking misstatements of what the authority bounds, then 2 more in round 2], [All 5 blocking, by round 3], [Stated limits, e.g. an allowed program runs with its own full authority; a child outlives the call that started it],
    [`Net::get` design, 2 rounds, GPT-6.1 Sol], [Network `only`, HTTP, TLS], [12 attack classes held, 4 findings], [1 blocking: `only([])` still let `bind` send name lookups, so a destinations-only reading was false], [1, by stating the true boundary], [3 limits stated, e.g. simulated peers are no evidence of a real TLS identity],
    [`Net::resolve` design, 1 round, GPT-6.1 Sol], [Name lookup], [15 attacks held, 2 findings], [No blocking hole], [—], [2 simulator limits stated],
    [`Rand` draws design, 1 round, GPT-6.1 Sol], [Randomness sources, ranges, simulated streams], [20 attacks held, 4 limits stated], [No blocking break], [—], [4 limits stated, e.g. one simulated word can reveal the seed],
    [Implementation reviews], [The landed code of `Proc::command`, `Net::get`, `Net::resolve`, `Rand`], [Every change], [12, 12, 3 and 2 blocking finds; 1 an authority bypass], [All, before landing], [—],
  ),
  kind: table,
  caption: [Red teams and implementation-review finds, each count from its record. _Tried_ counts the routes or attack classes a red team recorded; a _finding_ is a defect or an overstated claim it reported, _blocking_ when it had to be fixed before the change landed.],
) <tab-redteams>

#figure(
  fig("redteams"),
  kind: image,
  caption: [What each red team tried and found, next to @tab-redteams; counts span every round of each red team: most attacks held, every blocking find has been fixed, and the routes left open are stated limits.],
) <fig-redteams>

*Admission and the sandbox, 2026-11-17.* Claude Opus 5.5 tested the claim that admitted code reaches no authority it was not handed, and that a test child escaping the type system anyway meets the kernel (#repo("reviews/2026-11-17-red-team-opus.md")). It numbered each route and tried it against the driver and the sandbox, with accepted controls for the overlap route and unsandboxed controls for the process-memory and loader probes, so that a denial could be told from a probe that never worked. It tried 48 routes. The blanket form of \#57893 was admitted at the red team's starting revision; the projection form, an impl for an associated-type projection, was found by the Opus review of the fix and was still admitted then. `TD0402` now rejects both. Two routes stay admitted on purpose: one writes only to standard error, which every program holds as a sink, and one is an inert crate attribute. Every other admission route the record lists is rejected, and every RUN-3 route gets the denial RUN-3 specifies. The same red team replaced the allowed glob (a `::*` import of every item in a module) over `std::path` with a list of its items, so an input/output (I/O) method that a later rustc adds is not admitted until the pin review lists it. Unlike the later design red teams, this one also wrote the fixes, which the change's own Codex pass and Opus review then checked.

*`Proc::command`, three rounds.* Claude Opus 5.5 reviewed the design of process authority before it was built (#repo("reviews/fx-ayh2.1-redteam-opus.md")). Its first round recorded 27 attacks that held, among them ambient process calls, foreign functions, the program's search path, loader variables such as `LD_PRELOAD`, inherited file descriptors and attenuation tricks, and found no route that starts a process without `Proc`. It reported 9 findings: 3 blocking statements that described what the authority bounds as narrower than it is, for example that a program's full authority came only from its arguments and variables when files, scripts and the return path give it too, and 6 non-blocking items: limits to state, requirements for the implementation, and one nit. The second round found the 9 fixed and 2 new blocking text defects: a quoted list of the C library's dangerous loader variables left out seven entries, and a guard did not cover re-exports (#repo("reviews/fx-ayh2.1-redteam-recheck-opus.md")). The third round found both fixed and the design sound with stated limits, with 3 non-blocking text items (#repo("reviews/fx-ayh2.1-redteam-recheck2-opus.md")).

*`Net::get` and `Net::resolve`.* GPT-6.1 Sol, at high reasoning effort, reviewed the HTTP design (#repo("reviews/fx-ff13-redteam-codex.md")). It recorded 12 attack classes that held, among them host spellings, ports, redirects, request splitting and proxies, and 4 findings. The blocking one: `only([])` still let a program look up names through `bind`, so the design's account of `only` as an exact destination boundary for the whole capability was false. The fix states the actual boundary; the other three are limits stated in the design: simulated peers are no evidence of a real TLS identity, system name resolution keeps caches and traffic after a caller cancels, and the implementation must keep TLS key logging off. Its re-check found all four addressed and the design sound with stated limits (#repo("reviews/fx-ff13-redteam-recheck-codex.md")). The name-lookup design's red team recorded 15 attacks that held and 2 non-blocking simulator limits, and found the design sound with stated limits (#repo("reviews/fx-6d50-redteam-codex.md")).

*`Rand`’s draws.* GPT-6.1 Sol, at high reasoning effort, reviewed the design of the five draws before they were built (#repo("reviews/fx-qlek.20-s1-redteam-codex.md")). It recorded 20 attacks that held, among them ambient randomized maps, address and pointer entropy, real handles captured into simulated tasks, empty or reversed ranges and cancellation partway through a draw, and found no blocking break. It stated 4 limits, for example that one simulated word can reveal the seed and so predict the run's other simulated choices, which exposes simulator decisions, not host entropy; it found the design sound with stated limits.

*Implementation reviews.* The merge rule's reviews of the built code found 12 blocking defects in `Proc::command` over three rounds (#repo("reviews/fx-ayh2.1-s2-r1-codex.md") and the later rounds), 12 in `Net::get` over three rounds, 3 in `Net::resolve`, and 2 in `Rand`’s draws (#repo("reviews/fx-qlek.20-s2-codex.md"), #repo("reviews/fx-qlek.20-s2-opus.md")); all were fixed before each change landed. One was an authority bypass. An environment key containing `=`, such as `GCONV_PATH=/tmp/injected`, passed the check that refuses the C library's dangerous loader variables under `only`, because the check compared the whole key while the child's C library reads the name up to the first `=`; the fix checks that effective name, and a regression test covers it. The others were defects in process and connection lifecycles, HTTP parsing, cancellation, simulation checks, tests and documentation.

These are AI red teams, not a human security audit. A count of routes tried is not a count of all routes, the design red teams read designs rather than running code, and rustc's other soundness bugs stay open (@sec-admission).

== External bugs: a catch, the case studies and the search record <sec-search>

=== A malicious crate, rejected, then admitted with its authority declared <sec-catch>

RustSec is the Rust community's public database of security advisories, and 87 of its advisories report malicious crates: packages published to attack the people who use them. `chrono_anchor` (RUSTSEC-2026-0039) is one: it tried to send a project's `.env` files to a server impersonating `timeapi.io`. On a re-test registered in advance (`fx-zu4f.15`), a port of its code meets the caught rule of the earlier screen of all 87 advisories (`fx-zu4f.8`). The rule has two legs. Leg 1: a faithful port of the malicious code draws a TD code on an operation in the screened classes (ambient process, network, file, environment or clock use, a detached thread, global mutable state, or `unsafe` without a grant). Leg 2: a version that receives the authority it uses as capability parameters is admitted with no diagnostic (#repo("research/49-fx-zu4f.15-chrono-anchor-retest.md")).

The faithful port draws `TD0602` twice, on two `thread::spawn` calls where Tide requires tasks in a scope, and `TD0501` three times, on two ambient `Command::new` calls and an ambient `thread::sleep`. The capability-passing version is admitted, and its trust artifact, the file the driver writes listing each item's declared authority, declares the authority it uses: task authority (`bg`, the task scope it is handed), clock and process authority. The advisory is public, and the test ran admission and type-checking only; no port was executed.

Tide's std had a gap, the fx-zu4f.8 screen exposed it, we fixed it, and re-tested under a new registration. The gap was in leg 2: the documented idiom `clock.sleep(d)?` did not compile inside a task body, because the std had no conversion from a sleep's cancellation error into a task's error. A std fix (`fx-qlek.18`) added that conversion, and the re-test checked the row at the fixed revision with fx-zu4f.8’s port files, byte for byte. Leg 1 already held in fx-zu4f.8; leg 2 is what changed. fx-zu4f.8 itself stays a documented negative: 0 caught of 87 (#repo("research/47-malware-rescreen.md")).

The re-test's registration carries an OpenTimestamps (OTS) proof, which anchors a file's hash in a Bitcoin block and so shows that the file existed by that block's time. Every check is consistent with that anchoring preceding the fix's landing, and independent proof of that order is not established: the block's time is 61 minutes before the landing's self-reported time, inside a block timestamp's tolerance of about two hours (research/49 §3).

The record states the catch's scope. It is not a first-time catch and not a rate: the one-row pool was chosen after fx-zu4f.8 had seen its rows, and the outcome was close to predictable. The fix also reverses an earlier removal of the same conversion, made because it breaks type inference for an unannotated `tasks.scope(..)?` (rustc's `E0283`); 17 scope sites now name their error type. Admission does not judge intent: the admitted version still starts the same processes with the same arguments, and what changes is that its authority is declared; the `proc` grant says the code starts programs, not what they do, and the programs it starts, which receive the `.env` path, run with their own full authority. And the port was written from a public quote of code already known to be malicious, so Tide found nothing unaided.

=== Case studies: failures admission does not reject <sec-case-studies>

Three case studies rewrite real bug reports in Tide (#repo("examples/case-studies/")). Admission rejects none of the three reported failures, so the work falls to the simulator and, for hyper, to the fix's shape: a seeded test of a property someone states fails on the buggy revision, passes on the fixed one, and reruns exactly.

*Eon: discovery races a host's exit* (#repo("examples/case-studies/eon-discovery-race/README.md")). Here a seed found the failing interleaving, the order in which concurrent steps happen to run. Eon [17], the author's own terminal workspace, has a supervisor that finds its terminal hosts through files. In bug eon-8ehhn, the supervisor read a host's record as live, the host then exited, publishing its tombstone and removing its socket, and the supervisor's socket check failed with an error that failed whatever needed discovery. Eon's fix rereads the record once after a failed check, and its deterministic test places the exit by hand, through a hook added to discovery. The Tide rewrite models the order of the four file operations, not Eon's code. Its `#[sim_test]` starts the exit and discovery together with no hook, and every file operation is a scheduling point. On 13 of 100 seeds the seed puts the whole exit between discovery's read and its check, and discovery fails; the runner reruns each failing seed and requires an identical trace. The fixed discovery passes on all 100. With the host's two steps swapped, the fix fails on 11 of 100 seeds, which shows that the fix rests on the host's order, not that Eon's host keeps it. Beyond the deterministic test of the fix, which Eon already had, the seed finds the interleaving without knowing the window in advance, and the failing seed reproduces it. Rust can let a scheduler such as loom or Shuttle find it too, once the project puts its file operations behind a simulated implementation that it writes and maintains, or adopts from a simulation framework; Tide's std provides that file system because `Fs` is a program's only file access (the case study's README, "What the Rust version would need"). The buggy discovery is two ordinary file operations through the `Fs` it is handed, so admission accepts it, as it should.

*Eon: one slow close blocks every request* (#repo("examples/case-studies/eon-slow-close/README.md")). Here the test runs in virtual time. Eon's supervisor answers every control request on one loop thread. In bug eon-8wh7h, closing a tab stopped its terminals one after another on that thread, and each terminal slow to exit cost a 500 ms stop grace. Eon's own reproduction closed a 14-terminal tab in 6992 ms under input/output and processor stress, and the window's Inspect request timed out at 6011 ms against its 6 s deadline. Eon fixed it by stopping a close's terminals together under one shared deadline. The Tide rewrite models the loop's structure, not Eon's code. A `#[sim_test]` sends an Inspect every 250 ms during the close and requires an answer within 6 s. It fails on the buggy revision and passes on the fixed one, in which the bound is a scope deadline that every wait inside shares. Rust can write the same test. What Tide adds is that the test runs in virtual time, deterministically, with no fake clock written for it: the 7 s close takes milliseconds, where Eon's real-time red/green test fails at 3.78 s. Async Rust gets the same from the paused clock of tokio, the most used async runtime for Rust; Eon's thread-based supervisor would need a clock it is handed. A third file encodes the still-open eon-pimct, a terminal start that waits on the loop, as a failing test. The buggy loop is ordinary Tide, which admission accepts.

*hyper: the pool reaches for an ambient runtime* (#repo("examples/case-studies/hyper-ambient-runtime/README.md")). Here Tide gives the fix its shape. hyper 0.14’s client pool [18] runs its idle-connection reaper on whichever tokio runtime, the async executor, is ambient. In the rewrite, a pool can start its reaper only in a scope its caller hands it and read time only from a `Clock` it is handed, and the rewritten pool's `#[sim_test]` shows it closing idle connections on each of 20 seeds. Of the two reported failures, one is a panic in tokio's timer; in the rewrite that timer gets only rustc's `E0433`, an unresolved path, because Tide admits no external crate. The other, hyper\#2808, is a reaper that never starts; a pool whose optional clock is left unset is admitted, and the reaping test fails on it.

Across the three, a test of a property someone had to think of is what fails, and Tide makes that test cheap and deterministic: it runs in virtual time, a seed can find the interleaving over the simulated file system that Tide's std already provides, and a failing seed reruns exactly. Where a fix bounds time or authority, Tide also gives it a structure that makes the bound explicit.

=== The search record <sec-search-record>

The catch above comes from a series of registered searches for external bugs, and the series is what gives it a denominator. @tab-search lists every round with what it screened. The admission rounds port malicious or ambient code and apply the caught rule. The simulation rounds look for a real, fixed concurrency bug that a seeded `#[sim_test]` would catch and replay: a candidate is _eligible_ when it has a public fix and an observed symptom, is concurrency-shaped, can be written faithfully in Tide v0, and lies within what the simulator explores. Each record's counts describe its own pool, not a rate.

#figure(
  table(columns: (2.3cm, 1fr, 1fr, 1.6cm, 1.3cm, 1fr), align: left, inset: 4pt, stroke: 0.4pt,
    table.header([*Round*], [*Pool*], [*Screened*], [*Eligible*], [*Caught*], [*Registration*]),
    [`fx-zu4f.15` (research/49)], [`chrono_anchor`, chosen after fx-zu4f.8 had seen it], [1 row], [1], [*1*], [OTS-anchored; order consistent, not independently proved],
    [`fx-zu4f.8` (research/47)], [All 87 RustSec malicious-crate advisories], [87: 71 with no portable source, 13 ports incomplete, 2 with no TD code, 1 capability then absent], [—], [0], [Protocol committed before the run; OTS-anchored],
    [`fx-zu4f.5` (research/37)], [1,507 in-date RustSec advisories, of which 87 are malicious-crate advisories; 60 GitHub results], [All 1,567: 87 advisories qualify, 12 of them with quoted code ported or assessed; the 60 results were all open at search time, so none was testable], [—], [0], [Self-attested; criteria and protocol hash reached git about three minutes after the first request],
    [`fx-zu4f.6` (research/38)], [160 raw GitHub results (158 unique), before language and star filters], [32 unique issues], [0], [0], [Self-attested; criteria in git before the search, protocol hash after],
    [`fx-zu4f.7` (research/39)], [162 unique std-level issues], [119], [0], [0], [Self-attested; criteria in git before the search, protocol hash after],
    [`fx-zu4f.9` (research/41)], [100 bugs from Qin et al.’s dataset of Rust concurrency bugs (PLDI 2020, a programming-languages conference)], [100], [0], [0], [Self-attested; criteria and protocol committed before the first request],
    [`fx-zu4f.10` (research/44)], [24 popular crates], [5,076 keyword-match rows; 10 crates' concurrent cores], [0, and 1 row unresolved], [0], [OTS-anchored; examination times self-attested],
    [`fx-zu4f.11` (research/46)], [7 repositories whose commit messages mark agent-written commits], [42 fix candidates; 7 concurrent cores], [0], [0], [OTS-anchored; examination times self-attested],
    [`fx-zu4f.12` (research/52)], [30 cryptography and blockchain crates], [9 fix candidates; 430 properties the crates state], [0 fix candidates; 47 properties modeled], [0], [OTS-anchored; examination times self-attested],
    [`fx-zu4f.13` (research/51)], [4 rows of earlier rounds, chosen after they were seen, when `Proc::command`, `Net::get` and `Net::resolve` landed; its catch is the `fx-zu4f.15` catch, counted once], [4 rows], [—], [1, not new], [OTS-anchored; pool chosen after its rows were seen],
  ),
  kind: table,
  caption: [Every registered search round for an external bug, each count from its record. _Screened_ is what the round judged against its criteria. _Self-attested_ means the order of registration and search rests on our own git and tracker timestamps; _OTS-anchored_ means an OpenTimestamps proof shows only that each registered file existed by its attested time, not that no screening happened earlier.],
) <tab-search>

#figure(
  fig("search"),
  kind: image,
  caption: [The search record as stages: what each round of @tab-search started from, screened, found eligible and caught. The one catch is a re-test of `chrono_anchor` in a #(search-round("fx-zu4f.15").pool.n)-row pool chosen after `fx-zu4f.8`’s screen had seen the row. Tide's std had a gap, the `fx-zu4f.8` screen exposed it, we fixed it, and we re-tested under a new registration. Every check is consistent with the registration being anchored before the fix landed; independent proof of that order is not established. The re-test ran admission and type-checking only, and no port was executed. `fx-zu4f.8`’s own result stays #(search-round("fx-zu4f.8").caught.n) caught of #(search-round("fx-zu4f.8").pool.n). `fx-zu4f.13`’s #(search-round("fx-zu4f.13").caught.n) is that same catch, counted once. The other rounds' denominators are what make that catch credible; they differ in kind and give no catch rate.],
) <fig-search>

In `fx-zu4f.5`, the nearest row was the same `chrono_anchor`: its port was rejected with `TD0602` and `TD0501`, and its capability-passing version could not be written in v0, because `Proc` had no `command` then (#repo("research/37-external-admission-search.md")). `Proc::command` has since landed, and `fx-zu4f.8` was registered as a new test, not a revision of `fx-zu4f.5`, whose result stands as published. In the simulation rounds, every candidate that reached the portability check failed it: their affected paths ran through external crates, the network, subprocesses, signals (the operating system's notifications to a process), `async`, `unsafe` or weak-memory reorderings that Tide v0 does not express or the simulator does not explore (#repo("research/38-external-concurrency-search.md"), #repo("research/39-std-concurrency-search.md"), #repo("research/41-qin-dataset-search.md")). `fx-zu4f.10` keeps one row unresolved (B23.128), whose required evidence is missing (#repo("research/44-fx-zu4f.10-concurrency-round.md")); `fx-zu4f.11` withheld no private candidate (#repo("research/46-fx-zu4f.11-agent-marker-round.md")). `fx-zu4f.12` screened cryptography and blockchain crates in two ways: none of its 9 fix candidates was a bug fix in the crate's code, and none of the 47 properties it modeled in Tide met the caught rule; it holds privately one model-only candidate, a failure seen only in its Tide model of a crate, which the caught rule does not count (#repo("research/52-fx-zu4f.12-crypto-crates-round.md")). `fx-zu4f.13` re-screened 4 rows of earlier rounds once `Proc::command`, `Net::get` and `Net::resolve` had landed; its one catch is `chrono_anchor` again, not independent of `fx-zu4f.15` and counted once (#repo("research/51-fx-zu4f.13-landed-capability-rescreen.md")).

The project's commitment from `fx-zu4f.8` on is that a protocol is committed before any candidate is examined; once the repository is public, a registration means a committed protocol pushed before the results.

== E3: a pre-registered negative pilot of the trust diff <sec-e3>

E3 asks whether reviewers catch more planted authority-widening changes with the trust diff than with the plain code diff (D0). D0’s threshold is 1.5 times as many catches. The protocol was frozen before the run, an order that rests on our own git and tracker records (self-attested), and the report's first paragraph is the sentence the protocol's §12 requires. We quote it unchanged (#repo("trials/e3-pilot/report.md")):

#quote(block: true)[In the E3 pilot, Opus and Sol reviewed 12 planted authority-widening changes repeatedly, 6 times per change in each arm. With the trust diff, 66 of 72 review sessions caught the planted widening; with the plain diff, 67 of 72 (ratio 0.99, 95% interval [0.95, 1.00]). This does not meet D0’s 1.5x threshold on the point estimate. The interval does not rule out no benefit.]

#figure(
  fig("e3"),
  kind: image,
  caption: [E3, a pre-registered pilot with a negative result, drawn from #repo("trials/e3-pilot/result.json"). Left: catches in each arm. The plain arm is above two thirds of its sessions, where 1.5 times its catches exceeds the #e3.gate.S sessions an arm has, so the gate could not be met. Right: the ratio and its 95% bootstrap interval, against D0’s threshold.],
) <fig-e3>

The ceiling explains the result (@fig-e3). The plain arm caught more than two thirds of its plant sessions, and at that rate the gate could not be met whatever the trust arm did: 1.5 × 2S/3 = S, where S is the number of plant sessions per arm, and no arm can catch more than S. The interval is a 95% bootstrap interval, from resampling the sessions. It is narrow because 11 of the 12 pairs tie at 6 of 6 in both arms; its upper end of 1.00 reflects that ceiling, not evidence about harder changes (report, opening and "Pairs").

Two further observations are described, not tested. Every miss in either arm is on one plant, which keeps a writable directory capability instead of narrowing it with `read_only()`; eleven of its twelve sessions approved it. And the trust arm had 3 false positives, the plain arm none: all three are Sol sessions on one clean change, objecting to write authority on a function that only reads, which v0’s trust diff lists as `fs.read, fs.write` as it lists every `&Fs` parameter (report, "What the data shows, and what it cannot").

What the pilot does not test matters as much: changes too large to read in full, human reviewers, and the trust diff as a mechanical record that does not depend on anyone reading it. Its items were changes of 10 to 120 lines to small single-file programs, reviewed by two models in sessions of one prompt, one reply and no tools (report, "Limits"). We draw one conclusion: this pilot gives no evidence that the trust diff helps model reviewers catch planted authority widening. We present the trust diff only as a summary of what a change may do. Work after the alpha is planned on a follow-up pilot with harder cases and on per-operation file authority in the diff.

== E1: the learning cost <sec-e1>

E1 asks what a model pays to write Tide instead of Rust (#repo("trials/e1-checked/gate-report.md")). Each arm had 30 tasks with 2 independent replicates, 60 trials per arm. One fresh writer session ran per trial, all `claude-opus-5-5`. The Tide arm received the frozen core reference and its companion, a longer set of worked examples; the Rust arm received neither. The protocol was frozen on 2026-10-31.

#figure(table(columns: 5, align: left, inset: 4pt, stroke: 0.4pt,
  table.header([*Measure*], [*Tide*], [*Rust*], [*Protocol §8 threshold*], [*Ratio (Tide/Rust)*]),
  [First-check pass F], [56/60], [59/60], [F(Tide) ≥ 0.8 × F(Rust)], [56/59 = 0.949153],
  [Final pass P], [58/60], [58/60], [P(Tide) ≥ P(Rust)], [1],
  [Iterations I, passing trials], [86/58 = 1.482759], [59/58 = 1.017241], [I(Tide) ≤ 1.5 × I(Rust)], [86/59 = 1.457627],
),
  caption: [E1’s three measures in each arm against the protocol's thresholds: all three are met on the point estimates. First-check pass is the share of trials that passed on their first check, final pass the share that passed at the end, and iterations the checks a passing trial needed.],
) <tab-e1>

All three thresholds are met on the point estimates (@tab-e1; report, "Measures and verdict"). The result is parity within the protocol's margins: writing Tide cost this model about as much as writing Rust. Rust passed first check more often, final pass tied, and Tide needed 1.457627 times Rust's iterations against a 1.5 limit. The information-only bootstrap interval for the iteration ratio, 1.29 to 1.64, crosses 1.5, so this is not a confidence-bound claim (report, "Information-only uncertainty").

Both arms worked near ceiling. Median sessions took 40.3 seconds for Tide and 30.6 seconds for Rust, so the trial mostly measures rapid, small, single-file solutions; it says little about difficult maintenance or larger programs. Every final failure in both arms was the same literal structure-check slip on one task. Both arms used one writer model; the author declined a second-model run, so this is not evidence of learnability independent of the model (report, "Interpretation, coverage and limitations"). The reviewed failure classification found three first-check failures in the Tide arm where a Rust habit leaked, and no toolchain defects, reference gaps or design gaps among the observed failures (report, "Failure classification").

== Benchmarks against plain Rust <sec-benchmarks>

The benchmark note (#repo("research/36-benchmarks.md")) measures what Tide's additions cost against plain Rust on four small programs, each written to isolate one cost, on one shared laptop. It is not one of D0’s planned performance gates, E4 or E5. Its main hedge comes first:

#quote(block: true)[*Stability: almost no series met the tool's criterion.* All 60 series of the main run ran to the 30-round cap with a CV above 1% (from 1.04% to 46.7%), so the run marks every one unstable (§1.6). [...] The medians and ratios below are descriptive observations from these unstable runs, not settled measurements.]

The CV, the coefficient of variation, is a series' standard deviation divided by its mean. The note reads ties and losses from paired rounds, in which the two arms run one after the other: an arm is slower when it is slower in at least 20 of 30 rounds. Its summary (the note's §1.1), with the run-time and build ratios drawn in @fig-bench:

- *Building.* A cold debug build takes 3.8 to 4.6 times plain Rust's time; a rebuild after a one-line edit takes 5.2 to 6.3 times incremental Rust's, because the driver has no incremental build. Compiler peak memory is 17% to 39% higher. Release binaries load 1.4 to 2.0 times the bytes of plain Rust built with Tide's profile flags, the compiler flags of Tide's build profile.
- *Contracts in release.* "No measurable cost in this run": a tie, 0.99x.
- *Contracts in debug.* The plain debug build checks only the cheap clauses; the program's two sampled sortedness clauses run only in test builds, so these numbers exclude them. At the timed size of 1,200,000, a tie (1.05x). At 200,000, in a separate run outside a quiet window (a period with the machine otherwise idle), 1.19x, slower in all 30 rounds: a loss.
- *Contracts in a test build.* In that same separate run at 200,000, the debug test build, which also checks the sampled clauses, took 7.7x the plain Tide debug build and 9.2x Rust's debug build. Its sampled clauses scan a vector on every insert, which the note calls unrepresentative of typical contract cost, and the test build also differs by the test harness and its `cfg(test)` code, which the run does not separate.
- *Capability file I/O.* 1.04x by medians, slower in 26 of 30 rounds, paired median ratio 1.07x: "a small loss, of a few percent".
- *Capability TCP.* 1.18x, slower in all 30 rounds.
- *Arithmetic-heavy tasks.* 2.29x plain release Rust, of which 1.74x comes from Tide's profile flags alone; the run does not separate which flag costs what.
- *Simulator.* One seed under the simulator takes 12.8 times the real run in release builds.

#figure(
  fig("bench"),
  kind: image,
  caption: [Tide's time as a multiple of plain Rust's (medians divided), drawn from the main run's records, #repo(bench-path.slice(1)). Above 1× Tide is slower. A row is a loss when Tide was slower in at least #bench.loss_rounds of the paired rounds, otherwise a tie: the counts are made from the records with the note's rule (the note's §1). The release row for `tasks` compares with Rust built with Tide's flags, so it shows the #fmt2(bench.tasks_twin_ratio)× between the twins, not the #fmt2(bench.tasks_plain_ratio)× against plain release Rust.],
) <fig-bench>

The note prices Tide's additions and states its verdict plainly: "Every comparison above is a loss for Tide or a tie. None is a win." Its other limits include different linkers for the two arms and a Tide std built in each arm's profile, so the debug comparisons also compare an unoptimized std with Rust's optimized one (the note's §3). A fast edit loop remains a design target (D12), not a measurement.

= Related work <sec-related>

*Rust.* Tide depends on Rust [12] and is meant to complement it. Every Tide program is a valid Rust 2024 crate compiled by an unmodified rustc (D18), and Tide's memory safety is Rust's. Tide is a restricted subset with its own standard library, not a fork or a replacement. Its capability claims inherit rustc's soundness bugs (@sec-admission).

*Faster Rust toolchains.* lust [13] and Blaze aim at the edit loop, which Tide has not yet built (D12) (#repo("research/28-positioning-and-name.md") §2). lust checks a subset of valid Rust with its own front end, keeping rustc as the oracle; it has no authority bounds or replay. Blaze is a second Rust compiler with a warm daemon; it has partial overlap with Tide's thesis through attenuable `bstd` capabilities and a simulator, but does not admit a whole program against a capability standard library. This alpha does not benchmark either.

*Capabilities and effects.* Object-capability languages and libraries, in which holding a reference is the only route to authority, among them Joe-E, Pony, Wyvern, Austral and the `cap-std` crate, and runtime permission models such as Deno's and WASI's, are surveyed in #repo("research/07-effects-capabilities.md"). Effect systems such as Koka and Effekt track effects in types; Tide's design rejected effect rows, effect variables and, in v1, user-defined handlers, in favor of plain capability values (D7). Effect [14], a TypeScript library, is the closest prior art for requirements in types: its `Effect<A, E, R>` type carries requirements, services are provided through layers, and fibers', its lightweight threads', lifetimes are tied to their parent. Its `R` is a convention, though: nothing stops a dependency from calling `fetch` or reading the clock directly. Tide enforces the equivalent at admission, and by design in dependencies too (v0 admits a single crate), and its structured tasks forbid the detached fibers that Effect's `forkDaemon` creates (D8).

*Deterministic simulation.* FoundationDB-style simulation testing, TigerBeetle's VOPR, and Rust tools such as loom, Shuttle and madsim test concurrent code under controlled schedules (#repo("research/16-deterministic-simulation.md")). rr, the closest tool to Tide's replay, records a run of an unmodified Linux program and replays it deterministically under gdb; it needs the CPU's hardware performance counters and finds a failing run by recording runs, where Tide fixes the schedule from a seed and searches seeds, and a seed is a number that reruns on the same source (#repo("docs/debugging.md"), "rr compared"). Library-level simulation in Rust is limited by dependencies that reach ambient time, randomness and threads (decision record, "Evidence That Shaped The Design"). Tide's difference is that admission makes those reaches impossible, so simulation covers the whole program wherever the simulator has a model for every effect reached. Rivet [15], a Rust-core actor runtime whose workflows resume after a crash by skipping finished steps, does not record or replay nondeterminism; it is complementary, and with Temporal it is prior art for any future resume-after-crash feature in Tide. Independently, plusthree, an MIT-licensed Rust port of XState v6 statecharts, keeps its `no_std` (built without Rust's standard library) kernel free of a clock: the host fires timers, so tests can run timers instantly [19].

*Parallelism and confluence.* Bend and its HVM runtime evaluate pure programs in parallel by construction. Tide takes from them the separation of specification, checker and evidence, and does not take their runtime: D9’s parallel regions (@sec-concurrency) give schedule-independent successful results by admission, as a checked contract rather than a proved confluence theorem (#repo("research/45-bend-hvm-ideas-for-tide.md") §3 and §4).

*Verification.* Verus, Prusti, Creusot and Kani verify Rust programs (#repo("research/04-verification.md")). Tide borrows their contract names and spellings (D10) but proves nothing in v0. Bend 2 [16] targets proved, massively parallel pure code; Tide plans to call it for such kernels rather than compete (D16, Later tier). As context, Anthropic and Theorem reported in October 2026 on models carrying out formal verification at a much larger scale than before [11]. That work is independent of Tide; it is one sign that agents can do large amounts of checked work, and Tide is not formally verified.

#figure(
  fig("related"),
  kind: image,
  caption: [Where Tide sits: which neighbouring tools bound authority in signatures, enforce it in dependencies, search schedules from a seed, or replay a run exactly, as this section describes them.],
) <fig-related>

= Limitations <sec-limitations>

- *Linux on x86\_64 only.* The driver links and runs tests only on x86\_64 Linux. Elsewhere it builds with the sandbox compiled out and its run modes are unavailable (#repo("docs/owner-guide/08-sandbox.md")).
- *A single crate.* The driver admits one crate, or one suite paired with one library. Packages and dependencies are planned. Until they exist, the guarantees cover a closed world that grows weaker as programs reach more of the outside world, weakness W1 of the project's adversarial design review (#repo("reviews/2026-11-15-achilles-opus.md")).
- *Replay reproduces the model, not the program* (W7). The simulator does not model CPU time, memory exhaustion, stack depth, parallel speed-up, signals, real child processes, or kernel behavior beyond its models. HTTP peers and child processes are simulated only as declared models, with no wire parsing, TLS validation or real child; fault injection, crashable nodes, and `tide sim` for whole programs are planned, not built (#repo("docs/owner-guide/10-v0-and-planned.md")).
- *Capability safety rests on rustc's soundness* (W3, `fx-6xi2`). Three admitted routes are closed; rustc has other open `I-unsound` issues; until a toolchain pin bump is audited, `core::*` globs such as `core::mem::*` admit whatever a later rustc adds under them; and the red teams' records list what they tried and what they left open (@sec-redteams).
- *Narrowing has stated limits.* A network `only` bounds the hosts a program connects to, not listening until `no_bind()`; an allowed program runs with its own full authority (@sec-capabilities).
- *The sandbox is partial.* The level `full` requires a Linux kernel with Landlock interface version (ABI) 6 or later and seccomp. RUN-3 does not confine metadata reads such as `stat` and `access`. A test binary run directly gets no sandbox, the build has none yet (RUN-2), and deployed programs get kernel enforcement only in v1 (D7).
- *E1 is near ceiling and uses one writer model.* Both arms passed almost every task, on small single-file programs, with `claude-opus-5-5` alone.
- *E3’s plain arm sat at ceiling.* Plain review caught almost every planted change, so the pilot could not meet D0’s 1.5x threshold and left no room to show a benefit.
- *The benchmarks measure costs on unstable runs*, on four small programs on one shared machine (@sec-benchmarks).
- *The `tide` command is planned.* The driver's flags are its precursors; typed holes and machine-readable fix plans (D14) are not delivered, which the E1 report counts against the Tide arm.

#figure(
  fig("sim-model"),
  kind: image,
  caption: [Replay reproduces the model: for each area, what Tide v0’s simulator models and what it leaves out, so a replayed run says nothing about the right-hand column.],
) <fig-sim-model>

= Roadmap <sec-roadmap>

The decision record's tier table ("Tiers") owns what comes next. Planned for v1: proofs of contracts (`prove` mode), exercised-authority tracking, the evented runtime, direct import of Rust crates behind capability wrappers, production record and replay (D17), and kernel enforcement of authority for deployed binaries (D7). Later: the Bend tandem (D16), automatic parallelization, and signals and pseudo-terminals (the emulated terminals that interactive programs run in) for child processes. Planned evidence includes a follow-up to E3 on harder cases, E2 (productivity on multi-file tasks), and E4 to E6, D0’s remaining gates. The fast edit loop of D12 needs Tide's own incremental front end, which is not built.

#figure(
  fig("tiers"),
  kind: image,
  caption: [The decision record's tiers: selected items of the v0 core, those not built yet marked planned, then selected items of v1 and later.],
) <fig-tiers>

One research pass points toward a direction rather than a feature: account authority on Solana, a blockchain whose on-chain programs are commonly written in Rust, expressed as capabilities (#repo("research/43-solana-account-capabilities.md")). In a small pre-registered paper exercise (not a Tide feature; nothing was compiled or run), a proposed Solana account-capability rule, given as declared policy the account check that each later public fix added, rejected the vulnerable code and admitted the fixed code for two of five public, already-fixed Rust exploits (Solend 2021, Wormhole 2022); Cashio's fix could be judged on its vulnerable side only, two cases had no public code, and Anchor and the Solana SDK can already express the same checks. (Anchor is a widely used framework for Solana programs, and the SDK is Solana's own development kit.)

= How it was built <sec-built>

Tide is agent-produced under a written governance (#repo("GOVERNANCE.md")). Its primary rule: "No production Tide code should originate from direct human coding." Human-written production code is forbidden by default and must be explicitly marked if it occurs as an exception. The project is AI-led and human-steered. Agents design the language and its architecture, write the specifications, implement compiler components, write tooling and tests, apply fixes, and review and validate each change; the owner sets scope, priorities and direction and makes the owner calls.

No change lands on `main` without an AI supervisor pass. Review runs in sequence, each reviewer read-only on a frozen candidate: first a Codex pass (`gpt-6.1-sol`, reasoning effort high), whose findings the author fixes or rebuts until the supervisor reports the tree clean, then an Opus review on that tree. When the provider refuses the Codex pass, an Opus stand-in review takes its place and records the refusal. Planning-only changes (filings in the issue tracker) skip the Codex pass, and a protocol-import update that only regenerates `AGENTS.md` and `.agent-protocols.json` from a pinned source commit skips both reviews when its source-backed check passes. The human maintainer remains final merge authority. Each review is recorded in the pull request's discussion or, for a direct commit, as a file under #repo("reviews/"). Work is tracked as issues, called beads, in a committed ledger, and the design is a decision record that owns the language decisions.

#figure(
  fig("merge"),
  kind: image,
  caption: [How a change lands: a Codex supervisor pass until it reports the candidate clean, then an Opus review on that tree; the reviewer that found each finding re-checks only its fix.],
) <fig-merge>

This is why we call Tide AI-welcome: not because agents wrote it, but because what they wrote is checkable. Changes are agent-written unless explicitly marked otherwise, reviewed under that merge rule, and backed by proof that can be rerun: a demo, a conformance corpus, and trial reports with their supporting records. Not every raw record is in the repository: E1’s raw writer and broker streams and its grading logs are kept outside it, and its report says so. The same habit shaped this paper. Both trials ran under protocols frozen before the first session, an order our own records show, and both reports, the negative one included, are in the repository.

#heading(level: 1, numbering: none)[Decision IDs cited] <sec-decisions>

From the decision record (#repo("design/01-design.md")):
D0, the validation program with gates (E1 to E6);
D1, Rust spelling wherever the meaning is the same;
D4, traits with a hard power ceiling;
D5, errors as values, as in Rust;
D6, no user macros;
D7, authority as capability values, no ambient authority;
D8, structured tasks, no `async`;
D9, deterministic parallelism with defined grouping;
D10, contracts;
D11, the trust surface;
D12, compilation and backends, including the fast edit loop;
D14, the toolchain contract for agents, including the core reference's budget;
D15, deterministic simulation;
D16, the Bend tandem;
D17, record, replay and authority policies;
D18, a checked dialect of Rust.

#heading(level: 1, numbering: none)[References]

In-repository reports and design records, pinned to the release tag:

1. Tide decision record: #repo("design/01-design.md")\; first principles: #repo("design/00-first-principles.md")\; core reference: #repo("design/reference.md")\; authority and concurrency chapter: #repo("design/03-authority-concurrency.md").
2. Replay and trust-diff demonstration: #repo("demo/README.md").
3. Red teams: admission and the RUN-3 sandbox, #repo("reviews/2026-11-17-red-team-opus.md")\; `Proc::command`, #repo("reviews/fx-ayh2.1-redteam-opus.md"), #repo("reviews/fx-ayh2.1-redteam-recheck-opus.md"), #repo("reviews/fx-ayh2.1-redteam-recheck2-opus.md")\; `Net::get`, #repo("reviews/fx-ff13-redteam-codex.md"), #repo("reviews/fx-ff13-redteam-recheck-codex.md")\; `Net::resolve`, #repo("reviews/fx-6d50-redteam-codex.md")\; `Rand`, #repo("reviews/fx-qlek.20-s1-redteam-codex.md"). Implementation reviews: #repo("reviews/fx-ayh2.1-s2-r1-codex.md") and the other `fx-ayh2.1-s2`, `fx-ff13-s2`, `fx-6d50-s2` and `fx-qlek.20-s2` files in #repo("reviews/"). Designs: #repo("research/40-proc-command-v0.md"), #repo("research/42-net-get-v0.md"), #repo("research/48-net-resolve-v0.md"), #repo("research/53-rand-v0.md").
4. rustc's soundness bugs and what Tide closed: #repo("docs/owner-guide/03-soundness-bugs.md")\; the RUN-3 sandbox: #repo("docs/owner-guide/08-sandbox.md").
5. E3 pilot report: #repo("trials/e3-pilot/report.md").
6. E1 gate report: #repo("trials/e1-checked/gate-report.md").
7. Benchmarks against plain Rust: #repo("research/36-benchmarks.md").
8. External catch, re-test of chrono_anchor: #repo("research/49-fx-zu4f.15-chrono-anchor-retest.md"). Search rounds: #repo("research/47-malware-rescreen.md"), #repo("research/37-external-admission-search.md"), #repo("research/38-external-concurrency-search.md"), #repo("research/39-std-concurrency-search.md"), #repo("research/41-qin-dataset-search.md"), #repo("research/44-fx-zu4f.10-concurrency-round.md"), #repo("research/46-fx-zu4f.11-agent-marker-round.md"), #repo("research/52-fx-zu4f.12-crypto-crates-round.md"), #repo("research/51-fx-zu4f.13-landed-capability-rescreen.md"). Case studies: #repo("examples/case-studies/eon-discovery-race/README.md"), #repo("examples/case-studies/eon-slow-close/README.md") and #repo("examples/case-studies/hyper-ambient-runtime/README.md").
9. Adversarial review (weaknesses W1 to W8): #repo("reviews/2026-11-15-achilles-opus.md")\; round-1 agent-usability review: #repo("reviews/round1-agent-usability.md").
10. Research notes: #repo("research/04-verification.md"), #repo("research/05-llm-code-empirics.md"), #repo("research/07-effects-capabilities.md"), #repo("research/16-deterministic-simulation.md"), #repo("research/28-positioning-and-name.md"), #repo("research/43-solana-account-capabilities.md"), #repo("research/45-bend-hvm-ideas-for-tide.md").

External:

11. Anthropic and Theorem, "LLMs are making formal software verification feasible at a much larger scale", 2026-10-22. #link("https://www.anthropic.com/research/formal-verification")
12. The Rust programming language and rustc. #link("https://www.rust-lang.org/"); soundness issues rust-lang/rust\#25860 and \#57893: #link("https://github.com/rust-lang/rust/issues/25860"), #link("https://github.com/rust-lang/rust/issues/57893").
13. lust: #link("https://github.com/t3dotgg/lust-lang").
14. Effect: #link("https://github.com/Effect-TS/effect").
15. Rivet: #link("https://github.com/rivet-dev/rivet").
16. Bend 2: #link("https://github.com/bendlang/bend").
17. Eon: #link("https://github.com/Yazelix/eon").
18. hyper: #link("https://github.com/hyperium/hyper"); issue \#2808: #link("https://github.com/hyperium/hyper/issues/2808").
19. plusthree: #link("https://tangled.org/rockorager.dev/plusthree").
