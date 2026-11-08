#!/bin/sh
# Rust 1.96.0's merged runner spawns RUSTDOC_DOCTEST_BIN_PATH with a case index.
# Pin evidence: rust-lang/rust/1.96.0/src/librustdoc/doctest/runner.rs and doctest.rs.
set -eu
run_timed() {
    status=0
    /usr/bin/timeout --signal=KILL 10s "$@" || status=$?
    # A timeout must fail even when rustdoc treats a nonzero exit as an expected panic.
    if [ "$status" -eq 124 ] || [ "$status" -eq 137 ]; then
        : > "$E1_DOCTEST_TIMEOUT_FILE"
    fi
    exit "$status"
}
if [ "${RUSTDOC_DOCTEST_RUN_NB_TEST+set}" = set ]; then
    run_timed "$E1_DOCTEST_EXE" "$@"
fi
executable=$1
shift
if [ -f "${executable%/*}/libdoctest_bundle_2024.rlib" ]; then
    E1_DOCTEST_EXE=$executable
    RUSTDOC_DOCTEST_BIN_PATH=$0
    export E1_DOCTEST_EXE RUSTDOC_DOCTEST_BIN_PATH
    exec "$executable" "$@"
fi
run_timed "$executable" "$@"
