Model: claude-opus-5-5 · Effort: medium · Date: 2026-11-09

# fx-ecb round 1: Opus review of `db09517` (base `0583ecb`)

## Verdict

**approve_with_alternatives**, confidence high (about 0.85).

Nothing blocks. The change meets fx-ecb's Acceptance as written. The io core matches Rust's documented behaviour in every path I compared against Rust's std. The admission and Pure changes are a correct rekeying with nothing orphaned. One design alternative (N1, cancellation-fused `Lines`) should be taken now or bound to fx-byk.1/fx-hxw before either of them lands.

## Scope

Read:

- AGENTS.md and GOVERNANCE.md's Merge Rule;
- the bead (`br --no-db show fx-ecb`) and the implementer's report;
- the full diff `0583ecb..db09517`;
- all of `crates/tide-std/src/io/{shared.rs,mod.rs}`;
- `policy.rs` in full;
- in `walk.rs`, `admit_def`, `admit_impl`, `use_generic`, `clause_obligations` and `io_error_payload`;
- `pure.rs`'s header and `NOT_PURE`;
- `root.rs`'s `__sink`;
- the stub's `io`, `fs` and `net`;
- the io conformance cases `admit_accept`, `admit_std_io_method` and `admit_io_error_generic`;
- `conformance/rules.md`'s FX0517 and FX0520 rows;
- in 03, §1.1 (Notes, including the new bullet), TASK-3 and RUST-1..6;
- in 01-design, the tier table, ADMIT-1 and the Rust-consumer bullets;
- in reference.md, §1, §2.2, §6.9 and §7.2.

Ran, on a private `git archive db09517` copy:

- **P1.** A differential probe: `shared.rs` `include!`d as module `tio`, and each test body run once against `std::io` and once against `tio`, with the outputs compared. Built with `rustc +nightly-2026-08-09 --edition 2024 --test`, under `nice -n 19` rather than heavy.sh: it is a single-file rustc, not a cargo build, and the heavy queue was held by a 45-minute benchmark. Result: 7 tests, 5 passed and 2 failed. The 2 failures are the intended detections in N4 and N5.
  - Identical to Rust's std:
    - BufReader at capacities 1, 2, 3, 5, 8 and 64 over `read_exact`, `read_line`, `seek(Current(-2))`, `read_until`, `lines` and `read_to_end`, including `buffer()` contents;
    - BufWriter over 4 writer profiles (short writes, a full writer that errors, a capacity smaller than the write), across `write`, `write_all` and `into_inner`, including `buffer()` and the error text;
    - `Take` + `lines`, `Chain` + `lines`, `split` through a 2-byte buffer, `read_to_string` on invalid UTF-8 (both `&[u8]` and a BufReader at capacity 1, including what is kept in the `String`), and the `Display`, `Debug` and `source()` of nested and custom errors.
  - Differs from Rust's std: the Cursor `read_exact` position past the end (N4) and BufReader `stream_position` (N5).
  - Confirmed: `BufReader<dyn Read>` unsize-coerces; `ErrorKind::Uncategorized` can be named (N6); `BufReader::new(AlwaysCancelled).lines()` yields 1000 consecutive `Err(Cancelled)` out of 1000 (N1).
- I did not rerun `cargo test --workspace` or `conformance/run.sh`. The implementer's batch logs cover the candidate, and per AP-PROOF-001 I did not duplicate them. My conclusions about the driver path rest on reading the code plus their run.

## Blocking

None.

## Non-blocking

**N1. Generic `Lines` (and `Split`/`Bytes`) livelock under cancellation.** Recommended alternative: take it now.

- Where: `crates/tide-std/src/io/shared.rs:1054-1073` (also `Split` at 1029 and `Bytes` at 893).
- Scenario, once fx-byk.1 gives `File` a `Read`: `for line in BufReader::new(file).lines().flatten() { .. }` runs in a scope that is then cancelled. Under TASK-3 every read returns `Cancelled`, `flatten` drops each `Err`, and the loop never ends. The scope cannot exit while it waits on the task, so cancellation, whose job is to stop work, pins a core instead. P1 shows the iterator never ends.
- This is not reachable today, because no in-memory reader returns `Cancelled`. The design does not require a fix now either: 03 §1.1 fuses only `Stdin::lines()`.
- But the design's own pattern points one way. It fuses `Stdin::lines()` after a cancellation error for exactly this reason (03:152-154), and `ReadDir` ends after its first error (03:139-140). `BufReader<File>` and `BufReader<TcpStream>` lines are at least as common as stdin lines.
- The implementer's objection was that fusing "would drop lines a shielded caller could still read". It costs nearly nothing: once a read in this scope returned `Cancelled`, every later unshielded read does too. A caller that wants to keep reading inside a shield uses `r.by_ref().lines()` or `read_line`, and the reader keeps its buffer either way.
- Ownership also argues for now: this bead owns `Lines` (AP-OWNERSHIP-001). If generic `Lines` fuses, fx-5vj can return `io::Lines<_>` from `Stdin::lines()` and inherit 03's stdin rule instead of building a second iterator type.
- Smallest fix: a `done: bool` in `Lines` and `Split`, set when `next` yields an `Err` whose `is_cancelled()`, after which `next` returns `None`. Optionally the same for `Bytes`. One unit test (the existing `read_retries_interrupted_but_not_cancelled` case already holds a `Lines` that would assert `None` after the cancelled item; today it asserts that the next line is still read). Then one sentence in the new 03 §1.1 bullet: "`Lines`, `Split`, and `Bytes` end after yielding a cancellation error", and 03:152 can refer to it.
- If not taken now: add an Acceptance line to fx-byk.1 and fx-hxw ("`BufReader::new(h).lines().flatten()` ends in a cancelled scope"), so the hazard cannot land silently.

**N2. Rust interop cost of Tide-owned io types is unrecorded.** For fx-djp: the fix touches only v1 text.

- Where: 03 RUST-6 (03:1994-2000) and 01-design:829-834. Both still list only `HashMap`, `HashSet`, `Arc`, channel ends and capabilities as the Tide-owned types that need adapters.
- Now `io::Error`, `ErrorKind` and the `Read`/`Write`/`Seek`/`BufRead` traits are Tide-owned too. A RUST-1 crate that takes `impl std::io::Read`/`Write` (`csv`, `serde_json::from_reader`/`to_writer`, `flate2`, `tar`) cannot take a Tide `File` or `BufWriter`, and a RUST-6 consumer sees `tide_std::io::Error`.
- Decision 1 is still right: reference 7.2 names `io::ErrorKind::Cancelled`. But RUST-6's list should name the io types, and RUST-1 or a later bead should name the bridge: an adapter type implementing Rust's traits that maps `Cancelled` to a Rust `io::Error` with a payload, plus the reverse.

**N3. Commonly used Rust io items are missing.** Not a defect against the bead's scope list.

- `std::io::prelude` (`use std::io::prelude::*;` is the idiom in Rust's own `File` docs) costs four lines in `shared.rs`: `pub mod prelude { pub use super::{BufRead, Read, Seek, Write}; }`. `io::copy`, `io::read_to_string`, `io::empty` and `io::sink` are cheap too.
- `From<TryReserveError> for io::Error` exists in Rust 1.96 (reviews/round36-consistency.md lists it) and is missing here, so `v.try_reserve(n)?` in an `io::Result` fn fails to compile.
- The report's Gaps list names most of these but not `prelude` or `TryReserveError`. I would add `prelude` now; it is cheap and agent-written Rust uses it. The rest can wait for one follow-up bead, filed when E1 or a later task needs them.

**N4. `Cursor::read_exact` past the end leaves the position unchanged.**

- Where: `shared.rs:790-797`.
- Rust sets `pos = len` on failure. Here a cursor with `position() > len` keeps its position. P1: std gives `pos=3`, this code `pos=10`. It is observable only after `set_position` beyond the data.
- Fix: on `Err`, `self.set_position(self.get_ref().as_ref().len() as u64)`, as Rust does.

**N5. `BufReader::stream_position` discards the buffer.**

- Where: `shared.rs:1159` uses the default `stream_position`, which is `seek(Current(0))`.
- Rust overrides it to return `inner.stream_position() - remainder` and keep the buffer. P1: std keeps `buf=[98,99,100]`, this code empties it. For a `File`, every `stream_position()` then costs an extra seek plus a refill read, and each is a checkpoint.
- Fix: add Rust's 4-line override. `seek_relative`'s in-buffer path is the same kind of gap (already listed in the report).

**N6. `ErrorKind::Uncategorized` can be named and constructed.**

- In Rust the variant is unstable. Here it is a public `#[doc(hidden)]` variant, so `ErrorKind::Uncategorized.into()` compiles.
- It is harmless and I see no unsafe-free alternative that still decodes OS kinds faithfully. I agree with decision 7. Optionally the 03 bullet or the variant's doc can say "not for programs".

**N7. A panic inside `flush_buf` keeps already-written bytes in the buffer.**

- Where: `shared.rs:1246-1275`.
- Rust's `BufGuard` drains what was written even on unwind. Here the `drain` is skipped. With `panicked` left `true`, `Drop` will not rewrite them. But a caller that catches the panic and then calls `flush()` resends them.
- Only reachable through `catch_unwind` around a panicking writer. Fix if cheap: drain in a small guard struct, or note it.

**N8. Nit: the stub's crate doc has one 117-column line.**

- Where: `conformance/rustc/std/src/lib.rs:14` ("...keeps their API. Each of its own bodies is `unimplemented!()`, so").
- It needs rewrapping, and "Each of its own bodies is `unimplemented!()`" is now slightly loose, since the included io core has real bodies. "its own bodies" arguably excludes included items, so this is text only.

## Decisions

1. **Tide-owned io types: agree.** Reference 7.2 names `io::ErrorKind::Cancelled`. `Interrupted` is retried by Rust's provided methods, and no stable spare kind exists, so `ErrorKind`, and with it `Error` and every trait returning it, must be Tide's. Keeping `Cursor`/`SeekFrom` as Rust's under the orphan rule is the minimal split. Record the Rust-boundary cost (N2, for fx-djp).
2. **Shared `include!`: agree.** It follows `root.rs`'s precedent and makes fx-nkv equivalence hold by construction. The one constraint it adds is that `shared.rs` must compile on stable 1.96 as well as the pin. `conformance/run.sh` enforces that, and the file says so.
3. **FX0517 rekeyed: agree.** I checked that `walk.rs`'s `Role::TideStd if IO_ERROR_NEW` arm sees the same `E: Into<Box<dyn Error+Send+Sync>>` clause as before, and that `Error::other` → `Error::new` inside tide_std is not walked. The other ways into `Error` carry no user payload: `From<ErrorKind>`, `From<IntoInnerError<W>>` (drops `W`) and `from_raw_os_error`. So treating `tide_std::io::Error` (no type arguments) as `Pure` matches 02 §9. Removing Rust's `alloc::io::*`/`core::io::*` allows also turns any remaining path to Rust's `io::Error::new` into an unlisted, rejected key, which is stricter, not looser.
4. **`last_os_error` defined and denied: agree.** It keeps FX0520 with `OS_ERROR`'s help, instead of a bare E0599. Policy order is right: the `tide_std::io::Error::last_os_error` deny precedes TideStd's default admit. `from_raw_os_error` moves from an explicit Rust allow to TideStd's default admit, with the same meaning.
5. **`io::Lines` dropped from `NOT_PURE`: agree.** That entry was for a type that never existed. With generic `Lines<B>`, purity follows `B`, which is 02 §9's rule. fx-5vj must put its stdin reader type in `NOT_PURE`. That note belongs on fx-5vj and in the bead comment, where the report puts it.
6. **`Lines` keeps Rust's continue-after-error: disagree, mildly.** It is not wrong for this bead, but the better choice is to fuse after a *cancellation* error only, keeping Rust's behaviour for every other error (N1). It is cheap now, it matches 03's stdin and `ReadDir` precedents, and it gives one owner for the rule. At minimum, bind it to fx-byk.1's and fx-hxw's Acceptance.
7. **Unstable kinds → hidden `Uncategorized`: agree** (N6).
8. **Rust's fixed messages and Debug shapes: agree.** P1 confirms they match.
9. **`BufWriter<W: Write>` without `?Sized`: agree.** Moving `W` out of a `Drop` type needs `unsafe` or an `Option`, and `BufWriter<dyn Write>` is rare. `BufReader` keeps `?Sized`, and P1 confirms the unsize coercion works.
10. **No prelude items: agree** for the crate prelude. But add `std::io::prelude` (N3), which is a different thing.

## Reasoning

**Acceptance.**

- The driver program in `tests/build.rs::io_core_builds_and_runs` covers every clause:
  - `BufReader` over `&[u8]`, then `BufRead::lines`;
  - `Write` through `writeln!` and `flush` on `&mut impl Write`, and `BufWriter::write_all` with drop-flush into a `Cursor`;
  - `Seek` and `Read::read_to_string`;
  - `ErrorKind::NotFound.into()` through `?`, and `io::Error::new`;
  - `is_cancelled()` both true and false.
- The test builds it through the driver against production std and runs it, comparing stdout exactly.
- The E1 B3/B4/C10 diagnostics the report quotes no longer contain any io item, and the harness ran references only, which the held-out rule allows.
- The 7 `tide_std` unit tests execute the adapters, `IntoInnerError`, Cursor writes over all five backing types, and the error accessors.
- fx-nkv equivalence holds by construction through the shared `include!`.

**Correctness.**

- I compared each provided method and impl line by line with Rust's std source: `read_to_end`/`read_exact`/`append_to_string` (UTF-8 guard and error precedence), `write_all`/`write_fmt`, `read_until`/`skip_until`, `Lines`/`Split`/`Bytes`, `Chain`/`Take`, the `&[u8]`/`&mut [u8]`/`Vec<u8>` impls, Cursor's read/seek/`slice_write`/`vec_write`, BufReader's read-bypass/`fill_buf`/`consume`/`seek`, and BufWriter's `write`/`write_cold`/`flush_buf`/`into_inner`/`Drop`.
- They match, except for the N4, N5 and N7 edge cases. Retry is on `Interrupted` only, never `Cancelled`, which is the point of the change and is tested.
- P1's differential run confirms observable equality across buffer sizes and short or failing writers.

**Admission.**

- `core::io::cursor::Cursor::*` and `core::io::seek::SeekFrom::*` cover the inherent items of the two re-exports. Their trait impls (`Clone`, `Default`, `Debug`, `PartialEq`) pass `admit_impl` through an admitted trait and an exported head.
- Removing `core::io::*` and `alloc::io::*` narrows what is admitted, and no reachable path needs them now that the traits and error are Tide's. `root.rs`'s `__sink` uses Rust's `std::io` internally, so it is not walked.
- No conformance key, case or rules.md row names the removed keys. The remaining `alloc::io`/`core::io` mentions are in `reviews/` and `archive`, which are history.

**Design edit.**

- The new 03 §1.1 bullet is within the bead's Scope ("decide and record ... in design/03"). It is accurate and consistent with reference §1's "keeps Rust's API".
- It edits v0 text, so N1's proposed sentence would also be v0 and is in-round if taken. N2 is v1 text only and goes to fx-djp.

**Tests.** They are strong enough for the risk. The one gap that matters is cancellation behaviour of adapters, which N1 would add.

DONE
