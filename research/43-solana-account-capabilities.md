# Account authority as capabilities for Rust smart contracts

fx-qlek.17 · Research record · 2026-11-19 UTC · Author: Codex

**Five registered cases were examined in a paper comparison. Two pairs
support the proposed rule, conditional on declaring as policy the account
relationship each later fix added. Three pairs are incomplete, including
Cashio, for which only the buggy-side rejection could be assessed.** In C02
(Wormhole), the trusted operation library owns the exact-address requirement,
which the Solana SDK's `*_checked` loaders already enforce.

The proposed rule requires exact account identities and relationships at
sensitive operations, in addition to signer, owner and program-derived
address (PDA) checks. Correct policy recipes remain trusted input. Tide v0
implements none of this Solana profile, and **no result is a Tide v0 catch**.
Integration with SBF (Solana's bytecode format) and practical benefit remain
unproved. §2.4 records the alpha disposition. §4 preserves the frozen
protocol and all case outcomes.

## 1. Prior art

The sources below support the stated mechanisms, not a claim that a
framework or analyzer prevents every exploit. Read date for §1–§3 external
sources: **2026-11-18 UTC**. The [source manifest](protocols/fx-qlek.17-cases/sources.json)
gives exact retrieval times, URLs, versions/revisions where available and
SHA-256s of saved HTTP bodies. A live documentation page with no release
identifier is labeled an unversioned snapshot; it is not evidence about
all releases. Pinned source examples below are selected versions, not
claims that they are the latest. Proposed gaps are this author's
inferences, labeled as such. No analyzer was run.

### 1.1 Move: Sui and Aptos

Move's `copy`, `drop`, `store` and `key` abilities govern duplication,
discarding and storage of values. In Aptos's Move documentation, the VM
creates `signer` values; user code cannot manufacture one from an address.
`move_to` needs a signer reference to publish under that signer's address.
Aptos's framework also has a `SignerCapability` and APIs to delegate
signer creation. Thus the platform already supplies an authenticated
principal as a value; this proposal does not originate that idea.
[Move abilities][aptos-resources-redirect], [Move signer][aptos-signer-redirect],
[Aptos account module][aptos-capability-corrected].

On Sui, the runtime restricts use of address-owned objects to their owner;
shared objects need application authorization. The Move Book demonstrates
an `AdminCap` object passed as an argument, with creation in the defining
module and authority controlled through possession. These are already
capabilities in signatures, with a type system and runtime enforcing the
boundary. [Sui object ownership][sui-objects-redirect],
[The Move Book, Capability][sui-capabilities].

**Inferred gap for this proposal:** bringing a similarly compulsory
boundary to existing Solana Rust programs. There is no missing
“capability-value” mechanism to supply to Move itself. Applications on
either Move platform still choose which operation requires which
capability, which object instance it authorizes, and who receives it.
Sui's warning about shared objects makes that policy obligation explicit.
A wrongly issued AdminCap or a correct signer checked against the wrong
policy remains an application error. [Capability pattern][sui-capabilities],
[Sui ownership warning][sui-objects-redirect].

### 1.2 Anchor

Anchor already provides much of the proposed API. `Signer<'info>` checks
signer status; `Account<'info, T>` validates account ownership and invokes
T's checked deserializer. For Anchor account types the generated
deserializer checks the discriminator. `seeds`/`bump` validate a PDA,
`address` an exact key, `owner` an expected owner, `has_one` a stored key's
relationship to another supplied account, and `constraint` an explicit
predicate. These are runtime checks generated from declarations, with
typed values handed to the handler. [Account types][anchor-types],
[constraints][anchor-constraints], [Account implementation, v0.31.1][anchor-account-source],
[constraint code generation, v0.31.1][anchor-constraints-source],
[account deserializer generation, v0.31.1][anchor-discriminator].

A correctly constrained Anchor handler already gets the proposed signer,
owner and PDA checks. Tide cannot claim to add those checks to that
handler. Anchor also exposes `UncheckedAccount` and `AccountInfo`, which
require the author to supply validation; `Signer` by itself says nothing
about whether that signer is the intended authority. A program-owned
`Account<T>` identifies a type and owning program, not a particular pool
or collateral mint. The author still declares account relationships and
trust roots. [Account types][anchor-types],
[constraint declarations][anchor-constraints].

**Inferred gap:** a restricted application boundary could require use of
the checked operations throughout the call graph, forbid unchecked escape
routes except as visible trust holes, and compare declared authorization
recipes across upgrades. That is an enforcement/maintenance hypothesis.
It adds value only if it closes paths that existing typed Anchor code,
review, tests and analyzers leave open at an acceptable migration cost.
This research does not establish that cost or benefit.

### 1.3 Solana analyzers and Neodyme's checks

Sec3's 2022 X-Ray announcement calls the product formerly Soteria and
shows missing-owner findings in both native and Anchor examples. It
advertises multiple bug classes and CI integration. Those are vendor
claims about a scanner, not a proof that a clean report implies account
safety. Do not confuse this Solana product with unrelated projects also
named Soteria. [Sec3 X-Ray announcement, 2022-05-24][sec3-scanner].

Neodyme's 2021 *Common Pitfalls* gives owner, signer and invocation
validation checks. It explicitly recommends converting untrusted
`AccountInfo` to a distinct trusted type after validation. That is direct
prior art for the core sketch. This source describes checks and coding
discipline, rather than a complete static analyzer. We should credit the
pattern instead of presenting it as Tide's invention.
[Neodyme, 2021-08-20][neodyme-pitfalls-redirect].

VRust describes annotation-free, MIR-based inference rules for Solana
account vulnerabilities and requires a particular rustc nightly in its
README. Ackee's Solana extension documents detectors including missing
signer and improper sysvar access; it describes itself as an extra review
aid. Their published descriptions show existing automation for this
problem class; no comparative accuracy or current buildability was
measured here. [VRust README][vrust], [Ackee extension README][ackee-analyzer].

**Inferred gap:** the proposed profile would constrain which operations
application code can express, rather than only flag suspicious uses of a
permissive API. That distinction does not establish stronger practical
coverage. It shifts responsibility to the trusted API, policy recipes and
admission gate, and can reject useful programs. A fair later comparison
must credit any existing tool that detects the same missing predicate.

### 1.4 CosmWasm

CosmWasm supplies `MessageInfo.sender` and attached funds at instantiate
and execute entry points. In `cosmwasm-std` v2.2.2 the source specifies
that x/wasm ensures the sender signed or otherwise has authority to send
the message. The contract receives authenticated call context rather than
trusting a sender address inside its own message payload. The type has
public fields, so provenance comes from the runtime entry point, not
unforgeability of a Rust struct inside arbitrary helper code.
[MessageInfo, v2.2.2][cosmwasm-info].

**Inferred gap:** authenticated caller context does not choose which
caller may perform an application operation. A restricted internal API
could carry a checked role into helpers, but the Solana-specific raw
account/PDA problem is not a demonstrated missing CosmWasm feature.
Nothing here establishes a reason to change CosmWasm.

### 1.5 ink!

ink! v5 exposes environment functions such as `self.env().caller()` and
typed contract references for cross-contract calls. The runtime caller
value lets application code compare an identity with a stored authority.
Typed call encoding supplies an interface; it does not establish
that an arbitrary contract address is the application's intended
counterparty. [ink! v5 environment functions][ink-env-corrected],
[v5 cross-contract calls][ink-access].

**Inferred gap:** an internal API could require a verified role value
instead of repeatedly using caller comparisons. This is a library and
policy-discipline question, not evidence that ink! lacks authenticated
callers or needs Solana owner checks.

### 1.6 Stylus

The Stylus Rust SDK provides typed persistent storage and access to call
context, including the message sender. The storage API models EVM storage
layout; it does not use Solana's model of arbitrary account data supplied
to each instruction. Caller identity still needs application policy to
grant a role. [Stylus Rust SDK guide][stylus-msg],
[SDK host context][stylus-host].

**Inferred gap:** mandatory role-bearing internal parameters could help
preserve authorization boundaries across helpers or changes. That
would not invent caller authentication, and this research offers no
case evidence for Stylus. Runtime/SDK memory and reentrancy constraints
remain separate review obligations.

### 1.7 Soroban

Soroban's host-managed `Address` offers `require_auth` and
`require_auth_for_args`. The host authenticates authorization for the
invocation context and supplies replay protection; the contract chooses
which address must authorize which operation. In SDK v22.0.0 these calls
return unit on success, rather than a role token that a Rust helper must
receive. [Soroban authorization][soroban-auth],
[Address source, v22.0.0][soroban-address].

**Inferred gap:** a wrapper could retain that established authorization
as a restricted value for helpers. A forgotten `require_auth` remains
possible because the application decides where it belongs; the proposed
Solana rule faces the same policy-selection problem unless sensitive
operations require an appropriate proof. Soroban already handles the
harder signature-context and replay mechanics described above.

### 1.8 Agoric

Agoric's Hardened JavaScript guide treats an object reference as authority
and uses compartments to restrict ambient globals. Its write/read facets
illustrate delegation and least authority. Zoe adds escrow and offer
safety: when an offer exits, the user gets the requested allocation or
the escrowed assets back, subject to the proposal's terms. This supplies
no liveness guarantee. Neither mechanism claims that an application chose
a good price or profitable trade.
[Hardened JavaScript][agoric-ocap-redirect], [Zoe][agoric-zoe].

**Inferred gap:** there is no capability-model gap to fill here. Agoric
already builds contracts around object capabilities. The proposed work
would apply that established discipline to Rust/Solana's validation
boundary and explore an upgrade report; it does not supersede Agoric's
runtime isolation or Zoe's asset guarantees.

## 2. Sketch: authority values at the handler boundary

Everything in this section is a **proposal**, including the pseudocode.
The frozen operational rule is [protocol §3, R1](protocols/fx-qlek.17-protocol.md#3-frozen-proposed-rule-r1).
Opus has not approved a language or implementation change. The starting
mechanism is Neodyme's checked-type recommendation and Anchor's existing
account wrappers, not a new resource calculus. [Neodyme][neodyme-pitfalls-redirect],
[Anchor Account][anchor-account-source].

### 2.1 API and owners of the invariants

A trusted entry adapter receives raw runtime accounts and instruction
bytes. A small checked-account library owns constructors and the
operations that use those accounts. Each constructor either validates
its predicate and returns an invocation-bound value, or returns an error
before a sensitive operation. The admitted application gets those values
as arguments. A policy recipe, reviewed as trusted input, specifies the
expected program, layout, seeds and authority relationships.

D7 governs host authority that Tide passes to functions as values such as
`Fs`, `Net` and `Clock`.
R1's hypothetical `Signer`, `Owned<T>` and `Pda<S>` would carry
runtime-validated facts about caller-supplied accounts for one invocation.
The two designs share the discipline of passing authority as parameters.
[D7](../design/01-design.md#d7-authority-capabilities-as-values-no-ambient-authority),
[reference §6.1](../design/reference.md#61-the-rule-and-the-capability-set).

Illustrative signatures, not compilable Tide v0 or a claimed new library:

```text
trusted entry adapter:
    state = owned<State>(raw_state, expected_program)?
    authority = signer(raw_authority)?
    vault = pda(raw_vault, declared_seeds(state), expected_program)?
    permit = authorize_withdraw(state, authority, vault, policy)?
    withdraw(permit, amount)

application handler:
    withdraw(permit: WithdrawPermit<'inv>, amount: Amount) -> Result
```

`WithdrawPermit` retains the exact validated state/vault/signer bindings
and exposes only the intended operation. `Owned<T>` attests ownership and
checked decoding, `Signer` attests signer status for one account key,
and `Pda<S>` attests a derivation and seed recipe. An exact-address value
can identify a declared program or sysvar. A relationship proof ties
account instances together. None of those facts alone means “the user
is allowed to withdraw from this pool.” The authorization constructor
must check that policy against authenticated state and an independent
trust root. Equality between caller-chosen accounts is insufficient.

The invocation boundary owns provenance and lifetime. The operational
library owns exclusive writes, checked cross-program invocation (CPI)
targets and invalidation of proofs after mutations or CPI. The policy
recipe owns which principal and relationships the operation requires.
Admission owns confinement:
application code cannot forge values, obtain raw handles, deserialize
unchecked state or invoke an unvalidated target by another route. A
constructor bug remains inside the trusted computing base; putting it
in a wrapper does not make admission detect it.

### 2.2 What admission would check

R1 would check constructor visibility and resolved calls/types across
application code, generated code and admitted dependencies. It would
require proof-bearing entry signatures and confine the raw runtime
adapter to a named trusted boundary. It would forbid public fields,
user-defined deserialization, conversions or other routes that fabricate
proofs, and record `unsafe`/foreign/raw escapes as trust holes instead
of promising capability safety through them. It would preserve the
key, lifetime and access mode through helpers; a proof about A cannot
be used for B.

Runtime constructors still perform signer, owner, address, seed and
relationship comparisons. Static admission checks that the program
cannot skip those constructors on a protected operation; it does not
statically predict arbitrary incoming account bytes. Invalid layouts,
wrong owners, wrong keys/seeds, missing required signatures and policy
mismatches fail at that runtime boundary. An account can be genuinely
owned and correctly typed but still be the wrong instance. A privileged
CPI also needs a validated target identity and exact signing seeds.

A proof based on mutable data expires after a relevant change, close,
reinitialization or CPI. Reload and recheck before reuse. An implementation
must handle duplicate account aliases and error paths; a paper signature
sketch does not prove those properties. These obligations and the limits
on claiming a case rejection are explicit in R1.

### 2.3 Trust diff for upgrades

Proposal: compare per-instruction authorization recipes and named trust
holes, keyed by stable instruction identity. Report removed signer/role
requirements, changed owner/program ids, changed seed inputs or bump
policy, weakened account relationships, changed CPI targets, broader
write access and new unchecked paths. Bind the report to source, SDK,
compiler, policy and final SBF artifact identities; unknown or unsupported
expressions require review rather than an “unchanged” verdict.

This is broader than today's exported-type authority diff. Tide's current
artifact records declared authority and unsafe sites, with no exercised-
authority or proof-status measurement. It has no Solana predicate schema.
[Current driver](../docs/driver.md),
[artifact owner](../crates/tide-admit/src/interface.rs).

An unchanged recipe can still conceal a malicious implementation change,
a wrong pre-existing predicate, storage migration error or economic bug.
The report does not authorize an on-chain upgrade or control its signing
key. Evidence that deployed bytes match checked bytes would be a separate
requirement. Tide's E3 pilot found no review benefit for its current trust
diff; no benefit is established for this proposed extension.
[Current evidence and limitations](../README.md#e3-does-the-trust-diff-help-reviewers-pre-registered-pilot-negative).

### 2.4 Choices and disposition

The options and recommendations below describe the author's proposal. R1
freezes one candidate for evaluation. The disposition follows the list.

1. **Checked values or check-recognition lints.** Options: recognize
   handwritten predicates; require checked values at sensitive operations;
   leave the existing Anchor discipline alone. Recommend checked values
   for this paper model, reusing the Anchor/Neodyme pattern, because the
   operation can own its proof requirement. A lint remains a credible
   lower-cost outcome if enforcement adds little.
2. **Whole Solana program or restricted handlers.** Options: admit every
   Anchor/native program; restrict handlers behind a trusted adapter;
   create a separate language. Recommend restricted handlers and a small
   named adapter. Full Anchor compatibility would widen the trusted and
   admitted surface before feasibility is known.
3. **Proof granularity.** Options: independent signer/owner/PDA wrappers;
   compound operation permits; user-defined refinements. Recommend a few
   primitive validated values composed into operation permits. This keeps
   instance relationships at the operation that needs them; independent
   wrappers alone cannot establish authorization.
4. **Policy source.** Options: infer policy from existing code; require
   explicit trusted recipes; add a general specification language.
   Recommend explicit recipes with known predicates and an authenticated
   root. Inference cannot recover an omitted intended check. Do not
   claim a general solver or automatically correct business policy.
5. **Proof lifetime.** Options: invocation-wide validity; invalidate on
   relevant writes/CPI; prohibit all CPI. Recommend invalidation and
   revalidation. Invocation-wide tokens can retain stale facts; banning
   CPI discards a central Solana operation. Unsupported lifecycle paths
   stay explicit feasibility gaps in this pass.
6. **PDA bumps.** Options: require canonical bumps everywhere; check the
   declared bump/derivation; ignore bumps. Recommend canonical derivation
   by default and verification of a specifically declared bump when the
   policy requires one. Report that choice in the recipe; a PDA check
   must include program identity and origins of dynamic seeds.
7. **Upgrade comparison.** Options: exported-type diff alone; add a small
   authorization-recipe diff; attempt semantic equivalence. Recommend a
   versioned recipe diff plus artifact identities. Do not equate textually
   unchanged recipes with behavioral equivalence.
8. **Compiler route.** Options: adapt the authoritative SBF compiler's
   driver; analyze on a separate host compiler then build; assume today's
   Tide driver supports SBF. Recommend investigating the first route in
   a later bounded spike. Host-only checking can miss target-specific code
   and cannot attest the delivered SBF binary. §3 records the obstacles.
9. **Current disposition.** Options: implement before alpha; research
   further; decline after this pass. Recommend leaving the alpha unchanged
   after this registered paper comparison. Prior art is substantial; §4
   supplies conditional shape results, not compatibility, implemented
   enforcement or practical benefit.

**Disposition.** Tide-lead adopted choice 9 in the 2026-11-19 Opus review
round, leaving the alpha unchanged after this paper comparison. Choices
1–8 remain the author's recommendations for a later pass and were not adopted.

### 2.5 Limits

This proposal cannot certify general business logic, arithmetic,
rounding, oracle truth/freshness, market prices or economic solvency.
Checking an oracle account's identity does not make its price reliable.
A valid signer may hold a stolen key; a valid upgrade authority may
approve malicious code. It cannot infer the right collateral mint or
privileged role from a function name. It relies on correct runtime facts,
trusted constructors and recipes, confinement of bypasses, and compiler
soundness. These are proposed obligations and acknowledged limits, not
measurements of Tide v0.

## 3. Feasibility from documentation and cheap inspection

Inspection base: `a34ffe25eac689420ce9a98e876f28542cd36a37`, Linux worktree,
2026-11-18. No compilation, upstream code execution, spike or live-chain
query was performed. “Feasible” below means an architectural possibility;
no cost, compatibility or performance claim has been measured.

### 3.1 SBF and the SDK

`solana-program` v2.3.0 documents both host and on-chain compilation,
using `cfg(target_os = "solana")` for the latter. Programs use an SBF VM
and SDK interfaces for account access, syscalls and CPI. The platform-tools
v1.58 build script ships `sbpfv3-solana-solana` alongside earlier target
variants; its exact compiler/sysroot matters, not a generic “BPF” flag.
[Target distribution, v1.58][platform-build]. Host metadata
checking alone therefore cannot cover all delivered on-chain paths.
[solana-program v2.3.0][solana-program].

The SBF build machinery uses a dedicated platform-tools compiler and
sysroot. A compiler-private admission driver must match that compiler's
internal crates, target and ABI; it cannot link arbitrary `rustc-dev`
libraries from Tide's unrelated pin and claim equivalent SBF checking.
[Platform tools][platform-tools], [platform-tools v1.58 build script][platform-build],
[SBF builder documentation][sbf-builder-readme].
This compatibility requirement is an inference from the two compiler
architectures, not a failed build experiment.

### 3.2 Today's driver does not accept this workload

The local pin is `nightly-2026-08-09`, with `rustc-dev` and `rust-src`;
the pin's comment identifies rustc commit `771916f90`. The driver uses
`rustc_private`, owns compiler arguments and inspects resolved/typed
code and runnable instances. Its documented interface accepts one Rust
entry and `lib|bin`, rejects raw rustc flags, and supports linked outputs
on x86_64 Linux. It rejects user macro definitions (`TD0101`), derives and
bang forms outside the closed sets (`TD0102`, `TD0103`), and `#![no_std]`.
[Toolchain pin](../rust-toolchain.toml),
[driver architecture](../crates/tide-admit/src/main.rs),
[current interface](../docs/driver.md).

Those facts rule out “just run current tide-admit on an Anchor program.”
It has neither an SBF deployment path nor arbitrary SDK/dependency/macro
admission. The existing authority policy names Tide's capability types;
it has no signer/owner/PDA operations or proof constructors. This is an
inspection of current interfaces, not proof that the architecture cannot
be adapted. [D18](../design/01-design.md#d18-a-checked-dialect-of-rust-v0-direction),
[policy owner](../crates/tide-admit/src/policy.rs).

### 3.3 Anchor expansion

Anchor v0.31.1's `#[program]` is a procedural macro. Its `Accounts`
derive generates a `try_accounts` implementation, extracts account
values and emits validation through the constraint generator. The
checked `Account<T>` constructor verifies owner and deserializes T.
These are inspection-backed opportunities for resolved-code checking
and a trusted wrapper boundary. [Program macro][anchor-macro],
[Accounts derive][anchor-accounts-derive], [try_accounts generator][anchor-tryaccounts],
[constraint generator][anchor-constraints-source], [Account][anchor-account-source].

They do not imply that today's macro allowlist admits Anchor or that
post-expansion checks alone prove provenance. A future experiment would
pin the macro crates and their dependencies, examine the expansion on
the SBF target, and follow unchecked paths and CPI calls. Generated
validation inside a trusted adapter remains trusted code that must be
reviewed, not an admission success to count. Supporting all existing
Anchor wrappers is a separate scope and cost choice.

### 3.4 Toolchain pinning and an SBF std

A future compiler route needs an exact platform-tools/SBF compiler
revision, target features, sysroot, SDK, Anchor expansion and lockfile.
A host prepass followed by a different compiler needs an additional
argument that both passes check/build the same target-specific program;
this record supplies none. The costs include maintaining compiler-private
hooks across upgrades and auditing changes to admitted APIs.
[Platform tools][platform-tools], [SBF builder documentation][sbf-builder-readme],
[Tide pin](../rust-toolchain.toml).

An SBF std would expose checked account data, invocation context, program
calls and deterministic permitted computation. It would not simply
reuse Tide's current host std, which exposes host filesystem/network,
tasks, processes and a simulator through its own capabilities. Nor does
Solana's restricted Rust support imply that no Rust std code exists on
that target: the SDK documents target-specific availability. A small
trusted SDK facade over the target's supported core/alloc/runtime is the
recommended starting hypothesis, with allocation, panic, stack/heap,
logging and compute costs to investigate before any implementation claim.
[SDK target differences][solana-program], [Tide std surface](../docs/driver.md).

**Recommendation:** treat SBF integration as unproven and maintenance-heavy
until a bounded, separately authorized target-specific spike checks one
handler and one generated validator with the exact SBF compiler. This
research adds no spike or dependency. The registered case comparison is
a paper model and does not prove compiler integration.

## 4. Registered evidence and results

The [protocol](protocols/fx-qlek.17-protocol.md),
[fixed case list](protocols/fx-qlek.17-cases/cases.tsv),
[original-entry ledger](protocols/fx-qlek.17-cases/entries.tsv),
[selection ledger](protocols/fx-qlek.17-cases/selection.tsv) and
[source snapshot manifest](protocols/fx-qlek.17-cases/sources.json) freeze
this pass before assessment. Under protocol Amendment A1, the frame is
the union of the pinned Copenhagen0x catalog (11 entries), solsec's pinned
Exploits section (17 links), and Sec3's 2022-09-06 review (five incidents).
The first one-catalog candidate was not stamped and was amended at the
coordinator's request before any case assessment.

The 33 source entries map to 22 selection records: 20 incidents/disclosures
and two non-incident links. One compound report describes both SolSea and
SPL Token, so its entry maps to two protocol-specific records; Royal Flush
names their combination rather than a third independent incident.
Twelve repeated entry-to-incident mappings merge into existing records.
Each merge, root-cause selection quote or exclusion reason appears in the
ledgers. Incident dates remain unknown when the source gives only a
publication date. Shared names/dates and explicit same-incident references
support merges; undated reports are not silently assigned publication dates.

Five fixed cases meet the registered cause rule: Solend, Wormhole, Cashio,
Crema and SolSea. Seventeen records are not selected, including the two
non-incident links. The SolSea report states a missing signer check; its
SPL Token closure defect and Candy Machine's reinitialization defect are
recorded separately as causes outside the selected classes. These are
source-based selection decisions, not R1 verdicts. The manifest identifies
raw HTTP bodies versus saved web-tool extractions; the latter are not
original-HTML byte attestations. [Selection evidence](protocols/fx-qlek.17-cases/selection.tsv),
[snapshot identities](protocols/fx-qlek.17-cases/sources.json).

### 4.1 Registration, proof and reading chronology

The normative rule and selection procedure are the [protocol at
384d775](https://github.com/luccahuguet/tide/blob/384d7758cb7a0431bc69c3d6ee04ab2c6e744045/research/protocols/fx-qlek.17-protocol.md),
including Amendment A1, SHA-256
`110a06ce0d2f68a191196123f305cd3201fe4b33e441a25668f6c05b2e9a2f1b`.
The local [protocol](protocols/fx-qlek.17-protocol.md) and its five-case
list/ledgers/manifest remain byte-identical to that registration.
The [assessment manifest](protocols/fx-qlek.17-assessment-sources.json)
records post-go sources, exact code revisions, retrieval/read chronology,
errors and local artifact hashes separately from the frozen manifest.

| Event | Identity / UTC |
| --- | --- |
| Initial one-catalog registration, superseded before stamping | `ef356f9f5c757cc4846c80b2583b767c37700f0a` |
| A1 union frame registered, zero cases assessed | `384d7758cb7a0431bc69c3d6ee04ab2c6e744045`, 2026-11-18 |
| Six completed proofs committed by coordinator | `e245199610ec72b023554a0b17238240910f8985` |
| Coordinator reports Bitcoin anchoring | block 976490, complete before go |
| Explicit coordinator go | 2026-11-18T23:48:32Z |
| Builder checks six original subject hashes and proof-header digests | 2026-11-18T23:49:40.912834Z |
| First assessment begins / last first assessment recorded | 2026-11-18T23:50:17.469397Z / 2026-11-19T00:06:48.723622Z |

Proofs are adjacent `.ots` files for the six registration subjects:
the research draft, protocol, cases.tsv, entries.tsv, selection.tsv and
sources.json. Their full hashes are in the assessment manifest. Bitcoin
verification is **reported by tide-lead**, not independently verified by
this builder: no live-chain request was made. The local check establishes
subject identity and header-digest agreement, not a full OTS verification.
The research draft's proof attests its bytes **at 384d775**, not this
completed article; recover that subject with `git show 384d775:research/43-solana-account-capabilities.md`.
No existing proof was replaced to attest post-go results.

Pre-go reading included incident narratives, search summaries and repair
descriptions needed for selection. No fix diff was opened or verdict
assigned then, as self-attested in A1. This was retrospective, not blind.
Timestamps can attest bytes existed; they cannot attest the author's
reading or private reasoning. Post-go cases were assessed in frozen order
C01–C05; each start was logged before reading for its assessment.

### 4.2 Results and denominators

These are paper judgments under **proposed R1**, conditional on correctly
implemented trusted constructors and correctly declared operation policy.
No Tide compiler, SBF program or exploit was run. The comparison does not
show that admission could discover the right recipe from legacy code.
Putting a missing check inside the trusted adapter, choosing the wrong
recipe or permitting a bypass would defeat the credited guarantee.

| Frozen case | Buggy verdict | Fixed verdict | Pair and reason |
| --- | --- | --- | --- |
| C01 Solend, 2021-08-19 | rejects buggy shape | admits fixed shape | Conditional support: reserve-to-market relationship missing; signer, native owners and PDA already checked. Enabled handler pair. |
| C02 Wormhole, 2022-02-02 | rejects buggy shape | admits fixed shape | Conditional support: byte-only instruction decoder cannot authenticate sysvar identity; fixed SDK calls check its exact key. Enabled path pair. |
| C03 Cashio, 2022-03-23 | rejects buggy shape | not assessable | One-sided rejection for the missing bank-to-output-mint relationship. The compiled fixed action bodies need an R1.6 CPI/reload lifecycle this pass did not model. |
| C04 Crema, 2022-07-02 | not assessable | not assessable | No identified public historical Claim implementation and matched repair; exact predicate and Rust identity unverified. |
| C05 SolSea, incident date unknown | not assessable | not assessable | Disclosure lacks SolSea Rust handler/fix. The separate SPL Token repair cannot substitute. |

Source identities, mapped checks and derivations for every row follow
below. Total frozen **5**, examined **5**, pair-assessable **2**, incomplete
**3**, paired support **2**, assessable negatives **0**. Both supporting
pairs have enabled source paths. The buggy side has 3 rejections, 0
admissions and 2 not assessable. The fixed side has 2 admissions, 0
rejections and 3 not assessable. Unknowns count as neither success nor
negative. No case was backfilled, dropped or selected by its verdict.

The assessed pairs cover **instance relationships/substitution** (Solend)
and **exact sysvar identity/substitution** (Wormhole). Cashio adds a
one-sided observation about instance relationships. These results do not
establish a standalone missing-signer, generic native-owner, PDA-bump or
type-discriminator catch. SolSea supplies a signer-role narrative but no
assessable code pair; Crema supplies an owner/spoofing narrative but no
assessable code pair. Genuine type/owner wrappers already occur in the
assessed source. No catch rate, prevalence, prevented loss or superiority
over Anchor/Move/analyzers can be inferred from this small labeled frame.

### 4.3 Case derivations

#### C01. Solend, 2021-08-19: relationship proof, not missing signer/owner

Assessment began 2026-11-18T23:50:17.469397Z, after go. Rooter and Nope's
dated incident report says the attacker used a newly created market to
subvert authorization; its source code figure highlights an added
reserve-to-market comparison. Short symptom quote: “make accounts with
borrows liquidatable”. The report describes a thwarted theft attempt and
refunded liquidations, not funds stolen by the attacker. This corrects
the selection catalog's loose loss/owner framing without changing the
frozen case. [C01 report][c01-report].

The matching Rust fix is `132d74cf171dac66f896c4009ad6836f8c0b3799`, first
parent `871935cafc590bf1985bebfbc7e6399b153a5b40`, in
`token-lending/program/src/processor.rs`. The commit changes only four
inserted lines in `process_update_reserve_config`; both report figure and
patch reject a market key unequal to the market stored in the reserve.
This is a source-pair identification, not an attestation of deployed bytes.
[C01 fix][c01-fix], [buggy handler][c01-bug], [fixed handler][c01-fixed].

| Source map | Preserved computation and proposed R1 value |
| --- | --- |
| Bug/fix `processor.rs` 124–126, 1977–1987 | Dispatch and four runtime accounts; retain config validation and error returns. |
| Bug 1989–1997; fix 1989–1997 | Reserve decoding and program-owner check supply `Owned<Reserve>`; decode before owner check does not release a capability early. |
| Bug 1999–2015; fix 2003–2019 | Decode/owner-check market, match its stored owner to the supplied key, require that signer: `Owned<Market>` plus signer bound to that market. |
| Bug 2017–2028; fix 2021–2032 | Verify PDA using market key, its stored bump and program id; preserve the specified bump rather than adding canonicality. |
| Fix 1998–2001 only | The added equality supplies `ReserveBelongsToMarket(reserve, market)`. The buggy side has no corresponding check or proof. |
| Bug 2029–2031; fix 2033–2035 | Replace config and pack the same reserve; consume the write permit. No intervening CPI or subsequent use of a stale proof in this handler. |

The intended policy, common to both sides, permits a market's authenticated
authority to change that market's reserve. The root is the runtime program
id and authenticated reserve's recorded market, not whichever market the
caller supplies. A proposed update operation requires the relationship
proof together with the existing ownership/signer/PDA proofs. R1.5 blocks
that operation on the buggy path: correct proofs for an attacker's *other*
market cannot establish the missing relationship. The fixed path supplies
it. Model delta: one validated relationship, corresponding to the sole
source insertion; no extra principal or repair is introduced.

**Buggy: rejects buggy shape. Fixed: admits fixed shape. Pair: support for
the proposed rule on this handler path.** This is conditional on correct
trusted constructors and the declared reserve-specific policy. Moving the
omission inside a trusted constructor or declaring the wrong policy would
not be a rejection credited to R1. A generic raw-account ban is not the
reason for this verdict.

Mapping limits: source `Pack` types use checked layouts and initialized
version fields, not Anchor discriminators; they remain those native
layouts. State helpers at the fixed commit are unchanged by the one-file
fix. [Reserve layout][c01-reserve], [market layout][c01-market].
The model preserves validation helpers as ordinary computation and
does not prove interest math, reserve creation, global lifecycle rules,
or the trusted API's enforcement. No model was compiled or executed.
An Anchor `has_one`/equivalent explicit relationship constraint can express
the same check; R1 does not invent that protection.

[c01-report]: https://docs.google.com/document/d/1-WoQwT1QrPEX-r4N-fDamRQ50LM8DsdsOyq1iTabS3Q/edit
[c01-fix]: https://github.com/solendprotocol/solana-program-library/commit/132d74cf171dac66f896c4009ad6836f8c0b3799
[c01-bug]: https://github.com/solendprotocol/solana-program-library/blob/871935cafc590bf1985bebfbc7e6399b153a5b40/token-lending/program/src/processor.rs#L1977
[c01-fixed]: https://github.com/solendprotocol/solana-program-library/blob/132d74cf171dac66f896c4009ad6836f8c0b3799/token-lending/program/src/processor.rs#L1977
[c01-reserve]: https://github.com/solendprotocol/solana-program-library/blob/132d74cf171dac66f896c4009ad6836f8c0b3799/token-lending/program/src/state/reserve.rs
[c01-market]: https://github.com/solendprotocol/solana-program-library/blob/132d74cf171dac66f896c4009ad6836f8c0b3799/token-lending/program/src/state/lending_market.rs

#### C02. Wormhole, 2022-02-02: authenticate the instruction source

Assessment began 2026-11-18T23:55:07.066552Z. The registered Halborn report
describes a forged Instructions sysvar and the resulting signature bypass;
short symptom quote: “bypass signature verification”. The relevant
principal is the runtime's instruction-history account, not an ordinary
transaction signer. [C02 report][c02-report].

The resolved public change is `7edbbd3677ee6ca681be8722a607bc576a3912c8`,
first parent `79ab522f802ccc5ba34278d3c648fa62e06f4f1c`. It replaces both
current-index and instruction-at-index byte loaders with checked account
loaders in `solana/bridge/program/src/api/verify_signature.rs`. A mistyped
full SHA initially returned 422; resolving the documented `7edbbd3`
prefix identified this exact commit. The failed lookup is retained.
[C02 fix][c02-fix], [buggy source][c02-bug], [fixed source][c02-fixed].

| Source map | Preserved computation and proposed R1 value |
| --- | --- |
| Both handler versions 22–35 | Preserve payer, guardian set and signature-set roles. `instruction_acc` is plain `Info`, which Solitaire aliases to `AccountInfo`; its declaration is not an identity proof. |
| Both 73–90 | Preserve guardian PDA derivation and signer-index collection. |
| Bug 92–105 / fix 92–105 | Both reads must use a value bound to the Instructions sysvar identity, `Address<InstructionsId>` (R1.4). The old byte slice API cannot supply it; each new loader validates the account key. |
| Both 107–171, 198–215 | Keep secp program-id, index/message/address comparisons and signature bookkeeping unchanged; do not replace them with a generic `Signer`. |
| Both 173–196 | Preserve existing/new signature-set branches and the fixed-target System-program creation CPI. Account creation is an explicit trusted runtime operation, not a raw arbitrary CPI escape. |
| Solitaire `peel.rs`, Data/Signer/Mut/Info implementations | Preserve its signer, owner, initialized-state and mutability checks. `Info` checks mutability, not sysvar identity. Data persistence checks the current owner/writability before serializing after creation; no pre-creation ownership proof is reused as if unchanged. |

The version map is source-backed: old `solana-program = 1.7.0` loaders accept
bytes without a key; v1.9.4's checked loaders call `check_id` and return
`UnsupportedSysvar` on a mismatch. This is an exact-address check, not
merely a generic program-owner check. [SDK v1.7.0][c02-sdk-old],
[SDK v1.9.4][c02-sdk-new], [Solitaire checks][c02-peel],
[account types][c02-accounts].

**Buggy: rejects buggy shape. Fixed: admits fixed shape. Pair: support for
R1.4 on this account-validation path.** The exact Instructions sysvar
address is part of the trusted instruction-history operation. Without its
address proof the buggy path cannot use that operation. The fixed calls
provide the fact, with their existing error behavior. If a trusted
constructor issued the proof from the old unchecked byte decoder, that
constructor defect would be outside the credited rejection; it is not
assumed away as an implemented Tide check.

Limits: this is a paper source map, not execution of either version or an
on-chain incident reproduction. The matched commit also changes dependency
locks, Borsh, compiler features and other programs/build files; those
outside-path hunks are not modeled as additional repairs. Parsing,
cryptography, guardian-policy correctness and global signature-set
lifecycle remain unproved. The account operations are mapped to proposed
trusted primitives, not claimed available in Tide v0. Solitaire already
supplies typed checks for other accounts. No claim that Tide invented the
fix or would enforce deployment of a published patch follows.

[c02-report]: https://www.halborn.com/blog/post/explained-the-wormhole-hack-february-2022
[c02-fix]: https://github.com/wormhole-foundation/wormhole/commit/7edbbd3677ee6ca681be8722a607bc576a3912c8
[c02-bug]: https://github.com/wormhole-foundation/wormhole/blob/79ab522f802ccc5ba34278d3c648fa62e06f4f1c/solana/bridge/program/src/api/verify_signature.rs#L22
[c02-fixed]: https://github.com/wormhole-foundation/wormhole/blob/7edbbd3677ee6ca681be8722a607bc576a3912c8/solana/bridge/program/src/api/verify_signature.rs#L22
[c02-sdk-old]: https://github.com/solana-labs/solana/blob/v1.7.0/sdk/program/src/sysvar/instructions.rs
[c02-sdk-new]: https://github.com/solana-labs/solana/blob/v1.9.4/sdk/program/src/sysvar/instructions.rs#L39
[c02-peel]: https://github.com/wormhole-foundation/wormhole/blob/7edbbd3677ee6ca681be8722a607bc576a3912c8/solana/solitaire/program/src/processors/peel.rs
[c02-accounts]: https://github.com/wormhole-foundation/wormhole/blob/7edbbd3677ee6ca681be8722a607bc576a3912c8/solana/solitaire/program/src/types/accounts.rs

#### C03. Cashio, 2022-03-23: the bank-to-mint link and a disabled fix

Assessment began 2026-11-19T00:00:05.625059Z. The registered Halborn report
describes worthless collateral used to mint real CASH. Sec3's nominated
report provides code and a correction: its initial claim that the patch
was insufficient was withdrawn after examining bankman's creation
invariant. Short symptom quote: “mint 2,000,000,000 CASH tokens for almost
free”. The old Soteria URL returned 404; the same named article's current
X-Ray URL was recovered from the saved Sec3 page's link.
[C03 reports][c03-report], [corrected Sec3 analysis][c03-sec3].

The linked patch is `7df658184c2610139fa2c0058363c66b28add4c4`, first
parent `a51c3c59d544a5763b64abb4a8d82c49b0abd6d0`. In
`programs/brrr/src/actions/mod.rs` it adds bank.crate_mint equality with
the supplied crate_mint. **It also inserts unconditional failure into
both print_cash and burn_cash.** Those failures remain in the fixed
model. No hypothetical re-enabled function substitutes for the public
patch. [C03 fix][c03-fix], [fixed entrypoints][c03-fixed-lib].
Its lockfile resolves Anchor 0.22.1 and Vipers 2.0.1; that version's
`invariant!` expands to an error return
when false, without a debug-only guard. [Lockfile][c03-lock],
[Vipers macro][c03-vipers].

| Source map | Preserved computation and proposed R1 value |
| --- | --- |
| Bug lib.rs 71–119 / fix 73–121 | Keep Anchor account containers and program identities: bank, collateral, crate, Arrow, swap, mint and token accounts are already typed. Ordinary owner/type checks alone do not reject the attacker's valid but wrong instances. |
| Bug actions/mod.rs 10–22 / fix 10–23 | Preserve all existing key relationships and Saber validation. Only fix line 13 adds `BankForOutputMint(bank, crate_mint)`; the buggy side has no such proof. |
| Both actions/print_cash.rs 74–82 | Preserve depositor signer/owner and mint relationships, plus exact issue-authority key. |
| Both actions/print_cash.rs 14–68 | Keep calculations, collateral cap, transfer CPI, issue CPI and event. Do not replace financial calculations with a capability-safety claim. |
| Bankman lib.rs 28–71, 162–171 ([source][c03-bankman]) | Preserve the authenticated Bank state: creation records crate_token and crate_mint and derives bank from crate_token. The corrected report explains the further crate-token PDA invariant; no extra bank.crate_token comparison is inserted. |
| Fix lib.rs 38–41, 49–52 | Run validation and retain the unconditional disable before either action. The source's operational mint path does not execute in this fixed candidate. |

Under R1.5 the operation policy binds collateral authorization to the
same bank/output mint that the issue operation uses. The root is
authenticated bank state and the selected output crate/mint, not a
chain of mutually caller-chosen equal values. Missing this relationship
prevents constructing the proposed issue permit on the buggy path.
The patch supplies the equality. A trusted constructor that accepted the
wrong relationship, or a recipe author who deliberately omitted it,
would remain outside the credited guarantee.

**Buggy: rejects buggy shape. Fixed: not assessable. Pair: not assessable.**
The missing bank-to-output-mint relationship supports a one-sided buggy
rejection. The fixed code adds the relationship, but assessing the compiled
action bodies requires the CPI/reload lifecycle under R1.6.

Evidence limits and disagreements: the inspected validator already
compares collateral.mint with arrow.mint, so Halborn's simplified
missing-arrow-mint account is not adopted as the code-level cause.
Sec3's corrected explanation, rather than its struck-through criticism,
is recorded. [Validator pair][c03-bug-validator], [fixed validator][c03-fixed-validator].

The mapped CPI targets are typed `Program<Token>` and
`Program<CrateToken>`; Anchor 0.22.1's program wrapper checks the program
id and executable flag, while `Account<T>` checks owner and checked
decoding. The source binds issue authority to its published constant and
uses static `print`/255 signing seeds. These remain the source's identity
choices, not a newly inferred principal. Saber validation checks the
Arrow/pool mint and swap/reserve links; conversion reads those validated
accounts and the clock. [Pinned Anchor Account][c03-anchor-account],
[Program][c03-anchor-program], [addresses][c03-addresses],
[Saber adapter][c03-saber].

The runtime disable leaves both action bodies compiled and subject to
static admission. In `print_cash`, the transfer CPI may change checked
collateral account data, and the action reads from the pre-CPI copy after
both CPIs. R1.6 requires invalidation and reload before reusing dependent
proofs. This pass did not model that lifecycle in the fixed actions.
Existing Anchor wrappers and explicit constraints can perform these same
checks. The bank curator remains trusted; selecting worthless collateral
as an authorized curator is not prevented by the mint relationship.
The model adds no protection for price conversion, economics or SDK
implementation errors. Other patch hunks change Nix/build configuration;
they are disclosed outside this mapped path. No historical build, CPI
execution, deployed-byte identification or proof of a re-enabled fixed
workflow was performed.

[c03-report]: https://www.halborn.com/blog/post/explained-the-cashio-hack-march-2022
[c03-sec3]: https://sec3.dev/blog/cashioapp-attack-whats-the-vulnerability-and-how-x-ray-detects-it
[c03-fix]: https://github.com/cashioapp/cashio/commit/7df658184c2610139fa2c0058363c66b28add4c4
[c03-bug-validator]: https://github.com/cashioapp/cashio/blob/a51c3c59d544a5763b64abb4a8d82c49b0abd6d0/programs/brrr/src/actions/mod.rs#L10
[c03-fixed-validator]: https://github.com/cashioapp/cashio/blob/7df658184c2610139fa2c0058363c66b28add4c4/programs/brrr/src/actions/mod.rs#L10
[c03-fixed-lib]: https://github.com/cashioapp/cashio/blob/7df658184c2610139fa2c0058363c66b28add4c4/programs/brrr/src/lib.rs#L38
[c03-bankman]: https://github.com/cashioapp/cashio/blob/7df658184c2610139fa2c0058363c66b28add4c4/programs/bankman/src/lib.rs#L28

[c03-lock]: https://github.com/cashioapp/cashio/blob/7df658184c2610139fa2c0058363c66b28add4c4/Cargo.lock
[c03-vipers]: https://docs.rs/crate/vipers/2.0.1/source/src/assert.rs
[c03-anchor-account]: https://github.com/coral-xyz/anchor/blob/v0.22.1/lang/src/accounts/account.rs
[c03-anchor-program]: https://github.com/coral-xyz/anchor/blob/v0.22.1/lang/src/accounts/program.rs
[c03-addresses]: https://github.com/cashioapp/cashio/blob/7df658184c2610139fa2c0058363c66b28add4c4/programs/brrr/src/addresses.rs
[c03-saber]: https://github.com/cashioapp/cashio/blob/7df658184c2610139fa2c0058363c66b28add4c4/programs/brrr/src/saber.rs

#### C04. Crema, 2022-07-02: no identified public source pair

Assessment began 2026-11-19T00:03:38.018408Z. Halborn describes forged tick
data used to obtain fees; short symptom quote: “draining nearly $8
million”. Its wording says an owner check was circumvented, whereas the
selection catalog describes a missing owner check. Neither supplies the
actual Claim validation code. [C04 registered report][c04-report].

The other registered report, Rekt, distinguishes an earlier audited/fixed
swap-method issue from the claim-method issue exploited here. That earlier
audit repair cannot substitute for this incident's fix. CertiK's July 3
analysis states “the exploited project's source code is private” and
does not establish the exact defective function with code.
[Rekt][c04-rekt], [CertiK][c04-certik].

Resolution attempts: read both registered report narratives and links,
the contemporary CertiK analysis, a public same-incident code search, and
the connected GitHub repository search scoped to CremaFinance. The latter
returned six public repository entries; none identified a historical
buggy/fixed Claim implementation. This bounded search does not prove that
no public copy exists. No live transaction, bytecode download, later SDK
reconstruction or unrelated audit patch was substituted.

| Missing source-map item | Consequence |
| --- | --- |
| Historical Rust handler, exact input checks and matched repair | Cannot determine whether the absent fact was runtime ownership, exact instance/PDA binding, decoding, or another predicate; cannot map either side under R1.3–R1.6. |

**Buggy: not assessable. Fixed: not assessable. Pair: not assessable.**
Missing code is not a model rejection, an admitted bug, or evidence for a
working fix. A hypothetical `Owned<Tick>` example would illustrate R1
only; it would not faithfully assess Crema. The Rust implementation's
historical identity and repair also remain unverified. The frozen case
stays in the denominator, with no replacement.

[c04-report]: https://www.halborn.com/blog/post/explained-the-crema-finance-hack-july-2022
[c04-rekt]: https://rekt.news/crema-finance-rekt
[c04-certik]: https://www.certik.com/blog/crema-finance-exploit

#### C05. SolSea: signer report without the SolSea source pair

Assessment began 2026-11-19T00:05:44.140761Z. The registered SolShield/
OShield disclosure, published 2022-03-17, describes unauthorized delisting
combined with a separate SPL Token lifecycle defect. It says the SolSea
“program source code was not available to us”. Its transaction sketches
and SPL Token v3.2/v3.3 discussion are not SolSea's Rust handler or repair.
The incident date remains unknown. [C05 disclosure][c05-report].

The source says a fee-payer signature was required, but the intended
listing owner's signature was not. That distinction matters: possession
of *some* `Signer` cannot establish authorization. An R1.2 role-bound
value would need an authenticated listing-owner relationship, not just
the existing fee-payer fact. This is a requirement suggested by the
narrative, not a source-faithful admission verdict.

A same-incident public code/fix search resolved back to this disclosure
and current user documentation, not a matched historical source pair.
The statement that the SolSea issues were fixed supplies no diff or
code-level fixed checks. We do not import the public SPL Token repair
as SolSea's fix, replace the old program with a newer marketplace, or
invent a missing signer comparison to construct a positive model.

| Missing source-map item | Consequence |
| --- | --- |
| Historical Delist/Init Rust implementation and matched SolSea repair | Cannot preserve the handler's checks, signer roles, accounts, CPI/lifecycle and actual fixed branch under R1.2/R1.5/R1.6. |

**Buggy: not assessable. Fixed: not assessable. Pair: not assessable.**
The account-validation narrative is not enough to establish either side;
the historical Rust identity remains unverified. This is a coordinated
disclosure, not a confirmed theft count. The source snapshot is the
registered web-tool extraction, with its disclosed original-HTML
limitation; no stronger provenance is claimed after go.

[c05-report]: https://medium.com/@oshield/schrodingers-nft-an-incinerator-spl-token-program-and-the-royal-flush-attack-58e4ce4e63dc

### 4.4 What the comparison supports

The paper exercise supports one narrow design observation: independent
signer, owner and PDA values are insufficient without the correct
instance relationships and exact identity obligations at the operations
that use them. It does not establish an automatic way to choose those
obligations. Solend and Cashio already have important valid wrappers/checks;
their omitted relationships remain policy input. Wormhole's checked SDK
already provides the decisive address validation. The hypothesized gain
is compulsory use and confinement throughout admitted application code,
plus a reviewable upgrade recipe, not invention of these checks.

The disposition in §2.4 leaves compiler compatibility, lifecycle and alias
confinement, migration cost, recipe quality and upgrade-review benefit as
open questions. A bounded SBF/adapter experiment would require separate
authorization and a case for its cost against Anchor plus existing analysis.

R1 cannot establish business correctness, arithmetic safety, oracle truth,
economic solvency, correct collateral selection by a trusted curator,
safe use of stolen keys or benevolent upgrades. Its own trusted library,
runtime and policy remain review obligations. No case result changes
those limits or demonstrates a Tide v0 catch.

### 4.5 Deviations, access limits and verification surface

No post-go change to the rule, case list or selection was made; A1 is the
only amendment and predates stamping/go. The initial registration's
multi-query search pacing limitation and Medium extraction provenance
remain disclosed in the protocol/registered manifest. Controlled fetches
used serial pacing and five-minute backoff after 403/429 during registration.
Post-go requests used serial page/tool calls; no new 403/429 or provider
safety refusal occurred. Connector-internal HTTP pacing is unobservable;
no claim is made about internal requests beyond the controlled calls.

The assessment manifest retains the failed Wormhole SHA lookup (422),
the obsolete Cashio article URL (404), and an Anchor connector error
followed by a successful exact-URL HTTP fetch. None is treated as code
evidence. The initial post-go C01 discovery search output was not saved.
The matched primary report, code and commit-list request were saved and
support the comparison. The corrected Cashio article is an additional
post-go snapshot, not a replacement of a registered body. The record retains
the article's published correction and Cashio's disabled patch. Consolidation
corrected C01's PDA/write line locators by one line; its earlier scratch
note is retained, and no predicate, verdict or conclusion changed.

The 2026-11-19 Opus review found that the initial Cashio assessment had
treated the runtime disable as sufficient for static admission. The review
disposition applies the registered R1.6 to the compiled action bodies and
corrects the fixed verdict to `not assessable`. The buggy rejection stands
as a one-sided result. R1 and the registered case list are unchanged.

Verification checked the frozen subjects/proofs against Git, the five
ordered start/result records after go, manifest artifact hashes/sizes,
commit/first-parent identities, the Solend four-line patch, both Cashio
disables, local references and whitespace. These checks establish record
consistency, not semantic enforcement, cryptographic correctness, deployed
byte identity or independent reviewer agreement. No compilation, spike,
live-chain interaction or upstream code execution occurred. The coordinator
owns the review re-check and landing.

Saved pages/code remain gitignored under `target/work/sources/`; only
manifests and original analysis are committed. Preserve those snapshots
through review and archive them with the evidence bundle: a manifest
without its bodies cannot guarantee future retrieval of the same bytes.

## Source versions and read date

The §1–§3 entries below were read on 2026-11-18 UTC. Case sources in §4
were read on 2026-11-18/19 UTC in the logged order; their retrieval times,
exact revisions and hashes are in the separate assessment manifest.
An unversioned page has no inferred release: its exact body hash in the
manifest is the snapshot identity. Source tags and commits identify the
inspected code versions.

| Reference | Version or commit |
| --- | --- |
| [ackee-analyzer][ackee-analyzer] | commit `87bc85cfb320308db5e3e8227ca8fa87194bc983` |
| [agoric-ocap-redirect][agoric-ocap-redirect] | unversioned documentation/source snapshot (identity is SHA-256); SHA-256 `045e02d55b97…` |
| [agoric-zoe][agoric-zoe] | unversioned documentation/source snapshot (identity is SHA-256); SHA-256 `96e0195f186e…` |
| [anchor-account-source][anchor-account-source] | v0.31.1 |
| [anchor-accounts-derive][anchor-accounts-derive] | v0.31.1 |
| [anchor-constraints][anchor-constraints] | unversioned documentation/source snapshot (identity is SHA-256); SHA-256 `6dfbe5990e31…` |
| [anchor-constraints-source][anchor-constraints-source] | v0.31.1 |
| [anchor-discriminator][anchor-discriminator] | v0.31.1 |
| [anchor-macro][anchor-macro] | v0.31.1 |
| [anchor-tryaccounts][anchor-tryaccounts] | v0.31.1 |
| [anchor-types][anchor-types] | unversioned documentation/source snapshot (identity is SHA-256); SHA-256 `1dfc328bb77d…` |
| [aptos-capability-corrected][aptos-capability-corrected] | commit `4cddc9f17e6a37e055edcd05f6873d1137dd7cfe` |
| [aptos-resources-redirect][aptos-resources-redirect] | unversioned documentation/source snapshot (identity is SHA-256); SHA-256 `99623950c69e…` |
| [aptos-signer-redirect][aptos-signer-redirect] | unversioned documentation/source snapshot (identity is SHA-256); SHA-256 `3cbbbda28ab7…` |
| [cosmwasm-info][cosmwasm-info] | v2.2.2 |
| [ink-access][ink-access] | documentation v5 |
| [ink-env-corrected][ink-env-corrected] | documentation v5 |
| [neodyme-pitfalls-redirect][neodyme-pitfalls-redirect] | article published 2021-08-20; unversioned page snapshot; SHA-256 `8567d04caea6…` |
| [platform-build][platform-build] | v1.58 |
| [platform-tools][platform-tools] | unversioned documentation/source snapshot (identity is SHA-256); SHA-256 `8f811bc8f46e…` |
| [sbf-builder-readme][sbf-builder-readme] | unversioned documentation/source snapshot (identity is SHA-256); SHA-256 `50c33c429b6f…` |
| [sec3-scanner][sec3-scanner] | article published 2022-05-24; unversioned page snapshot; SHA-256 `25938a93adf3…` |
| [solana-program][solana-program] | solana-program 2.3.0 |
| [soroban-address][soroban-address] | v22.0.0 |
| [soroban-auth][soroban-auth] | unversioned documentation/source snapshot (identity is SHA-256); SHA-256 `450733b468b7…` |
| [stylus-host][stylus-host] | v0.9.0 |
| [stylus-msg][stylus-msg] | unversioned documentation/source snapshot (identity is SHA-256); SHA-256 `b96cc8ec6cdc…` |
| [sui-capabilities][sui-capabilities] | unversioned documentation/source snapshot (identity is SHA-256); SHA-256 `dfcd1bff3b54…` |
| [sui-objects-redirect][sui-objects-redirect] | unversioned documentation/source snapshot (identity is SHA-256); SHA-256 `db7d3528d6c8…` |
| [vrust][vrust] | commit `aa461e5bdaa47292c0cdb4380d7c2ed59b8d2d20` |

[ackee-analyzer]: https://raw.githubusercontent.com/Ackee-Blockchain/solana-vscode/87bc85cfb320308db5e3e8227ca8fa87194bc983/README.md "Read 2026-11-18 UTC; version and full SHA-256 in sources.json"
[agoric-ocap-redirect]: https://docs.agoric.com/guides/js-programming/hardened-js "Read 2026-11-18 UTC; version and full SHA-256 in sources.json"
[agoric-zoe]: https://docs.agoric.com/guides/zoe/ "Read 2026-11-18 UTC; version and full SHA-256 in sources.json"
[anchor-account-source]: https://raw.githubusercontent.com/coral-xyz/anchor/v0.31.1/lang/src/accounts/account.rs "Read 2026-11-18 UTC; version and full SHA-256 in sources.json"
[anchor-accounts-derive]: https://raw.githubusercontent.com/coral-xyz/anchor/v0.31.1/lang/derive/accounts/src/lib.rs "Read 2026-11-18 UTC; version and full SHA-256 in sources.json"
[anchor-constraints]: https://www.anchor-lang.com/docs/references/account-constraints "Read 2026-11-18 UTC; version and full SHA-256 in sources.json"
[anchor-constraints-source]: https://raw.githubusercontent.com/coral-xyz/anchor/v0.31.1/lang/syn/src/codegen/accounts/constraints.rs "Read 2026-11-18 UTC; version and full SHA-256 in sources.json"
[anchor-discriminator]: https://raw.githubusercontent.com/coral-xyz/anchor/v0.31.1/lang/attribute/account/src/lib.rs "Read 2026-11-18 UTC; version and full SHA-256 in sources.json"
[anchor-macro]: https://raw.githubusercontent.com/coral-xyz/anchor/v0.31.1/lang/attribute/program/src/lib.rs "Read 2026-11-18 UTC; version and full SHA-256 in sources.json"
[anchor-tryaccounts]: https://raw.githubusercontent.com/coral-xyz/anchor/v0.31.1/lang/syn/src/codegen/accounts/try_accounts.rs "Read 2026-11-18 UTC; version and full SHA-256 in sources.json"
[anchor-types]: https://www.anchor-lang.com/docs/references/account-types "Read 2026-11-18 UTC; version and full SHA-256 in sources.json"
[aptos-capability-corrected]: https://raw.githubusercontent.com/aptos-labs/aptos-core/4cddc9f17e6a37e055edcd05f6873d1137dd7cfe/aptos-move/framework/aptos-framework/sources/account/account.move "Read 2026-11-18 UTC; version and full SHA-256 in sources.json"
[aptos-resources-redirect]: https://aptos-labs.github.io/move-book/generics-and-abilities.html "Read 2026-11-18 UTC; version and full SHA-256 in sources.json"
[aptos-signer-redirect]: https://aptos-labs.github.io/move-book/primitive-types.html "Read 2026-11-18 UTC; version and full SHA-256 in sources.json"
[cosmwasm-info]: https://raw.githubusercontent.com/CosmWasm/cosmwasm/v2.2.2/packages/std/src/types.rs "Read 2026-11-18 UTC; version and full SHA-256 in sources.json"
[ink-access]: https://use.ink/docs/v5/basics/cross-contract-calling/ "Read 2026-11-18 UTC; version and full SHA-256 in sources.json"
[ink-env-corrected]: https://use.ink/docs/v5/basics/environment-functions/ "Read 2026-11-18 UTC; version and full SHA-256 in sources.json"
[neodyme-pitfalls-redirect]: https://neodyme.io/en/blog/solana_common_pitfalls/ "Read 2026-11-18 UTC; version and full SHA-256 in sources.json"
[platform-build]: https://raw.githubusercontent.com/anza-xyz/platform-tools/v1.58/build.sh "Read 2026-11-18 UTC; version and full SHA-256 in sources.json"
[platform-tools]: https://raw.githubusercontent.com/anza-xyz/platform-tools/master/README.md "Read 2026-11-18 UTC; version and full SHA-256 in sources.json"
[sbf-builder-readme]: https://raw.githubusercontent.com/anza-xyz/cargo-build-sbf/master/README.md "Read 2026-11-18 UTC; version and full SHA-256 in sources.json"
[sec3-scanner]: https://sec3.dev/blog/sec3-pro-auto-auditor "Read 2026-11-18 UTC; version and full SHA-256 in sources.json"
[solana-program]: https://docs.rs/solana-program/2.3.0/solana_program/ "Read 2026-11-18 UTC; version and full SHA-256 in sources.json"
[soroban-address]: https://raw.githubusercontent.com/stellar/rs-soroban-sdk/v22.0.0/soroban-sdk/src/address.rs "Read 2026-11-18 UTC; version and full SHA-256 in sources.json"
[soroban-auth]: https://developers.stellar.org/docs/learn/fundamentals/contract-development/authorization "Read 2026-11-18 UTC; version and full SHA-256 in sources.json"
[stylus-host]: https://raw.githubusercontent.com/OffchainLabs/stylus-sdk-rs/v0.9.0/stylus-sdk/src/msg.rs "Read 2026-11-18 UTC; version and full SHA-256 in sources.json"
[stylus-msg]: https://docs.arbitrum.io/stylus/reference/rust-sdk-guide "Read 2026-11-18 UTC; version and full SHA-256 in sources.json"
[sui-capabilities]: https://move-book.com/programmability/capability/ "Read 2026-11-18 UTC; version and full SHA-256 in sources.json"
[sui-objects-redirect]: https://docs.sui.io/develop/objects/object-ownership "Read 2026-11-18 UTC; version and full SHA-256 in sources.json"
[vrust]: https://raw.githubusercontent.com/parasol-aser/vrust-open-source/aa461e5bdaa47292c0cdb4380d7c2ed59b8d2d20/README.md "Read 2026-11-18 UTC; version and full SHA-256 in sources.json"
