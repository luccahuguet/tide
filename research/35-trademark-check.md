# 35 — Trademark check of the Tide name and tagline, and the tidelang Nix names (fx-gbq4)

Research for fx-gbq4, 2026-11-17, by a Claude Opus 5.5 research builder.
Every web source was read on 2026-11-17. The note is docs-only and changes
no code, README.md or bead.

**Not legal advice.** This is an engineer's desk search of public sources.
It is not a trademark clearance opinion. A professional clearance search,
before 1.0, sponsorship or any paid offering, remains the owner's option
(`tide-naming/decision.md`, step 5).

Evidence labels apply to each following paragraph, list or table:

- **[web: source]**: read from that source on 2026-11-17.
- **[repo evidence]**: Tide's tree at branch `fx-gbq4` (base `13519b4`).
- **[inference]**: this note's reasoning from the evidence.

## Summary

| Mark | Use | Verdict |
| --- | --- | --- |
| "A capability-safe, replayable subset of Rust." | README headline | **Low (preliminary).** An accurate compatibility statement of the kind the Rust policy allows; the policy's endorsement rule still applies. No change needed. |
| "Tide" | Name of a free programming language and its tools (Nice 9, 42) | **Low, provisional.** In the 100 USPTO records read (of 1,265), no live mark covers programming tools; the one live software filing is limited to banking and finance. Not assessed: the other 1,165 records, EU, UK and WIPO registers, unregistered (common-law and state) rights, including those of the same-field projects http-rs/tide (the closest), IlanCosman/tide, ananthakumaran/tide and Stanislas7/tide, and of the other TIDE developer tools. research/28 §4's disambiguation (`tidelang`, "the Tide language") answers those uses. The dilution risk from P&G's TIDE is unresolved and provisionally low. |
| `tidelang` | Handle, future CLI and package prefix | **Low, provisional.** No crate or Nix attribute. No trademark register was searched for it. LazyAshleyz/TideLang, a programming language pushed 2026-11-06, is a concrete name collision whose rights were not assessed. |
| Nix names | fx-1n5n's flake | `tidelang-admit` (also the default package and app), `tidelang-std`. `tidelang-macros` is not exposed. |

"Low" here means that no evidence was found of a conflict likely to force a
rename; it is not a clearance. The name verdicts rest on a partial
registration search. A professional clearance search, which also covers
common-law use, is what would make them final.

## 1. The tagline against the Rust trademark policy

**What README says** [repo evidence: `README.md` lines 3–5, commit
`a02a6e0`]:

- The headline: "**A capability-safe, replayable subset of Rust.**"
- The next sentence: "Tide is a checked subset of valid Rust 2024".

design/01-design.md D18: "Every Tide program is a valid Rust 2024 crate
that a pinned, unmodified nightly rustc compiles, with Tide's meaning,
against Tide's std. The driver passes that std as `std` (`--extern std=`),
so rustc's std injection loads it and the real std's paths cannot be
named." The conformance runner checks each corpus case through stable
rustc and the admission driver (`README.md`, Verification;
`crates/tide-conformance/Cargo.toml`).

**The policy's words** [web: https://rustfoundation.org/policy/rust-trademark-policy/]:

- What is barred. The Rust word marks "may not be used:
  - To refer to any other programming language;
  - To refer to a modified version of the Rust programming language, except as permitted below for minor changes;
  - In a way that is misleading or may imply association of unrelated modules, tools, documentation, or other resources with the Rust programming language;
  - In ways that confuses the community as to whether the Rust programming language is open source and free to use."
- What is allowed without approval. "Stating accurately that software is written in the Rust programming language, that it is compatible with the Rust programming language, or that it contains code written in the Rust programming language, is allowed. In those cases, you may use the word "Rust" to indicate this, without prior approval. This is true both for non-commercial and commercial uses."
- Further allowed uses. "Using the word "Rust" on websites, brochures, documentation, academic papers, books, and product packaging to refer to the Rust programming language or the Rust Project is allowed."
- The rule for every use. "the Rust trademarks cannot be used in ways that appear (to a casual observer) official, affiliated, or endorsed by the Rust Project or Rust Foundation, unless you have written permission from the Rust Foundation. [...] This is a requirement for all uses, even those that are listed as not requiring explicit approval." It names "'official', 'original' or 'only'" as words that are likely to give that appearance.
- Trademarks. "Using the Rust trademarks within another trademark requires written permission from the Rust Foundation and may include additional conditions of use and a fee for use."

**Analysis** [inference]:

1. In "a [...] subset of Rust", the word "Rust" names the Rust language itself. Tide is the subset, and Rust is the language it is a subset of. The phrase does not call Tide "Rust", so the bar on referring "to any other programming language" does not apply.
2. The claim is accurate. Every Tide program is a valid Rust 2024 crate that a pinned, unmodified nightly rustc compiles against Tide's std (D18). That makes it a compatibility statement: Tide code "is written in the Rust programming language". The policy allows such statements without approval, commercial or not.
3. "Modified version". tide-admit does two things. It rejects programs outside the subset, and it passes Tide's std to rustc as `std` through `--extern std=`, so `std::` paths in a Tide program name Tide's std, not Rust's. That substitutes a library through a standard rustc option; it changes neither the Rust language nor the compiler, and Tide distributes no modified rustc. This is the strongest reading against the tagline: someone could call a Rust with a different std "a modified version of the Rust programming language". The policy's modified-version rules, though, restrict calling a modified Rust "Rust". Tide calls nothing "Rust", and does not call its std "Rust's std".
4. Not a trademark. "Tide" contains no "Rust", so the within-another-trademark rule does not apply (research/28 §4 already rejected "rerust" on this rule).
5. The residual risk is the casual-observer rule. Nothing in README uses "official", "original" or "only", and Tide uses no Rust logo.
6. The earlier headline, "Capability-safe, replayable Rust" (fx-gbq4's Why), was the riskier form: it used "Rust" as the noun for Tide itself. The fx-vp4u rewording to "subset of Rust" removed that.

**Practice elsewhere** [web: https://github.com/cryspen/hax;
https://github.com/verus-lang/verus]:

- hax describes its input as "a large subset of Rust".
- Verus calls itself "Verified Rust for low-level systems code".

These show common usage, not the Foundation's approval.

**Verdict: low (preliminary)** [inference]. No change to the headline is
needed. Three things keep it low. Each is an example, not a safe harbour:
the policy's appearance rule is a judgement about how a casual observer
reads every use.

- Keeping the claim accurate. If Tide ever admits a program that is not
  valid Rust 2024 for an unmodified rustc, "subset of Rust" stops being a
  compatibility statement.
- Never describing Tide's std as Rust's std. Tide's std is Tide's library,
  which the driver passes as `std` (D18).
- Avoiding any implied endorsement. For example, never pair "Rust" with
  "official", "original" or "only", or with the Rust logo, in Tide's
  materials.

Given this preliminary low-risk assessment, this note recommends no
replacement wording. If the owner wants the most literal form, the README's own second
sentence, "a checked subset of valid Rust 2024", rests on the same allowed
use.

## 2. "Tide" for software (Nice classes 9 and 42)

### Sources searched

- **USPTO Trademark Search** (tmsearch.uspto.gov). One word-mark query for
  "tide" through the JSON backend the search page uses returned 1,265
  records, live and dead. The note reads the top 100 by relevance, which
  rank the bare word mark TIDE first [web: tmsearch.uspto.gov, 2026-11-17].
  - Gap: further calls to that backend were declined by this session's
    permission check, so the class-filtered query was not run.
  - The query to run by hand, in the search page's field-tag syntax
    (check the tags against its help page):
    `CM:tide AND (IC:009 OR IC:042) AND LD:true`, then the same for
    `tidelang`.
- **EUIPO eSearch** (euipo.europa.eu/eSearch). The page is a
  JavaScript-only application.
  - Gap: not searched.
  - The queries to run: first, owner "Tide Platform" (Ltd, S.A., SAS),
    to see whether its class 9 and 42 specifications carry the US filing's
    "banking, finance, accounting, and tax" limit; then basic search
    "tide", then "tidelang", Nice classes 9 and 42, status filed or
    registered.
- **TMview** (tmdn.org/tmview). Its API answered with a bot challenge.
  - Gap: not searched.
  - The queries to run: "tide", then "tidelang", Nice classes 9 and 42, all
    offices, status Filed or Registered. For the UK, search the UK IPO
    register (https://trademarks.ipo.gov.uk/ipo-tmtext), or TMview with
    the office filter set to GB: first owner "Tide Platform Ltd" (UK
    company 09595646) and its class 9 and 42 specifications, as for EUIPO;
    then word "tide", then "tidelang", classes 9 and 42. Tide Platform is a
    UK fintech whose Madrid designation in the US, 79193510 (class 36), is
    cancelled, so its home and EU registrations are the likely ones.
- **WIPO Global Brand Database** (branddb.wipo.int). A JavaScript-only
  page.
  - Gap: not searched.
  - The queries to run: brand name "tide", then "tidelang", Nice class 9
    or 42, status active.
- **Unregistered rights**: not assessed. Rights from use (US common law,
  US state registers, UK passing off) need no registration and survive a
  dead federal registration [inference]. Neither this note nor the
  register queries above cover them.
  - The searches to run: web, app-store and package-registry searches for
    "Tide" used as the name of a programming language, compiler, IDE or
    developer tool; the same-field users below, http-rs/tide first, then
    IlanCosman/tide, ananthakumaran/tide and Stanislas7/tide; the other
    named users (the Tcl IDE, Tibbo's IDE, and "Tide Software Inc."); the
    dead class 9 and 42 owners in the table, to
    see whether they still use the name; and US state registers.
- **Secondary sources**: web searches for TIDE marks held by Tide Platform
  and Tide Software, and for P&G dilution cases (Justia, trademarkdir,
  markinton, UK IPO journal, legal blogs).

### Hits in classes 9 and 42

From the USPTO sample [web: tmsearch.uspto.gov]:

| Mark | Serial / reg. | Owner | Classes | Status | Goods in 9 / 42 |
| --- | --- | --- | --- | --- | --- |
| TIDE | 98483640 / — | Tide Platform Ltd (UK) | 9, 35, 36, 42 | Live application, "report completed suspension check – case still suspended" | Software, APIs, SaaS and PaaS "in the fields of banking, finance, accounting, and tax"; payment terminals and cards |
| TIDE OPTICAL | 90452380 / 6587349 | Hongliang Yuan (CN); Chongqing Legend Industrial Co., Ltd. (CN). Both names are returned; their ownership chronology is unresolved | 9 | Live, registered | Eyeglasses and sunglasses |
| TIDE | 74321607 | Telxon Corporation | 9 | Dead | — |
| TIDE | 87638260 / 5650653 | Thorlabs, Inc. | 9, 10 | Dead | — |
| TIDE | 76610450 | Battelle Memorial Institute | 42 | Dead | — |
| TIDE | 87455511 / 5473328 | Agile Pursuits, Inc. | 9, 37, 39 | Dead | — |
| TIDE | 76294289, 76297871 | Note Printing Australia | 2, 9, 16 | Dead | — |
| BLACK TIDE | 77335935 / 3857245 | Black Tide LLC | 9, 25, 41, 42 | Dead | — |
| HIGH TIDE | 76134681 / 2636671 | Lockheed Martin | 9 | Dead | — |
| RISING TIDE | 98275293 | Everi Games | 9 | Dead | — |

Other sources:

- Same-field projects named tide, recorded in research/28 §4 and fx-gbq4's
  Why [repo evidence; web: crates.io and GitHub APIs, 2026-11-13]. No
  registration for any of them appears in the sample, and any unregistered
  rights from their use were not assessed:
  - http-rs/tide, a Rust async web framework: the `tide` crate (about 3.7M
    downloads) and many `tide-*` plugin crates. The closest in field: the
    same ecosystem and audience as Tide.
  - IlanCosman/tide, a fish-shell prompt that defines a `tide` command.
  - ananthakumaran/tide, an Emacs TypeScript IDE that owns `tide-mode`.
  - Stanislas7/tide, an experimental programming language written in C.

  research/28 §4 accepted these clashes. Its answer to them is to tell Tide
  apart: the `tidelang` handle and prefix, the `tidelang` CLI,
  `tidelang-mode`, and "the Tide language" in prose.
- A trademark directory lists one pending class 42 filing by
  "Tide Software Inc.", but it does not name the mark [web:
  https://trademarkdir.com/owner/tide-software-inc]. Unverified.
- Developer tools named TIDE, with no registration found in the sample
  [web: https://nikit.tcl-lang.org/page/TIDE;
  https://docs.tibbo.com/intro_tide]: a proprietary Tcl IDE, and Tibbo's
  IDE. Any unregistered rights from their use were not assessed (see the
  gap above).

### P&G's TIDE (class 3) and dilution

**Evidence** [web: tmsearch.uspto.gov sample]:

- P&G's live TIDE registrations include:
  - 1063680, "DETERGENT FOR LAUNDRY, HOUSEHOLD AND INSTITUTIONAL USE" (class 3, registered and renewed);
  - 3495689, charitable volunteer programs (class 35);
  - 6555117, 6555118 and 8161353, laundry services (classes 37 and 39);
  - 3373965, clothing (class 25).
- None of P&G's records in the sample is in class 9 or 42.
- Live third-party TIDE registrations coexist with P&G's in other fields:
  - Tide Ocean SA, #TIDE, 6896053 and 6907787 (classes 17, 22, 40);
  - Tide Seltzer Brands, 7952376 (class 32);
  - Playa Bowls, 8096013 (class 32);
  - Emma Coalition, 6654086 (class 35);
  - Tide International (USA), 7573059 and 5448720 (classes 1 and 5);
  - Koninklijke Mosa, 6242204 (class 19);
  - Coastal Conservation Association, 5570595 (classes 16 and 41).

**The statute** [web: https://www.law.cornell.edu/uscode/text/15/1125]:

- 15 U.S.C. §1125(c)(1) entitles "the owner of a famous mark" to an
  injunction against a use "in commerce that is likely to cause dilution by
  blurring or dilution by tarnishment [...] regardless of the presence or
  absence of actual or likely confusion, of competition, or of actual
  economic injury".
- §1125(c)(2)(A): "a mark is famous if it is widely recognized by the
  general consuming public of the United States as a designation of source
  of the goods or services of the mark's owner".
- §1125(c)(2)(B): "dilution by blurring" is "association arising from the
  similarity between a mark or trade name and a famous mark that impairs
  the distinctiveness of the famous mark". Its listed factors include
  "(i) The degree of similarity between the mark or trade name and the
  famous mark", "(iii) The extent to which the owner of the famous mark is
  engaging in substantially exclusive use of the mark" and "(v) Whether the
  user of the mark or trade name intended to create an association with
  the famous mark".
- §1125(c)(2)(C): "dilution by tarnishment" is "association arising from
  the similarity between a mark or trade name and a famous mark that harms
  the reputation of the famous mark".
- §1125(c)(3)(C) excludes "Any noncommercial use of a mark".
- A secondary opinion, a law-firm blog, says dilution protection suits
  only household-name "super marks" [web search result for
  https://www.stites.com/?p=16166; the page itself could not be fetched or
  saved (HTTP 403 to the reviewer), so the claim is unverified background
  only].
- This search found no reported decision in which P&G asserted dilution of
  TIDE against an unrelated field.

**Analysis** [inference; provisional, not a legal assessment]:

- TIDE plausibly meets the US fame test.
- Dilution applies "regardless of [...] competition", so a different
  market or Nice class is no defence to it. Separate markets bear on
  likelihood of confusion, a different test (the registration findings
  below).
- Factor (i) weighs against Tide: the name is the identical word.
- Factor (iii) may weigh for it: the live third-party TIDE registrations
  above suggest P&G's use of the word is not substantially exclusive.
- Factor (v) weighs for it: Tide's name comes from tide tables
  (`tide-naming/decision.md`, Reasons Tide won), not from the detergent.
- Not assessed:
  - Whether Tide's free distribution is "noncommercial use" under
    §1125(c)(3)(C). This note does not assume the exclusion applies: free
    software can still be use in commerce.
  - Tarnishment, which turns on harm to the famous mark's reputation.
  - EU reputation protection (EUTMR Article 9(2)(c)); the EU register was
    not searched (gap above).
- Provisionally low: nothing found makes a claim likely, but nothing here
  shows one would fail. A paid offering, sponsorship or 1.0 is the point
  to have counsel assess dilution together with the clearance search.

### Verdict for "Tide": low, provisional

"Low" means that no evidence was found of a conflict likely to force a
rename; it is not a clearance.

Registration-search findings, within the sample [web: tmsearch.uspto.gov]:

- In the 100 USPTO records read, out of 1,265, no live mark covers
  programming languages or developer tools.
- Tide Platform's live application covers software only "in the fields of
  banking, finance, accounting, and tax", a different field and buyer
  [inference].

Not assessed:

- The other 1,165 USPTO records.
- The EU, UK and WIPO registers.
- Unregistered rights, including those of the same-field projects named
  above (http-rs/tide, the closest; IlanCosman/tide; ananthakumaran/tide;
  Stanislas7/tide) and of the other TIDE developer tools. research/28 §4's
  disambiguation is the answer to those uses.
- P&G dilution, which stays unresolved and provisionally low (above).

Three steps keep the risk down. They are examples, not a safe harbour
[inference]:

- Present it as "the Tide language" with the `tidelang` handle, as
  research/28 §4 decided.
- Use no detergent, laundry or P&G-like imagery (orange and yellow
  bullseye, "pods").
- Run the gap queries and the unregistered-rights searches above, or a
  professional search, before any paid offering, sponsorship or 1.0.

## 3. `tidelang`

- **Trademark registers**: none searched for "tidelang", whether USPTO,
  EUIPO, UK IPO, TMview or WIPO. The "tide" sample does not cover it. The
  queries to run are under §2's sources.
- **Web search**: no exact match for "tidelang". The nearest names are
  TileLang (a GPU kernel language) and timelang [web: web search
  "tidelang", 2026-11-17].
- **GitHub**: `tidelang in:name` returns 2 repositories [web: GitHub search
  API, 2026-11-17; response saved in the fx-gbq4 work directory as
  `dl/gh-repo-search-tidelang.json`]:
  - orbyfied/tidelang: ★0, "Trying Zig", created 2024-08-17, last pushed
    2024-08-18.
  - LazyAshleyz/TideLang: ★1, "Tide is a interpreted programming
    language", created and last pushed 2026-11-06, eleven days before this
    note.
- **crates.io**: `tidelang`, `tidelang-admit`, `tidelang-std`,
  `tidelang-macros` and `tide-lang` all return 404, which means free [web:
  crates.io API].
- **nixpkgs**: no attribute; see §4.

**Verdict: low, provisional** [inference]:

- No registered or commercial holder was found. No register was searched,
  though.
- LazyAshleyz/TideLang is a concrete name collision: a programming
  language called Tide, with "TideLang" as its repository name, active
  this month. Any rights it holds from use were not assessed. Its small
  size does not by itself establish that Tide can use the name alongside
  it.
- The register queries and the unregistered-rights searches in §2 apply to
  `tidelang` as well.

## 4. Nix attribute names for fx-1n5n

**nixpkgs** [web: GitHub API on NixOS/nixpkgs `master` at `5ee8932f5bf3`,
2026-11-17T16:32Z]:

- `pkgs/by-name/ti/` has no `tide*` package. The nearest are `tideways-cli`,
  `tideways-daemon` and the `tidal*` packages.
- `pkgs/top-level/all-packages.nix`, `python-packages.nix`,
  `perl-packages.nix`, `ocaml-packages.nix` and `haskell-packages.nix`
  contain no `tide` attribute.
- Nested sets that do hold `tide`:
  - `fishPlugins.tide`, IlanCosman's fish prompt
    (`pkgs/shells/fish/plugins/tide.nix`, 6.2.1);
  - `emacsPackages.tide`, from MELPA (`recipes-archive-melpa.json`).
- An authenticated GitHub code search, `tidelang repo:NixOS/nixpkgs`, returns
  0 hits (`incomplete_results: false`, 2026-11-17T17:03Z). A control
  search, `tide repo:NixOS/nixpkgs path:pkgs/shells/fish/plugins`, returns
  the 2 expected files. Both responses are saved in the fx-gbq4 work
  directory as `dl/gh-code-search-tidelang-nixpkgs.json` and
  `dl/gh-code-search-tide-fishplugins-control.json`. Reproducing them needs
  a GitHub token; the unauthenticated API answers 401.
- `vimPlugins` has no `tide`.
- Gap: search.nixos.org is a JavaScript page; the query
  `https://search.nixos.org/packages?channel=unstable&query=tide` was not
  read.

**Decision** [inference]:

- `packages.<system>.tidelang-admit`: the admission driver, crate
  `tide-admit`.
- `packages.<system>.tidelang-std`: Tide's std, crate `tide_std`.
- `tidelang-macros` is not exposed. `tide-macros` is a proc-macro crate
  that only `tide_std` consumes, so it is a build input of `tidelang-std`.
  If fx-1n5n finds it needs its own output, the name is `tidelang-macros`.
- `packages.<system>.default` = `tidelang-admit`.
- `apps.<system>.default` and `apps.<system>.tidelang-admit` run its
  `tide-admit` binary. Binary and crate renames are out of scope (fx-gbq4);
  the Nix attribute names and package names (`pname`) are set to
  `tidelang-*`. The repository has no flake yet.
- No bare `tidelang` attribute yet. research/28 §4 reserves `tidelang` for
  the future command-line tool.

**Reasons** [inference]:

- Top-level `tide` is free in nixpkgs, but `tide` already names the fish
  prompt and the Emacs package there. `nix search` and any overlay would
  list `tide-*` beside them.
- The prefix matches research/28 §4: `tidelang-*` crates, the `tidelang`
  CLI, and `tidelang-mode`.
- The driver is the default because it is what a user runs.
- The default must work with nothing else installed. tide-admit passes
  Tide's std as `std` (D18), so it cannot admit a crate without
  `tidelang-std`. This note recommends that fx-1n5n wire `tidelang-std`
  into the driver's runtime closure, for example a wrapper that points the
  driver at the std's store path. Merely documenting the dependency would
  ship a default app that fails on its own. How to wire it is fx-1n5n's
  design.
- Naming the default `tidelang` now would give the name two meanings once
  the CLI ships.

## Sources

- Rust Foundation, "Rust Language Trademark Policy": https://rustfoundation.org/policy/rust-trademark-policy/ (read 2026-11-17).
- 15 U.S.C. §1125(c): https://www.law.cornell.edu/uscode/text/15/1125
- design/01-design.md D18 (the std swap).
- USPTO Trademark Search: https://tmsearch.uspto.gov/ (word-mark query "tide"; top 100 of 1,265 records). Record pages, for example https://trademarks.justia.com/984/83/tide-98483640.html
- Tide Platform marks: https://trademarks.justia.com/owners/tide-platform-ltd-5841602
- Tide Software Inc.: https://trademarkdir.com/owner/tide-software-inc
- Dilution threshold, secondary opinion, not fetched: https://www.stites.com/?p=16166
- Gaps, not searched: EUIPO eSearch https://euipo.europa.eu/eSearch/, TMview https://www.tmdn.org/tmview/, UK IPO https://trademarks.ipo.gov.uk/ipo-tmtext, WIPO Global Brand Database https://branddb.wipo.int/, search.nixos.org, and unregistered (common-law and state) rights.
- GitHub repository search for `tidelang in:name`: https://api.github.com/search/repositories?q=tidelang+in%3Aname
- hax: https://github.com/cryspen/hax; Verus: https://github.com/verus-lang/verus
- Same-field projects named tide (via research/28 §4): https://github.com/http-rs/tide, https://github.com/IlanCosman/tide, https://github.com/ananthakumaran/tide, https://github.com/Stanislas7/tide
- Other software named TIDE: https://nikit.tcl-lang.org/page/TIDE, https://docs.tibbo.com/intro_tide
- crates.io API: https://crates.io/api/v1/crates/tidelang (and the four other names).
- nixpkgs at `5ee8932f5bf3`: `pkgs/by-name/ti/`, `pkgs/top-level/*.nix`, `pkgs/shells/fish/plugins/tide.nix`, `pkgs/applications/editors/emacs/elisp-packages/recipes-archive-melpa.json`, `pkgs/applications/editors/vim/plugins/vim-plugin-names`.
- Tide: `README.md`, research/28 §4, `/home/lucca/.cache/eon-agents/tide-naming/decision.md` and `notes.md`, beads fx-gbq4 and fx-1n5n.
