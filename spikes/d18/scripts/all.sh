#!/usr/bin/env bash
# Runs every criterion script this part of the spike owns; exits nonzero on the first failure.
set -euo pipefail
here="$(dirname "$0")"
for s in c1-facade c3-contracts c5-floats c6-eon c7-sizes; do
  printf '\n### %s\n' "$s"
  bash "$here/$s.sh" </dev/null
done
