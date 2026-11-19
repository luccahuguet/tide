# Rule inventory

Every v0-core normative rule in `design/02-memory.md`,
`design/03-authority-concurrency.md`, and `design/04-contracts.md` that makes
rustc or the Tide checker accept, reject, or warn on a program at compile time, taken
at `a3b0177`, plus runtime rules exercised by cases that also run against
production std. The chapters own the rules. The runner
(`cargo run -p tide-conformance`) reads IDs from the first column, and its
`--trace` gives each ID's cases.

Columns: **ID** is the rule's ID, `none yet`, or `= ID` for a row that
restates a rule whose ID another chapter defines. **Decided by** is `rustc`
(plain rustc rejects it against Tide's std), `checker` (it needs the Tide
checker, the admission driver, or the contract macros), `both`, or `runtime`
(a driver test executes the case against production std). **Codes**
are the TD codes the design gives (`ref` is `design/reference.md`, `comp`
its companion) and the rustc errors expected; a rustc code the design does
not cite is a best mapping that a case must confirm. Codes were spelled
`FX####` before bead fx-qlek.12 landed (2026-11-17); the numbers are
unchanged, so an `FX` code in a historical record means the `TD` code with
the same digits.

## 02-memory.md

| ID | Section (file:line of the definition) | Rule, one line | Decided by | TD code(s) / rustc |
| --- | --- | --- | --- | --- |
| MOVE-1 | §1.2, 02:43 | `Copy` never applies to `&mut T` or to a type with a destructor | rustc | E0184 / E0204 |
| MOVE-2 | §1.2, 02:44 | Reading or borrowing a moved-out place before it is reassigned is an error | rustc | E0382 |
| MOVE-3 | §1.2, 02:47 | Nothing moves out of a place behind a reference, or out of a field of a type with a destructor | rustc | E0507 / E0509 |
| MOVE-4 | §1.2, 02:50 | A closure captures each place path in the weakest mode its body needs (shared borrow, mutable borrow, or move); `move` forces by-value capture | rustc | E0382 / E0502 / E0503 |
| LOAN-1 | §1.4 Reborrows, 02:68 | A reborrow's origin includes each dereferenced reference's origin, from the nearest inward, stopping after the first shared one | rustc | uncoded "lifetime may not live long enough" |
| LOAN-2 | §1.4 [Loan], 02:84 | An access that conflicts with a live loan is an error | rustc | E0499 / E0502 / E0505 / E0506 |
| LOAN-3 | §1.4, 02:92 | Assigning to a reference variable `r` kills the loans of places reached through `*r` | rustc | E0499 |
| LOAN-4 | §1.4 Implicit reborrow, 02:95 | A `&mut T` used where a `&mut T` is expected is reborrowed, not moved | rustc | E0382 |
| LOAN-5 | §1.4, 02:99 | Two-phase borrows apply only in Rust's three cases | rustc | NLL |
| ORIGIN-1 | §2.1, 02:116 | A value of `&ρ1 T` flowing into `&ρ2 T` needs `ρ1 ⊆ ρ2` | rustc | lifetime errors |
| ORIGIN-2 | §2.2, 02:146 | Well-formedness of `&ρ T` needs every origin of `T` to be `⊆ ρ` | rustc | E0491 / implied bounds |
| LIFE-1 | §2.3 grammar, 02:160 | Lifetime parameters come before type parameters | rustc | parse error |
| LIFE-2 | §2.3 grammar, 02:169 | `use<..>` only on a return-position `impl Trait`, listing every in-scope type parameter; not while an argument-position `impl Trait` is in scope | rustc | precise-capturing errors |
| LIFE-3 | §2.3, 02:187 | Rejects lifetime parameters on traits, explicit `for<'a>`, and GATs (beyond D4's ceiling) | checker | TD0201, TD0404, TD0403 |
| LIFE-4 | §2.5, 02:233 | An elided output lifetime that rules 1 and 2 do not resolve is an error | both | TD0202; rustc E0106 |
| LIFE-5 | §2.5, 02:237 | Elided lifetimes in consts and statics are `'static` | rustc | E0597 |
| LIFE-6 | §2.5, 02:238 | Fields have no elision: a reference field names a struct lifetime or `'static` | rustc | E0106 |
| LIFE-7 | §2.5, 02:241 (also 02:181) | The bare name of a borrowing type in an impl header is an error | rustc | E0726 |
| LIFE-8 | §2.5, 02:243 | Associated types in impls have no elision | rustc | uncoded error ("declared without lifetime parameters") |
| ORIGIN-3 | §2.7 Bodies, 02:275 | A body may use inclusions between signature origins only from declared or implied bounds | rustc | "lifetime may not live long enough" |
| ORIGIN-4 | §2.7, 02:281 | A local loan reaching a placeholder origin (a returned reference to a local) is an error | rustc | E0515 |
| ORIGIN-5 | §2.7, 02:284 | An impl method's signature must be at least as general as the trait's | rustc | E0195; uncoded "`impl` item signature doesn't match" |
| ORIGIN-6 | §2.8, 02:292 | v0 accepts a body exactly when NLL accepts it (no Polonius), apart from CLOSURE-3's `fn`-pointer rule | rustc | NLL |
| BORROW-1 | §3.1, 02:305 | Every reference, borrowing type, and `dyn` bound in a field uses a declared lifetime or `'static` | rustc | E0106 / E0261 |
| BORROW-2 | §3.6, 02:360 | In `impl<'a> Lexer<'a>`, `fn new(src: &str) -> Self` is an error | rustc | lifetime error |
| BORROW-3 | §3.7, 02:375 | `Owned` (`'static`) needed for `Box<dyn Trait>` in a signature or field, `dyn Any`, and `spawn_isolated` closures (SCOPE-3) | both | rustc `'static` errors; TD0203 (ref:207) |
| VAR-1 | §4.1, 02:383 | Subtyping comes only from origins: `&` is covariant, `&mut T` invariant in `T` | rustc | lifetime / variance errors |
| VAR-2 | §4.1, 02:392 (table 02:396, per-parameter rule 02:405) | Variances as tabled and computed per parameter of a struct or enum (cells, `Shared`, and `dyn Trait<A>`'s `A` invariant; `fn` arguments contravariant) | rustc | lifetime / variance errors |
| VAR-3 | §4.1, 02:407 | Unsizing to `dyn` may shorten the object's origin even behind `&mut` | rustc | lifetime errors |
| DYN-1 | §5.1, 02:447 | `dyn` object-lifetime defaults: behind `&` it takes the reference's origin; elsewhere outside a body `'static` | rustc | `'static` errors |
| DYN-2 | §5.1, 02:460 | A `dyn` is `Send`/`Sync` only if it names them, and `Pure` only through a companion supertrait (`dyn Fn() + Pure` is E0225) | both | E0225 / E0277; TD0517 |
| OPAQUE-1 | §5.2, 02:471 | A return-position `impl Trait` captures every in-scope generic; with `+ 'static` the hidden type must satisfy it | rustc | capture / `'static` errors |
| OPAQUE-2 | §5.2, 02:479 | An opaque type is `Send`, `Sync`, or `Pure` only if its bounds state it (no auto-trait leakage) | checker | TD0205 (`Send`/`Sync`, defined in 02 §5.2; rustc accepts it when the hidden type has the trait); TD0517 (`Pure`) |
| CLOSURE-1 | §5.3, 02:496 | A closure that captures a borrow may be returned or stored only where its origins are named or captured | rustc | E0373 / lifetime errors |
| CLOSURE-2 | §5.4, 02:514 | With no expected type, a closure cannot return a borrow of a parameter whose annotation elides its lifetime (a named lifetime or inferred types are accepted) | rustc | lifetime error |
| CLOSURE-3 | §5.4, 02:541 | Coercing a fn item to a `fn` pointer is an error when the item (its signature, a method's impl header, a trait method's selected impl) implies an outlives bound that does not hold trivially (not `'static: 'a`, `'a: 'a`, a concrete type, or a type parameter declared `'static`), or when no impl can be selected; a syntactic conservative rejection, independent of the instantiation (closes rust#25860's `fn`-pointer route; the closure route stays closed through TD0404). Cost: no `fn` pointer from generic functions over references such as `first<T>(&[T])`, trait methods with a reference receiver such as `Clone::clone`, `&self` methods of types with a lifetime parameter, or functions such as `fn f(v: &mut Vec<&str>)`; a closure calling the item is accepted | checker | TD0206 (defined in 02 §5.4; rustc accepts) |
| DROP-1 | §6, 02:530 | "Rule DC": Drop check: every origin in `dropck(T)` must be live where `T` drops | rustc | E0597 / E0713 |
| DROP-2 | §6, 02:555 | Users cannot write a drop-check exemption (`#[may_dangle]`) | rustc (unstable feature, under ADMIT-2) | E0658 / E0725 |
| SEND-1 | §8.1, 02:606 (table 02:610) | `Send`/`Sync` are derived structurally per the table | rustc | E0277 |
| = UNSAFE-1 | §8.1, 02:623 | `unsafe impl Send`/`Sync` only in packages granted `unsafe` | checker | TD0901 (ref:953) |
| SEND-3 | §8.1, 02:626 | Task boundaries need `Send` (by value) or `Sync` (by reference) | rustc | E0277 |
| SCOPE-1 | §8.2, 02:682 | Child captures and results outlive `σ`; the parent cannot mutate, move, or free a lent place before the scope ends | rustc | E0373 / E0499 / E0505 / E0506 / E0597 |
| SCOPE-2 | §8.2, 02:690 | Neither a `Scope` nor a `Task` escapes the scope body through `T` | rustc | higher-ranked lifetime error (uncoded) |
| SCOPE-3 | §8.2, 02:692 | `spawn_isolated` takes an `Owned` closure | both | TD0203 (ref:207); E0373 / E0521 |
| SCOPE-4 | §8.2, 02:696 | A child spawned on a passed-down scope owns its data or borrows only what the signature ties to the scope | rustc | E0373 / E0621 |
| = LOCK-1 | §8.4, 02:738 | `Shared<T>` needs `T: Send + Pure`; creating one takes `&Tasks` | both | TD0606; TD0505 (ref:725); E0277 / E0061 |
| SHARED-1 | §8.4, 02:743 | `with`/`with2` payload references are higher-ranked, so nothing carrying `σ` escapes | rustc | higher-ranked lifetime error (uncoded) / E0521 |
| = LOCK-4 | §8.4, 02:747 | The `with` closure is `Pure` | checker | TD0603 |
| = PAR-5 | §8.5, 02:764 | `join` closures are `FnOnce + Pure + Send` | both | TD0610; E0277 |
| = PAR-6 | §8.5, 02:765 | `par_map`/`par_for` closures are `Fn + Pure + Send + Sync` | both | TD0610; E0277 / E0507 |
| = PAR-7 | §8.5, 02:768 | Shared captures `Pure + Sync`; `&mut` captures disjoint | both | TD0611; E0277 / E0524 |
| = PAR-4 | §8.5, 02:775 | Elements: `Pure`, plus `Sync` when shared and `Send` when split or moved | both | TD0612; E0277 for `Send`/`Sync` |
| PURE-1 | §9, 02:789 (not-Pure list 02:815) | `Pure` is decided structurally and deeply (fields, `Box`/`Rc`, greatest fixed point, opaque and `dyn` by stated bounds, type parameters by bound) | checker | TD0517, or the needing rule's code |
| = UNSAFE-1 | §9, 02:793 | `#[trusted_pure]` only in a package granted `unsafe` | checker | TD0901 (03:609) |
| PURE-2 | §9, 02:807 | A missing auto or marker trait never makes two impls disjoint | both | TD0402 (ref:307); E0119 (rustc agrees, for `Pure` via the blanket impl) |
| = ADMIT-1 | §9, 02:857 | `OnceLock` and `LazyLock` are not admitted | checker | TD0520 |
| = GLOBAL-6 | §9, 02:870 | `Arc` omits `strong_count`, `weak_count`, `get_mut`, `try_unwrap`, and `into_inner` | both | E0599 / TD0520 |
| = ADMIT-1 | §9, 02:886 | `io::Error::new`/`other` conversions must reach one of alloc's four `From` impls (no user `From`/`Into`, no reflexive impl) | checker | TD0517 (02:889) |
| = ADMIT-1 | §9, 02:893 | Generic code that wraps those constructors bounds its parameter by `Error + Send + Sync + Pure + 'static`, checked at the definition | checker | TD0517 |
| = ADMIT-1 | §9, 02:898 | `Waker` is not admitted | checker | TD0520 |
| = AFREE-2 | §9 table, 02:927 | `const fn`: `Pure + Sync` parameters, no `&mut` parameters | checker | TD0518 |
| ARENA-1 | §10.9, 02 ARENA-1; ref §3.6 | Generational Arena handles reject stale/repeated access; Id<T> is Pure for every T, Arena<T> only when T is | runtime | stale index panic; TD0517 / TD0511 |

## 03-authority-concurrency.md

| ID | Section (file:line of the definition) | Rule, one line | Decided by | TD code(s) / rustc |
| --- | --- | --- | --- | --- |
| CAP-1 | §0, 03:41 | World and nondeterminism operations exist only as capability methods; ambient std calls in the closed policy table are rejected | both | TD0501 (ref:549); rustc E0425 / E0599 / E0433 / E0432 (unresolved) |
| TCP-1 | 03 §1.1, reference §6.1/§6.7; research/21 §§1.9–1.13 | `Net::bind/connect`, attenuations, TCP addresses, accept/incoming, owned/borrowed I/O, half-close and cancellation have the same tested values/ErrorKinds on real Linux loopback and seeded simulation | rustc and driver; execution in tide-admit build tests | `authority/tcp.rs`; excludes ports/timing; post-close writes assert the allowed set |
| PROC-1 | 03 §1.1, §9.9 (lifecycle subset), reference §6.1, companion 20; research/40 §§2, 6 | `Proc::command` with `output`, `status`, `spawn`, piped stdin and stdout, `wait`, `try_wait`, `wait_with_output` and `kill` gives the same tested values/ErrorKinds on real Linux (a `main(root)` binary) and seeded simulation with declared models: exit 0 and nonzero, captured output, a missing program (`NotFound`; `ProcessModel::missing()` under simulation); an unregistered program or unsupported mode is a coverage failure; a seed that uses `Proc` replays byte for byte; argv[0] is the program as spelled; a real child sees exactly its three descriptors while the parent holds an extra inherited one, and a failed descriptor sweep fails the spawn; every outcome in research/40 §12's seed-witness table has a seed witness and its broken control fails | runtime | `authority/proc.rs`, executed on both backends by tide-admit's `simulated_processes_match_real_processes`; descriptors, refusals and the seed-witness table with controls in tide-std `process/tests.rs`; excludes PIDs and timing; real checks assert allowed sets; node crash waits for simulated nodes |
| PROC-2 | 03 §1.1 PROC-ONLY; research/40 §3 | On both backends, `Proc::only` matches programs byte for byte (bare names and absolute paths; relative paths with `/` grant nothing; lists intersect); then, under `only`, an explicitly set `LD_`/`DYLD_` or glibc unsecure variable (03's list, `GLIBC_TUNABLES` included), or a name holding `=`, is refused (`PermissionDenied`); then NUL in the program, arguments, environment or directory is `InvalidInput`; each before lookup or model lookup, starting nothing, so a model that accepts any environment still sees the refusal; bare names resolve against the start-time `PATH`'s absolute elements, never a command's own `PATH`, and the child execs the canonical path resolved in the parent before `current_dir`: an absolute entry or `PATH` element through a symlink to `/proc/self/cwd/bin`, with a `current_dir` holding another program of that name, runs the parent's file, argv[0] as spelled | runtime | `authority/proc.rs` (both backends); `PATH`, `/proc/self/cwd` symlink and refusal cases in tide-std `process/tests.rs` (`real_lookup_and_canonical_exec`, `simulated_refusals_precede_model_lookup`); `cap_ambient_paths_reject`; tide-admit `proc_command_is_admitted_with_proc_in_the_trust_diff`. Ambient `std::process::Command::new` stays `TD0501` under CAP-1, and the capability version is admitted with `proc` in the trust diff, its line naming the child as ambient (03 §4.3) |
| PROC-3 | 03 §0 CAP-1, §2.5; research/40 §§2, 12 | `tide_std::process` exposes no public path to a process except `Proc::command`'s `Command`: admission's default for Tide's std items is guarded by a test that lists its public items, `pub use` re-exports included (admission admits a re-exported item before any rule), and fails on additions; a `deny("tide_std::process::*")` glob may sit beside that test, never instead of it; backend entry points stay `pub(crate)` or under `__rt`; every type that can hold a handle is in `NOT_PURE` | checker | tide-admit `std_process_public_items_are_fixed` |
| NET-HTTP-1 | 03 §1.1 NET-HOST; research/42 §4 | `only` reduces entries, `get` URLs and `connect` addresses to one key: IPv4 aliases (`0x7f.1`, `2130706433`, `0177.0.0.1`) key as their address, IPv4-mapped IPv6 as IPv4, IPv6 spellings as one address, domains lowercased with one trailing dot dropped; entry port required; URL default ports are 80 for `http` and 443 for `https`; before the host parser, a host with a non-ASCII byte, a `%` (`b%C3%BCcher.example`, `%78n--bcher-kva.example`, `%EF%BD%81pi.example.com`), an empty label (`a..b`) or an `xn--` label is refused as an entry (grants nothing), a `Url` (`UrlError`) and a `connect` address (`InvalidInput`), while `%` in a path or query still parses; wildcard, port-less and scheme entries grant nothing; a name never matches its address (`localhost` vs `127.0.0.1`); a refused host is never resolved, dialled or shown to a peer (`Denied` with no lookup or peer call) | runtime | `tide-std` `net::http_tests::net_host_*`, `denied_and_refused_shapes_need_no_lookup`; the `connect` cases are TCP-1's |
| NET-HTTP-2 | 03 §1.1, companion 21; research/42 §§3, 5 | `Net::get` returns a 2xx body, `Status` for other final statuses (3xx not followed), and NET-ERR's variants on both backends: `Io(InvalidInput)` for other schemes and userinfo, `Timeout` at the deadline, `Cancelled` winning at the same wait, `Reset` for a truncated body, `Io(InvalidData)` for TLS handshake/verification failure, a `101`, both framings, an unsupported transfer coding, or an invalid `Content-Length`; bodyless statuses before framing: a `204` held open is `Ok` and empty before the deadline, a `304` with `Content-Length` then close is `Status(304)`, a `100` then a `200` gives the `200`, a lone `100` then close is `Reset`; a close-delimited HTTPS body needs TLS `close_notify` to end (`Reset` on a bare close), while a complete Content-Length or chunked 2xx body stays `Ok` on a bare close; all failed connection attempts give `Refused` if all refused, otherwise `Io` of the first non-refusal error; the request is exactly NET-GET's (no path or query CR/LF, no fragment; key's host in `Host`, IPv6 bracketed, and SNI for DNS only); proxy, `SSL_CERT_*` and `SSLKEYLOGFILE` variables change nothing (no key-log file created or appended) | runtime | `tide-std` `net::http_tests::real_get_*`, `tls_get_*`, `environment_changes_nothing` (loopback servers and a crate-private test root) and `simulated_get_*`. The real-backend cases include cancellation over HTTP and HTTPS, with equal deadlines tested over HTTP. TLS chain and name failures precede HTTP data. A 64 MiB body succeeds and one byte more is `Io(InvalidData)`. |
| NET-HTTP-3 | 03 §9.2/§9.3, companion 15.2/17; research/42 §9 | `sim.world.http` peers, `Reply::{ok, status, hang, reset, tls_error}` and `.after(d)` give the timeline and results of research/42 §9; `Reply::status` outside 200–599 panics; `Request::url()` removes fragments; every reply obeys the 64 MiB limit (`Reply::ok` at 64 MiB succeeds, one byte more is `Io(InvalidData)`); `get` reaches only `world.http` peers, never TCP listeners; an IP with no peer is `Refused` after one round trip, even with a TCP listener, while an unknown domain is `Dns` at once; registering a peer on a listener-held address panics at setup, and a later bind on a peer address is `AddrInUse` (wildcard/loopback aliases included); `only` applies before any peer; companion 15.2 builds through tide-admit and its `#[sim_test]` passes; a seed that uses `get` replays byte for byte, and one that does not makes today's draws | runtime | `tide-std` `net::http_tests::simulated_get_*`. `authority/net_fetch` compiles companion 15.2. The tide-admit build test `companion_fetcher_builds_and_its_sim_test_passes` runs its simulated fetch. Trace records omit path, query and body. |
| NET-HTTP-4 | 03 §0 CAP-1; research/42 §§2, 11 | `tide_std::net` exposes no public path to the network except `Net`'s methods and their handles: `Url` is a newtype (no `socket_addrs`), and a test lists the module's public items, `pub use` re-exports included, and fails on additions (no `url`, `rustls`, `httparse` or resolver item); connection-holding types are in `NOT_PURE`; `Url`, `UrlError`, `NetError` are `Pure` | checker | tide-admit build test `std_net_public_items_are_fixed`; driver test `net_get_is_admitted_with_net_in_the_trust_diff` |
| NET-HTTP-5 | 03 §1.1 NET-HOST; research/42 §§4, 14 | On the real backend, `only([]).no_bind().bind(("unlisted.test", 8080))` returns `PermissionDenied` before any resolution, with zero resolver calls observed; simulated bind failure alone is not proof | runtime | `tide-std` `net::http_tests::no_bind_refuses_before_resolution` |
| NET-RESOLVE-1 | 03 §1.1 Name resolution; research/48 §§2–4, 11 | Planned (fx-6d50 stage 2). One admitted suite checks the same values and `NetError` variants on the real and simulated `Net`: IP literals in every NET-HOST spelling and `localhost` return their key's address with no lookup (including IPv4 for unbracketed `::ffff:127.0.0.1`); a port, URL, empty label, `%`, `xn--` or `*` host is `Io(InvalidInput)`; under `only` a host is allowed when a retained entry names it on any port, a name grant never allows its address nor an address grant a name, and an empty list or empty chained intersection yields `Denied`. A passed deadline is `Timeout`, including `resolve("127.0.0.1", past)`. A cancelled scope is `Cancelled`, winning over a passed deadline. `nothing.invalid` without a simulated peer or with a real lookup failure supplied by the `#[cfg(test)]` seam in `net/resolve.rs` next to `CALLS` is `Dns`. The suite sends no live resolver queries | runtime | planned: an `authority/resolve.rs` case like `authority/tcp.rs`, with seam-backed real coverage in `tide-std` |
| NET-RESOLVE-2 | 03 §9.3; research/48 §§6, 11 | Planned (fx-6d50 stage 2). Simulated resolution is deterministic. `localhost` returns `127.0.0.1` with and without a peer. Other domains named by a `world.http` peer resolve to `198.18.0.n` on every seed. Ordinals start at 1 and count distinct domain peer host keys after NET-HOST canonicalization in setup order. `localhost` and IP-address peers take no ordinal. Register `localhost:8080` first, an IP-address peer second, then `a.test:443` and check that `a.test` resolves to `198.18.0.1`. Domains with no peer return `Dns` except `localhost`. The call takes no virtual time and no draw (a run's draws equal those of the run without it) and records one `net.resolve` trace event. A seed that resolves replays byte for byte | runtime | planned: `tide-std` `net::http_tests` resolve cases and a replay case |
| NET-RESOLVE-3 | 03 §1.1 NET-HOST, Name resolution; research/48 §§3, 5, 11 | Planned (fx-6d50 stage 2). On the real backend a `Denied` or `InvalidInput` resolve makes zero resolver calls. A non-literal lookup uses the one NET-RESOLVE pool. Supply the same answer to both lookups with the `#[cfg(test)]` seam in `net/resolve.rs` next to `CALLS`. `resolve(h)` returns the list `connect`'s lookup uses with ports removed, preserving order and duplicates. With the seam holding a lookup, the call returns `Timeout` at its deadline or `Cancelled` on scope cancellation. The abandoned lookup releases its slot when it finishes. These cases send no live resolver queries | runtime | planned: `tide-std` `net::http_tests` and `net/resolve.rs` tests |
| CAP-2 | §1.2, 03:215 | Every synchronization-handle constructor takes a final `&Tasks`; missing-argument checks use its resolved std signature | both | TD0505 (ref:725); E0061 |
| CAP-3 | §1.2, 03:220 | `std::sync::raw` (`Mutex`, `RwLock`, weaker orderings) needs the `unsafe` grant | checker | TD0520 (via ADMIT-1) |
| ROOT-1 | §2.1, 03:268 | `Root` may be named in `main`'s signature, field patterns destructuring `root` (including `main`'s parameter), and `Root::sim`; every binary's `main` carries the attribute | both | TD0503 for naming `Root` elsewhere; TD0512; rustc E0580 for `fn main(root: Root)` without the attribute |
| ROOT-2 | §2.1, 03:273 | `root` cannot be passed, borrowed into a call, stored, returned, or captured whole; field moves, borrows and field patterns destructuring `root` (including `main`'s parameter) pass | checker | TD0503 |
| ROOT-3 | §2.1, 03:277 | `Root::sim`, `SimRoot`, `NodeRoot`, and `std::sim` compile only in test code (test-only cfg ancestors, test functions, separate suites) | checker | TD0504 |
| ROOT-4 | §2.1, 03:283 | Capabilities and handles cannot be built from data (private field, no `Default`/`Decode`); Tide's std bounds what `mem::zeroed`, `transmute`, raw-pointer reads, and union field reads produce by `Pure` | both | E0423 / E0599; `Pure` part TD0517; no own code |
| ROOT-5 | §2.1, 03:294 | No code calls `main` or uses it as a value | checker | TD0506 |
| GLOBAL-1 | §2.2, 03:299 | A `static` must be `Sync + Pure + 'static`; no `static mut` in safe code | both | TD0502; E0277 for `Sync` |
| GLOBAL-2 | §2.2, 03:302 | No lazy statics: a static's initializer is evaluated at compile time by `const fn` | both | rustc E0015; `LazyLock`/`OnceLock` TD0520; `lazy_static!` TD0502 |
| GLOBAL-3 | §2.2, 03:307 | Safe code has no thread-locals or task-locals | checker | `thread_local!` TD0502 |
| GLOBAL-4 | §2.2, 03:310 | Raw pointer types, pointer-to-integer casts, `addr`, `{:p}`, and a function pointer's address observers only under the `unsafe` grant; under it, an address counts as a `usize` in D3's lossless rule | checker | TD0520 (via ADMIT-1; ref:956); TD0301 for a lossy address cast |
| GLOBAL-6 | §2.2, 03:320 | `Arc` and `sync::Weak` are not `Pure`, take `&Tasks` to create, and lack count observers | both | `Pure` part: needing rule's code or TD0517; TD0505; E0599 / E0061 |
| INPUT-1 | §1.1, 03 INPUT-1; §1.3; ref §6.2 | Arguments, attenuated environment, cwd and stdin execute on real and simulated capabilities; main uses Rust Termination (0, 1 for Err, given ExitCode, 101 for panic) | runtime | — |
| SINK-1 | §2.4, 03 SINK-1 | Stdout/stderr handles always return Ok; sinks expose no world state | runtime | — |
| SINK-2 | §2.4, 03 SINK-2 | LogConfig defaults to Info/text/stderr; setters configure the log sink | runtime | — |
| SINK-3 | §2.4, 03 SINK-3 | Every record formats its arguments before filtering | runtime | — |
| SINK-4 | §2.4, 03 SINK-4 | Within a task, records appear in program order; parallel branches' buffers are appended in branch order at the join | runtime | — |
| SINK-5 | §2.4, 03 SINK-5 | Synchronous sinks preserve records under load; closed stdout exits quietly with 141 | runtime | — |
| SINK-6 | §2.4, 03 SINK-6 | Simulated sink events assemble per-stream lines, including filtered log levels, without host writes | runtime | — |
| SINK-8 | §2.4, 03:424 | `dbg!` is an error in release and CI profiles; `print!` in a library and printing in a `with` closure are lints | checker | TD0507 (error); TD0508, TD0607 (lints) |
| SINK-10 | §2.4, 03:435 | `Secret<T>` has no `Debug`/`Display`, so formatting one is a type error | rustc | E0277 |
| ADMIT-1 | §2.5, 03:446 | Only listed `core`/`alloc` items and std inherent methods are admitted (paths, methods, operators, derefs, selected impls, macro expansions), and `#![no_std]` is not | checker | TD0520; TD0517 for `io::Error` conversions (03:464) |
| ADMIT-2 | §2.5, 03:484 | The driver passes rustc no user flags and passes `-Zallow-features=`, so user code cannot enable unstable features, and `-Zpolonius=no` (ORIGIN-6) | rustc (driver-configured) | E0725 |
| UNSAFE-1 | §2.6, 03:606 (RUN-4 hole 1) | `unsafe` blocks and fns, `unsafe extern` blocks and items, `unsafe impl`, unsafe attributes, and `#[trusted_pure]` need the `unsafe` grant; so do `asm!`, `global_asm!`, and `naked_asm!`; no grant admits `unsafe impl Pure` | both | TD0901; `asm!`, `global_asm!`, and `naked_asm!` TD0520; `unsafe impl Pure` E0119 / TD0402 |
| = PURE-1 | §3.1, 03:727 | `Pure` is decided structurally at each `Pure`-implying obligation and each coercion to a `Pure` trait object | checker | TD0517, or TD0502 / TD0518 / TD0603 / TD0606 / TD0610 / TD0611 / TD0612 |
| = UNSAFE-1 | §3.1, 03:735 | `#[trusted_pure]` on a type only in a package granted `unsafe` | checker | TD0901 (03:609) |
| = DYN-2 | §3.1, 03:762 | A `Pure` trait object goes through a companion supertrait, checked at each coercion (E0225 for `dyn Trait + Pure` is 02 §5.1's) | checker | TD0517 |
| AFREE-1 | §3.2, 03:795 | `#[pure]` asserts authority-freedom for every instantiation the bounds allow | checker | TD0511 |
| AFREE-2 | §3.5 table, 03:852 | `const fn`: `Pure + Sync` parameters, no `&mut` parameters (rustc accepts both); type parameters bounded at the definition | checker | TD0518 |
| TASK-6 | §6.2, 03 TASK-6 | Shielding hides enclosing cancellation; BufWriter Drop discards unflushed bytes and warns with their count | runtime | — |
| = SEND-3 | §6.4, 03:1258 | Channel payloads must be `Send` | rustc | E0277 |
| CHANNEL-1 | §6.4, 03:1259 | An unbounded `channel` in a library is a lint | checker | TD0608 (lint) |
| LOCK-1 | §6.5, 03:1307 | Payload `T: Send + Pure` | both | TD0606; E0277 for `Send` |
| LOCK-2 | §6.5, 03 LOCK-2 | Repeated cell operands return `Aliased` without running the callback, before poison is checked | runtime | — |
| LOCK-3 | §6.5, 03 LOCK-3 | Multi-cell acquisition uses global cell ID order, FIFO per cell, and deterministic simulator IDs; acquisition proceeds in cancelled scopes | runtime | — |
| LOCK-4 | §6.5, 03:1317 | `f` must be `Pure`; printing inside it is a lint | checker | TD0603; TD0607 (lint) |
| LOCK-5 | §6.5, 03:1323 | `with`/`with2`/`with_all` return `Result<R, LockError>` | rustc | type errors (ignoring the result is TD0702, ref:803) |
| LOCK-6 | §6.5, 03 LOCK-6 | A callback panic poisons every held cell; `reset` replaces the payload and clears poison | runtime | — |
| = SEND-3 | §6.6, 03:1343 | `Send`/`Sync` are checked at task boundaries only | rustc | E0277 |
| PAR-5 | §8.1, 03:1425 | `join` closures are `FnOnce + Pure + Send` | both | TD0610 (also ref:785); E0277 |
| PAR-6 | §8.1, 03:1426 | `par_*` closures and `par_sort`'s comparator are `Fn + Pure + Send + Sync` | both | TD0610 (03:1429); E0277 / E0507 |
| PAR-7 | §8.1, 03:1429 | Parallel closures capture shared data (a `&T`, borrowed or moved in) only if `T: Pure + Sync`, reported in place of PAR-5's and PAR-6's code; `&mut` captures must be disjoint | both | TD0611 (03:1430); E0277 / E0524 |
| PAR-4 | §8.1, 03:1435 | `par_map` needs `T: Pure + Sync`; `par_for`, `par_chunks_mut`, and `par_sort` need `T: Pure + Send` | both | TD0612; E0277 for `Send`/`Sync` |
| PAR-3 | §8.2, 03:1465 | Allowlist part: float sign/bit observers and math methods are outside the allowlist | checker | TD0520 (via ADMIT-1) |
| PAR-1 | §8.2, 03 PAR-1 | When no branch panics, a parallel call's values, writes through `&mut`, and sink output equal sequential evaluation with branches left to right | runtime | — |
| PAR-2 | §8.2, 03 PAR-2 | The panic first in sequential order propagates in the enclosing task, and buffered sink output from branches after it is discarded | runtime | — |

## 04-contracts.md

| ID | Section (file:line of the definition) | Rule, one line | Decided by | TD code(s) / rustc |
| --- | --- | --- | --- | --- |
| FORM-1 | §1.2, 04:180 (§1.2 rule 1) | One expression per clause: a top-level comma is an error (safe fix splits it) | checker (production contract macro) | TD0804 in an uncoded compile error |
| FORM-2 | §1.2, 04:187 (§1.2 rule 3) | `result` only in `ensures`; a function with `ensures` cannot have a parameter named `result` | checker (production contract macro) | TD0805 in an uncoded compile error |
| FORM-3 | §1.2, 04:202 (§1.2 rule 6; table 04:71, 04:109) | v0: `requires`/`ensures` on a trait method or trait-impl method is rejected; `#[invariants]` and `invariant!` are allowed there | checker | TD0802 |
| FORM-4 | §1.2, 04:213 (§1.2 rule 10; also 04:33) | `invariant!` must open the loop body, and the fn needs `#[invariants]` or a contract attribute; an unrewritten clause is a compile error | both | TD0806 in an uncoded compile error |
| CLAUSE-1 | §1.8, 04:385 | v0 clause forms only; quantifiers must be bounded (`lo <= x && x < hi ==>` or `c.contains(&x) ==>`) | both | TD0807 in an uncoded compile error; a v1 form is an attribute v0's std lacks, rustc's uncoded error |
| CLAUSE-2 | §1.8, 04:392 | A clause is a Rust boolean expression, typed as code; names resolve as in the body | rustc | type errors (e.g. E0308); E0425 for an unresolved name |
| CLAUSE-3 | §1.8, 04:401 | Clauses call only authority-free fns with `Pure + Sync` arguments and `&mut` only to clause temporaries; generic code needs the bound at the definition | checker | TD0808 (defined in 04 §1.8) |
| CLAUSE-4 | §1.8, 04:417 (also §1.2 rule 4, 04:189) | `old` snapshots are borrow-checked; non-`Copy` non-`Clone` `old` is an error; `old(e)` cannot name `result` or a bound variable; prophetic `&mut` param outside `old` is TD0803; mutated non-snapshot param in `ensures` is an error | both | TD0803 is enforced by admission; TD0809 awaits admission; production lowering gives uncoded errors for invalid `old`, E0599 for a missing value Clone or E0277 for a non-Clone referent, and E0502/E0505 for conflicting entry borrows |
| CLAUSE-6 | §1.8, 04:466 | Name-resolution part: over an iterator other than `a..b`, a `for` loop's invariant cannot name the loop pattern's bindings | rustc (through the contract macros' lowering) | E0425: the head comes before `next`, so the bindings are out of scope |
| SEAL-2 | §11.1, 04:1391 | What an interface, whole, or manifest seal entry fixes (content and identity, checked by SEAL-3) | checker (admission driver) | TD0920 via SEAL-3 |
| SEAL-3 | §11.1, 04:1482 | After rustc's analysis and before any artifact is written, the driver checks the crate against its own `tide.seal`; a changed or missing item, or an unlisted target, is an error and the crate is not admitted | checker (admission driver) | TD0920 |

SEAL-2 and SEAL-3 need a crate together with its `tide.seal`, which a
one-file case cannot hold; their cases wait for the admission driver.

## Reference and decision-record rules

Rules owned by `design/01-design.md` and `design/reference.md` that have no
chapter ID use `REF-<TD digits>` here. The inventory does not imply the driver
already implements them; `--tide-print-codes` names the enforced subset.

| ID | Section | Rule, one line | Decided by | TD code(s) / rustc |
| --- | --- | --- | --- | --- |
| REF-0102 | D6; ref §2.2 | Only the closed derive set is admitted | checker | TD0102 |
| REF-0103 | D6; ref §2.1 | Only the fixed bang forms are admitted; named carve-outs retain their own codes | checker | TD0103 |
| REF-0210 | D2; ref §3.6 | When a literal moves a whole owner and borrows it in another field, label E0505 at the owner value or E0515/E0597 inside the paired borrow expression (E0515 also via its secondary borrowed-here span); omit uncertain evidence | both | TD0210 |
| REF-0301 | D3; ref §4.2 | Only lossless numeric as casts; pointer-sized integers and addresses count as 32 to 64 bits | checker | TD0301 |
| REF-0401 | D4; ref §5.2 | Impls live in the exact module of their local ADT (through nested `&T`, `&mut T`, and `Box<T>` wrappers) or trait; other non-local self types use the trait module | checker | TD0401 |
| REF-0402 | D4; ref §5.1 | No specialization or overlapping impls, including an impl for a type parameter or a projection that may be unsized, of a dyn-compatible trait that declares an associated type without `where Self: Sized`, which may overlap the built-in impl of the trait's `dyn` type (rust-lang/rust#57893) | both | TD0402; E0119 for overlap |
| REF-0405 | D4; ref §5.1 | No negative impls or user auto traits | both | TD0405 |
| REF-0406 | D4; ref §5.1 | Impl bounds use parameters, their projections, concrete types, or strict subterms of self/trait arguments | checker | TD0406 |
| REF-0407 | D3; ref §5.1 | No expanding cycles between (function, type parameter) nodes: parameter flow under a constructor must not cycle; trait edges require selected local impls; unresolved aliases and rustc instance-depth errors alone prove no growth | checker | TD0407 |
| REF-0408 | D3; ref §5.1 | One inherent method per name per type | checker | TD0408 |
| REF-0601 | D8; ref §7 | No async/await or core future/pin APIs; core task machinery keeps TD0520 | checker | TD0601 |
| REF-0602 | D8; ref §7.1 | Use Tasks scopes instead of Rust thread spawn, Builder, thread scopes, or tokio::spawn | both | TD0602 |
| REF-0604 | D8; ref §7.4 | Mutex/RwLock habits use Shared and its scoped closures | both | TD0604 |
| REF-0605 | D8; ref §7.2 | No user-written select!, regardless of provider; use one enum channel or deadline/cancellation operations | both | TD0605 |
| REF-0701 | D5; ref §9 | No catch_unwind; isolate a task with spawn_isolated | both | TD0701 |
| REF-0702 | D5; ref §9 | A Result in expression statement position (including `Result<(), Infallible>` and macro results) cannot be ignored; `let _ =` discards explicitly | checker | TD0702 |
| REF-0902 | D11; ref §11 | Each unsafe block, with or without the grant, needs a SAFETY line comment directly above the block or its statement; unsafe impls need none | checker | TD0902 |
| REF-0910 | D12; ref §12.1 | An out-of-line module needs its source file | both | TD0910 |
| REF-0911 | D12; ref §12.1 | No path attributes on modules | checker | TD0911 |
| REF-0912 | D12; ref §12.2 | No user extern crate items | checker | TD0912 |
| REF-0913 | D12; ref §12.2 | Unused imports are errors | checker | TD0913 |
| REF-0914 | D12; ref §12.2 | No use-item cycles between modules; edges target the defining module, including re-exports and globs | checker | TD0914 |
| REF-0915 | D13; ref §12.4 | Packages cannot have build scripts | checker | TD0915; package cases in tide-admit tests/invocation.rs `build_scripts_are_rejected_without_running` assert error severity, exit 1, and accepted exit 0 without a build script |

## Considered and excluded (runtime, simulator, sandbox, tooling, non-rejecting, or v1)

- PANIC-1 (03:347): runtime abort, and the harness reruns in-flight tests.
- PANIC-2 (03:363): runtime abort on a panic during unwinding.
- SINK-4 (03 §2.4): runtime output ordering; `tasks/parallel_regions.rs` and driver panic tests exercise ordered parallel buffers.
- SINK-7 (03:422): output elision under memoization.
- Memoization (03:852, 02:927): rejects nothing; the compiler memoizes only calls that meet AFREE-2's conditions.
- SINK-9 (03:430): v1, plus lockfile tooling.
- GLOBAL-5 (03:317): `HashMap` key behaviour in the library at runtime.
- RUN-1 (03:498): sandbox guarantee.
- RUN-2 (03:517): build sandbox and file loader. Its compile-time rejections are ADMIT-1 (TD0520), TD0911, and TD0915, which are build tooling.
- RUN-3 (03:555): test-process sandbox.
- RUN-4 (03:604): hole list. Only hole 1's grant requirement is compile-time, and it is a row above.
- RUN-5 (03:644): `tide review` coverage.
- RUN-6 (03:666): result channel and output escaping.
- TASK-1 (03 §6.2): runtime failure, panic propagation and isolation; `tasks/task_controls.rs` executes isolation through the driver. Its `Owned` closure typing is the SCOPE-3 row.
- TASK-2 (03:1135): runtime scope-outcome precedence.
- TASK-3 (03 §6.2): runtime level-triggered cancellation; `tasks/task_controls.rs` executes checkpoints and cancelled limited spawns; `tasks/channels.rs` executes cancellation-aware sends/receives and preserves buffered values.
- TASK-4 (03 §6.2): runtime deadlines; `tasks/task_controls.rs` executes deadline errors and partial Ok results; `tasks/channels.rs` executes receive timeouts and cancellation of channel waits.
- TASK-5 (03:1223): runtime list of what cancellation cannot interrupt.
- JOIN-1 (03:1240): runtime; scope exit joins and is shielded.
- JOIN-2 (03:1243): runtime `Task::join` semantics.
- PAR-1 (03 §8.2): runtime determinism; `tasks/parallel_regions.rs` executes join/map/for/sort on both runtimes in debug and release through the driver.
- PAR-2 (03 §8.2): runtime panic order; driver tests check panic propagation and sink prefixes, and production runtime tests force competing panics.
- PAR-3 (03:1461): float codegen and the math library. Only its allowlist part is a row above.
- SUSP-1 (03:1357): compile-time may-suspend classification that rejects nothing.
- SUSP-2 (03:1368): compile-time per-instance classification that rejects nothing.
- SUSP-3 (03:1371): codegen refinement.
- REPLAY-1 (03 §9.6): driver/harness evaluation manifests (runtime proof in tide-admit tests).
- REPLAY-2 (03 §9.6): `tide-admit --replay` identity validation (tooling/runtime proof); `tide sim` frontend planned.
- CLAUSE-5 (04:451): compile-time class assignment that rejects nothing.
- CLAUSE-6 (04:460): runtime check schedule. Only its name-resolution part is a row above.
  `contracts/generated_tests.rs` is also executed by the driver build tests to check generated
  ensures failures, requires guidance, unchecked skips and allowed vacuity; production owns
  its runner and the conformance std includes that same implementation.
- CLAUSE-7 (04 §1.8): Runtime violation outcome; `contracts/executable.rs`, `contracts/scoped_clauses.rs` and `contracts/clause_expressions.rs` execute through the production driver tests.
  `contracts/guided_properties.rs` also checks property guidance and
  rejection/vacuity, direct versus deeper call blame, and simulation-parameter shrinking
  through production driver tests.
- CLAUSE-8 (04:506): approval hash (review and CI tooling).
- SEAL-1 (04:1336): `tide seal` command and file format (tooling).
- SEAL-4 (04:1500): TD0921 from `tide review` and the CI gate, not from compilation.
- SEAL-5 (04:1518): review output.
- SEAL-6 (04:1522): informative limits.
- v1, excluded by tier: FFI-1 to FFI-7 (03:969-1015), RUST-1 to RUST-6 (03:1778-1812), LOCK-7 (03:1330), and 02 §7 field-borrow rules (02:561-600).
- Non-rules: `SE-0414` (02:1335) matches the ID pattern but is a Swift proposal.

## Tier unclear

- `#[contracts(assert | test | prove)]`: 04:15 puts per-item uses (§2.1) in v1, but 04:216-217 (§1.2 rule 10) lists `#[contracts]` among the v0 attributes that satisfy the loop-clause rule, and 04:84 lists it without a tier. Also, 04:515-516 says an in-source module override (`#![contracts(test)]`) is rejected because it is unstable Rust, and that sits in the v1 section.
- TD0516 (authority policies): 01:956 says v0 and ref:1044 gives the code, but no chapter in 02/03/04 defines the rule.
- ROOT-4 names `Shape` construction (D6), and `Shape` reflection is Later tier in 01:84.
