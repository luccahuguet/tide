# Tide review, fx-gcs.3 round 1 (Opus): authority, admission, and Pure cases

Candidate: commit 19bc1a2 on main e2a7063 (worktree tide-fx-gcs.3), read-only
review. `conformance/run.sh` on 19bc1a2 (rustc 1.96, offline, load ~6/16):
exit 0, 32 PASS, 44 AWAITS CHECKER, about 6 s. `scripts/count-tokens.sh
design/reference.md`: 24,996.

## Verdict

revise

The corpus is broad and mostly right, and the runner change is sound. Six
findings need a fix before merge. Two are design contradictions this change
creates (FX0518 versus §3.1's code list, and GLOBAL-1 versus reference 6.4).
Three are cases whose verdicts the design does not support: ROOT-1 against
`root_forge` and `root_fields`, and the unmarked method call in `admit_waker`.
The last is AFREE-2 and the ROOT-4 `zeroed` clause, which the open items
raise, and which need explicit text and a case each.

## Blocking

**B1. FX0518 is a `Pure`-needing rule's own code, but §3.1 and D18 still
send every other `Pure` failure to FX0517.**
`design/03-authority-concurrency.md:788-791` says a `Pure` failure goes
"under the code of the rule that needs `Pure` (`FX0502`, `FX0603`, `FX0606`,
`FX0611`, `FX0612`), and otherwise ... as `FX0517`". `design/01-design.md:1009-1011`
says the same. AFREE-2 (`03:919-922`) now reports a `const fn` parameter
that is not `Pure` as `FX0518`, so the two texts give two codes for one
failure. `conformance/rules.md` PURE-1 rows (65, 94) repeat the old list.
*Fix:* add `FX0518` to the list at 03 §3.1, 01 D18, and the two rules.md
rows.

**B2. Reference 6.4 still lists a `Cell` static as FX0502, but GLOBAL-1 and
the case leave it to rustc.**
`design/reference.md:431-433`: "(`static mut`, a `static` holding atomics,
`Mutex`, or `Cell`, `thread_local!`, `lazy_static!`: `FX0502`)". GLOBAL-1
as edited (`03:355-357`) says "rustc reports a missing `Sync` itself", and
`conformance/cases/authority/global_static_cell.rs:5` marks only `E0277`.
`Cell` is `Pure` and not `Sync`, so only the `Sync` check fails. Once
the checker's FX marks are enforced, the reference and the case cannot both
be right. GLOBAL-1's "Both are `FX0502`" also reads as covering the `Sync`
requirement in the sentence before it.
*Fix:* drop "or `Cell`" from reference.md:432, which also saves tokens. Then
reword GLOBAL-1 to "A non-`Pure` static and a `static mut` are `FX0502`;
rustc reports a missing `Sync` (E0277)."

**B3. ROOT-1 makes two ROOT-4 cases wrong. Its naming code is stated only
in the companion.**
- `conformance/cases/authority/root_forge.rs:12-13` names `Root` as a
  return type and in a struct literal outside `main`. ROOT-1 (`03:323`)
  forbids that, and `reference-companion.md:41` gives `FX0503` for
  `fn helper(root: &Root)`. Neither line carries an FX mark, so the case
  fails for an unrelated reason once FX marks are enforced.
- `conformance/cases/authority/root_fields.rs:5`,
  `let Root { fs, clock, .. } = root;`, names `Root` in `main`'s body. ROOT-1
  says `Root` "may be named only as the parameter", and reference 6.2 says
  "only in `main`'s signature". The design does not state the accepting
  verdict this case expects.
- `root_borrowed.rs:3` expects `FX0503` for naming `Root` in a helper. That
  code comes only from the companion: 03 ROOT-1 states no code for naming
  `Root`.
- `conformance/rules.md` ROOT-4 row, rustc column: the cases show an uncoded
  private-field `error` and `E0599`, not `E0451 / E0277`.

*Fix:* have `root_forge.rs` forge a capability, not `Root`:
`pub fn forge() -> Fs { Fs(()) } //~ E0423`. I checked this against the
stub; rustc gives E0423. Delete `root_fields.rs` and add `ROOT-4` to
`root_main_accept.rs`'s rules line, which already moves and borrows fields
out one by one. Add "(`FX0503`)" to ROOT-1's naming sentence, and fix the
rules.md ROOT-4 rustc column.

**B4. `admit_waker.rs` leaves a rejected method call unmarked.**
`conformance/cases/authority/admit_waker.rs:4` calls `w.wake_by_ref()`, a
method of a type that is not admitted. ADMIT-1 (`03:500`) resolves "every
path, method call, ...", so the checker rejects line 4 as well. The spike
agrees: `spikes/d18/admit/tests/neg/cpuid.rs` marks the use of the
non-admitted `CpuidResult` on its own line. With FX marks enforced, line 4
is an unmarked `FX0520`.
*Fix:* mark line 4 `//~ FX0520`, or make the body empty
(`pub fn wake(_w: &core::task::Waker) {}`).

**B5. Open item (a): AFREE-2 does not say how it treats type parameters,
and reference 2.3 omits `Sync`.**
- **The drafter's reading is the design's.** 03 §3.2 (`03:846-848`) says
  "a generic definition that needs the property states it as a bound".
  D4 (`01:348`) says generics are definition-checked. 02 §9 (`02:895-897`)
  already applies this to `io::Error` wrappers: "the check runs at its
  definition (D4)". So `impl<T> Stack<T> { const fn len(&self) }` needs
  `T: Pure + Sync`, and an unbounded one is `FX0518`. The alternative,
  exempting type parameters as the spec-fn row does (`03:916`), would need
  a reason D4 does not give.
- **Nothing states or tests the consequence.** It reaches common Rust
  idioms (`const fn new(v: T) -> Self`, `const fn len(&self)` on a generic
  type), yet AFREE-2's text does not mention it and no case pins it.
- **Reference 2.3 omits `Sync`.** `design/reference.md:99-100` gives
  `const fn` as "authority-free (6.6), deterministic, and fuel-limited,
  with no `&mut` parameters". An agent who loads only the core cannot know
  that `const fn peek(c: &Cell<u32>)` (`afree_const_cell.rs`) is rejected;
  only `reference-companion.md:39` says so.

*Fix:* add to AFREE-2: "A type parameter, including one in `Self`'s type,
needs the bound at the definition (§3.2, D4)." Add one case with an
unbounded generic `const fn len(&self)` marked `FX0518` and a bounded one
accepted. In reference 2.3, write "with `Pure + Sync` parameters, none
`&mut` (`FX0518`)" in place of "authority-free (6.6), ... with no `&mut`
parameters". That is roughly token-neutral, and the B2 cut gives slack.

**B6. Open item (c): ROOT-4's `mem::zeroed`/`transmute` clause has no case.**
`03:333-334`: "`mem::zeroed`, `transmute`, and raw-pointer reads in safe std
require `T: Pure`". This is the only `Pure` obligation that guards a
capability under the `unsafe` grant, and the corpus can express it. I checked
against the stub that rustc accepts
`pub fn z() -> Fs { unsafe { std::mem::zeroed() } }`.
*Fix:* add one case with `//@ rules: ROOT-4, UNSAFE-1` and
`//@ grants: unsafe`: a `// SAFETY:` comment, then
`unsafe { std::mem::zeroed::<Fs>() } //~ FX0517`, and an accepted
`zeroed::<u64>()`. "In safe std" probably means "std's own (safe-facing)
versions". If the author reads it differently, reword it while there.

## Non-blocking

**N1. Open item (b): marking a line with both rustc's code and the FX code
is right.** CAP-1 and ADMIT-1 say the driver reports `FX0501`/`FX0520`
"not as rustc's unresolved name". The corpus records rustc's verdict and
the checker's separately (README "the corpus compares the two checkers").
So `E0425 FX0501` correctly means "raw rustc says E0425; Tide says
FX0501". Optional: one README clause saying that on such lines the driver
replaces rustc's diagnostic.

**N2. CAP-3's code breaks the pattern for grant-gated items.** Grant-gated
*names* are `FX0520` (raw pointer types, `global_asm!`; reference:957,
03:667), and grant-gated *syntax* is `FX0901` (`unsafe`, unsafe attributes,
`#[trusted_pure]`). `std::sync::raw` is a name, so the pattern gives
`FX0520`. FX0901's "add the grant" fix also fits, so either is defensible.
Recording the reason on the bead would be enough.

**N3. Two cases have the same shape.** `admit_once_lock.rs` and
`global_lazy_lock.rs` are each one `use std::sync::X; //~ E0432 FX0520`
line. They could be one case (rules `GLOBAL-2, ADMIT-1`) with both lines
(AP-MINIMAL-001).

**N4. Coordination with fx-gcs.2: UNSAFE-1 omits `unsafe impl Send`/`Sync`.**
02 §8.1 (`02:623`) gates them, and reference:954 gives them `FX0901`. That
row (rules.md:54) is fx-gcs.2's. Either fx-gcs.2 cites UNSAFE-1, and then
UNSAFE-1 should list unsafe impls, or it gives the rule its own ID. Tell the
other session.

**N5. Merge-conflict risk in 02 §8.4.** The edits at `02:738-749`
(LOCK-1/LOCK-4 citations) sit next to the `Shared::with` escape bullet that
fx-gcs.2 owns, so expect a conflict. The rewrap also left long lines at
`02:749` and `03:788`.

**N6. PURE-1 and PURE-2 lead paragraphs, not bullets.** The README's ID
format says "at the start of the rule's bullet or list item". Widen it to
"or paragraph", or make them list items.

**N7. A rules.md row has a stale "Decided by" value.** The
`= PURE-1 | §3.1, 03:762` row says `both`, but its rustc part (E0225) now
belongs to 02 §5.1. It should say `checker`.

**N8. `run.sh` reads only the first `//@` line.** A typo such as
`//@ grant: unsafe` is silently ignored. That costs nothing until the
checker reads directives, when it would quietly change a case's meaning.
Optional: reject unknown `//@` keys.

**N9. The spike's `cpuid` negative case was not ported.** It is the first
item ADMIT-1 lists, but the `core::arch::x86_64` path makes it
arch-specific. Port it under `cfg(target_arch = "x86_64")` or note why it
was skipped.

**N10. Wording nits.**
- `cap_handle_tasks.rs:2` says each constructor takes `&Tasks` "last",
  while CAP-2 calls the position provisional.
- CAP-1 sits as a rule bullet in §0 "Terms".

## Checked

- **Governance:** GOVERNANCE.md Merge Rule; the bead and fx-gcs.1, with
  inventory findings #1, #4 and #5. #1 (`thread_local!`/`lazy_static!`
  under FX0502 in reference.md:61-64, 03, and the companion) and #4
  (FX0518, and CAP-3's code) are addressed. For #5, the 02 restatements now
  cite LOCK-1/4/5, GLOBAL-6, ADMIT-1, UNSAFE-1, and AFREE-2, and 03 §3.1
  cites PURE-1/2.
- **Runner:** `conformance/run.sh` exits 0 (32 PASS, 44 AWAITS). The five
  seeds (`admit_atomic`, `lock_accept`, `lock_payload_not_send`,
  `lock_payload_shared`, `tasks/par_map_cell`) are unchanged and keep
  their verdicts. `RUSTC_BOOTSTRAP=1` is scoped to rustc only, not the
  cargo build of the stub. `-Zallow-features=` matches ADMIT-2 and gives
  E0725 in `admit_feature.rs`. `--trace` works. The rules with no case are
  PAR-3, CLAUSE-*, and SEAL-2/3, all other beads'.
- **Every one of the 71 new cases against its rule text:** CAP-1..3,
  ROOT-1..5, GLOBAL-1..4/6, SINK-8/10, ADMIT-1/2, UNSAFE-1, LOCK-4/5,
  PURE-1/2, AFREE-1/2. Each in-scope ID has accepting and rejecting cases,
  except where B3 and B5 note gaps. I ran rustc directly on `root_forge`,
  `global_lazy_static`, `global_thread_local`, and `cap_ambient` to confirm
  the uncoded and coded errors fall on the marked lines.
- **Stub additions against 03 §1.2 and reference 6.4/6.5:** `Arc`/`Weak`
  signatures, missing count observers, `make_mut`, and
  `downgrade(&a, tasks)`. `Root`'s private field. `Secret` (prelude, no
  `Debug`/`Display`; `new` marked GUESS). `sync::raw` marked GUESS. The new
  `conformance/rustc/macros` crate is needed, since neither existing macros
  crate defines `pure` or `trusted_pure`.
- **FX0518:** it collides with nothing (grepped the whole repo, archive
  and trials included). It appears in 03 and the companion. B1 and B5
  cover the places it is missing.
- **Reference edits:** the core stays at 24,996 tokens, and the companion's
  count line is unchanged and still correct. Dropping "`Arc` is not `Pure`"
  from reference 6.6 is safe, since `Arc` stays in the not-`Pure` list two
  lines above.
- **CAP-1 against ADMIT-1:** no conflict. CAP-1 covers free functions and
  constructors that duplicate a capability method. `Path::exists` is a
  method, and `io::Error::last_os_error` duplicates none, so both stay
  `FX0520` as ADMIT-1 says.
