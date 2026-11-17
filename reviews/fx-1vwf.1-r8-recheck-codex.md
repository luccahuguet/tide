Model: gpt-6.1-sol (high) · Date: 2026-11-17

Verdict: approve

## Scope and candidate

Read-only finder re-check of `git diff 44fd03f e8973cf`, the affected canary and launch paths, the added test, protocol §8.5, and report.md's Round 8. HEAD is `e8973cfe24439ae5a853e9841bc8044d8f80a2e3`; the worktree is clean. The amendment changes only `trials/e3-pilot/harness/src/main.rs` and `trials/e3-pilot/protocol.md` (66 insertions, 6 deletions). Independently computed SHA-256 of `git diff fab99b3...e8973cf` matches `a4d28f9517ecf8ca1eb6ef0549e3c0de52e62a5931deedd6a99a7c113398f34a`.

The contract checked: each control must demonstrate an instruction surface and a hook surface with valid plants; the isolated attempts must suppress the planted context and hooks through the production launch path. A reported hook-configuration parse error must prevent canary success even when another hook fires.

## Source citations and c8 diagnosis

Confirmed. I independently resolved GitHub's annotated [rust-v0.169.0 tag](https://api.github.com/repos/openai/codex/git/ref/tags/rust-v0.169.0) through the GitHub API: [tag object a41fe5b](https://api.github.com/repos/openai/codex/git/tags/a41fe5b443652cbdbee57717fbb9fd78d8d8b32a) points to commit `d32063f96c2dff1dc96af839b3952abb6acfeda4`, exactly the report's source revision. The existing sparse source checkout has that HEAD, a clean status, and origin `https://github.com/openai/codex`. I read the source there; paths below are relative to `codex-rs/` at that revision.

The cited locations support the claims:

- `features/src/lib.rs:1515–1520` enables the stable `hooks` feature by default. `exec/src/lib.rs:809` starts the embedded server with `SessionSource::Exec`. `core/src/session/mod.rs:6166–6169` builds the hook configuration from the feature and bypass flag. `core/src/session/turn.rs:411,775,1049` dispatches startup and pending-input hooks. `core/src/hook_runtime.rs:132–178,715–736` includes non-subagent exec sessions in SessionStart and user input in UserPromptSubmit. Exec has a hook surface.
- `hooks/src/engine/discovery.rs:156–182,390–431` loads hooks.json from enabled configuration layers. `config/src/state.rs:229` associates the user layer with its config file's parent, and a project layer with its .codex directory. Parsing errors produce a warning and discard the file.
- `utils/cli/src/shared_options.rs:61–64`, `exec/src/cli.rs:173`, and `exec/src/lib.rs:653` define and propagate the hook-trust bypass. `hooks/src/engine/discovery.rs:917–923,1000–1017` confirms that bypass admits enabled handlers without persisted handler hashes. Project trust and handler trust are separate gates; the control supplies both project trust and the existing bypass flag.
- `config/src/loader/mod.rs:1694–1739` falls back to cwd without a project marker and accepts a .git directory containing HEAD. `config/src/project_root_markers.rs:5` supplies the default .git marker. Project-layer gating and trust lookup match the cited loader and config locations; `config/src/state.rs:744–746` filters disabled layers. `core/src/agents_md.rs:1–17,214–249` independently confirms root-to-cwd instruction discovery.

I also inspected saved c8 evidence. Its control log reports only X-HOME-MD and X-CONFIG for Sol. `canary-c8/control/sol/events.jsonl` line 4 records the planted home hooks.json parse failure with missing field `type`; lines 2–3 acknowledge the bypass flag. This supports a malformed plant, not an absent exec hook facility.

## Planted schema and ancestor discovery

Fixed. The [HooksFile definition](https://github.com/openai/codex/blob/rust-v0.169.0/codex-rs/config/src/hook_config.rs#L12-L19) permits the generated top-level hooks object. The [event, matcher-group, and handler definitions](https://github.com/openai/codex/blob/rust-v0.169.0/codex-rs/protocol/src/hooks.rs#L140-L161) require the internally tagged command handler. The amended `probes()` (`harness/src/main.rs:2279`) supplies `type: command`, a string command, and timeout 10, an accepted u64, under valid SessionStart and UserPromptSubmit event names. The matcher may be omitted. No unsupported field is planted.

`canary()` (`:2486`) creates parent/.git/HEAD before either control starts. The parent path is canonical because the canary canonicalizes its directory before calling `probes()`. The trust entry (`:2286`) is placed in the planted home's config.toml, keyed by that parent path. Thus the control can discover the ancestor's .codex layer and AGENTS.md. A missing project config.toml does not omit the hook layer: `config/src/loader/mod.rs:1957–1969` creates an empty project layer for that case. The existing bypass then admits its hook handlers.

The added test checks both Codex plants' generated shape, tag, marker command, integer timeout, and trust entry. It is a local schema regression check, not execution of the actual Codex parser; direct source inspection supplies the schema comparison in this review. Protocol §8.5 and Round 8 accurately describe the amendment.

## Control trust does not leak into isolated configuration

Confirmed by tracing `canary()` → `attempt()` → `reviewer()` → `clean_command()` and `make_homes()`.

The control gets canary/home as HOME and canary/home/.codex as CODEX_HOME. After the controls, the harness deliberately points its own environment at those planted locations. The isolated attempt still creates its own root beneath parent/tmp, with a different home, cwd, and temporary directory. `make_homes()` (`:1432`) copies only auth.json into its Codex home; it copies no config.toml or trust entry.

`clean_command()` (`:1552`) clears the child environment, passes only the six listed ordinary environment variables, then explicitly sets the attempt HOME and CODEX_HOME. `reviewer()` (`:1579`) replaces XDG paths and TMPDIR and retains `--ignore-user-config`, `--ignore-rules`, `project_doc_max_bytes=0`, and `--disable hooks`. The control-only hook-trust bypass is not passed to the reviewer. The credential hand-back copies only authentication bytes, so it provides no configuration transfer either.

The ancestor marker remains above the isolated cwd, as intended. The isolated process encounters the same potential project surfaces but receives no planted trust setting. At the pinned source revision, `config/src/loader/mod.rs:558` substitutes an empty user layer when user configuration is ignored; project trust cannot be recovered from the control home. Separately, `core/src/agents_md.rs:65` skips project instructions at a zero budget and `hooks/src/engine/mod.rs:300–309` skips ordinary hook discovery when disabled with no plugin sources. The new ancestor therefore strengthens the plant without changing the isolated launch policy.

## Parse errors cannot silently produce a passing canary

Confirmed for the pinned CLI. Discovery emits the exact substring `failed to parse hooks config`. `core/src/session/session.rs:1802` forwards startup warnings as events, and `exec/src/event_processor_with_jsonl_output.rs:445` forwards warnings into JSONL error items; its `runtime_warning_emits_a_non_fatal_error_item` test checks this behavior. The saved c8 event demonstrates this path for the actual failure.

The new check (`harness/src/main.rs:2509`) inspects that same stdout event log and appends a failure independently of the hook/instruction coverage condition. Neither later controls nor isolated attempts clear the failure vector. The final branch (`:2558`) returns `Fail::Canary` whenever it is nonempty; `main()` prints the failure and returns exit 1. Consequently a valid home hook cannot mask a malformed ancestor hook, or vice versa, when the pinned CLI reports the parse error. Earlier operational errors also return failure rather than canary success.

## New findings and outstanding live proof

No new blocking findings or non-blocking improvements in this amendment. No alternative implementation or further patch is required. Confidence is high for this source-level re-check. No override.

Original finding 3 remains open: approval of the amendment does not establish that the corrected hooks fire live or that the isolated attempts pass. The coordinator must rerun the canary on this candidate and publish its actual coverage before closing that proof requirement. Round 8 explicitly preserves this limitation.

## Evidence and limits

`git diff --check 44fd03f e8973cf` passes. I inspected `harness-test-r8.log`: three main suites each report 20 passed, 0 failed, 1 ignored; the private-mode child passes separately each time. These are saved builder results, not independently reproduced execution or independent binding of the tested binary to this revision.

Only this requested review file was written. No repository or Beads edits, Cargo, harness execution, or reviewer sessions.

Not checked: independent runtime execution or the corrected live canary.
Risk: live hook firing and isolation still require the coordinator's canary proof.

DONE
