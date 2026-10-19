#!/usr/bin/env bash
# C1: the facade std injected with --extern std=; capability programs build
# and run under both entry spellings, and ambient std calls do not compile.
source "$(dirname "$0")/env.sh"
build_facade
EX="$D18/examples"

# Entry A: `pub fn main(root: Root)` in a library crate, plus a generated bin
# crate (compiled against the real std) that mints Root and calls it.
tide_rustc --crate-type lib --crate-name caps "$EX/caps.rs"
cat >"$OUT/caps_entry.rs" <<'EOF'
fn main() -> std::process::ExitCode {
    std::process::Termination::report(caps::main(tide_std::__rt::root()))
}
EOF
rustc --edition 2024 --crate-name caps_main --out-dir "$OUT" \
  --extern "tide_std=$FACADE_RLIB" --extern "caps=$OUT/libcaps.rlib" \
  -L "dependency=$FACADE_DEPS" "$OUT/caps_entry.rs"
run="$(mktemp -d "$OUT/run.XXXXXX")"
first="$(cd "$run" && "$OUT/caps_main")" || fail "caps_main exited nonzero"
printf '%s\n' "$first"
second="$(cd "$run" && "$OUT/caps_main")"
[ "$(grep hashmap <<<"$first")" = "$(grep hashmap <<<"$second")" ] || fail "HashMap order differs between runs"
pass "entry A (library + generated entry): caps program ran; HashMap order equal across two runs"

# Entry B: `#[tide::main] fn main(root: Root)` in a bin crate.
tide_rustc --crate-type bin --crate-name caps_attr "$EX/caps_attr.rs"
out="$(cd "$run" && "$OUT/caps_attr")" || fail "caps_attr exited nonzero"
printf '%s\n' "$out"
touch "$run/FAIL"
code=0; (cd "$run" && "$OUT/caps_attr") >/dev/null 2>&1 || code=$?
rm "$run/FAIL"
[ "$code" = 1 ] || fail "caps_attr: Err from main should exit 1, got $code"
pass "entry B (#[tide::main]): ran; Err(..) from main exits 1"

# Plain `fn main(root: Root)` is what rustc rejects (E0580), which is why
# one of the two spellings above is needed.
expect_bin_error() {
  local log
  log="$(tide_rustc --crate-type bin --emit metadata "$1" 2>&1)" && fail "$1 compiled"
  grep -qE "$2" <<<"$log" || { printf '%s\n' "$log"; fail "$1: wanted /$2/"; }
  pass "$(basename "$1") rejected: $(grep -m1 -E '^error' <<<"$log")"
}
expect_bin_error "$EX/bad/main_arg.rs" 'E0580'

# Ambient std: none of these names exist in the facade.
expect_compile_error "$EX/bad/fs_read.rs" 'cannot find function `read` in module `std::fs`'
expect_compile_error "$EX/bad/thread_spawn.rs" 'E0433.*`thread`'
expect_compile_error "$EX/bad/instant_now.rs" 'E0599.*`now`'
expect_compile_error "$EX/bad/env_var.rs" 'E0433.*`env`'
expect_compile_error "$EX/bad/env_macro.rs" 'cannot find macro `env`'
# Root cannot be built by user code: its capability fields have private constructors.
expect_compile_error "$EX/bad/forge_root.rs" 'private|cannot construct'

# Not confined by the facade (checker's job): these must still compile here,
# so a change in that is noticed.
tide_rustc --crate-type lib --emit metadata "$EX/leaks.rs" || fail "leaks.rs no longer compiles"
pass "leaks.rs compiles (core::option_env!, include_str!, Path::exists, io::Error::last_os_error, std::__rt::root, f64::sin/to_bits): checker-owned"

# Cargo can inject the facade itself: `std = { package = "tide_std", path = .. }`.
probe="$EX/cargo-rename/Cargo.toml"
cargo build --quiet --offline --manifest-path "$probe" --target-dir "$OUT/cargo-rename" || fail "cargo rename probe"
log="$(RUSTFLAGS="--cfg probe_bad" cargo build --offline --manifest-path "$probe" --target-dir "$OUT/cargo-rename-bad" 2>&1)" \
  && fail "cargo rename probe: std::fs::read compiled"
grep -q 'cannot find function `read` in module `std::fs`' <<<"$log" || { printf '%s\n' "$log"; fail "cargo rename probe: wrong error"; }
pass "Cargo dependency renamed to std injects the facade: std::fs::Fs resolves, std::fs::read is E0425"
echo "C1 PASS"
