#!/usr/bin/env bash
# Tide's first running demo (fx-nkv): a lost-update bug in a small concurrent service, found by a
# seed and replayed identically, the fix passing a fixed seed set, and the trust diff of a patch
# that widens the service's public interface to carry `Net`. README.md is the guide.
#
# Usage: demo/run.sh
# Builds the admission driver and Tide's std with the pinned toolchain into $CARGO_TARGET_DIR
# (default: <repo>/target) and works in its tide-demo/ directory, which each run replaces.
# Exit: 0 when every step shows what it should, 1 when one does not, 2 on setup errors.
set -euo pipefail
# Physical paths: the driver rejects a package path that traverses a directory symlink.
HERE="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd -P)"
ROOT="$(cd "$HERE/.." && pwd -P)"
TEST=every_view_is_counted
SUITE="$HERE/tests/src/hits_sim.rs"
FIXED="$HERE/service/src/hits.rs"

say() { printf '%s\n' "$*"; }
step() { printf '\n== %s\n' "$*"; }
setup_error() { say "demo: $*" >&2; exit 2; }
fail() { say "FAIL: $*" >&2; exit 1; }
pinned() { (cd "$ROOT" && env -u RUSTUP_TOOLCHAIN "$@"); }

command -v git >/dev/null || setup_error "git is required to apply the demo patches"
command -v sha256sum >/dev/null || setup_error "sha256sum is required to compare traces"
TARGET="${CARGO_TARGET_DIR:-$ROOT/target}"
mkdir -p "$TARGET" && TARGET="$(cd "$TARGET" && pwd -P)" \
  || setup_error "cannot create the target directory $TARGET"
WORK="$TARGET/tide-demo"
{ rm -rf "$WORK" && mkdir -p "$WORK/logs"; } || setup_error "cannot replace the work directory $WORK"

step "0. Toolchain and build"
dirty="$(git -C "$ROOT" status --porcelain 2>/dev/null || true)"
say "revision:  $(git -C "$ROOT" rev-parse HEAD 2>/dev/null || echo unknown)${dirty:+ (with uncommitted changes)}"
say "toolchain: $(pinned rustc --version)"
pinned cargo build --quiet --locked -p tide-admit --target-dir "$TARGET" \
  || setup_error "cannot build the admission driver"
DRIVER="$TARGET/debug/tide-admit"
# Tide's std and every library it loads, as README.md's demo build recipe collects them.
messages="$(pinned cargo build --quiet --locked -p tide_std --target-dir "$TARGET" --message-format=json)" \
  || setup_error "cannot build Tide's std"
libs="$(grep -o '"filenames":\[[^]]*\]' <<<"$messages" | grep -o '"[^"]*\.\(rlib\|so\)"' | tr -d '"')" \
  || setup_error "cargo reported no library artifacts for Tide's std"
STD=""
DEPS=()
while IFS= read -r lib; do
  DEPS+=(--tide-dep-dir "${lib%/*}")
  [[ "$lib" == */libtide_std.rlib ]] && STD="$lib"
done <<<"$libs"
[[ -n "$STD" ]] || setup_error "cargo reported no libtide_std.rlib"
say "driver:    $DRIVER"
say "std:       $STD"

# The two other revisions: the fixed service with one patch applied. A patch that no longer
# applies is a setup error, so the revisions cannot drift from the service.
revision() { # revision <name> <patch>
  { mkdir -p "$WORK/revisions/$1/src" && cp "$FIXED" "$WORK/revisions/$1/src/hits.rs"; } \
    || setup_error "cannot copy the service to $WORK/revisions/$1"
  # The ceiling keeps git from treating the target directory as part of the repository.
  (cd "$WORK/revisions/$1" && GIT_CEILING_DIRECTORIES="$WORK/revisions" git apply "$2") \
    || setup_error "$2 does not apply to demo/service/src/hits.rs"
  printf '%s' "$WORK/revisions/$1/src/hits.rs"
}
BUGGY="$(revision buggy "$HERE/patches/lost-update.patch")"
WIDENED="$(revision widened "$HERE/patches/metrics-report.patch")"

# The simulation suite, admitted and linked with one revision as `tide_candidate`, and run in
# the driver's sandboxed child. Output goes to a log; the exit status is returned.
suite() { # suite <library> <out> <log> [run arguments]
  local status=0
  "$DRIVER" --test --crate-type=lib --emit=link --crate-name=hits_sim \
    --tide-std "$STD" "${DEPS[@]}" --out-dir "$2" --tide-test-library "$1" "$SUITE" "${@:4}" \
    >"$3" 2>&1 </dev/null || status=$?
  return "$status"
}
summary() { grep -m1 "^simulation $TEST: " "$1" || true; }
manifests() { find "$1/target/sim" -name run.manifest.json 2>/dev/null | sort; }
# The sink records of a trace, the run's output, one message per line.
output_of() { grep '"kind":"sink"' "$1" | sed 's/.*"message":"\([^"]*\)".*/\1/; s/\\u000a$//'; }
hash_of() { sha256sum | cut -c1-16; }

step "1. Find a seed that exposes the bug (buggy revision, demo/patches/lost-update.patch)"
code=0
suite "$BUGGY" "$WORK/out/search" "$WORK/logs/search.log" --tide-run-test "$TEST" || code=$?
line="$(summary "$WORK/logs/search.log")"
[[ $code == 101 && -n "$line" ]] || { cat "$WORK/logs/search.log" >&2; fail "the buggy suite did not fail as a test (exit $code)"; }
failing="$(sed -n "s/^simulation failure seed=\([0-9]*\) test=$TEST\$/\1/p" "$WORK/logs/search.log" | tr '\n' ' ')"
passed="$(sed -n 's/.*: \([0-9]*\) passed; .*/\1/p' <<<"$line")"
failed="$(sed -n 's/.*; \([0-9]*\) failed (.*/\1/p' <<<"$line")"
failing="${failing% }"
SEED="${failing%% *}"
[[ -n "$SEED" ]] || fail "no failing seed reported"
(( passed > 0 )) || fail "every seed failed, so this failure does not depend on the schedule"
# Each failing seed's own cause, the first cause line after its failure line, must be the lost
# hit: a seed that failed the determinism self-check or panicked otherwise fails the demo.
read -r reported lost < <(awk -v test="$TEST" '
  $0 ~ "^simulation failure seed=[0-9]+ test=" test "$" { open = 1; n++; next }
  open && /^  cause: / { open = 0; if (index($0, "failed: lost hits")) lost++ }
  END { print n + 0, lost + 0 }' "$WORK/logs/search.log")
[[ -n "$failed" && $reported == "$failed" && $lost == "$failed" ]] \
  || fail "only $lost of the $failed failing seeds lost a hit ($reported failure reports)"
say "$line"
say "failing seeds: $failing (each cause: lost hits)"
say "same code and input on every seed; only the schedule differs"
say "first failing seed: $SEED"

step "2. Record seed $SEED, then replay the recording twice"
code=0
suite "$BUGGY" "$WORK/out/record" "$WORK/logs/record.log" --tide-run-test "$TEST" --seed "$SEED" || code=$?
mapfile -t recorded < <(manifests "$WORK/out/record")
[[ $code == 101 && ${#recorded[@]} == 1 ]] || fail "seed $SEED did not fail again on a fresh run (exit $code)"
cause="$(grep -m1 '^  cause: ' "$WORK/logs/record.log" || true)"
[[ "$cause" == *"failed: lost hits" ]] || fail "seed $SEED failed for another reason: ${cause:-no cause reported}"
say "${cause#  }"
traces=("${recorded[0]%/*}/trace.ndjson")
for n in 1 2; do
  before="$(manifests "$WORK/out/record")"
  code=0
  suite "$BUGGY" "$WORK/out/record" "$WORK/logs/replay-$n.log" --replay "${recorded[0]}" || code=$?
  # A reproduced failing test exits 101, like the recording; any other status is a failed replay.
  [[ $code == 101 ]] && grep -q "^replay reproduced: test=$TEST seed=$SEED " "$WORK/logs/replay-$n.log" \
    || { cat "$WORK/logs/replay-$n.log" >&2; fail "replay $n did not reproduce the recording (exit $code)"; }
  added="$(comm -13 <(printf '%s\n' "$before") <(manifests "$WORK/out/record"))"
  [[ -n "$added" && "$added" != *$'\n'* ]] || fail "replay $n wrote no single new manifest"
  traces+=("${added%/*}/trace.ndjson")
done
say "                 events  event trace       output trace"
names=(recording replay-1 replay-2)
for i in 0 1 2; do
  printf '%-16s %6s  %s  %s\n' "${names[$i]}" "$(grep -c . "${traces[$i]}")" \
    "$(hash_of <"${traces[$i]}")" "$(output_of "${traces[$i]}" | hash_of)"
done
for i in 1 2; do
  cmp -s "${traces[0]}" "${traces[$i]}" || fail "${names[$i]}'s trace differs from the recording"
done
output="$(output_of "${traces[0]}")"
say "output: $output"
read -r views hits < <(sed -n 's/^views served: \([0-9]*\), hits recorded: \([0-9]*\)$/\1 \2/p' <<<"$output") || true
[[ -n "${hits:-}" ]] && (( hits < views )) || fail "the recorded output shows no lost hit"
say "both replays reproduce the recording: identical event and output traces"
say "recording: ${recorded[0]#"$TARGET/"}"

step "3. Run the fixed revision (demo/service) over the fixed seed set"
code=0
suite "$FIXED" "$WORK/out/fixed" "$WORK/logs/fixed.log" --tide-run-test "$TEST" || code=$?
line="$(summary "$WORK/logs/fixed.log")"
[[ $code == 0 && "$line" == *" 0 failed ("* ]] \
  || { cat "$WORK/logs/fixed.log" >&2; fail "the fixed revision failed its seed set (exit $code)"; }
say "seed set: the suite's #[sim_test(seeds = 16)], seeds 0 to 15, the set step 1 searched"
say "$line"
code=0
suite "$FIXED" "$WORK/out/fixed-seed" "$WORK/logs/fixed-seed.log" --tide-run-test "$TEST" --seed "$SEED" || code=$?
[[ $code == 0 ]] || { cat "$WORK/logs/fixed-seed.log" >&2; fail "the fixed revision failed seed $SEED (exit $code)"; }
say "seed $SEED on its own: passed"

step "4. Authority diff of demo/patches/metrics-report.patch"
grep -E '^[-+]pub fn ' "$HERE/patches/metrics-report.patch"
for side in base:"$FIXED" head:"$WIDENED"; do
  "$DRIVER" --crate-type=lib --crate-name=hits --tide-std "$STD" "${DEPS[@]}" \
    --out-dir "$WORK/out/interface-${side%%:*}" "${side#*:}" >"$WORK/logs/interface-${side%%:*}.log" 2>&1 </dev/null \
    || { cat "$WORK/logs/interface-${side%%:*}.log" >&2; fail "the ${side%%:*} revision was not admitted"; }
done
report="$("$DRIVER" --tide-diff "$WORK/out/interface-base/hits.interface.json" \
  "$WORK/out/interface-head/hits.interface.json" </dev/null)" || fail "the trust diff failed"
say "$report"
grep -qx '  ~ hits::serve (function) widened' <<<"$report" \
  && grep -qx '      + metrics: net' <<<"$report" \
  || fail "the trust diff does not show hits::serve widened with metrics: net"

step "Done: every step showed what it should. Logs and artifacts: ${WORK#"$ROOT/"}"
