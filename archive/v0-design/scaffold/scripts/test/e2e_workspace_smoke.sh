#!/usr/bin/env bash
set -euo pipefail

log() {
  printf '[e2e-workspace-smoke] %s\n' "$*"
}

log "running cargo test --workspace"
cargo test --workspace

log "running ferox bootstrap-status smoke"
status_output="$(cargo run -p ferox_cli -- bootstrap-status)"
printf '%s\n' "$status_output"

if ! grep -q "Ferox Rust bootstrap workspace is wired" <<<"$status_output"; then
  log "expected bootstrap status banner was not found"
  exit 1
fi

log "running ferox fmt-smoke"
fmt_output="$(cargo run -p ferox_cli -- fmt-smoke 'fn main() -> Int { 1 }')"
printf '%s\n' "$fmt_output"

if ! grep -Fxq "fn main() -> Int { 1 }" <<<"$fmt_output"; then
  log "unexpected fmt-smoke output"
  exit 1
fi

log "workspace smoke passed"
