#!/usr/bin/env bash
# C5: math::sin/exp are the pinned libm; float::bits canonicalizes NaN;
# float::min/max order -0.0 below +0.0. Inherent f64 methods stay callable.
source "$(dirname "$0")/env.sh"
build_facade
libm_rlib="$(ls -t "$FACADE_DEPS"/liblibm-*.rlib | head -1)"
tide_rustc --crate-name floats --extern "libm=$libm_rlib" "$D18/examples/floats.rs"
"$OUT/floats" || fail "floats"
pass "float::bits canonical NaN; math::sin/exp == libm; min/max signed zeros"
echo "C5 PASS"
