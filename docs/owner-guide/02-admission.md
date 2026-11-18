# 2. Admission: what Tide checks, and what it trusts rustc for

## The idea

Tide doesn't have its own compiler. Every Tide program is ordinary Rust,
compiled by a pinned, unmodified version of rustc (the exact nightly named in
`rust-toolchain.toml`, `nightly-2026-08-09`). What Tide adds is a gate in front
of that compiler, called **admission**. A program that passes it is
*admitted*; one that doesn't gets `TD` errors, and no output is written.

The gate is `tide-admit`, a **rustc driver**: a program that runs rustc's own
code as a library and adds checks between its stages. So Tide sees exactly
what rustc sees: the same parsed code, the same resolved names, the same types.
It doesn't need a second, separate understanding of Rust that could drift from
the real one.

```text
  your .rs file
       |
  tide-admit builds the one rustc invocation (no user flags, no Cargo,
       |      no build scripts; environment cleared)
       v
  rustc parses and expands ---> Tide checks the source:
       |                         subset rules, authority, allowlist, unsafe grant
       v
  rustc type-checks and borrow-checks  <- memory safety, data-race freedom
       |
       v
  Tide's backstop: every piece of code that can actually run,
       |            re-checked against the allowlist
       v
  rustc + LLVM generate the program; Tide writes the trust artifact
```

**What Tide checks** (each family has its own `TD` codes):

- **Authority**: no ambient calls (`TD0501`), `Root` only in `main`
  (`TD0503`), simulation only in tests (`TD0504`), no global mutable state
  (`TD0502`). Chapter 1.
- **The allowlist** (`TD0520`): only listed items from Rust's lower layers.
  For example, no hardware queries, no raw memory addresses, no compile-time
  file reads such as `include_str!`.
- **`unsafe` needs a grant** (`TD0901`). `unsafe` is the Rust keyword that
  permits a few operations the compiler cannot prove safe, such as reading
  memory without a bounds check; the author must make sure they are. rustc's
  ordinary type and borrow checks still run inside it. In
  Tide, only the operator (whoever runs the build) can allow it, with
  `--tide-grant=unsafe`. The program itself can't grant it.
- **Concurrency habits**: no `async`, no bare threads, no `Mutex` (chapter 4).
- **A smaller language**: no user macros (code that writes code, `TD0101`),
  no lossy number conversions (`TD0301`), and a ceiling on the trait system's
  power (`TD0401`–`TD0408`). These keep programs predictable for agents and
  reviewers.
- **The driver owns the build**: no build scripts (`TD0915`), no
  file-including attributes (`TD0911`), no unstable compiler features, and no
  compiler flags from the program.
- **Closed soundness routes**: `TD0206` and `TD0402` (chapter 3).

**What Tide trusts rustc for**: parsing, names, types, and above all **the
borrow checker**. That is the part of rustc that proves memory safety: no
reading freed memory, no two parts of a program changing the same data at
once (a *data race*). Tide doesn't reimplement any of it. Because every Tide
program is Rust, rustc's borrow checker runs on every Tide build. Tide also
trusts LLVM, the code generator behind rustc, and the operating system's
linker.

## Run it

First, the list of codes this driver enforces:

```sh
target/debug/tide-admit --tide-print-codes
```

Output:

```text
TD0101
TD0102
…
TD0206
…
TD0501
TD0502
TD0503
…
TD0915
```

(57 codes in all.)

A memory-safety bug, which rustc catches, not Tide:
[examples/borrow/src/lib.rs](examples/borrow/src/lib.rs).

```sh
target/debug/tide-admit --crate-type=lib --tide-std target/guide/std/libtide_std.rlib --tide-dep-dir target/guide/std --out-dir target/guide/borrow docs/owner-guide/examples/borrow/src/lib.rs
```

Output (paths shortened):

```text
…/borrow/src/lib.rs:6:5: error[E0502]: cannot borrow `*names` as mutable because it is also borrowed as immutable: mutable borrow occurs here
error: aborting due to 1 previous error
```

An `unsafe` block, with and without the operator's grant:
[examples/unsafe_hole/src/lib.rs](examples/unsafe_hole/src/lib.rs).

```sh
target/debug/tide-admit --crate-type=lib --crate-name=unsafe_hole --tide-std target/guide/std/libtide_std.rlib --tide-dep-dir target/guide/std --out-dir target/guide/unsafe docs/owner-guide/examples/unsafe_hole/src/lib.rs
```

Output (paths shortened):

```text
…/unsafe_hole/src/lib.rs:9:5: error: TD0901: an `unsafe` block needs the package's `unsafe` grant
error: aborting due to 1 previous error
```

```sh
target/debug/tide-admit --crate-type=lib --crate-name=unsafe_hole --tide-grant=unsafe --tide-std target/guide/std/libtide_std.rlib --tide-dep-dir target/guide/std --out-dir target/guide/unsafe docs/owner-guide/examples/unsafe_hole/src/lib.rs
```

It prints nothing and exits 0: admitted.

## What to look at

- The code list is the gate's whole vocabulary. Every code is a rule with an
  ID in `conformance/rules.md`, and the conformance corpus (`conformance/`)
  holds example programs with the verdict each must get.
- The borrow error has a plain `E` code (`E0502`), not a `TD` code: rustc
  found it. Tide never has to.
- Without the grant, `TD0901` rejects the `unsafe` block. With it, the
  program is admitted, and chapter 7 shows where that `unsafe` site is listed.

## Read the code

**`conformance/cases/authority/cap_ambient.rs`** (9 lines, 10 minutes). One
conformance case. The case file is itself a small program, and both checkers
are run on it; comments in it hold the verdicts they must give.

```sh
cat conformance/cases/authority/cap_ambient.rs
```

- Line 1, `//@ rules: CAP-1`: which rule this case tests. `CAP-1` is a row in
  `conformance/rules.md`, which points to its section of the design.
- Line 2: a comment saying what the case shows: world operations exist only
  as capability methods.
- Lines 3–5: a function that tries to read a config file the Rust way.
- The end of line 4, `//~ E0425 TD0501`: the expected verdict for that line.
  Two checkers must agree. Stable rustc, compiling against a stub of Tide's
  std (a stand-in with the same names, not the pinned nightly the driver
  uses), must report `E0425` (no such function, since Tide's std has no
  `std::fs::read_to_string`), and the driver must report `TD0501`.
- Lines 7–9: the same for reading the clock. `E0599` is rustc's "no such
  method".

The conformance runner (`cargo run -p tide-conformance`) runs every case
through stable rustc (with the stub std) and through the driver, and fails if
either verdict differs from the marks. 389 such cases
(on `main` today) are the checker's specification in executable form. To see the cases for one
rule: `bash -c 'grep -rl "rules: CAP-1" conformance/cases'`.

## The hardest question

> "You built on rustc's internals, on a nightly. What happens when Rust
> changes? And how do you know your checker is right?"

**The coupling is real, and it's a cost Tide chose.** The driver links
seventeen of rustc's internal libraries (`rustc_*` crates), which have no stability promise.
Tide pins one nightly and builds everything with it. The pin hasn't moved
since it was set. Moving it means porting the driver to the new internals and
reviewing every new item in Rust's lower layers against the allowlist. The
2026-11-15 adversarial review, code-named Achilles (finding W5), rates this
serious but slow-burning. Its cheapest test is a rehearsal bump of the pin,
and its mitigation is bumping monthly.

**Why accept it:** it lets Tide reuse rustc's type and borrow checker, and
rustc and LLVM's code generation, instead of building and maintaining its
own compiler. Earlier drafts planned that, with three backends. The checked
dialect (D18) was chosen because it's far cheaper, and every Tide program
stays a Rust program.

**How we know it's right:** the conformance corpus runs every case through
both plain rustc and the driver and compares verdicts with the expected ones.
In the fx-ip4x landing run on `main`, the conformance runner, then a shell script, reported
369 PASS, 3 AWAITS CHECKER, 0 FAIL (372 cases), and `cargo test --workspace`
passed 342 tests. **AWAITS CHECKER** means rustc's verdict and the driver's
existing codes match, but the case also expects a code the driver doesn't
enforce yet (here `TD0205`, `TD0808`, `TD0809`). The runner still exits 0, so
a passing run doesn't mean every expected rule is built. The red team
(fx-qlek.5) then added 17 cases, 389 in all, and both suites passed at its
landing (bead fx-qlek.5). A gate can still have gaps; chapter 3 is about the
ones found so far.


**Also say:** each check today is a full rustc run, taking seconds. Tide's own
fast incremental front end, aimed at tens of milliseconds per edit (D12), is
planned and not built.

## Going deeper

- `design/01-design.md` D18 (the checked dialect), D12 (checker architecture
  "A, then B": A is today's driver on rustc, which stays the authority; B is
  Tide's own fast front end, planned, tested against A).
- `design/03-authority-concurrency.md` §2.5 (ADMIT-1, the allowlist and the
  backstop; ADMIT-2, the driver owns rustc).
- `docs/driver.md`: every check the driver enforces today.
- `conformance/README.md` and `conformance/rules.md`.
