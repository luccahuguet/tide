# Tide review, fx-kw8.1 round 2 (Opus): converge round

Candidate: HEAD ed09473 on 187a22b (worktree `/home/lucca/pjs/lab/tide-fx-kw8.1`, branch
fx-kw8.1). This round reviewed `git diff 187a22b HEAD` in full and looked hardest at the round-1
fixes (`git range-diff 9268da8..027350d 187a22b..HEAD`). It fixed what it found in the working
tree, uncommitted. Probes are in this scratchpad's `t2/`, run by `adm2.sh` (the round-1
`adm.sh`, writing to `t2/`, with a `--bin` mode).

## Verdict

fixed

## Findings and fixes

**F1. A toolchain expansion exempted the impls that the user's types select (admission hole).**
The fix for round 1's B3 added `#[tide::main]` to `generated_by_toolchain` (`walk.rs:219`).
`check` skips every rejection whose span lies in such an expansion, and that covered
`obligation`'s impl check too (`walk.rs:360` at HEAD). Expansions call user-typed code. With
`#[tide::main] fn main() -> Result<(), fn()> { Err(noop) }`, the generated
`Termination::report` selects `fn(): Debug`, so the program prints a code address. The driver
admitted it without the grant (probe `t2/mainhole.rs`, exit 0). Core's `#[test]` had the same
hole before the fixes: `#[test] fn t() -> Result<(), fn()>` was admitted under `--test`
(`t2/testhole.rs`), because the harness's `assert_test_result` selects the same impl. The
round-1 reviews missed it.
*Fix:* `walk.rs` `obligation` now reports a rejected impl with `reject` directly, not
through `check`, with a comment giving the reason. The depth-limit rejection does the same.
`check_with` lost its only other caller, so I folded it into `check`. Items that the toolchain
names (the `test` crate, `__rt::root`) stay exempt. Both probes now give FX0520 at the
attribute. `global_fn_pointer.rs` gains a `#[test] //~ FX0520` returning `Result<(), fn()>`.

**F2. `#[derive(PartialEq)]` on an enum with two or more data variants was rejected (false
rejection).** rustc's derive expands to `unsafe { ::core::intrinsics::unreachable() }` in the
catch-all arm. `policy.rs` allowed `discriminant_value` for derives, but not `unreachable`.
I found this by running the E1 trial programs through the driver: `longest_lines.rs:8` and
`tally.rs:12` (`#[derive(Debug, Error, PartialEq)] enum ...`) got FX0520.
*Fix:* `policy.rs` now has `allow("core::intrinsics::unreachable")` beside
`discriminant_value`. Both are unstable to name directly, so only expansions reach them.
`admit_accept.rs` gains such an enum.

**F3. `ExitCode::SUCCESS` and `ExitCode::FAILURE` were rejected (false rejection).**
Reference 6.2 lets `main` return `ExitCode`, and the companion's example returns these
constants. They are inherent associated consts of Rust's `ExitCode`, which Tide's std
re-exports, and no rule listed them. The E1 trial's `dedupe.rs:108` hit this.
*Fix:* `policy.rs` allows the two constants, not `ExitCode::*`, which would include the
unstable `exit_process`. `admit_accept.rs` covers both.

**F4. `std::path::absolute` was admitted (ambient authority).** `allow("std::path::*")`
covered it. It reads the process's working directory, which Tide gives only through
`Env::current_dir` (03 §1.1 table, reference 6.1). Probe `t2/p1.rs` showed it admitted.
*Fix:* `policy.rs` has `deny("std::path::absolute", CWD)` with the help "use
`env.current_dir()?.join(path)`". `admit_std_io_method.rs` covers it.

**F5. `admit_hardware.rs` broke the corpus on any host that is not x86-64.**
`core::arch::x86_64` exists only on x86-64 targets. On other hosts stable rustc reports E0433,
the case FAILs, and `run.sh` exits 1. A `cfg` gate does not help, because the FX mark would
then go unmatched. It was the only host-dependent case.
*Fix:* I deleted the case. The same check is now a driver test,
`hardware_queries_are_rejected` in `tests/driver.rs`, under `#[cfg(target_arch = "x86_64")]`.
It asserts the exact `core::core_arch::x86::cpuid::__cpuid` diagnostic at `2:5`. This is the
form round 1's B8 proposed.

Nit, fixed with F3: the `ExitCode` entries have their own comment, separate from the
`_print`/`_eprint` one.

## Not changed

- **Spoofing `#[tide::main]`'s tokens:** not possible. The exemption requires the expansion's
  macro to be one Tide's std re-exports, named `main`, of kind `Attr`. Tokens the user passes
  through the attribute keep their own syntax context. `std::__rt::root()` written in a
  `#[tide::main]` body, directly or from a user `macro_rules!`, is rejected (`t2/spoof.rs`).
  Applying `#[tide::main]` to some other function, or calling the generated `main`, is
  ROOT-1/ROOT-5 (`FX0503`/`FX0506`), which later checker slices own.
- **Stub artifacts:** design-admitted code that the stub std makes fail. The conformance stub
  re-exports Rust's `HashMap`/`HashSet` (random key) and Rust's `fs::DirEntry`/`Metadata`.
  The design gives Tide's std its own fixed-key `HashMap::new()` (GLOBAL-5) and its own
  `DirEntry` and `Metadata` (03 §1.1). So under the driver, `HashMap::new()`, `entry.path()`,
  and `metadata.is_file()` get FX0520: E1's `canon`, `dedupe`, `deps`, `rules`, and `backup`.
  The policy is right for Rust's types. The stub is what diverges, and no case hits it yet.
  This belongs to the conformance track (fx-gcs): the stub should define those types before a
  case uses them. I suggest a bead, which I did not file.
- **The spikes/ and trials/ edits** (`export RUSTUP_TOOLCHAIN=stable`) are justified. rustup
  resolves `rust-toolchain.toml` from the current directory, not the script's, so nested
  toolchain files only work when a script runs from inside its directory. The env var is the
  smallest change that restores what the scripts' own headers say. `env.sh` is sourced only
  by the stable criterion scripts (c1, c3, c5, c6). `spikes/d18/admit/scripts` do not source
  it, so the spike's nightly driver keeps its own pin. `archive/v0-design/scaffold` now
  resolves to the nightly pin. It is a record and was not run.
- `run.sh` is correct as fixed. It keeps the highest exit status of the two driver runs,
  treats anything above 1, ICE text, or panic text as a crash, and builds with
  `--target-dir "$ROOT/target"`. `pinned` unsets `RUSTUP_TOOLCHAIN` and runs from the root, so
  a caller's stable override cannot leak in. The dependency directories survive spaces.
- The design edits are consistent. GLOBAL-4's fn-pointer sentence and Tide's
  `ptr::eq(&T, &T)` agree with D18 (01:426), reference 7 (:554), and rules.md. `-Zpolonius=no`
  appears in 03 ADMIT-2, D18 (01:990), rules.md, README, and `main.rs`. No other design text
  names `ptr::eq`.
- The two-level diagnostic dedup is kept. The per-line `(code, message, line)` filter merges
  an expression, its pattern, and its type, which sit at different columns. The
  per-position filter picks the narrowest span. They do different jobs.
- `core::ptr::fn_addr_eq` needs the grant though `==` does not. It is niche and sits under
  `core::ptr::*`. I left it.
- `std::hash::DefaultHasher::new` (fixed key) is rejected. The design does not mention it,
  and a fail-closed allowlist is right here.
- The silent returns in `obligation` (normalization failure, ambiguous selection, leftover
  inference variables) are as in round 1. No probe exploited them: post-analysis typing
  reveals `impl Trait`, so `fn f() -> impl Debug { noop as fn() }` used in `format!` is
  rejected (`t2/p1.rs`).
- `design/reference.md` is untouched, at 24,992 tokens.

## Checked

- `git diff 187a22b HEAD`: every non-review file read. The range-diff over the fixes was read.
- AGENTS.md rendering: the sha256 of `.agent-protocols.local.md` (`04e1d383…`) equals the
  manifest's `overlay.sha256`, and AGENTS.md's local section is byte-identical to the overlay.
- Probes (`adm2.sh`, against the pinned-nightly stub):
  - before the fixes:
    - `mainhole.rs` (bin) and `testhole.rs` (`--test`) exit 0 (F1);
    - `p1.rs` admitted `std::path::absolute` (F4) and rejected `ExitCode::SUCCESS` (F3).
  - after the fixes:
    - `mainhole.rs` and `testhole.rs` give FX0520 at the attribute;
    - `mainok.rs` (`fn main(root: Root) -> Result<(), io::Error>`) exits 0;
    - `p4.rs` (`-> ExitCode`, `ExitCode::from`, `#[test] -> Result<(), String>`) gives only
      the intended `&fn()` to `&dyn Debug` rejection;
    - `p3.rs` (fn-pointer calls, `vec!`, `clone`, `==`, `contains`, derived
      `PartialEq/Eq/Hash/PartialOrd/Ord` on a data enum) exits 0 in both modes;
    - `spoof.rs` rejects both `__rt::root()` uses.
- E1 trial programs (`trials/e1-dialect/programs/*.rs`) through the driver, both modes. Before
  the fixes, F2 and F3 showed. After, only the stub artifacts listed above remain.
- `CARGO_BUILD_JOBS=4 cargo test --workspace`: before, 2 + 3 passed. After, 2 + 4 passed, the
  new `hardware_queries_are_rejected` included.
- `conformance/run.sh`: before, exit 0 with 140 PASS and 57 AWAITS CHECKER. After the final
  edit, exit 0 with 139 PASS, 57 AWAITS CHECKER, and 0 FAIL (one case moved to the driver
  test). The changed cases (`admit_accept`, `admit_std_io_method`, `global_fn_pointer`) and
  `admit_runtime_entry`, `root_main_accept`, and `root_main_called` were also run on their own.
- `rustfmt --edition 2024 --check` (stable rustfmt, since the pin has none) on the crate's
  sources is clean. `git diff --check` is clean.
- `scripts/count-tokens.sh design/reference.md`: 24,992.
- Load stayed below 1 on every build, and builds ran one at a time.
