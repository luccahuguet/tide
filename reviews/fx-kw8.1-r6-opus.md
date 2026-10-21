# Tide review, fx-kw8.1 round 6 (Opus): converge round

Candidate: HEAD 4c2b2e2 on 187a22b (worktree `/home/lucca/pjs/lab/tide-fx-kw8.1`, branch
fx-kw8.1). This round reviewed `git diff 187a22b HEAD` in full and looked hardest at round 5's
three edits: `use_ty` checking every fn item's where-clauses, the literal leading `*` in policy
globs, and `entry_point`'s `Termination` check. It then ran a bounded hunt for routes and false
rejections. Fixes are in the working tree, not committed. Probes are in this scratchpad's `t6/`
and run through `adm6.sh` (round 5's `adm5.sh`, writing to `t6/`).

## Verdict

fixed

Only comment text changed. No admission hole or false rejection in a Tide build was found.

## Findings and fixes

**N1. A module doc line was 103 characters (nit).** In `policy.rs`, round 5's glob sentence
pushed line 9 past the 100-column width that every other line keeps. rustfmt does not reflow
comments, so its check passed anyway. *Fix:* reflowed the sentence. The text is unchanged.

**N2. Two comments in `crates.rs` described less than the code does (nit).** The doc on
`Role::TideStd` said "unless that is Rust's own std". The comment above the reset said the
same. Since round 1's B1 fix, the reset also clears `tide_std` when the injected `std` is `core`
or `alloc`, which an `--extern std=` naming them would cause. *Fix:* both comments now name
all three crates.

## Not changed

- **Under Rust's own std, a binary's plain `fn main() {}` now gets FX0520.** The message is
  `impl std::process::Termination for ()` is not admitted (`t6/r1.rs`, run without
  `--extern std=`). Round 5's driver, in `base-admit/`, admits it. The cause is `entry_point`'s
  new obligation. With Rust's std, nothing is re-exported, so the unlisted Rust-std trait
  `Termination` is rejected. That is the documented fail-closed behaviour of this mode
  (`crates.rs`: "the allowlist checks it"). `-> ExitCode` was already rejected there before
  round 5, because the `ExitCode` type is unlisted. No Tide build runs in this mode: the stub
  and Tide's std export `Termination`. Neither the driver tests (libraries only) nor the
  corpus build a binary against Rust's std. So no current caller is affected, and an allow
  rule for this mode alone would be policy for a configuration nobody uses. Under the stub,
  plain `main` returning `()`, `ExitCode`, `Result<(), String>`, `Result<(), io::Error>`, or
  `Result<(), Box<dyn Error>>` is admitted (`t6/e5`-`e8`, `r1`-`r3`).
- **Round 5's glob change.** No pattern relies on a leading wildcard. The only patterns that
  begin with `*` are the two raw-pointer pseudo-items. Every other pattern begins with a crate
  name, `impl`, or `<`. Raw-pointer impls keep the grant through `admit_impl`'s head check. I
  probed `{:p}` of `&mut u32` and `&u32` (both FX0520 `new_pointer`) and `Hash` and `Debug`
  through `&mut fn()` (both FX0520). Hashing, comparing, `AsRef`, and formatting through
  `&mut` references are admitted (`t6/g1.rs`).
- **Round 5's fn-item where-clauses (`use_ty`).** I looked for false rejections from fn items
  used as values and from std adapter types:
  - fn items as values: `map(String::from)`, `Some`, `ToString::to_string`, `Box::new`,
    `Rc::new`, `u32::from`, a tuple-struct constructor, an enum variant, `Vec::len`,
    `Clone::clone`, `String::as_str`, `str::to_owned`, `str::parse::<u32>`, `u8::wrapping_add`
    in a fold, `u8::is_ascii_digit`, `Default::default`, and `Into::into` (`t6/f1.rs`);
  - std adapters: the `str` iterators (`lines`, `escape_*`, `matches(char::is_numeric)`,
    `split(char::is_whitespace)`, `encode_utf16`), `char::to_lowercase` in `flat_map`,
    `successors`, `from_fn`, `scan`, `map_while`, the `BTreeSet` set operations, `VecDeque`,
    and `BinaryHeap` (`t6/f2.rs`).

  Both files are admitted as a library and under `--test`. Associated-type bounds
  (`I: Iterator<Item: Debug>`, in argument position, on a struct and its `Drop` impl, and on a
  constructor passed as a function) are all FX0520 when instantiated with `fn()` (`t6/h1.rs`).
- **`entry_point`.** FX0520 at `fn main` for:
  - `-> impl Termination` hiding `Result<(), fn()>`;
  - a `main` re-exported from a module with `use m::main;`;
  - a type alias;
  - a `main` that a user `macro_rules!` defines;
  - a user `Termination` impl with `X: Debug`;
  - nested `Result<Result<(), fn()>, String>`;
  - `[fn(); 2]`;
  - `const fn main` (`t6/e1`-`e4`, `e9`, `e10`, `x7`, `x8`).

  `#[tide::main]`'s skip cannot be spoofed. The macro always writes `-> ExitCode` and reports
  the user's fn through `Termination::report` in a body the walk checks. A user attribute that
  `#(#attrs)*` forwards could rewrite that `main` only if it were a proc macro, and a proc
  macro is `Role::Other` and rejected. `#![no_main]` leaves no entry point to check; a C
  `main` then needs an unsafe attribute (UNSAFE-1, a later slice).
- **Erroneous programs on the new paths cause no ICE or panic.** These were tried: an unknown
  error type, a generic `main`, `fn main(x: u8)`, `async fn main`, and constructors and
  `.map(..)` over unresolved types (`t6/x1`-`x6`, `x3`). Each gives only rustc's errors, plus
  FX0520 for `Future` in `async fn main`.
- **A dyn-compatible trait method with its own `where X: Debug`.** It is checked where it is
  called (`t6/h1.rs:16`). A vtable built for `dyn Tr<fn()>` (`h1.rs:17`) includes it. Only a
  checked call can reach it, because std cannot call a user trait's methods.
- **Duplication.** The canonical-path printing (`with_no_visible_paths!` around
  `with_no_trimmed_paths!` around `with_resolve_crate_name!`) appears three times in
  `walk.rs`: `key`, `admit_impl`, and the depth rejection. A helper would save a few lines. It
  would also change code at this late round, for no behaviour change. I left it.
- **`obligation`'s doc** lists the cases that check nothing. It does not name ambiguous
  selection, which in post-analysis typing comes with a rustc error ("where rustc reports an
  error"). I left it.
- **Test coverage of the closed holes** is complete and not redundant:
  - round 1: B1 by `admit_crate_identity`, B2 by `global_fn_pointer`, B3 by
    `admit_runtime_entry`, B4 by the driver test `hidden_random_keys_…`, B8 by
    `hardware_queries_are_rejected`, and B9 by `admit_accept`;
  - Codex's round-1 points: by `admit_accept`, `global_fn_pointer`, `unsafe_granted`, and the
    driver test;
  - round 2: F1 by `global_fn_pointer`'s `#[test]`, F2 and F3 by `admit_accept`, F4 by
    `admit_std_io_method`;
  - round 3: F1 by `admit_obligation_routes`, F2 by the driver test's `sub.rs`;
  - round 4: by `admit_obligation_routes`;
  - round 5: F1 by `mapped`, F2 by `show`, F3 by `admit_main_return`.

  `run.sh`'s crash handling (B5) has no committed regression. It was probed with a fake driver
  in round 1 and is a few lines of script. AP-TEST does not call for more.
- **README, conformance/README, and AGENTS.md** match the final behaviour:
  - the two commands and their exit codes;
  - the PASS, AWAITS CHECKER, and FAIL rules as `run.sh:119-137` applies them;
  - the driver's flags (`-Zallow-features=`, `-Zpolonius=no`);
  - `--tide-print-codes`;
  - the `Pure` check and flag ownership, listed as not built.

  AGENTS.md's local section is byte-identical to `.agent-protocols.local.md`, whose sha256
  (`04e1d383…`) matches the manifest's overlay hash.
- The MIR or mono-item backstop and the choice of the HIR walk are round 5's decisions. They
  were not reopened.
- `design/reference.md` is untouched, at 24,992 tokens.

## Checked

- Read AGENTS.md, GOVERNANCE.md's Merge Rule, the bead, the six earlier reviews, every
  non-review file in `git diff 187a22b HEAD`, and the `#[tide::main]` macro
  (`spikes/d18/macros/src/lib.rs`).
- `sweep5.sh`, 225 runs over rounds 1-5's probes and the E1 programs in three modes:
  - on HEAD's build, the output is identical to round 5's `sweep5-final.txt` once sorted;
  - after the edits, still identical (`sweep5-r6final.txt`).
- Round-6 probes (`t6/`, 27 files) gave the verdicts above, before and after the edits.
- `CARGO_BUILD_JOBS=4 cargo test --workspace`: 2 + 4 passed, with no build warnings.
- `conformance/run.sh`: exit 0, 140 PASS, 58 AWAITS CHECKER, 0 FAIL.
- `rustfmt --edition 2024 --check` (stable) on the crate's sources is clean, and no line
  exceeds 100 columns. `git diff --check` is clean.
- `scripts/count-tokens.sh design/reference.md`: 24,992.
- Load stayed at or below 1.0, builds ran one at a time, and `CARGO_BUILD_JOBS=4` was used.
  Clippy is not installed for the pinned nightly, so it was not run.
