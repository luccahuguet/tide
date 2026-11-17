# Capabilities: what a function may touch is in its signature

**Feature:** capability-passing I/O and authority-free functions
(`design/01-design.md` D7; `design/03-authority-concurrency.md`, CAP-1 and
§3.2 AFREE-1).
Tide's std has no free function that reads files, the clock, the environment,
or the network. Code reaches the outside world only through a capability value
it is handed (`Fs`, `Clock`, `Net`, ...), so a function's parameter types
bound what it can do. Admission enforces this with named codes.

| File | What it is | Declared outcome |
| --- | --- | --- |
| `config.rs` | A config loader: `load(fs: &Fs, path)` reads one file through the `Fs` it is handed; `parse(text)` is declared `#[pure]`. Its tests hand `load` a simulated file system (`Root::sim`, `sim.world.file`). | its tests pass |
| `ambient_read.rs` | `load` written as a Rust habit: it calls `std::fs::read_to_string` and takes no `Fs`. | admission rejects it with `TD0501` (ambient authority), beside rustc's `E0425`: Tide's std has no such function |
| `pure_reads_clock.rs` | `parse` handed a `Clock` to stamp when the config was read. Passing the capability is legal, but `parse` is declared `#[pure]`. | admission rejects it with `TD0511` (a `#[pure]` function takes authority) |

## What it shows

- The only file access `load` has is the `Fs` it is handed. Reaching past it
  for Rust's ambient `std::fs::read_to_string` fails admission with
  `TD0501`.
- `parse` is declared `#[pure]`, so an edit that hands it a capability, here a
  `Clock`, fails admission with `TD0511` instead of quietly widening what it
  can do.
- Because `load` takes its `Fs` as a parameter, its tests pass a simulated
  one and create the file in it, so they need no temporary directory and no
  fake file-access trait.

## What the Rust version would need

Rust's std lets any function call `std::fs::read_to_string` or
`Instant::now()`, so the signature does not bound what a function does. To get
close:

- pass handles by convention, for example `cap_std::fs::Dir` from the
  `cap-std` crate, and forbid the ambient APIs with Clippy's
  `disallowed_methods` and `disallowed_types` lists. Those lists name each
  path to forbid, and they apply only to the crates Clippy lints with that
  configuration, not to dependencies;
- to test `load` without the real file system, put file access behind a
  trait with a fake implementation; a temporary directory also works, but the
  test then reads and writes the real file system;
- for `parse`, nothing in the language marks a function as authority-free;
  review has to read its body and everything it calls.

## Run it

`cargo test --workspace` checks every declared outcome, through the examples
runner in `crates/tide-admit/tests/examples.rs`; to run only the examples:

```sh
cargo test -p tide-admit --test examples
```
