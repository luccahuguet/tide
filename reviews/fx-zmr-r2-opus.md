Model: claude-opus-5-5 · Effort: medium · Date: 2026-11-09

# fx-zmr round 2, Opus narrow re-check

## Verdict

**approve.** All eight of my round-1 findings are resolved. The changed lines introduce no new factual error.

## Scope

- **What I read:** `git diff 23b37e3 fx-zmr` (commit `d56520c`), checked only against my findings B1, B2 and N1–N6. I also read any changed line for a new factual error.
- **Sources:** I fetched these as text on 2026-11-09 with curl and ran nothing from them:
  - mbx `docs/limits.md` (main);
  - the Rust blog post on rust-lld in 1.90.0;
  - wild's `benchmarks/mac-mini-m4.md` and `ripgrep-time.svg`;
  - ovnfs `docs/checkouts.md`;
  - kache v2.0.0 `docs/getting-started/configuration.mdx`.
- **Local files:** `design/03-authority-concurrency.md` RUN-2.

## Per-finding status

- **B1, resolved.** `research/03-compile-speed.md:124` no longer says build scripts and user proc macros are mbx's "largest bypass classes". It now attributes the bypasses to what build scripts produce, which the limits page supports:
  - `cargo:rustc-link-search` with no kind bypasses ("Only `-L native=` and `-L dependency=` search paths are modeled");
  - "Most other `-C link-arg` values bypass";
  - generated files that hold checkout paths force checkout-specific keys, which restricts sharing.

  The author avoided "many" because the page gives no count. That is right. The sentence does not mention native libraries (`-l`, which build scripts emit). That makes it incomplete, but not wrong. The D13 caveat is kept.
- **B2, resolved.** The sentence at `:131` is now limited to wild, mold and LLD. The chart's labels are wild 0.11.0 +0%, mold 3.0.0 +95% and LLD 22.1.2 +209%, with GNU ld 2.46 at +1296%. With wild at about 26 ms, the three named linkers are all under 0.1 s, and GNU ld is excluded. The author's correction of my GNU ld estimate (about 0.38 s read off the bar, not 0.36 s) does not change the text.
- **N1, resolved.** The Cargo sentence is now conditional: "could cache … if the cache ships and its scope admits the renamed `std` dependency and toolchain proc macros". This matches the goal's "basic crates (no build scripts, proc macros, etc)" scope.
- **N2, resolved.** The LLD claim now cites the Rust blog post. The post says rustc uses rust-lld by default on `x86_64-unknown-linux-gnu` from 1.90.0 stable. Narrowing the text to that exact target is accurate.
- **N3, resolved.** The need-10 inference (`research/13-agent-fleets.md:367`) and row 10's Gap cell (`:444`) now agree: concurrent cache reuse plus one machine-wide compiler budget held by the driver.
- **N4, resolved.** `:413` limits the frozen-snapshot caveat to the default live-tree view and cites `ovnfs new --rev REV` [T08]. checkouts.md says that command "starts with REV's tracked files and index". T07 and T08 resolve to limitations.md and checkouts.md.
- **N5, resolved.** Both inferences now link RUN-2. They say rustc's source reads go through the driver's file loader, which matches RUN-2 ("rustc reads source through the driver's file loader"). Binding the key to the bytes the loader serves is correctly left as research inference.
- **N6, resolved.** The REFERENCES.md wild row and §11 now mention the preliminary Mach-O benchmarks. The page itself says "Debug info: Removed all debug info from the input files" and "these results should be seen as very preliminary". It does not claim Mach-O is supported.

## New findings

None. I also checked the changed lines that fix the Codex findings, for factual errors only:
- **`:107`, restored versus recompiled.** mbx limits, "A rebuilt workspace crate records its checkout", says: "a workspace crate that can be restored is restored, byte for byte. The problem appears when a crate's key includes its checkout path…" The text matches.
- **T14, kache `test-runner`.** kache v2.0.0's test-runner section matches every claim:
  - the runner uses the scheduler's pool, the same one compiles use;
  - with no thread count set, it sets `RUST_TEST_THREADS` to the slots it holds;
  - on Unix, `kache init` enables it unless passed `--no-shell`;
  - it "waits up to 30 minutes for slots, then runs without them".

DONE
