Model: gpt-6.2-sol (xhigh) · Date: 2026-11-19 · Re-check bundle -> Tide 801da4e, site cac0ca3

## Verdict

Clean. B1, B2 and C1 are resolved. Both follow-up wording edits are correct, and the amended site pin matches Tide `801da4e`.

## Findings

- **B1 resolved** (`docs/at-a-glance.md:12`–`:17`, `README.md:18`–`:23`). Item 3 uses “no accidental ambient authority” and limits it to admitted safe code outside the `unsafe` grant, apart from printing and up to rustc's soundness bugs. All six capability types and the forge-routes link are present.
- **B2 resolved** (`docs/at-a-glance.md:18`–`:22`, `README.md:24`–`:28`). Item 4 describes `only` as limiting outbound peers and `no_bind()` as removing permission to listen from the narrowed capability. This matches `crates/tide-std/src/net.rs:159` and `:182`.
- **C1 resolved** (`src/pages/questions.astro:138`–`:143`). The heading asks what to do if a service uses async. The answer describes Tide's deliberate choice of structured tasks and gives Tokio as an example of when to use plain Rust. Both adoption claims are gone from the rendered FAQ.
- **Trust diff wording resolved** (`docs/at-a-glance.md:27`–`:31`, `README.md:33`–`:37`). Item 6 says “declared authority” and retains the E3 negative result.
- **WASM opening resolved** (`src/pages/questions.astro:72`–`:80`). The recommendation specifies a sandboxed component with narrowly scoped host imports. The static-check distinction and Tide target limit remain intact.

## Re-pin and regressions

The README mirror is byte-identical to the glance source. The site constant, generated glance links, status and chart pins agree on `801da4ea1e5edeb6c4a1b2ceab956638f1770aeb`. All 33 status and three chart input digests match, and the repository link targets and anchors exist locally. The PDF is byte-identical across the live and candidate distributions and site source. Its 54 distinct internal repository links use `v0-alpha.1`, which resolves to `e5627ef`.

I reproduced the candidate text dump. After pin normalization, the only rendered differences from the previous candidate are the five wording edits. All 13 FAQ entries remain present. The added scope and clarification bring the glance from 343 to 370 words. No regression found in the reviewed surfaces.

The supplied logs show all nine site steps exit zero, with 40 browser tests passed and two expected skips. Both code-prefix tests passed with exit zero. I did not rerun builds or browser tests.

DONE
