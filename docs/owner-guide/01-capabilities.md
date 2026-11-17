# 1. Capabilities: no ambient authority

## The idea

In most languages, any function anywhere in a program can read a file, open a
network connection, or read the clock. It just calls the standard library.
That is called **ambient authority**: the power to touch the world is in the
air, available to every line of code. The cost is that a function's signature
(its name, its inputs, and its output) tells you nothing about what it might
do. `fn parse(text: &str) -> Config` might also upload your SSH keys. To know,
you have to read its body, and the body of everything it calls, in every
library it uses.

Tide removes ambient authority. In Tide, every operation that touches the
outside world, except writing output (below), is a method on a **capability**: a value that carries the right
to do one kind of thing.

| Capability | What it lets you do |
| --- | --- |
| `Fs` | files and directories |
| `Net` | network connections |
| `Clock` | read the time, sleep |
| `Rand` | randomness (planned: today it exists but has no methods) |
| `Env` | program arguments and environment variables |
| `Stdin` | read standard input |
| `Proc` | start other programs (planned: no methods yet) |
| `Tasks` | start concurrent tasks (chapter 4) |

A function that wants to read a file must be handed an `Fs`. So its signature
shows it: `fn load_config(config: &Fs) -> String` can touch files;
`fn count_words(text: &str) -> usize` holds no capability. It can compute an
answer from its input, and print, but not read or change anything outside.

Where do capabilities come from? From exactly one place: the program's `main`
function, written `#[tide::main] fn main(root: Root)`. `Root` is the bundle of
every capability (`root.fs`, `root.net`, `root.clock`, ...). Three rules keep
it the only source:

- **Only `main` may name `Root`, and it cannot pass `root` on whole**
  (`FX0503`). It hands out single fields, so each callee gets only what it
  needs.
- **No code calls `main`** (`FX0506`), so a test can't get a real `Root`
  by calling it. (Tests get *simulated* roots instead; chapter 5.)
- **Capabilities cannot be made from data** (ROOT-4): each type has a private
  field, and none can be built from a default value or decoded from JSON.

`main` can also **attenuate** a capability, which means hand out a narrower
version of it: `root.fs.dir("/etc/app", Mode::Read)` is an `Fs` that can only
read inside `/etc/app`; `root.net.only(["db:5432"])` can only reach one host.

```text
                 main(root: Root)        <- the only source of authority
                /        |          \
  root.fs.dir(.., Read)  root.clock   (nothing)
          |                  |             |
   load_config(&Fs)   uptime_label(&Clock)   count_words(&str)
   files under /etc/app   the clock only     pure computation
```

How does Tide make this stick, given that Tide code is Rust? It compiles your
code against **Tide's own std**, which simply has no `std::fs::read` or
`Instant::now()`: those functions don't exist to call. Rust's lower layers
(`core` and `alloc`, which every Rust program gets) are still there, so the
checker also keeps an **allowlist** of the items from them that cannot reach
the world, and rejects everything else (`FX0520`). Chapter 2 covers the
checker.

One deliberate exception: **printing is not a capability**. Any code may write
to standard output and standard error (`println!`, `eprintln!`, the log
macros). The design calls these *write-only sinks*. Making them capabilities
would force agents to thread a `Root`-like bundle through every function just
to log, and the reviews found that pushes them to pass everything everywhere
(`design/01-design.md` D7, "Rejected"). The cost is real: data can leave a
program through its output. Chapter 7 comes back to this.

Two more terms you will meet:

- **`Pure`**: a type is `Pure` when no capability (and no handle shared between
  tasks) can be reached from it. A function whose inputs are all `Pure` is
  **authority-free**: its result depends only on its arguments. Several rules
  use this (chapter 4).
- **`FX` codes**: Tide's error codes, such as `FX0501`. Each names one rule.

## Run it

Two versions of the same small program. The first uses Rust habits:
[examples/ambient/src/main.rs](examples/ambient/src/main.rs).

```sh
target/debug/tide-admit --crate-type=bin --tide-std target/guide/std/libtide_std.rlib --tide-dep-dir target/guide/std --out-dir target/guide/ambient docs/owner-guide/examples/ambient/src/main.rs
```

Output (paths shortened):

```text
…/ambient/src/main.rs:6:14: error[E0425]: cannot find function `read_to_string` in module `std::fs`: not found in `std::fs`
…/ambient/src/main.rs:11:25: error[E0599]: no associated function or constant named `now` found for struct `tide_std::time::Instant` in the current scope: associated function or constant not found in `tide_std::time::Instant`
…/ambient/src/main.rs:6:5: error: FX0501: `std::fs::read_to_string` is not admitted
…/ambient/src/main.rs:11:5: error: FX0501: `std::time::Instant::now` is not admitted
…/ambient/src/main.rs:14:18: error: FX0503: `Root` may be named only as main's parameter type, in a field pattern destructuring `root`, or as `Root::sim`; pass a capability instead
…/ambient/src/main.rs:21:20: error: FX0503: the whole `root` cannot be passed, borrowed, stored, returned or captured; move or borrow its fields
error: aborting due to 6 previous errors
```

The second is the Tide way: [examples/granted/src/main.rs](examples/granted/src/main.rs).
`main` narrows the file system to one directory, read-only, and gives each
function only what it uses.

```sh
target/debug/tide-admit --crate-type=bin --tide-std target/guide/std/libtide_std.rlib --tide-dep-dir target/guide/std --out-dir target/guide/granted docs/owner-guide/examples/granted/src/main.rs
```

It prints nothing and exits 0.

## What to look at

- In the first run, each rejected line carries an `FX` code: `FX0501` for an
  ambient call (reading a file, reading the clock), `FX0503` for naming
  `Root` outside `main` and for passing the whole `root`. The `FX0503`
  messages say what to do instead.
- The two `E` lines come from rustc itself: `E0425` and `E0599` mean "no such
  function". Tide's std really has no `read_to_string` in `std::fs` and no
  `Instant::now`. The `FX0501` lines say the same thing in Tide's terms.
- The paths are printed in full; the outputs here shorten them to `…`.
- The second run prints nothing and exits 0: the program is admitted.
- Open the two files side by side. The only difference is where authority
  comes from. In the second, you can tell what each function may touch from
  its first line alone.

## Read the code

**`crates/tide-admit/src/policy.rs:64-72` and `:92-135`** (about 7 minutes).
This is the closed table behind `FX0501`: the list of Rust's ambient
operations the driver refuses, each with a message saying what to use
instead.

```sh
bash -c 'sed -n "64,72p;92,135p" crates/tide-admit/src/policy.rs'
```

- Lines 64–72, `const fn ambient(pattern) -> Rule`: a small helper that
  builds one table row. Read `const fn` as "a function the compiler runs while
  building". It says: a row made by `ambient(..)` carries code `FX0501` and the
  help text "use a capability in scope, or thread that capability through the
  parameters". The `..deny(..)` part means "everything else as in a plain
  `deny` row". That help text is in the table, but the driver prints errors
  in a short one-line format that leaves it out (chapter 3 says the same), so
  in Run it you saw only "`std::fs::read_to_string` is not admitted".
- Line 92, `pub const RULES: &[Rule] = &[`: the table itself, a fixed list
  compiled into the driver. The comment above it says the order matters:
  narrow rules come before the broad ones they carve out of.
- Lines 94–97: the first rows. `deny("tide_std::__rt::*", ENTRY)` forbids
  naming the runtime's internal entry point, the one place a real `Root` is
  made, so only `#[tide::main]` can start a program. Two `allow` rows let in
  plain network address types, which are data, not authority.
- Lines 99–124: one `ambient(..)` row per Rust file operation: `read`,
  `write`, `File::open`, `remove_dir_all`, and so on. `*` would mean "anything
  under this path", but these rows name single items.
- Lines 125–135: the same for the environment (`std::env::args`, `var`) and
  the clock (`std::time::Instant::now`).

What to take away: this list is not what makes Tide safe. Tide's std simply
doesn't have these functions, so a program using them wouldn't compile anyway.
The table turns rustc's confusing "no such function" into a Tide error that
names the rule (`FX0501`). What does the safety work is the allowlist, the
`allow` and `deny` rows further down, which admit only listed items from
Rust's lower layers.

## The hardest question

> "Isn't this just a lint, or a library like cap-std? And can't a malicious
> dependency get around it?"

**Not a lint.** A lint (such as Clippy's `disallowed_methods`) is a list of
calls to forbid. Anything not on the list gets through, and anyone can turn it
off. Tide does the reverse: the ambient functions don't exist in its std, and
from Rust's lower layers it admits only listed items (the allowlist). A Tide
program can't call `std::fs::read`, because there is no such function to
call. **Not a library either.** cap-std gives Rust capability-style file
APIs, but every other line of the program can still call Rust's ambient ones.
Tide's rule applies to every line the checker admits, within the limits
below.

**Against a malicious author, be precise.** What Tide claims today is **no
accidental ambient authority**: admitted code can't call the world's
operations (other than printing) without being handed a capability. That
rests on rustc being sound (chapter 3). A deliberately malicious
author could try to break the compiler itself, through rustc's own soundness
bugs (chapter 3), or ask for the `unsafe` grant (chapter 2). The answers
there: Tide has closed two known compiler routes, rust#25860's
function-pointer route (`FX0206`) and rust#57893 (`FX0402`); others may
remain. Test runs the driver starts (`--tide-run-test`, `--replay`) sit
inside a kernel sandbox (chapter 8); a test binary run directly gets
none. Every `unsafe` site is listed in the trust artifact (chapter 7). Kernel enforcement for deployed programs is planned for
v1, not built.

**Also say:** today the driver checks one crate at a time. There are no
dependencies yet, because packages (`tide.toml`, `tide.lock`) are planned. So
"in any dependency" holds today only because there are none. And output to
stdout and stderr is open to all code by design.

## Going deeper

- `design/01-design.md` D7 (authority) and the Thesis.
- `design/03-authority-concurrency.md` §0 (terms, CAP-1), §1 (the capability
  set), §2.1 (`Root`, ROOT-1 to ROOT-5), §2.4 (sinks), §3 (`Pure`).
- `design/reference.md` §6 (the rules as an agent learns them).
