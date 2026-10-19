#!/usr/bin/env bash
# C7: size of each prototype part (code lines, excluding build output).
set -euo pipefail
D18="$(cd "$(dirname "$0")/.." && pwd)"
for part in facade/src macros/src scripts examples; do
  printf '== %s\n' "$part"
  tokei --exclude target "$D18/$part" | grep -E '^ (Rust|Shell|BASH|Total)' || true
done
