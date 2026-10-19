#!/usr/bin/env bash
# C3: #[requires]/#[ensures] and loop-invariant lowering, from the facade
# prelude; checked in debug builds, erased (type-checked, never evaluated)
# in release builds.
source "$(dirname "$0")/env.sh"
build_facade
EX="$D18/examples"

tide_rustc --test --crate-name contracts_test "$EX/contracts.rs"
log="$("$OUT/contracts_test" 2>&1)" || { printf '%s\n' "$log"; fail "contract tests"; }
grep -E '^test ' <<<"$log"
grep -q 'test result: ok. 12 passed' <<<"$log" || fail "expected 12 passing contract tests"
pass "contract tests (debug): 12 passed"

tide_rustc -D warnings -C debug-assertions=off -C opt-level=1 --crate-name contracts_release "$EX/contracts.rs"
"$OUT/contracts_release" || fail "release build ran a violated clause"
pass "release build: violated requires/ensures/invariants erased, no warnings"

expect_compile_error "$EX/bad/invariant_no_attr.rs" 'invariant!\(\.\.\) must be the first statement'
expect_compile_error "$EX/bad/unbounded_forall.rs" 'unbounded forall'
echo "C3 PASS"
