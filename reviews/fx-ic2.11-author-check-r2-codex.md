# fx-ic2.11 — Codex scheduling re-check, round 2

Model: gpt-6.1-sol. Reasoning effort: high. Same read-only reviewer.
Candidate: c3b8203863910da9c6a63362c4996d91293c998d.
Verdict: request_changes. Confidence: high. No override.

The author shortened the preflight envelope to 40 minutes and total regrade
backstop to 1800 seconds, retaining grade/test limits and all match criteria.

The reviewer found another constraint in the archived preflight: its standing
15:00Z quiet slot unconditionally blocks until 16:30Z (preflight.sh:35–38).
QUIET OVER and the booking exemption do not override that branch. The new
END−1800 guard requires a grant by 16:15Z. These conditions do not overlap.
Align booking and exact command with the preflight's allowed interval; do not
alter the frozen harness or relax required proof.

No code/archive change, compilation or grading. Publication proof stays pending.
