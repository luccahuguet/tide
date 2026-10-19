#!/usr/bin/env bash
# Print FILE's size in Claude input tokens: the input tokens `claude -p`
# reports with FILE as the whole prompt, minus those for the prompt ".".
# Both runs use the same model, system prompt, no tools, and no MCP servers
# (whose varying instructions add noise), so the difference is FILE's own
# cost. Needs a logged-in `claude` CLI and python3.
# Usage: scripts/count-tokens.sh FILE   (D14's budget check for reference.md)
set -euo pipefail

[ $# -eq 1 ] && [ -f "$1" ] && [ -s "$1" ] || { echo "usage: $0 FILE (a non-empty file)" >&2; exit 2; }

tokens() {
  timeout 300 claude -p --model claude-opus-5-5 --system-prompt "Reply with OK." \
    --tools "" --strict-mcp-config --no-session-persistence \
    --output-format json -- "$1" </dev/null |
    python3 -c 'import json, sys
r = json.load(sys.stdin)
if r["is_error"]: sys.exit(r["result"])
u = r["usage"]
print(u["input_tokens"] + u.get("cache_creation_input_tokens", 0)
      + u.get("cache_read_input_tokens", 0))'
}

base="$(tokens ".")"
full="$(tokens "$(cat "$1")")"
echo "$((full - base))"
