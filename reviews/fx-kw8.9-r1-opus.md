# fx-kw8.9 round 1: Opus review

Candidate: commit 93c1dec on branch `fx-kw8.9` (base d12d7e0)

Model: claude-opus-5-5 · Date: 2026-11-07

## Verdict

**Changes required.** The single checked `--emit=link` compilation, the fixed
host linker, the separate cache identities, the uncached granted links, and
the disclaimers (no RUN-5, no RUN-1 unreviewed-branch guarantee, no execution
by the driver) are sound. The Acceptance holds for the debug profile. It
fails for the release profile: `-Copt-level=3` turns on rustc's MIR inliner,
which removes inlined instances from the mono items the backstop checks. So
release can admit and link a program that debug rejects (B1). That is the
semantic constraint the bead's Scope says the fx-kw8.4 edge puts on codegen.
Two smaller defects: the README's build example points `--tide-dep-dir` at a
directory this cargo does not create (B2, text-only), and every link build
prints a warning the driver causes itself (B3).

Proof I ran on a copy of the candidate
(`git archive 93c1dec`, pinned nightly-2026-08-09, offline):
`cargo test --workspace` exited 0 (45 passed). Probes run with the
candidate's `target/debug/tide-admit` are cited under each finding. I did
not run `conformance/run.sh`; no finding depends on it.

## Decisions

1. **Build boundary, a single checked `--emit=link` compilation: accept.**
   It leaves no second admission or snapshot handoff, and
   `abort_if_errors` after the backstop keeps rejected instances away from
   codegen. Rejected source (walk, backstop, FX0506, FX0512, granted
   assembly with `.error`) left no executable and did not run the
   assembler. Condition: this holds only once B1 makes the backstop see
   the same instances in every profile.
2. **Linker and host, x86_64 Linux GNU with fixed `/usr/bin/cc` and
   `-B/usr/bin` and no bundled LLD: accept.** I checked that the output was
   linked by GNU ld: `.comment` has no `Linker: LLD`, and `/usr/bin/ld`
   resolves to `ld.bfd`. Add one line to the linker contract in the README:
   per GCC's directory options, `-B/usr/bin` also passes `-L/usr/bin` to
   the linker.
3. **Profiles, debug = opt 0 + debug assertions and release = opt 3 without
   them, overflow checks on in both: accept, with the B1 change.** The
   mapping must also pin `-Zinline-mir=no` (or an equivalent) in every
   profile. Otherwise the profile changes what the backstop checks, and an
   operator-chosen profile turns into an admission flag. LLVM inlining runs
   after mono collection, so the release runtime target (D15/D18 backend)
   stays reachable.
4. **Executable tests, Rust's builtin parameterless `#[test]` built with
   `--test --emit=link` and run by the trusted caller: accept.** It is
   documented as not `tide test`/`tide sim` and not RUN-1/RUN-5. Re-exporting
   `core::prelude::v1::test` is the smallest production surface, and the
   shared fixture keeps the stub's form and production's form identical.
5. **Caching, one linked-build entry with normalized mode, profile, and test
   arguments plus cc/ld fingerprints, and no reuse for unsafe-granted links:
   accept.** Metadata, bin link, and test link get distinct keys (five
   entries in the test). The granted-native-input test proves the uncached
   rule directly. Transitive native inputs (collect2, the plugin, crt
   files, libc) that are trusted and not fingerprinted are documented.
6. **Outputs, a flat `--out-dir` with `<name>` / `<name>-test`, removal
   before checking, and a shared `interface.json`: accept.** `-test` cannot
   collide with a crate name, publication is atomic (tmp then rename), and
   the latest-invocation ownership of `interface.json` is documented. See N5
   on the delete-before-check scope.
7. **Trusted demos, an explicit link opt-in plus operator review: accept.**
   An acknowledgement flag would enforce nothing, and the docs (README,
   `main.rs`, `tide-std/src/lib.rs`, `tests/build.rs`) consistently say
   this is not RUN-5.

## Findings

### B1. The release profile hides inlined instances from the backstop (not text-only)

`crates/tide-admit/src/invocation.rs:204-216` maps `--tide-profile=release`
to `-Copt-level=3`. That raises rustc's default `mir-opt-level` to 2, which
enables the MIR inliner. `check_mono_items`
(`crates/tide-admit/src/walk/mono.rs:35-40`) checks the items in
`collect_and_partition_mono_items`. When an upstream impl method is inlined
into a local body, it is no longer a mono item, even though its code still
runs. That breaks ADMIT-1's "the instances that can run" and the Scope's
"forced mono collection/share-generics decisions constrain codegen". The
flag also applies to `--emit=metadata --tide-profile=release`.

Failure scenario, observed on the candidate (bin, `--emit=link`,
`--tide-test-backstop`, production std):

```rust
fn noop() {}
fn other() {}
trait Addr { fn f(&self, which: bool) -> impl PartialOrd; }
impl Addr for u8 {
    fn f(&self, which: bool) -> impl PartialOrd {
        if which { noop as fn() } else { other as fn() }
    }
}
#[tide::main]
fn main() {
    let first = "1".parse::<u8>().unwrap() == 1;
    println!("{}", 1u8.f(first) < 1u8.f(!first));
}
```

- debug: `FX0520: impl core::cmp::PartialOrd for fn() is not admitted (it
  runs as core::ptr::<impl PartialOrd for fn()>::partial_cmp)`, and no
  executable is written.
- release: no error. `out/demo` is linked, and running it prints `false`.
  That is a code-address comparison (GLOBAL-4) without the `unsafe` grant.

The existing backstop route test
(`tests/driver.rs:190`, `the_backstop_rejects_denied_impls_whatever_route_reaches_them`)
also changes under `--tide-profile=release`. Its `HashMap: From<[(K, V); N]>`
route is no longer reported at `lib.rs:8:5`. The case is still rejected
only because a deeper denied impl, `RandomState: Default`, happens to remain
a mono item (`lib.rs:6:1`). Several other routes move to different sites.
In normal mode the walk's obligation steps still caught every route I tried.
But the backstop exists for the routes the walk misses, so the release
profile removes that layer for exactly the cases it guards.

Fix: in `rustc_args`, pass `-Zinline-mir=no` in every profile, next to
`-Zshare-generics=no`, with a comment saying why. Record it in ADMIT-2
(`design/03-authority-concurrency.md`, beside `-Zshare-generics=no`) and in
the README's profile paragraph. I verified the fix on a patched copy: the
probe above is rejected in release with the same diagnostic as debug, the
driver.rs route test's 11 sites under release match debug exactly, and
`cargo test -p tide-admit` passes. Proof to add: run the
`the_backstop_rejects_denied_impls…` assertions under both profiles (they
fail on the candidate), or add the probe above as a release case in
`rejected_source_never_links_or_runs_and_removes_stale_outputs`.

### B2. The README build example uses a dependency directory that does not exist (text-only)

At `README.md:126-133`, `--tide-dep-dir target/debug/deps` fails with
`tide-admit: cannot resolve trusted dependency directory`. Pinned cargo
1.99 (2026-08-07) uses the new build-dir layout, so it creates no
`target/debug/deps` (neither does the worktree's own `target/`). The
dependency `libtide_macros-<hash>.so` is under
`target/debug/build/tide-macros/<hash>/out/`, as `conformance/run.sh:50-56`
already handles. `--tide-dep-dir target/debug` works, but `fingerprint`
then hashes the whole tree, including the ~96 MB driver and `incremental/`.
Fix: name the directory that cargo's artifact message gives for
`libtide_macros` (as `run.sh` does), or write the example with that
directory as a shell variable and say how to find it.

### B3. Every link build warns "ignoring --out-dir flag due to -o flag" (not text-only)

`invocation.rs:219-220` always passes `--out-dir`, and `:231-232` adds
`-o <stage>/executable` in link mode. Every link build, clean ones and
cache hits included (the warning is cached in the `stderr` file and
replayed), prints `warning: ignoring --out-dir flag due to -o flag`, and
the final line becomes `1 warning emitted` (seen in every probe above). An
operator cannot tell this self-inflicted warning from a warning about the
candidate. No test checks a clean build's stderr. Fix: omit `--out-dir` in
link mode. `-o` and `-Ztemps-dir` already place every output, and the trust
artifact goes to `Admission.out`. Also assert `out.stderr.is_empty()` in
`checked_hello_builds_without_running_and_ignores_candidate_configuration`,
which also makes its "the driver never ran it" check stricter.

### Nits

- **N1.** `invocation.rs:86` stores `profile: String` and compares it
  against `"release"` twice. A `release: bool` (or a two-variant enum) gives
  that one invariant one owner. After the new `abort_if_errors()` at `:64`,
  the `has_errors().is_none()` test at `:66` is dead.
- **N2.** `/usr/bin/cc` appears at `:225` and `:764`, and `/usr/bin/ld` is
  fingerprinted at `:765` but reached only implicitly through `-B/usr/bin`
  at `:228`. Put the host-linker contract in one constant or function, so
  the fingerprint and the arguments cannot drift apart (AP-OWNERSHIP-001).
- **N3.** `conformance/README.md:90-91` says `//@ profile: release` is
  something "the checker acts on". Now that the driver has
  `--tide-profile=release`, `conformance/run.sh:111` passes the grant but
  not the profile. Wire it now, or when FX0507 (`sink_dbg_release.rs`)
  lands.
- **N4.** `crates/tide-admit/src/interface.rs:3` says the artifact is
  written "beside the metadata". In link mode it sits beside the
  executable.
- **N5.** `invocation.rs:707-712` deletes any file at
  `<out-dir>/<crate-name>` before checking, including one the driver never
  wrote (`--out-dir ~ --crate-name=notes`). rustc and cargo overwrite only
  when they succeed. This is acceptable given the documented explicit
  out-dir, but one README sentence should say it.
- **N6.** If a step between `entry = staging` (`:862`) and
  `remove_dir_all` (`:892`) fails in an uncacheable build, the staging
  directory is left behind. A later run with the same pid clears it, so
  this is cosmetic.
- **N7.** The README's "private cache staging" means `<out-dir>/.tide-cache`,
  which is in the operator's output directory, not a private one. Say
  "the driver's cache staging directory".
