# Tide consistency review, round 48: confirmation of the round-47 fix (fx-1am)

## Verdict

**request changes**

Codex's B1 is closed for what it named: associated types and constants in
the trait impls written for a whole-sealed type. Its non-blocking note is
closed too. The reflow changed no wording. But the same escape, an API
change that no seal hash covers, is still open by three nearby routes:

- associated constants in inherent impls;
- a trait impl method's refined return type;
- the associated items of a blanket impl.

rustc accepts all three (see Validation). Each needs about one clause in
SEAL-2.

Reviewed the uncommitted working tree on 2026-10-20 against HEAD `5ec5c70`
("Decide the Pure status of std types that erase a payload (fx-cze)"):
`git diff -- design/ README.md` (SHA-256 `cda383dd…0eb044`). This is a
narrow confirmation round. Scope, as asked:

- the status of Codex round 47's B1 and its non-blocking note;
- whether the related escape remains for associated constants in inherent
  impls, and for trait impl methods' signatures;
- whether the edits add a contradiction or false statement, and whether the
  reflow of SEAL-1..6 changed any wording.

Nothing else was re-audited.

## Round-47 item status

| Item | Status | Evidence |
| --- | --- | --- |
| Codex B1 (`type Item = u8` -> `u16` in a sealed type's trait impl) | **Resolved** for impls written for the type | `04:1276–1282`: a whole entry fixes "the set of trait impl headers written for it, ... and the associated types and constants in those impls, with what their definitions depend on, so `type Item = u8` cannot become `u16`". Codex's `Feed` is a public item of the sealed module, so it is sealed whole (`04:1245`, `04:1283`). Its `impl Iterator for Feed` header and its `type Item` are both now in the hash, so the edit is `FX0920`. "What their definitions depend on" resolves names "as §11 does" (`04:1260–1263`), so changing `Token` in `type Item = Token` is also caught. The interface bullet's "seal the module that defines the type for that" (`04:1274–1275`) now gets associated items through the whole entry. Open variants: B1–B3 below. |
| Codex note (review and CI recompute every recorded hash) | **Resolved** | `04:1313–1316`: review and the CI gate check "against the base's seal, ... recomputing every hash it records, for every target and configuration, whatever the local build covers". There is no clash with SEAL-3's "in each configuration it builds" (`04:1299–1301`), which is about the local admission driver. It is also consistent with §11's gate (`04:1225–1227`) and SEAL-1's per-target hashes (`04:1242–1244`). |
| Reflow of SEAL-1..6 | **No wording change** | Two checks, below. |

**Reflow.** Codex's log holds its `nl -ba` read of `04:1233–1343` at diff
hash `762462…8b24`.

- **Word diff.** Against the current §11.1 up to "Worked example", the only
  word-level differences are:
  - "and the set" becomes "the set";
  - "is a change;" becomes "is a change, and the associated types ...
    cannot become `u16`;";
  - "applied." becomes "applied, recomputing ... whatever the local build
    covers.".
- **Hash.** I put that snapshot's SEAL-1..6 back into the current file and
  regenerated the diff. Its SHA-256 is exactly `762462…8b24`, Codex's
  recorded hash. So nothing else in any file changed since round 47.

Every §11.1 line still fits 79 columns.

**Trait impl method signatures, apart from B2.** A call to a trait method
type-checks against the trait's signature, with the impl's substitutions.
An impl that drops a where-clause or generalizes a lifetime is therefore
invisible to callers. Declared authority is computed from parameter types
(`03:561–564`), and those must match the trait's. So outside a refined
`impl Trait` return (B2), the trait fixes the signature callers see. That
holds if the trait is sealed or is std's, which the toolchain version pins.

## New blocking problems

### B1. Associated constants in inherent impls are sealed by nothing

- **Where.** `04:1245–1246` ("a module path seals each public item in it,
  methods of its inherent impls included"), and `04:1276–1282`, where a
  whole type's associated items are those "in those impls", meaning its
  trait impls.
- **Risk.** Neither D4 (`01:339–344`) nor reference 5.1 rejects
  `impl Feed { pub const MAX: usize = 10; }`.
  - The module seal names inherent *methods* only. An inherent associated
    constant is neither a module-level item nor a method.
  - The whole-type entry covers trait impls only.
  - A delegate can therefore change `MAX` to `0` with no `FX0920`, unless a
    sealed signature or clause happens to name it.
  - A parallel caller of `Feed::MAX` sees a changed sealed API, and SEAL-6
    (`04:1333–1342`) does not admit this gap.
- **Smallest fix.** In SEAL-1, write "methods and associated constants of
  its inherent impls included". Constants are always sealed whole
  (`04:1283`), so nothing else changes. Rust's inherent associated types
  are unstable, so constants are the whole case.

### B2. A trait impl method's refined return type is not in any hash

- **Where.** `04:1276–1282`. The whole-type entry fixes impl headers and
  associated types and constants, not the impl's method signatures.
  SEAL-1's module expansion (`04:1245–1246`) does not include trait impl
  methods.
- **Risk.** D3 allows `impl Trait` in return position "as in Rust"
  (`01:300–303`), so a trait method may return
  `-> impl Iterator<Item = u8>`.
  - An impl may refine that type to `-> std::vec::IntoIter<u8>`. Callers
    that know the type observe the refined type (rustc 1.96's
    `refining_impl_trait` is only a warning, and `f.items().as_slice()`
    compiles).
  - A delegate who changes the impl's return type back to `impl Iterator`,
    or to another concrete type, breaks parallel callers. The header,
    trait, and body hashes are all unchanged, so there is no `FX0920`.
  - This is the same escape as Codex's B1, because rustc lowers a
    return-position `impl Trait` in a trait to a hidden associated type
    whose value the impl's signature sets.
  - An orchestrator could seal `<Feed as Items>::items` by its qualified
    path (`04:1246–1247`), but nothing says it must.
- **Smallest fix.** In the whole-type clause, add "and the signatures of
  the methods in those impls" after "the associated types and constants in
  those impls". Their bodies stay free, as Codex's patch intended.
  Alternatively, D3 could reject refinement (deny `refining_impl_trait`),
  but that is a language change and larger.

### B3. A blanket impl's associated items are not "written for" the type

- **Where.** `04:1277–1281` ("the set of trait impl headers written for it
  ... the associated types and constants in those impls"). The trait
  clause is `04:1282` ("for a trait, its items, their contracts and
  default bodies, and its laws").
- **Risk.** D4 puts impls in the module of the type or the trait
  (`01:340–341`).
  - `impl<T: Marker> Bar for T { type Out = u8; }` in a sealed trait's
    module, with `impl Marker for Feed`, gives `<Feed as Bar>::Out = u8`.
  - That impl is not written for `Feed`, so `Feed`'s entry does not cover
    it. `Bar`'s whole entry covers its items, not its impls.
  - A delegate can change the impl to `type Out = u16`, or tighten its
    bound to `T: Marker + Debug`, which removes `Bar` from `Feed`. Neither
    change triggers `FX0920`.
  - The same applies to `impl Bar for Vec<Feed>` in `Bar`'s module.
  - This is Codex's B1 by another impl shape. It is open even when both
    modules are sealed.
- **Smallest fix.** Give the trait the rule the type has. In the trait
  clause, add "and, for the impls of it written in its module, their
  headers, associated types and constants, and method signatures". To
  keep adding such impls allowed, also add "adding one is not a change".
  The type side could instead say "trait impls that apply to it, blanket
  impls included". That needs trait solving inside the hash, and it would
  forbid any new blanket impl that covers a whole-sealed type.

One sentence can carry all three fixes, the whole-type clause then
covering both its own impls and its inherent impls.

## Non-blocking notes

### N1. Defaults inherited from an unsealed trait

- **Where.** `04:1280–1281` ("in those impls") and SEAL-6 (`04:1335–1336`).
  SEAL-6 tells the orchestrator to seal "the modules that define the types
  its interfaces name", and does not mention traits.
- **Risk.** A trait impl that inherits an associated constant's default
  does not define it "in" the impl. If the trait's module is unsealed, a
  delegate can change the default, and with it `<Feed as T>::C`, or the
  trait's method signatures. This is the general unsealed-dependency limit
  rather than a new hole, but SEAL-6 states it for types only.
- **Smallest fix.** In SEAL-6, write "the modules that define the types and
  traits its interfaces name or its sealed types implement".

## Validation

Only this file was written; nothing was staged or committed.

- **Read:** AGENTS.md; `reviews/round47-codex.md`, `round46-consistency.md`,
  and `round41-consistency.md` (format); `04:1205–1384`; `01:296–344` (D3,
  D4) and `01:733–753` (D11's bullet); reference 3.5 and 5.1–5.2;
  `03:557–600` (§4.1); companion §18 (`companion:466–490`), which is
  consistent with the edits.
- **Reflow and hash:** the round-47 text came from Codex's `nl -ba` read
  in the author's `codex.log`. I diffed it word by word and recomputed the
  diff hash with Python and `git diff`/`git hash-object`.
- **Witness:** compiled with
  `rustc +1.96.0 --edition=2024 --crate-type=lib --emit=metadata`, stdin
  closed, 60 s timeout. Exit 0, with only the `refining_impl_trait_reachable`
  warning. It covers:
  - a refined RPITIT impl, with a caller that uses `.as_slice()` on its
    result;
  - `<Feed as Bar>::Out` resolved through a blanket impl;
  - an inherent `pub const MAX` read by another function.

  This proves only what rustc accepts. Tide's verdicts are deductions from
  the text, since no checker exists.
