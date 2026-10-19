#!/usr/bin/env bash
# RUSTC_WRAPPER: cargo runs `facade-wrapper.sh <rustc> <args...>`. For crates
# named in TIDE_FACADE_CRATES (default: orbit_protocol), inject the facade as
# std; every other invocation only gains the facade's dependency search path,
# which dependents need to load it. Each injection is logged
# to TIDE_WRAPPER_LOG when set.
set -euo pipefail
rustc="$1"
shift
name=""
args=("$@")
for ((i = 0; i < ${#args[@]}; i++)); do
  if [ "${args[i]}" = "--crate-name" ]; then name="${args[i + 1]}"; fi
done
for c in ${TIDE_FACADE_CRATES:-orbit_protocol}; do
  if [ "$name" = "$c" ]; then
    if [ -n "${TIDE_WRAPPER_LOG:-}" ]; then
      kind="lib"
      case " $* " in *" --test "*) kind="test" ;; esac
      printf '%s %s\n' "$name" "$kind" >>"$TIDE_WRAPPER_LOG"
    fi
    exec "$rustc" "$@" --extern "std=$TIDE_FACADE_RLIB" -L "dependency=$TIDE_FACADE_DEPS"
  fi
done
# Dependents of a facade crate must find tide_std (E0463 otherwise).
exec "$rustc" "$@" -L "dependency=$TIDE_FACADE_DEPS"
