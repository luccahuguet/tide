#!/usr/bin/env bash
# Conformance runner: checks each case's rustc marks with rustc against the
# stub Tide std in rustc/. README.md owns the mechanism, case format, verdicts,
# and requirements.
#
# Usage: conformance/run.sh [case ...]   check cases (default: all of cases/)
#        conformance/run.sh --trace      list the cases of each rule in rules.md
# Exit: 0 when every case is PASS or AWAITS CHECKER, 1 on any FAIL, 2 on setup errors.
set -uo pipefail
HERE="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
CASES="$HERE/cases"
TARGET="$HERE/target"
OUT="$TARGET/conformance"

# Rule IDs: the first column of rules.md's tables.
mapfile -t IDS < <(sed -n 's/^| *\([A-Z][A-Z0-9]*-[0-9][0-9]*\) *|.*/\1/p' "$HERE/rules.md")
[[ ${#IDS[@]} -gt 0 ]] || { echo "no rule IDs in $HERE/rules.md" >&2; exit 2; }

rules_of() { sed -n '1s#^//@ rules: *##p' "$1" | tr ', ' '\n\n' | sed '/^$/d'; }

if [[ "${1:-}" == --trace ]]; then
  # One "ID case" line per rule each case names, read once per case.
  pairs="$(find "$CASES" -name '*.rs' | sort | while IFS= read -r f; do
      rules_of "$f" | while IFS= read -r id; do printf '%s %s\n' "$id" "${f#"$CASES"/}"; done
    done)"
  for id in "${IDS[@]}"; do
    hits="$(awk -v id="$id" '$1 == id { sub(/^[^ ]+ /, ""); printf " %s", $0 }' <<<"$pairs")"
    printf '%-10s%s\n' "$id" "${hits:- -}"
  done
  exit 0
fi

mkdir -p "$OUT"
cargo build --quiet --offline --manifest-path "$HERE/rustc/std/Cargo.toml" --target-dir "$TARGET" || exit 2
RLIB="$TARGET/debug/libtide_std_stub.rlib"
DEPS="$TARGET/debug/deps"

status=0
fail() { printf 'FAIL  %s: %s\n' "$name" "$1"; status=1; }

check() { # check <file>
  local file="$1" rules bad marks want fx kind mode log="" code=0 got
  name="${file#"$CASES"/}"
  [[ -f "$file" ]] || { fail "no such case"; return; }
  rules="$(rules_of "$file")"
  [[ -n "$rules" ]] || { fail "no '//@ rules:' first line"; return; }
  bad="$(grep -vxF -f <(printf '%s\n' "${IDS[@]}") <<<"$rules" | tr '\n' ' ')"
  [[ -z "$bad" ]] || { fail "rule IDs not in rules.md: $bad"; return; }
  # Marks: `//~ CODE ...` ends a line; a code is E####, `error` (uncoded), or FX####.
  marks="$(awk '{ i = index($0, "//~"); if (!i) next
      n = split(substr($0, i + 3), c, " "); if (!n) print "bad " NR ":(empty)"
      for (k = 1; k <= n; k++)
        print (c[k] ~ /^(E[0-9][0-9][0-9][0-9]|error)$/ ? "want " : \
               c[k] ~ /^FX[0-9][0-9][0-9][0-9]$/ ? "fx " : "bad ") NR ":" c[k] }' "$file")"
  bad="$(sed -n 's/^bad //p' <<<"$marks" | tr '\n' ' ')"
  [[ -z "$bad" ]] || { fail "malformed marks: $bad"; return; }
  want="$(sed -n 's/^want //p' <<<"$marks" | sort -u)"
  fx="$(sed -n 's/^fx [0-9]*://p' <<<"$marks" | sort -u | tr '\n' ' ')"
  kind=lib
  grep -qw '^fn main' "$file" && kind=bin
  for mode in "--crate-type=$kind" --test; do
    log+="$(rustc --edition 2024 --extern "std=$RLIB" -L "dependency=$DEPS" "$mode" \
        --crate-name "$(basename "$file" .rs | tr -c 'a-zA-Z0-9_\n' _)" --emit metadata \
        --out-dir "$OUT" --error-format=short "$file" 2>&1)"$'\n' || code=1
  done
  # Each rustc error as line:code in the case, path:line:code elsewhere, ?:error if
  # unlocated; the code is `error` for an uncoded error.
  got="$(awk -v f="$file" '
      /^error: aborting due to/ { next }
      match($0, /:[0-9]+:[0-9]+: error(\[E[0-9]+\])?:/) {
        path = substr($0, 1, RSTART - 1); split(substr($0, RSTART + 1, RLENGTH - 1), p, ":")
        c = p[3] ~ /\[/ ? substr(p[3], 8, length(p[3]) - 8) : "error"  # p[3] is " error[E....]"
        print (path == f ? "" : path ":") p[1] ":" c; next }
      /^error/ { print "?:error" }' <<<"$log" | sort -u)"
  if [[ "$want" != "$got" || ( -z "$want" && $code != 0 ) ]]; then
    fail "rustc errors [${got//$'\n'/ }], marked [${want//$'\n'/ }], exit $code"
    sed '/^$/d; s/^/      /' <<<"$log"
  elif [[ -n "$fx" ]]; then
    printf 'AWAITS CHECKER  %s: %s\n' "$name" "${fx% }"
  else
    printf 'PASS  %s\n' "$name"
  fi
}

if [[ $# -gt 0 ]]; then
  for a in "$@"; do
    [[ "$a" == /* ]] || a="$CASES/${a%.rs}.rs"
    check "$a"
  done
else
  while IFS= read -r f; do check "$f"; done < <(find "$CASES" -name '*.rs' | sort)
fi
exit "$status"
