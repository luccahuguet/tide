Model/Effort/Date: claude-opus-5-5, low, 2026-10-31

## Verdict

request_changes (high confidence)

## Scope

Narrow confirmation of the fix for reviews/fx-gfq-r4-codex.md blocking finding 1, in worktree
/home/lucca/.cache/eon-agents/tide-gfq at HEAD 1e0cbac (working-tree diff SHA-256
78969cf422abdb538ed73af5398bcacf25d1987ca53fe8d5676431f92eab5de9, verified). Texts checked:
trials/e1-checked/protocol.md §10.1 row E1-P14 (line 260), §10.2 readiness item 3 (lines 295-299),
and bead fx-u65 Scope and Acceptance. Checked against design/reference.md (every FX code it names
and every "lint" mention), conformance/rules.md, and conformance/README.md. Nothing else reviewed.

## Blocking

1. trials/e1-checked/protocol.md:260 and fx-u65 Scope ("report the lint-level ones (FX0508,
   FX0510, FX0607, FX0608)"). design/reference.md never names FX0510. Its only lint-level codes are
   FX0508 (reference.md:460, 979; also the unnumbered "a lint in libraries" for the print family at
   :53), FX0607 (:736), and FX0608 (:690). FX0510 is the least-authority lint, which
   design/03-authority-concurrency.md §4.2 marks "(v1)" and design/01-design.md:1015 describes.
   Risk: both texts misstate the reference's lint set. fx-u65's Scope then tells the implementer to
   build a v1 lint as an E1 prerequisite that blocks fx-ic2, beyond what the reference teaches the
   Tide arm. It also contradicts its own defining clause and Acceptance ("every FX code that
   design/reference.md names"), so whether FX0510 is required is ambiguous. Smallest fix: delete
   `FX0510` from the parenthesized lint list in protocol.md:260 and in fx-u65's Scope. Readiness
   item 3 lists no codes and needs no change.

## Non-blocking

1. fx-u65 Acceptance asks that a conformance case for each lint code get that code "reported
   without rejection", and that run.sh show no AWAITS CHECKER line for those codes. Today marks
   record errors only (conformance/rules.md:169: FX0608 is "a warning, which a case cannot mark";
   conformance/README.md:34-37, 106). So the lint half needs a harness extension (a lint mark or
   similar), and the AWAITS CHECKER clause is vacuous for lints, which can never produce that line
   now. This is delivery work the implementer owns. One sentence in fx-u65's Scope noting that the
   mark syntax must learn lints would save the implementer from rediscovering it.

## Reasoning

The r4 finding is otherwise resolved. All three texts now require each code "at the reference's
severity", rejecting errors and reporting lints, and readiness item 3 adds "lint-level code is not
rejected". The example error codes are correct: FX0301 (lossy `as`, reference.md:261), FX0702
(ignored `Result`, :803), and FX0913 (unused imports, :985) are all errors. The texts agree with
each other on mechanism. Readiness item 3 cites "E1-P14's conformance cases", which matches fx-u65's
Acceptance. The one defect is the added FX0510. Codex's finding named only FX0508, FX0607, and
FX0608, and the reference's own lint mentions confirm that this set is exactly those three.
