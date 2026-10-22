#!/usr/bin/env bash
# Conformance runner: checks each case's rustc marks with stable rustc, and its
# marks for the codes the admission driver decides with the driver, both against
# the stub Tide std in rustc/. README.md owns the mechanism, case format,
# verdicts, and requirements.
#
# Usage: conformance/run.sh [case ...]   check cases (default: all of cases/)
#        conformance/run.sh --trace      list the cases of each rule in rules.md
# Exit: 0 when every case is PASS or AWAITS CHECKER, 1 on any FAIL, 2 on setup errors.
set -uo pipefail
# Physical paths: the driver rejects a package path that traverses a directory symlink.
HERE="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd -P)"
ROOT="$(cd "$HERE/.." && pwd -P)"
CASES="$HERE/cases"
TARGET="$HERE/target"
OUT="$TARGET/conformance"
# The oracle is stable rustc, whatever toolchain the working directory pins; the
# driver, and the stub std it reads, build with the repository's pinned nightly.
oracle() { RUSTUP_TOOLCHAIN=stable "$@"; }
pinned() { (cd "$ROOT" && env -u RUSTUP_TOOLCHAIN "$@"); }

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

mkdir -p "$OUT/admit"
oracle cargo build --quiet --offline --manifest-path "$HERE/rustc/std/Cargo.toml" --target-dir "$TARGET" || exit 2
RLIB="$TARGET/debug/libtide_std_stub.rlib"
DEPS="$TARGET/debug/deps"

pinned cargo build --quiet --offline -p tide-admit --target-dir "$ROOT/target" || exit 2
DRIVER="$ROOT/target/debug/tide-admit"
# The FX codes the driver decides, as an extended regex alternation.
CODES="$("$DRIVER" --tide-print-codes | paste -sd '|')" && [[ -n "$CODES" ]] || exit 2
# The stub std again, for the driver; each library it loads is in a directory
# that cargo's artifact messages name.
json="$(pinned cargo build --quiet --offline --manifest-path "$HERE/rustc/std/Cargo.toml" \
    --target-dir "$TARGET/admit" --message-format=json)" || exit 2
ADMIT_RLIB="$TARGET/admit/debug/libtide_std_stub.rlib"
ADMIT_DEPS=()
while IFS= read -r dir; do ADMIT_DEPS+=(--tide-dep-dir "$dir"); done < <(
  grep -o '"filenames":\[[^]]*\]' <<<"$json" | grep -o '"[^"]*\.\(rlib\|so\|dylib\)"' | tr -d '"' \
    | while IFS= read -r lib; do printf '%s\n' "${lib%/*}"; done | sort -u)

status=0
fail() { printf 'FAIL  %s: %s\n' "$name" "$1"; status=1; }

check() { # check <file>
  local file="$1" rules bad marks want fx driven awaits kind mode log="" code=0 got
  local grant=() dlog="" dcode=0 dgot c
  name="${file#"$CASES"/}"
  [[ -f "$file" ]] || { fail "no such case"; return; }
  rules="$(rules_of "$file")"
  [[ -n "$rules" ]] || { fail "no '//@ rules:' first line"; return; }
  bad="$(grep -vxF -f <(printf '%s\n' "${IDS[@]}") <<<"$rules" | tr '\n' ' ')"
  [[ -z "$bad" ]] || { fail "rule IDs not in rules.md: $bad"; return; }
  bad="$(awk 'NR > 1 && /^\/\/@/ && !/^\/\/@ (grants: unsafe|profile: release)$/ { printf "%d ", NR }' "$file")"
  [[ -z "$bad" ]] || { fail "unknown '//@' lines: $bad"; return; }
  # Marks: `//~ CODE ...` ends a line; a code is E####, `error` (uncoded), or FX####.
  marks="$(awk '{ i = index($0, "//~"); if (!i) next
      n = split(substr($0, i + 3), c, " "); if (!n) print "bad " NR ":(empty)"
      for (k = 1; k <= n; k++)
        print (c[k] ~ /^(E[0-9][0-9][0-9][0-9]|error)$/ ? "want " : \
               c[k] ~ /^FX[0-9][0-9][0-9][0-9]$/ ? "fx " : "bad ") NR ":" c[k] }' "$file")"
  bad="$(sed -n 's/^bad //p' <<<"$marks" | tr '\n' ' ')"
  [[ -z "$bad" ]] || { fail "malformed marks: $bad"; return; }
  want="$(sed -n 's/^want //p' <<<"$marks" | sort -u)"
  fx="$(sed -n 's/^fx //p' <<<"$marks" | sort -u)"
  # The FX marks the driver decides, as line:code, and the codes it does not.
  driven="$(grep -E ":($CODES)\$" <<<"$fx")"
  awaits="$(grep -vE ":($CODES)\$" <<<"$fx" | sed -n 's/^[0-9]*://p' | sort -u | tr '\n' ' ')"
  kind=lib
  grep -qw '^fn main' "$file" && kind=bin
  # -Zallow-features= as the admission driver passes it (ADMIT-2); stable rustc
  # takes a -Z flag only under RUSTC_BOOTSTRAP.
  for mode in "--crate-type=$kind" --test; do
    log+="$(RUSTC_BOOTSTRAP=1 oracle rustc -Zallow-features= --edition 2024 --extern "std=$RLIB" -L "dependency=$DEPS" "$mode" \
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
    return
  fi
  # The driver passes rustc -Zallow-features= itself (ADMIT-2) and takes the
  # case's grant as a flag.
  grep -qx '//@ grants: unsafe' "$file" && grant=(--tide-grant=unsafe)
  # Exit 1 is a rejection; any other failure, or a panic, is the driver's crash.
  for mode in "--crate-type=$kind" --test; do
    c=0
    dlog+="$("$DRIVER" "${grant[@]}" --edition 2024 --tide-std "$ADMIT_RLIB" "${ADMIT_DEPS[@]}" "$mode" \
        --crate-name "$(basename "$file" .rs | tr -c 'a-zA-Z0-9_\n' _)" --emit metadata \
        --out-dir "$OUT/admit" --error-format=short "$file" 2>&1)"$'\n' || c=$?
    (( c > dcode )) && dcode=$c
  done
  if (( dcode > 1 )) || grep -qE "internal compiler error|thread '.*' panicked" <<<"$dlog"; then
    fail "the driver crashed (exit $dcode)"
    sed '/^$/d; s/^/      /' <<<"$dlog"
    return
  fi
  # Each driver diagnostic as line:code in the case, path:line:code elsewhere.
  dgot="$(awk -v f="$file" 'match($0, /:[0-9]+:[0-9]+: error: FX[0-9][0-9][0-9][0-9]:/) {
      path = substr($0, 1, RSTART - 1); split(substr($0, RSTART + 1, RLENGTH - 2), p, ":")
      print (path == f ? "" : path ":") p[1] ":" substr(p[4], 2) }' <<<"$dlog" | sort -u)"
  # The driver must report exactly the marks it decides, and reject exactly the
  # cases that rustc or those marks reject.
  if [[ "$driven" != "$dgot" || -n "$want$driven" && $dcode == 0 || -z "$want$driven" && $dcode != 0 ]]; then
    fail "driver errors [${dgot//$'\n'/ }], marked [${driven//$'\n'/ }], exit $dcode"
    sed '/^$/d; s/^/      /' <<<"$dlog"
  elif [[ -n "$awaits" ]]; then
    printf 'AWAITS CHECKER  %s: %s\n' "$name" "${awaits% }"
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
