Model: gpt-6.1-sol · Effort: xhigh · Date: 2026-11-03

**Verdict: request_changes · Confidence: high (90%)**

**Scope:** Verified the candidate’s diff SHA-256. Read the Merge Rule, repository-local rules, bead and comments, all eight changed-file diffs, relevant rules in 01/02/03, companion tables, conformance format and runner, and the affected admission policy and walk. Did not read the parallel review file.

**Blocking findings**

1. **Granted assembly remains excluded by the core’s closed macro list.** [design/reference.md:61](/home/lucca/pjs/lab/tide/.worktrees/fx-k0z/design/reference.md:61)

   §2.1 says std supplies “exactly” the listed forms, then assigns FX0103 to every other form. The amendment gives **ungranted** `asm!` and `global_asm!` FX0520, but never admits their granted forms. Consequently, `unsafe { core::arch::asm!("") }` in a granted package still falls under the core’s FX0103 catch-all, contrary to 03 ADMIT-1, `policy.rs:86–87`, and `unsafe_granted.rs`.

   **Concrete risk:** An agent or implementation using the core reference rejects valid granted assembly. Bead item 1 therefore remains partly unresolved.

   **Smallest patch:** Before the catch-all, explicitly state that `asm!` and `global_asm!` are also admitted under the package’s `unsafe` grant. Retain the ungranted FX0520 ruling; trim elsewhere and recount to preserve the budget.

**Non-blocking improvements**

- [design/reference-companion.md:77](/home/lucca/pjs/lab/tide/.worktrees/fx-k0z/design/reference-companion.md:77): Add an elapsed-time replacement row. §6.9 directs readers here for capability-based fixes, but the table currently covers only `now()`.

**Reasoning**

The remaining alignment work is sound: `Root::sim` matches ROOT-1/ROOT-3; the hole-table diagnostics match their owners; retaining the hardware-intrinsic restriction agrees with the driver; pointer casts encounter the raw-pointer admission check; and the structural `Pure` detail remains owned by 02 PURE-1.

The elapsed cases follow the corpus format and distinguish capability calls from forbidden ambient methods. Their FX0501 marks correctly remain **awaiting the checker**, rather than constituting implemented diagnostic proof. The reported conformance exit 0 and 24,998-token count satisfy those acceptance checks for this candidate. `git diff --check` also passes.

Not rerun: conformance or token counting; those results rely on the supplied candidate evidence. Runtime clock behavior remains untested by these compile-time fixtures.