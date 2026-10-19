#!/usr/bin/env bash
# Extract Orbit and orbit-protocol from the local Eon checkout at the pinned
# commit, so the spike does not carry a copy of Eon's sources. eon/Cargo.toml
# (the trimmed workspace) and eon/Cargo.lock (cargo's lock for it) are kept.
set -euo pipefail
D18="$(cd "$(dirname "$0")/.." && pwd)"
EON_SRC="${EON_SRC:-$HOME/pjs/eon/eon}"
EON_REV=2332d4b
if [ ! -d "$D18/eon/orbit" ]; then
  git -C "$EON_SRC" archive "$EON_REV" orbit \
    | tar -x -C "$D18/eon" --exclude='orbit/tools' --exclude='orbit/docs'
fi
if [ ! -d "$D18/admit/fixtures/orbit-protocol" ]; then
  mkdir -p "$D18/admit/fixtures/orbit-protocol"
  git -C "$EON_SRC" archive "$EON_REV" orbit/crates/protocol \
    | tar -x -C "$D18/admit/fixtures/orbit-protocol" --strip-components=3
fi
