# Tide review, fx-kw8.1 round 1 (Opus): tide-admit workspace and admission gate

Candidate: commit 027350d on 9268da8 (worktree `/home/lucca/pjs/lab/tide-fx-kw8.1`, branch
fx-kw8.1). This is a converge round that reports and does not fix. No tracked file was changed.
Probe sources live in this review's scratchpad (`t/*.rs`, `adm.sh`). `adm.sh` runs the driver
the way `conformance/run.sh` does, against the pinned-nightly stub std.

## Verdict

revise

The structure is sound and the conformance gate works: both passes run, and the
`origin_nll_reject` case shows that `-Zpolonius=no` is needed. But the probes found admission
holes that D18 and ADMIT-1 say must be closed, a runner path that hides driver crashes, and a
regression in the prototype scripts.

## Blocking

**B1. Crate identity can be spoofed: `extern crate core as std;` turns `core` into Tide's std.**
`crates/tide-admit/src/crates.rs:95` (`injected_std`) takes the first `extern crate` item named
`std`, whichever crate it loads. `crates.rs:47` drops it only when it equals Rust's std, and
`role()` (`crates.rs:62`) checks `TideStd` before `Core` and `Alloc`. In a `#![no_std]` crate
whose root says `extern crate core as std;`, every `core` item gets `Role::TideStd`. It is keyed
as `tide_std::…`, and unlisted items are then admitted. Probe `t/nostd.rs` exits 0 with
`std::sync::atomic::AtomicU64::new`, `Ordering::Relaxed`, and `f64::to_bits`. The same code
written as `core::…` (`t/nostd2.rs`) gets 7×FX0520. The same works with `alloc as std` (Arc
without `Tasks`) and with any `Other` crate. This is the "wrong crate identity" ADMIT-1 forbids
("keyed on crate identity rather than crate name").
*Smallest fix:* accept only the `extern crate std` that rustc injects (its span's outer
expansion is `ExpnKind::AstPass(AstPass::StdImports)`). Also reset `tide_std` to `None` when it
equals `core` or `alloc`, as is already done for Rust's std. Add a driver test with the
`no_std` probe.

**B2. Function-pointer address observers are admitted without the `unsafe` grant.**
Probe `t/fnptr.rs` exits 0, with only rustc lints, for:
- `format!("{:?}", f)` with `f: fn()`, which prints the address;
- `f as usize` and `noop as usize`, which are pointer-to-integer casts;
- `f < g`, an ordering by address.

Probe `t/hw.rs` adds `Hash::hash(&f, h)`, which feeds the address to a user `Hasher`. GLOBAL-4
and the rules.md row both list "pointer-to-integer casts", and ADMIT-1 and D18 say "every
address observer". The walk sees no `ExprKind::Cast` at all. `admit_impl`
(`walk.rs:149-153`) treats a `FnPtr` self type as `_ => Ok(())`, so core's blanket
`impl<F: FnPtr> Debug/Hash/PartialOrd/Ord for F` pass on "trait admitted". Builtin `<` on fn
pointers never reaches an impl. The spike had the same gap.
*Smallest fix:* without the grant, reject:
- a cast whose source is an `FnDef` or `FnPtr` and whose target is an integer;
- `Debug`, `Pointer`, `Hash`, `PartialOrd`, and `Ord` impls selected for a `FnPtr` self type.
  One way is to key such impls by `def_path_str(impl)`, which gives `core::ptr::<impl …>`, so
  they fall under `granted("core::ptr::*")` with `PartialEq`/`Eq` carved out (identity, like
  `ptr::eq`);
- builtin ordering comparisons on fn-pointer operands.

Add one driver test or case.

**B3. Tide's std's runtime entry point is admitted from user code.**
ADMIT-1 lists "the internal runtime entry points of Tide's std" as outside the list. RUN-1
rests on it: test code cannot name "the runtime's minting entry point (ADMIT-1)". In the policy,
every `tide_std` item is admitted unless a rule says otherwise, and no rule names
`tide_std::__rt`. Probe `t/hw.rs`'s `pub fn rt() -> std::Root { std::__rt::root() }` gets no
FX0520, and it mints a real `Root`. The stub's `__rt::root` is what `#[tide::main]` expands to
(`spikes/d18/macros/src/lib.rs:48`).
*Smallest fix:* add `deny("tide_std::__rt::*", …)` to `policy.rs`, and exempt uses whose span
comes from the toolchain's `tide::main` attribute expansion, the way `generated_by_toolchain`
exempts core's `#[test]`. Add a case. If the author judges the entry point's form unsettled,
record that as a scoped deferral on the bead rather than leave the rule silently unenforced.

**B4. The obligation walk fails open past `MAX_OBLIGATION_DEPTH`.**
`walk.rs:274` silently returns at depth 12. Probe `t/depth.rs`: `Default::default()` for
`W<HashMap<u32,u32>>` (with `#[derive(Default)] struct W<T>(T)`) is rejected for
`impl Default for RandomState`, whose key comes from the OS (GLOBAL-5). The same call nested 14
deep is admitted. Also, `seen_obligations` (`walk.rs:286`) marks a trait ref as done even when
its where-clauses were cut off at the depth limit, so reaching it again at a shallower depth
skips it.
*Smallest fix:* at the limit, report FX0520 at the use's span, for example "obligation chain
too deep to check", instead of returning. Insert into `seen_obligations` only for refs whose
clauses were walked. The other silent returns are in the same family and could at least be
noted in the doc comment: normalization failure, ambiguous selection, and leftover
non-region inference variables.

**B5. `conformance/run.sh` counts a driver crash as a rejection.**
`run.sh:114` sets `dcode=1` for any nonzero exit, and `run.sh:122` only asks whether the driver
rejected. Consider a case with rustc marks and no driver-decided FX marks. If the driver ICEs
(exit 101) or refuses its arguments (exit 2), no FX lines appear, `driven == dgot` (both
empty), and `dcode != 0`, so the case prints PASS. The stable pass catches an ICE through
`?:error`, but the driver pass does not.
*Smallest fix:* keep the exit code (`|| dcode=$?`). FAIL any value other than 0 and 1, or any
`dlog` line matching `internal compiler error|unexpectedly panicked`.

**B6. The root `rust-toolchain.toml` breaks the prototype scripts as they document themselves.**
`trials/e1-dialect/rustc/run.sh` says it "needs stable rustc" and runs plain `cargo` and
`rustc`. Under the root pin it now resolves to nightly-2026-08-09: every program FAILs with
E0463 ("can't find crate for `oracle_macros`"), because the nightly cargo no longer puts
dependencies in `debug/deps`. With `RUSTUP_TOOLCHAIN=stable` it passes 22/22.
`spikes/d18/scripts/c1-facade.sh` fails the same way (27 errors, E0463). The only mitigation
is one sentence in AGENTS.md. The scripts' own headers still read as if the default toolchain
were stable.
*Smallest fix:* add `rust-toolchain.toml` with `channel = "stable"` in `trials/`, `spikes/`,
and `archive/`. `spikes/d18/admit/`'s own nightly pin is nearer and still wins. This restores
the old behaviour without editing any script, and the AGENTS.md caveat sentence (with its
overlay re-render) can then go.

**B7. `-Zpolonius=no` is a v0 semantic flag the design does not record.**
The flag is passed at `main.rs:94`. The pinned nightly accepts `nll.rs`, a get-or-insert body,
without it, and rejects it with E0502 with it. Stable rejects it too. ORIGIN-6 requires NLL's
verdicts. ADMIT-2 (`design/03-authority-concurrency.md:546`) and D18
(`design/01-design.md:989`) list `-Zallow-features=` as the flag the driver passes, and nothing
else. So the borrow-check mode the build relies on is decided only in code.
*Smallest fix:* add a clause to 03 ADMIT-2: "and `-Zpolonius=no`, so the pinned nightly checks
borrows with NLL (ORIGIN-6)". Optionally do the same in D18's sentence and in rules.md's
ADMIT-2 row.

**B8. The hardware-intrinsic rule and its test use a key rustc never produces.**
`policy.rs:72` denies `core::arch::*` with the help "hardware intrinsics are not admitted", and
`policy.rs:258` asserts `core::arch::x86_64::__cpuid` → Deny. The real key (probe `t/hw.rs`) is
`core::core_arch::x86::cpuid::__cpuid`, as is `CpuidResult`. Intrinsics are still rejected,
because unlisted `core` items are denied by default, but without the help. The unit test also
passes no matter how the walk keys them. No driver test or case covers a hardware query, which
is D18's first named closure.
*Smallest fix:* add `deny("core::core_arch::*", "hardware intrinsics are not admitted")`. Keep
`core::arch::*` for `asm!`/`naked_asm!`. Change the unit test to the real key, and add a driver
test line for `core::arch::x86_64::__cpuid(0)`.

**B9. Some float associated constants are rejected, some admitted (false rejection).**
`policy.rs:97-103` admits `EPSILON`, `MAX`, `MIN`, `MIN_POSITIVE`, `INFINITY`, `NEG_INFINITY`,
and `NAN`. The catch-all deny then rejects `MANTISSA_DIGITS` (probe: FX0520), `RADIX`,
`DIGITS`, `MIN_EXP`, `MAX_EXP`, `MIN_10_EXP`, and `MAX_10_EXP`. None of these is an observer
or a math method (reference 4.3, PAR-3), so a correct program is rejected. Low severity.
*Smallest fix:* list the remaining seven constants with the others.

## Non-blocking

**N1.** A single use yields several FX0520s at one span. For example
`core::sync::atomic::AtomicU64::new(0)` gives three: `Atomic::<u64>::new`, `AtomicU64`, and
`Atomic`. That is noisy for agents. Deduplicating on (code, span) and keeping the first, the
path's own item, would give one diagnostic per use. `tests/driver.rs` pins the current
three-line output.

**N2.** The driver pass checks only the driver's exit status against the rustc marks, not its
E-codes or lines. The pinned nightly builds the code, so a nightly-only divergence at a
different line or code goes unseen (stable stays the oracle by the corpus's design). A
follow-up could compare the driver's E-codes with `want` too.

**N3.** `policy.rs:187-189` describes the `std::io::stdio::_print`/`_eprint` allows as "in
Tide's std (D7)". They are Rust std's, reachable only through the stub's re-exported Rust
`print!` macros. The comment should say so, so the entry is revisited when Tide's std has its
own sinks.

**N4.** `run.sh:56` uses `xargs -n1 dirname`, which splits paths on whitespace. A checkout path
with spaces breaks `-L`. Use `while read` with `${f%/*}`. Nit.

**N5.** GLOBAL-4 says `ptr::eq` "compare[s] identity without exposing it". `core::ptr::eq(a, b)`
is rejected twice: by `granted("core::ptr::*")` and by the `*const T` coercion at its
arguments. This is fine if Tide's std is to provide a reference-taking `std::ptr::eq`. If
core's is meant, the coercion check would need a carve-out. Worth one line in 03 when Tide's
std's `ptr` is fixed.

**N6.** `main.rs:75` uses `std::env::args()`, which panics on a non-UTF-8 argument. rustc uses
`rustc_driver::args::raw_args`. Nit.

**N7.** Coverage gaps for the conformance track (fx-gcs), not this bead: there are no cases for
timed waits (`recv_timeout`), hardware queries, fn-pointer casts, or the runtime entry point.

**N8.** The `rust-src` component in `rust-toolchain.toml` is not needed to build or test the
driver (`rustc-dev` is). Keep it only if rust-analyzer on `rustc_private` needs it. Nit.

## Checked

- `git diff 9268da8 027350d`: every file read. `spikes/` is untouched. The worktree is clean at
  027350d.
- `CARGO_BUILD_JOBS=4 cargo test --workspace`: exit 0, 4 passed (2 policy unit tests, 2 driver
  tests).
- `conformance/run.sh`: exit 0, 136 PASS, 57 AWAITS CHECKER, 0 FAIL, about 32 s. This matches
  the author's figures.
- AGENTS.md rendering: the sha256 of `.agent-protocols.local.md` (`436cf9c6…`) equals
  `.agent-protocols.json`'s `overlay.sha256`. AGENTS.md's "Repository-local rules" section is
  byte-identical to the overlay. It was rendered, not hand-edited.
- Polonius: on `nll.rs`, `rustup run nightly-2026-08-09 rustc` exits 0 and the same with
  `-Zpolonius=no` gives E0502. Stable 1.96.0 gives E0502. The flag is needed (see B7).
- Driver probes, as lib against the pinned-nightly stub (`adm.sh`):
  - `nostd.rs` admitted (B1); `nostd2.rs` rejected 7×; `submod.rs` (`extern crate core as std`
    in a submodule) rejected, because the injected item comes first.
  - `fnptr.rs` and `hw.rs`: fn-pointer `{:?}`, `as usize`, `<`, and `Hash` admitted (B2);
    `std::__rt::root()` admitted (B3); `__cpuid` keyed `core::core_arch::…` (B8).
  - `depth.rs`: depth 1 rejected, depth 14 admitted (B4).
  - `probes.rs`: `Path::exists` through `PathBuf` deref, UFCS `Path::is_file`, `HashMap::new`,
    `black_box`, `spin_loop`, `Cell::as_ptr`, and `as_ptr_range` all rejected correctly;
    `MANTISSA_DIGITS` rejected (B9); `core::ptr::eq` rejected (N5).
  - `macros.rs`: unresolved bang, derive, and attribute macros cause no ICE in the expansion
    scan. `#[doc = include_str!]`, `env!`, and `x.sin()` inside a user macro are reported at
    the user's span.
  - `unres.rs`: `use std::sync::{LazyLock, OnceLock as Once}`, the type and expression
    positions of `OnceLock`, and `std::task::Waker` are each FX0520 at the right column.
  - `fnptr.rs --test`, with both Tide's and Rust's std: an atomic inside a `#[cfg(test)]`
    `#[test]` body is reported, so the harness skip does not hide user test code.
- Prototype scripts under the root pin: `trials/e1-dialect/rustc/run.sh` gives 22 FAIL (E0463)
  bare and 22 PASS with `RUSTUP_TOOLCHAIN=stable`; `spikes/d18/scripts/c1-facade.sh` bare
  fails with E0463 (B6). `archive/` was not run.
- Load was below 2 on 16 cores for every build. All builds were sequential.
