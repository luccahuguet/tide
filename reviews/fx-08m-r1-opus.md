Model: claude-opus-5-5 · Effort: default · Date: 2026-11-11

Verdict: approve

Candidate: `8f06e9a` on base `8df2ee5`, branch `fx-08m`. I checked that the worktree is clean and that `git diff 8df2ee5 8f06e9a | sha256sum` is `b636caae253c151a1c0d23b235e03025c54dd6f4f7fd57992c31c2c543739737`. Read-only review. I ran no build or probe: every finding below comes from reading the source. I did not open anything under `trials/e1-checked/`.

What I read:
- AGENTS.md and GOVERNANCE.md;
- `br --no-db show fx-08m`, including the binding 2026-11-10 decisions comment;
- ROOT-4 (`design/03-authority-concurrency.md:360`), and `design/reference.md` §2.2 and §12.3;
- `research/22-json-codec.md`;
- the whole diff: both codec files, json.rs, root.rs, the macros, the build tests, both conformance cases, and the docs;
- the callers it relies on, `crates/tide-std/src/collections.rs` and `conformance/rustc/std/src/lib.rs`;
- the handoff, the completion report, the fix report and both Codex reports. I read the Codex reports only after my own pass.

Checked and found correct:
- **RFC 8259 grammar** (`codec.rs:257-304`):
  - leading zeros, `-` with no digits, `1.`, `.5`, `+1`, `1e`, `NaN` and `Infinity` are all rejected;
  - the number is lexed before Rust's wider parser sees it;
  - integer targets reject a fraction or exponent, and `-0` decodes into unsigned types;
  - floats are parsed once per type, and an infinite result is an error.
- **Strings:**
  - each escape run is validated as UTF-8, so invalid UTF-8 inside ignored values is caught too;
  - raw control bytes are rejected;
  - surrogate pairs combine, and a lone or mismatched surrogate is an error;
  - encoding escapes only the required set, in lowercase hex.
- **Structure:**
  - whitespace is exactly the RFC's four bytes, so a BOM is rejected;
  - the parser rejects trailing input, trailing commas, `[,1]`, `{,}`, a non-string key and a missing colon;
  - the `fresh` flag's first-item logic holds through nesting, through `variant_begin` and through the defaulted-tuple `ended` path.
- **Positions:** line and byte column, 1-based, computed once per error, so there is no repeated-prefix cost.
- **Limits:**
  - `open` enforces the 128-container limit, `skip_value` included;
  - every decode edge (top level, fields, newtypes, `Option`, `Box`, sequences, tuples and map values) goes through `Decoder::decode`, so transparent recursion is bounded.
- **Derives:**
  - hygiene: absolute `::std` and `::core` paths, and mixed-site or `__tide_codec_` names;
  - bounds: params, projections and `Default` for generic skip/default fields;
  - the serde case table matches `case.rs` for both variants and fields;
  - `rename` wins over `rename_all`, which renames variants only;
  - field `default` takes precedence over the container default, including with `skip`;
  - externally tagged enums work for all four variant shapes, plus `V()`, `V{}` and the empty enum;
  - a missing `Option` decodes to `None` through `__missing`, so a type alias behaves the same;
  - unknown keys are skipped but still checked;
  - a duplicate field is an error.
- **ROOT-4:**
  - no capability or handle type has a `Decode`, `Encode` or `Default` impl, and the orphan rule stops a user from adding one;
  - `skip` requires `Default`, and container `default` requires a `Self::default()`, which cannot produce a capability;
  - `conformance/cases/authority/root_no_decode.rs` and the `capability` and `capability_skip` cases in build.rs cover both paths.
- **Production/conformance equivalence:** both crates use one definition, through root.rs's `#[path = "codec.rs"] pub mod __codec` and `pub mod json`.
- **Decisions 1-10:** each is implemented as recorded.

## Findings

1. **Non-blocking: decoding a `HashMap` from untrusted JSON is open to precomputed hash flooding.**
   - Evidence:
     - `crates/tide-std/src/codec.rs:893` builds every decoded map with `S::default()`.
     - For Tide's default `RandomState` that is SipHash-1-3 with the fixed key (0, 0) (`crates/tide-std/src/collections.rs:23-43`).
     - The collections module tells callers to "Use `from_rand` for untrusted keys" (`collections.rs:4-6`), but `json::decode` cannot follow that advice: `from_rand` needs `&Rand`, and decode only calls `Default`.
     - An attacker who knows the key can compute, once and offline, keys that collide in the low bucket bits. That key set then turns every Tide program's `json::decode::<HashMap<String, _>>` into quadratic inserts: each insert probes an ever-longer chain, plus the extra `contains_key` probe at `codec.rs:881`.
   - Why it matters: this change adds the first entry point from untrusted bytes into that hasher. The hashing policy itself belongs to GLOBAL-5 and fx-yg5.1, not here.
   - Fix I'd want: one sentence in `crates/tide-std/src/json.rs`, beside the map bullets at `:39-40`. It should say that a decoded `HashMap` uses the fixed default key, so input from untrusted peers should decode into `BTreeMap`. Optionally, note the limitation as a comment on fx-yg5.1. No code change is needed in this bead.

2. **Non-blocking (test strength): nothing checks that the decode-call budget is released.**
   - Evidence: `crates/tide-std/src/codec.rs:164-172` decrements `calls` after each value. No committed test decodes more than 256 values in one `json::decode` call:
     - the largest are `tree(64)`, about 190 calls, and small vectors and maps (`conformance/cases/codec/json.rs:524-531`, `:687-711`);
     - the nesting tests go through `skip_value`, which does not count calls;
     - the 4000-float loop makes one decode call per value.
   - Consequence: deleting `self.calls -= 1` would pass the whole suite, yet then any input with more than 256 values would fail with "recursion limit exceeded". Codex's r2 checked this with 400 sibling `Link`s in a local probe, but that check was never committed.
   - Fix I'd want: one assertion in the shared fixture, such as `json::decode::<Vec<u8>>` of a 1000-element array, or a `Vec<Named>` of 300 elements, round-tripping.

3. **Nit: the call budget allows less than the documented 128-deep nesting for the usual recursive shape.**
   - Evidence: for `struct Node { v: u8, next: Option<Box<Node>> }`, each level holds three active calls: `Node`, `Option` and `Box` (`codec.rs:758`, `:773`).
   - Consequence: the 256-call budget at `codec.rs:27` therefore stops decoding at about 85 nested objects, well below the 128 containers that `json.rs:26` names first. `json.rs:27-28` does state the independent budget, so the docs are accurate, but a reader will take 128 as the practical depth.
   - Fix I'd want, either:
     - raise `DECODE_LIMIT` so that three wrappers per container fit 128 levels (for example 3 × 128 + slack); stack use per call is small, so this costs little;
     - or add a clause to `json.rs:27` saying that each `Option` or `Box` layer counts toward the 256.

4. **Nit: `variant_begin` builds its error text on every enum decode, success included.** `crates/tide-std/src/codec.rs:584-591` formats and joins every variant name before reading the tag, so each enum value costs an allocation proportional to the number of variants. Fix: build `expected` lazily in the three error branches, for example with a small closure.

5. **Nit: container `default` on a unit struct or newtype is accepted and ignored.**
   - Evidence:
     - `crates/tide-macros/src/codec.rs:141-144` accepts `default` on any struct;
     - the newtype branch at `:509` comes before the defaulted-tuple branch at `:513`, and `Fields::Unit` at `:505` never consults the default;
     - for a generic newtype, `:358` still adds a `Self: Default` bound that nothing uses;
     - serde rejects `#[serde(default)]` on unit structs.
   - The docs are also slightly off: `json.rs:10` lists "`default` (named field or struct)", and `:17` says a tuple struct's default fills missing trailing elements of "its array", which is untrue for a newtype, since it is not an array.
   - Fix I'd want: a spanned error for `default` on a unit struct and on a one-field tuple struct, with a negative case in build.rs. Alternatively, document that the key is ignored there and skip the needless bound.

6. **Nit: a missing `Box<Option<T>>` field is an error, where serde gives `None`.** `crates/tide-std/src/codec.rs:771-775` keeps the default `__missing`. serde's missing-field deserializer reaches `Option` through `Box`. Fix: `fn __missing(input, field) { T::__missing(input, field).map(Box::new) }`.

7. **Nit (doc): "The mapping is serde's" does not say that decoding is stricter than serde_json.**
   - Evidence: `crates/tide-std/src/json.rs:5`. Decoding deliberately rejects two forms serde_json accepts:
     - a named struct written as an array (`conformance/cases/codec/json.rs:96`);
     - a unit variant written as `{"Z":null}` (`:103`).
   - Both rejections are tested and follow the research's recommendation, but the module doc never says so. A serde user reading `json.rs:5` will expect both forms to decode.
   - Fix I'd want: one sentence under "Decoding" saying that only the forms above are accepted, and naming those two.

No blocking findings. Findings 1 and 2 are the ones I would fix before landing, since each is a one-line change. The rest can be fixed or rebutted.

DONE
