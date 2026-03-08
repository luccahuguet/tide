# Orbis Domain Strategy Review

Historical note:

* this document records the domain strategy review for the earlier `Orbis` candidate
* `Ferox` is now the selected public name, so this document is retained only as naming-history context

This document records the first-pass domain strategy review for the public name **Orbis** and the CLI name `orb`.

This is **not legal clearance** and **not a registrar availability confirmation**.

Review date:

* March 8, 2026

## Summary

Current conclusion:

* do **not** use `orb.*` as the primary public website domain
* do **not** plan around exact-root `orbis.com` or `orbis.org`
* use a **compound `orbis` domain** as the primary public web identity
* keep `orb` as the CLI binary, package/tooling namespace, and terminal brand rather than the main web brand

Current recommendation:

* preferred public domain pattern: `orbis` + a clear qualifier such as `lang`, `language`, or `get`
* primary planned public domain: `orbislang.com`
* secondary fallback or defensive candidates:
  * `orbislang.org`
  * `orbis-lang.org`
  * `orbislang.dev`
  * `getorbis.dev`

These domains are planning targets, not confirmed registrations or confirmed availability.

## Exact-Root Findings

### `orbis.com`

Why it is not viable as the primary plan:

* the domain is already registered
* public WHOIS shows it is held by `Orbis Holdings Limited`

Source:

* https://www.whois.com/whois/orbis.com

### `orbis.org`

Why it is not viable as the primary plan:

* it is already the live public site of Orbis International
* this is a long-established nonprofit with active global operations

Sources:

* https://www.orbis.org/en
* https://www.whois.com/whois/orbis.org

### Exact `orbis.*` space is broadly occupied

Why it matters:

* the exact word root already appears across many TLDs
* this means the project should not assume it can build a clean web identity around plain `orbis` plus any desirable extension

Inference:

* this is an inference from DNS listing sources and exact-root usage already confirmed above, not a complete registrar census

Sources:

* https://robtex.com/en/dns-lookup/es/orbis
* https://robtex.com/en/dns-lookup/za/org/orbis

### `orb.com`

Why it is not viable as the primary plan:

* the domain is already registered
* public WHOIS shows a long-standing registration dating to 1992

Source:

* https://www.whois.com/whois/orb.com

### `orb.net`

Why it is not viable as the primary plan:

* `orb.net` is an active software product site
* the product already uses `Orb` as the public software name
* the product also has a live CLI and install flow, which makes `orb` especially crowded in developer and terminal contexts

Sources:

* https://orb.net/
* https://orb.net/get-orb/linux
* https://orb.net/the-forge/coming-soon

## Brazil Considerations

Brazil does **not** rescue the exact-root domain strategy.

### `orb.com.br`

Why it matters:

* `orb.com.br` is already in use by a Brazilian industrial company

Sources:

* https://www.cbca-acobrasil.org.br/guia-brasil-da-construcao-em-aco/busca-detalhes.php?bsc=&cod=40432
* https://www.cbca-acobrasil.org.br/guia-brasil-da-construcao-em-aco/busca.php?ancora=40214&bsc=ativar&e=buscar&slcBscCategoria=&txtBscCidade=&txtBscKeyword=

### `orbis.com.br`

Why it matters:

* `orbis.com.br` is already associated with existing Brazil-facing usage
* even if one visible use is inactive or weak, it is enough to make the exact local root unattractive as a clean default

Sources:

* https://www.lojaorbis.com.br/
* https://www.lojaorbis.com.br/orbis/about
* https://br.todosnegocios.com/pt/aussie-education-ltda-me

## Recommended Strategy

### 1. Keep `orb` out of the primary website domain

Reason:

* `orb` is too short and too occupied
* it already has strong software and CLI usage outside this project
* it is better suited to the local command name than to the global web identity

Recommendation:

* use `orb` only for:
  * CLI commands
  * package/tool namespaces
  * local filenames and command examples

### 2. Use a compound `orbis` domain as the canonical website

Reason:

* `Orbis` is the public language name
* a compound domain is more ownable than exact-root `orbis.*`
* it avoids tying the whole public web brand to the already-crowded `orb` root

Recommended patterns:

* `orbislang.com` as the primary target
* `orbislang.*`
* `orbis-lang.*`
* `getorbis.*`

### 3. Prefer one canonical web root and keep the rest optional

Reason:

* early-stage projects do not need a large domain portfolio
* one strong canonical domain is better than several weak or inconsistent ones

Recommended structure:

* primary site: `orbislang.com`
* docs: `docs.<primary-domain>` or `<primary-domain>/docs`
* optional redirects only if easy to obtain later

### 4. Treat exact `orbis.*` or `orb.*` domains as bonuses, not dependencies

Reason:

* the exact roots are already too occupied to anchor launch planning
* if a future acquisition ever becomes possible, it should be treated as a redirect or brand-defense upgrade rather than the foundation of the project

## Practical Recommendation

For launch planning:

* use `orbislang.com` as the canonical public-site plan
* keep `orb` as the CLI only
* do not block the project on obtaining exact `orbis.*` or `orb.*` domains
* if `orbislang.com` is not obtainable on acceptable terms, fall back to the next compound `orbis` candidate rather than shifting the project back to `orb.*`

## Scope Limits

This review does **not** include:

* registrar checkout or purchase attempts
* exhaustive global domain availability checks
* legal opinion on domain-related trademark risk
* social handle review
