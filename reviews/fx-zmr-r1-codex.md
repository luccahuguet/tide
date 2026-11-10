Model: gpt-6.1-sol · Effort: xhigh · Date: 2026-11-09

## Verdict

request_changes — high confidence (0.96). The research update covers the five requested finds, but three factual overstatements need small text corrections.

## Scope

Reviewed `git diff 0583ecb 23b37e3`, candidate `23b37e3b93737bfa8a23a4f6150638149e085553`, in `/home/lucca/pjs/lab/tide/.worktrees/fx-zmr`. The diff changes only `REFERENCES.md`, `research/03-compile-speed.md`, and `research/13-agent-fleets.md` (+179/-6). Its SHA-256 is `b1bd3c9f464b3f1081f9aa65d00125df71e93d116ae5798f93d50b8f4324bf03`.

Read the touched files in full, AGENTS.md, GOVERNANCE.md, the bead through `br --no-db show fx-zmr`, the supplied implementation report, and the relevant tier, D0, D12, D13, and D18 sections of `design/01-design.md`. Searched the repository for overlapping claims. Tracked candidate files matched the commit; the existing untracked `ZMR-REPORT.md` was outside the candidate. No repository or bead writes were made.

## Blocking

### B1. The Cargo reuse argument misplaces the proposed shared cache

**Location:** `research/03-compile-speed.md:126`; repeated in `REFERENCES.md:76`.

The paragraph says Cargo's cross-workspace cache lives inside Cargo's build directory and uses that as a reason Tide cannot reuse it. The cited project status describes a shared content-addressed store serving multiple workspace build directories. The linked design explicitly distinguishes `<build-cache>/content/` and cache manifests from the `build-dir` where compilation happens. This is the shared-cache proposal, not merely the existing workspace artifact directory. [September status](https://github.com/rust-lang/goals/issues/626#issuecomment-5760036786), [proposed storage layout](https://github.com/rust-lang/cargo/issues/5931#issuecomment-5498506395).

The practical conclusion can remain: D18 owns the driver and admission policy, the initial Cargo design excludes path crates, and the recorded cache work is unfinished. The incorrect location claim unnecessarily rules out storage reuse on a false premise.

**Smallest fix:** Remove the claim that the shared cache lives inside `build-dir` from both locations. Describe it as Cargo-integrated shared storage whose direct reuse by Tide has not been established; retain the supported scope and admission-policy objections. No design change is needed.

### B2. Kache already budgets test processes with compiler processes

**Location:** `research/13-agent-fleets.md:369`–`370`.

The statement that neither tool budgets test and simulation processes is false for kache's tests. Its released v2.0.0 documentation describes `kache test-runner`, which takes test slots from the compiler scheduler's pool and adjusts test thread counts. Unix `kache init` enables the runner unless shell setup is disabled. This capability exists in the very version discussed by the candidate, so this is not a later development. [Kache v2.0.0 configuration, lines 190–235](https://github.com/kunobi-ninja/kache/blob/v2.0.0/docs/getting-started/configuration.mdx#L190-L235).

A reader specifying Tide's fleet budget would otherwise miss existing prior art for coordinating tests with compilation.

**Smallest fix:** Replace the negative claim with the documented kache test-runner mechanism and cite it. Leave Tide simulation scheduling as an open implication; do not adopt a scheduling design here.

### B3. Embedded checkout paths do not preclude cross-worktree cache hits

**Location:** `research/03-compile-speed.md:125`.

The opening sentence says cross-worktree hits are possible only if rustc output does not embed the checkout path. That confuses reusing an existing artifact with independently rebuilding identical artifact bytes.

Mbx's cited limits explicitly distinguish those cases. It restores a cacheable workspace crate byte for byte, including its original embedded paths. The dependent invalidation problem occurs when a checkout-specific key forces that crate to compile again and the new output bytes differ. Thus checkout B can restore checkout A's artifact despite its embedded A paths; path normalization is not a prerequisite for that hit. [Mbx's workspace-path limitation](https://github.com/jdx/mr-boxington/blob/main/docs/limits.md#L497-L517), [restored-artifact behavior](https://mr-boxington.jdx.dev/limits#restored-artifacts-are-equivalent-not-always-identical).

**Smallest fix:** Say that normalized action keys permit cross-worktree restores, while normalizing embedded output paths can preserve dependent cache hits when an upstream crate must be rebuilt independently in another checkout. Keep remapping or a fixed compilation root as recommendations, with their stated debugging costs.

## Non-blocking

None. No wording preferences or unrelated cleanup are requested.

## Reasoning

The coverage part of Acceptance is met: mbx, kache, Cargo's cross-workspace cache, and Wild have cited entries in research/03; mbx's fleet scheduling and ovnfs have entries in research/13; all five have REFERENCES rows. The diff contains no `design/` changes. The three findings above prevent an approval of the evidence as written.

Primary-source spot checks support the main recorded measurements:

- Mbx's published table matches the aube 1m7s/172-compilers versus 54.7s/31-compilers comparison, its hit counts, and the hk comparison. The note correctly identifies the scheduler comparison as mbx in every row and retains the benchmark's workload and variation limits. [Mbx benchmarks](https://mr-boxington.jdx.dev/benchmarks).
- Kache's vendor report supports the Firefox/APFS storage figures and the 10.75x/98.5% warm-build figures. These are appropriately identified as one self-reported run. Direct page fetching failed, but the search service returned the primary article's indexed text, including those figures. [Kache storage report](https://www.kunobi.ninja/blog/kache-storage-worktrees).
- Cargo's goal and status updates support the initial eligibility limits, layout rollback/restabilization history, and the October design/prototype status. Those sources do not support B1's cache-location assertion. [Cargo goal](https://goals.rust-lang.org/2026/cargo-cross-workspace-cache.html), [October update](https://github.com/rust-lang/goals/issues/626#issuecomment-6162920449).
- Wild's README confirms that incremental linking is unimplemented. The three cited SVG charts support the approximate times and percentage labels; the note correctly distinguishes link time from whole-build time. [Wild README](https://github.com/wild-linker/wild), [benchmark page and charts](https://github.com/wild-linker/wild/blob/main/benchmarks/ryzen-9955hx.md).
- Ovnfs's README and limitations support its platform-specific frontends and snapshot/sandbox caveats. The oh-my-pi issue and unmerged removal PR support the corrected backend history; the pinned 18.17.0 changelog confirms the conditional ovnfs integration. The 500/500 reproduction remains clearly attributed to its reporter. [Ovnfs limitations](https://github.com/stencil-hq/ovnfs/blob/main/docs/limitations.md), [issue #4627](https://github.com/can1357/oh-my-pi/issues/4627), [PR #4629](https://github.com/can1357/oh-my-pi/pull/4629), [18.17.0 changelog](https://github.com/can1357/oh-my-pi/blob/v18.17.0/packages/coding-agent/CHANGELOG.md).

The Fast Builds roadmap and linked Cranelift goal give no reason to change the existing Cranelift figures. [Roadmap](https://goals.rust-lang.org/2026/roadmap-fast-builds.html), [Cranelift goal](https://goals.rust-lang.org/2026/improve-cg_clif-performance.html).

The new inference blocks read as research recommendations, not adopted language decisions. D12/D18 still own the backend and driver cache; no scheduler, linker dependency, checkout service, or new E4 gate is adopted. Older descriptions of the pre-D18 backend and older fleet gaps already existed in the base and are not regressions from these additions.

No builds, installations, benchmark reproductions, or external project code were run. Published performance remains self-reported; the untested FUSE restore inference is explicitly labeled. Only the requested report was written.

DONE
