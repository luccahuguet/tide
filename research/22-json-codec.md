# JSON codec: derive(Encode, Decode) and std::json (research for fx-08m)
Drafted by Fable on 2026-11-10; its decisions are recorded on fx-08m.

Research agent report, 2026-11-10, against main `db3d10c`. Nothing in the repository was edited, no build was run, and no file under `trials/e1-checked/` was opened.

Evidence labels: **[repo]** is read from the tree, **[web]** is fetched and cited, **[memory]** is my recollection of serde's behaviour that the fetched pages did not confirm and that the implementer should check before relying on it.

## 0. What the design fixes, and what it leaves open

Fixed by `design/reference.md` §2.2 and §12.3, and `design/01-design.md` D6 **[repo]**:

- `std::json::encode(&v) -> Vec<u8>`. It returns no `Result`, so encoding is infallible.
- `std::json::decode::<T>(&bytes) -> Result<T, json::Error>`. The Feel example calls `json::decode(body)` with `body: &[u8]`.
- The keys `rename = ".."`, `rename_all = "camelCase"`, `default`, `skip`, "with serde's spellings and defaults (provisional)".
- A missing `Option` field decodes as `None`. Enums are externally tagged: `{"Set":{"key":"a"}}`, `"Clear"`.
- `json::Error` is `Error + Send + Sync + Pure`.
- The derives are toolchain proc macros in the prelude, and `#[codec]` is their helper attribute.
- The companion leaves out of v0: other formats, the codec plug-in API, and the `flatten`, `deny_unknown_fields` and `try_from` keys.
- ROOT-4 (`design/03`): no capability or handle type has a `Decode` impl.
- `design/04-contracts.md` §4.2: `Decode` checks invariant clauses at run time. The contract attributes do not exist yet, so this is out of scope here.

The design is silent on everything else in this report. Each such point is marked **silent** with a recommendation.

Current code **[repo]**: `tide-macros` has `error.rs` (syn 2.0.119, quote, proc-macro2, pinned) and a `lib.rs` entry per macro. `root.rs` is shared with the conformance stub by `include!` and re-exports the macros. `__error` is a `#[doc(hidden)]` support module that the Error derive names as `::std::__error::..`. The conformance stub has no json or codec items today.

## 1. Design choices

### 1.1 Trait shape

| Option | Cost in an admitted std |
| --- | --- |
| A. serde's visitor model (`Serializer`/`Deserializer`/`Visitor`, 29 data-model types [web: https://serde.rs/data-model.html]) | Large generic surface. Every item the expansion names becomes admitted std API. It serves format plug-ins, which v0 leaves out. |
| B. Value tree (`encode(&self) -> Value`, `decode(&Value)`) | Small, but allocates the whole tree twice, loses line and column for data errors unless every node carries a span, and adds a public `Value` type that the reference does not list. |
| C. Direct JSON writer and reader, as nanoserde does: `fn ser_json(&self, d: usize, s: &mut SerJsonState)` and `fn de_json(state: &mut DeJsonState, input: &mut Chars) -> Result<Self, DeJsonErr>` [web: https://docs.rs/nanoserde/latest/nanoserde/trait.SerJson.html, https://docs.rs/nanoserde/latest/nanoserde/trait.DeJson.html] | Smallest. One pass, positions come free, no intermediate type. It ties the trait to JSON. |

miniserde shows the other small design: trait objects, no recursion, an error with no information, and a derive that takes only braced structs and C-style enums [web: https://docs.rs/miniserde/latest/miniserde/]. That derive is too narrow for the Acceptance, which needs data enums, and its empty error fails "malformed input gives json::Error" in any useful sense.

**Recommendation: C.**

```rust
pub trait Encode { #[doc(hidden)] fn __encode(&self, out: &mut __codec::Encoder); }
pub trait Decode: Sized { #[doc(hidden)] fn __decode(input: &mut __codec::Decoder<'_>) -> Result<Self, json::Error>; }
```

- The traits are public and in the prelude under the derives' names. A macro and a trait share a name in separate namespaces, as serde's do. Users need the names for bounds such as `T: Decode`.
- The methods and `Encoder`/`Decoder` are hidden and undocumented. A later plug-in API (D6) can then replace them without breaking a documented contract.
- `Encoder` wraps a `Vec<u8>`. `Decoder` wraps `&[u8]`, a byte offset and a depth counter.
- `Decode` has no lifetime parameter and decodes owned values only. A struct with a `&str` field can derive `Encode` but not `Decode`. **Silent**; serde's borrowed decoding needs the visitor model.
- **Silent** on hand-written impls. Add no rule against them, and do not document the methods.

### 1.2 Dependency (AP-DEPENDENCY-001)

Capability: a JSON lexer, parser and writer, plus the derive's generated calls. Float digits are a sub-capability.

| Candidate | Finding |
| --- | --- |
| serde + serde_json | The derive would have to target serde's traits, so serde's API would become admitted std surface. A crate with `Role::Other` is rejected by `admit_def` unless std re-exports it **[repo: `walk.rs`]**. It adds serde, serde_core, itoa, memchr and zmij [web: https://raw.githubusercontent.com/serde-rs/json/master/Cargo.toml] to the trusted runtime and to the hermetic `--frozen` build. Its default float parsing is "best-effort precision", and exact round trips need the `float_roundtrip` feature at about 2x cost [web: https://docs.rs/crate/serde_json/latest/features]. `to_vec` is fallible, which conflicts with the reference's signature [web: https://docs.rs/serde_json/latest/serde_json/fn.to_vec.html]. |
| nanoserde, vendored | Zero dependencies [web: https://docs.rs/nanoserde/latest/nanoserde/]. Its attribute set and enum encoding are its own, not serde's, so most of it would be rewritten. |
| ryu or zmij, for floats only | Not needed; see 1.5. |
| Owned code | An estimated 600 to 900 lines in `tide-std` and about 500 in `tide-macros`. No new crate. syn, quote and proc-macro2 are already accepted (fx-3iu). |

**Choice: owned code, no new dependency.** fx-3iu rejected thiserror for the same reason: its `::thiserror` paths do not resolve through Tide's std. Replacement cost: the hidden trait methods and `__codec` are internal, so swapping the engine later changes no consumer code. One exception to "owned": port serde's case-conversion table exactly (1.4), with attribution, since "serde's spellings" is the contract.

### 1.3 Derive expansion

serde's JSON mapping, which the reference adopts for enums [web: https://serde.rs/json.html]:

| Rust | JSON |
| --- | --- |
| `struct W { a, b }` | `{"a":0,"b":0}` |
| `struct X(i32, i32)` | `[0,0]` |
| `struct Y(i32)` | `0` (the inner value) |
| `struct Z;` | `null` |
| `E::W { a, b }` | `{"W":{"a":0,"b":0}}` |
| `E::X(i32, i32)` | `{"X":[0,0]}` |
| `E::Y(i32)` | `{"Y":0}` |
| `E::Z` | `"Z"` |

Recommendations:

- **Named struct, decode.** One `Option<FieldTy>` local per field. Loop over keys: a known key decodes into its local, a second occurrence is an error (2.2), an unknown key calls `skip_value`. serde ignores unknown fields by default for self-describing formats [web: https://serde.rs/container-attrs.html], and the Acceptance requires it. After the loop each `None` local goes through a hidden `Decode::__missing(name)`, whose default returns the "missing field" error and which `Option<T>` overrides to return `None`. That reproduces serde's rule: "If the missing field is of type `Option<T>` then treat is as `None`, otherwise it is an error" [web: https://raw.githubusercontent.com/serde-rs/serde/master/serde/src/private/de.rs]. The rule is by trait impl, not by the spelling of the type, so a type alias for `Option` works.
- **Enum, decode.** A string selects a unit variant. An object must hold exactly one key, which selects a data variant. Anything else is a data error that names the expected variants. serde also accepts `{"Z":null}` for a unit variant [memory]; do not add that, since the reference shows only the string form.
- **Encode.** Fields in declaration order, compact output with no whitespace. `None` is written as `null`. serde omits a field only under `skip_serializing_if`, which is not in Tide's key set [web: https://serde.rs/field-attrs.html].
- **Generics.** Follow serde: put `T: Encode` (or `T: Decode`) on every type parameter that appears in an encoded field [web: https://serde.rs/attr-bound.html]. Do not bound field types, as `error.rs`'s `bound()` helper does. For a recursive type such as `struct Tree<T> { kids: Vec<Tree<T>> }`, a `Vec<Tree<T>>: Encode` bound makes the trait solver cycle. That cycle is my inference from rustc's rules, so test it (3, T9). Serde's escape hatch, `bound = ".."`, is not in Tide's key set.
- **Skipped or defaulted generic fields.** Add `FieldTy: Default` only when the field type mentions a type parameter. Otherwise let the generated `Default::default()` call fail type checking at the field's span.
- **Raw identifiers.** `r#type` encodes as `"type"` [memory: serde strips `r#`].
- **Empty enum.** `Decode` always returns an error; `Encode` is `match *self {}`.
- **Unions.** Rejected with a spanned error.
- **Paths.** Every generated path is absolute (`::std::__codec::..`, `::core::..`), as in `error.rs`.

Std impls to provide. **Silent**; the Acceptance needs `Option`, `Vec` and maps:

- `bool`; all integer types including `usize`/`isize` and the 128-bit ones; `f32`, `f64`.
- `char` as a one-character string, `String`, and `&str`/`str` for `Encode` only.
- `()` as `null`; `Option<T>`; `Box<T>`; `Vec<T>`, `[T]` and `VecDeque<T>` as arrays.
- Tuples up to a small arity as arrays.
- `HashMap<String, V, S>` and `BTreeMap<String, V>`; `HashSet` and `BTreeSet` as arrays.

Leave out `Rc`, `Arc`, `RefCell`, `Shared`, every capability and handle type (ROOT-4), and `Secret`. The Feel example's `link: Url` needs `Decode for Url`; that belongs to the bead that adds `Url`, so record it there.

### 1.4 Attribute semantics

serde's wording [web: https://serde.rs/container-attrs.html, https://serde.rs/field-attrs.html, https://serde.rs/variant-attrs.html]:

| Key | Where | Recommended Tide semantics |
| --- | --- | --- |
| `rename = "x"` | field, variant | "Serialize and deserialize this variant with the given name instead of its Rust name." It wins over `rename_all` [memory]. |
| `rename_all = ".."` | struct, enum | "Rename all the fields (if this is a struct) or variants (if this is an enum)". Accept exactly serde's eight values: `lowercase`, `UPPERCASE`, `PascalCase`, `camelCase`, `snake_case`, `SCREAMING_SNAKE_CASE`, `kebab-case`, `SCREAMING-KEBAB-CASE`. Any other string is a compile error that lists them. |
| `default` | field | "If the value is not present when deserializing, use the `Default::default()`." No effect on `Encode`. |
| `default` | struct | "When deserializing, any missing fields should be filled in from the struct's implementation of `Default`. Only allowed on structs." Build one `Self::default()` and move the missing fields out of it. |
| `skip` | field | "Skip this field: do not serialize or deserialize it." On decode the field gets `Default::default()`. A key of that name in the input is treated as unknown and ignored. |

The conversion algorithm: serde assumes fields are written in snake_case and variants in PascalCase, and converts from that assumption. It does not tokenize words in general. Port `serde_derive/src/internals/case.rs` exactly (https://github.com/serde-rs/serde/blob/master/serde_derive/src/internals/case.rs, MIT OR Apache-2.0) [memory for the algorithm's details; the file is the source of truth], and record the attribution.

**Silent** points, each with a recommendation. The rule behind them: the reference's key list is the contract, so anything outside it is a spanned compile error that names the supported keys.

1. **`skip` on a variant.** serde allows it ("Never serialize or deserialize this variant") and fails at run time when such a value is serialized [memory]. Tide's `encode` cannot fail. **Reject `skip` on variants.** The alternative is a panic in `encode`, which the infallible signature should not hide.
2. **`default = "path"`, `rename(serialize = .., deserialize = ..)`, `alias`, `skip_serializing`, `skip_deserializing`, `skip_serializing_if`, `rename_all_fields`, `rename_all` on a variant, `rename` on a container.** Reject all. Container `rename` has no effect in JSON anyway. The messages for `flatten`, `deny_unknown_fields` and `try_from` should say they are not in v0, as the companion does.
   - Risk: an agent porting serde code writes `default = "path"` or `skip_serializing_if`. If E1 runs show this is common, widening the key set is a design change for the reference's holder, not for this bead.
3. **Name collisions** after `rename`/`rename_all`: two fields or two variants with one JSON name. **Compile error at the second.** serde's own handling here is unverified [memory].
4. **`skip` with `default`** on one field: accept; `default` is then redundant. **`skip` under a container `default`**: take the field from the container's default value, as for a missing field [memory: serde does the same].
5. **`#[codec]` on a type that derives only one of the two traits**: accept every key under either derive. `default` and `skip` are checked for syntax by both and used by the one that needs them.

### 1.5 Number model and floats

Facts:

- RFC 8259 §6 forbids `Infinity` and `NaN`, and calls integers in [-(2^53)+1, 2^53-1] interoperable [web: https://www.rfc-editor.org/rfc/rfc8259.txt]. I-JSON says a sender "cannot expect a receiver to treat an integer whose absolute value is greater than 9007199254740991" exactly [web: https://www.rfc-editor.org/rfc/rfc7493.txt].
- serde_json stores numbers as `PosInt(u64)`, `NegInt(i64)` or `Float(f64)` (always finite). It writes NaN and infinities as `null`, and formats finite floats with a shortest-digits library, now zmij [web: https://raw.githubusercontent.com/serde-rs/json/master/src/number.rs, https://raw.githubusercontent.com/serde-rs/json/master/src/ser.rs].
- Rust's `f64::from_str` returns "the closest representable floating-point number", but its grammar is wider than JSON's: it accepts `inf`, `nan`, `5.`, `.5` and a leading `+` [web: https://doc.rust-lang.org/std/primitive.f64.html].
- Rust's docs do **not** promise that float `Display`/`Debug` output is the shortest string that round-trips [web: the f64 page and https://doc.rust-lang.org/std/fmt/index.html state nothing]. The implementation in `core::fmt::float` does use a shortest algorithm (Grisu with a Dragon fallback) [memory; source at https://doc.rust-lang.org/src/core/fmt/float.rs.html]. Treat the round trip as a property that Tide's tests own.

Recommendation. Tide has no `Value` type, so the target type drives number decoding:

- **Lexing.** The decoder first checks the token against RFC 8259's grammar: an optional `-`, then `0` or a nonzero digit followed by digits, then an optional fraction and exponent. That keeps Rust's wider grammar from leaking in.
- **Integer targets.** The token must have no fraction and no exponent, and must fit the type. Otherwise it is a data error: "number out of range", or an invalid-type message for `1.0`. serde_json also refuses a float for an integer [memory]. `-0` into an unsigned type decodes as 0 [memory: as serde_json].
- **Float targets.** Any valid number token, parsed with `str::parse::<f64>` or `::<f32>` on the checked token.
  - Parse `f32` directly, not through `f64`, so the value is rounded once.
  - If the result is infinite (for example `1e400`), return "number out of range", as serde_json does [memory]. A decoded float is then always finite.
- **Encoding floats.** Finite values use Rust's `{:?}`. NaN and infinities become `null`, as in serde_json; the infallible `encode` leaves only that or a panic.
  - Why `{:?}` and not `{}`: `Debug` prints `1.0`, `1e300` and `1e-7`. `Display` prints `1`, a 301-digit string, and `0.0000001` [memory]. Every finite `Debug` output is a valid JSON number, and the forms match serde_json's style.
- **Encoding integers.** Exact decimal digits at every width, including above 2^53 and the 128-bit types. serde_json does the same [memory]. Document the I-JSON interoperability limit; do not quote large integers.
- **Relation to `std::float`.** Canonical NaN never reaches the wire. `-0.0` encodes as `-0.0` and decodes to `-0.0`, so `float::bits` survives a round trip for every finite value. `std::json` runs inside trusted std, so it may call `to_bits`-free core formatting and parsing directly. The expansion in user code calls only `::std::__codec` functions, so no denied `core::f*` method appears in admitted code **[repo: `policy.rs` denies most float methods]**.

### 1.6 Strings and UTF-8

- **Decode.** The input is bytes, so the decoder validates UTF-8. RFC 8259 §8.1 requires UTF-8 [web].
  - Raw control characters below U+0020 inside a string are an error [web: serde_json's `ControlCharacterWhileParsingString`, https://raw.githubusercontent.com/serde-rs/json/master/src/error.rs].
  - `\uXXXX` escapes combine surrogate pairs. A lone surrogate is an error, as in serde_json ("lone leading surrogate in hex escape") and as I-JSON requires ("MUST NOT include code points that identify Surrogates") [web: rfc7493].
- **Encode.** Escape `"`, `\`, and U+0000 to U+001F, using `\b \t \n \f \r` where they exist and `\u00XX` otherwise. Do not escape `/`, U+007F or non-ASCII. This is RFC 8259 §7's required set and serde_json's table [web: rfc8259; https://raw.githubusercontent.com/serde-rs/json/master/src/ser.rs, where the fetch summarized the table and did not quote it].
- **BOM.** RFC 8259 lets a parser ignore one ("MAY ignore"). serde_json rejects it [memory]. **Silent**; recommend rejecting it, which is the smaller guarantee.

### 1.7 `json::Error`

serde_json's error has `line()` and `column()`, both one-based, and `classify()` returning `Io`, `Syntax`, `Data` or `Eof`. Its `Display` is `"{} at line {} column {}"` [web: https://docs.rs/serde_json/latest/serde_json/struct.Error.html, https://docs.rs/serde_json/latest/serde_json/error/enum.Category.html, https://raw.githubusercontent.com/serde-rs/json/master/src/error.rs]. nanoserde's has public `line`, `col` and `msg` [web: https://docs.rs/nanoserde/latest/nanoserde/struct.DeJsonErr.html]. miniserde's carries nothing.

**Silent** beyond the name and the `Error + Send + Sync + Pure` bound. Recommendation:

- An opaque struct holding a `String` message, a line and a column. Those fields are structurally `Pure`, and the type is `Send + Sync`.
- `Display` in serde_json's format, `<message> at line L column C`. Reuse serde_json's message texts where a case matches (`expected value`, `EOF while parsing a string`, `trailing characters`, `recursion limit exceeded`, ``missing field `x` ``), since agents recognize them.
- `Debug` and `core::error::Error` with no source. fx-3iu's plan-review comment assigns these impls to this bead **[repo]**.
- Accessors `line()` and `column()`, one-based. serde_json does not say whether its column counts bytes or characters [web]. Count bytes after the last newline, and document that.
- No public kind enum yet. Keep the kind private (syntax, end of input, data); a `classify()` can be added later without a break. No `Io` kind, since `decode` reads a slice.
- Tests assert exact positions only for syntax errors. For data errors assert only that the message names the field or variant.

### 1.8 What `encode` and `decode` accept and produce

- `pub fn encode<T: Encode + ?Sized>(value: &T) -> Vec<u8>`. Output is compact UTF-8 with no trailing newline.
- `pub fn decode<T: Decode>(bytes: &[u8]) -> Result<T, Error>`. A `&Vec<u8>` argument coerces, so the reference's `decode::<T>(&bytes)` compiles.
- The top level may be any JSON value (RFC 8259 §2: "A JSON text is a serialized value" [web]). Whitespace around it is allowed. Any other trailing byte is an error; serde_json calls it "trailing characters" [web: error.rs].
- **Silent** on `&str`/`String` forms, `io::Read`/`io::Write` forms and pretty printing. Add none: the reference gives exactly two functions. A caller with a `&str` writes `s.as_bytes()`, and one that wants a `String` writes `String::from_utf8(v)`.
- Agents with serde_json habits will try `json::to_string` and `json::from_str` and get an unresolved-name error. A help text on that error would be new diagnostic work, so report it as a finding. It is not in this bead's scope.

### 1.9 Determinism

- Struct fields and enum variant fields are written in declaration order, as serde does.
- `BTreeMap` is written in key order.
- **`HashMap`: sort entries by key bytes before writing.** With the fixed key, iteration order is repeatable for one construction history. I believe it still depends on capacity and insertion order, so two equal maps can iterate differently (my inference about hashbrown; T8 checks it). A map built with `RandomState::from_rand` differs from run to run. Sorting makes `encode` a function of the value: equal values give equal bytes.
  - serde_json writes in iteration order and does not sort [memory].
  - Byte order of UTF-8 keys equals code-point order. RFC 8785 sorts by UTF-16 code units [web: https://www.rfc-editor.org/rfc/rfc8785.txt], which differs for supplementary characters. Do not claim JCS conformance.
- `HashSet`: sorting needs `T: Ord`, a bound serde does not have. Recommend iteration order, with a doc sentence. **Silent**; the Acceptance does not mention sets, so leaving set impls out is the cheaper option.
- **Map keys.** `String` only in v0, enforced by the impl's type. serde_json fails at run time on "a map with non-string keys" [web: https://docs.rs/serde_json/latest/serde_json/fn.to_string.html] and writes integer keys as quoted strings [memory]. Tide's `encode` cannot fail, so the restriction has to be a compile-time bound. A hidden sealed key trait can add integers and `char` later.

### 1.10 Admission and the trusted macro crate

**[repo]** findings:

- `generated_by_toolchain` exempts only `#[tide::main]`'s entry point and `core`'s `#[test]`. A derive's expansion is walked as user code, as the Error derive's is. Every path it names must therefore be admitted.
- Items of Tide's std are admitted when no rule lists them (`admit_def`: `role == Role::TideStd`). `::std::__codec::*` needs no allowlist entry, as `__error` needs none. Only `tide_std::__rt::*` is denied.
- `core::option`, `core::result`, `core::default`, `core::convert` and `core::ops` are allowed, so `?` and `Default::default()` in the expansion pass.
- The macro crate is trusted and is built by the hermetic helper in `tests/support/mod.rs`. Two more derives add no crate and no build step.
- User macro definitions are `FX0101`, so the expansion must define no `macro_rules!`. It also needs no `unsafe`.

Recommendations:

- Register both derives in `tide-macros/src/lib.rs` as `#[proc_macro_derive(Encode, attributes(codec))]` and `#[proc_macro_derive(Decode, attributes(codec))]`. Put them in one new module, `codec.rs`, as fx-3iu's comment asks. Both derives parse `#[codec]` through one function, so the two cannot disagree on a name.
- Re-export them from `root.rs` and add them to the prelude list. The conformance stub picks them up through its `include!`.
- Put the traits, `Encoder`, `Decoder`, the std impls and `json` in files that the stub also includes, as `io/shared.rs` and `error.rs` are. That meets fx-nkv's equivalence clause with one definition.
- ROOT-4 holds by construction: std gives capability types no `Decode` and no `Default`, so a struct holding one cannot derive `Decode`, with or without `skip`. A hand-written `Decode` impl still cannot build a capability, because the field is private.
- `Pure` needs no driver change. `json::Error` is structurally `Pure`, and the generated impls are ordinary local impls.
- The generic-argument recovery that fx-3iu added to `walk.rs` (`path_args`) was written for one derive's expansion. Check that it also covers this expansion's generic impls. A failure would show as a false `FX0520` or a `Pure` diagnostic on derived generic types (T9).

## 2. Pitfalls

1. **Precision loss.** Integers above 2^53 survive a Tide round trip but not a JavaScript peer. A float written into an integer field is an error, not a truncation. NaN and infinities encode as `null`, which does not decode back into `f64`; it does decode into `Option<f64>`, as `None`. So `decode(encode(x)) == x` holds only for finite floats. State this in the docs.
2. **Duplicate keys.** RFC 8259 says names "SHOULD be unique" and that receivers differ: last pair, error, or all pairs [web]. I-JSON says "MUST NOT" [web]. Bishop Fox shows parsers that disagree on precedence being exploited and recommends "Generate fatal parse errors on duplicate keys" [web: https://bishopfox.com/blog/json-interoperability-vulnerabilities]. serde's derived structs reject a repeated known field ("duplicate field") [memory: `de::Error::duplicate_field`]. serde_json's maps keep the last value [memory; its docs do not say].
   - Recommend an error for a repeated struct field (serde's behaviour) and for a repeated map key. The map case is stricter than serde_json and costs nothing, since the insert reports it. Mark it as a deliberate deviation.
   - A repeated unknown key is ignored, because unknown keys are not tracked.
3. **Deep nesting.** serde_json stops at 128 nested arrays and objects ("recursion limit exceeded") unless `unbounded_depth` is enabled [web: error.rs, Cargo features]. RFC 8259 §9 allows such limits [web]. Use the same limit of 128, counted in the `Decoder`.
   - `skip_value` for unknown keys must count depth too, or be iterative. Otherwise an ignored field becomes the stack-overflow path.
   - Do not reserve capacity from the input: no `with_capacity(n)` from a claimed length.
   - Encoding recurses through the value's own structure; a very deep `Box` chain can overflow the stack. The value is the program's own data, so document it and add no limit.
4. **Missing and null.**
   - A missing `Option` field gives `None`, and so does `null`.
   - `Some(None)` in an `Option<Option<T>>` encodes as `null` and decodes as `None`.
   - `null` for a non-`Option` field is an error even under `#[codec(default)]`. `default` covers absence only: "If the value is not present" [web: field-attrs].
   - `()` and unit structs are `null`.
5. **`skip` and `default`.** `skip` needs `Default` only for `Decode`. With `derive(Encode)` alone it must not demand `Default`. A skipped field does not round-trip; it comes back as the default. A container `default` needs `Self: Default`, and a derived `Default` plus a container `default` is the common pairing.
6. **Rename collisions and conventions.**
   - `rename_all` on an enum renames variants, not the fields of struct variants; serde uses `rename_all_fields` for those [web].
   - serde's conversion assumes the source convention. A field named `userID` under `snake_case` does not become `user_id` [memory].
   - Collisions are compile errors (1.4).
7. **Round trip with unknown keys.** Decoding and re-encoding drops unknown fields. That is the price of the Acceptance's "unknown keys are ignored".
8. **Types containing borrowed data or authority.** They can derive `Encode` only if std gives their fields `Encode`. Keep `Secret` and the capability types without it.

## 3. Cheapest falsifying tests

These are executable programs built through `tide-admit` against the production std, in the style of `io_core_builds_and_runs` in `crates/tide-admit/tests/build.rs`. The Acceptance asks that items execute, not only type-check. Write new fixtures; take none from `trials/e1-checked/`.

| # | Test | Falsifies |
| --- | --- | --- |
| T1 | Round trip of a named struct, tuple struct, newtype, unit struct, and an enum with all four variant kinds; assert the exact bytes of 1.3's table and the reference's `{"Set":{"key":"a"}}` and `"Clear"` | 1.1, 1.3, and that the expansion is admitted |
| T2 | Missing `Option` field gives `None`; missing plain field gives an error naming it; `null` into a plain `default` field is an error; an `Option` alias behaves like `Option` | 1.3's `__missing`, pitfall 4 |
| T3 | Unknown keys with nested values are ignored, including one nested 200 deep, which must return the depth error and not crash | 1.3, pitfall 3 |
| T4 | Each attribute: `rename`, each of the eight `rename_all` values on one struct and one enum, field `default`, container `default`, `skip` (key absent on encode, default on decode, and a present key ignored) | 1.4 |
| T5 | Compile-fail cases: `skip` on a variant, an unknown key, `default = "f"`, a rename collision, a capability field with `skip` | 1.4, 1.10 |
| T6 | Malformed input table: empty input, `{`, `[1,]`, `{"a":1,}`, `"\ud800"`, a raw control byte in a string, invalid UTF-8, `01`, `1.`, `.5`, `+1`, `NaN`, `1 2`, a BOM, 129 nested `[`. Each returns `json::Error`, and two fixed cases assert the line and column | 1.5 to 1.7 |
| T7 | Floats: for a fixed list (`0.1`, `-0.0`, `5e-324`, `f64::MAX`, `1e16`, `1e-7`, `f32` values such as `16777217.0`) and a few thousand values from a fixed-seed generator, `float::bits(decode(encode(x))) == float::bits(x)`. NaN and infinity encode as `null`. `1e400` is an error. `u64::MAX` and `i64::MIN` round-trip, and `256` into `u8` is an error | 1.5, including the undocumented shortest-digits property |
| T8 | Two `HashMap`s with equal contents and different insertion orders and capacities encode to identical bytes, with keys sorted; so does a map built with `RandomState::from_rand` | 1.9 |
| T9 | Generic `Wrapper<T>`, recursive `Tree<T> { kids: Vec<Tree<T>> }`, and a derived generic type passed to a function with a `Pure` bound; all compile and round-trip | 1.3's bounds, 1.10's `walk.rs` note |
| T10 | String escapes: every byte from 0x00 to 0x1F, `"`, `\`, `/`, U+007F, a non-ASCII character and an astral character round-trip; assert the escaped bytes for a few | 1.6 |
| T11 | The same source checks under the conformance stub std with the same verdict | fx-nkv equivalence |

The Feel example in `design/01-design.md` (`derive(Decode)` plus `#[from] json::Error`) is a good twelfth fixture once `Url` exists. Until then, use the same shape with `String` fields.

## 4. Where the design is silent or ambiguous

| Point | Recommendation |
| --- | --- |
| Whether "`#[codec(default)]`" means the field form, the container form, or both | Both, as serde |
| Which serde spellings beyond the four keys are accepted | None; spanned errors (1.4) |
| Variant `skip`, given the infallible `encode` | Reject (1.4) |
| Trait paths, and hand-written impls | Prelude names; hidden methods; no new rule (1.1) |
| `json::Error`'s API | Opaque, with `Display` and `line()`/`column()` (1.7) |
| Number rules, NaN, large integers, the depth limit, duplicate keys, the BOM | 1.5, 1.6, pitfalls 2 and 3 |
| Map key types and map output order | `String` keys; sorted `HashMap` output (1.9) |
| Which std types implement the traits | The list in 1.3; `Url` with its own bead |
| D6's "format plug-ins" against the companion's "plug-in API not in v0" | Not a conflict for this bead: JSON-only hidden methods keep the plug-in API open |
| `04-contracts.md`'s invariant check in `Decode` | Out of scope until the contract attributes exist; the derive's decode path has one obvious place for it, after the fields are built |

None of these recommendations adds a language rule. The points that change what the reference promises belong to the holder of `design/reference.md`, not to fx-08m: any wider key set, a `to_string`/`from_str` pair, or documented number rules. The implementer should record the choices on the bead and in the std docs.

DONE
