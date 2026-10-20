#!/usr/bin/env bash
# rustc oracle for the E1-dialect trial: type- and borrow-checks each program
# in ../programs as a Rust 2024 crate against a stub Tide std injected as
# `std` (the spikes/d18 mechanism), once as a library or binary and once with
# --test. The stub's bodies are unimplemented!(), so this decides type and
# borrow verdicts only; it runs nothing. Needs stable rustc and cargo 1.96 and
# the crates in Cargo.lock (offline).
#
# Usage: trials/e1-dialect/rustc/run.sh [program ...]   (default: all)
set -uo pipefail
HERE="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROGRAMS="$HERE/../programs"
TARGET="$HERE/target"
OUT="$TARGET/oracle"
mkdir -p "$OUT"

cargo build --quiet --offline --manifest-path "$HERE/std/Cargo.toml" --target-dir "$TARGET" || exit 2
RLIB="$TARGET/debug/libtide_std_stub.rlib"
DEPS="$TARGET/debug/deps"

check() { # check <file> <label> <rustc args...>
  local file="$1" label="$2" log
  shift 2
  if log="$(rustc --edition 2024 --extern "std=$RLIB" -L "dependency=$DEPS" -D unused-imports \
      --emit metadata --out-dir "$OUT" "$@" "$file" 2>&1)"; then
    printf 'PASS  %-14s %-5s%s\n' "$(basename "$file" .rs)" "$label" "$(grep '^warning' <<<"$log" | grep -vc ' emitted$' | sed 's/^0$//;s/.\+/ (& warnings)/')"
  else
    printf 'FAIL  %-14s %s\n' "$(basename "$file" .rs)" "$label"
    status=1
  fi
  [[ -n "$log" ]] && sed 's/^/      /' <<<"$log"
}

status=0
names=("$@")
[[ ${#names[@]} -eq 0 ]] && names=($(cd "$PROGRAMS" && ls *.rs | sed 's/\.rs$//'))
for n in "${names[@]}"; do
  f="$PROGRAMS/$n.rs"
  kind=lib
  grep -q '^fn main' "$f" && kind=bin
  check "$f" "$kind" --crate-type "$kind" --crate-name "$n"
  check "$f" test --test --crate-name "${n}_test"
done
exit "$status"
