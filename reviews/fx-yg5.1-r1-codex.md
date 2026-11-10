Model: gpt-6.1-sol · Effort: xhigh · Date: 2026-11-09

## Verdict

**approve — high confidence for the scoped collection surface.** No blocking correctness, soundness, admission, or production/conformance ownership defect found in `git diff b4f1302 e581f25`. No override.

The change makes the existing collection fixture executable through one production implementation. Its fixed constructors use a fixed builder throughout; its Rand-gated constructor uses the pinned standard library's OS-seeded hashing path. This verdict covers candidate `e581f25`, not the forthcoming rebase onto the parallel change.

## Scope

Read:

- `AGENTS.md`, `GOVERNANCE.md`, the complete candidate diff and all nine touched files.
- All production std source files, the conformance std source and manifest, relevant existing collection/Pure/borrow fixtures, `conformance/run.sh`, and `conformance/README.md`.
- `tide-admit/src/policy.rs`, `pure.rs`, `crates.rs`, `walk/mono.rs`, the relevant admission/obligation handling in `walk.rs`, all of `tests/build.rs`, and related admission/backstop tests in `tests/driver.rs`.
- `design/reference.md` §§1, 6.4, 6.6, 6.9, 12.3; `01-design.md`'s tier table, D7 and D18; `02-memory.md` §9; `03-authority-concurrency.md` GLOBAL-5; `trials/e1-checked/protocol.md` §§2 and 10.
- Read-only `br --no-db show` results for `fx-yg5.1`, `fx-yg5`, and `fx-nkv`.
- The implementation report, final hash manifest, workspace/conformance logs and source-free E1 summary. Ignored the implementer's unofficial review verdicts.
- Installed pinned rust-src: `std/src/hash/random.rs`, Linux `std/src/sys/random/linux.rs`, and the relevant native HashMap/HashSet constructors and trait implementations.

Independent checks used an archive of `e581f25` at `/tmp/fx-yg5.1-r1-codex.wWwAUh`; no repository/worktree or Beads writes. No Cargo command ran. Every rustc invocation was serial, prefixed with `nice -n 19 timeout 60`, with closed stdin, and preceded by the brief's quiet-window check; each was permitted to start.

1. **Runtime probe:** `supervisor_probe.rs` imports the archived, unchanged `collections.rs` with `#[path]` and supplies a local Rand stand-in solely to exercise the module. Compiled with `/home/lucca/.rustup/toolchains/nightly-2026-08-09-x86_64-unknown-linux-gnu/bin/rustc --edition=2024 supervisor_probe.rs -o supervisor_probe`. Exit 0. Two separate executions exited 0, produced identical 33,812-byte fixed-order stdout (SHA-256 `084bbf3367d30bc51bb6a70bbb14ac84d7042aa6a7ebc720f5c8e5eda1e35c89`), and different seeded fingerprints. Assertions cover a handwritten user `Hash` impl, entry mutation/removal, growth/removal/reinsertion, Clone/PartialEq/Debug/Index, a custom builder without Clone or Debug, const `with_hasher` without key/build-hasher bounds, nonzero-capacity maps/sets, default/collect/array construction, and seeded map/set lookup. This proves module behavior, not driver admission or capability unforgeability.
2. **Stable compatibility probe:** same source compiled with `/home/lucca/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/bin/rustc --edition=2024 supervisor_probe.rs -o supervisor_probe_stable`. Compile and execution exited 0; all assertions passed. Its output happened to match the nightly result; no cross-toolchain ordering guarantee follows.
3. **Inference probe:** `inference_probe.rs` selects either Tide's or Rust's map with `--cfg tide`. Both pinned-rustc metadata invocations (`--edition=2024 [--cfg tide] --emit=metadata inference_probe.rs -o inference_{tide,rust}.rmeta`) exited 1 with E0283 at unannotated `HashMap::default()` followed by insertion. This is Rust's existing unresolved-hasher inference behavior, not a Tide regression. Successful new/array/annotated-default/custom-builder routes are covered by probe 1.
4. A normalized signature comparison against the base stub found all **33 public function signatures unchanged**, ignoring whitespace and the unused parameter's spelling. Every existing explicit impl header remains; the added explicit Debug impl replaces RandomState's derived Debug. All nine archived changed files match `final-manifest.json` hashes.

The supplied logs record 49 passing workspace tests and 198 PASS / 25 AWAITS CHECKER / zero FAIL conformance results. I inspected these records and their matching source identity; I did not independently rerun them. The source-free E1 summary records zero unresolved-collections failures for A1/A6/A7/A8/B5/B7/B9. Its other failures remain, and its overall run was not an E1 gate pass.

## Blocking

None. No corrective implementation patch is required for approval of this candidate.

## Non-blocking

**N1. Make the partial HashMap/HashSet API explicit in public availability docs** — `crates/tide-std/src/lib.rs:15`, `crates/tide-std/src/collections.rs:2`.

The implementation intentionally follows the old fixture's listed methods. It does not expose all stable Rust collection APIs: for example, `map.capacity()` and set union operations are unavailable. The production availability summary now lists collections without stating that limit, while reference §1 describes their eventual Rust API. A reader can reasonably try an ordinary Rust method and hit a missing-method error.

Smallest improvement: add one sentence saying that HashMap/HashSet currently expose the methods listed in this module, with the remaining Rust methods still unavailable. Do not expand the implementation merely to address this wording. This is a documentation clarification, not a blocker or a later-tier finding.

## Decisions

1. **Share through `root.rs`: agree.** The production module is included by both std crates at the same canonical paths. Deleting the duplicate placeholder module satisfies fx-nkv's single-runtime-owner requirement and prevents signature drift. The signature comparison and existing conformance evidence support this choice. Keep it.
2. **Store a pristine DefaultHasher and clone it: agree.** The pinned implementation constructs SipHash-1-3 with keys `(0, 0)` in `DefaultHasher::new()`. Cloning that untouched state implements BuildHasher's requirement to start every hash identically. There is no need for a new dependency, locally implemented keyed hashing, or an enum branch. Pin dependence is documented and consistent with the existing allowlist/toolchain model. Keep it.
3. **Use Rust RandomState to obtain seeded hashing: agree for the supported host and this slice.** The source path is `RandomState::new()` → thread-local keys initialized by `hashmap_random_keys()` → Linux OS randomness; `build_hasher()` embeds those keys in DefaultHasher. Subsequent states increment one cached key. This is real OS-seeded hashing, not fresh independent OS entropy on every constructor call. Linux's native map-key path uses its nonblocking/early-boot entropy policy; it is not a general cryptographic random-draw API. That is appropriate to the requested native hash-table protection. Keep it rather than adding a separate entropy implementation.
4. **Use the existing Rand token without adding a draw/backend API: agree.** Production Rand is currently an unforgeable capability identity. Taking `&Rand` gates the OS operation; retaining only the resulting key carries data, not authority. Hiding RandomState's Debug fields avoids accidental disclosure through builder formatting and preserves the stub's Debug trait. When simulated Rand becomes executable, this constructor must dispatch through that backend; it must not keep drawing OS keys for a simulated capability. There is no executable production simulator in this candidate, so that missing backend does not block this slice.
5. **Implement the listed stub signatures rather than every Rust method: agree with the bounded milestone interpretation.** The bead explicitly names stub equivalence and the selected methods/traits; all those signatures and bounds are retained. The wrappers should not gain Deref escape routes or unrelated APIs. This is a partial implementation of the reference's eventual Rust API, not a language decision to remove the unlisted methods; N1 makes that distinction clearer.
6. **Reuse generic-argument Pure handling and existing admission owners: agree.** No Pure table entry was added or needed. Both wrappers expose key/value/builder type arguments to the existing structural decision. A seeded RandomState holds only key data and is correctly Pure. The nine negative fixtures cover capability-bearing keys, values and builders across every named collection. Keep the shared decision mechanism.

## Reasoning

**Fixed hashing reaches every constructor family.** `HashMap::new()` and `HashSet::new()` call their generic Default impls with Tide RandomState selected by the type defaults. Their capacity constructors explicitly supply the same builder. FromIterator's native destination is the field's `HashMap<K,V,S>` or `HashSet<T,S>`, so `S` remains Tide RandomState when omitted; Rust's randomized default is not substituted. Array conversion collects into that same wrapper type. Extend preserves the existing builder. Entry and iterator types are native delegates and introduce no entropy. Determinism still requires deterministic key hashes, the same construction/mutation history, and the same pinned toolchain/target; capacity or insertion strategy may change the unspecified order.

**Admission does not open ambient randomness.** The added Tide collection rule admits the new public owner. The DefaultHasher type/inherent rules admit a deterministic hasher. Existing explicit denials of Rust RandomState's `new` and Default impl remain ahead of native collection allowances. The new production-driver negative tests target both map and set generic defaults, whose hidden native Default use is caught by the unchanged mono-item backstop. Local user Hash/BuildHasher bodies still undergo normal checks; their obligations and any denied instances reached through std are not exempted by the collection namespace rule. The independent user-Hash runtime probe passed, while driver admission for that particular probe was not rerun.

**API and ownership are consistent at the scoped surface.** Methods preserve borrowed lookup bounds and lifetimes, mutable and owned iterator forms, unconstrained non-hashing accessors, const `with_hasher`, entry variants, copied-reference Extend, and the requested standard traits. `S = RandomState` and array/new inference retain the base stub's shape. Native storage preserves auto traits, variance and drop behavior; no unsafe implementation or extra dependency was introduced. BTreeMap/BTreeSet/VecDeque/BinaryHeap and the pre-existing ancillary collection exports remain native re-exports. No broken caller or orphaned duplicate collection definition was found.

**Tests are adequate for these delegates and the changed hashing boundary.** The committed production build test runs every named collection and the wrapper routes; it compares independent processes and separately rejects ambient generic defaults. The conformance suite checks the same implementation with stable and pinned compilers and checks negative Pure cases. My probes add independent coverage for user-defined hashing, nonzero capacity, mutation history and weak trait-bound edges. Fixed-key tables intentionally lack randomized hash-flood protection; the documentation correctly directs untrusted keys to `from_rand`. I found no evidence of E1 task material in the added generic fixtures. I did not open held-out task/reference/writer sources to establish textual originality.

Not checked independently: full Cargo/conformance/driver/E1 reruns, unsupported targets, or the forthcoming rebase. Full-suite results above remain attributed to the supplied logs.
Risk: fixed-key tables require the documented Rand-seeded route for untrusted keys; this approval applies only to the frozen candidate and its supported partial surface. DONE
