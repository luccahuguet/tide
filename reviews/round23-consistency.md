# Tide consistency review, round 23: fx-eky (G10, G11, G12)

## Verdict

**changes requested**

Reviewed the uncommitted working tree on 2026-10-20 against HEAD `557d4e5`
("Add the E1 dialect round and rounds 21-22; close fx-jv9"):
`git diff -- design/`, which changes `01-design.md` (D10), `04-contracts.md`
(§1.4, CLAUSE-2, CLAUSE-6, CLAUSE-7, §2.3, §2.4), `reference.md` (10.1,
10.2), and the companion's recorded count. It implements bead fx-eky. The
owner's decisions are the bead's two comments of 2026-10-20. Scope, as
asked:

- whether each changed line carries out those decisions and agrees with 01
  (D10, D18), the rest of 04, 02, 03, and the reference pair, with a search
  for leftovers;
- whether the D18 spike (`spikes/d18/macros/src/lib.rs`) agrees with the new
  text, and whether 04 records any disagreement where it lists the spike's
  gaps;
- whether the reference stays accurate where it compresses 04;
- the core's token count against the companion's figure and D14's budget.

Rules outside the changed lines were not re-audited.

## Decision status

| Decision | Status | Evidence |
| --- | --- | --- |
| G11, names resolve as in the body | **Carried out** | CLAUSE-2 (`04:397–400`) and §1.4 (`04:302–304`) match the comment: private fields and functions are allowed, and §1.4's rule applies only under `prove`. The reference has it at `reference.md:886`. The spike splices clause tokens into the function body (`lib.rs:127–130`), so it already resolves names this way. One gap remains: N3 (classes) and N4 (02's API claim). |
| G10, generator skips a failing call | **Carried out**, wording point N2 | `04:589–591`; `reference.md:937–938`. |
| G10, full-range scalars biased to small and boundary values | **Carried out** | `04:591–594`; `reference.md:938` ("scalars over their full range"). The bias is left out of the reference, which is acceptable. |
| G10, fixed message prefix | **Carried out**; the spike differs (B2) | `04:476–480`; `reference.md:887–889` drops "(provisional)". |
| G12, check before each condition evaluation; the last check is the exit check; a side effect of the failing condition is unchecked | **Carried out** | `04:461–464`, `04:566–569`; 01 D10 (`01:605–607`); `reference.md:833–834`. Matches the spike's `while` rewrite (`lib.rs:373`). |
| G12 consequence: debug builds check every head except `for` over `a..b`; `tide test` checks every head; sampled invariants keep their schedule | **Carried out in 01 and 04; the reference contradicts it (B1)** | 01 D10 (`01:608–610`), CLAUSE-6 (`04:465–468`), §2.4's schedule unchanged (`04:569–572`). |

**Leftovers.** A grep of `design/`, `README.md`, and `GOVERNANCE.md` finds
no remaining claim that invariants run only "at loop entry and exit", that a
`pub` contract may call only `pub` functions in v0, or that the violation
text is provisional. 03 says nothing about clause visibility or loop
invariants. 02 has one related claim (N4). The companion's example loop
(`companion:254–263`) does not describe when the invariant runs.

**Budget.** `scripts/count-tokens.sh design/reference.md </dev/null`
printed **24,998**. That equals the companion's figure (`companion:12`) and
is 2 tokens under D14's 25,000. Any fix to the core (B1, N3, N5) has to give
up text of equal length elsewhere.

## New blocking problems

### B1. Reference 10.1 says debug builds check every iteration of a `for` over `a..b`

The new table row reads "checked at each loop head: entry (even for zero
iterations), every iteration, exit" (`reference.md:828`). "When clauses
run" then says "Debug and test builds evaluate checked clauses at the points
in the table" (`reference.md:881–882`). Together they tell a reader that a
debug build checks `invariant!` before every iteration of `for i in a..b`.
CLAUSE-6 says the opposite: "Debug builds check a `for` over `a..b` only at
entry and exit" (`04:465–466`). 01 D10 agrees with CLAUSE-6 (`01:608–609`),
and so does the owner's second comment ("for over a..b keeps entry and exit
in debug builds"). The old row had the distinction ("under `tide test`,
also after every iteration"). The new row dropped it, and fx-eky's
acceptance requires that a review find no contradiction among 01, 04, and
the reference pair.

**Fix.** Put the exception back in the core. Keep the row and change the
"When clauses run" sentence to:

> Debug and test builds evaluate checked clauses at the points in the table,
> except that debug builds check a `for` over `a..b` only at entry and exit;

This costs about 15 tokens, and the core has 2 to spare. One way to pay for
it is to fold the Visibility bullet (`:886`) into "Clause expressions" as
"…and are read-only and name what the body can, private fields included".
That drops the bullet marker and its bold label. Recount afterwards. If that
is not enough, shorten the Loops bullet's new sentence (`:833–834`) to "A
`while` or `while let` head precedes each condition evaluation."

### B2. The spike's disagreements with the new text are not recorded

fx-eky's implementation note 1 says to "check each decision against the D18
spike's contract lowering … and say where it would change". CLAUSE-7 lists
what the spike does not implement (`04:483–496`), and §2.4 lists the loop
forms it does not rewrite (`04:579–581`). Neither list, and neither bead
comment, records two places where the spike differs from the new text:

- **Debug-build loop checks.** The spike checks every head of a `for` over
  `a..b` in every debug build (`lib.rs:341`, inside `loop`, gated only by
  `cfg!(debug_assertions)` at `:130`). It has no `tide test` mode. CLAUSE-6
  checks only entry and exit there in debug builds. The spike checks more,
  so it is a superset, but the two still disagree.
- **Message.** The spike's panic text is
  `"{kind} violated: {text} (in `{fname}`)"` (`lib.rs:129`). The prefix
  matches CLAUSE-7, so `expected = "requires violated"` works. The message
  has no blamed span, though, and for `requires` CLAUSE-7's span is the
  caller's, which a check inside the callee cannot name without
  `#[track_caller]` or something like it. Also, `{text}` is the clause's
  token stream as rustc prints it, not its source text. That the printing
  can differ from the source in spacing is an inference; this review did not
  run it.

The section also says "Not yet implemented in the D18 spike", which implies
the spike implements the rest of the rules. With these two items missing,
that claim is false.

**Fix.** Add both to CLAUSE-7's list, which costs no core tokens: "checking
a `for` over `a..b` only at entry and exit in debug builds (it checks every
head), and the blamed span and source text in the violation message (it
prints the clause's tokens and the function name)". Alternatively, record
them on fx-eky and point to the bead from CLAUSE-7.

## Non-blocking notes

### N1. The head of a `for` over a general iterator, and of `loop {}`, is not defined

CLAUSE-6 says "A `for` loop's heads are its next indices (§2.4)"
(`04:464–465`), and 01 D10 says "at each next index of a `for` loop"
(`01:607`). §2.4 defines a next index only for `a..b` (`04:575–578`).
`for x in iter` over a general iterator has no index, yet the owner's
comment names that loop explicitly. 01 and CLAUSE-6 also explain the
debug-build rule as "its exit is known only once the condition fails"
(`01:609–610`, `04:467`), and a `for` has no condition.

`loop {}` is in a similar position. §2.4 implies the design rewrites it
(`04:581`, "does not yet rewrite `loop {}`"), but no text says where its
head is, and nothing says that an exit by `break` or `return` gets no exit
check. "After every iteration, and at exit" (`04:566–567`) and the
reference's "every iteration, exit" (`reference.md:828`) can be read as
including such exits. Dafny does not check them.

The spike already chose a head for general iterators: the check comes
before each call of `next` (`lib.rs:355–356`).

**Fix** (04 and 01 only, no core tokens). In CLAUSE-6, write: "A `for`
loop's head comes before each call of its iterator's `next`. Over `a..b`
the invariant sees the loop variable as the next index (§2.4). A `loop`'s
head is the top of its body. An exit by `break` or `return` is not a head."
Change "the condition fails" to "the loop ends" in CLAUSE-6 and in 01 D10.

### N2. "Building a value never reports a violation" says too much

`04:589–591`: "it evaluates each call's `requires` first and skips a call
that fails it, so building a value never reports a violation." Skipping
avoids only `requires` violations. A constructor's `ensures`, or a v1 type
invariant, can still fail during generation, and it should be reported:
CLAUSE-6 says "nothing else decides whether a clause runs" (`04:470–471`).
Read literally, the sentence lets an implementation swallow those failures.

**Fix.** "…so building a value never reports a `requires` violation."

### N3. CLAUSE-5 does not say that a field read is `checked`, so G11's motivating case stays open

The bead and grading Q14 (`trials/e1-dialect/grading.md:266`) give G11's
consequence as whether `self.len == old(self.len) + 1` is `checked` or has to
go through `len()` and be `sampled`. CLAUSE-5's checked list (`04:451–454`)
names operators, literals, `result`, `old` of `Copy` values, and std
accessors. It does not name parameter or field reads. The reference's list
is the same (`reference.md:877–880`). Reading a field as a leaf, as a
parameter is read, is the natural interpretation. Still, this change makes
private fields legal in clauses so that ring's clauses can be `checked`, and
the text never says that they are.

**Fix.** In CLAUSE-5, write "operators on std types, literals, parameters
and their field reads, `result`, …". Changing the reference to match would
cost about 4 tokens and can go with B1's trade.

### N4. 02 still says private field names never enter a public API

`02-memory.md:586–587`: "A method visible outside its package may name only
fields that are also visible outside it, so private field names never enter
a public API." Under G11, a `pub` method's contract may name private fields
(`04:397–399`). Contracts are part of the interface artifact (`01:738–740`,
`04:1060`) and of the documentation. The rule in 02 concerns field-borrow
sets and still holds. Its "so" clause states something broader that is no
longer true.

**Fix.** "…so private field names never enter a public signature through a
field-borrow set." Contracts are the one exception, and it follows from
CLAUSE-2.

### N5. Reference 10.2's provisional marker now also covers the decided skip rule

The new parenthetical "(skipping any whose `requires` fails)"
(`reference.md:938`) is followed by "(Instantiation, skipping, and `allow`
are provisional.)" (`:943–944`). That "skipping" meant "skipped as
unchecked" (`:941–942`). Now it can be read as covering G10's skip rule
too, which fx-eky's acceptance says should no longer be marked provisional.

**Fix**, at no token cost: write "(dropping any whose `requires` fails)" at
`:938`, or "skipping as unchecked" at `:943`.

### N6. Nits

- **Line length.** `01:610` is 137 characters. The line it replaced was
  also too long, so rewrap it while editing.
- **CLAUSE-6 wording.** "A loop head is reached before each evaluation of a
  `while` or `while let` condition, the last time before the evaluation
  that ends the loop, which is the exit check" (`04:461–463`) is hard to
  parse. Better: "…condition; the last such head, before the evaluation
  that ends the loop, is the exit check, and a side effect of that
  evaluation is not checked."
- **Sampled invariants in the reference.** The reference row says "every
  iteration", but `tide test` evaluates a `sampled` invariant on iterations
  1–64, 128, 256, and so on (`04:569–572`). The old row omitted this too,
  so it is not new. Mention it only if B1's trade frees the tokens.

## Ready to merge

**No.** B1 is a contradiction between the reference and 01 and 04 on the
rule this change introduces. Fixing it means trading core tokens, since the
core has 2 left. B2 costs no core tokens. The N items are cheap, and only
N3 and N5 touch the core.

Validation: this was a read-only review of the working tree at `557d4e5`,
with the four `design/` files modified, on 2026-10-20.

- **Read:**
  - the full diff;
  - 01 D10 and D18's contract bullets;
  - in 04: §1.2 rule 10, §1.4, all of §1.8, §2.3, §2.4, §2.6, and §8.1;
  - reference 10.1 and 10.2, and companion 15.3;
  - 02's API stability rules;
  - the spike's `check_lowered` and `Loops` rewrite;
  - grading.md's C5, C6, and Q14–Q16;
  - fx-eky with its comments, via `br show`.
- **Leftovers:** grepped `design/`, `README.md`, and `GOVERNANCE.md` for
  "loop entry", "at exit", "entry and exit", "after every iteration",
  "private-in-public", "provisional", and "violated".
- **Count:** `scripts/count-tokens.sh design/reference.md </dev/null` was
  run once, with a 400 s timeout, on the working tree. It printed `24998`
  and exited 0.

No Tide checker exists, and the spike was read but not run, so every verdict
above is an inference from the text. Only this file was written; nothing was
staged or committed.

## Resolution

Answered in the commit that adds this review.

- **B1:** reference 10.1's "When clauses run" says debug builds check a
  `for` over `a..b` only at entry and exit. The Visibility bullet folds
  into "Clause expressions", and 10.2's vacuity bullet drops its sorted-`Vec`
  aside, to pay for it. The core measures 24,996 tokens.
- **B2:** CLAUSE-7 lists where the spike differs: it checks every head of a
  `for` over `a..b` in debug builds, and its message lacks the blamed span
  and prints tokens rather than source text.
- **N1:** CLAUSE-6 and 01 D10 put a `for` loop's head before each call to
  its iterator's `next` (the next-index view only over `a..b`), explain the
  debug rule by "its condition or iterator ends it", and say a loop left by
  `break`, `return`, or `?` gets no exit check. `loop {}` stays as 04 §2.4
  already leaves it.
- **N2:** "never reports a `requires` violation (a failing `ensures` is
  still reported)".
- **N3:** CLAUSE-5 lists field reads as `checked`, and the reference's list
  adds "fields".
- **N4:** 02 §7's API-stability bullet says a declaration never puts private
  field names in a public API, and that a v0 clause may still read them.
- **N5:** reference 10.2 says "calls that meet their `requires`", so
  "skipping" in the closing note refers only to skipped functions.
- **N6:** 01's long line is rewrapped and CLAUSE-6's exit-check sentence is
  split. The sampled-invariant schedule stays out of the core for budget.
