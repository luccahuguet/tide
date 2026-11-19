# 51: fx-zu4f.13, landed-capability re-screen of the rows that failed only for Proc or Net

Bead fx-zu4f.13. The registered protocol is
`research/protocols/fx-zu4f.13-protocol.md`, with its pool in
`research/protocols/fx-zu4f.13-pool/`. Both were committed at `6dced32`,
stamped with OpenTimestamps at `7ac84b1` and upgraded at `fb03fe8` (Bitcoin
block 976549 for every file). Their SHA-256s at this record's commit equal the
registered ones (protocol `8182fcce…2e61`, pool.tsv `b2e5ec23…0a59`,
select.py `2a42cd6c…b039`; the rest are listed in section 7). One Claude
Opus run builder (RUN) did the work after tide-lead's go. This record revises
none of the source rounds' records.

Terms used below:

- **R**: the revision of `main` every row is judged against,
  `49799c2f594168f280e7a44261ab58a2d3ae9a21` (section 5). **RC**: Tide's
  release candidate.
- **Arm B** rows are past upstream bug fixes. Each is judged by its source
  round's eligibility criteria **E1-E5**, in order: E1 a non-test bug fix;
  E2 a stated symptom of this bug; E3 concurrency-shaped; E4 small and
  faithfully expressible in Tide v0 (at most 400 lines, only registered
  mappings, no external crate); E5 explorable by the simulator. The first
  criterion that fails decides the row.
- **Arm A** rows are published crate versions whose concurrent core is
  located and ported whole (fx-zu4f.10's protocol, §6). A core that is not
  portable decides the row.
- **TD codes** are Tide admission diagnostics. `TD0501` marks an ambient std
  call that needs a capability. `TD0602` marks a Rust thread spawn in place
  of a Tasks scope.
- **Leg 1 and leg 2** are the two parts of P1's caught rule. In leg 1, the
  faithful port of the malicious code draws a named TD code. In leg 2, the
  capability-passing version, which receives the authority it uses as
  parameters, is admitted cleanly.
- **tide-lead** is the coordinating agent session: it sends go, runs the
  reviews and lands the branch. **WORK** is the run's git-ignored work
  directory, `target/work` in the worktree. **TARGET** is the run's build
  directory, `/mnt/build/tide-agents/fx-zu4f.13-target`.

## 1. The pool was chosen after its rows were seen

- Every pool row was screened before this round existed. Each failed its
  own round and is listed in that round's committed record.
- The pool was defined after those rows were seen. A capability landing on
  `main` triggered it (Proc::command, Net::get, Net::resolve), not the bugs.
  The bead named two members in advance: fx-zu4f.10's B09.003 and arm-A rank
  9 (bindgen).
- chrono_anchor (fx-zu4f.5) was run again before this round, by fx-zu4f.8
  and by fx-zu4f.15 (research/49), so its outcome at R was largely known
  before go (section 4).
- A catch here is "caught on re-screen after the capability landed", never
  "caught the first time". Whether it counts toward the RC's public catch is
  fx-qlek.7's decision and its cold-reader pass's, not this record's.

## 2. Result

Pool: 4 rows. UDP-only: 1 row, listed and never counted. Resolve-only: 0
rows. Counts describe this selection only; they are not a rate.

### 2.1 Negatives

| # | Row | Outcome | Deciding criterion |
| ---: | --- | --- | --- |
| P2 | fx-zu4f.10 arm B, B09.003, bindgen `02da6536e25a` | not caught | E4: an external crate operation on the affected path, `clang_sys::support::Clang::find(None, &[])`, the first statement of `dump_preprocessed_input` (`src/lib.rs:825` at first parent `159fe962`) |
| P3 | fx-zu4f.10 arm A, rank 9, bindgen 0.73.2 | not caught (unportable) | arm-A core portability (§6 of fx-zu4f.10's protocol): an external dependency on the core path. `format_tokens` takes `&proc_macro2::TokenStream` (`lib.rs:1008`), and the bytes it writes to the child's stdin come from `tokens.to_string()` (`lib.rs:1046`) |
| P4 | fx-zu4f.11 arm B, rank 7, confeden/Antigravity `fbcbbb8b47d0` | not caught | E4: the waited-on call `powershell` goes through `no_window`, which on Windows, the only platform where the reported hang occurs, calls `std::os::windows::process::CommandExt::creation_flags`. This protocol's §5.1 lists that call as "no mapping" (`src/utils.rs:15-19, 28-32` at first parent `db8db55`). E3 passes, as written (section 3) |

### 2.2 Catches

| # | Row | Outcome | Rule met |
| ---: | --- | --- | --- |
| P1 | fx-zu4f.5, RUSTSEC-2026-0039 chrono_anchor | **caught on re-screen after the capability landed** (not independent of fx-zu4f.15; counted once, section 4) | fx-zu4f.5's rule, both parts: the faithful buggy port draws named TD codes (`TD0602` ×2, `TD0501` ×3), and the capability-passing port is admitted cleanly at R (driver exit 0, no diagnostic) |

How this catch came about: std had a gap, the fx-zu4f.8 screen exposed it,
we fixed it, and re-tested under a new registration. The gap was that the
table form `clock.sleep(d)?` did not compile in a task body. fx-zu4f.8's
capability-passing port failed on it. The fix is fx-qlek.18, and the new
registration is fx-zu4f.15 (research/49). This round's P1 applies
fx-zu4f.5's rule to fx-zu4f.5's port of the same code at R. It is not an
independent test: the catch is counted once, under research/49.

Limits of this result:
- The pool is a selection of four previously screened rows, chosen after
  they were seen (section 1).
- The checks are admission and type-checking only.
- Nothing archive-derived was executed.

Counts over this pool: 4 rows. Caught 1 (P1), the same catch as
fx-zu4f.15's, counted once under research/49 (section 4). Not caught 3: E4
2 (P2, P4) and arm-A portability 1 (P3). UDP-only 1, not counted.
Resolve-only 0.

## 3. Rows

### P2: bindgen B09.003 (fx-zu4f.10, arm B)

Judged by fx-zu4f.10's protocol (§4, E1-E5 in order), with only the
subprocess ban lifted for the operations Proc::command provides (this
protocol §5.1).

- **E1, pass.** Commit `02da6536e25a0c45dd8ee77ee37d7993721666e8`, "Drain
  the child processes stdout before waiting", changes `src/lib.rs`
  `dump_preprocessed_input`, non-test code.
- **E2, pass.** "For very large logs to stdout, this prevents deadlocks."
  (commit message,
  https://github.com/rust-lang/rust-bindgen/commit/02da6536e25a0c45dd8ee77ee37d7993721666e8).
  Under fx-zu4f.10's consistency rule an
  unhedged statement that names its trigger is a stated deadlock, as the
  source round held.
- **E3, pass.** The parent blocks in `child.wait()` while clang blocks
  writing to its full stdout pipe: a blocking cycle between concurrent
  activities.
- **E4, fail.** The affected path is `dump_preprocessed_input` at the first
  parent `159fe962b2e389dc834bb7e51c66a14cb905015c`, lines 824-897: 55
  nonblank, non-comment lines (union counted in full; fx-zu4f.10 measured the
  same 55). Its first statement calls `clang_sys::support::Clang::find(None,
  &[])` to locate the clang executable, and `clang.path` is the program the
  child runs. clang-sys is an external crate. fx-zu4f.10's E4 bans an
  external crate operation anywhere on the path, and fx-zu4f.9 §4 says an
  external crate's operation is never mapped; this round keeps that ban
  (protocol §4.1). The call is inside the affected function, not test setup,
  so the starting-state exception does not apply. The subprocess operations
  on the path (`Command::new`, `.arg`, `.stdout(Stdio::piped())`, `spawn`,
  `wait`, reading `child.stdout`) pass only through the lifted ban
  (Proc::command, fx-ayh2.1, `e0b0d62`).
- **Additional (not counted).** `File::create`, `Write::write` on that file
  and `io::copy` into `__bindgen.i`/`.ii` have no row in the mapping table
  ("anything else: unportable"); this round adds only Proc::command's rows.
- **Why the source round missed it.** fx-zu4f.10 recorded the first
  restriction it met, the subprocess ban, and did not assess the rest of E4.
  This protocol requires each row to be re-judged from E1 (§4.1), so the
  first restriction that survives the lifted ban decides the row.

### P3: bindgen 0.73.2 (fx-zu4f.10, arm A)

Judged by fx-zu4f.10's protocol, §6 (arm-A core), with the subprocess ban
lifted for Proc::command's operations.

- **Core.** As fx-zu4f.10's map: `Bindings::format_tokens` runs rustfmt with
  piped stdin and stdout, a `std::thread::spawn` writer feeds the child's
  stdin while the main thread reads its stdout, then `child.wait()` and
  `join`. Source: the verified archive (`787ef8ef…`, extracted copy, read
  only).
- **Union, measured over the named items: 164 lines.** fx-zu4f.10 left it
  partial at 107.
  - `lib.rs`: `write` 30 (956-989), `rustfmt_path` 10 (992-1003),
    `format_tokens` 67 (1006-1086), `rustfmt_non_fatal_error_diagnostic` 14
    (1089-1104).
  - `features.rs`: `RustTarget::latest_edition` 8 (212-219) and its
    `is_available` 8 (179-187, inside a same-crate macro).
  - `time.rs`: `time::Timer` `new` 7, `with_output` 4, `elapsed` 3,
    `print_elapsed` 10, `drop` 3.
  - Not counted: `RustEdition`'s `Display` impl, which formats the
    `--edition` argument.

  So the union is a lower bound, with only that small impl left out. It
  does not decide the row: the external dependency below decides
  portability on its own, whatever the count.
- **First failing criterion: external dependency on the core path.**
  `format_tokens` takes `tokens: &proc_macro2::TokenStream` (`lib.rs:1008`),
  and `let source = tokens.to_string();` (`lib.rs:1046`) produces the bytes
  the writer thread sends to rustfmt. proc-macro2 is a non-optional
  dependency of bindgen 0.73.2 (`Cargo.toml`). §6 of fx-zu4f.10's protocol
  makes "an external dependency on the core path" unportable. The core's
  child-process operations (piped stdin and stdout, `spawn`, the pipe reads
  and writes, `wait`) pass only through the lifted ban (Proc::command,
  fx-ayh2.1, `e0b0d62`).
- **Additional (not counted).** `env::var("RUSTFMT")` in `rustfmt_path` has
  no mapping. With the `logging` feature, `warn!` in the status-3 branch is
  the external `log` crate; without it, a same-crate stub. The
  `Formatter::Prettyplease` branch, behind a feature, calls `prettyplease`
  and `syn`. `write` takes a caller-supplied `impl Write`.
- No property was frozen, no model was built and §7's real-crate
  reproduction never applied: the core failed portability first.

### P4: Antigravity fbcbbb8 (fx-zu4f.11, arm B)

Judged by fx-zu4f.11's protocol (§4), with its amendments A1-A2, which
exclude owner-affiliated repositories and do not touch this row. The
subprocess ban is lifted only for the operations Proc::command provides
(this protocol §5.1). fx-zu4f.11 had passed E3 tentatively, and this
protocol requires that check in full (§4.1).

- **E1, pass.** Commit `fbcbbb8b47d0c55fee10a34c32512fc9b9183722`, "fix:
  bound every child process, so a hung helper cannot stop the program",
  changes non-test code (`src/utils.rs`, `src/dns.rs`, `src/background.rs`,
  `src/dns_forwarder.rs`).
- **E2, pass.** "Users reported an eternal hang on "Патч для Google
  серверов..." ... `Command::output()` waits forever, and nothing here had a
  limit." (commit message,
  https://github.com/confeden/Antigravity/commit/fbcbbb8b47d0c55fee10a34c32512fc9b9183722).
- **E3, pass, as written.** fx-zu4f.11's E3 (protocol, lines 451-455)
  includes "a ... deadlock or hang ... in shared state/blocking between
  concurrent activities". The parent process waits in `Command::output()`
  for a separate PowerShell child, which waits on WMI. That is a hang in
  blocking between concurrent activities. The fix bounds that wait: a
  `try_wait` loop, then `kill` past a deadline.
  - The wording does not require a cycle, a race or a schedule-dependent
    outcome, so none is required here.
  - The ambiguity, disclosed: the text does not say whether a hang caused
    by a child that stops answering on its own qualifies. It is applied as
    written, so it does.
  - The commit's second cause, slow sequential network probes, is not part
    of this E3 pass.
  - Revision history: this run first recorded E3 as failing (property log,
    08:15:11Z). Codex review round 1 overturned that as an unregistered
    exclusion, and the E4 decider below was then identified (correction,
    08:54:51Z).
- **E4, fail: an unmapped operation on the path.** The fix changes
  `src/utils.rs` `powershell` and the step that hangs, `src/dns.rs`
  `setup_dns_nrpt_with`. The waited-on operation is `powershell`, at the
  first parent `db8db55bcd583722fb3afcb3a77008296db9167b`:
  `Command::new("powershell")`, `.args([..])`, then
  `no_window(&mut cmd).output()` (`utils.rs:28-32`). The trigger is a child
  that stops answering during the Google-server patch step. `Command::new`,
  `.args` and `.output()` pass only through the lifted ban (Proc::command,
  fx-ayh2.1, `e0b0d62`).
  - **Deciding.** The hang is reported only on Windows: the CIM cmdlets go
    through WMI, and the NRPT rules are Windows DNS policy. On Windows,
    `no_window` is the `cfg(target_os = "windows")` variant
    (`utils.rs:15-19`). It calls
    `std::os::windows::process::CommandExt::creation_flags(CREATE_NO_WINDOW)`.
    This protocol's §5.1 lists platform `CommandExt` extensions, naming
    `creation_flags`, as "no mapping: E4 fails (unmapped operation)".
  - **The flag is incidental, and the negative does not hinge on it.**
    `creation_flags(CREATE_NO_WINDOW)` only suppresses a console window and
    plays no part in the hang. The row fails on the registered rule, not on
    anything about the hang. The E4 negative also stands without the flag:
    - The same path makes UDP DNS probes before PowerShell. They are a
      network operation outside §5, which §4.1 keeps banned (see below).
    - On Windows, when `egress::detect` finds an interface, that path also
      calls the `unsafe` `ws2_32` `setsockopt` FFI (`dns_client.rs:27-62`;
      `query_raw_via` binds when `if_index != 0`). FFI is an E4 exclusion
      apart from UDP.
  - **Ambiguity, disclosed.** On other targets, `no_window` is the identity
    (`utils.rs:22-24`). The evidence's hang path does not exist there, so the
    Windows variant is the affected path.
  - **Additional (not counted).**
    - The same step probes the network before it runs PowerShell:
      `setup_dns_nrpt_with` calls `nameservers_for`, then
      `resolvers::substituting_addrs`, then `dns_client::query_raw_via`,
      which sends UDP DNS queries. That is a network operation outside §5,
      and UDP has not landed. So the row is not UDP-only: the unmapped
      `creation_flags` and the `setsockopt` FFI each block it apart from
      UDP.
    - The hang's cause, WMI stopping, is an OS service. It may also fall
      under fx-zu4f.11's ban on "futex/OS wait queues/kernel mechanisms".
      §5.3's "never exits" child-model data could instead model it. This
      record does not decide that.
  - **Path accounting** at the first parent, nonblank and non-comment lines,
    measured over the named items: 186 lines.
    - `utils.rs`: `powershell` 5, `no_window` (Windows) 5.
    - `dns.rs`: `setup_dns_nrpt_with` 65, `remove_dns_nrpt` 14,
      `nameservers_for` 3, `assemble_nameservers` 21, `ps_string_list_owned`
      4, `take_over_conflicting_rules` 30, `parse_conflicts` 24,
      `pin_substituted_hosts` 15.
    - Not counted: `egress::detect`, `background::is_enabled`, the DNS-cache
      helpers, and the network helpers in `resolvers.rs` and
      `dns_client.rs`.

    So the 186 is a lower bound. The cap is not established either way. The
    unmapped operation decides on its own.
- **E5:** not assessed.

### UDP-only: Antigravity 12a36c9 (fx-zu4f.11, arm B)

| Source | Row | Operation | Other blocker named in the record | Would it otherwise meet its caught rule? |
| --- | --- | --- | --- | --- |
| fx-zu4f.11, B, rank 7 | confeden/Antigravity `12a36c940b10` (parent `5ab40fac5439`) | UDP DNS: racing provider threads call `resolvers.rs` `ask_provider`, which calls `dns_client::query_raw_via`, which does `UdpSocket::bind(("0.0.0.0", 0))`, `send_to` and `recv_from` (`dns_client.rs:336-345` at the parent) | none (tag cap:net, provisional; transport helpers not counted) | not evaluated: the protocol lists this row without porting it (§4.3), so this round neither fires nor rules out the UDP trigger |

Read only, for this check: on Windows, `query_raw_via` calls
`sys::bind_socket_to_interface` when its interface index is nonzero, an
`unsafe` FFI call to `ws2_32` `setsockopt` (`dns_client.rs:27-62`). FFI and
unsafe beyond the two mapped slots are E4 exclusions apart from UDP. This
record does not decide the row; it notes that UDP may not be its only
blocker.

### Resolve-only

None. No gated row in any source round names a system DNS lookup as its only
blocker (protocol §2.4).

## 4. chrono_anchor (P1) and fx-zu4f.15

std had a gap, the fx-zu4f.8 screen exposed it, we fixed it, and re-tested
under a new registration. The details:

- fx-zu4f.8's capability-passing port failed at `972aba0` on rustc `E0277`
  (`?` could not convert `Cancelled` into `TaskError`; research/47).
- fx-qlek.18 (`4c5b8c0`, "TaskError from Cancelled") is an ancestor of R
  (`git merge-base --is-ancestor 4c5b8c0 R` holds).
- fx-zu4f.15 (research/49) re-tested chrono_anchor under fx-zu4f.8's rule at
  R' `b1f0447` and recorded a catch. P1 here applies fx-zu4f.5's rule to
  fx-zu4f.5's port of the same quoted code at R. The two outcomes are not
  independent tests. The catch is counted once, under research/49
  (fx-zu4f.15). That is the registration the committed fx-qlek.7 texts
  already cite for this catch at R: `README.md`, lines 71 and 143, and
  `paper/tide-v0.typ`, line 210. This record adds no second catch.

### P1: the run

Judged by fx-zu4f.5's operational protocol (research/37), with §4.2's
narrowing. The checks are admission and type-checking only, and nothing
archive-derived was executed. P1 is one of four previously screened rows,
selected after they were seen. Its outcome rests on the std fix described
above (std had a gap, the fx-zu4f.8 screen exposed it, we fixed it, and
re-tested under a new registration). It is not independent of fx-zu4f.15
and is counted once, under research/49.

- **Ports.** fx-zu4f.5's own files (`buggy.rs` `393b22cb…2b0c1a`,
  `capability.rs` `a6093630…804090`), with two changes, both frozen in the
  property log at 2026-11-19T08:09:34Z, before any run:
  - In both, `REF_HOST` became `"placeholder.invalid"` (RFC 6761), for the
    archive condition's first limit. fx-zu4f.5's `"<host>"` was already
    inert; the change only makes the endpoint an RFC 6761 name.
  - In `capability.rs`, `let _ = clock.sleep(Duration::from_secs(1));`
    became the table form `clock.sleep(Duration::from_secs(1))?;`. §4.2
    bars an added `let _ =`, and the source's `thread::sleep` returns `()`,
    so the discard was fx-zu4f.5's own addition. The `?` is allowed: it is
    the table form `clock.sleep(d)?` (companion §14.2; research/47, line
    121). §4.2's own last sentence treats the table form's `?` as the form
    to judge, not as an added conversion.

  The header comments were updated to say so. Nothing else changed: the
  `bg.spawn(move || { ..; Ok(()) })` tasks, `proc.command("<program>")
  .args([..]).output()` and the threaded `bg`, `clock` and `proc`
  parameters are fx-zu4f.5's. Every mapping is in §5.1 (`command`, `.args`,
  `.output()`) or among fx-zu4f.5's rows (scope tasks, `Clock`).
- **Faithful buggy port** (`p1/buggy.rs`, SHA-256 `f117d8bf…6f61`, the
  same before and after): exit 1. The counted Tide codes are all on in-class
  operations of the affected functions:
  - `TD0602` at 19:5 and 33:5: `std::thread::spawn` in
    `fetch_ref_background` and in `submit_snapshot`.
  - `TD0501` at 21:17 and 37:17 (`std::process::Command::new`) and at 34:9
    (`std::thread::sleep`).

  rustc adds `E0433` ×3 (`std::thread` is not in Tide's std) and `E0599`
  ×2 (Tide's `process::Command` has no `new`, because a `Command` comes only
  from a `Proc`). fx-zu4f.5's run had `E0433` ×5 on the same paths. At R,
  the two `Command` paths resolve to Tide's type and fail on `new` instead.
  The TD codes are the same as fx-zu4f.5's.
- **Capability-passing port** (`p1/capability.rs`, SHA-256
  `4e3d4d89…f511`, the same before and after): exit 0, no diagnostic. Its
  interface artifact (`capability.interface.json`, SHA-256 `dbc20e74…1f43`,
  the same as fx-zu4f.15's) declares, for `run_optional_syncs`, the
  authority `bg` (sync), `clock` (clock) and `proc` (proc). This leg-2 entry
  relies on `proc.command`, `.args` and `.output()` (Proc::command,
  fx-ayh2.1, `e0b0d62`).
- **Result.** Both parts of fx-zu4f.5's caught rule hold at R (not
  independent of fx-zu4f.15; counted once under research/49). fx-zu4f.5's
  own `capability.rs`, with `let _ =`, is not this round's port, because of
  §4.2.
- **What it shows.** The outcome was predictable before go. fx-qlek.18 made
  the table form compile, and fx-zu4f.15 had already admitted the same
  authority shape. The only new fact is that fx-zu4f.5's rule, applied to
  fx-zu4f.5's port, is also met at R.

## 5. Revision and capability set

- **R** = `49799c2f594168f280e7a44261ab58a2d3ae9a21`, main's HEAD when
  tide-lead sent go. tide-lead merged R into the branch (`87f9072`).
  `git diff --quiet R HEAD -- crates Cargo.toml Cargo.lock
  rust-toolchain.toml` holds at this record's commit, and the tracked tree
  was clean for every run.
- Capability set at R (each `git merge-base --is-ancestor <commit> R`
  holds):

| Capability | Bead | Landing commit | At R |
| --- | --- | --- | --- |
| Proc::command | fx-ayh2.1 | `e0b0d6216bed121c784c76e27b10582a2e0053ed` | present |
| Net::get | fx-ff13 | `8ac13175c98ec29b3f313a860d657e1c34c1fafe` | present |
| Net::resolve | fx-6d50 | `d378a7d39e0edc70cbb409e53d28ad042f0c1606` | present |
| UDP (`bind_udp`) | none | not landed | absent |

- fx-qlek.18 (`4c5b8c0`) is in R.
- Proc::command (`e0b0d62`) lifted the subprocess ban for every pool row.
  P1's leg 2 relies on its mappings, and P2-P4 fail E4 or portability on a
  non-capability operation. Net::get and Net::resolve map no pool row
  (protocol §5.2).
- The simulated pipe capacity at R is 65,536 bytes
  (`crates/tide-std/src/process/sim.rs`, `CAPACITY`). No row reached a child
  model, so none was frozen.

## 6. Disclosure and privacy

- **Private finds by the method:** 0 (no row reached a model).
- **Public record.** P1 is a public RustSec advisory, and P2 and P4 are
  fixed upstream. P3's arm-A result is a portability negative, not a find.
- **Archive condition (P1).** The ports derive, through fx-zu4f.5's files,
  from socket-3. socket-3 is the owner-approved archived copy of Socket's
  write-up, the only source that quotes chrono_anchor's code. Endpoints use RFC 6761 `.invalid`
  hosts, nothing archive-derived was executed (admission only), only the
  authority shape is ported, and all port work stayed in `target/work`. No
  port body or archive-derived code is committed.
- **Owner-affiliated repositories:** none in the pool.
- **AI-report policies** (read for this record):

| Repository | Documents read | Read (UTC) | Rejects AI-generated reports? |
| --- | --- | --- | --- |
| rust-lang/rust-bindgen (P2, P3) | `CONTRIBUTING.md`, `README.md`, `.github/ISSUE_TEMPLATE.md` at `0d213bc3dbad03e27fe729b7fb1bb5c77cffda1d` (https://github.com/rust-lang/rust-bindgen/blob/0d213bc3dbad03e27fe729b7fb1bb5c77cffda1d/CONTRIBUTING.md); no SECURITY.md in the root or `.github` | 2026-11-19 08:14 | No rule found |
| confeden/Antigravity (P4, UDP-only) | `README.md` at `4c9711805fd8` (https://github.com/confeden/Antigravity/blob/4c9711805fd8a1c868502e22df67f8e85c659c4c/README.md), as fx-zu4f.11 read it | 2026-11-18 22:21 (fx-zu4f.11) | No rule found |
| chrono_anchor (P1) | none: a malicious crate removed from crates.io, with no maintainer channel; the advisory is public | n/a | n/a |

- **Neutral framing.** This record states the selection and each criterion.
  It does not judge the authors.

## 7. Proof

Every compile or admission run went through heavy.sh in the form
`bash ~/.cache/eon-agents/eon-overseer/heavy.sh env CARGO_BUILD_JOBS=4
CARGO_TARGET_DIR=/mnt/build/tide-agents/fx-zu4f.13-target
TMPDIR=/mnt/build/tide-agents/fx-zu4f.13-target/tmp timeout <secs> <command>
</dev/null`. A separate check confirmed that the 1-minute load was below 12
before each step (5.40, 3.76, 6.16, 5.99). The first attempt, at 08:09Z, was
held because the load was 12.26 and then 14.18; the run started at 08:13Z.

- **Setup.** The script is `target/work/ports/admit.sh` (the form of
  fx-zu4f.8's `dev.sh`), and its log is `target/work/ports/final.log`.
  - The script's SHA-256, taken after the run:
    `88042bcc186c3b5d1a37cc59a409e52dec88b0d265a5fc639ca258798f92e402`. The
    registration did not freeze the harness by digest, so this hash is not
    a pre-run identity.
  - Revision: HEAD `87f9072`, tree-identical to R for `crates`,
    `Cargo.toml`, `Cargo.lock` and `rust-toolchain.toml`, with the tracked
    tree clean.
  - Toolchain: `rustc 1.99.0-nightly (771916f90 2026-08-08)`.
- **Builds.**
  - `cargo build --locked -p tide-admit` (timeout 1500):
    08:13:02Z-08:31:59Z, exit 0, mostly spent queued in heavy.sh. Driver
    SHA-256 `17c536db…5d52`.
  - `cargo build --locked -p tide_std --message-format=json` (timeout
    1200): 08:31:59Z-08:35:05Z, exit 0. `libtide_std.rlib` SHA-256
    `66986f94…3664`.
- **Admission runs** (timeout 600 each): `tide-admit --crate-type=lib
  --emit=metadata --tide-std <libtide_std.rlib> --tide-dep-dir <dir>...
  --out-dir <out> <port>.rs`. This is library metadata mode: nothing is
  linked or run.
  - `p1/buggy`: 08:35:05Z-08:37:35Z, exit 1. Diagnostics SHA-256
    `f727bbd5…860c`.
  - `p1/capability`: 08:37:35Z-08:41:12Z, exit 0. Diagnostics SHA-256
    `be11ba37…ba91`.
- **P2, P3 and P4** needed no build or run.
  - Their sources were read only, with `git show`, from the source rounds'
    frozen clones and verified archive.
  - Line counts used `target/work/count.py` (plain `python3 -I`; comments
    and literal contents masked, nonblank lines, union counted once). It
    gives 55 for P2 and 107 for fx-zu4f.10's partial P3 set, the same as
    fx-zu4f.10's counter.
- **Registered files at this commit**, SHA-256, equal to the registration:
  - protocol `8182fccecef6ab07fc4760e258cf3f3b4472c872e3f03165c6c3ff4b835a2e61`
  - select.py `2a42cd6c32b1e305294631cc669533e2985719bbdf48462214d7ca1b7073b039`
  - pool.tsv `b2e5ec230bc7cfc14baa00f16794f78468c876f00e9fa5edd18d7bd491ec0a59`
  - udp-only.tsv `33caea8a4f156fca1790eed291364e73495038098061323b1591db76ebb601fe`
  - resolve-only.tsv `3377186263a508d4b4b53f66954bd0dfcccd1945fdcdcdf977ac927e35bdb2fb`
  - screen.tsv `7cc739ce1f9adb8412a410247241d265fb13c855c1b01bea63e67f3eea56392a`
  - selection.json `c08b3c504b19fe9758ad6f83b6f4dfd04435e66b77ee951368dc70d7736e5d2b`
- **Property log:** `target/work/property-log.md`. The P1 entry was frozen
  at 08:09:34Z, before any run.
- **Undeleted artifacts**, under
  `/mnt/build/tide-agents/fx-zu4f.13-target/ports-out/final-20261119T081302Z/`.
  None was used. Under tide-lead's answer (08:44Z), they stay unused in
  TARGET, which tide-lead cleans when it retires the run.
  - `p1-capability/libcapability.rmeta` and its `.tide-cache` copy:
    `116d8564c65194c48c42f3975419ee18652a9945400034f51a4436079cd5213a`.
  - `p1-capability/capability.interface.json` and its `.tide-cache` copy:
    `dbc20e74360d613ed6b0a1f5c7e4db955e94a9416ab04d2977ca592464371f43`.
  - `.tide-cache/…/stdout` and `stderr`, both empty: `e3b0c442…b855`.
  - `p1-buggy.diagnostics`:
    `f727bbd545107f2be6567fc8f60913704e9084f41c952f926a0cb98fe9ca860c`.
  - `p1-capability.diagnostics`:
    `be11ba37d41b9921d38659a0f60575c1c49fc9c496b29d8114d19318ae39ba91`.
  - `tide_std-build.json`:
    `8e9fd5478ac3ed3aa217b78dc64d748263e235b678621d78ceeb988d9b4578d5`.
- The port files and their diffs stay in `target/work/ports/p1/` and are
  not committed, under the archive condition.
- The branch lands by `git merge --no-ff` after GOVERNANCE.md's Merge Rule.
