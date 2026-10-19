# Tide consistency review, round 15: confirmation of the round-14 answers

## Verdict

**approve**

Reviewed commit `353b51672014d1703c6bc34ccf2ba13a059bbdfd` (`353b516`, "Add
round 14 and answer it: body-independent parameter snapshots"), dated
2026-10-19 18:22 -0300. It was diffed against `9d1ac8b`. Scope, as asked:

- whether round 14's B1 is closed: 04 CLAUSE-4 against reference §10.1, 01
  D10, and 04 CLAUSE-5, and whether the D18 attribute can lower the rule;
- whether the other changed lines (02 §2.7, 01 D10's bare-`*x` sentence)
  contradict 01, 02, 03, 04, or the reference.

The rest of the design was not re-audited; rounds 12–14 did that.

## Round-14 item status

| Item | Status | Evidence |
| --- | --- | --- |
| B1 | **Resolved** | See below. |
| N1 | **Resolved** by B1's main fix | The macro decides the snapshot alone; no driver-fed capture set is needed. |
| N2 | **Resolved** | `reference.md:1174` adds "through a `&mut` it holds included". |
| N3 | **Open, as recorded** | Settle before the checker's test corpus. |
| N4 | **Resolved** for `02:267` and 01 D10; README qualifier **open, as recorded** | `02-memory.md:267` lists `'static` instead of `Owned`. 01 D10's wording is narrower than 04 rule 4; see N2 below. |

**B1 holds on all four counts.**

- **CLAUSE-4 and the reference agree.** In both texts (`04-contracts.md:422–429`,
  `reference.md:1171–1175`):
  - a `Copy` parameter is copied;
  - a `Clone` one is cloned at entry, whatever the body does;
  - a parameter that is neither is an error only when the body assigns,
    moves, or mutably borrows it, through a held `&mut` included;
  - the snapshot does not change the clause's class.
- **The class depends only on the clause text.** CLAUSE-5
  (`04:430–437`) and the reference's "When clauses run"
  (`reference.md:1195–1198`) both name only explicit `old` of `Copy`
  values. 01 D10 (`01:629–632`) classes clauses by their operators,
  literals, and accessors. None of the three counts implicit snapshots. Round 14's example
  `#[ensures(result.len() == v.len())] fn sorted(mut v: Vec<i64>)` is
  `checked` under all three texts, whatever the body.
- **Entry values do not depend on the body.** The body matters only for
  the non-`Clone` error, which is the intended exception.
- **The D18 attribute can lower the rule.** The rule is the candidate that
  round 14 compiled. It was re-checked here (Validation).

## New blocking problems

None.

## Non-blocking notes

### N1. "The `Clone` restriction" points at the wrong rule, and CLAUSE-3 may not cover implicit clones

`04-contracts.md:427–429` says implicit snapshots "follow this item's rules
for `old` (the `Clone` restriction, panics as violations)". Item 4's only
`Clone` restriction makes `old` of a type that is neither `Copy` nor `Clone`
an error (`04:413–414`). The sentence before it relaxes that rule for
implicit snapshots: such a type is an error only if the body mutates it.
Specific-over-general resolves the clash, but the parenthetical names a rule
that does not apply.

Round 14 proposed "CLAUSE-3's `Clone` restriction". That wording points at
the rule that matters: item 3 (`04:395–396`) makes the `Clone` impls that
`old` calls authority-free with `Pure + Sync` arguments. Whether item 3
covers implicit clones is now unstated, and the gap is observable:

- `#[ensures(result > s.n)] fn field(s: S) -> u32`, with
  `#[derive(Clone)] struct S { n: u32, r: Rc<u8> }`, passes `s` to no
  function, so item 3's argument check never sees `S`.
- The implicit snapshot still runs `Rc::clone` on a non-`Sync` value.
  Validation compiled this case.

**Fix.**
- In `04:428`, replace "(the `Clone` restriction, panics as violations)"
  with "(item 3's limits on the `Clone` impls they call, and panics as
  violations)".
- In item 3 (`04:395–396`), say "the `Clone` impls that `old` and item 4's
  entry snapshots call".

The lowering then asserts `Pure + Sync` on each type it clones. Item 3
could also let a non-`Pure + Sync` value fall back to a borrow, which
would make the non-`Clone` error apply. The rule should choose one of
these before the checker's test corpus is written.

### N2. 01 D10 limits the bare-`*x` error to `prove`, but 04 rule 4 applies it to every use in v1

- **01 D10** (`01-design.md:605–607`) now says "Under `prove` (v1), a bare
  `*x` … is an error".
- **04 §1.2 rule 4** (`04-contracts.md:189–192`) says "From v1, a bare `*x`
  is an error", with no use restriction. It also says `tide fix` rewrites
  "a v0 clause", in any use, to `final(*x)`.

For a v1 function under `assert` or `test`, 04 makes a bare `*x` an error,
while 01 implies it is not. The resolution cites 04 §1.3. §1.3 is `prove`'s
spec language, but it does not state this rule. This is a v1 matter and
does not block v0.

**Fix.** Use one tier mark in both places. "From v1" in 01 matches 04 rule 4
and the `tide fix` migration. If the error is meant for `prove` only,
narrow 04 rule 4 and its `tide fix` sentence instead.

### N3. Nits

- **Generic code.** In generic code, "if it is `Clone`" means "bounded
  `Clone` at the definition", because autoref specialization resolves
  against the definition's bounds (D4). So `fn g<T>(mut v: Vec<T>)` with
  `v.push(..)` and `v` in an `ensures` is the non-`Clone` error, and its fix
  can also add `T: Clone`. CLAUSE-4 could say both in a few words.
- **Line length.** `02-memory.md:266–268` were not reflowed after the edit.

## Ready to implement v0

**Yes.** Round 14's B1 is closed. The rule matches CLAUSE-5, the reference,
and 01 D10, and the D18 attribute can lower it with no type or body
information in the macro. rustc's borrow checker rejects every non-`Clone`
case.

Before the checker's test corpus is written, settle:
- N1's choice of how item 3 treats implicit clones;
- round 14's N3 (reassigned `&mut` parameters);
- the round-12 and round-13 notes still open.

N2 is v1.

Validation: this was a read-only review of `353b516` (diffed against
`9d1ac8b`) on 2026-10-19. Programs were compiled with rustc 1.96.0
(ac68faa20 2026-05-25), edition 2024, with stdin closed, in a session
scratchpad. They used round 14's autoref-specialization lowering
(`let __p_x = (&&&Probe(&x)).snap();`, body as `(|| body)()`, clause
reading `*__p_x`).

- **Accepted cases.** Each compiled and its clause held:
  - `sorted(mut v: Vec<i64>)` with `v.sort()` (cloned);
  - `gpush<T: Clone>(mut v: Vec<T>, t: T)` (cloned);
  - `field(s: S)` with an `Rc` field and a clause reading only `s.n`
    (cloned; this is N1).

  Output: `sorted 3`, `gpush 2`, `field 2`.
- **Rejected case.** `gpush<T>(mut v: Vec<T>, t: T)` without a `Clone`
  bound fell back to the borrow and was rejected with `E0502`.

No Tide checker exists, so every Tide verdict above is an inference from the
text. Only this file was written; nothing was staged or committed.
