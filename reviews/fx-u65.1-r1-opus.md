Model: claude-opus-5-5 · Effort: medium · Date: 2026-11-09

# fx-u65.1 round 1, Opus review (converge round, report only)

## Verdict

**request_changes**, confidence high (0.85). The candidate meets the Acceptance as written. One FX0607 mechanism skips printing that does run while the cell is held. That contradicts SINK-8's stated reason for the lint and the report's own description of Decision 2. A conformance case also locks this behavior in. The fix is small (B1). Everything else is sound. The runner change does not make the corpus pass vacuously.

## Scope

Read: AGENTS.md; the brief; `br --no-db show fx-u65.1` and `fx-u65`; `impl-report.md`; `git diff e6fa043 baee486` (16 files); and in full or in the relevant parts: `crates/tide-admit/src/{diag.rs, walk.rs (check_expansions, flag/report, generated_by_toolchain, typeck_of, visit_nested_body, check_crate), walk/sinks.rs, walk/mono.rs (its Walk use), crates.rs (exported, is_std, key, crate_main), invocation.rs (parse, rustc_args, after_analysis, expansion check), main.rs}`; `crates/tide-macros/src/lib.rs` (print/println/eprint/eprintln proc macros); `crates/tide-std/src/{root.rs, lib.rs}` header; `conformance/rustc/std/src/lib.rs` (dbg, `__log`, prelude, sync additions); `conformance/run.sh` (whole); `conformance/README.md`; `conformance/rules.md`; all ten new and touched cases plus `sink_dbg_{debug,release}.rs`; spec: `design/reference.md` lines 50-59, 455-462, 687-692, 730-737, 975-981; `design/01-design.md` D7 (lines 446-455); `design/03-authority-concurrency.md` §2.4 (SINK-1..8), §6.4 (CHANNEL-1), LOCK-4; `trials/e1-checked/protocol.md` §7.4 and §10.2 item 3.

Ran: no compiler probes. The worktree's stub-std dependency directory (`conformance/target/admit/debug/deps`) is gone. Rebuilding it needs cargo, which the brief forbids, so I could not run the driver on a single file. I relied on the implementer's logs only as evidence of what they record: `conformance.log` (229 cases, 0 FAIL; the ten target cases and `lock_log_accept` PASS) and `runner-probes.py` plus `probe-*.log` (seven runner mutations). Quiet-window check at review time: now 1794273922 ≥ quiet-until 1794272757.

## Blocking

**B1. FX0607 skips every closure nested in the `with` callback, including ones that run while the cell is held.**
`crates/tide-admit/src/walk/sinks.rs:90-118` (`LockPrints`), and the case `conformance/cases/authority/lock_log_accept.rs:9`.

- `LockPrints` keeps rustc's default `NestedFilter` (`nested_filter::None`), so `visit_nested_body` never enters a closure inside `f`.
- The passing `lock_log_accept` case shows this: `let _later = || { eprintln!(..) };` inside `a.with(..)` gets no FX0607, and `conformance.log` records `PASS authority/lock_log_accept.rs`.
- The same mechanism therefore drops the common iterator shape: `a.with(|v| v.iter().for_each(|x| println!("{x}")))`, or `.map(|x| { eprintln!(..); x })`. That printing runs inside `f` while the cell is held. SINK-8 gives exactly that as the reason for FX0607: "since `out` can block while the cell is held". SINK-5 says `out` and `err` block under backpressure.
- The report describes Decision 2 as "lexical printing in the supplied callback", excluding only "nested bodies not executed merely by creating them". The code excludes all nested bodies, executed or not, so the mechanism does not match its stated contract.

Failure scenario: a library function writes `cell.with(|xs| xs.iter().for_each(|x| println!("{x}")))?`. The driver reports FX0508 (from `check_expansions`) but no FX0607. E1 records lint codes (protocol §7.4), so the Tide arm's FX0607 count is undercounted for the most common way to print a collection.

Smallest fix:
1. Give `LockPrints` `type NestedFilter = nested_filter::OnlyBodies;` and a `maybe_tcx` that returns `self.walk.tcx`. Closure bodies inside `f` are then visited and nested `fn` items are not, so the existing exclusion of uncalled nested `fn`s stays.
2. In `lock_log_accept.rs`, drop the `_later` closure line, or move it to `lock_print.rs` as `//~ WARN FX0607`.
3. Add one executed nested closure to `lock_print.rs`, such as `xs.iter().for_each(|x| eprintln!("{x}")); //~ WARN FX0607`.

A lexical lint that also flags an uncalled closure inside `f` is acceptable: such code is dead inside a lock callback, and the result is a warning, not a rejection. Dedup by (code, message, file, line) and by (span.lo, code) already prevents double reports.

## Non-blocking

- **N1. The reference's wording covers more macros than the rule.** `design/reference.md:51-52` says "`print!`, `println!`, `eprint!`, `eprintln!` (… a lint in libraries)", which reads as covering stderr. D7 (`01-design.md:453`, "printing to stdout from a library package"), SINK-8, reference §6.5 (`:461`) and §12.1 (`:980`) all say stdout. The implementation follows the decision record, which is correct. The reference parenthetical should later be narrowed to `print!`/`println!`. This is pre-existing and lint-only, and a writer who follows it is only more conservative.
- **N2. FX0608 catches only direct calls.** `sinks.rs:38-62` flags `Call`/`MethodCall` whose callee is a `FnDef` of `tide_std::sync::mpsc::channel`. It misses `let f: fn(&Tasks) -> _ = channel;` (an `FnPtr` callee) and `ids.iter().map(|_| ..)`-style uses that pass `channel::<T>` as a value into a std combinator (the call happens inside std). Flagging any resolved use of that def, in `visit_path`/`use_def` where `self.key(def_id)` is already at hand, would catch every creation path with less code. It would flag at the reference site rather than the call site; `channel_library.rs:8-9` would move its mark to line 8. Optional.
- **N3. Library tests get the library lints.** Under `--test` of a library, `sess.opts.crate_types` is still `[Lib]`, so `println!` in a library's `#[test]` functions gets FX0508, and `channel` in them gets FX0608. The spec doesn't say otherwise ("in a library package"), so this is acceptable. Record it, because E1's lint counts will include test code. An exemption for `#[cfg(test)]` code is a later scope call, not a defect.
- **N4. The CHANNEL-1 line reference is inconsistent.** `conformance/rules.md:112` cites `03:1418`, the current line. The table says it is "taken at `a3b0177`": the removed "Other" entry cited `03:1259`, and the neighbouring SEND-3 row cites `03:1258`. Use `03:1259`, or the table's sort order and line-number convention break. Text-only.
- **N5. README wording.** `conformance/README.md:107-108`: "without rejection unless the case also has error marks". The runner's rule is `$want$errors`, so rustc marks also expect rejection. Say "error or rustc marks". Text-only.
- **N6. A std macro that prints internally would be misread.** Both `check_expansions → sink_expansion` and `LockPrints` walk the whole expansion chain. If a std macro ever expands to a std print macro internally (for example a future real `info!` built on `eprintln!`), logging would be reported as FX0607, and a stdout-based one as FX0508. None does today: the stub's `__log` uses `format_args!`, and the real tide-std has no log macros yet. To future-proof, ignore a sink expansion whose `call_site` is itself inside a std or exported macro's expansion.
- **N7. Missing false-positive guard.** No case covers a user-defined `macro_rules! println`/`dbg` (or a non-std crate's), which must not fire. The `is_std || exported` filter handles it; one accepted line would protect it. Optional.
- **N8. Known gaps, consistent with the decisions.** `io::stdout()` writes, which are an `out` form per §2.4, are not covered by FX0508 or FX0607. Neither are callbacks reaching `with` through a generic `F`, fn pointers, or helper calls. All are acceptable lint narrowness. The real tide-std does not provide `io::stdout()` yet.

## Decisions

1. **Library printing is stdout `print!`/`println!` only. Agree.** D7, the decision record, says "printing to stdout from a library package", and SINK-8, reference §6.5 and §12.1 agree. Only the reference's line-52 parenthetical reads wider (N1, fix the wording later). `eprint!`, `eprintln!` and logging stay unlinted in libraries. No semantics change.
2. **FX0607 reach is a lexical check of the supplied callback. Agree with the reading, but not with the implementation (B1).** Lexical without transitive effect inference is the right cost for a lint. Recognizing local closure bindings, references and local `fn` items is good. "Lexical" must include closures written inside `f`; only nested `fn` items should be excluded. Also agree with counting `eprint!`, `eprintln!` and `dbg!` as "printing": SINK-5 says `err` blocks like `out`; `dbg` is not among the dropping sinks (`log`, `trace`); and D7 lists `dbg!` with the print forms.
3. **FX0507 uses the session's `debug_assertions`. Agree.** `invocation.rs` is the single owner of the profile: it accepts only `debug|release`, always passes `-Cdebug-assertions=on|off` to match, and rejects arbitrary `-C` flags, so the walk reads a derived fact and cannot be bypassed from the command line. If a CI profile is ever added, it must keep debug-assertions off or FX0507 will not fire there. Record this beside the profile parser when that happens. No CI profile now is right: none exists.
4. **`//~ WARN FX####`. Agree.** The grammar is strict: `WARN` only once, never last, and E or `error` codes after `WARN` are malformed. Severity is part of the compared key (`line:severity:code`), and warning-only cases must exit 0. The probes show unmarked, wrong-severity, wrong-code, missing, malformed, and error-marked-as-warning cases all fail, and the mixed release case passes. A missing expected diagnostic still fails, because `driven != dgot`. I see no vacuous-pass path.
5. **Fixed severities in `Code::is_lint`, with no allow or deny control. Agree.** The reference defines no lint control. `struct_span_warn` is not a rustc lint, so `--cap-lints` and `#[allow]` cannot hide it, which is the right property for E1's recorded counts. Severity has one owner (`diag.rs`).
6. **CHANNEL-1. Agree.** It names the existing §6.4 sentence without changing it, and it removes the stale "a case cannot mark" entry from rules.md, so nothing is left behind. Fix only the line citation (N4). The inline bold ID mid-paragraph in 03 is unusual, but it doesn't change semantics.

## Reasoning

- **Severity and rejection.** `diag.rs` emits FX0508, FX0607 and FX0608 with `struct_span_warn`. These don't count in `has_errors()`, so `after_analysis`'s `abort_if_errors` and the trust-artifact gate are unaffected. FX0507 stays an error. The mono backstop's `diags.is_empty()` failure signal in `walk/mono.rs` comes from a separate `Walk` that never runs `visit_expr` or `check_expansions`, so no sink warning reaches it and none is emitted twice.
- **Binary versus library.** `library_target()` reads `sess.opts.crate_types`. That is sound only because `invocation.rs` always passes `--crate-type=<kind>` (default `lib`). The runner fix (`--crate-type="$kind"` in both modes) brings the corpus in line with what the driver sees for binary tests. `tcx.crate_types()` would be wrong here, because under `--test` it is always Executable. The code comment says so.
- **Macro resolution.** `print`, `println`, `eprint` and `eprintln` are `tide_macros` proc macros that tide-std re-exports (`root.rs:20`). `exported(mac)` resolves them through renames (`use std::println as announce`, a passing case). `dbg` in the fixture std is a TideStd `macro_rules` (`is_std`). Matching on `item_name` of the resolved definition, not on the written name, is the right identity. A user-defined `println` does not fire (N7 would protect this).
- **Channel resolution.** `self.key(def)` is the canonical defining path, so renamed imports and local `FnDef` aliases resolve (both pass in `channel_library`). Coercions and indirect uses do not (N2).
- **Held-out rule.** Cases and messages use generic names (`report`, `queues`, `half`, `items:`). I saw nothing taken from E1 tasks.
- **Tests.** There is one violating case and one or two accepted cases per code. FX0507 has debug and release cases; FX0508 has library, library-accept and binary-accept (release, both modes); FX0607 has `with`, `with2`, `with_all` through a reference binding, and a fn item; FX0608 has renamed, aliased, bounded and binary cases. That is strong enough for the risk, apart from the B1 shape, which the accepted case currently asserts the wrong way.

DONE
