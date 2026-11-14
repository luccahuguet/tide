Model: claude-opus-5-5 · Date: 2026-11-14
Verdict: request_changes

Reviewed `git diff 79d7a5c d22b255` (coordinator.py, gate_checks.py, harness_test.py, README.md) on the candidate worktree at d22b2559, which stayed clean. I also read the bead's latest comments, the author's "## CLI 2.1.321 fix" and "## Codex r1 fix" sections, and Codex's r1 review and re-check. Probes and source excerpts are in `tide-e1-gate/opus-cli/`. I made one tiny live call (`live-toolset/`) and one init-only call that was killed at init (`init-connectors-off/`). The token went through the environment only, and all output was filtered for `sk-ant`. Nothing compiled, so no heavy run was needed.

## Findings

1. **Medium, blocking. Without `--safe-mode`, the CLI still tries to load claude.ai connectors after init. The only thing that stops it is the scope of the current token.** `trials/e1-checked/coordinator.py:171` (`WRITER_ENV`); README `trials/e1-checked/README.md:288-291`.
   - **The safe-mode gate.** The pinned CLI gates fifteen customization sources on safe mode: `hr("hooks"|"skills"|"mcpAutoDiscovered"|"agents"|"themes"|"keybindings"|"outputStyles"|"plugins"|"statusLine"|"mcpClaudeAi"|"lspServers"|"fileSuggestion"|"mcpAgentFrontmatter"|"workflows"|"pluginMonitors")`.
   - **Covered sources.** The replacements and the unchanged flags cover every source except `mcpClaudeAi`:
     - hooks: `disableAllHooks`;
     - plugins and their LSP servers, monitors and forced output styles: `enabledPlugins` false, plus `plugins == []` at init;
     - auto-discovered MCP: `--strict-mcp-config`;
     - skills and workflows: `--disable-slash-commands`;
     - agents: no Agent or Task tool;
     - output styles: `--restricted` drops the user and project sources, and the new `/etc/claude-code` check covers the managed source;
     - themes, keybindings, status line and file suggestion: UI only in `-p` mode.
   - **The uncovered source.** The connector loader (`Tys`, excerpt in `opus-cli/tys-source.txt`) checks in this order: the env or setting opt-out, then safe mode, then auth, then the token's `user:mcp_servers` scope. It never checks `--strict-mcp-config`.
   - **Evidence.** The live candidate run (`opus-cli/live-toolset/summary.json`) logs:
     - `[MCP] claude.ai connectors running fully async (nonblocking)`;
     - then `[claudeai-mcp] Missing user:mcp_servers scope (scopes=user:inference)`.

     The fetch starts, and the only thing that stops it is the token scope. Under `--safe-mode` the loader returned at "Disabled in safe mode" before reaching any of this.
   - **Why it matters.** The loader runs async and nonblocking, so connectors that do load arrive after init. The exact init assertion can't see them. `research/24-e1-gate-machinery.md:183` records that the research session "received an unrequested MCP server mid-run" and counts `--safe-mode` as one of the guards.
     - With a token that has full scope, writers would see the owner's claude.ai connector tools mid-session, in both arms. The owner's account has Gmail, Drive, Calendar and Docs connectors.
     - Calls to them would be denied by `dontAsk` and fail the audit as forbidden tools, so nothing leaks. But a writer's tool list would drift from the frozen five, and init can't detect it.
     - The current token (`user:inference`) blocks this today. Nothing records or asserts that property, so the confinement now depends on a credential detail outside the identity.
   - **Fix.** Add `"ENABLE_CLAUDEAI_MCP_SERVERS": "0"` to `WRITER_ENV`. Name claude.ai connectors in the comment at `coordinator.py:164-167` and in the README sentence at :288-291.
   - **Verified fix.** In `opus-cli/init-connectors-off/summary.json`, the candidate argv and env plus that variable give `[claudeai-mcp] Disabled via env var`, and the unchanged `init_assertion` passes. The variable is arm-independent and is not in argv, so `writer_argv_template` is unchanged.
   - **Proof after the fix.** Rerun `harness_test`, plus the scratch probe or an init-only check with `--debug-file`, and grep for `Disabled via env var`. Re-check scope: only this delta.

2. **Info, no change needed. `CLAUDE_CODE_DISABLE_CLAUDE_MDS=1` does the same thing safe mode did. It is not merely "belt and braces" as the author's account says.**
   - In the pinned startup code, `--safe-mode` itself runs `process.env.CLAUDE_CODE_SAFE_MODE="1", process.env.CLAUDE_CODE_DISABLE_CLAUDE_MDS="1"` (excerpt in `opus-cli/safe-mode-gate.txt`).
   - Codex r1 also showed that the variable matters for `--add-dir`.
   - The code comment at `coordinator.py:164-167` is already accurate. Only the report wording overstates, and it needs no fix.

## Answers to the brief

- **Confinement.** Once finding 1 is fixed, dropping `--safe-mode` is safe for this pinned CLI on this host.
  - **Builtin plugins.** `--settings` disables all four, and `plugins == []` at init catches any newcomer. That includes cc-plugin-agents-md, which would otherwise load AGENTS.md.
  - **Hooks.** `disableAllHooks` closes the remaining inline or managed route. User, project and local hooks are already ignored under `--restricted` with a fresh home, and Codex showed the inline-settings case fires without the setting.
  - **Env switches.** They reproduce safe mode's own CLAUDE.md switch and close auto-memory.
  - **`/etc/claude-code` precondition.** The `lstat` check covers every managed source rooted there, including output styles, skills, agents, CLAUDE.md, rules and `managed-settings.json`. It runs before session writes in `run_writer`, before probe directories, and in `identity` for non-dry `init`. Dry `init` records `{"dry_run": true}` and never calls it, which is correct.
  - **Ancestor `.claude` directories.** The writer's cwd sits under `/home/lucca`, so `/home/lucca/.claude` is an ancestor project `.claude` directory. The project-source loader (`p3o`) reads only when `qt("projectSettings")` holds, and `--restricted` turns that off. The probe init showed `skills: []` and `output_style: default`.
  - **Init assertion.** It is unchanged and still exact.
  - **Remote org policy.** It applies the same way under safe mode, so this change doesn't open it.
- **Writer-facing changes.** `WRITER_SETTINGS` and `WRITER_ENV` are module constants. `writer_argv(cli, model, mcp)` and the env in `run_writer` take no arm input. The only per-session value is the `mcp.json` path, as before, so both arms launch identically.
  - What the writer sees changes only in the restored e1 tools. The init `agents` list gains `statusline-setup`, which is unreachable without an Agent tool.
  - The live call returned exactly `Edit, Read, Write, mcp__e1__check, mcp__e1__test`.
  - The `/etc` refusal raises before the grant, so it can't fall on one arm's draw as a writer attempt.
- **The probe.** It is still a meaningful live check. It runs the real argv and env, the exact init assertion (which caught both 2.1.321 regressions), the denied outside Read/Read/Write, and the inside write and read controls.
  - It doesn't exercise passive context sources such as CLAUDE.md, output styles or async connectors. Those rest on the precondition, the env switches and review evidence.
  - Making the probe assert them is not needed. Finding 1's fix closes the one source that relied on an unrecorded property.
- **Leftovers and tests.**
  - The diff is minimal: one owner (`gate_checks.managed_config_absent`) with three call sites, and no orphans.
  - `research/24-e1-gate-machinery.md:34/51/183` is a historical research record, and leaving it is fine.
  - `CliCompatTests` is strong. It covers four filesystem forms on both entry points, asserts that no launch artifacts are written, and was shown by mutation to catch a missing call.
  - No unit test pins `WRITER_SETTINGS` or `WRITER_ENV`. That is acceptable under AP-TEST-001: the plugin half fails init loudly, and a test pinning constants would be implementation trivia.

DONE
