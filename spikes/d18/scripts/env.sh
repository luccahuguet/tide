# Sourced by the spike scripts. Builds the facade once and defines `tide_rustc`.
set -euo pipefail
D18="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
OUT="$D18/examples/target"
FACADE_TARGET="$D18/facade/target"
FACADE_RLIB="$FACADE_TARGET/debug/libtide_std.rlib"
FACADE_DEPS="$FACADE_TARGET/debug/deps"
mkdir -p "$OUT"

build_facade() {
  # The facade and every user crate must come from the same rustc.
  cargo build --quiet --manifest-path "$D18/facade/Cargo.toml" --target-dir "$FACADE_TARGET"
}

# rustc for a Tide crate: the facade injected as `std`, and nothing else.
tide_rustc() {
  rustc --edition 2024 --extern "std=$FACADE_RLIB" -L "dependency=$FACADE_DEPS" --out-dir "$OUT" "$@"
}

pass() { printf 'PASS  %s\n' "$*"; }
fail() { printf 'FAIL  %s\n' "$*"; exit 1; }

# expect_compile_error <file> <pattern>: the file must fail to compile with
# an error matching the pattern.
expect_compile_error() {
  local file="$1" pat="$2" log
  log="$(tide_rustc --crate-type lib --emit metadata "$file" 2>&1)" && fail "$file compiled"
  grep -qE "$pat" <<<"$log" || { printf '%s\n' "$log"; fail "$file: wrong error, wanted /$pat/"; }
  pass "$(basename "$file") rejected: $(grep -m1 -E '^error' <<<"$log")"
}
