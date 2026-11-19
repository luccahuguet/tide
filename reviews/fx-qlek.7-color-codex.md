Model: gpt-6.1-sol (high) · Date: 2026-11-19 · Candidate 8ba5e47 (tide-admit --color)

Verdict: request_changes. High confidence in the static review; the implementation direction is sound. One missing regression test blocks approval under GOVERNANCE.md's requirement to protect consequential behavior. No production-code defect was confirmed.

What this change does: Adds --color auto|always|never, including =VALUE spelling, defaulting to whether stderr is a terminal. The parent resolves auto before piping compiler stderr, the replacement emitter applies that choice, and successful diagnostic caches are separated by the resolved color value. Compiler/replay semantic options remain unchanged.

Blocking findings

1. [P2] Exercise successful cached warnings when changing color modes — crates/tide-admit/tests/invocation.rs:41-45; crates/tide-admit/src/invocation.rs:1310-1312.

   The only color fixture is rejected with TD0301, and every run asserts exit 1. Failed builds discard their staging directory at invocation.rs:1361-1369; they never store or replay stderr through the successful-cache paths at 1371-1372 and 1415-1417. Thus this test, the existing cache tests, and the reported pty rejection probe all miss the new cache/color interaction. Removing the new color record would still satisfy the new color test, while a successful --color=always build with a warning could leave ANSI in a later piped/default or --color=never run. Conversely, a plain cached warning could suppress requested color. Preserving plain bytes in non-tty output is an explicit acceptance condition, and this new boundary has no regression protection.

   Minimum fix: add one integration test using an admitted source that succeeds with a warning (for example, an unused local). With stderr captured and the same output directory, run always, never, and always again, including repeats that exercise cache hits. Assert success, ANSI only for always, equal text after stripping ANSI, byte-identical repeated output, and two cache entries (or equivalent evidence of separate, reused entries). Include default/auto piped output in the plain comparison. This must fail if the color component is removed from the cache key. The existing rejection test can remain.

Non-blocking

2. Document the environment convention explicitly — docs/driver.md:96-98; crates/tide-admit/src/invocation.rs:438-444, 1245-1253.

   NO_COLOR and CLICOLOR_FORCE are ignored, as is TERM: the driver clears its environment before compilation and resolves auto solely with stderr.is_terminal(). Consequently NO_COLOR=1 on a tty still colors, while CLICOLOR_FORCE=1 on a pipe does not color in auto mode. Explicit always/never wins through the resolved ColorConfig. The documented tty-only rule is accurate and does not promise environment overrides, so this is not a contract violation. One sentence saying these variables do not override the flag would make the choice clear to operators expecting those conventions; no new environment support is needed.

3. Add the cheap parser boundary cases to the existing table — crates/tide-admit/tests/invocation.rs:67-69, 138-155.

   The parser is correct by inspection, but the new test covers only one invalid value. Adding bare --color, --color=, --color=yes, and duplicate color options to the malformed-argument table would preserve the intended error behavior at little cost. --color always is valid, not an error: the driver deliberately accepts both value spellings, as its docs specify.

Review observations

- Parsing: exactly --color was added to the fixed value-option list. A bare terminal --color returns missing value; an empty value returns invalid value; an unsupported value returns unsupported argument; a duplicate with a value falls through to unsupported argument. --color followed by another flag returns invalid value. Two-argument --color always/auto/never is accepted. No other option's parsing branch changed. without_color runs only after successful parsing, removes both supported spellings, and replaces them with exactly one resolved child argument.
- Propagation: the original arguments survive the initial environment-clearing re-exec, whose stderr is inherited. The normal compile parent then resolves tty state, supplies always/never to its piped child, and keys cached stdout/stderr by the same bool. The in-process paired-library route copies color to its library Invocation at invocation.rs:683. No new dependency or public export was introduced.
- rustc: the operator flag is consumed, not forwarded in rustc_args. Rustc-owned diagnostics emitted through the installed ParseSess diagnostic context and Tide diagnostics share RustcLabels' AnnotateSnippetEmitter, including its delegated supports_color(). Thus those diagnostics use one color policy in a run. Earlier operator errors and direct eprintln! diagnostics retain their existing plain formatting; this patch does not change those emission paths. No claim of runtime coverage for every early rustc failure is made here.
- Plain output: resolving to Never constructs the same emitter with the same source map and short_message(true) as the base revision. Labels, codes, wording, admission decisions, and status handling are untouched. Always changes the emitter destination's styling. The byte comparison in the added rejection test is deterministic because Command::output captures stderr, regardless of the test runner's tty. The successful-cache boundary remains the blocking coverage gap above.
- Identity: color stays outside rustc_args and normalized replay semantic options in invocation/replay.rs:58-103. Only the presentation cache gains the resolved bool. Replay configuration restrictions are unchanged.
- Documentation: docs/driver.md lists both value spellings and the default correctly; --help lists the flag. Searched README.md, docs/, design/, conformance/rules.md, and crate Markdown for flag inventories and plain/uncolored-output claims. No other conflicting current claim or exhaustive flag inventory was found; crates/*/README* has no matches/files in this tree.
- Callers: inspected the conformance runner's fixed argument construction and merged() at crates/tide-conformance/src/main.rs:832-847: it directs stderr to a pipe. The demo records driver output to files or captures it. These consumers retain plain auto diagnostics without adding flags. The existing recipes and fixed argument lists need no change merely to enable tty auto. The separate site's recording sources are outside this candidate; their behavior was assessed only from the supplied builder account.

Evidence and limits

Reviewed the complete dc0997f..8ba5e47 diff excluding .beads, affected code and surrounding invocation, label, replay, test and caller paths, GOVERNANCE.md, and the builder's color-flag account. Candidate HEAD is 8ba5e47f73df1eb34e1a88be80dc18cd9ec2a42d; working tree was clean. Diff SHA-256: 5f3d4f978ef41f33296f33879569f945e782a5474770b613e4e577414320b47f.

Own check: git diff --check dc0997f 8ba5e47 -- . ':!.beads' passed on this candidate. Read the existing color-focused, full tide-admit, and post-fix color-probe logs. The focused test reports passing after the final emitter resolution fix. Existing pty logs show default and always with ANSI and never without ANSI, all exiting 1. The full suite predates that final fix, as the builder states; it is not proof of the exact final candidate. These are builder observations, not independently rerun checks.

Minimum patch for approval: finding 1's deterministic successful-cache regression test and its passing focused run. No implementation redesign is requested. Re-check can be limited to that test and its evidence if production behavior stays unchanged.

Not checked: no cargo, nix, docker, builds, driver execution, or new pty probes, per the review restrictions. No files were changed except this report; no commits, .beads writes, or contact with other sessions occurred.
Risk: cache/color separation is presently correct by inspection but lacks an executable regression guard; environment color conventions are intentionally not implemented by the current tty-only rule.

DONE
