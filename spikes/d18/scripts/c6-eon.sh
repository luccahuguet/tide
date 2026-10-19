#!/usr/bin/env bash
# C6: Eon's orbit-protocol compiled against the facade (via RUSTC_WRAPPER, no
# source edits), then Orbit built unchanged against it, and the tests run.
source "$(dirname "$0")/env.sh"
"$D18/scripts/fetch-eon.sh"
build_facade
EON="$D18/eon"
TARGET="$EON/target/tide"
export RUSTC_WRAPPER="$D18/scripts/facade-wrapper.sh"
export TIDE_FACADE_RLIB="$FACADE_RLIB" TIDE_FACADE_DEPS="$FACADE_DEPS"
export TIDE_WRAPPER_LOG="$OUT/wrapper.log"
: >"$TIDE_WRAPPER_LOG"
cargo_eon() { cargo "$@" --manifest-path "$EON/Cargo.toml" --target-dir "$TARGET"; }

# Force orbit-protocol (and its dependents) to recompile through the wrapper.
cargo_eon clean --quiet -p orbit-protocol

log="$(cargo_eon build -p yazelix-orbit 2>&1)" || { printf '%s\n' "$log"; fail "orbit build"; }
grep -E '^(warning|error)|Compiling (orbit|yazelix)|Finished' <<<"$log" || true
grep -q '^orbit_protocol lib$' "$TIDE_WRAPPER_LOG" || fail "orbit_protocol was not compiled with the facade"
pass "orbit-protocol built against the facade; yazelix-orbit (bin) built unchanged against it"

# orbit-protocol's unit tests: the lib compiled with --test, under the facade.
# Its integration test (tests/management.rs) uses the real std and links the facade-built lib.
log="$(cargo_eon test -p orbit-protocol 2>&1)" || { printf '%s\n' "$log"; fail "orbit-protocol tests"; }
grep -E '^test result|Running' <<<"$log"
grep -q '^orbit_protocol test$' "$TIDE_WRAPPER_LOG" || fail "unit tests were not compiled with the facade"
pass "orbit-protocol unit tests (facade) and integration tests (real std over facade lib) passed"

cargo clean --quiet -p orbit-protocol --manifest-path "$EON/Cargo.toml" --target-dir "$EON/target/tide-it" 2>/dev/null || true
# The integration test crate under the facade too (own target dir: cargo's
# fingerprint does not see the wrapper's crate list).
log="$(TIDE_FACADE_CRATES="orbit_protocol management" cargo test -p orbit-protocol --test management \
  --manifest-path "$EON/Cargo.toml" --target-dir "$EON/target/tide-it" 2>&1)" \
  || { printf '%s\n' "$log"; fail "management test under facade"; }
grep -q '^management test$' "$TIDE_WRAPPER_LOG" || fail "management test was not compiled with the facade"
pass "integration test crate also builds and passes under the facade: $(grep -m1 '^test result' <<<"$log")"

# Orbit's own unit tests (bin crate, real std) against the facade-built protocol.
log="$(cargo_eon test -p yazelix-orbit --bins 2>&1)" || { printf '%s\n' "$log" | tail -30; fail "orbit unit tests"; }
pass "yazelix-orbit unit tests: $(grep -m1 '^test result' <<<"$log")"
echo "C6 PASS"
