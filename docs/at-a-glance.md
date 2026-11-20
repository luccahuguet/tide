## Tide at a glance

Tide is a subset of Rust for code that AI agents write and people have to
trust.

1. **It's Rust.** Every Tide program is a valid Rust 2024 crate, compiled by
   an unmodified rustc pinned to one nightly. [No fork](/docs/tide-or-rust.md).
2. **A checker runs first.** [`tide-admit`](/docs/driver.md) admits or
   rejects each crate before rustc compiles it. Its rules' rejections carry
   a `TD` code, such as `TD0501`, that says why. What rustc rejects, such as
   a call to a function Tide's std lacks, gets rustc's errors.
3. **Authority shows in the signature.** Code reaches the outside world through
   [capabilities](/design/reference.md#61-the-rule-and-the-capability-set) it is
   handed (`&Fs`, `&Clock`, `&Env`, `&Net`, `&Proc`, `&Rand`), so a function's
   parameters bound what it can do. The claim is no accidental ambient authority
   in admitted safe code, apart from printing and outside the `unsafe` grant, up
   to [rustc's soundness bugs](/README.md#forge-routes).
4. **Authority can be narrowed.** `fs.dir(path, mode)` confines files to a
   directory, and `only([...])` limits hosts, programs or variables
   ([limits](/docs/tide-or-rust.md#what-tide-gives-you)). A network `only`
   limits outbound peers, and listening remains possible unless the
   capability is narrowed with `no_bind()`. An allowed shell allows anything.
5. **Tests replay.** A `#[sim_test]` runs under a seeded simulator, and
   `tide-admit` replays a failing run on the same machine with identical
   traces. Replay reproduces the
   [simulator's model](/README.md#simulator-fidelity), not the machine.
6. **Any change can get a trust diff.** `tide-admit --tide-diff` summarizes what
   a change may do: public items whose declared authority widened or narrowed,
   and `unsafe` sites added or removed. The
   [E3 pilot](/README.md#e3-does-the-trust-diff-help-reviewers-pre-registered-pilot-negative)
   found no evidence that it helps reviewers.
7. **Built for agents.** The core reference that teaches Tide to a model fits
   a budget of
   [25,000 tokens](/design/01-design.md#d14-toolchain-contract-for-agents).
8. **Known limits.** An alpha, x86_64 Linux only. No async, by design, and safe
   code sees one memory order ([costs](/docs/tide-or-rust.md#what-it-costs)).
   Admission closes [three known routes](/README.md#forge-routes) through
   rustc's soundness bugs, and the others stay open.
9. **Evidence, negatives included.** One catch, on a later re-test of one
   screened crate,
   [not first-time and not a rate](/README.md#a-malicious-crate-rejected-as-written-admitted-only-with-its-authority-declared),
   its registration before the fix not independently proven
   ([research/49](/research/49-fx-zu4f.15-chrono-anchor-retest.md)). The malware
   screen caught 0 of 87. For the pilot and four of the searches, the order of
   method and results rests on our own records, and the others' timestamps show
   only when their method files existed. A possible security defect appears
   publicly only as a count. The [status page](https://tidelang.com/status/)
   lists every round with its counts and qualifier.
