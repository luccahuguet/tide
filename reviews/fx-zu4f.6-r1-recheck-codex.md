Model: gpt-6.1-sol (high) · Date: 2026-11-18 · Re-check 11f2782

## Verdict

**changes required**. The four original findings and the filter-label clarification are addressed. Two factual defects were introduced in the touched prose. Neither changes the corrected eligibility outcomes or requires a search/model run.

## Scope and identity

Re-checked only the r1 findings, their fixes, the five affected assessments and rows, counts, and derived prose. Read `git diff 99b827c 11f2782`, the builder's “Round r1 fixes” account as claims, the four added evidence files, the relevant saved issue threads/timeline, and the registered E1–E4 rules. No frozen search was rerun; no GitHub write or new GitHub request was made in this re-check.

- HEAD: `11f2782a8861485266a7bea2aa51006f58317f9f`.
- Round-diff SHA-256 (`git diff 99b827c 11f2782`): `faeb530dc1603a205a21508e7a4dc2a0fd13b24f649def0719a4da9ac4df43b7`.
- Base-to-candidate diff SHA-256 (`git diff 7863e8f..11f2782`): `577b4f7ba5c809864306f8e4b05188ddafff2ecb7d8c231411838914fa68bd6e`.
- Research file SHA-256: `62e5eeacf9a9fbf24c1d722b52e46c918813cdb3c7ed1e9e0512386b5110652f`.
- The embedded protocol remains byte-identical to the registered file, SHA-256 `0a02f5be72ea01ca887498fc5d938a35cfc9c25d1de3fa728a89ca5a46576308`.

## Original findings' status

1. **A15: fixed classification; new rationale defect below.** `research/38-external-concurrency-search.md:171` now accepts the named peer-close/shutdown race. Saved `issues/tokio-rs__tokio__8520/pull-8526-files.json` contains the fix's matching race explanation and async socket tests. E4 is an appropriate first failure because that socket/async path has no permitted faithful mapping. Q7.4 now says E4. The added “external crate” claim is separately incorrect (B1).

2. **A19 and A29: fixed.** At `research/38-external-concurrency-search.md:187`, A19 records the pre-fix comment's observed 755-to-600 permission output, then the bind-to-chmod race; its socket/permissions path fails E4 and Q7.8 agrees. At line 224, A29 quotes the pre-fix report of immediate `readable()` returns after `splice`. Its subsequent E3 failure is supported: the stale cache in the reported single-task loop is the symptom, whereas the maintainer's lost-readiness race concerns the proposed manual-clearing workaround. Q7.18 correctly says E3. This distinction does not invent another eligibility restriction. A19's added external-crate claim is covered by B1.

3. **A26: fixed.** `research/38-external-concurrency-search.md:217` now fails E3 rather than E4. Saved `issues/tokio-rs__tokio__7881/pull-7882-files.json` shows the successive `write()`/`remaining_wait()` checks and elapsed-time window within one poll. This agrees with the previously inspected `action()` implementation and the registered two-concurrent-activities requirement. Q7.15 agrees; E4 is no longer assessed.

4. **A22: fixed; E1 failure is correct under the registered direct-link rule.** `research/38-external-concurrency-search.md:195` distinguishes the actual starvation fix from the later correction. Saved issue #8034's body/comments do not link or name #13085, its closed timeline event has no commit ID, and its direct cross-references include issue #13040 and PRs #8036, #13511, and #8305, not #13085. The added #13085 response confirms that it fixes #13040. Following #13040 to that fix does not satisfy the registered direct-link condition for #8034. The saved #13511 response fixes a later #13085 regression and closes #13507; its presence in #8034's timeline does not make it that original bug's named fix. #8305 closes #8034 administratively after the upstream fix and re-enables its test; it does not implement the bug fix. Thus none supplies a qualifying direct bug-fix link under E1. Q7.11 correctly changes to E1, and no rule is widened.

5. **Non-blocking filter label: fixed.** Lines 43–44 now distinguish the 125 Rust issues below 1,000 stars from the one non-Rust issue also below the threshold. These are disjoint buckets totaling 126.

## New blocking defects in the fixes

- **B1 — Tokio is not external to these Tokio-repository paths.** `research/38-external-concurrency-search.md:172` and `research/38-external-concurrency-search.md:189` newly call Tokio an external crate. The registered protocol, lines 168–169, expressly exempts crates of the same repository/workspace. Both assessed issues and their affected Tokio implementations belong to `tokio-rs/tokio`. The new rationale therefore contradicts that exemption, even though other valid E4 grounds remain. **Minimum fix:** remove the external-crate assertion from both entries and rely on the already evidenced unsupported socket/async operations and, for A19, permissions. Keep their E4 outcomes and all counts. Align the builder account if retained as the fix report.

- **B2 — Incorrect count of added evidence requests.** `research/38-external-concurrency-search.md:96` says “three more read-only GETs,” then lists four: #8526 files, #7882 files, #13085, and #8305. There are four corresponding saved response files, and the builder account also says four. **Minimum fix:** replace “three” with “four.” No new request or check is needed.

## Touched counts and derived prose

Local assertions confirmed that only A15, A19, A22, A26, and A29 changed, each affected row agrees with its assessment, and the resulting first-failure counts are E1 10, E2 1, E3 6, E4 15, totaling 32. Both summary representations agree. The 15 E4s still split into nine Tokio-organization issues (one Mio) and six application issues; adding “OS interfaces” and “sockets” accurately describes the changed membership. The 160 pool rows retain their identities and metadata. The negative result and Eon-only demonstration framing remain consistent with these corrections.

`git diff --check 99b827c 11f2782` passed. No implementation, runtime test, upstream reproduction, search, or broader review was repeated. Only this requested re-check report was written; the candidate and `.beads/` were left unchanged.

DONE
