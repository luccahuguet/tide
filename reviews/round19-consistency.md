# Tide consistency review, round 19: rule preservation in the reference split

## Verdict

**request changes**

Reviewed commit `6d5285c52bede9adaf5161de3fdf367eb2300833` (`6d5285c`, "Fit
the core reference under D14's budget with an on-demand companion
(fx-fbq)"), dated 2026-10-19 19:57 -0300. It was diffed against `d98e3a1`.
Scope, as asked:

- whether every v0 rule, FX code, provisional marker, and constraint in the
  old `design/reference.md` is still stated, with the same meaning, in the
  new core or in `design/reference-companion.md`. The comparison went
  section by section;
- whether any rule that ordinary v0 code needs now lives only in the
  companion with no pointer from the core;
- whether citations from `design/0*.md` and `README.md` into either file
  still resolve;
- the ten spots the author flagged.

The owner's decision to move the simulator's faults, nodes, and crashes to
companion §17, with pointers in core 6.8, is taken as given. The design
chapters were not re-audited.

Two changes are needed, and both are small. One sink guarantee was dropped
and needs about 15 tokens to restore. Three citations in 03 still point at
the core for text that moved to the companion.

What holds:

- **No FX code was lost.** The old file's codes are all in the new pair, and
  the pair adds none. The only code now found only in the companion is
  `FX0510`, a v1 lint in Appendix A.
- **Every provisional marker survives.** The old file has 43; the core has
  39 and the companion 5. Each old marker maps to one in the new pair. One
  marker moved away from the rows it qualifies (N3).
- **The moved sections are verbatim.** These match the old text exactly:
  - 15.1–15.3, 16, and Appendix A;
  - the old 14 table, now 14.1, with one cross-reference changed from 6.9
    to 14.2;
  - the old 6.9 and 7.6 tables, now 14.2 and 14.3.
  15.4 joins the old 6.3 `Log` example to the old 6.8 crash test.
- **The core still states every rule the companion's 14–16 tabulate.**
  Every 14.1 row has its rule and code in the core (for example `FX0505` in
  7.4, `FX0605` in 7.3, and `FX0602` in 7.1). Each 16 reading is taken in
  the core: 6.3 for relative paths, 7.4 for `with_all`, and 6.6 for
  `#[pure]`.
- **The 6.6 requirements table survives.** The table became one sentence
  (`reference.md:488–490`), and each row is still stated where it applies:
  - `join` and slice-method closures and captures in 8 (`:784–787`);
  - `Shared` payloads and `with` closures in 7.4 (`:703`, `:718`, `:732`);
  - statics in 6.4 (`:427`);
  - `const fn` in 2.3 (`:101–102`);
  - contract-clause calls in 10.1 (`:855–857`).
- **The 6.8 pointers are enough for what moved.** 6.3 points to companion
  17 for crash loss (`:416`) and to 15.4 for the crash-safe log (`:420–421`).
  6.8 points to it for `sim.faults` and `sim.node` (`:510–511`) and for the
  replay manifest (`:537`).
- **01, the README, and 03 §8.3 are updated.** D14 names the companion
  (`01-design.md:824–825`). Open question 8 is removed, and nothing still
  cites it. The README lists the companion (`README.md:57–60`). 03 §8.3 now
  cites `reference-companion.md` Appendix A (`03:978–979`). 01's citation
  of §12.1–§12.2 (`01:175`) and 03's of §6.10 (`03:582`) still resolve.

## The author's flagged spots

| Spot (new core) | Judgment |
| --- | --- |
| `:34` §1 enumerations | **Acceptable**, with one gap: N1. Syntax, patterns, ownership, drop check, and the trait-system list are Rust restatement, and "everything not listed works as in Rust 2024" covers them. Every Tide-specific clause in the old list is kept: immutable `static`, `unsafe` with a grant, `Arc` with `&Tasks`, and test parameters. What the old list also did was say what 6.9's allowlist admits. |
| `:108` Tide attribute list | **Note N2.** Each attribute is still spelled in its section. The closed list is gone, and so is the distinction between proc macros and derive helper attributes. |
| `:193` `dyn` lifetime defaults | **Acceptable.** The old text restated Rust's object-lifetime defaults (RFC 599/1156) in full. §3's opening (`:115–117`) says object-lifetime defaults are Rust 2024's, and the `Box<dyn Trait + 'a>` rule an agent needs is kept. |
| `:200` "return or store it only where its lifetime is named or captured" | **Acceptable.** This is Rust behaviour, and the bullet keeps what matters: a borrowing closure is a borrowing value, a returned `impl Trait` captures lifetimes, and a `'static` position needs `FX0203`'s fix. One wording point is in N5. |
| `:218` `Rc<RefCell<T>>`/`Weak`/`Arc` row | **Acceptable.** These are Rust's types, and §1 (`:40–41`) keeps them with their API. `Arc`'s Tide constraints are in 6.4 and 7.4. |
| `:331` `Env`/`Stdin` "as std's", `Command` builder list | **Acceptable.** The dropped signatures equal std's (`var -> Result<String, VarError>`, `current_dir -> io::Result<PathBuf>`, `read_line -> io::Result<usize>`). The old builder list was examples after "Rust's builder", not a closed set. On `Stdin::lines`, see N5. |
| `:447` "so sink configuration never changes results or panics" | **Blocking: B1.** |
| `:670` 7.2 "Soft" merged, "destructors run" dropped | **Acceptable.** The merged sentence keeps "never interrupted" for authority-free code, `with*`, and joins, and adds scope exit, which 7.1 (`:608–609`) already said cannot be cancelled. "Cancellation is observed at the next checkpoint" follows from the level-triggered bullet. Destructors running on an `Err` return is Rust behaviour. |
| `:844` `old()` wording | **Same meaning.** `Copy` values are copied and others cloned. Neither `Copy` nor `Clone` is an error, and a reference snapshots its referent. The cost examples and the `result`/quantifier restriction are kept. Only line length is off (N5). |
| `:511` 6.8's pointers to companion 17 | **Sufficient** for faults, nodes, and crashes, within the owner's decision. Two leftovers are in N3 and N4. |

## New blocking problems

### B1. The sink guarantee "configuration never changes results or panics" is gone from both files

The old 6.5 (`old reference.md:641–642`) says: "Format arguments are always
evaluated, whatever level is enabled, so sink configuration never changes
results or panics."

The new 6.5 (`reference.md:447`) keeps only the first half: "Format
arguments are always evaluated, whatever level is enabled." The companion
has nothing on sinks.

The second half is not just a corollary of the first:

- **The normative text states it as a rule.** 01 D7 (`01-design.md:443–445`)
  says "so configuration cannot change results or panics". 03 SINK-3
  (`03:287–292`) goes further: "Format arguments are always evaluated, and
  the record formatted, … Configuration therefore cannot change results,
  panics, or state that an argument or a formatting impl mutates."
- **The half that is left does not imply it.** Argument evaluation is not
  formatting. In Rust's `log` and `tracing`, a disabled record's `Display`
  impl never runs. A Rust programmer reading only "arguments are always
  evaluated" can still expect a panicking or `Cell`-mutating `Display` impl
  under a disabled `debug!` to stay silent. The dropped clause is what told
  them otherwise.

**Fix.** In `reference.md:447`, write: "Format arguments are always
evaluated and the record formatted, whatever level is enabled, so sink
configuration never changes results or panics." This costs about 15
tokens; the core has 176 to spare.

### B2. Three citations in 03 still credit `reference.md` with the v0 exclusions that moved to the companion

`pct` scheduling and `--minimize` are left out of v0 only by Appendix A,
now `reference-companion.md:454–458`. The core's "Not in v0" line
(`reference.md:1071–1073`) does not name them. 03 still cites the core
file:

- `03-authority-concurrency.md:18–19`: "Not tiered by 01 and left out of
  v0 by `reference.md`: `pct` scheduling and `--minimize` (§9)";
- `03:1053`: "`pct(depth)` (… not in v0 per `reference.md`)";
- `03:1136`: "`--minimize` (not in v0 per `reference.md`)".

The commit already fixed the same kind of citation at `03:978`, for
`par_reduce`, so these three are leftovers.

A related scope sentence needs the same edit. `03:29` says "API names
marked provisional in `reference.md` are fixed here unless noted". Some of
those markers are now only in the companion: `SimNode`, `NodeTask`, the
retransmission timeout, the power cut on a finished node, and restarts. As
written, 03's override no longer reaches them.

**Fix.** At `03:19`, `03:1053`, and `03:1136`, cite
`reference-companion.md` (Appendix A). At `03:29`, write "in
`reference.md` and its companion". None of these edits changes the core's
token count.

## Non-blocking notes

### N1. The core no longer says which library items 6.9's allowlist admits

`reference.md:557–563` says the checker "admits only listed `core`,
`alloc`, and std inherent items; everything else is `FX0520`". The list
itself is not in the reference; 03 ADMIT-1 (`03:333–354`) defines it.

The old §1 was the reader's positive side of that rule. It named the std
traits that work as in Rust (`Deref`, `Iterator`/`IntoIterator`,
`From`/`Into`/`TryFrom`/`TryInto`, `Display`/`Debug`, `Default`, the `Fn`
traits, and the operator traits). It also said "every `Iterator` adapter"
works and that `collect` works into any `FromIterator` (`old
reference.md:63–71`).

The new §1 (`reference.md:34–37`) says "traits and generics within 5's
limits", and its standard-types list covers types only. An agent reading
6.9's "admits only listed" cannot tell from the core whether `Iterator`
adapters and the conversion traits are on the list. The old text answered
that, and the intent is clear, so this is a note.

**Fix.** In §1, after "within 5's limits", add: "std's traits, every
`Iterator` adapter included, are admitted (6.9)." This costs about 15
tokens.

### N2. The Tide attribute names are no longer listed, and helper attributes are now called proc macros

The old 2.3 (`old reference.md:179–183`) listed the Tide attributes:
`#[error]`, `#[from]`, `#[source]`, `#[codec]`, `#[tide::main]`, `#[pure]`,
`#[sim_test]`, `#[requires]`, `#[ensures]`, `#[invariants]`,
`#[protected]`, and `#[trusted_pure]`. It called them "toolchain proc
macros and their derives' helper attributes".

The new 2.3 (`reference.md:108–109`) says "Tide's own attributes (each in
its section) are toolchain proc macros". Each attribute is still spelled
in its section, so no rule is lost. Two things are:

- **Which names are v0.** The companion's v1 list has `#[invariant(..)]`,
  `#[contracts(..)]`, `#[decreases(..)]`, `#[spec]`, and `#[trusted]`
  (`reference-companion.md:430–434`). Without the list, nothing in the core
  sets the v0 set apart from these. The closest pair is v0's `#[invariants]`
  (`reference.md:826`) and v1's `#[invariant(..)]`.
- **Helper attributes.** `#[error]`, `#[from]`, `#[source]`, and `#[codec]`
  are derive helper attributes, not proc macros. The new sentence calls
  every Tide attribute a proc macro.

**Fix.** Restore the old sentence. It costs about 50 tokens. With B1 and
N1, the total stays under the 176 tokens of headroom; re-measure after the
edit.

### N3. The "setup parameter types are provisional" marker moved away from the rows it qualifies

The old file says (`old reference.md:740–741`): "(`SimNode`, `NodeTask`,
`Request`, `Record`, and the setup parameter types are provisional.)" It
was split in two:

- core: "(`Request` and `Record` are provisional.)" (`reference.md:525`);
- companion: "(`SimNode`, `NodeTask`, and the setup parameter types are
  provisional.)" (`reference-companion.md:392`).

The setup rows stayed in the core (`reference.md:519–521`): `world.file`,
`world.args`, and `world.http` with its `impl Fn(&Request) -> Reply +
'static`. Their marker is now only in the companion. A core-only reader
takes those signatures as decided.

**Fix.** At `reference.md:525`, write "(`Request`, `Record`, and the setup
parameter types are provisional.)", and drop "and the setup parameter
types" from `companion:392`.

### N4. `tasks.fault_point` is now described only in the companion, and its core citation points at 7.1, which does not mention it

The 6.1 `Tasks` row lists `fault_point` and cites 7.1 (`reference.md:334`).
7.1 has no `fault_point`, and neither did the old 7.1. The old reference
described it in one place, the 6.8 row `sim.faults.fault_point(name, p)`:
"probability that `tasks.fault_point(name)` returns `true`" (`old
reference.md:733`). That row is now `companion:386`.

`fault_point` is called from production code, not from tests. 03 §1.1
(`03:105–107`) defines it: it returns `false` under every real runtime and
is a seed-driven trigger under the simulator. Neither file has ever stated
the `false` half. Moving the fault row was the owner's call, but it leaves
the core with no description and a citation that does not resolve.

**Fix.** In the `Tasks` row, replace `fault_point` (7.1) with
`fault_point(name) -> bool` ("`false` outside simulation; companion 17").
This costs about 12 tokens.

### N5. Nits

- **Line length.** `reference.md:846` is 91 characters; reflow it.
- **The `Id<T>` marker's reach.** `reference.md:233–234` ends a single list
  with "`Ord`, and `Debug` (provisional)". The old text separated the
  decided traits from the provisional ones ("and also `Ord` and `Debug`
  (provisional)"). The `(02 §10.9)` citation still marks the boundary, but
  "and also" makes it plain.
- **"Companion trait".** `reference.md:474`, "Declare a companion trait",
  now shares a word with the companion file. "a helper trait" avoids the
  collision.
- **`Stdin` "as `io::Stdin`'s".** In std, `Stdin::lines` takes `self`. The
  next bullet's `&self` rule settles it, but "(receivers `&self`, below)"
  would save a lookup.
- **What `impl Trait` captures.** `reference.md:201` says it "captures every
  lifetime in scope". The old text said "every generic parameter in scope,
  lifetimes included". That is still true, and it matters for `use<..>`,
  which must list the type parameters too.

## Ready to implement v0

**Yes, for the checker.** The checker's test corpus does not depend on the
reference.

Fix B1 before E1's held-out round (`fx-jv9`) loads the reference: that
round measures what an agent learns from the core alone. B2 changes no
reference text and can land with B1. N1–N4 cost about 85 tokens together,
inside the core's headroom, and are worth doing in the same edit. Re-measure
the core afterwards with the method in its header.

Validation: this was a read-only review of `6d5285c` (diffed against
`d98e3a1`) on 2026-10-19. The old reference was extracted with `git show
d98e3a1:design/reference.md` into a session scratchpad, and then:

- **Read in full:** the old reference, the new core, and the companion,
  section by section;
- **FX codes:** the old file's set compared with the core-plus-companion
  set;
- **Code spans:** every backticked span in the old file that appears in
  neither new file was listed and checked by hand. Each is Rust
  restatement, an example, or reworded in place, except the B1 and N1–N4
  cases;
- **Provisional markers:** each of the old file's 43 was matched to the new
  pair;
- **Moved text:** 15.1–15.3, 16, Appendix A, and the 14 table were diffed
  against their old text;
- **Citations:** `reference.md` and `reference-companion.md` were grepped
  for in `design/0*.md` and `README.md`, and each hit was checked against
  the new files.

The token count was not re-measured. The 176-token headroom comes from the
bead's recorded count (24,824), and the costs above are estimates. Only
this file was written; nothing was staged or committed.

## Resolution

Answered in the commit that adds this review.

- **B1:** 6.5 restores the rule: the record is formatted whatever level is
  enabled, so sink configuration never changes results or panics.
- **B2:** 03's tier line, its `pct` strategy line, and its `--minimize`
  line now cite `reference-companion.md`. Its intro now says names marked
  provisional "in `reference.md` or its companion" are fixed there.
- **N1:** §1 states that std's traits, every `Iterator` adapter included,
  are admitted (6.9).
- **N2:** 2.3's list of Tide attribute names is back, with the
  derive-helper wording.
- **N3:** the setup parameter types' provisional marker is back in core
  6.8, and the companion drops it.
- **N4:** the `Tasks` row gives `fault_point(name) -> bool`, which is
  `false` outside simulation (03 §1.2), and points to companion 17.
- **N5:**
  - line 846 is rewrapped;
  - `Id<T>` reads "and also `Ord` and `Debug` (provisional)";
  - `Stdin` points to the `&self` rule;
  - `impl Trait` captures every generic parameter, lifetimes included.
  - Declined: renaming "companion trait", since D7 and 03 §3.1 use the
    term.
- **Budget:** the fixes came to 25,052 tokens. The counting method moved
  from the core's header to the companion's, and the core now measures
  24,950.
