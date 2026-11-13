# Strict, tagged codecs without serde (fx-v4c9)

Research and proposal, 2026-11-13, drafted by GPT-6.1 Sol. **[repo evidence]**
Baseline: Tide `09643272093860b7c95a8b66f911068a7c2ab001`, initially clean,
branch `fx-v4c9`. The note was drafted as a proposal; the coordinator's
decision is in §10, and D6 states it. No builds ran, the author wrote no
Beads state, and nothing under
`trials/e1-checked/` was opened or searched.

Evidence labels throughout:

- **[repo evidence]**: inspected Tide source or design at that baseline.
- **[Eon evidence]**: inspected the separate Eon checkout identified below.
- **[report evidence]**: the named eon-desk reports; their assessments are not
  fresh measurements.
- **[web evidence]**: linked primary documentation read 2026-11-13; crate
  documentation uses versioned URLs where available.
- **[inference]**: analysis, a proposed choice or contract, or untested
  implementation guidance. Examples under that label are proposed behavior.

## 1. Question and smallest contract

**[report evidence]** The eon-desk reports
`/home/lucca/.cache/eon-agents/eon-desk/tide-in-eon-report.md` and
`tide-beads-from-eon.md` ask for unknown-field rejection and internally
tagged enums, without bringing serde's lifetime-heavy APIs into Tide.
Their recommendation is to use Eon as a reference workload, not adopt Tide
in Eon yet; E1 and the first demo take priority. Bead `fx-v4c9` asks for a
design decision with research, a tier, and possible implementation follow-ups.

**[inference] Recommendation:** add two opt-in keys to the existing
toolchain-owned derive helper in **v1**:
`#[codec(deny_unknown_fields)]` and `#[codec(tag = "type")]`. A protocol
author derives `Decode`; when `json::decode::<T>(&bytes)` sees a forbidden
field or invalid tag it returns `json::Error`. When both traits are derived,
`json::encode(&value)` emits the selected representation. No input borrow
escapes, and no capabilities are acquired. The cheapest eventual falsifier
is a tagged enum decoded with its tag last and an extra key: accept the
valid permutation, reject the extra key. A round trip alone cannot prove
strictness or compatibility with an independently written peer.

## 2. What Tide already owns

**[repo evidence]** `design/01-design.md` owns language decisions and says
chapters must not contradict it. D6 owns the closed derive set and
toolchain-owned helper attributes; D18 keeps programs valid Rust.
`design/reference.md` §2.2 fixes JSON-only v0 entry points, four helper keys
(`rename`, `rename_all`, `default`, `skip`), absent `Option` fields becoming
`None`, and externally tagged enums. The companion's tier appendix explicitly
leaves `deny_unknown_fields` outside v0 and not yet tiered by 01.

**[repo evidence]** `design/03-authority-concurrency.md` §11 RUST-1 excludes
direct use of macro-driven/lifetime-heavy Rust APIs unless wrapped by a
synchronous adapter; 01's scope also excludes serde's `'de`. This is an
interop boundary, not a prohibition on familiar schema annotations.
`design/04-contracts.md` uses JSON for artifacts/examples, not a second
codec API. Searching the design chapters found no public `std::codec`
entry point. In production, `root.rs` exports `std::json` and the two traits;
`std::__codec` is hidden derive support.

**[repo evidence]** The codec already exists (fx-08m):

| Owner | Inspected behavior |
| --- | --- |
| `crates/tide-macros/src/codec.rs`: `attrs`, `fields`, `decode_fields` | Both derives share helper parsing; only four keys accepted; `deny_unknown_fields` explicitly rejected as outside v0. Renamed field/variant collisions fail. Known duplicates fail; unknown fields call `skip_value`. |
| `crates/tide-std/src/codec.rs`: `Decode`, `Decoder` | `Decode: Sized` has no lifetime parameter; hidden decoding methods take a transient `Decoder<'_>`. Owned strings and collections are constructed. Parsing enforces depth 128 and active codec calls 256. |
| `crates/tide-std/src/json.rs` | `encode` returns `Vec<u8>`; `decode` returns `Result<T, Error>` and checks that only whitespace follows the value. Errors retain line and byte column. A skipped field's incoming key is treated as unknown. |
| `conformance/cases/codec/json.rs`: `keys` | Exercises ignored unknown values (including repeated unknown names), rejected repeated known fields, defaults, malformed skipped values and nesting. |
| 03 ROOT-4; `conformance/cases/authority/root_no_decode.rs` | Capabilities cannot be reconstructed by decoding. |

**[repo evidence]** `research/22-json-codec.md` records why the current
implementation chose owned code and hidden direct JSON methods rather than
serde visitors or an intermediate value tree. Its historical statements
about missing code are superseded by the files above.

**[inference]** The least change is to extend that owner, not introduce
reflection, an alternate parser, or a second policy at every decode call.
No production caller, fixture, export, configuration or dependency changes
in this design-only bead. Later implementation must reach both derives,
the shared engine, conformance, and their user documentation together.

## 3. Eon actually uses two different mechanisms

**[Eon evidence]** Read-only checkout: `/home/lucca/pjs/eon/eon`, HEAD
`7d1f921427a6c094c85f4a4dd9688f92b2356db4` before and after inspection.
Only `.beads/issues.jsonl` was dirty at the initial status check. The desk's
`mobile-brief.md` identifies this absolute repository path; its assessment
uses relative paths and an older Eon baseline, `b30e80f`.

**[Eon evidence]** `runtime/crates/eon-workspace-protocol/Cargo.toml` has
no dependency section; `src/lib.rs:1-9` forbids unsafe and exposes `v42`.
`src/v42.rs:1-25` describes a **binary** protocol: a 12-byte EONW header,
little-endian version and length, a kind byte and zero flags. Most messages
are bounded at 2 MiB, with a larger History response allowance.
`decode_request` (`:1246`) reads an action byte and dispatches named
constants; it is not parsing a JSON `"type"` field. `unframe` (`:3803`)
rejects bad magic, version, flags, length and trailing bytes. `Decoder::take`
(`:3926`) uses checked offsets, and `finish` (`:3933`) rejects unread payload.
Orbit's `orbit/crates/protocol/src/lib.rs` and `session/codec.rs` likewise
contain explicit binary encode/decode functions.

**[Eon evidence]** Actual serde users include:

- `runtime/crates/eon-runtime/src/session_registry.rs:19-34`: strict owned
  JSON registry records; `Registration` also uses `skip_serializing_if`,
  which Tide does not currently support.
- `runtime/crates/eon-runtime/src/slots.rs:48-80`: strict, defaulted nested
  settings, parsed with `toml::from_str` at `:91`. The attribute is not
  specifically a wire-protocol or JSON feature there.
- `eon/crates/eon-manifest/src/lib.rs:70-75`: `Source` uses
  `#[serde(tag = "kind", rename_all = "lowercase", deny_unknown_fields)]`
  on `Git { url: String }` and `Workspace { path: String }`. `parse`
  (`:205-209`) validates a JSON value and then deserializes the original
  input into typed manifest records.

**[inference]** The workload supports strict objects and a configurable
internal tag key, but does not justify a JSON rewrite of EONW. `"type"`
is the requested example, not Eon's universal current tag spelling.
The proposal covers the manifest's Source shape; it is not a complete
Eon manifest/runtime migration, a TOML codec, or EONW binary-format support.
The dependent EONW contract experiment can still test hand-written byte
decoders without these derives. No adoption or defect-catching claim is
proved here.

## 4. Primary-source comparison

**[web evidence] Serde:** `deny_unknown_fields` is container policy;
otherwise JSON unknown keys are ignored. It cannot be combined with
`flatten`. `tag = "type"` selects an internal discriminator.
([Container attributes](https://serde.rs/container-attrs.html).)
Internal tagging supports unit and struct variants and certain newtype
payloads, but rejects tuple variants. External tagging is the default.
([Enum representations](https://serde.rs/enum-representations.html).)
Serde's `'de` supports borrowing from input; it is distinct from schema
selection. ([Deserializer lifetimes](https://serde.rs/lifetimes.html).)

**[inference]** Reusing two spellings does not require importing
`Deserialize<'de>`, serde visitors, or serde's full attribute matrix.
Tide should explicitly document its smaller tagged-variant set rather than
promise serde compatibility in general.

**[web evidence] miniserde 0.1.46:** its documented derive supports named
structs and unit enums, not data-carrying enums; its only customization is
`rename`. It uses trait objects and reports decoding errors without detail.
([Crate documentation](https://docs.rs/miniserde/0.1.46/miniserde/).)

**[inference]** This shows a deliberately small serde-free model, but is
insufficient as Tide's replacement: tagged data variants and useful existing
error positions would require additional owned machinery. Its compile-time
claims are not Tide measurements.

**[web evidence] nanoserde 0.2.1:** `DeJson: Sized` exposes
`de_json(state, input: &mut Chars<'_>) -> Result<Self, DeJsonErr>`, without
a lifetime parameter on the trait. Its documentation describes per-format
derives without an intermediate data model.
([Trait](https://docs.rs/nanoserde/0.2.1/nanoserde/trait.DeJson.html),
[crate](https://docs.rs/nanoserde/0.2.1/nanoserde/).)
The project's published feature matrix lists customization such as `rename`
and `default`, but does not list `deny_unknown_fields` or internal tags.
([Project README](https://github.com/not-fl3/nanoserde#features-support-matrix).)

**[inference]** The trait shape supports Tide's owned-decoding direction.
Absence from that matrix is not proof about every source branch; this note
does not claim nanoserde cannot implement the features. Replacing Tide's
engine would still need a behavior audit and new integration work. No
dependency is proposed, so there is no new version or license to admit.

**[web evidence] Go:** `encoding/json.Decoder.DisallowUnknownFields` rejects
object keys without a corresponding non-ignored exported struct field.
`Decode` reads the next value from a stream. Go's v1 mapping also accepts
case-insensitive struct keys and permits duplicate names.
([Official package documentation](https://pkg.go.dev/encoding/json#Decoder.DisallowUnknownFields),
[Unmarshal](https://pkg.go.dev/encoding/json#Unmarshal).)

**[inference]** Go demonstrates strict decoding without serde or its
lifetime API, but a decoder switch gives call sites ownership of policy.
It is also not a complete strict-protocol contract: Tide must retain exact
decoded-name matching, duplicate checks, and whole-input consumption.
Do not copy Go's independent stream and compatibility defaults.

## 5. Options and costs

**[inference]** These are alternatives for the coordinator, not concurrent
APIs to implement:

| Option | Benefit | Cost and disposition |
| --- | --- | --- |
| A. Two existing-helper keys, owned generated codecs | Schema policy visible on each type; same `json::decode` callers; no user macros or dependency | Internal tags need order-independent object inspection and careful duplicate/error handling. **Recommend v1.** |
| B. Public decoder builder with a strict switch; explicit tag dispatch | Familiar Go-like control at a boundary; custom policy possible | Call sites can disagree on one type; nested dispatch and strictness become more API and repeated work. Reject for this workload. |
| C. Keep v0 and hand-write or explicitly generate protocol codecs | EONW already works this way; no new derive behavior | Repeats field sets and validation for routine JSON schemas. Hidden `__codec` support is not a supported public custom-decoder API. Keep for fixed binary protocols; insufficient as the only JSON answer. |
| D. Defer all support, use Rust adapters or replace the engine with a library | Avoids extending Tide's helper now | Defers the same owned-schema need, adds a trust boundary for adapters or a migration for replacement; miniserde is too narrow and nanoserde is not a verified drop-in. Reject replacement; defer implementation until v1. |

**[inference]** Bounded choices inside A, each with alternatives and a
recommendation:

| Choice | Alternatives | Recommendation and reason |
| --- | --- | --- |
| Tier | v0 now / v1 / Later | **v1**: companion already excludes strictness from v0; protect E1 and the demo while assigning a concrete tier. |
| Strictness scope | type-local / recursively forced / per call | **Type-local**, on named structs and enum containers; nested types own their policy and maps accept arbitrary data keys. An enum container covers its named-field payloads, without a new variant-level knob. |
| Unannotated types | make all strict / keep ignored keys | **Keep current behavior**, including `skip` and defaults; changing the default breaks existing callers. |
| Internal shapes | all serde shapes / unit and named only | **Unit and named only**; no newtype/map flattening or tuple interpretation required by inspected users. Unsupported shapes fail at derive time. |
| Discriminator | fixed `type` / literal chosen key; first only / any position | **Chosen literal key, any position**: Eon uses `kind`; ordinary JSON object order must not change acceptance. |
| Invalid/duplicate tags | first or last wins / fallback / error | **Error**, even for identical repetitions; one variant owner and no ambiguity. Repeated known payload fields retain existing rejection. |
| Names and defaults | add new alias/default rules / reuse existing keys | **Reuse existing rules**: variant rename wins over enum case conversion; field rename affects payload names; missing tag is never defaulted. Compile-time collisions prevent impossible encoding. |
| API/mechanism | public values/visitors/reflection / private parser support | **Private support**, retain both public signatures and owned `Decode`; no new public trait methods. Mechanism remains an implementation choice subject to the contract below. |

## 6. Proposed decision and falsifiers

**[inference] Owning chapter:** `design/01-design.md`, D6, because it owns
the derive/attribute decision. Its v1 tier row must include the addition.
03 owns capabilities and interop, not schema selection; the reference is
the v0 teaching surface, not the place to originate a v1 language choice.
The coordinator accepted it (§10).
Rule IDs CODEC-1 and CODEC-2 allow later conformance tracing under D0.

**[inference] Proposed v1 usage:**

```rust
#[derive(Debug, PartialEq, Encode, Decode)]
#[codec(tag = "type", deny_unknown_fields)]
enum Message {
    Set { key: String },
    Clear,
}
// Message::Set { key: "a".into() } -> {"type":"Set","key":"a"}
// Message::Clear                 -> {"type":"Clear"}
```

**[inference]** Both derives parse and validate both keys, even when a
type derives only one trait. Strictness affects only decoding. Allowed
keys are the selected object's non-skipped renamed fields and, for tagged
enums, its discriminator. A skipped payload field adds no allowed key.
The tag owns its own key. A tag/non-skipped-field collision, duplicate wire
names, malformed/repeated helper keys or unsupported placement is a compile
error. `tag` accepts only enum containers whose variants are unit or named;
`deny_unknown_fields` accepts named structs or enum containers. The keys
may be combined or used independently. No `flatten`, `content`, `untagged`,
catch-all, custom borrowing API, or automatic TOML/binary support is added.

**[inference]** On tagged inputs, a missing, non-string, unknown or repeated
tag is always `json::Error`. Payload precedes/follows tag freely. Unknown
payload keys are rejected only with strictness enabled. Wrong payload types,
missing required fields, duplicate known fields, trailing input and excessive
nesting still fail. `Option` and explicit defaults retain current behavior;
neither changes an invalid present value. Nested strictness is independent.
No promise is made about which error wins when input violates several rules.

**[inference] Proposed implementation mechanism, not a public contract:**
reuse the current lexer and validated value skipping to locate a tagged
object and its discriminator in a private scan, then decode that object as
the selected variant from the original input. Preserve duplicate occurrences
rather than first converting to a map; preserve raw number tokens rather
than routing through floating-point `Value`. Retain absolute positions,
nesting and active-codec-call accounting when revisiting input. Keep the
scan private; it must not construct handles. This trades extra scanning for
no public DOM/visitor API. Deeply nested tagged objects can be rescanned at
multiple enclosing levels; no linear-time or performance result is claimed.
The implementation bead must reject a design that resets limits, loses
duplicate tags or reports positions relative to a detached substring.

**[inference] Cheapest future observable checks:**

| Surface | Observation that would falsify the proposal |
| --- | --- |
| Valid wire shape | Set and Clear encode to the shown objects; first/last/escaped tag-key spellings decode identically. |
| Schema closure | Unknown key before or after tag, wrong-variant field, or extra key on strict Clear succeeds. |
| Tag failure | Missing, null, numeric, unknown, identical repeated, conflicting repeated, or escaped-equivalent duplicate tag succeeds. |
| Existing policies | A repeated known field succeeds; default repairs malformed input; enclosing strictness rejects a nested loose type's extension; an unannotated type stops ignoring extras. |
| Derive checks | Either derive accepts a tuple/newtype tagged variant or a renamed field/tag collision, or rejects valid strict-only/external-tagged usage. |
| Parser boundary | Tag scanning bypasses nesting/call limits, accepts trailing values or malformed JSON, changes large integer tokens, loses absolute error locations, or permits ROOT-4 handle construction. |

**[inference]** The first row's positive cases should be acceptance assertions;
the remaining rows describe forbidden results. Include a nested strict
struct, a nested loose struct, a map, skipped/default/Option fields, and
rename combinations in a small table-driven fixture. No such tests or
implementation were written or executed by this design bead.

## 7. Low-entropy admission and D0

**[repo evidence]** GOVERNANCE's admission filter rejects substantial
additions by default when semantic entropy exceeds safer change; D0 makes
design provisional and requires normative rules to reach conformance tests.

**[inference]** A passes provisionally as a v1 library/toolchain addition:

| Admission question | Assessment |
| --- | --- |
| Mechanically safer change? | Unknown schema extensions and ambiguous discriminators fail at the boundary; accepting a new field requires an explicit schema change. Strict protocols sacrifice silent forward compatibility, deliberately. |
| Overlapping mechanisms? | Two knobs on the existing helper and one decode entry point; no second derive, reflection construction, or builder. Yes, one overlap: a second enum representation, chosen by the peer's wire schema rather than by the author; external tagging remains the default. |
| Local reasoning? | Policy is visible on a type; nested types own their closure. Schema strictness is not authentication or capability safety by itself. |
| Incremental feedback? | Placement/name collisions fail at derive time; wire errors remain ordinary `Result`. Compile cost is unmeasured. |
| One obvious shape? | Named structs and named/unit enum variants cover the inspected JSON consumer; no shape guessing. |
| Library/tooling instead of core? | Existing toolchain derives and std parser own the work; no new grammar, trait-system power or user macro permission. |

**[inference]** This is a reason to admit the bounded proposal to design
review, not proof that implementation or agent learnability passes. Preserve
the current E1 materials and v0 reference. Run future conformance against
the public JSON boundary when implementing; extend D0 evaluation when v1
enters its taught surface. No fresh-agent trial or supervisor review ran in
this author session; the coordinator owns sequential Codex/Opus review and
landing on the eventual frozen candidate.

## 8. Reference text and follow-ups (author proposal; §10 records what was done)

**[inference] Reference-slot recommendation:** no edit to `reference.md` is
needed for a v1-only decision. If the coordinator wants a short pointer in
§2.2, append this text, without teaching v1 as available:

> Planned v1 (D6): `#[codec(deny_unknown_fields)]` makes object decoding
> strict; `#[codec(tag = "type")]` selects internally tagged unit or
> named-field enum variants. Neither key is available in v0.

**[inference]** When taking the companion/reference slot, also replace the
companion's “not yet tiered” listing for `deny_unknown_fields` with a v1
pointer to D6 and include internal tags there. Other omitted keys and
format plug-ins retain their existing status. The author edited neither
single-holder file and claims no reference token recount.

**[inference] Suggested follow-up beads, text only:**

1. **Implement v1 strict object decoding (CODEC-1).** Extend shared helper
   validation and derived object dispatch; prove strict/loose, skipped,
   defaulted, nested and external-variant cases through `std::json`, with
   precise errors and unchanged v0 behavior. Update API documentation,
   conformance mapping and supported-tier teaching together.
2. **Implement v1 internal JSON tags (CODEC-2), after 1.** Add private
   order-independent discriminator handling and generated unit/named
   variants, with the tag, collision and parser-boundary falsifiers above.
   No dependency or public custom-decoder API is implied. Carry a small
   Eon Source-shaped fixture with `tag = "kind"` and lowercase names.

**[repo evidence]** Existing fx-r9e7 owns the EONW codec-contract experiment;
fx-ho6o owns adoption gates. **[inference]** Keep those separate, and do not
file a new EONW JSON rewrite, full Eon port, adapter or TOML codec on the
strength of this note. Implementation timing remains after the current E1
and demo priorities.

## 9. Coordinator choices

**[inference]** All pending choices, with recommended answers:

1. **Accept A in v1?** Recommend yes: decide CODEC-1/2 in D6 after review of
   this proposal; leave v0 defaults and four-key surface intact.
2. **Take the reference slot now?** Recommend no: the core reference teaches
   v0. Reconcile the companion's stale tier listing when that slot is next
   held; the exact optional pointer is above.
3. **Schedule implementation follow-ups?** Recommend the two bounded beads
   above after E1/demo; no implementation, filing or adoption is authorized
   by this author proposal. Keep EONW on its existing binary experiment.

## 10. Decision

Coordinator (Opus), 2026-11-13, under the owner's delegation of technical
choices (AGENTS.md, Decisions):

1. **Option A, in v1.** D6 states CODEC-1 and CODEC-2 as decided, and the
   tier table lists them under v1. v0's four-key surface and its defaults
   stay unchanged. A is chosen because it puts the schema policy on each
   type, adds no derive, user macro, dependency or public API, and covers
   the one internally tagged JSON shape Eon uses (§3). B splits policy
   across call sites, C repeats every schema by hand, and D only defers
   the same need.
2. **No `design/reference.md` edit.** The core teaches v0. The companion's
   v1 list gains a Codecs entry, and its "not yet tiered" sentence no longer
   lists `deny_unknown_fields`, so the companion and D6 agree. The
   companion's token count of the core is unchanged.
3. **Follow-ups.** The two implementation beads in §8 are filed, and they
   start after E1 (fx-ic2) and the first demo (fx-nkv). EONW stays on fx-r9e7's binary experiment, and
   fx-ho6o keeps the adoption gates.
