# Ferox Domain Strategy Review

This document records the first-pass domain strategy review for the public name **Ferox**.

This is **not legal clearance** and **not a registrar checkout or purchase confirmation**.

Review date:

* March 8, 2026

## Summary

Current conclusion:

* do **not** make the project depend on exact-root `ferox.*` domains
* exact `ferox` roots are already occupied or noisy enough that they should be treated as optional upgrades, not launch prerequisites
* use a **compound `ferox` domain** as the canonical public web identity
* keep `ferox` as both the language name and the CLI binary regardless of the public-site domain choice

Current recommendation:

* preferred public domain pattern: `ferox` + a clarifying qualifier such as `lang`, `language`, or `get`
* primary planned public domain: `feroxlang.com`
* secondary fallback or defensive candidates:
  * `feroxlang.org`
  * `ferox-lang.org`
  * `getferox.dev`
  * `feroxlang.dev`

These are planning targets, not confirmed registrations or confirmed availability.

## Exact-Root Findings

### `ferox.com`

Why it is not viable as the primary plan:

* the domain is already registered
* it is tied to the public history site for `Ferox Microsystems`

Sources:

* https://www.ferox.com/
* https://www.ferox.com/about.html
* https://whois.is/whois/ferox.com

### `ferox.com.br`

Why it is not viable as the Brazil-local root:

* the domain is already registered and active
* it is associated with an existing Brazil-facing software and diagnostics business

Sources:

* https://www.who.is/rdap/ferox.com.br
* https://store.ferox.com.br/fale-conosco
* https://www.reclameaqui.com.br/empresa/ferox/sobre/

### Exact `ferox` roots already have technical and commercial usage

Why it matters:

* `Ferox` already appears on live technical and software-facing domains beyond `ferox.com`
* this weakens the case for anchoring the public identity to the hope of acquiring plain `ferox.*`

Examples:

* `ferox.net.au` is an active software and IT consulting company
* `ferox.systems` is an active technical product/company site

Sources:

* https://www.ferox.net.au/
* https://www.crunchbase.com/organization/ferox-consulting
* https://www.ferox.systems/

## Brazil Considerations

Brazil does **not** rescue the exact-root strategy.

Why:

* `ferox.com.br` is already registered and active
* the existing root is not just parked; it is tied to a live Brazil-facing business

Practical implication:

* even if the project remains Brazil-led, it should still plan around a compound public domain rather than assume a local exact root is obtainable

Sources:

* https://www.who.is/rdap/ferox.com.br
* https://store.ferox.com.br/fale-conosco

## Recommended Strategy

### 1. Use a compound `ferox` domain as the canonical website

Reason:

* the project name is `Ferox`
* exact-root `ferox.*` is already occupied or noisy
* a compound domain is more realistic and easier to plan around

Recommended patterns:

* `feroxlang.com`
* `feroxlang.*`
* `ferox-lang.*`
* `getferox.*`

### 2. Keep one canonical public web root

Reason:

* early-stage projects do not need a large domain portfolio
* one stable canonical site is better than several partially used roots

Recommended structure:

* primary site: `feroxlang.com`
* docs: `docs.<primary-domain>` or `<primary-domain>/docs`
* redirects: optional later, not required now

### 3. Treat exact-root `ferox.*` acquisitions as bonuses, not dependencies

Reason:

* the project does not need to own `ferox.com` or `ferox.com.br` to launch
* waiting on exact-root acquisition would stall the project for little gain

Practical interpretation:

* if an exact-root `ferox.*` domain ever becomes obtainable on acceptable terms, use it as a redirect or later brand upgrade
* do not shape the current public identity around that possibility

### 4. Keep the web brand and CLI brand aligned

Reason:

* unlike the rejected `Orbis` plus `orb` split, `Ferox` already has a coherent language and CLI identity
* there is no need for a separate short command brand or separate public web root concept

Recommendation:

* language name: `Ferox`
* CLI binary: `ferox`
* public site: compound `ferox` domain, preferably `feroxlang.com`

## Practical Recommendation

For launch planning:

* use `feroxlang.com` as the canonical public-site plan
* do not block the project on exact-root `ferox.*`
* if `feroxlang.com` is not obtainable on acceptable terms, fall back to the next compound `ferox` candidate rather than reopen naming on domain grounds alone

## Scope Limits

This review does **not** include:

* registrar checkout or purchase attempts
* exhaustive global domain availability checks
* social-handle review
* legal opinion on domain-related trademark risk
