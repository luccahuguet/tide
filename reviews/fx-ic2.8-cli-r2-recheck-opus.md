Model: claude-opus-5-5 · Date: 2026-11-14
Verdict: approve

1. **Opus r2 finding 1 is resolved.** The writer now switches off claude.ai connectors itself, so it no longer depends on the token's scope. Location: `trials/e1-checked/coordinator.py:174-175` (`WRITER_ENV`).
   - **Delta.** I reviewed `git diff d22b255 62335e9`, which changes only three things:
     - `WRITER_ENV` adds `"ENABLE_CLAUDEAI_MCP_SERVERS": "0"`;
     - the comment at `coordinator.py:164-169` names the connectors;
     - the README sentence at `README.md:288-293` names them too.
   - **Unchanged.** The argv, `writer_argv_template` and `init_assertion` are unchanged. The variable is a module constant in the env, so both arms still launch identically.
   - **Rerun at 62335e9.** I reran my init-only check with no extra env: `opus-cli/init_variant.py recheck-62335e9`.
     - It imports the candidate's own `writer_argv` and `WRITER_ENV` from the worktree at HEAD 62335e9b.
     - It runs the pinned CLI 2.1.321 with a fresh config home. The token went through the env only, and the CLI was killed at init, before any model call.
     - The debug log shows `[MCP] claude.ai connectors running fully async (nonblocking)`, then `[claudeai-mcp] Disabled via env var`.
     - The unchanged `init_assertion` passes.
     - Without the variable, my r2 run `live-toolset` stopped later, at `Missing user:mcp_servers scope (scopes=user:inference)`. With it, the loader returns at the env opt-out, before any auth or scope check, which is the same position the safe-mode gate held.
     - Evidence: `opus-cli/recheck-62335e9/summary.json`. It contains no credential strings.
   - **Text.** The comment and README wording are accurate.
   - **Tests.** No test change was needed. The author reports `harness_test` at 62335e9b as 79 OK; I didn't rerun it, since the delta adds one env constant that no test pins.

No new findings.

DONE
