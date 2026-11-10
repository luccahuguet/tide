Model: gpt-6.1-sol · Effort: xhigh · Date: 2026-11-09

## Verdict

**request_changes — high confidence (0.96).**

The ownership direction is sound. Tide needs its own cancellation-aware error and I/O traits, and sharing their definitions with conformance is appropriate. The candidate is not ready: it contradicts TASK-6, changes documented buffered-seek behavior, loses provided-method overrides through forwarding implementations, and can duplicate output after a partial flush followed by a panic. Five blocking findings follow.

The narrow acceptance example works against production std in my runtime check. The implementer's driver/conformance/E1 results are plausible, but I did not independently reproduce those gates. Passing that example and the seven unit tests does not establish the broader Rust-compatible behavior claimed by the report and design edit.

## Scope

Reviewed `0583ecb..db09517`. `git diff 0583ecb db09517 | sha256sum` produced the brief's exact `71c075dd21b02deb17ec4ee665cf55f732cef5beec20c2662c2440cb58040af8`.

Read AGENTS.md, GOVERNANCE.md, README.md, `br --no-db show fx-ecb`, `br --no-db show fx-nkv`, the implementation report, all nine changed files, the surrounding crate-root definitions and admission crate-identity/obligation/backstop code, the production entry tests and relevant conformance cases, conformance's README/rules/runner, and the requested specification sections: 01's tier table/D11/D18, 02 §9, 03 §1.1 and TASK rules, reference §1/§6.9. Compared implementation details with the installed pinned Rust sources and official Rust documentation. No E1 task or solution source was used to construct probes.

All executable checks ran in a private archive created by:

```sh
review_tmp=$(mktemp -d /tmp/fx-ecb-codex.XXXXXX)
git -C /home/lucca/pjs/lab/tide archive db09517 | tar -x -C "$review_tmp"
```

The resulting directory was `/tmp/fx-ecb-codex.cHFHEI`. Toolchain: `rustc 1.99.0-nightly (771916f90 2026-08-08)`, the repository's nightly-2026-08-09 pin. Independent probes:

- **P1 — Candidate unit tests.** Built `crates/tide-macros/src/lib.rs` with `rustc --edition=2024 --crate-type=proc-macro --crate-name=tide_macros --extern=proc_macro`, then `rustc --edition=2024 --test --crate-name=tide_std crates/tide-std/src/lib.rs --extern=tide_macros=review-artifacts/libtide_macros.so -Coverflow-checks=on -o review-artifacts/tide-std-tests`. Running that binary passed all seven candidate I/O tests.
- **P2 — Production runtime example.** Built the actual production `lib.rs` as an rlib with the same compiler/overflow flags. Extracted only the raw source literal from `io_core_builds_and_runs` in the candidate's `tests/build.rs`, compiled it with `--extern=std=review-artifacts/libtide_std.rlib -Ldependency=review-artifacts`, and ran it. Exit 0; output was exactly `1: alpha beta`, `2: gamma`, and `words=3 rest=yz missing=entity not found NotFound cancelled=false true custom=bad header`. This checks runtime behavior against production std, **not admission**.
- **P3 — Failure and compatibility probes.** `rustc --edition=2024 --test review-probes.rs -o review-probes && ./review-probes --nocapture`. This scratch harness includes the candidate's unchanged `io/shared.rs` directly. Final result: **13 tests, five passed, eight failed**, exit 101. The eight failures cover B1–B5, including both `&mut` and `Box` variants in B4. Passing probes checked short reads plus `Interrupted` then `Cancelled`, partial valid UTF-8 preservation and error precedence, short/zero writes plus `Interrupted`, recovery of the unwritten buffered tail after `Cancelled`, and the finite cancellation-spin observation in N1. Each failure's input/output is described below.
- **P4 — Inherent-method compatibility.** `rustc --edition=2024 review-relative.rs -o review-relative` exited 1/E0599. The same source contains both Rust and Tide versions of `BufReader::new(Cursor::new(b"abc")).seek_relative(0)` without importing `Seek`; only the Tide call fails.
- **P5 — Unwind cleanup without catching a panic.** `rustc --edition=2024 review-unwind.rs -o review-unwind && ./review-unwind` exited 101 for the deliberate writer panic and printed `cleanup wrote ababcd`. A user-owned outer destructor explicitly retries `flush` during unwinding. This demonstrates B5 without depending on a Tide `catch_unwind` API.

I queued `cargo test --workspace --offline` using the prescribed `heavy.sh timeout 3600 ... </dev/null`. The scheduler's quiet window extended to **2026-11-10 01:02:54 UTC / November 9 22:02:54 São Paulo**; compilation never started. I cancelled that waiting invocation and a waiting probe wrapper, then used the small direct-rustc checks above. No Cargo build/test bypassed the scheduler. Workspace, admission, conformance, and E1 results in `impl-report.md` remain the implementer's claims, not independent results from this pass.

No repository/worktree source or Beads state was changed. Only private scratch artifacts and this requested report were written.

## Blocking

### B1 — `BufWriter::drop` performs the I/O that TASK-6 expressly forbids (P1)

**Locations:** `crates/tide-std/src/io/shared.rs:1333`; corresponding expectations in `crates/tide-std/src/io/mod.rs:342` and `crates/tide-admit/tests/build.rs:403`; added compatibility claim at `design/03-authority-concurrency.md:183`.

TASK-6, `design/03-authority-concurrency.md:1404`, says std Drop does no I/O beyond closing descriptors and an unflushed buffered writer warns. The candidate instead invokes `flush_buf`, potentially calling arbitrary `W::write` repeatedly, and silently ignores errors. P3 buffers `b"pending"` into `BufWriter<&mut Vec<u8>>`; its backing vector is empty before Drop and contains `pending` afterwards. The unit and driver tests actively depend on this behavior.

For a capability-backed writer this makes scope exit or panic unwinding attempt unrequested writes, potentially block, or silently lose buffered output on cancellation. The core owns this behavior; implementing File/TCP later cannot reconcile it without changing this code. This is a v0 contract conflict, not later-tier text for fx-djp.

**Smallest fix:** make Drop discard pending output and warn, leaving descriptor cleanup to the inner writer's normal Drop. Keep actual writes in explicit `flush`/`into_inner`; change the acceptance example to flush explicitly and replace the unit's flush-on-drop assertion with a no-write-on-drop check. Qualify the new design/module text with this existing Tide exception. Do not weaken TASK-6 to preserve the implementation.

### B2 — Querying a buffered reader's position destroys its buffer (P2)

**Location:** `crates/tide-std/src/io/shared.rs:1159`, which implements only `seek` and inherits `stream_position` from `:495`.

P3 wraps `Cursor::new(b"abcdef")` in a capacity-four BufReader and reads one byte. Both Rust and Tide then buffer `b"bcd"`. Both position queries return 1, but Tide's `stream_position()` empties the buffer and seeks the inner cursor backwards; Rust leaves `b"bcd"` buffered. Code inspecting `buffer()` after an otherwise observational query therefore changes behavior, and a reader with a separate position query can unnecessarily incur a failing underlying seek.

Rust explicitly documents buffer preservation for this method. [Rust BufReader::stream_position](https://doc.rust-lang.org/std/io/struct.BufReader.html#method.stream_position)

The existing test at `io/mod.rs:266` calls the query only with an empty buffer, so it cannot detect this.

**Smallest fix:** override `BufReader::stream_position` using the inner reader's `stream_position` minus the unread-buffer length, including Rust's checked-subtraction behavior. Add one regression with unread buffered bytes and assert both the returned position and retained buffer.

### B3 — `BufReader::seek_relative` is missing as an inherent API and loses buffered-seek semantics (P2)

**Locations:** `crates/tide-std/src/io/shared.rs:1107`, `:1159`, and inherited default at `:499`.

P4's minimal expression compiles with Rust's BufReader and fails with Tide's unless the caller adds a `Seek` import. The pre-change conformance re-export supplied this inherent method. With `Seek` imported, P3 reads one byte into a capacity-four reader, then calls `seek_relative(-1)`: Rust retains `b"abcd"` in its buffer; Tide discards it and seeks the inner reader. This also turns an in-memory adjustment into a possible underlying error/checkpoint.

Rust exposes the inherent method and specifies that movements within the buffer retain it. [Rust BufReader::seek_relative](https://doc.rust-lang.org/std/io/struct.BufReader.html#method.seek_relative)

The report's description of this omission as merely a slower correct default misses the source-compatibility failure and observable buffer behavior.

**Smallest fix:** add the inherent method with the small in-buffer adjustment and fallback to `seek` outside the buffer; have the trait override delegate to it. Check a call without a `Seek` import and one backward move within buffered data. No additional I/O feature is required.

### B4 — Borrowing or boxing an I/O object loses its provided-method overrides (P2)

**Locations:** `crates/tide-std/src/io/shared.rs:645`, `:651`, `:657`, `:672`.

The `Seek` forwarding impls forward only `seek`; they rebuild `rewind`, `stream_position`, and `seek_relative` from defaults. The BufRead forwarding impls similarly omit `skip_until`.

P3 uses a legal mock whose `stream_position` returns `Ok(7)` and whose `seek` returns `Unsupported`. Calling the query directly works; calling `Seek::stream_position` through either `&mut Mock` or `Box<Mock>` fails with `Kind(Unsupported)`. A second mock supplies `skip_until -> Ok(7)` while `fill_buf` is unsupported: direct skipping succeeds, but both wrappers fail. These are four separate failing assertions. Generic helpers taking a borrowed/boxed reader therefore stop honoring the object's implementation. After B2/B3, this would also lose buffered-reader overrides through common wrappers.

Rust forwards these methods. [Rust core forwarding impls](https://doc.rust-lang.org/src/core/io/impls.rs.html), [Rust alloc forwarding impls](https://doc.rust-lang.org/src/alloc/io/impls.rs.html)

**Smallest fix:** explicitly forward the three implemented Seek convenience methods and BufRead's `skip_until` in both wrapper impls. Add a small override-observing test covering borrowed and boxed dispatch; do not expand into omitted vectored/unstable APIs.

### B5 — A panic after a successful partial flush leaves already-written bytes pending (P1)

**Location:** `crates/tide-std/src/io/shared.rs:1246`, especially the post-loop drain at `:1273`.

`written` is local bookkeeping and the buffer is drained only after the loop returns normally. A writer first accepts `b"ab"` from buffered `b"abcd"`, then panics before accepting bytes on the next call. Unwinding skips the drain, leaving all four bytes buffered despite the known successful first write. `panicked` suppresses automatic Drop flushing, but does not repair the buffer or prevent a subsequent explicit flush.

P3 retries after observing the panic: Tide produces `b"ababcd"`, Rust produces `b"abcd"`. P5 reproduces the duplicate prefix through an outer object's destructor during unwinding, with no caught panic. This can corrupt a framed stream or append log while cleanup is trying to preserve it. It remains a defect after B1 removes automatic flushing.

Rust keeps successful progress in a buffer guard that drains during unwinding too. [Rust BufWriter flush guard](https://doc.rust-lang.org/src/alloc/io/buffered/bufwriter.rs.html#216)

**Smallest fix:** use a small safe guard over the buffer and successful-byte count whose Drop removes the acknowledged prefix on normal return, error, or unwind. Preserve `panicked` protection. Add the partial-success-then-panic regression; testing a panic on the first call alone would miss the defect.

## Non-blocking

### N1 — The generic `Lines::flatten()` cancellation hazard is real, but current text does not require fusing generic Lines

**Locations:** `crates/tide-std/src/io/shared.rs:1047`, `:1070`; `io/mod.rs:120`; design/03 §1.1 at `:146` and TASK-3.

P3 supplies a reader that always returns `Cancelled`. Three direct `Lines::next` calls return three cancellation errors. A single `lines().flatten().next()` keeps requesting reads until the probe's bounded sentinel deliberately panics on the fifth attempt. No progress or termination occurs. `Bytes` and `Split` have the same general persistent-error pattern; merely applying `Iterator::fuse()` would not help because the iterator keeps yielding `Some(Err(...))`.

I **accept decision 6 for this bead**. The normative fusion requirement names inherent `Stdin::lines()`. Generic BufRead adapters otherwise retain Rust behavior, and TASK-3 requires each checkpoint to report cancellation immediately; it does not force a caller that discards errors to terminate. Each generic Lines call does return that error without retrying internally. Fusing on cancellation would introduce an additional API semantic exception and prevent resuming the same iterator inside a shield, so it should not be silently imposed here.

Document the generic adapter hazard beside Lines, recommend propagating errors, and retain this as a concrete integration concern for fx-byk/fx-hxw/fx-5vj. When Stdin arrives, test its specifically required terminal cancellation behavior and keep its authority-bearing reader/iterator non-Pure. The existing one-error-then-success mock does not represent unshielded level-triggered cancellation; label what it actually proves. This is a v0 integration note, not an fx-djp later-text finding.

### N2 — Availability and compatibility summaries have leftovers

**Locations:** `README.md:158`, `conformance/README.md:114`, `crates/tide-std/src/io/mod.rs:7`, `shared.rs:10`.

README's supported production surface omits the I/O core, and conformance's description still identifies only crate-root definitions as shared before calling every other item a stub signature. Mention the shared I/O file in both summaries. Also state the deliberate `BufWriter<W: Sized>` restriction and TASK-6 exception instead of claiming completely unchanged Rust API/behavior. These are small accuracy fixes; omitted unrelated utilities and vectored I/O need not be implemented for this bead.

### N3 — Finish the decision record when landing

The scope asks for the ownership decision on the bead as well as in design/03. It exists in design and as a drafted comment in the implementation report, but has not been posted. The landing coordinator should post the reconciled decision after the fixes, including the Drop exception. I made no Beads write in this read-only pass.

## Decisions

1. **Tide-owned I/O core — agree.** A distinct Cancelled kind requires a local error and traits returning it; aliasing Interrupted would break retry loops. Reusing Rust's data-only Cursor/SeekFrom avoids needless duplication.
2. **Shared `include!` — agree.** It follows the established root.rs mechanism and satisfies the production/conformance ownership requirement for these items by construction. The changed fs results and NetError payload correctly use the local error too.
3. **Keep FX0517 payload checks — agree.** The new keys and Role::TideStd branch match the new definitions. Constructor selection still checks the conversion's identity and concrete payload purity. The argument-based Pure walk correctly treats the non-generic Error as Pure under that invariant.
4. **Define and deny last_os_error — agree.** Keeping a resolvable item preserves FX0520 and useful help; internal capability code can still capture OS errors. The removed Rust error-constructor keys no longer own the exported API.
5. **Remove generic Lines from NOT_PURE — agree.** Purity must follow B: Lines over a Cursor is data-only; Lines over an authority-bearing reader is not Pure through its type argument. The old unconditional entry would reject valid in-memory iterators. A future hidden Stdin reader must be classified by its actual identity.
6. **Continue generic Lines after errors — agree, with N1.** The hazard is confirmed, but the current Stdin-specific rule and TASK rules do not mandate a new generic terminal state. Preserve the explicit Stdin exception when implementing it.
7. **Map unstable Rust kinds to Uncategorized — agree for the pin.** I checked the pinned kind definitions: FilesystemLoop, InProgress, TooManyOpenFiles, and InputOutputError are unstable there. Reserving Other for user errors preserves the intended distinction.
8. **Preserve messages/Debug forms — agree.** The fixed messages and Error representations are consistent with the inspected Rust definitions. This does not justify copying Rust's Drop flushing against TASK-6.
9. **Sized BufWriter with Option<W> — acceptable bounded tradeoff.** It avoids unsafe extraction from a Drop type and keeps ordinary Vec, reference, and Box<dyn Write> writers usable. It is still a public compatibility restriction and belongs in the supported-surface documentation (N2); do not describe it as exact API parity.
10. **No prelude additions — agree.** Callers explicitly import the I/O traits, as in Rust. This is distinct from B3: Rust's BufReader relative-seek method is inherent and does not need the trait import.

## Reasoning

The change has one clear runtime owner and no added dependency or wrapper layer that should be rejected. The Error ownership decision is in the bead's scope, and design/03's added bullet describes a v0 decision; the tier rule does not defer its contradictions to fx-djp. I found no new admission widening or orphaned production call caused by moving the error keys: Tide-owned items use the normal std admission path with the explicit last_os_error deny, and only Cursor/SeekFrom need the remaining Rust inherent-item allowances. The special constructor branch remains reachable from generic uses and the backstop. These conclusions come from source tracing, not a fresh driver run.

The basic algorithms handle the probed short reads, Interrupted retries, Cancelled propagation, UTF-8/error precedence, short/zero writes, and ordinary buffered-flush errors correctly. In particular, `into_inner` preserves the unwritten tail after a returned error and flushes buffered bytes without wrongly requiring an inner `flush`, consistent with Rust. BufReader's explicit seek accounting includes the two-seek underflow case and clears its buffer at the appropriate successful-seek points. The defects are concentrated in cleanup, omitted overrides, and incomplete buffered-seek API behavior.

The seven unit tests and acceptance example are useful but too weak for 1,391 lines of replacement core I/O. One test asserts the wrong Drop contract, the position test uses an empty buffer, no test observes forwarding of overrides, and none exercises a successful short flush followed by a panic. Approval needs the five small fixes with those focused regressions, then the affected production/admission/conformance checks under the normal scheduler. The deliberately missing File/TCP/Stdin implementations, owned-buffer extensions, vectored I/O, and unrelated I/O helpers remain outside this bead.

Not independently checked: Cargo suites, admission/conformance gates, E1 references, or real capability cancellation; the implementer's claimed results were not promoted to reviewer proof. Remaining risk: generic fallible adapters can spin when callers discard persistent cancellation errors, as documented in N1. DONE
