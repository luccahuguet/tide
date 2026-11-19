Model: Claude Opus 5.5 · Date: 2026-11-19 · Candidate e3f708a (S5 re-check)

Scope: B1, B2 and the fixes the author took for NB1, NB2, NB3, NB4, NB6, NB7 and NB9. Inputs: `git diff 8d7181c e3f708a` (6 files) and report.md "S5 r2".

## B1: RESOLVED

- **Status list:** README.md:448-451 now lists "`Proc` for child processes (`command`, `only`), `Rand` draws".
- **Dropping "declared" is correct.** My suggested wording said "declared child processes". On the real backend, `Proc` starts any program unless `only` narrows it. Only the simulator's processes are declared models (README.md:489 says "child processes as declared `sim.world.proc` models").
- **At-a-glance item 3** now names randomness and `&Rand`, in docs/at-a-glance.md and in its README copy. `python3 -I` confirms the README block is still verbatim. The new wording is correct on two counts:
  - **`&Rand` is the real parameter form.** conformance/cases/authority/rand.rs:10 and :20 take `rand: &Rand`, and :71 passes `&root.rand`. root.rs:244 declares `pub rand: rand::Rand`.
  - **No free function reaches randomness.** design/reference.md:441-443 says `HashMap::new()` uses a fixed key, and seeded hashing goes through `RandomState::from_rand(&rand)`. So item 3's "no free function ... that reaches them" still holds with randomness added.
- **Agreement:** the texts now agree with "Not in this alpha" (README.md:470-473), answers item 10 and the paper's abstract.

## B2: RESOLVED

- **The fix:** docs/owner-guide/03-soundness-bugs.md:48-51 now carries the prescribed sentence word for word: 48 routes, controls included, "found that admission let through two forms of one more known route, rust#57893 (open upstream since 2019, both forms already on record there)".
- **The lead-in:** it now reads "**fx-qlek.5** landed 2026-11-17.", which reads cleanly.
- **The paragraph's tail is consistent.** It goes on, unchanged, to "Every other escape route the record lists was rejected or denied. Two routes stay admitted on purpose", which still agrees with answers.md:28-32 and the README.

## The fixes the author took

- **NB1: no defect.** README.md:163-168 is split into "The property: …" and "The mechanism: …". Every qualifier in the 8d7181c sentence is kept:
  - safe code;
  - apart from printing;
  - outside the `unsafe` grant, with `--tide-grant=unsafe`;
  - up to rustc's soundness bugs.

  It reads clearly.
- **NB2: no defect.** At paper/tide-v0.typ:213 the definition now follows "_accidental_" directly. The scope is unchanged (outside declared trust holes such as the `unsafe` grant), so Codex's B2 scoping is preserved.
- **NB3: no defect.** docs/tide-or-rust.md:83-85 now uses item 9's wording ("That re-test's pool, one crate the screen had already seen, was chosen afterwards, so its outcome was close to predictable").
- **NB4: no defect.** The Figure 14 caption at paper/tide-v0.typ:414 now says "selected items of the v0 core, those not built yet marked planned, then selected items of v1 and later". That is true of the figure's contents and of README.md's longer v0-core list. No figure record changed, and build.mjs --check passes.
- **NB6: no defect.**
  - docs/owner-guide/09-experiments.md:192 now says "One re-test counts one catch", which matches item 9.
  - :211-212 now gives the reason, "so that the documented idiom compiles in a task body", worded as at README.md:203-204.
- **NB7: no defect.** At paper/tide-v0.typ:431 the clause "an order our own records show" now agrees with the §4.5 self-attested clause.
- **NB9: no defect.** README.md:562-564 is rewrapped with its text unchanged. The Status line was also rewrapped, under B1.

I found no new inconsistency across the README, at-a-glance, tide-or-rust, the owner's guide chapters 3 and 9, and the paper.

## Checks run on e3f708a

- `git diff --check 8d7181c e3f708a`: exit 0.
- `node paper/figures/build.mjs --check`: "figures match their records (22 files)".
- `typst compile --root . paper/tide-v0.typ /mnt/build/tide-agents/fx-qlek.7-target/s5o/tide-v0.pdf`: exit 0, no warnings.
- The README's at-a-glance block is verbatim docs/at-a-glance.md (python3 -I).
- The tracked worktree is clean.

NB5 (GOVERNANCE.md, single-holder) stays open, as recorded under Left open. It is outside this re-check.

## Verdict

approve. B1 and B2 are resolved, and the fixes the author took introduce no defect or new inconsistency.

DONE
