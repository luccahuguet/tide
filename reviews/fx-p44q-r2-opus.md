Model: claude-opus-5-5 · Date: 2026-11-16 · Candidate 1f6ef58

# fx-p44q, Opus review r2 (docs review round, read-only)

## Verdict

request_changes. Two blocking findings. Both are one-line text labels in v0-tier text that this change edits.

## Scope

- Diff `git diff 502650c...1f6ef58`. SHA-256 verified: ea4991de25ad90bc4f85123a55a8ead1504aa1c8d1bac9ab8812a73cb3513066.
- Contract: the bead's Scope and Acceptance, and the owner comments of 2026-11-15 (the Achilles carry, and no claim of safety against a malicious dependency).
- Files read:
  - all of `design/reference.md`;
  - the companion's header, 14, 15.1–15.3, 17 and 21, plus the diffed hunks of 15.4, 18, 19 and 20;
  - README's lead, Status (lines 60–200, 355–450) and the tier list;
  - `reviews/fx-p44q-audit.md`;
  - D11 and the tier table of `design/01-design.md`.
- Labels checked against `crates/`:
  - `tide-std`: `lib.rs`, `sync.rs` (Arc/Weak API), `sim.rs` (world setup, `Record`, `Sink`), `fs.rs`, `par.rs`, `collections.rs` (`unmodeled("Rand")`), `rand` and `process` (identity only), and a search for `Pcg64`;
  - `tide-admit`: `interface.rs` and `interface/diff.rs` (what the trust diff covers), and `invocation.rs` (replay flags);
  - every `FX` code the core teaches, checked for a `Code::FxNNNN` use outside `diag.rs`. Only `FX0516` and `FX0920` have none, and both sit under planned surfaces (`tide.toml`, `tide.seal`).
- Mechanical checks:
  - `scripts/count-tokens.sh design/reference.md` gives 24931. This is within D14's 25,000, and the companion's "Core count: 24,931" matches.
  - `cmp reviews/2026-11-15-achilles-opus.md /home/lucca/.cache/eon-agents/tide-achilles/report.md`: byte-identical.
- Not run: no build, test or conformance run (docs-only round).

## Blocking findings

### B1. `design/reference.md:346-350`, also `:1003`: `Pcg64` is taught as available but does not exist

- **What's wrong:** The Rand row (line 340) is now labelled planned. The bullet beneath it is not. It still teaches `Pcg64` as "an ordinary `Pure` value with `&mut self` methods (`next_u64`, `gen_range`, `gen_bool`, `shuffle`)" and says "`Pcg64::seed_from_u64(n)` needs no capability". The 12.3 table also lists `Pcg64` at `std::rand`.
- **Evidence:**
  - Nothing under `crates/` defines `Pcg64` or `seed_from_u64`; a grep finds no match.
  - `crates/tide-std/src/lib.rs:283-289`: `std::rand` holds only the identity-only `Rand`, "It has no draw methods yet."
- **Why the row's label does not cover it:** The label covers `rand.rng() -> Pcg64`. But the bullet explicitly presents `Pcg64::seed_from_u64(n)` as usable without the capability. An agent writing a seeded PRNG will reach for it and hit a missing item.
- **Audit:** the audit has no row for `Pcg64` apart from row 41 (Rand draws), so it was missed rather than judged.
- **Fix:** add `(planned)` after `Pcg64` in that bullet. That costs about 3 tokens, and 69 remain in the budget.
- **Why it blocks:** The fix changes v0-core text in `design/reference.md`, which this change edits. It falls under Acceptance 1: "every taught v0 surface … labelled with its tier". It is the same class as Codex r1's `RandomState::from_rand` finding.

### B2. Companion `design/reference-companion.md` 14.1 (the `FX0520` row and the `world.http` peer row): planned fixes are taught without labels

- **What's wrong:**
  - The `FX0520` row gives `AtomicU64::new(0, tasks)` as the replacement for `core::sync::atomic`. Atomics are planned: audit row 68, core 7.4 line 723 and core 7.4 line 742 all say so.
  - The row "An `Rc<Cell<u32>>` counter captured by a `world.http` peer → `Shared::new(0u32, &sim.tasks)`; peers are `Send` (17)" teaches `sim.world.http`, which is planned. This change itself moved `sim.world.http` into companion 17 because it is planned.
- **Inconsistent with 14.2:** the change labels the planned rows of 14.2 (`rand.gen_range`, `proc.command`, `proc.exit`). The habit table right above it is left unlabelled.
- **Audit:** audit row 90 marks 14.1 "implemented — as rows 18–80". That holds for the codes but not for the fixes this table teaches.
- **Fix:**
  - Append `(planned)` to `AtomicU64::new(0, tasks)` in that cell, and mark the `world.http` row planned.
  - Correct audit row 90 to partial.
  - These are companion-only edits, outside the core budget.
- **Why it blocks:** The fix changes v0 text in the companion, which this change edits and whose labels it defines ("**(planned)** and **(partial)** mean what they mean in the core"). It falls under Acceptance 1.

## Non-blocking findings

- **N1. `design/reference.md:40-41`: the Arc/Weak gap list reads as complete but is not.** The label reads "(both partial: no `ptr_eq` or unsized payloads yet)". `crates/tide-std/src/sync.rs:18-137` also lacks:
  - `Weak::ptr_eq`;
  - `AsRef` and `Borrow` impls;
  - `unwrap_or_clone`;
  - `new_cyclic`.

  Some Rust constructors (`From`, `Default`, `Weak::new()`) are absent by design, because they cannot take `&Tasks`. Suggest "(both partial, e.g. no `ptr_eq` …)", or simply "(both partial)".
- **N2. `Net::get` is used in core examples without a local label.** The places are:
  - 7.1's failure bullet (`s.spawn(move || Ok(net.get(u, deadline)))`);
  - the `fetch_both` example (lines 644-653);
  - 7.2's "Operation deadlines" bullet;
  - 6.8's Time bullet ("a fault-free `Net::get` 2–20 ms").

  The surface is labelled where it is defined (6.1, 6.7), so a reader can find out, but `fetch_both` is a copyable program that does not compile today. If the budget allows after B1, add a short "(`Net::get` is planned)" near `fetch_both`.
- **N3. `design/reference.md:515-516`: the scope of "planned" is ambiguous.** The text reads "`sim.faults` and `sim.node(name)` (faults and crashable nodes; HTTP peers: companion 17; planned)". The trailing "planned" can be read as applying only to HTTP peers. Suggest "(faults, crashable nodes and HTTP peers: companion 17; planned)".
- **N4. `design/reference.md:159`: "(partial: rustc's error today)" is placed mid-clause and reads awkwardly.** Suggest "(today rustc's own error)". The meaning is right: README Status says "Wrong-input elision keeps rustc's region error".
- **N5. Companion 14.1 marks fixes `safe` and "error with fix", but no fix exists yet.** The core labels fixes planned once, at 0 (line 24), and the `tide` commands once, at line 15. The companion's header carries over the meaning of the labels, not these blanket statements. Suggest one clause in the companion header saying the core's blanket labels (fixes, `tide` commands) apply here too.
- **N6. `design/reference.md:1019`: "reserved" is vague.** The sentence "A path whose items this reference marks planned is reserved" could be read as a new rule about names. "… does not exist yet" says the factual thing.
- **N7. README tier list (around line 640) leaves out the blocking runtime.** It lists "structured tasks on blocking and thread runtimes" as v0, and its "largest parts not built yet" list does not name the blocking runtime. "Largest" hedges this, and reference 7.5 labels it. The sentence "`design/reference.md` labels each one **(planned)**" is also slightly loose:
  - the incremental front end is not taught in the reference;
  - 10.3 uses a lead-in "Planned." rather than the bold label.

  Nit.

## Reasoning

**README** claims only what v0 delivers:
- The lead is scoped to "the part of a program written in Tide".
- "No accidental ambient authority" keeps the malicious-dependency caveat the owner required on 2026-11-15, and says packages and dependencies are planned.
- "Tests replay" is restricted to `--tide-run-test` and `--replay` on the same machine. This matches README Status (lines 398-450) and `invocation.rs:185-190`.
- The trust-diff bullet claims declared authority and `unsafe` holes only. This matches `interface/diff.rs:1-4` and `interface.rs:22-23`; the "unsafe sites" wording is accurate.
- The speed figure is now stated as D12's target.
- `dbg!` is correctly removed from the unimplemented list: `tide-std/src/lib.rs:16-17` provides it.
- I found nothing understated.

**Labels in the core** are true wherever I sampled:
- HTTP, UDP, resolve, Rand draws and Proc are absent.
- The simulated `Rand` panics through `unmodeled("Rand")` (`collections.rs:37`).
- Sampled-clause unchecked printing is planned, per `lib.rs:190-192`.
- `span` and `Secret` are absent, per `lib.rs:240-245`.
- The faults and nodes are absent; `Faults` is identity-only.
- The 1 ms simulated file timing exists (`fs.rs:10`).
- `world.args`, `env_var`, `stdin` and `file` take the types companion 17 states (`sim.rs:88-119`).
- `Record` and `Sink` have the stated fields (`sim.rs:131-143`).

**Audit rows** sampled as implemented and confirmed: 18, 31, 36, 40, 43, 55, 56, 61, 72, 77, 78, plus the FX-code sweep. The two gaps above are omissions in the audit (no `Pcg64` row; row 90 over-claims), not mislabelled rows.

**Language decisions** are unchanged:
- Every core edit adds a label, or moves text without changing what the language decides.
- The 6.7 HTTP paragraph and the two 6.8 rows reappear verbatim in companion 21 and 17. Companion 21 adds only the `Net::get` signature already given in 6.1.
- The 7.4 and 10.1 rewordings keep the same rules. Over-budget clauses still count as unchecked, not failed; only the reporting is planned.
- Cross-references resolve: core 6.7 → companion 21; core 6.8 → companion 17 (HTTP peers now open its table); companion 15.2 → 17; 14.1 `(17)`; companion 19 → bead `fx-ip4x`.
- 01-design's v0 tier still lists everything the labels call planned, so labelling rather than deleting was correct.

**Budget:** 24,931 tokens, matching the companion's count. B1's fix fits in the remaining 69 tokens; N2's optional note may not.

**Writing** is plain and consistent with the reference's existing label style (provisional). N3, N4 and N6 are small clarity nits.

Both blocking fixes are text-only labels in tiers this change edits. Per AP-PROOF-001, text-only fixes need no re-check, but under the Merge Rule I am the finder of record and will confirm them if asked.

DONE

## Author response (tide-lead, 2026-11-16)

tide-lead made these fixes, text only, because the builder's sclaude session was rate-limited until 01:20Z.
- B1: `Pcg64` is labelled (planned) in 6.1's bullet and the 12.3 table.
- B2: companion 14.1's `AtomicU64::new(0, tasks)` fix and its `world.http` peer row are labelled (planned). Audit row 90 is now partial (totals 54 / 14 / 34 / 8).
- Folded in: N1 ("e.g."), N2 (a `Net::get` planned note before `fetch_both`), N3 (scope of "planned"), N4 (wording), N5 (companion header clause), N6 ("does not exist yet"). N7 is left as a nit.
- Core recount: 24,961 tokens (`scripts/count-tokens.sh design/reference.md`); the companion's count line matches.
These fixes are text-only, so under AP-PROOF-001 they need no re-check.
