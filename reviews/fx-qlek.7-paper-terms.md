# Paper terms pass (fx-qlek.7, stage S2a)

Checked on `paper/tide-v0.typ` at the S2a commit, in reading order: title,
abstract, disclosure and status notes, sections 1 to 8, figure and table
captions, the decision-ID list and the references. Rule (Lucca, relayed
2026-11-18 19:51): every technical term, Tide name and acronym is defined in
plain words at its first appearance; later uses rely on that definition.
"Abstract ¶2" means the abstract's second paragraph; section numbers are the
PDF's. Figures marked "to come" are placeholders whose captions are final
text; S2b draws them and must keep each caption's definitions.

MISSING entries: none. Terms the title uses are defined in the abstract's
first paragraph, which is the first text after the title.

| Term | First use | Definition there |
| --- | --- | --- |
| Authority | Title slogan, Abstract ¶1 | "the power to observe or change the world outside the program" (same paragraph, sentence 4) |
| Capability, `&Fs` | Abstract ¶1, sentence 2 | "a file capability, a value of type `&Fs`, as a parameter"; capability values are what a function must be handed to reach files, network, clock, environment, programs, randomness |
| Capability-safe | Title; Abstract ¶1 | authority "reachable only through such capability values, apart from write-only diagnostic output (§2.2), so a function's signature bounds what it may do"; guards against accident, not a deliberate soundness-bug exploit ("much as Rust's memory safety holds only up to those bugs") |
| Soundness bug | Abstract ¶1 | "cases where the compiler accepts code that breaks its own type rules" |
| Replayable | Title; Abstract ¶1 | a test runs under a seeded simulator and "a failing run can be rerun exactly from its recorded manifest" |
| Seed | Abstract ¶1 | "one number ... decides every scheduling choice" |
| Simulator | Abstract ¶1 | the run environment in which the seed decides every scheduling choice |
| Manifest | Abstract ¶1 | "a file that records the run's seed, sources and inputs" |
| Rust 2024 | Abstract ¶1 | "Rust, in its 2024 edition" |
| Agent-written code | Title | Abstract ¶1: "code that artificial-intelligence (AI) models write and people review" |
| AI | Abstract ¶1 | "artificial-intelligence (AI)" |
| std | Abstract ¶1 | "Tide's own standard library (std)" |
| Tide v0, alpha | Abstract ¶1 | "Tide v0, the alpha release" |
| rustc | Abstract ¶1 | "rustc, the Rust compiler" |
| Crate | Abstract ¶1 | "Rust's unit of compilation" |
| Admission, admission driver | Abstract ¶1 | the driver "checks each crate against Tide's rules and compiles it against Tide's own standard library before rustc writes any artifact" |
| Lost update | Abstract ¶2 | "one concurrent write silently overwriting another" |
| Trace | Abstract ¶2 | "the logs of what the run did" |
| Red team | Abstract ¶2 | "reviews that try to break a security claim"; fuller in 4.3 ("an AI model, given a security claim of Tide's, tries to break it and records every route it tried") |
| Kernel sandbox, test sandbox | Abstract ¶2 | "the kernel sandbox that confines test runs"; mechanism in §3 (RUN-3) |
| Pre-registered, registration | Abstract ¶2 | "their method and counting rule are recorded before the search", with a pointer to §4.4 for what the records prove about the order |
| RustSec | Abstract ¶2 | "Rust's public security-advisory database"; again in 4.4.1 |
| Malicious crate | Abstract ¶2 (in "malicious crate from RustSec"); 4.4.1 | plain words in the abstract; 4.4.1: "packages published to attack the people who use them" |
| Caught rule | Abstract ¶2 | "admission rejects the faithful rewrite ... and admits a version handed the authority it uses"; legs 1 and 2 in 4.4.1 |
| Port | Abstract ¶2 | "the faithful rewrite of the malicious code into Tide, its port" |
| Screen | Abstract ¶2 | "an earlier screen of all 87 malicious-crate advisories" (a pass that applies the caught rule to every advisory) |
| Anchored | Abstract ¶2 | "A registration is anchored when its hash is timestamped in the Bitcoin blockchain; §4.4.1 explains" |
| Trust diff | Abstract ¶2 | "a summary of the authority each exported item gains or loses between two revisions" |
| Learnability trial, paired | Abstract ¶2 | "the same 30 small tasks written in each language" |
| Protocol | Abstract ¶2 | "its protocol, the plan fixed before the trial" |
| Contracts | Abstract ¶2 | "checked preconditions and postconditions"; full in 2.4 |
| Eon | Abstract ¶2 | "the author's terminal workspace"; §4.4.2: "the author's own terminal workspace" |
| hyper | Abstract ¶2 | "a library for HTTP" |
| HTTP | Abstract ¶2 | "HTTP, the web's request protocol" |
| Property | Abstract ¶2 | "the condition the code must keep" |
| Interleaving | Abstract ¶2 | "the order in which concurrent steps run" |
| tokio | Abstract ¶2 | "tokio, Rust's most used runtime for asynchronous code"; again in §4.4.2 ("the most used async runtime for Rust") |
| Point estimates | Abstract ¶2 | "the measured values, not their intervals" |
| Near ceiling | Abstract ¶2 | "both arms passed almost every task" |
| Codex CLI, GPT Sol, Claude Opus, Fable | Disclosure | named as the AI agents and models that produced the work; "OpenAI's Codex command-line tool (CLI)" |
| AI supervisor pass, merge rule | Disclosure | "a change lands only after an AI supervisor pass, normally a pass by GPT Sol ... and then a Claude Opus review"; §8 gives the rule |
| D0 to D18 | Status | "IDs in the project's decision record"; a list at the end gives each cited ID a line |
| Edit loop | §1 ¶2 | "the cycle of editing code and checking it" |
| Locality, convergence | §1 ¶2 | given in parentheses there |
| RustAssistant | §1 ¶3 | "a tool in which models fix Rust compile errors" |
| Lints | §1 ¶4 | "extra compiler checks", in "Libraries and lints (extra compiler checks) that keep Rust's std" |
| Ambient authority | §1 ¶4 | "Rust's authority is ambient, reachable by any code without being handed to it" |
| Data race | §1 ¶4 | "two threads touching the same memory without synchronization, at least one writing" |
| rr, record and replay | §1 ¶4 | "Record-and-replay tools such as rr can record one real run and replay it" |
| User macros, `async`/`await` | §2.1 | Rust features Tide removes (D6, D8); `async`/`await` is "Rust's syntax for asynchronous functions"; user macros are named for Rust readers |
| Token | §2.1 | "the units a language model reads text in" |
| Core reference | §2.1 | "written for agents to load whole", with its 25,000-token budget |
| `Fs`, `Net`, `Clock`, `Env`, `Proc`, `Tasks` | §2.2 | each named with what it reaches ("`Env` for the environment (the variables and arguments a program starts with)", "`Proc` for other programs", "`Tasks` for concurrency") |
| `Root` | §2.2 | "the capability that holds all the others" |
| Simulated root (sim root) | §2.2; §2.3 | "a test can take a simulated root instead"; §2.3: `Root::sim(seed)` |
| `only([...])`, narrowing | §2.2 | "keeps a network capability to listed hosts, a process capability to listed programs and an environment capability to listed variables" |
| `no_bind()` | §2.2 | "accepting incoming connections stays possible until `no_bind()` removes it" |
| TCP | §2.2 | "TCP connections (the internet's reliable byte streams)" |
| TLS | §2.2 | "TLS (which encrypts the connection)" |
| UDP | §2.2 | "UDP (the internet's connectionless datagrams)", planned |
| `Rand`, randomness draws | §2.2 | "`Rand`, the randomness capability, currently exists only as a capability identity, with no draw methods"; draws planned |
| `Proc::exit` | §2.2 | "which ends the program", planned |
| Shell | §2.2 | "a program that runs other programs" |
| Sink | §2.2 | "Write-only diagnostic output ... goes to sinks, which are not capabilities and cannot read anything back" |
| Effect rows, effect variables | §2.2 | "type-level lists of the effects a function may perform" |
| Task | §2.3 | "Tide's units of concurrent work" |
| Scope | §2.3 | "blocks that wait for every task spawned in them" |
| Level-triggered cancellation | §2.3 | "every later wait in a cancelled scope sees it" |
| Virtual time | §2.3 | "the simulator's own clock" |
| `Pure` | §2.3 | "a marker for types from which no capability, synchronization handle or foreign handle is reachable" |
| `Send`, `Sync` | §2.3 | "Rust's ... thread-safety markers" |
| Parallel region; `join`, `par_map`, `par_for` | §2.3 | "D9's parallel regions give schedule-independent successful results without exploring task schedules"; the three are its library functions, whose closures and elements must be `Pure` |
| Sequential equivalence | §2.3 | "when no branch panics, a region returns what its sequential version returns" |
| Confluence theorem | §2.3 | "a proof that every schedule gives the same result" |
| Trust hole | §2.3 | "a place where admitted code may do what admission cannot check, such as `unsafe` code admitted under an explicit grant"; again in §2.6 |
| `unsafe` grant | §2.3 | "`unsafe` code admitted under an explicit grant" |
| Checked and sampled clauses | §2.4 | each defined in the sentence that introduces it |
| Property test | §2.4 | "which check a stated property on generated inputs" |
| Dialect | §2.5 | "Every Tide program is a Rust 2024 crate that a pinned, unmodified rustc nightly (rustc's unstable daily build) compiles against Tide's standard library" |
| Nightly | §2.5 | "rustc nightly (rustc's unstable daily build)" |
| `core`, `alloc` | §2.5 | "Rust's lower-level libraries" |
| TD codes | §2.5 | "the driver's own error codes, such as `TD0206`, each with a message that says why" (first in the body; the abstract does not use the term) |
| Allowlist | §2.5 | "an admission allowlist over ..." (the items admission accepts) |
| `TD0501` | Fig. 3 caption (fig-diagnostic) | "a call to an ambient standard-library function that needs a capability" |
| Trust surface, declared authority | §2.6 | "each public item's signature and declared authority"; exercised authority contrasted in the same paragraph |
| Cargo workspace | §3 | "one Cargo (Rust's build tool) workspace" |
| `rustc_private` driver | §3 | "one that links rustc's internal crates" |
| `tide-admit`, `tide-std`, `tide-conformance` | §3 | each bullet defines its crate |
| RUN-3 | §3 | "a kernel sandbox, which the design rule RUN-3 specifies" |
| Landlock, seccomp | §3 | "Landlock, which limits the files a process can open, and seccomp, which limits its system calls" |
| RUN-2 | §3 | "the build itself (rustc and the linker) has none yet, the planned rule RUN-2" |
| Conformance corpus | §3 | "every normative rule ... carries an ID and is traced to cases, small programs with expected verdicts" |
| `rust-gdb` | §4.1 | "the Rust wrapper of the GNU debugger" |
| Tracker IDs (`fx-…`) | §4.2 | "IDs of this form name issues in the project's tracker" |
| `transmute` | §4.2 | "a function that reinterprets one type's bytes as another's" |
| Blanket impl | §4.2 | "one that implements a trait for every type" |
| Projection form | §4.2 | "an impl for another trait's associated type" |
| `I-unsound` | §4.2 | "its label for soundness bugs" |
| Unforgeable | §4.2 ("When capabilities would be unforgeable") | admitted safe code could forge no capability; the paragraph states the condition and that v0 does not claim it |
| Finding, blocking | §4.3, Table 1 caption | "a defect or an overstated claim it reported, blocking when it had to be fixed before the change landed" (the table and caption precede the first prose use) |
| Tried (routes, attack classes) | Table 1 caption | "counts the routes or attack classes a red team recorded" |
| Controls | §4.3 | unsandboxed or accepted runs "so that a denial could be told from a probe that never worked" |
| I/O | §4.3 | "an input/output (I/O) method" |
| Glob | §4.3 | "the allowed glob (a `::*` import of every item in a module) over `std::path`"; again in §6 ("`core::*` globs") |
| Loader variables, `LD_PRELOAD` | §4.3 | "the C library's dangerous loader variables" (`GCONV_PATH` example in the implementation paragraph) |
| Leg 1, leg 2 | §4.4.1 | each defined in full |
| `TD0602` | §4.4.1 | "on two `thread::spawn` calls where Tide requires tasks in a scope" |
| Trust artifact | §4.4.1 | "the file the driver writes listing each item's declared authority" |
| `proc` grant | §4.4.1 | the trust artifact's declared "process authority"; the scope paragraph says "the `proc` grant says the code starts programs, not what they do" |
| OpenTimestamps (OTS) | §4.4.1 | "anchors a file's hash in a Bitcoin block and so shows that the file existed by that block's time" |
| `E0283` | §4.4.1 | rustc's error for broken type inference on "an unannotated `tasks.scope(..)?`" |
| loom, Shuttle | §4.4.2 | "a scheduler such as loom or Shuttle" |
| tokio | §4.4.2 (slow close) | "the most used async runtime for Rust" |
| `E0433` | §4.4.2 (hyper) | "an unresolved path" |
| Eligible | §4.4.3 | "has a public fix and an observed symptom, is concurrency-shaped, can be written faithfully in Tide v0, and lies within what the simulator explores" |
| Screened | Table 2 caption | "what the round judged against its criteria" |
| Self-attested, OTS-anchored | Table 2 caption | both defined in full |
| PLDI 2020 dataset (Qin et al.) | Table 2 | "Qin et al.'s dataset of Rust concurrency bugs (PLDI 2020, a programming-languages conference)" |
| OS | §2.3 | "operating-system (OS) threads" |
| CPU | §4.4.2 | written out: "processor stress" (I/O is defined in §4.3) |
| E3 | §4.5 | "asks whether reviewers catch more planted authority-widening changes with the trust diff than with the plain code diff"; heading names it a pre-registered pilot |
| Plant, planted change | §4.5 | "planted authority-widening changes" (changes inserted on purpose for reviewers to catch) |
| Bootstrap interval | §4.5 | "a 95% bootstrap interval, from resampling the sessions" (the verbatim E3 quote uses "95% interval" first and is kept unchanged) |
| E1 | §4.6 | "asks what a model pays to write Tide instead of Rust" |
| Companion | §4.6 | "a longer set of worked examples" |
| First-check pass, final pass, iterations | Table 3 caption | each defined |
| E4, E5 | §4.7 | "D0's planned performance gates" |
| CV | §4.7 (quoted hedge, then next sentence) | "a series' standard deviation divided by its mean" (the quote is verbatim; the definition follows it at once) |
| Paired rounds; tie, loss | §4.7 | "the two arms run one after the other"; "an arm is slower when it is slower in at least 20 of 30 rounds" |
| Tide's profile flags | §4.7 | "the compiler flags of Tide's build profile" |
| Quiet window | §4.7 | "a period with the machine otherwise idle" |
| lust, Blaze | §5 | each described in the paragraph |
| Object-capability | §5 | "in which holding a reference is the only route to authority" |
| Koka, Effekt | §5 | "Effect systems ... track effects in types" |
| Effect (TypeScript), fibers | §5 | library described; fibers: "its lightweight threads" |
| FoundationDB simulation, VOPR, madsim | §5 | "test concurrent code under controlled schedules" |
| Bend, HVM | §5 | "evaluate pure programs in parallel by construction" |
| `no_std` | §5 | "`no_std` (built without Rust's standard library)" |
| Rivet, Temporal, plusthree, XState | §5 | Rivet and plusthree described; Temporal named with Rivet as resume-after-crash prior art; XState named as what plusthree ports |
| Verus, Prusti, Creusot, Kani | §5 | "verify Rust programs" |
| W1, W3, W7 | §6 | "weakness W1 of the project's adversarial design review"; W3 and W7 are later weaknesses of the same review, each named by its bullet heading |
| Landlock ABI 6 | §6 | "Landlock interface version (ABI) 6" |
| `prove` mode, evented runtime | §7 | named as v1 plans from the tier table |
| E2, E6 | §7 | "E2 (productivity on multi-file tasks)"; "E4 to E6, D0's remaining gates" |
| Solana | §7 | "a blockchain whose on-chain programs are commonly written in Rust" |
| Anchor, Solana SDK | §7 | "Anchor is a widely used framework for Solana programs, and the SDK is Solana's own development kit" |
| Beads | §8 | "Work is tracked as issues, called beads" |
| Frozen candidate | §8 | "each reviewer read-only on a frozen candidate" (the change as reviewed, unchanged during the review) |
| AI-welcome | §8 | "not because agents wrote it, but because what they wrote is checkable" |

Terms the comment named that the paper does not use: SBF and PDA (the
Solana paragraph uses only the recorded launch sentence, which needs
neither) and "sim root" as a phrase (the paper says "simulated root").
