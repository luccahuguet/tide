# Tide review, fx-gcs.1 round 8 (Opus): narrow converge round

Narrow round over the two `conformance/run.sh` edits made for round 7's Codex findings (the error code taken only from the diagnostic header; `--trace` keeping the whole case path). One change, staged, not committed. Diff hash (`git diff HEAD -- . ':!.beads' ':!reviews' | sha256sum`): start `5982038bc80b164f2527acc9070f54f302c014a3768de26e01d89514d07d879a`, end `ec53741a6367deb4e0e9ae02e060f64d3b26afdd9b6df8f96d11467cbe7d1db3`.

## Fixed

- `conformance/run.sh:23-25` (`--trace`): the awk step now keeps everything after the ID, but the step building the ID/path pairs still pasted the path into a sed replacement (`sed "s|\$| ${f#"$CASES"/}|"`). An `&` in a path was silently dropped (`x&y.rs` became `xy.rs`), and a `|` made sed fail, dropping the case from the trace while `--trace` still exited 0. Replaced with `rules_of "$f" | while IFS= read -r id; do printf '%s %s\n' "$id" "${f#"$CASES"/}"; done`, which copies the path unchanged and still reads each case once.

## Left alone

- The error-parser edit (run.sh:67-73) is correct: awk's `match` takes the leftmost `:L:C: error[...]:`, the real header; the code comes only from `p[3]`, and `substr(p[3], 8, length(p[3]) - 8)` yields `E0308` from ` error[E0308]`.
- Unlocated errors and multi-line message text can only add an entry to rustc's error list, which fails a case; they never let a wrong case pass.
- `conformance/README.md` matches the script after both edits.

## Checks (rustc 1.96.0, offline)

- `conformance/run.sh`: exit 0 before and after (3 PASS, 2 AWAITS CHECKER: FX0520, FX0606); `bash -n` clean; `--trace` exit 0 with identical output.
- Parser probes under a directory whose name has a space: `::core::compile_error!(": error[E0308]: text"); //~ E0308` FAILs (rustc errors `[3:error]`, marked `[3:E0308]`); the same macro with `"x.rs:1:1: error[E0308]: text"` and `//~ error` PASSes; a real E0308 marked E0308 PASSes and marked E0599 FAILs.
- Trace probes with cases `sp ace dir/a b.rs`, `x&y.rs`, and `p|q\r.rs`: before the fix `&` was dropped and the `|` case went missing; after it all three print exactly.
