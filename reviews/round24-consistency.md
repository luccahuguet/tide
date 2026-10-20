# Tide consistency review, round 24: confirmation of the round-23 answers

## Verdict

**approve with notes**

Reviewed the uncommitted working tree on 2026-10-20 against HEAD `557d4e5`
("Add the E1 dialect round and rounds 21-22; close fx-jv9"):
`git diff -- design/`, which changes `01-design.md` (D10), `02-memory.md`
(§7 API stability), `04-contracts.md` (§1.4, CLAUSE-2, CLAUSE-5, CLAUSE-6,
CLAUSE-7, §2.3, §2.4), `reference.md` (10.1, 10.2), and the companion's
recorded count. It implements bead fx-eky, whose two comments of 2026-10-20
record the owner's decisions. Scope, as asked:

- whether each round-23 item (B1, B2, N1–N6) is closed as its Resolution
  claims, including the parts it declines;
- whether the changed lines contradict 01, 02, 03, 04, the README, the rest
  of the reference pair, or the owner's decisions on fx-eky;
- the core's token count against the companion's figure and D14's budget.

Rule preservation beyond the changed lines was not re-audited; round 23 did
that for the first pass of this change.

## Round-23 item status

| Item | Status | Evidence |
| --- | --- | --- |
| B1 | **Resolved** | `reference.md:882–884` reads "Debug and test builds evaluate checked clauses at the points in the table (debug builds: `for` over `a..b` only at entry and exit)". This matches CLAUSE-6 (`04:467–468`), 01 D10 (`01:607–609`), and the owner's second comment. The Visibility bullet is folded into "Clause expressions" (`reference.md:867–868`), and the vacuity bullet lost its sorted-`Vec` aside (`:932`; see N3). |
| B2 | **Resolved**, with a completeness point (N2) | `04:486–489` lists both differences. Both match the spike: the `a..b` check sits inside `loop` before the bound test (`lib.rs:340–342`), gated only by `cfg!(debug_assertions)` (`:130`), and the message is `"{kind} violated: {text} (in `{fname}`)"` (`:129`). |
| N1 | **Resolved** for `for` loops and early exits; the `loop {}` part is **declined without a stated reason** (N1 below) | CLAUSE-6 puts a `for` head before each call to `next` (`04:465–466`), explains the debug rule by "its condition or iterator ends it" (`04:469–470`), and gives `break`, `return`, and `?` no exit check (`04:466–467`). 01 D10 says the same (`01:606–610`). The early-exit rule follows from the owner's "the check before the failing evaluation is the exit check", and matches the spike, where `break` leaves the rewritten `loop` without reaching the check (`lib.rs:340–347`, `:356`, `:373`). |
| N2 | **Resolved** | `04:595–597`: "never reports a `requires` violation (a failing `ensures` is still reported)". |
| N3 | **Resolved** | CLAUSE-5 lists "field reads" (`04:452`), and the reference's list adds "fields" (`reference.md:879`). `self.len == old(self.len) + 1` is now `checked`. |
| N4 | **Resolved**, wording point in N3 | `02:586–588`: "a declaration never puts private field names in a public API (a v0 clause may still read them, 04 CLAUSE-2)". |
| N5 | **Resolved** | `reference.md:937–938` says "calls that meet their `requires`", so "skipping" at `:943` now refers only to functions "skipped as unchecked" (`:941`). |
| N6 | **Resolved**; the sampled schedule stays out of the core, **reason holds** | `01:600–612` are all 77 characters or fewer. CLAUSE-6's exit-check sentence is split (`04:461–464`). With 4 tokens of headroom, leaving `tide test`'s schedule to 04 §2.4 is right. The answers added two long lines (N3). |

**Owner decisions.** Each comment on fx-eky is carried out, and none is
contradicted:

- G11: CLAUSE-2 (`04:397–400`), §1.4 (`04:302–304`), `reference.md:867–868`;
- G10: `04:595–599`, CLAUSE-7's prefix (`04:479–483`), `reference.md:888–890`
  and `:937–938`, with both "provisional" markers gone;
- G12 and its consequence: CLAUSE-6 (`04:459–474`), §2.3 (`04:553`), §2.4
  (`04:571–575`), 01 D10 (`01:605–611`), `reference.md:828–834` and
  `:882–884`.

**Leftovers.** A grep of `design/`, `README.md`, and `GOVERNANCE.md` for
"loop entry", "entry and exit", "after every iteration", "private-in-public",
"private field", "loop head", and "violated" finds no claim the change left
behind. 03 says nothing about clause visibility or loop invariants, and the
README does not describe either. The companion's only change is its count.

**Budget.** `scripts/count-tokens.sh design/reference.md </dev/null` printed
**24,996**. That equals the companion's recorded figure (`companion:12`) and
is 4 tokens under D14's 25,000.

## New blocking problems

None.

## Non-blocking notes

### N1. Two loop heads are still undefined: the loop variable over a general iterator, and `loop {}`

**The loop variable.** CLAUSE-6 now gives the next-index view only "over
`a..b`" (`04:466`), and round 23's Resolution says so ("the next-index view
only over `a..b`"). §2.4 still says "The function attribute's loop rewrite
binds the next index; the D18 spike does so for `a..b` ranges, not yet for
`a..=b` or general iterators" (`04:585–587`). That implies the design binds
a next index for a general iterator, which has none. In the spike, the check
comes before `let Some(pat) = next(..)`, so an invariant cannot name the
loop variable there at all (`lib.rs:354–356`). Neither text says what
`x` denotes in an invariant of `for x in iter`.

**`loop {}`.** CLAUSE-6 and 01 D10 say debug builds check "every other loop
at every head" (`04:468–469`, `01:609`), which includes `loop {}`. No text
places that head, and the reason given, "its exit is known only once its
condition or iterator ends it", does not fit a loop with neither. Round 23's
Resolution keeps `loop {}` "as 04 §2.4 already leaves it", but §2.4 says
only that the spike does not rewrite it (`04:587`). The natural reading,
the top of the body where `invariant!` sits (§1.2 rule 10, `04:213–219`),
is probably what is meant.

**Fix** (04 only, no core tokens). In CLAUSE-6, after the `a..b` clause:
"over any other iterator the loop's pattern is not in scope in an invariant.
A `loop`'s head is the top of its body, and it gets no exit check." In §2.4,
change "not yet for `a..=b` or general iterators" to "not yet for `a..=b`".
If the owner wants the variable in scope over general iterators, that is a
new decision for fx-eky, not a wording fix.

### N2. CLAUSE-7's list of spike differences omits clause classes and generated tests

`04:486–489` now reads "Where the D18 spike differs: …", followed by "Not
yet implemented in the D18 spike: …". The two lists read as complete, but
the spike has no clause classes: it evaluates every clause, `sampled` ones
included, in every debug build (`lib.rs:130`), and it has no `tide test`
mode or generator. So the spike exercises neither G11's consequence
(a field read makes a clause `checked`, not `sampled`) nor G10's generator
rule. Both gaps predate this change, which is why this is not blocking, but
the change's new sentence makes the lists look exhaustive.

**Fix** (no core tokens). Add to the "Not yet implemented" list: "clause
classes (it evaluates every clause in every debug build), and generated
tests".

### N3. Nits

- **The reference's "exit".** The table row says "every iteration, exit"
  (`reference.md:828`), and nothing in the core says that a loop left by
  `break`, `return`, or `?` gets no exit check (`04:466–467`). The cost is
  only that a reader may expect a check that never runs. Add "(not after
  `break`)" only if a later change frees the tokens.
- **The vacuity example lost its subject.** "For a rare condition, construct
  such inputs (`xs.sort()`)" (`reference.md:932`) no longer says which
  condition `xs.sort()` produces. The recognized shape `xs.is_sorted()`
  (`:926`) lets a reader infer it, so this is acceptable as the price of B1.
- **"v0 clause" is narrower than the owner's rule.** The owner decided that
  §1.4's visibility "applies only under prove", and CLAUSE-2 says so
  (`04:399–400`). §1.4's parenthetical says "v0 clauses resolve names as
  the body does" (`04:303–304`), and 02 says "a v0 clause may still read
  them" (`02:588`). Under `assert` or `test` in v1 the same rule holds, so
  "a clause outside `prove`" would be exact. In 02, the parenthetical also
  sits awkwardly after "a declaration never puts private field names in a
  public API", since a clause is part of the declaration. Round 23's wording
  ("never enter a public signature through a field-borrow set") avoids that.
- **D10's use list.** `01:666–667` says the `test` use adds "and loop
  invariants are checked", which implies `assert` does not check them.
  Under the new rule, `assert` checks them at every head except inside a
  `for` over `a..b`. The phrase predates this change. "and every loop head
  is checked" would match CLAUSE-6.
- **Line length.** The answers added two lines over 80 columns:
  `04:454` (98 characters, N3's fix) and `04:597` (121, N2's fix). `04:470`
  and `04:575` end after a few words and could be reflowed with them.

## Ready to merge

**Yes.** Every round-23 item is closed, and B1's fix keeps the core at
24,996 tokens, under D14's 25,000. No changed line contradicts 01, 02, 03,
04, the README, the reference pair, or the owner's decisions on fx-eky.
N1 and N2 cost no core tokens and can land before merge. N1 needs the owner
only if the loop variable should be in scope over general iterators.

Validation: this was a read-only review of the working tree at `557d4e5`,
with five `design/` files modified, on 2026-10-20.

- **Read:**
  - the full diff;
  - 01 D10, including its use list;
  - in 02, the API-stability bullets;
  - in 04: §1.2 rule 10, §1.4, §1.8 CLAUSE-1 to CLAUSE-8, §2.3, and §2.4;
  - reference 10.1 and 10.2;
  - the spike's module header, `check`, and `Loops` rewrite
    (`spikes/d18/macros/src/lib.rs`);
  - fx-eky with its comments, via `br show` and `br comments`.
- **Leftovers:** grepped `design/`, `README.md`, and `GOVERNANCE.md` for the
  phrases listed above, and for "invariant" in 01, 03, the companion, and
  the README.
- **Line lengths:** checked every added line of the diff, and 01 D10, for
  lines over 80 columns.
- **Count:** `scripts/count-tokens.sh design/reference.md </dev/null` was
  run once, with a 400 s timeout, on the working tree. It printed `24996`
  and exited 0.

No Tide checker exists, and the spike was read but not run, so every verdict
above is an inference from the text. Only this file was written; nothing was
staged or committed.

## Resolution

Answered in the commit that adds this review.

- **N1:** CLAUSE-6 says a `for` over a non-range iterator cannot name the
  loop pattern's bindings in an invariant and that a `loop`'s head is the
  top of each iteration; §2.4 says the rewrite binds the next index of an
  integer range, and drops "general iterators" from the spike's gaps.
- **N2:** CLAUSE-7's not-yet-implemented list adds clause classes (the
  spike evaluates every clause in debug builds) and generated tests.
- **N3:** 04's two long lines are rewrapped. Declined, for the core's
  4-token headroom or as predating this change: `break` in the reference,
  the `xs.sort()` example, the "v0 clause" wording, and D10's use list.

The verdict is approve with notes, so no further round is run.
