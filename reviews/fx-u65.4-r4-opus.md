Model: claude-opus-5-5 · Date: 2026-11-12

Verdict: request_changes

Scope: the full diff `dd40b78..e992283` (30 files), read-only. I read AGENTS.md, both beads (including the 2026-11-11 decisions comment), the three Codex passes, the implementer's report, reference §6.1-6.2, 6.8, 6.9, 7.4, the companion's FX05xx rows, chapter 03 CAP-1 and ROOT-1..3, and every touched file with its callers. The policy table, diagnostic plumbing, test-code predicate, FX0505 and the import-identity recovery are sound apart from the findings below. Two findings block. Each is small, and each fix changes code and cases this change adds.

## Findings

1. **Blocking: a non-entry function can name `Root` in its signature by writing a field pattern as the parameter (FX0503 false negative).**
   - Evidence:
     - `crates/tide-admit/src/walk/authority.rs:123-140`. The parameter exemption has two arms. The second, `input.span.contains(path.span) && root_fields(param.pat)`, applies to every function, not only the entry: unlike the first arm it lacks the `entry &&` guard.
     - `conformance/cases/authority/root_destructure_accept.rs:3-4` asserts the behavior as accepted: `pub fn fields(Root { fs, .. }: Root)` and `pub fn borrowed(Root { fs, .. }: &Root)`.
     - `conformance/rules.md` (ROOT-1 and ROOT-2 rows) and `conformance/README.md:141-142` ("in field patterns (including their parameter types)") document it.
     - Probe B: `pub fn helper(Root { fs, .. }: &Root)` and `pub fn helper_owned(Root { clock, .. }: Root)` are both admitted with no FX0503, in normal and `--test` modes. Only `pub fn helper2(root: &Root)` gets FX0503.
   - Why it is wrong:
     - Reference §6.2 says "`Root` can be named only in `main`'s signature and as `Root::sim`".
     - Chapter 03 ROOT-1 says the same.
     - The companion's FX0503 row is exactly `fn helper(root: &Root)`.
     - The 2026-11-11 decision covers destructuring *`root`*, the entry's value, in a pattern. It does not cover a helper whose own signature takes a `Root`. A helper parameter pattern destructures the caller's argument, so the helper still has a `Root`-typed signature.
     - Calling such a helper is still FX0503 at the call site (`helper(&root)`), so no executable bypass exists. But a library can export `pub fn f(Root { fs, .. }: &Root)` and pass admission, which is exactly the "code that is not Tide" the bead exists to reject. The FX0503 lands at the call instead of at the signature the companion names.
   - Fix:
     - Guard the second arm with `entry` too. The parameter-pattern exemption should apply only to the actual entry `main` and its generated `__tide_main`, which `root_parameter_pattern_accept.rs` already covers.
     - Move lines 3-4 of `root_destructure_accept.rs` into a reject case, with FX0503 on each signature's `Root`.
     - Reword the ROOT-1 and ROOT-2 rows in `rules.md` and the `conformance/README.md` sentence to say "field patterns destructuring `root` (including `main`'s parameter)".
     - Keep the let, if-let and match field patterns, and the `let Root { .. }: &Root = &root` annotation, as they are. They can only destructure a value that came from `root`.

2. **Blocking: the by-name `thread_rng` rule rejects a user's own function (FX0501 false positive).**
   - Evidence: `crates/tide-admit/src/walk/authority.rs:80-87` applies the `thread_rng` pseudo-key to every resolved `Fn`/`AssocFn` with that name, including local definitions. Probe A: `pub fn thread_rng() -> u64 { 4 } pub fn draw() -> u64 { thread_rng() }` gets `demo.rs:2:24: error: FX0501: `thread_rng` is not admitted`, exit 1.
   - Why it is wrong: the decision says "thread_rng by name" so that the external `rand` function is caught without a key into an unadmitted crate. A local Tide function of that name, for example one taking `&Rand`, is ordinary Tide code, and the error tells the writer something false.
   - Fix:
     - Skip the resolved-def name check when the definition is in the local crate, and likewise for any other Tide-admitted crate whose role the walk can tell. Keep `authority_unresolved`'s name check, which covers the realistic `rand::thread_rng()` / `use rand::thread_rng` route.
     - Add the local `fn thread_rng` to `cap_ambient_accept.rs` as the accepted twin.

3. **Non-blocking: by the decision's own membership rule, the closed table misses a few items.**
   - The rule is "items of the same std modules that perform an operation in §6.1's capability table". Probe C shows these get only rustc's E0433/E0432/E0599 with no FX0501:
     - `std::thread::park_timeout` and `park_timeout_ms` are timed waits that read time without a `Clock`. CAP-1's "timed wait such as `recv_timeout`" covers them.
     - `std::thread::sleep_ms` is a deprecated sleep.
     - `str`/`String`/`(&str, u16)` `ToSocketAddrs::to_socket_addrs` performs DNS, which is `Net::resolve`.
   - Rustc still rejects all of these, so none can pass admission. Only the teaching diagnostic is missing. `policy.rs:84-141` has the table.
   - Fix: add `std::thread::functions::park_timeout`, `park_timeout_ms` and `sleep_ms` rows. Settle the DNS item after fx-hxw lands, because fx-hxw's `Net::connect`/`bind` address parameter may rely on an address trait. Don't deny the `ToSocketAddrs` trait wholesale; key the string impls' resolution only.

4. **Non-blocking: two mechanisms now recover unresolved std paths.**
   - Evidence:
     - `walk.rs:1450-1500`, the existing `unresolved`, walks absolute `std::…` module paths and applies any Deny rule.
     - `authority.rs:328-486`, the new `authority_unresolved`/`rust_path`/`imported_*`, recovers absolute paths, aliases, imports, globs and associated items. Its `ambient_key` (`:488-489`) then filters to FX0501 only.
   - Result: an absolute `std::fs::read` goes through both, and the duplicate is removed by `Diagnostics::emit`'s per-code dedupe. Meanwhile an aliased or imported *FX0520* item still loses its code while ambient ones keep theirs. For example, `use std::sync as s; s::LazyLock::new(..)` or `use std::sync::LazyLock as L;` gets only rustc's error, while the absolute `std::sync::LazyLock` gets FX0520. I found this by reading `unresolved` (it returns unless the first segment is `std`) and `ambient_key`'s filter; I did not probe it.
   - Related: `authority_use`'s `self.ambient_key(key)` for resolved defs (`authority.rs:67-68`) repeats the verdict `use_def` → `admit_def` → `policy::lookup` already returns with the same rule's `code`. Probe D shows each ambient Rust-std path reported once, so it is redundant.
   - Fix: make `rust_path` the one owner of unresolved-path recovery, and have `unresolved` apply every Deny rule (with `rule.code`) to the item it recovers. Delete the old module walk and the resolved-def `ambient_key` call. This is a simplification and ownership cleanup with no new behavior beyond the FX0520 alias consistency.

5. **Non-blocking (reference routing): the reference text does not yet allow the decided field destructuring.**
   - Reference §6.2 and chapter 03 ROOT-1 say `Root` is named only in `main`'s signature and `Root::sim`. `let Root { fs, .. } = root;`, which the binding decision accepts, names `Root` in a pattern.
   - `conformance/rules.md` now describes the decided behavior and cites rows whose text says otherwise.
   - The code follows the recorded decision, so this does not block the candidate. The coordinator should route the implementer's suggested §6.2 sentence through the reference holder (a single-holder file, with the D14 recount), narrowed per finding 1: "field patterns that destructure `root`", not any parameter pattern.

6. **Nit:** the rustc column of the CAP-1 row in `conformance/rules.md` lists "E0425 / E0599 (unresolved)", but this change's cases also expect E0433 (`cap_ambient_paths_reject.rs:14-15,17-18`) and E0432 (`cap_ambient_import_reject.rs`). Add them.

## Checked and found sound

- **FX0501 table (`policy.rs`).**
  - All 52 keys exist at their canonical paths in the pinned std (probe D: every line gets FX0501).
  - Membership matches the decision's enumerated list: file openers, env readers, `Instant`/`SystemTime` `now` and `elapsed`, sleep, exit and abort, `Command::new`, stdin, socket constructors, and `thread_rng`. There are no stray entries. Env mutators are excluded, consistent with "readers".
  - The ambient rules sit ahead of every other rule except `tide_std::__rt::*`, so first-match cannot recode them. `Path` I/O observers and `last_os_error` stay FX0520 (unit test).
  - Aliases and other path forms keep FX0501 (probe E): `use std::time::Instant as I; I::now()`, `<I>::elapsed(t)`, a type alias `Alias::now()`, `use std as s; s::fs::read`, method calls inside `format!` and inside closures.
- **FX0503.**
  - Field let, if-let and match forms and `match &root { Root { .. } => .. }` are accepted (probe F).
  - The guarded-match and wildcard rule (`authority.rs:155-160`) is correct.
  - Whole-root `@` bindings, generic passing, storage and whole capture are rejected.
  - Tuple packing `(root, 1)` is rejected even when destructured, which is conservative and consistent with "stored".
- **FX0504.** Evaluating the cfg trace with `test=false` while keeping the target fixed is the right predicate. `any(test, not(test))` is rejected and `any(test, not(unix))` is accepted on unix. The `--tide-test-library` suite flag is set only for separate suites.
- **FX0505.** Resolved Tide-std callee, final `&Tasks`, arity = supplied + 1: this covers path, method and fn-item-binding forms, with no constructor registry.
- **Ownership and severity.**
  - `Rule.code` is owned by `policy.rs`.
  - Every other `Rejection` site keeps FX0520.
  - All four codes are errors in both modes and against production std (`build.rs` authority test).
- **Acceptance.**
  - Each code has a violating and an accepted case: FX0501 `cap_ambient_paths_reject` / `cap_ambient_accept`; FX0503 `root_names_reject` and `root_uses_reject` / `root_fields_accept`; FX0504 `sim_production_reject` / `sim_test_context_accept`; FX0505 `cap_handle_no_tasks` / `cap_handle_tasks`.
  - `target/debug/tide-admit --tide-print-codes` lists FX0501 through FX0505, so `run.sh` prints no AWAITS CHECKER line for these codes.
  - Every `//@ rules:` ID in the touched cases exists in `rules.md`: ADMIT-1, CAP-1, CAP-2, CHANNEL-1, CLAUSE-1, CLAUSE-6, FORM-4, ROOT-1..3, SCOPE-1, SCOPE-3, SEND-3.
  - The contract generator's "Root parameter" skip (`synthesis/contracts.rs:504,668`) is still a needed defense, because synthesis plans before admission. It is not orphaned.
- **fx-hxw.**
  - The only conflict is the shared insertion anchor after `deny("tide_std::__rt::*")`, plus the known E0433→E0599 marks on `cap_ambient_paths_reject.rs:14-15`.
  - FX0501 cannot catch Tide's `Net` methods: the table has no `tide_std::` keys, `rust_counterpart(tide_std::net::Net)` finds no Rust `std::net::Net`, and the unit test pins absence for `Net::bind`/`connect`.
  - A Tide `TcpStream`/`TcpListener` missing `connect`/`bind` maps to Rust's type and correctly gets FX0501 whether Tide defines it at `net::` or in a submodule.
  - `core::net::*` and `std::net::Shutdown::*` overlap no table row.

## Probes

Environment: Linux x86_64, pinned nightly. All probes used the candidate's existing `target/debug/tide-admit`, built at 14:04 -03, after the last source edit at 13:57, and production std `target/tmp/build/std/artifacts/libtide_std.rlib` (or Rust std, where noted).

Flags: `--edition 2024 --emit=metadata --error-format=short [--tide-std … --tide-dep-dir …]`, under `ulimit -c 0`, `CARGO_BUILD_JOBS=4 nice -n 19 timeout 60` and closed stdin, through `scratchpad/probes/run.sh` (session scratchpad).

Gating:
- The first `build-gate.py 120` refused (`load >= 12`).
- Later gates passed at 17:15:30Z and 17:15:58Z before the two serial batches. The quiet-until mark (1794497591) had expired.

| Probe | Mode | Source | Result |
| --- | --- | --- | --- |
| A | lib, Tide std | local `fn thread_rng` and its caller | exit 1; `2:24 FX0501 thread_rng` (finding 2) |
| B | lib, Tide std, normal and `--test` | `helper(Root { fs, .. }: &Root)`, `helper_owned(Root { clock, .. }: Root)`, `helper2(root: &Root)` | exit 1; FX0503 only on line 3 (`helper2`); lines 1-2 admitted (finding 1) |
| C | lib, Tide std | `thread::sleep_ms`, `thread::park_timeout`, `to_socket_addrs`, `UdpSocket::bind` | only `UdpSocket::bind` gets FX0501; the others get rustc E0433/E0432/E0599 only (finding 3) |
| D | lib, Rust std | all 52 table paths as fn items | every line gets FX0501 with exactly the table key; ok |
| E | lib, Tide std | aliased, type-alias, qualified-self, module-alias, macro-argument and closure forms of `Instant::now`/`elapsed` and `fs::read` | all six sites get FX0501; ok |
| F | bin, Tide std | `if let Root { ref fs, .. } = root`, `match &root { Root { clock, .. } => .. }`, `let (Root { env, .. }, _n) = (root, 1)` | first two accepted; tuple packing gets FX0503 at 5:35 (conservative, acceptable) |

No full suites, `heavy.sh` runs, repository writes, `br` writes, or held-out reads (`trials/e1-checked/` was neither opened nor searched).

DONE
