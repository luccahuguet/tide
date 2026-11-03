# Tide review, fx-k0z round 1 (Opus): converge round, report only

Candidate: commit 0939683 on branch fx-k0z (worktree `.worktrees/fx-k0z`),
`git diff main...fx-k0z`, SHA-256
`cade039e6018d5b427ddf0a683b542de1ae63ba258732e0aeebdb7e53a671049`
(re-hashed, matches). This is the first-round Opus half of the parallel pair.
It reports and does not fix. `git diff --check` passes.

## Verdict

**approve**. Confidence: high (about 0.85).

No blocking findings. Every bead item is fixed or decided, and the clock
follow-through is done. Nothing in `design/` or `conformance/rules.md` now
contradicts the result. The core measures 24,998 tokens, which I re-ran
with `scripts/count-tokens.sh design/reference.md`. Both trims are covered
elsewhere in the core. The cases follow `conformance/README.md` and test
what they claim. Seven non-blocking notes follow. N1 is the most useful.
N6 records a divergence outside the bead's scope that needs a decision.

## What was reviewed

- Brief: AGENTS.md "Governance", GOVERNANCE.md "Merge Rule", `br show fx-k0z`
  (items 1–6, the clock follow-through, acceptance).
- The whole diff (8 files), and the surrounding text it depends on:
  - reference 1, 2.1, 4.2, 6.2, 6.5, 6.8, 6.9, 11;
  - 03 CAP-1, CAP-3, ROOT-1..3, GLOBAL-4, ADMIT-1, UNSAFE-1, RUN-2, RUN-4,
    and §3.1;
  - 02 §9 (PURE-1 and the lists after it); 01 D3, D7, D14, D15;
  - companion 14.1 and 14.2; `conformance/README.md`; the `rules.md` rows
    CAP-1, CAP-3, ROOT-1, ROOT-3, GLOBAL-4, ADMIT-1, UNSAFE-1;
  - `crates/tide-admit/src/policy.rs:66-100` and the raw-pointer paths in
    `walk.rs`;
  - the related cases `admit_hardware`, `cap_ambient`, `cap_sync_raw`,
    `global_pointer_addr`, `global_pointer_granted`, `global_fn_pointer`,
    and `unsafe_granted`.
- Stale-text sweep across `design/*.md`, `conformance/rules.md`, and
  `conformance/README.md` for: `asm!`/`global_asm!`, `Root::sim` and
  "named only", `sync::raw`, `trusted_pure`, `*const`/raw pointer/pointer
  casts, `core::arch`, `elapsed`, `Instant`/`SystemTime`.
- I did not rerun `conformance/run.sh`, as the brief says. Nothing in the
  source gives me a reason to doubt the author's PASS / AWAITS CHECKER
  result. Both rejecting lines in `cap_elapsed.rs` are E0599 on a stub type
  without the method, and the shape matches `cap_ambient.rs:8`.

## Items

| Item | Result |
| --- | --- |
| 1 `asm!` under the grant | Fixed. Ungranted `asm!` is now `FX0520` in reference 2.1:63, 03 ADMIT-1:557-559, UNSAFE-1:707, RUN-4:821, 01 D7:430-431, and `rules.md` UNSAFE-1. Granted `asm!` is admitted by implication: it is carved out of 2.1's `FX0103` catch-all, and `unsafe_granted.rs:31` admits it. `unsafe_global_asm.rs:8` now covers the ungranted side (`FX0901 FX0520`). 01 D15:843-844 ("unsafe grants (unsafe attributes and `global_asm!` included)") is still right, because `asm!` needs an `unsafe` block |
| 2 `Root::sim` | Fixed. Reference 6.2:373 now agrees with ROOT-1, `rules.md` ROOT-1, and D7:417-418. Existing cases cover both sides: `root_sim_in_test`, `root_sim_outside_test` |
| 3 hole table codes | Fixed. Reference 11:953-954 now gives `FX0901` and `FX0520`, which match UNSAFE-1, CAP-3, and `cap_sync_raw.rs` |
| 4 `core::arch` | Agreed (below) |
| 5 pointer casts | Agreed. It is complete for what the bead asked; see below and N2 |
| 6 03 §3.1 trim | Fixed. Nothing is lost (below) |
| Clock follow-through | Done. The decision (`FX0501`) is in CAP-1 (03:64-71). Reference 6.9:547 lists `t.elapsed()`. `cap_elapsed.rs` has accepting and rejecting sides. The stub adds `Clock::system_time` (signature from reference 6.1) and `Instant::duration_since` (marked GUESS). `rules.md` CAP-1 already lists `FX0501` with E0599 |

**Item 4.** I agree with leaving reference 6.9 as it is. ADMIT-1 rests on
"admits only listed items". Its grant-gated entries are named explicitly:
raw pointers and `std::sync::raw` "without an `unsafe` grant" (03:534), and
`asm!` and `global_asm!` (03:557). The hardware-intrinsics bullet (03:533)
has no grant qualifier, and neither does D7 (01:424). `policy.rs:86-90`
matches this: `deny("core::arch::*")` and `deny("core::core_arch::*")` sit
after the three `granted(...asm)` rules. In 6.9, raw pointers carry
"without the `unsafe` grant" and intrinsics do not, so the two readings are
consistent. The bead's premise, that intrinsics need the grant, was wrong;
they are denied whether granted or not. The only remaining friction is
that 6.9's "(`core::arch`)" gloss also covers the `asm!` macros, which 2.1
then carves out. The more specific rule wins, so this is not a
contradiction (see N3). The "no change" decision and its reason should be
recorded on the bead when it closes (N5).

**Item 5.** GLOBAL-4:389-397 ("casts to them included") and reference 11:955
("casts to or from them") settle the ungranted case: a cast to or from a raw
pointer is `FX0520`. That matches the driver, which rejects any expression
or type of raw-pointer type (`walk.rs:238`, `:369`). So the clause states
the existing verdict and changes nothing. `global_pointer_addr.rs:4` already
marks `x as *const u8` (together with `.addr()`) as `FX0520`, and
`global_pointer_granted.rs:6` admits it under the grant. I agree that 4.2
and D3 need no edit. 4.2 is about numeric `as`, and it does not claim that
every unlisted cast is `FX0301`, so it does not contradict 11. A narrow
corner is still open; it is outside the bead (N2).

**Item 6.** The removed 03 §3.1 sentence listed these properties. All of
them are in 02 §9:

- deep through every field, private ones too, and through `Box`/`Rc`
  (PURE-1, first bullet);
- "std's two exceptions" (02 §9 "Std's types ... with two exceptions");
- the greatest fixed point (PURE-1, second bullet);
- `impl Trait` and `dyn` auto traits and `Pure` only as their bounds state
  (PURE-1, third bullet);
- `Owned` follows origins (PURE-1, third bullet, §3.7);
- type parameters and unnormalized projections only when bounded (PURE-1,
  fourth bullet).

The new sentence keeps the citation and the `dyn` companion pointer.

**Budget and trims.** I measured 24,998, at or under 25,000. The companion
line is updated. The trims removed:

- the `include_str!`/`env!`/`option_env!` tail of 6.9, which is still
  stated in 2.1:62-63 as `FX0520`;
- "an error in release, `FX0507`" in 6.5, which is still in 2.1:55-56;
- "the companion records the count" in the header. The companion's line and
  D14 still carry the count and the rule.

Nothing an agent needs was lost.

## Blocking findings

None.

## Non-blocking notes

**N1. Companion 14.2 has no row for `elapsed`.** Reference 6.9:547-550
lists `t.elapsed()` as `FX0501` and says "the fix uses a capability in
scope ... (companion 14.2)". 14.2 (`reference-companion.md:68-80`) has rows
for `Instant::now()` and `SystemTime::now()` but none for `elapsed`, so an
agent that follows the pointer finds no replacement. The case's comment
gives the answer. Smallest patch, outside the core budget, one row after
line 77:
`| `start.elapsed()` | `clock.now().duration_since(start)`; for a `SystemTime`, `clock.system_time().duration_since(t)` |`

**N2. Lossy pointer casts under the grant are undecided.** D3 (01:318)
accepts `as` "only where it is lossless". GLOBAL-4 and reference 11 admit
"casts to or from" raw pointers under the grant. Two cases are therefore
open: whether `p as u8` and `f as u8` (a truncating address cast) are
`FX0301` under the grant, and whether `usize as *const T` counts as
lossless. This only affects granted code. The bead puts new rules out of
scope, and no verdict here was decided before. File a follow-up if wanted;
one clause in GLOBAL-4 or D3 settles it ("under the grant, D3's lossless
rule still applies to address casts"). Not for fx-djp, since the fix is v0
text.

**N3. 6.9's `core::arch` gloss is still slightly broad.** "hardware
intrinsics (`core::arch`)" at reference.md:553 also covers `asm!`, which 2.1
admits under the grant. It is consistent, but a reader who reads only 6.9
might think `asm!` is always `FX0520`. With 2 tokens of headroom, leave it
unless a later trim frees space. A possible wording is "hardware intrinsics
(`core::arch`, even granted)".

**N4. The new §3.1 sentence repeats a citation.** 03:845-847 cites PURE-1
again, four sentences after 03:832 already did, and the `dyn` half repeats
the "Not `Pure`" list three lines below. One option is to cut it to
"`02-memory.md` §9 has the full rule." Note that §9's content is a list,
not a "table". Style only.

**N5. Record the item 4 and 5 decisions on the bead.** The acceptance says
each item is "fixed, or decided with reasons and recorded". The commit
message is bare. Item 4 is a no-change decision, so its reason exists only
in this review. Before closing, add a comment on fx-k0z covering:

- item 4: the reasoning above, citing 03:533 and `policy.rs:86-90`;
- item 5: settled in GLOBAL-4, with 4.2 and D3 unchanged because they cover
  numeric casts only;
- the elapsed decision: `FX0501`, recorded in CAP-1.

**N6. `naked_asm!`: the driver and the design diverge (pre-existing, out of
scope).** `policy.rs:88` grants `core::arch::naked_asm`. No design text
names `naked_asm!`, so reference 2.1's catch-all makes it `FX0103` even
under the grant. The author's note that 2.1 now names every granted
`core::arch` item is accurate for `asm!` and `global_asm!` only. Ungranted
use is still rejected, because the required `#[unsafe(naked)]` attribute is
`FX0901` and the macro is `FX0520`. The only gap is the granted verdict.
This needs a decision, not a fix in this change: either 03 ADMIT-1/UNSAFE-1
names `naked_asm!` beside `asm!`, or `policy.rs` denies it. The 03-side fix
is v0 text, so the route is a new bead, not fx-djp. My recommendation is to
name it in 03 next to `asm!` (its holes are the same) and leave the core
alone until there is budget.

**N7 (nits).** Some added or rewrapped lines run past the files' wrap width:

- reference.md:63, :375, :459;
- 03:393;
- 01:430-431.

Doc comment wording in the stub std differs slightly: "no `now()` or
`elapsed()`" for `Instant` against "no `now()` and no `elapsed()`" for
`SystemTime`. The case file `unsafe_global_asm.rs` now also covers `asm!`,
which its name does not say. None of this matters for the verdict.

## Minimal patch for approval

None required.
