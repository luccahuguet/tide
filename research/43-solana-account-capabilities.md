# Account authority as capabilities for Rust smart contracts

fx-qlek.17 · Step-1 research draft · 2026-11-18 · Author: Codex

**Status: awaiting coordinator timestamp and go. No case has a
verdict.** This record proposes a rule for a possible Solana research
profile. Tide v0 implements none of it. The task authorizes research,
not a language decision or an alpha feature. The coordinator will arrange
Codex and Opus review before landing; Opus decides the sketch's choices.

## 1. Prior art

The sources below support the stated mechanisms, not a claim that a
framework or analyzer prevents every exploit. Read date for all external
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
library owns exclusive writes, checked CPI targets and invalidation of
proofs after mutations/CPI. The policy recipe owns which principal and
relationships the operation requires. Admission owns confinement:
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

### 2.4 Choices for Opus

These are options and the author's recommendations, not owner-approved
language decisions. R1 freezes one candidate for evaluation; adoption
can still be rejected after the research.

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
   further; decline after this pass. Recommend completing the registered
   paper comparison first and leaving the alpha unchanged. Prior art is
   substantial and no compatibility or coverage result exists yet.

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
entry and `lib|bin`, rejects raw rustc flags and user proc macros, and
supports linked outputs on x86_64 Linux. It also rejects `#![no_std]`.
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
handler and one generated validator with the exact SBF compiler. Step 1
adds no spike or dependency. The registered case comparison is a paper
model and can finish without proving compiler integration.

## 4. Registered evidence: pending go

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

No case code/fix comparison, verdict, case model or catch count exists yet.
Case verdicts and the full evidence table belong to the post-go
continuation; this is not the completed bead.

## Source versions and read date

All entries below were read on 2026-11-18 UTC. An unversioned page has
no inferred release: its exact body hash in the manifest is the snapshot
identity. Source tags and commits identify the inspected code versions.

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
