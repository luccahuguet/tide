Model: `gpt-6.1-sol`. Effort: `xhigh`. Date: 2026-10-20.

## Verdict

**approve — high confidence within this narrow re-check.**

B2 is closed. The replacement preserves the previously accepted coverage and is implementable by D18’s driver. No new contradiction, false statement, or escape outside SEAL-6 was found.

## Scope

Bead `fx-1am`: round 51’s B2 fix, including the two wording adjustments recommended by round 52.

- **HEAD:** `5ec5c70d42491bd1f1e5e99ac2a718b926b6f924`
- **SHA-256 of `git diff -- design/ README.md`:** `430097b1b78be09142c8c99b771d07812636bcd14db1977193b4c76da985f60d`

The diff identity remained unchanged during review. No files were modified.

## Item status

| Item | Status |
| --- | --- |
| B2: sealed generic types | **Closed.** `Feed<T>` names `Feed`; inclusion of `impl<T: Copy> Iterator for Feed<T>` no longer depends on finding concrete instantiations or proving `T: Copy` under the type definition’s bounds. |
| Inherent associated constants | **Still closed.** SEAL-1 includes them; applicable inherent impls remain covered by whole-type entries. |
| Refined method returns | **Still closed.** Explicitly included in the fixed method signatures. |
| Blanket and generic wrapper impls | **Still closed.** “It may hold” covers any impl, including `impl<T: Marker> Bar for &T`; removing “blanket” avoids narrowing this clause. |
| `&Feed`, `&mut Feed`, and `Box<Feed>` impls | **Still closed.** Their self types name `Feed`. The second clause also covers indirect matches, including `impl<T: Copy> Bar for T` through `&Feed`. |
| Resolution hashing for sealed calls | **Still closed.** SEAL-2 continues recording selected definitions; SEAL-6 retains the distinction for unsealed callers. |

Evidence: [SEAL-1 and SEAL-2]( /home/lucca/pjs/lab/tide/design/04-contracts.md:1241), [applicability definition](/home/lucca/pjs/lab/tide/design/04-contracts.md:1285), and [SEAL-6](/home/lucca/pjs/lab/tide/design/04-contracts.md:1347).

## Blocking findings

None. **Minimal patch: none.**

## Non-blocking improvements

None requested. No alternative design is warranted for this fix.

## Reasoning

The first clause fixes B2 directly: nominal occurrence in the impl’s resolved self type includes the generic impl independently of its conditional bounds. Its header, associated definitions, and method signatures therefore enter the whole-type hash. Changing `Item`, tightening bounds, or adding or deleting an applicable impl changes the seal.

The second clause supplies conservative coverage where the self type does not name `Feed`. “Cannot rule … out” includes ambiguous applicability; it does not require a unique, proven selection. This can freeze additional impls, but does not permit an applicable impl to escape.

D18 can implement this using resolved type inspection, fresh inference arguments, header matching, and conservative obligation evaluation over the finite candidate shapes. I checked the installed compiler matching the spike’s pin: `rustc 1.99.0-nightly (771916f90)`. Its coherence implementation already uses fresh generic arguments and retains ambiguous intersections. This matches the documented [rustc coherence approach](https://rustc-dev-guide.rust-lang.org/coherence.html). The spike’s existing [selection helper](/home/lucca/pjs/lab/tide/spikes/d18/admit/src/main.rs:453) discards ambiguous results, so it is not sufficient unchanged; the required compiler machinery is available.

The additions exception inherits the revised definition consistently. Resolution changes in unsealed callers remain admitted by SEAL-6; resolution changes in hashed calls remain detectable. `git diff --check -- design/ README.md` passed.

Not checked: the rest of the diff or executable Tide enforcement.  
Risk: this approves the design text; seal enforcement remains unimplemented.