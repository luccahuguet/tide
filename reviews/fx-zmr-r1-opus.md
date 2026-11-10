Model: claude-opus-5-5 · Effort: medium · Date: 2026-11-09

# fx-zmr round 1, Opus docs review

## Verdict

**request_changes**, high confidence. The change meets the bead's Acceptance in structure and is well sourced. Every figure I checked matches its primary source. Two sentences misstate what their cited sources say. Both are one-clause text fixes, so after them this is an approve.

## Scope

Candidate `23b37e3` on base `0583ecb` (`git diff 0583ecb 23b37e3`): `REFERENCES.md`, `research/03-compile-speed.md` (new §11, §9 pointer, status line, sources), `research/13-agent-fleets.md` (new "Tooling prior art" T1/T2, two needs-table evidence cells, T01–T13 sources). I read AGENTS.md, GOVERNANCE.md's Merge Rule, the bead (`br --no-db show fx-zmr`), the author's report, the full diff, D0/D12/D13/D18 and the open questions in `design/01-design.md`, `design/03-authority-concurrency.md` RUN-2, and research/13's P5, needs table and implications.

I fetched these primary sources with curl on 2026-11-09 and read them as text, running nothing from them:
- mbx: benchmarks, how-it-works, limits, README;
- wild: README, FAQ, `benchmarks/ryzen-9955hx.md`, and the zed-debug, rust-analyzer and ripgrep chart SVGs;
- the Cargo goal page, the Fast Builds roadmap, rust-lang/goals#626 comments, and cargo#5931;
- ovnfs: README, limitations, checkouts, development;
- oh-my-pi #4627 with its comments, and PR #4629;
- the kache storage post and architecture page;
- the kernel overlayfs doc.

Acceptance check:
- **Five finds, each with a cited entry in the named file:** yes. mbx is in 03 and 13, kache in 03 (and 13), the Cargo cache in 03, wild in 03, and ovnfs in 13.
- **A REFERENCES.md row for each:** yes, five rows.
- **Every number traced:** yes, and I confirmed the ones I spot-checked.
- **Self-reported figures labelled:** yes.
- **No design/ change:** yes.
- **Docs-only review round:** this report is that round.

The four corrections to the bead's own claims are right, and I confirmed each one:
- #4627 was closed by the maintainer and PR #4629 was closed unmerged;
- ovnfs uses FUSE on Linux, NFSv3 on macOS, and SMB or WinFsp on Windows;
- wild's README still says incremental linking "isn't yet implemented";
- kache's zero-copy restores need reflinks: its post says that without them the store "has to be populated by copying… so the cache becomes a real second copy".

## Blocking

**B1. `research/03-compile-speed.md:124`: the text misdescribes mbx's bypass classes.** It says "Tide crates have no build scripts or user proc macros, which removes mbx's largest bypass classes." mbx's limits page says the opposite about both:
- build-script execution is "Eligible using Cargo's declared freshness inputs" ("mbx caches running build.rs, not only compiling it");
- proc macros are "Eligible on described Linux, macOS, and Windows hosts".

The bypasses the page does list come from what build scripts *emit*: `cargo:rustc-link-search` with no kind or `all=`, native libraries named on links, checkout paths embedded in generated output, and `-C link-arg` text. The page also ranks no bypass class as "largest". The sentence appears to conflate mbx with the Cargo goal, whose initial cache excludes "build scripts, proc macros, etc".

Smallest fix: "Tide crates have no build scripts, which removes the build-script outputs (unmodeled search paths, native libraries, checkout paths in generated code) behind many of mbx's bypasses." Keep the D13 sentence that follows.

**B2. `research/03-compile-speed.md:131`: "At ripgrep's size every current linker is under 0.1 s" is false against the cited chart.** The cited ripgrep chart also includes GNU ld 2.46 at +1296% of wild's time. At the text's own figure of about 26 ms for wild, that is about 0.36 s.

Smallest fix: "At ripgrep's size wild, mold and LLD all link in under 0.1 s."

## Non-blocking

**N1. `research/03-compile-speed.md:126`, the last sentence of inference 4, overstates its case.** "A Rust consumer's Cargo would cache a Tide crate with no work on Tide's side" is stronger than the evidence. The cache is unshipped. Its initial scope is "basic crates (no build scripts, proc macros, etc)". A Tide crate is built against a renamed `std` (`std = { package = "tide_std" }`) and uses the toolchain's proc macros. Whether such a unit qualifies is unknown. Suggest: "could cache a Tide crate it fetched from a registry, if that cache ships and its scope admits a crate built against a renamed `std`."

**N2. `research/03-compile-speed.md:131`: one factual claim has no URL.** "LLD, which Rust uses by default on x86-64 Linux" is true, but the file's status line promises that every factual claim has a URL. Add a citation, such as the Rust blog post announcing the rust-lld default on `x86_64-unknown-linux-gnu`, or drop the clause.

**N3. `research/13-agent-fleets.md:363` and `:438`: the needs table now disagrees with the text above it.** T1's inference says "Need 10 has two halves, and Tide's table states only the first", but row 10's Gap cell is unchanged. The table is research's own (not a design/ file), so extend that cell to match. For example, append: "and one compiler budget for the machine, held by the driver (T1)".

**N4. `research/13-agent-fleets.md:409-411`: the T2 inference reads as if ovnfs has no point-in-time base.** The text says "ovnfs itself does not claim one for files the user is rewriting". That is accurate for the default base, the live working tree. But ovnfs has `ovnfs new --rev REV`, which starts a checkout from a revision's tracked files, and snapshots or forks. Its limitations page also says a tracked file the user changes after open "is read from the repository's objects". One clause naming `--rev` would keep the lesson ("the base must be a point-in-time snapshot") without implying that ovnfs cannot provide one.

**N5. `research/13-agent-fleets.md:416` and `research/03-compile-speed.md:124`: a closer design reference exists for the torn-read point.** The texts point to "D18's driver snapshots the manifest with the sources" as the place to close the torn-read and key-before-compile issues. `design/03-authority-concurrency.md` RUN-2 is the closer reference: "rustc reads source through the driver's file loader", so the driver serves every byte rustc reads. Citing it would make the inference concrete. This is an optional cross-reference; I found no contradiction.

**N6. `REFERENCES.md`, the wild row's "Linux ELF and Wasm only" (also at research/03 §11):** this matches the README's feature list. The README also links a "Mac Mini - Mach-O" benchmark page, which suggests Mach-O work is under way. Optional precision; the author's report lists that page as unread.

## Reasoning

**The figures match their sources.** Every number I checked agrees with its primary source:
- mbx, aube: 2m12s; 1m7s (66.4–68.1 s) with 172 compilers at peak; 54.7 s (52.5–54.8 s) with 31 of 32 permits; cache hits 23 and 3,550.
- mbx, hk: 50.7 s, 48.2 s with 162 compilers, 39.2 s with 32.
- mbx, other rows: the commit, warm and edit rows, the first-edit costs of 24.9 s, 27.5 s and 8.0 s, the versions (1.17.0, 1.19.0, kache 0.26.3, Rust 1.94.0), and the page's caveats.
- mbx's limits page on the working directory, every dependent missing, and the opt-in `share_workspace_root` costs.
- Cargo goal, wording: the goal's wording ("basic crates", "will likely not be completed").
- Cargo goal, status updates:
  - the July rollback over `-L` arguments hitting OS limits;
  - "expected to release in 1.100.0";
  - the 1 October update on garbage collection and `cargo clean`;
  - the 14 October update on absolute paths and the design document.
- cargo#5931's "only immutable items (registry, git, not path)".
- The roadmap's "2026 and beyond" Wild row.
- wild's chart labels (+95%/+209%, +136%/+687%, +107%/+688%), with axis ranges consistent with the approximate absolute times.
- kache: the Firefox figures (13.5 GB, about 3 GB against 16.6 GB, 10.75x, 98.5%, one run, "vendor making a vendor claim") and the architecture quote.
- ovnfs: the 49k-file timings.
- oh-my-pi #4627: the 500/500 figure attributed to the reporter on fuse-overlayfs, the `index.lock` finding, the maintainer's quote, and the kernel's "undefined" sentence.
- "Sections 1–10 predate D18": `2d32580` is an ancestor of `a20bbae`.

**The research makes no design decision.** Each design-facing statement is marked **Inference** and recommends rather than decides. The D12 quote is verbatim, and RUN-2's rule that the branch cannot choose the linker is cited correctly. The recommendations in the author's report stay out of the repository, as the bead's checkpoint asked. The bead's question of whether D18 could reuse Cargo's cache is answered with sourced reasons. Changing D0–D17 to D0–D18 in REFERENCES.md is a needed leftover: before this change no row cited D18.

**Nothing in the repository contradicts the change.**
- The §9 pointer keeps research/03 internally consistent.
- research/13's P5, its implications section, and its open questions do not conflict with T1 and T2.
- Two problems the author reported as out of scope already existed before this change: REFERENCES.md's Cranelift row and research/13's header citing draft 3.4. Neither blocks this change.

B1 and B2 block only because the brief blocks on factual error. Both are text-only, so a narrow re-check covers them.

DONE
