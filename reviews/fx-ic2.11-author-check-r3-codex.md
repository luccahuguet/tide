# fx-ic2.11 — Codex scheduling re-check, round 3

Model: gpt-6.1-sol. Reasoning effort: high. Same read-only reviewer.
Candidate: 7477a0ea4aa3bc193975e80bf76d7681e4c55d25.
Verdict: technical review clean — GO to Opus. Confidence: high. No override.

Author response: start at 16:30–16:31Z after the unchanged standing-slot check;
use a 14-minute preflight envelope, 240-second preflight guard, 600-second
regrade-attempt cap and END−600 late-grant guard. Explicitly preserve incomplete
status and require a fresh booking if the capped attempt does not finish.

Reviewer result: the scheduling finding is resolved. The allowed start clears
the quiet slot, and the envelope and guard fit the booking. Helper, frozen
harness, sandbox/test limits and match criteria remain unchanged. Bash syntax
and diff checks pass. This changes executable scheduling instructions, so is
not text-only. No broader behavior or trust-boundary change was identified.

No builds or grading. Publication acceptance still requires all sampled matches
and the remaining governance review; the technical candidate is clear to review.
