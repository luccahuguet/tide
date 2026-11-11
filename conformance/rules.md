# Rule inventory

Every v0-core normative rule in `design/02-memory.md`,
`design/03-authority-concurrency.md`, and `design/04-contracts.md` that makes
rustc or the Tide checker accept, reject, or warn on a program at compile time, taken
at `a3b0177`, plus runtime rules exercised by cases that also run against
production std. The chapters own the rules. `run.sh` reads IDs from the first
column, and `run.sh --trace` gives each ID's cases.

Columns: **ID** is the rule's ID, `none yet`, or `= ID` for a row that
restates a rule whose ID another chapter defines. **Decided by** is `rustc`
(plain rustc rejects it against Tide's std), `checker` (it needs the Tide
checker, the admission driver, or the contract macros), `both`, or `runtime`
(a driver test executes the case against production std). **Codes**
are the FX codes the design gives (`ref` is `design/reference.md`, `comp`
its companion) and the rustc errors expected; a rustc code the design does
not cite is a best mapping that a case must confirm.

## 02-memory.md

| ID | Section (file:line of the definition) | Rule, one line | Decided by | FX code(s) / rustc |
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
| LIFE-3 | §2.3, 02:187 | Rejects lifetime parameters on traits, explicit `for<'a>`, and GATs (beyond D4's ceiling) | checker | FX0201, FX0404, FX0403 |
| LIFE-4 | §2.5, 02:233 | An elided output lifetime that rules 1 and 2 do not resolve is an error | both | FX0202; rustc E0106 |
| LIFE-5 | §2.5, 02:237 | Elided lifetimes in consts and statics are `'static` | rustc | E0597 |
| LIFE-6 | §2.5, 02:238 | Fields have no elision: a reference field names a struct lifetime or `'static` | rustc | E0106 |
| LIFE-7 | §2.5, 02:241 (also 02:181) | The bare name of a borrowing type in an impl header is an error | rustc | E0726 |
| LIFE-8 | §2.5, 02:243 | Associated types in impls have no elision | rustc | uncoded error ("declared without lifetime parameters") |
| ORIGIN-3 | §2.7 Bodies, 02:275 | A body may use inclusions between signature origins only from declared or implied bounds | rustc | "lifetime may not live long enough" |
| ORIGIN-4 | §2.7, 02:281 | A local loan reaching a placeholder origin (a returned reference to a local) is an error | rustc | E0515 |
| ORIGIN-5 | §2.7, 02:284 | An impl method's signature must be at least as general as the trait's | rustc | E0195; uncoded "`impl` item signature doesn't match" |
| ORIGIN-6 | §2.8, 02:290 | v0 accepts a body exactly when NLL accepts it (no Polonius), apart from CLOSURE-3's `fn`-pointer re-check | rustc | NLL |
| BORROW-1 | §3.1, 02:305 | Every reference, borrowing type, and `dyn` bound in a field uses a declared lifetime or `'static` | rustc | E0106 / E0261 |
| BORROW-2 | §3.6, 02:360 | In `impl<'a> Lexer<'a>`, `fn new(src: &str) -> Self` is an error | rustc | lifetime error |
| BORROW-3 | §3.7, 02:375 | `Owned` (`'static`) needed for `Box<dyn Trait>` in a signature or field, `dyn Any`, and `spawn_isolated` closures (SCOPE-3) | both | rustc `'static` errors; FX0203 (ref:207) |
| VAR-1 | §4.1, 02:383 | Subtyping comes only from origins: `&` is covariant, `&mut T` invariant in `T` | rustc | lifetime / variance errors |
| VAR-2 | §4.1, 02:392 (table 02:396, per-parameter rule 02:405) | Variances as tabled and computed per parameter of a struct or enum (cells, `Shared`, and `dyn Trait<A>`'s `A` invariant; `fn` arguments contravariant) | rustc | lifetime / variance errors |
| VAR-3 | §4.1, 02:407 | Unsizing to `dyn` may shorten the object's origin even behind `&mut` | rustc | lifetime errors |
| DYN-1 | §5.1, 02:447 | `dyn` object-lifetime defaults: behind `&` it takes the reference's origin; elsewhere outside a body `'static` | rustc | `'static` errors |
| DYN-2 | §5.1, 02:460 | A `dyn` is `Send`/`Sync` only if it names them, and `Pure` only through a companion supertrait (`dyn Fn() + Pure` is E0225) | both | E0225 / E0277; FX0517 |
| OPAQUE-1 | §5.2, 02:471 | A return-position `impl Trait` captures every in-scope generic; with `+ 'static` the hidden type must satisfy it | rustc | capture / `'static` errors |
| OPAQUE-2 | §5.2, 02:479 | An opaque type is `Send`, `Sync`, or `Pure` only if its bounds state it (no auto-trait leakage) | checker | FX0205 (`Send`/`Sync`, defined in 02 §5.2; rustc accepts it when the hidden type has the trait); FX0517 (`Pure`) |
| CLOSURE-1 | §5.3, 02:496 | A closure that captures a borrow may be returned or stored only where its origins are named or captured | rustc | E0373 / lifetime errors |
| CLOSURE-2 | §5.4, 02:514 | With no expected type, a closure cannot return a borrow of a parameter whose annotation elides its lifetime (a named lifetime or inferred types are accepted) | rustc | lifetime error |
| CLOSURE-3 | §5.4, 02:523 | Coercing a fn item to a `fn` pointer re-checks its implied bounds (closes rust#25860) | checker | FX0206 (defined in 02 §5.4; rustc accepts) |
| DROP-1 | §6, 02:530 | "Rule DC": Drop check: every origin in `dropck(T)` must be live where `T` drops | rustc | E0597 / E0713 |
| DROP-2 | §6, 02:555 | Users cannot write a drop-check exemption (`#[may_dangle]`) | rustc (unstable feature, under ADMIT-2) | E0658 / E0725 |
| SEND-1 | §8.1, 02:606 (table 02:610) | `Send`/`Sync` are derived structurally per the table | rustc | E0277 |
| = UNSAFE-1 | §8.1, 02:623 | `unsafe impl Send`/`Sync` only in packages granted `unsafe` | checker | FX0901 (ref:953) |
| SEND-3 | §8.1, 02:626 | Task boundaries need `Send` (by value) or `Sync` (by reference) | rustc | E0277 |
| SCOPE-1 | §8.2, 02:682 | Child captures and results outlive `σ`; the parent cannot mutate, move, or free a lent place before the scope ends | rustc | E0373 / E0499 / E0505 / E0506 / E0597 |
| SCOPE-2 | §8.2, 02:690 | Neither a `Scope` nor a `Task` escapes the scope body through `T` | rustc | higher-ranked lifetime error (uncoded) |
| SCOPE-3 | §8.2, 02:692 | `spawn_isolated` takes an `Owned` closure | both | FX0203 (ref:207); E0373 / E0521 |
| SCOPE-4 | §8.2, 02:696 | A child spawned on a passed-down scope owns its data or borrows only what the signature ties to the scope | rustc | E0373 / E0621 |
| = LOCK-1 | §8.4, 02:738 | `Shared<T>` needs `T: Send + Pure`; creating one takes `&Tasks` | both | FX0606; FX0505 (ref:725); E0277 / E0061 |
| SHARED-1 | §8.4, 02:743 | `with`/`with2` payload references are higher-ranked, so nothing carrying `σ` escapes | rustc | higher-ranked lifetime error (uncoded) / E0521 |
| = LOCK-4 | §8.4, 02:747 | The `with` closure is `Pure` | checker | FX0603 |
| = PAR-5 | §8.5, 02:764 | `join` closures are `FnOnce + Pure + Send` | both | FX0610; E0277 |
| = PAR-6 | §8.5, 02:765 | `par_map`/`par_for` closures are `Fn + Pure + Send + Sync` | both | FX0610; E0277 / E0507 |
| = PAR-7 | §8.5, 02:768 | Shared captures `Pure + Sync`; `&mut` captures disjoint | both | FX0611; E0277 / E0524 |
| = PAR-4 | §8.5, 02:775 | Elements: `Pure`, plus `Sync` when shared and `Send` when split or moved | both | FX0612; E0277 for `Send`/`Sync` |
| PURE-1 | §9, 02:789 (not-Pure list 02:815) | `Pure` is decided structurally and deeply (fields, `Box`/`Rc`, greatest fixed point, opaque and `dyn` by stated bounds, type parameters by bound) | checker | FX0517, or the needing rule's code |
| = UNSAFE-1 | §9, 02:793 | `#[trusted_pure]` only in a package granted `unsafe` | checker | FX0901 (03:609) |
| PURE-2 | §9, 02:807 | A missing auto or marker trait never makes two impls disjoint | both | FX0402 (ref:307); E0119 (rustc agrees, for `Pure` via the blanket impl) |
| = ADMIT-1 | §9, 02:857 | `OnceLock` and `LazyLock` are not admitted | checker | FX0520 |
| = GLOBAL-6 | §9, 02:870 | `Arc` omits `strong_count`, `weak_count`, `get_mut`, `try_unwrap`, and `into_inner` | rustc | E0599 |
| = ADMIT-1 | §9, 02:886 | `io::Error::new`/`other` conversions must reach one of alloc's four `From` impls (no user `From`/`Into`, no reflexive impl) | checker | FX0517 (02:889) |
| = ADMIT-1 | §9, 02:893 | Generic code that wraps those constructors bounds its parameter by `Error + Send + Sync + Pure + 'static`, checked at the definition | checker | FX0517 |
| = ADMIT-1 | §9, 02:898 | `Waker` is not admitted | checker | FX0520 |
| = AFREE-2 | §9 table, 02:927 | `const fn`: `Pure + Sync` parameters, no `&mut` parameters | checker | FX0518 |

## 03-authority-concurrency.md

| ID | Section (file:line of the definition) | Rule, one line | Decided by | FX code(s) / rustc |
| --- | --- | --- | --- | --- |
| CAP-1 | §0, 03:41 | World and nondeterminism operations exist only as capability methods; ambient std calls are rejected | both | FX0501 (ref:549); rustc E0425 / E0599 (unresolved) |
| CAP-2 | §1.2, 03:215 | Every synchronization-handle constructor takes `&Tasks` | both | FX0505 (ref:725); E0061 |
| CAP-3 | §1.2, 03:220 | `std::sync::raw` (`Mutex`, `RwLock`, weaker orderings) needs the `unsafe` grant | checker | FX0520 (via ADMIT-1) |
| ROOT-1 | §2.1, 03:268 | `Root` may be named only as the parameter of `#[tide::main] fn main` and in the path `Root::sim`; every binary's `main` carries the attribute | both | FX0503 for naming `Root` elsewhere; FX0512; rustc E0580 for `fn main(root: Root)` without the attribute |
| ROOT-2 | §2.1, 03:273 | `root` cannot be passed, borrowed into a call, stored, returned, or captured | checker | FX0503 |
| ROOT-3 | §2.1, 03:277 | `Root::sim`, `SimRoot`, `NodeRoot`, and `std::sim` compile only in test code | checker | FX0504 |
| ROOT-4 | §2.1, 03:283 | Capabilities and handles cannot be built from data (private field, no `Default`/`Decode`); Tide's std bounds what `mem::zeroed`, `transmute`, raw-pointer reads, and union field reads produce by `Pure` | both | E0423 / E0599; `Pure` part FX0517; no own code |
| ROOT-5 | §2.1, 03:294 | No code calls `main` or uses it as a value | checker | FX0506 |
| GLOBAL-1 | §2.2, 03:299 | A `static` must be `Sync + Pure + 'static`; no `static mut` in safe code | both | FX0502; E0277 for `Sync` |
| GLOBAL-2 | §2.2, 03:302 | No lazy statics: a static's initializer is evaluated at compile time by `const fn` | both | rustc E0015; `LazyLock`/`OnceLock` FX0520; `lazy_static!` FX0502 |
| GLOBAL-3 | §2.2, 03:307 | Safe code has no thread-locals or task-locals | checker | `thread_local!` FX0502 |
| GLOBAL-4 | §2.2, 03:310 | Raw pointer types, pointer-to-integer casts, `addr`, `{:p}`, and a function pointer's address observers only under the `unsafe` grant; under it, an address counts as a `usize` in D3's lossless rule | checker | FX0520 (via ADMIT-1; ref:956); FX0301 for a lossy address cast |
| GLOBAL-6 | §2.2, 03:320 | `Arc` and `sync::Weak` are not `Pure`, take `&Tasks` to create, and lack count observers | both | `Pure` part: needing rule's code or FX0517; FX0505; E0599 / E0061 |
| SINK-8 | §2.4, 03:424 | `dbg!` is an error in release and CI profiles; `print!` in a library and printing in a `with` closure are lints | checker | FX0507 (error); FX0508, FX0607 (lints) |
| SINK-10 | §2.4, 03:435 | `Secret<T>` has no `Debug`/`Display`, so formatting one is a type error | rustc | E0277 |
| ADMIT-1 | §2.5, 03:446 | Only listed `core`/`alloc` items and std inherent methods are admitted (paths, methods, operators, derefs, selected impls, macro expansions), and `#![no_std]` is not | checker | FX0520; FX0517 for `io::Error` conversions (03:464) |
| ADMIT-2 | §2.5, 03:484 | The driver passes rustc no user flags and passes `-Zallow-features=`, so user code cannot enable unstable features, and `-Zpolonius=no` (ORIGIN-6) | rustc (driver-configured) | E0725 |
| UNSAFE-1 | §2.6, 03:606 (RUN-4 hole 1) | `unsafe` blocks and fns, `unsafe extern` blocks and items, `unsafe impl`, unsafe attributes, and `#[trusted_pure]` need the `unsafe` grant; so do `asm!`, `global_asm!`, and `naked_asm!`; no grant admits `unsafe impl Pure` | both | FX0901; `asm!`, `global_asm!`, and `naked_asm!` FX0520; `unsafe impl Pure` E0119 / FX0402 |
| = PURE-1 | §3.1, 03:727 | `Pure` is decided structurally at each `Pure`-implying obligation and each coercion to a `Pure` trait object | checker | FX0517, or FX0502 / FX0518 / FX0603 / FX0606 / FX0610 / FX0611 / FX0612 |
| = UNSAFE-1 | §3.1, 03:735 | `#[trusted_pure]` on a type only in a package granted `unsafe` | checker | FX0901 (03:609) |
| = DYN-2 | §3.1, 03:762 | A `Pure` trait object goes through a companion supertrait, checked at each coercion (E0225 for `dyn Trait + Pure` is 02 §5.1's) | checker | FX0517 |
| AFREE-1 | §3.2, 03:795 | `#[pure]` asserts authority-freedom for every instantiation the bounds allow | checker | FX0511 |
| AFREE-2 | §3.5 table, 03:852 | `const fn`: `Pure + Sync` parameters, no `&mut` parameters (rustc accepts both); type parameters bounded at the definition | checker | FX0518 |
| = SEND-3 | §6.4, 03:1258 | Channel payloads must be `Send` | rustc | E0277 |
| CHANNEL-1 | §6.4, 03:1259 | An unbounded `channel` in a library is a lint | checker | FX0608 (lint) |
| LOCK-1 | §6.5, 03:1307 | Payload `T: Send + Pure` | both | FX0606; E0277 for `Send` |
| LOCK-3 | §6.5, 03:1494 | Single-cell acquisition proceeds in a cancelled scope | runtime | — |
| LOCK-4 | §6.5, 03:1317 | `f` must be `Pure`; printing inside it is a lint | checker | FX0603; FX0607 (lint) |
| LOCK-5 | §6.5, 03:1323 | `with`/`with2`/`with_all` return `Result<R, LockError>` | rustc | type errors (ignoring the result is FX0702, ref:803) |
| = SEND-3 | §6.6, 03:1343 | `Send`/`Sync` are checked at task boundaries only | rustc | E0277 |
| PAR-5 | §8.1, 03:1425 | `join` closures are `FnOnce + Pure + Send` | both | FX0610 (also ref:785); E0277 |
| PAR-6 | §8.1, 03:1426 | `par_*` closures and `par_sort`'s comparator are `Fn + Pure + Send + Sync` | both | FX0610 (03:1429); E0277 / E0507 |
| PAR-7 | §8.1, 03:1429 | Parallel closures capture shared data (a `&T`, borrowed or moved in) only if `T: Pure + Sync`, reported in place of PAR-5's and PAR-6's code; `&mut` captures must be disjoint | both | FX0611 (03:1430); E0277 / E0524 |
| PAR-4 | §8.1, 03:1435 | `par_map` needs `T: Pure + Sync`; `par_for`, `par_chunks_mut`, and `par_sort` need `T: Pure + Send` | both | FX0612; E0277 for `Send`/`Sync` |
| PAR-3 | §8.2, 03:1465 | Allowlist part: float sign/bit observers and math methods are outside the allowlist | checker | FX0520 (via ADMIT-1) |

## 04-contracts.md

| ID | Section (file:line of the definition) | Rule, one line | Decided by | FX code(s) / rustc |
| --- | --- | --- | --- | --- |
| FORM-1 | §1.2, 04:180 (§1.2 rule 1) | One expression per clause: a top-level comma is an error (safe fix splits it) | checker (contract macro) | FX0804 (defined in 04 §1.2); the D18 spike's lowering gives E0600 |
| FORM-2 | §1.2, 04:187 (§1.2 rule 3) | `result` only in `ensures`; a function with `ensures` cannot have a parameter named `result` | checker (contract macro) | FX0805 (defined in 04 §1.2); E0425 where the lowering binds no `result` |
| FORM-3 | §1.2, 04:202 (§1.2 rule 6; table 04:71, 04:109) | v0: `requires`/`ensures` on a trait method or trait-impl method is rejected; `#[invariants]` and `invariant!` are allowed there | checker | FX0802 |
| FORM-4 | §1.2, 04:213 (§1.2 rule 10; also 04:33) | `invariant!` must open the loop body, and the fn needs `#[invariants]` or a contract attribute; an unrewritten clause is a compile error | both | FX0806 (defined in 04 §1.2); rustc uncoded error (the unexported `!` form, or the D18 spike's `compile_error!`) |
| CLAUSE-1 | §1.8, 04:385 | v0 clause forms only; quantifiers must be bounded (`lo <= x && x < hi ==>` or `c.contains(&x) ==>`) | both | FX0807 (defined in 04 §1.8) for an unbounded quantifier, the D18 spike's `compile_error!` for some; a v1 form is an attribute v0's std lacks, rustc's uncoded error |
| CLAUSE-2 | §1.8, 04:392 | A clause is a Rust boolean expression, typed as code; names resolve as in the body | rustc | type errors (e.g. E0308); E0425 for an unresolved name |
| CLAUSE-3 | §1.8, 04:401 | Clauses call only authority-free fns with `Pure + Sync` arguments and `&mut` only to clause temporaries; generic code needs the bound at the definition | checker | FX0808 (defined in 04 §1.8) |
| CLAUSE-4 | §1.8, 04:417 (also §1.2 rule 4, 04:189) | `old` snapshots are borrow-checked; non-`Copy` non-`Clone` `old` is an error; `old(e)` cannot name `result` or a bound variable; prophetic `&mut` param outside `old` is FX0803; mutated non-snapshot param in `ensures` is an error | both | FX0803 (04:439); FX0809 (defined in 04 §1.8) for the in-place parameter; the rest via the lowering's rustc errors (E0425, E0599, E0382, and E0500 in the D18 spike) |
| CLAUSE-6 | §1.8, 04:466 | Name-resolution part: over an iterator other than `a..b`, a `for` loop's invariant cannot name the loop pattern's bindings | rustc (through the contract macros' lowering) | E0425: the head comes before `next`, so the bindings are out of scope |
| SEAL-2 | §11.1, 04:1391 | What an interface, whole, or manifest seal entry fixes (content and identity, checked by SEAL-3) | checker (admission driver) | FX0920 via SEAL-3 |
| SEAL-3 | §11.1, 04:1482 | After rustc's analysis and before any artifact is written, the driver checks the crate against its own `tide.seal`; a changed or missing item, or an unlisted target, is an error and the crate is not admitted | checker (admission driver) | FX0920 |

SEAL-2 and SEAL-3 need a crate together with its `tide.seal`, which a
one-file case cannot hold; their cases wait for the admission driver.

## Considered and excluded (runtime, simulator, sandbox, tooling, non-rejecting, or v1)

- PANIC-1 (03:347): runtime abort, and the harness reruns in-flight tests.
- PANIC-2 (03:363): runtime abort on a panic during unwinding.
- SINK-1 (03:377): runtime/library semantics of sinks.
- SINK-2 (03:382): runtime sink configuration and defaults.
- SINK-3 (03:387): runtime argument evaluation.
- SINK-4 (03:394): runtime output ordering.
- SINK-5 (03:412): runtime loss and backpressure.
- SINK-6 (03:420): simulator trace.
- SINK-7 (03:422): output elision under memoization.
- Memoization (03:852, 02:927): rejects nothing; the compiler memoizes only calls that meet AFREE-2's conditions.
- SINK-9 (03:430): v1, plus lockfile tooling.
- GLOBAL-5 (03:317): `HashMap` key behaviour in the library at runtime.
- RUN-1 (03:498): sandbox guarantee.
- RUN-2 (03:517): build sandbox and file loader. Its compile-time rejections are ADMIT-1 (FX0520), FX0911, and FX0915, which are build tooling.
- RUN-3 (03:555): test-process sandbox.
- RUN-4 (03:604): hole list. Only hole 1's grant requirement is compile-time, and it is a row above.
- RUN-5 (03:644): `tide review` coverage.
- RUN-6 (03:666): result channel and output escaping.
- TASK-1 (03:1127): runtime failure and panic propagation. Its `Owned` closure typing is the 02:692 row.
- TASK-2 (03:1135): runtime scope-outcome precedence.
- TASK-3 (03:1177): runtime level-triggered cancellation.
- TASK-4 (03:1209): runtime deadlines.
- TASK-5 (03:1223): runtime list of what cancellation cannot interrupt.
- TASK-6 (03:1231): runtime shielding.
- JOIN-1 (03:1240): runtime; scope exit joins and is shielded.
- JOIN-2 (03:1243): runtime `Task::join` semantics.
- LOCK-2 (03:1309): runtime `Aliased` check.
- LOCK-3 (03:1494): multi-cell acquisition in cell ID order and deterministic simulator IDs await fx-ugm.
- LOCK-6 (03:1325): runtime poisoning.
- PAR-1 (03:1445): runtime determinism.
- PAR-2 (03:1453): runtime panic order.
- PAR-3 (03:1461): float codegen and the math library. Only its allowlist part is a row above.
- SUSP-1 (03:1357): compile-time may-suspend classification that rejects nothing.
- SUSP-2 (03:1368): compile-time per-instance classification that rejects nothing.
- SUSP-3 (03:1371): codegen refinement.
- REPLAY-1 (03:1662): simulator manifest.
- REPLAY-2 (03:1671): `tide sim --replay` identity check (tooling).
- CLAUSE-5 (04:451): compile-time class assignment that rejects nothing.
- CLAUSE-6 (04:460): runtime check schedule. Only its name-resolution part is a row above.
- CLAUSE-7 (04:478): runtime violation outcome.
- CLAUSE-8 (04:506): approval hash (review and CI tooling).
- SEAL-1 (04:1336): `tide seal` command and file format (tooling).
- SEAL-4 (04:1500): FX0921 from `tide review` and the CI gate, not from compilation.
- SEAL-5 (04:1518): review output.
- SEAL-6 (04:1522): informative limits.
- v1, excluded by tier: FFI-1 to FFI-7 (03:969-1015), RUST-1 to RUST-6 (03:1778-1812), LOCK-7 (03:1330), and 02 §7 field-borrow rules (02:561-600).
- Non-rules: `SE-0414` (02:1335) matches the ID pattern but is a Swift proposal.

## Tier unclear

- `#[contracts(assert | test | prove)]`: 04:15 puts per-item uses (§2.1) in v1, but 04:216-217 (§1.2 rule 10) lists `#[contracts]` among the v0 attributes that satisfy the loop-clause rule, and 04:84 lists it without a tier. Also, 04:515-516 says an in-source module override (`#![contracts(test)]`) is rejected because it is unstable Rust, and that sits in the v1 section.
- FX0516 (authority policies): 01:956 says v0 and ref:1044 gives the code, but no chapter in 02/03/04 defines the rule.
- ROOT-4 names `Shape` construction (D6), and `Shape` reflection is Later tier in 01:84.
