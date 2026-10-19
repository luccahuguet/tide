# E1-origins grading: `from` clauses against Rust lifetime syntax

**Inputs** (revision `1ac8fe8`):

- `tasks.md`: ten borrow-heavy library and binary tasks, held out from both
  references.
- **Arm A** (`from` clauses): `arm-a/reference.md`, ten programs in
  `arm-a/programs/` (1,380 lines), and `arm-a/notes.md`. The reference is
  `design/reference.md` with three "(provisional)" markers on §0 item 3 and
  §3 removed, so the writer was not told that §3 is under trial.
- **Arm B** (Rust lifetime syntax, origin-equivalent subset):
  `arm-b/reference.md`, ten programs in `arm-b/programs/` (1,279 lines), and
  `arm-b/notes.md`. `diff` against arm A shows 30 hunks, all in §0 item 3,
  the §1 ownership bullet, §3, the `Scope`/`Task` signatures and the
  scope-passed-down bullet in §7.1, and the lifetime rows of §14. The rest is
  identical.

Each writer was a fresh agent that read only its arm's reference and
`tasks.md`.

**Normative sources.** `design/01-design.md` (D1, D2, Tiers);
`design/02-memory.md` for origin formation, places, elision, variance, drop
check, the legacy mapping (§2.6), and the translation to Rust (§11);
`03-authority-concurrency.md` §9 and SINK-6 for the simulator;
`04-contracts.md` §2.4 for generated tests. 01 and 02 were last edited at
12:25 and 12:01, before the programs were written (13:03–13:06), so nothing
moved during the trial.

**Protocol deviation.** Arm B's §13 table (line 1375) still says
`tide fmt` "rewrites legacy lifetime spellings to `from`", a leftover from
arm A. Writer B noticed it (notes, cross-cutting) and wrote `'a` throughout.
No arm-B program contains `from`. The row did not change any verdict, though
it did tell writer B that another spelling exists.

## Method

The rubric is that of `trials/e1/grading.md` and
`trials/e1-round2/grading.md` (their Method sections). In brief:

- **Classes:** Rust habit leaked, reference gap, design gap or
  contradiction, ordinary bug.
- **Definite or contingent:** a definite error is one the normative design
  rejects, or a failing test. A contingent item is a detail no normative
  source decides; the writer's guess is graded as passing.
- **Severity:** 0–3.
- **Check runs** count `tide check` runs to green. **All runs** add failing
  `tide test` runs. Both assume diagnostics that carry the fixes.

This round adds one dimension: every definite error and contingent item is
tagged **borrow-related** (origin or lifetime spelling, origin or lifetime
semantics, elision, borrowing types, captures) or **other**. Review nits are
listed but not counted, as in round 2.

**How each arm is judged.**

- **Arm A** is judged by its reference §3 and by 02. A body is accepted
  exactly when rustc's NLL checker accepts its §11 translation (02 §2.8).
- **Arm B** is judged as rustc would judge that subset, under Rust 2018's
  rule for returned `impl Trait` (arm-B reference §3.5). Arm-B §3 is the
  spelling rule, and 02 §11 is the statement that the two spellings mean the
  same thing.

Where a question is decided by 02 but not by an arm's reference, it counts as
a reference gap that caused no error. Section 3.0 lists every such question
and the reading taken.

**Tracing.** Every test was traced by hand. Each drop-the-container test was
also traced as the checker would see it: which origin the result carries,
which loans that origin holds, and whether the drop conflicts with a live
loan (02 §1.4) or with drop check (02 §6). The writers' confidence ratings
did not move any verdict.

**Annotation counts** exclude comments.

- **Arm A:** each `from` keyword after a type is one clause.
- **Arm B:** each written lifetime name other than `'_` and `'static` is one
  token, whether it is a declaration or a use. `'_` is counted separately.

## 1. Results at a glance

### Arm A (`from` clauses)

| Program | First `tide check` | Definite errors | Contingent | Severity | Check runs | All runs |
| --- | --- | --- | --- | --- | --- | --- |
| ini | pass | none | none | 0 | 1 | 1 |
| path_norm | pass | none | none | 0 | 1 | 1 |
| diff_lines | pass | none | none | 0 | 1 | 1 |
| merge_words | pass | none | C3 | 0 | 1 | 1 |
| highlight | pass | none | none | 0 | 1 | 1 |
| filters | pass | none | none | 0 | 1 | 1 |
| group_words | pass | none | none | 0 | 1 | 1 |
| table | pass | none | none | 0 | 1 | 1 |
| logscan | pass | none | C1, C2 | 0 | 1 | 1 |
| kv_cursor | pass | none | C3 | 0 | 1 | 1 |

### Arm B (Rust lifetimes)

| Program | First `tide check` | Definite errors | Contingent | Severity | Check runs | All runs |
| --- | --- | --- | --- | --- | --- | --- |
| ini | pass | none | none | 0 | 1 | 1 |
| path_norm | pass | none | none | 0 | 1 | 1 |
| diff_lines | pass | none | none | 0 | 1 | 1 |
| merge_words | pass | none | C3 | 0 | 1 | 1 |
| highlight | pass | none | none | 0 | 1 | 1 |
| filters | pass | none | none | 0 | 1 | 1 |
| group_words | pass | none | none | 0 | 1 | 1 |
| table | pass | none | none | 0 | 1 | 1 |
| logscan | pass | none | C1, C2 | 0 | 1 | 1 |
| kv_cursor | pass | none | none | 0 | 1 | 1 |

## 2. Aggregate

### 2.1 Pass rates and runs

| Measure | Arm A | Arm B |
| --- | --- | --- |
| First-check pass rate | **10/10** | **10/10** |
| Final pass rate | 10/10 | 10/10 |
| Check runs to green (total, mean) | 10 (1.0) | 10 (1.0) |
| All runs (total, mean) | 10 (1.0) | 10 (1.0) |
| Failing tests | 0 | 0 |
| Drop-the-container tests, all passing | 12 | 11 |

The plausible range is also 10/10 in both arms. No contingent item can fail a
check: C1 and C2 could at most force test edits in logscan, in both arms
alike. C3 cannot fail under any reading (§2.3).

### 2.2 Definite errors by class

| Class | A, borrow-related | A, other | B, borrow-related | B, other |
| --- | --- | --- | --- | --- |
| Rust habit leaked | 0 | 0 | 0 | 0 |
| Reference gap | 0 | 0 | 0 | 0 |
| Design gap or contradiction | 0 | 0 | 0 | 0 |
| Ordinary bug | 0 | 0 | 0 | 0 |
| **Total** | **0** | **0** | **0** | **0** |

Neither arm leaked a Rust habit:

- Arm A wrote no `'a` and no `<'_>`.
- Arm B wrote Rust 2018's `+ '_` on every returned `impl Trait` that
  borrows, not Rust 2024's implicit capture, and used none of the excluded
  forms (`'a: 'b`, `T: 'a`, `for<'a>`, trait lifetime parameters,
  `use<..>`).

Review nits, not counted (§3): N1 (arm A ini), N2 (arm B kv_cursor), N3 (arm A
kv_cursor). All three are tagged other.

### 2.3 Contingent items

| ID | Arms, programs | Tag | Writer's assumption | Status |
| --- | --- | --- | --- | --- |
| C1 | A and B, logscan | other | A `println!`/`eprintln!` from code the test body calls becomes one `Record` per line in `sim.records()`, with `sink: Sink::Out`/`Sink::Err` and `message: String` without the newline | 03 SINK-6 and §9.2 fix that records exist and that the test body is the simulation's main task. The `Record` shape and per-line granularity are provisional in reference §6.8. A different shape would force test edits, not program changes. |
| C2 | A and B, logscan | other | `sim.world.args([..])` takes `&str` arrays of any length and returns `()`, not a `Result` (FX0702 otherwise). `SimRoot` has an `env` field whose `args()` returns those strings, program name first. | 03 §9.2 gives `SimRoot` "the same capability fields as `Root`" and lists `world.args`, but not its types. Reference §6.8 marks the setup types provisional. |
| C3 | A merge_words, kv_cursor; B merge_words | other | Generated contract tests (reference §10.2, 04 §2.4) either build or skip functions with an `ensures` whose parameters hold nested borrows: `merge_sorted(&[&str], &[&str])`, `dedup_adjacent(&mut Vec<&str>)`, and A's `parse_pairs(&str, &mut Vec<(&str, &str) from src>)` | Undecided: 04 §2.4 rule 1 derives generators for "structural types" and is silent on borrowed elements. Under every reading the outcome is pass or "unchecked", never fail: each `ensures` holds for all inputs that satisfy the `requires`, and `is_sorted` is a recognized shape, so `merge_sorted` cannot go vacuous. Tagged other because the question is about the test generator and is identical in both arms. |

No borrow-related contingent item remains in either arm. Every borrow
question the writers raised is decided by 02 or by rustc (§3.0).

### 2.4 Annotation burden

| Program | A: `from` clauses | B: named lifetime tokens | B: `'_` |
| --- | --- | --- | --- |
| ini | 6 | 13 | 2 |
| path_norm | 0 | 2 | 3 |
| diff_lines | 4 | 17 | 0 |
| merge_words | 1 | 5 | 0 |
| highlight | 3 | 14 | 0 |
| filters | 3 | 8 | 1 |
| group_words | 3 | 11 | 0 |
| table | 5 | 11 | 1 |
| logscan | 0 | 4 | 4 |
| kv_cursor | 3 | 14 | 0 |
| **Total** | **28** | **99** | **11** |

- **Arm A.** 28 `from` clauses name 30 places: one per-origin binding names
  two, and one union names two. 26 clauses sit on 22 function signatures and
  2 on field declarations (ini's `entries`, table's `rows`). Apart from
  `from`, A writes one `'_` (the sanctioned `Box<dyn Fn(&str) -> bool + '_>`
  field, reference §3.4) and one `'static`. No struct, enum, or impl carries
  an annotation. Two programs (path_norm, logscan) need none.
- **Arm B.** 30 lifetime declarations:
  - 13 on 11 types: `Section`, `Ini`, `Component`, `Difference`, `Span`,
    `Highlighter`, `Pipeline`, `Table`, `Entry`, `Scan`, `Cursor`;
  - 5 on impl headers;
  - 12 on 10 functions.

  It also has 69 uses, 6 of them optional `let` annotations in bodies. The
  `'_` count is 11 (3 in bodies), and `'static` appears once. Every program
  carries annotations, because every task defines or returns a borrowing
  type.
- **Ratio.** B writes about 3.5 times A's count (99 against 28). Two of B's
  choices inflate it: the optional body annotations and a `Difference<'a,
  'b>` struct where A returns a tuple. Without them B needs about 89, about
  3.2 times A's count.
- **Signatures needing explicit origins.** Both arms are about equal: A 22,
  B 23. The gap comes from B's type and impl declarations and from B
  repeating each name at every use.

### 2.5 Origin and lifetime questions in the notes

- **Arm A: 14.** Nine are explicit uncertainty ("not shown", "I assumed", "I
  treated these as"): QA1–QA5, QA7, QA8, QA10, QA13. Five are stated
  reliance on a documented rule: QA6, QA9, QA11, QA12, QA14.
- **Arm B: 5.** Three are explicit uncertainty: QB1–QB3. One is a hedge
  (QB4), and one is the §13 deviation noticed and set aside (QB5).

All 19 are resolved in the writer's favor (§3.0).

The writers' own confidence ratings point the same way:

- A rated six programs "medium" (ini, highlight, filters, group_words,
  table, logscan). Five of those use a `from` form the reference does not
  show.
- B rated three "medium" (merge_words, group_words, logscan). One of those
  is for a lifetime reason: group_words' implied bound.

The ratings did not affect verdicts.

### 2.6 Legacy lifetime spelling in arm A

None. The only apostrophe forms in arm A are the `'_` object bound in
filters' field (reference §3.4, 02 §5.1) and `&'static str` in logscan
(kept by D1).

## 3. Per-program findings

### 3.0 Borrow questions the references leave open

Every borrow-related question the writers raised, or that grading turned up,
appears below with its resolution. None is a definite error or a contingent
item.

| ID | Arm, location | Question | Resolution and source | Class |
| --- | --- | --- | --- | --- |
| QA1 | A table.tide:17, 42, 49, 55; highlight.tide:29, 51; filters.tide:25 | Which origin does an owned collection field name (`from header`, `from self.rows`, `from self.keywords`, `from self.filters`)? | `orig(p)` is `orig(type of p)` (02 §2.2), and 02 §3.2 names `words: Vec<&str>` as origin `words`. The writer's reading is the normative one. Arm-A §3.3 lists `src`, `*out`, `lx.src`, and `Ident.0`, but not this case. | reference gap, no error |
| QA2 | A filters.tide:25 | Is `from` allowed on an argument-position `impl Fn`? | Yes. 02 §2.4 item 3: `impl Trait from P` imposes `F: 'p` (Rust `F: 'p`). Arm-A §3.5 shows only return position. | reference gap, no error |
| QA3 | A filters.tide:25 | Can a parameter type name an impl origin as `self.field`? | Yes. 02 §3.5 says `self.f` denotes `Self.f`, and 02 §10.5 has `a: &str from self.args`. Arm-A §3.3 shows only `Self.src` on a constructor. | reference gap, no error |
| QA4 | A ini.tide:83, table.tide:42 | Does `impl Iterator<Item = &str from *self> from self` capture both origins? | Yes. The trailing `from self` is the capture clause (02 §5.2), and `self`'s origin covers `*self`'s by well-formedness (02 §2.2). 02 §5.2's wording, "its `from` clause plus the elided positions inside its bounds", does not mention origins named by a `from` inside the bounds. The verdict holds under either reading, because the trailing `from self` covers them, but the wording should be fixed (D-1). | reference gap and design wording gap (D-1), no error |
| QA5 | A group_words.tide:23 | `Option<(char, &[&str from *groups])> from groups`: inner and trailing `from` on one return type | The inner `from` fixes the element positions. The trailing `from` applies to "every elided origin position" (02 §2.3), which leaves only the slice reference. Rust: `&'g [&'t str]`. | reference gap, no error |
| QA6 | A ini.tide:25 | Does `from` after a tuple cover both elements? | Yes: "every elided origin position in the type before it" (arm-A §3.2, 02 §2.3). | decided by the reference |
| QA7 | A ini.tide:74, 83, 89 | Does `*self` reach an origin two levels down (`Vec<Section>` → `Section.name`)? | Yes. A struct's origins include those of its field types (02 §3.1), and `Ini` has exactly one, so `from *self` is an identification (02 §2.4 item 1). | decided by 02 |
| QA8 | A ini.tide:54 | Does `""` coerce into the identified origin of `section_index`? | Yes. `'static` is a subset of every origin (02 §2.1), and `name` is a covariant parameter position. | decided by 02 |
| QA9 | A diff_lines.tide:9 | Does a comma end `from` inside a tuple? | Yes (arm-A §3.2, 02 §2.3). | decided by the reference |
| QA10 | A highlight.tide:29 | Does the comma inside `{ word: text, keyword: self.keywords }` end `from`? | No. Braces are one `Origins` production (02 §2.3), and arm-A §3.2 shows the brace form inside `Option<..>`. | decided by the reference |
| QA11 | A filters.tide:21 | Is `Pipeline::new() -> Self` right when there are no inputs? | Yes. `-> Pipeline` has no input origin to elide from (02 §2.5), and `-> Self` leaves the impl's origin to the caller (02 §3.5). | reference gap (minor), no error |
| QA12 | A table.tide:49 | `from self.rows` with a by-value `self` | Allowed: a place's root may be `self` (02 §2.3). Plain elision would be an error here: the parameters use two origins (`Self`'s and `name`'s), and a by-value receiver triggers no rule 2 (02 §2.5, as in Rust). | decided by 02 |
| QA13 | A filters.tide:5, 10, 25 | Are references inside `Fn(&str) -> bool` higher-ranked, not captures? | Yes (02 §2.5 closure-bound sugar, §5.4). | decided by 02 |
| QA14 | A kv_cursor.tide:67 | May `&mut fmt::Formatter` omit its origin? | Yes (arm-A §3.1, 02 §2.5). | decided by the reference |
| QB1 | B group_words.tide:23 | May `-> Option<(char, &'m [&'t str])>` rely on the relation `'t: 'm` that the argument `&'m BTreeMap<char, Vec<&'t str>>` implies? | Yes. Nothing is written, and rustc accepts it. 02 §2.1 and §2.7 let bodies rely on relations implied through well-formedness, and 02 §2.6 rejects only bounds "beyond what well-formedness implies". Arm A's equivalent signature needs the same relation. Arm-B §3.2 bans written bounds and says nothing about implied ones. | reference gap (RB1), no error |
| QB2 | B ini.tide:66, table.tide:51 | Does `impl Iterator<Item = &'a str> + '_` capture `'a`? | Yes. Arm-B §3.5 adopts Rust 2018's rule, under which lifetimes that appear in the bounds are captured. §3.5's own sentence names only elided positions and `+ 'a` bounds. | reference wording gap (RB3), no error |
| QB3 | B filters.tide:24 | Is `filter: impl Fn(&str) -> bool + 'a` an excluded outlives bound? | No. Arm-B §3 lists "`+ 'a` on `dyn` and `impl` types" as part of Tide, and FX0201's suggested rewrite is exactly that form. | decided by the reference |
| QB4 | B filters.tide:6, 11 | Closure parameters annotated `|word: &str|` "where Rust may need it" | Correct with or without the annotation, since the expected `impl Fn(&str)` bound supplies the signature. | decided (Rust) |
| QB5 | B notes | §13's `from` mention | The protocol deviation. Ignoring it was correct. | protocol deviation |

### 3.1 ini

**Arm A: severity 0.** Passes check and tests.

- **Origins.**
  - `Section { name: &str, entries: Vec<(&str, &str) from name> }` has one
    origin. The field `from` follows 02 §3.3 and covers both tuple elements.
  - `Ini { sections: Vec<Section> }` therefore has one origin (QA7).
  - `get`, `entries`, and `section_names` return `from *self`, the
    `Ini`'s origin, so results borrow `src` and not the `&self` call.
  - `entries`' outer `&` elides to `self` by rule 2. `section_names` adds a
    capture `from self` (QA4).
  - `section_index(sections: &mut Vec<Section from name>, name: &str)` is
    the table's `add` form. It identifies the element origin with `name`'s,
    the exact form an invariant `&mut Vec` needs. Rust:
    `fn section_index<'n>(&mut Vec<Section<'n>>, &'n str)`.
  - `parse` returns `Ini` by rule 1.
- **Borrow-sensitive test.** In `a_value_outlives_the_ini` (148–156), `get`
  yields `&str` in the `Ini`'s origin, which holds only loans of `src`. The
  `&ini` autoref loan ends at the call. Dropping `ini` at the block's end
  conflicts with no live loan, and `Ini` has no destructor, so drop check
  adds nothing. `user == "admin"`.
- **Other tests.** Sample: names `["", "server", "client"]`; `top`→`level`,
  `host`→`example.org` (the trailing space is trimmed), `port`→`8080`,
  `retries`→`3`, unknown section → `None` and `&[]`. Errors: line 3
  unclosed, line 3 missing `=`, line 2 empty key. Display: `"line 2: empty
  key"`. Repeats: `get` returns the last value, `entries` returns all, and
  the names are `["a", "b"]`.
- **Borrow detail.** `get_or_insert_with(|| section_index(&mut sections,
  ""))` borrows `sections` only for the call.
- **N1 (nit, other).** The implicit section's name is the literal `""`
  (line 54), not a slice of `src` as the task's "stores section names ... as
  slices of `src`" asks. The writer disclosed it. It is unobservable without
  the unsafe grant, and arm B's `&src[..0]` meets the wording.

**Arm B: severity 0.** Passes check and tests.

- **Lifetimes.** `Section<'a>` and `Ini<'a>` share one `'a`. `get`, `entries`,
  and `section_names` return `'a` data, and `section_index(&mut self, name:
  &'a str)` ties a new name to the element lifetime. `section_names` uses
  `impl Iterator<Item = &'a str> + '_` (QB2), and `parse` uses
  `Result<Ini<'_>, IniError>`.
- **Borrow-sensitive test.** `value_outlives_the_ini` (128–136) traces as in
  arm A: `get` returns `Option<&'a str>`, and `'a` is the `src` borrow.
- **Other tests.** In the `SAMPLE` const, the `\` continuation keeps the
  lines at column 0. `name`→`top level`, `empty =`→`""`.
  `entries("server")` against an array uses `&[A] == [B; N]`, which core
  provides. Error lines 1, 3, 2. Repeats: `get` returns `"3"`.

### 3.2 path_norm

**Arm A: severity 0.** Passes.

- **Origins.** `Component` has one origin (`Normal.0`).
  `components(path: &str) -> impl Iterator<Item = Component>` borrows `path`
  through the elided origin in its bound, as reference §3.5's `adults` does.
  The private `names` helper uses the same form. `normalize -> Vec<&str>`
  uses rule 1. This is one of A's two programs with no annotation.
- **Borrow-sensitive test.** In `components_outlive_the_iterator_that_made_them`
  (96–114), the collected `Component`s carry `path`'s loan and not the
  iterator's. The iterator is consumed by `collect` inside the block.
  Result: `Normal(usr)`, `CurDir`, `Normal(lib)`, `ParentDir`,
  `Normal(share)`.
- **Other tests.** `/a//b/./../c/` yields `Root a b CurDir ParentDir c`.
  `normalize` gives `[a, c]`; `/../a`→`[a]`; `/..` and `/a/../..`→`[]`;
  `../../x` keeps both `..`; `a/../../b`→`[.., b]`.

**Arm B: severity 0.** Passes.

- **Lifetimes.** `Component<'a>` with
  `impl Iterator<Item = Component<'_>>`. The `normalize` logic matches A's.
- **Borrow-sensitive test.** `collected_components_outlive_their_iterator`
  (91–109) passes.
- **Property test.** `normalized_names_are_clean` (111–125) holds: a `..`
  is pushed only onto an empty result or one ending in `..`, so every `..`
  sits in a leading run, and an absolute path pushes none.
- **Edge cases.** `""` gives no components; `/` gives `[Root]`.

### 3.3 diff_lines

**Arm A: severity 0.** Passes.

- **Origins.**
  - `Option<(usize, &str from a, &str from b)>`: the comma ends each `from`
    (QA9). Rust: `Option<(usize, &'a str, &'b str)>`.
  - `only_in_first -> Vec<&str from a>` and `longest_common_line ->
    Option<&str from a>`. Both are required: with two inputs, elision has
    nothing to bind.
- **Borrow-sensitive tests.** In `only_in_first_outlives_b` and
  `longest_common_line_outlives_b`, the results carry only `a`'s loan. The
  local `HashSet<&str>` over `b` dies inside the call. Results:
  `["keep", "also keep"]` and `"a much longer line"`.
- **Other tests.** `"x\ny\n"` against `"x\ny"`: `None`. Line 2 `y`/`Y`;
  `(2, "y", "")`; `(1, "", "new")`. The longest common line on a tie keeps
  `"ab"`, the first in `a`.

**Arm B: severity 0.** Passes.

- **Lifetimes.** A `Difference<'a, 'b>` struct with independent lifetimes.
  `only_in_first<'a>` and `longest_common_line<'a>` leave `b`'s lifetime
  free.
- **Tests.** Traced: `("a", "a\n\n")` gives `{2, "", ""}`. The property
  test (104–107) holds trivially. Both drop-`b` tests pass.

### 3.4 merge_words

**Arm A: severity 0.** Passes, given C3.

- **Origins.** `merge_sorted(a: &[&str], b: &[&str]) -> Vec<&str from (*a,
  *b)>` is a union at a covariant position (02 §2.4 item 1). Rust: a fresh
  `'r` with `'x: 'r` and `'y: 'r`. The body pushes and `extend_from_slice`s
  both inputs' elements into it.
- **Borrow-sensitive test.** In `merged_words_outlive_the_intermediate_vecs`
  (58–68), the result carries the element loans on `first` and `second`. The
  `&a` and `&b` loans end at the call, so dropping `a` and `b` is fine.
  Merged: `apple apple banana fig kiwi pear`, with `a`'s `apple` first, since
  `b[j] < a[i]` is strict.
- **Contracts.** The `requires` use the recognized `is_sorted` shape. The
  `ensures` hold, and `dedup_adjacent`'s guard `1 <= i && i < words.len()`
  is a recognized bounded form. The property test sorts first (10.2), and
  `String` order equals `str` order. `merge_then_dedup` gives `[a, b, c,
  d]`.
- **Contingent.** C3.

**Arm B: severity 0.** Passes, given C3.

- **Lifetimes.** One `'a` for both inputs, which is covariant and so
  equivalent to A's union. The loop `invariant merged.len() == i + j` holds
  at entry, after each iteration, and at exit.
- **Borrow-sensitive test.** In `merged_result_outlives_both_vectors`
  (56–70), `drop(a); drop(b)` while `merged` lives. No loan on `a` or `b`
  remains, because the slice borrows are not in `'a`. Merged: three `apple`s,
  then `banana cherry fig pear`. Dedup gives five words.
- **Other tests.** `merge_sorted(&[], &["x", "y"])` coerces `&[_; 0]`. The
  property test is identical to A's.
- **Contingent.** C3.

### 3.5 highlight

**Arm A: severity 0.** Passes.

- **Origins.**
  - `Highlighter` has one origin (`keywords: Vec<&str>`), and `Span` has
    two (`word`, `keyword`).
  - `new(keywords: &[&str], ..) -> Highlighter from *keywords` is required,
    because the input has two elided origins. It names the elements, not
    the caller's slice. `keywords.to_vec()` copies references only.
  - `spans(&self, text) -> Vec<Span from { word: text, keyword:
    self.keywords }>` binds each origin separately (QA1, QA10). Rust:
    `Vec<Span<'t, 'h>>`.
  - `keyword_for -> Option<&str from self.keywords>`.
- **Borrow-sensitive tests.**
  - `texts_come_and_go_while_keywords_stay` (66–86): each `text` dies at
    the end of its iteration after its `spans`. `matched_keywords`, declared
    outside the loop, collects `s.keyword`, which carries only `config`'s
    loan. Words `[let, if, match, fn]`; keywords `[let, IF, match, fn]`.
    `fn_` and `x_fn` are not `fn`.
  - `spans_outlive_the_highlighter` (88–104): the result carries `text` and
    `config` loans, not the highlighter, which has no destructor. `beta`@0,
    `alpha`@13.
- **Other tests.** Case-sensitive `Foo` matches only @4; relaxed matches
  @0, 4, 8; `Foo_bar` is one word.
- **Borrow detail.** `Highlighter::new(&["Foo"], ..)` borrows only the
  array's `'static` elements.

**Arm B: severity 0.** Passes.

- **Lifetimes.** `Span<'t, 'k>`, `Highlighter<'k>`, and `new(keywords:
  &[&'k str], ..) -> Self`, the arm-B reference's own constructor form.
  `spans<'t>(&self, text: &'t str) -> Vec<Span<'t, 'k>>`.
- **Borrow-sensitive tests.**
  - In the loop test (66–82), every iteration's text gives words `[let,
    LET, match]`, keywords `[let, let, Match]`, offsets `[0, 20, 28]`.
  - `spans_outlive_the_highlighter` (84–94) also builds the keyword `Vec`
    inside the block. That passes, because `new` ties the `Highlighter` to
    the elements' `'k`, not to the `Vec`. This test is stronger than the
    task asks. Result: `alpha`@25.
- **Other tests.** In `"é IF STRASSE Straße"`, `IF`@3 matches `if`,
  `STRASSE` does not match `straße` (byte-wise ASCII folding), and
  `Straße`@14 matches.

### 3.6 filters

**Arm A: severity 0.** Passes.

- **Origins.**
  - `not_banned(banned: &[String]) -> impl Fn(&str) -> bool from banned` is
    reference §3.5's `over` form. The `&str` in the bound is higher-ranked
    (QA13).
  - `min_len -> impl Fn(&str) -> bool` is `Owned`.
  - `Pipeline { filters: Vec<Box<dyn Fn(&str) -> bool + '_>> }` has one
    origin, named `filters` (02 §5.1).
  - `add(&mut self, filter: impl Fn(&str) -> bool from self.filters)` is
    02 §2.4 item 3's `F: 'p` (QA2, QA3).
  - `new() -> Self` (QA11).
  - `apply(&self, words: &[&str]) -> Vec<&str from *words>` borrows the
    words, not the pipeline.
- **Borrow-sensitive tests.**
  - `pipeline_result_outlives_the_pipeline` (57–69): `apply`'s result
    carries only `text`'s loans. Dropping `pipeline` at the block end needs
    `filters`' origin live, since a `dyn` object is in `dropck` (02 §6). It
    is live, because the captured loan is on `banned`, which outlives the
    block. Kept: `[quick, brown, jumps, over]`.
  - `pipeline_accepts_ad_hoc_borrowing_closures` (71–80): a closure
    borrowing `prefix` is added. `prefix` is declared before `pipeline`, so
    it is dropped after it. Results: all four words with no filters, then
    `[bat, bee]`.
- **Other tests.** `äöü` is three characters, so `min_len(4)` rejects it.

**Arm B: severity 0.** Passes.

- **Lifetimes.** `not_banned -> impl Fn(&str) -> bool + '_`.
  `Pipeline<'a>` with `Box<dyn Fn(&str) -> bool + 'a>`, `add(&mut self,
  filter: impl Fn(&str) -> bool + 'a)` (QB3), and `apply<'w>(&self, words:
  &[&'w str]) -> Vec<&'w str>`.
- **Tests.** The drop test (38–51) traces as A's; it also asserts
  `banned.len()` afterwards. `custom_filter_borrowing_a_local` (64–70)
  borrows `stop` (`&str != String`), giving `[up, down]`.
  `empty_pipeline_keeps_everything` passes.

### 3.7 group_words

**Arm A: severity 0.** Passes.

- **Origins.**
  - `group_by_initial -> BTreeMap<char, Vec<&str>>` uses rule 1.
  - `largest_group(groups: &BTreeMap<char, Vec<&str>>) -> Option<(char,
    &[&str from *groups])> from groups` is the composed form (QA5). It is
    required, because the input has two elided origins.
  - `words_longer_than -> Vec<&str from *groups>` borrows the text, not the
    map.
- **Borrow-sensitive test.** In `long_words_outlive_the_map` (82–91), the
  result carries the element origin, which holds loans on `text`. The
  `&groups` loan ends at the call, and the map (an audited container) drops
  at the block end. Result: `[avocado, apricot, banana, Blueberry, cherry]`.
  With `n = 9`, nothing qualifies (`Blueberry` has exactly 9 characters).
- **Other tests.** `...` trims to nothing and is skipped. `É`→`é`. Ties go
  to `a` in `"bob alice"`. The empty map gives `None`.

**Arm B: severity 0.** Passes.

- **Lifetimes.** `largest_group<'m, 't>(groups: &'m BTreeMap<char, Vec<&'t
  str>>) -> Option<(char, &'m [&'t str])>` relies on the implied relation
  `'t: 'm` (QB1). `words_longer_than<'t>` leaves the map's lifetime free.
- **Tests.** `TRIMMED: &[char]` is a valid pattern. Groups: `a` holds
  `Apple avocado apricot`, `b` holds `banana b`, `c` holds `Cherry`. The
  drop test (71–80) gives `[avocado, apricot, banana, Cherry]`.

### 3.8 table

**Arm A: severity 0.** Passes.

- **Origins.**
  - `Table { header: Vec<&str>, rows: Vec<Vec<&str from header>> }` has one
    origin, by the field-sharing rule with an owned-collection sibling
    (QA1).
  - `parse(text) -> Result<Table, TableError>` uses rule 1 and correctly
    writes `-> Table`, not `-> Self`.
  - `column -> Option<impl Iterator<Item = &str from self.rows> from self>`
    (QA4).
  - `into_column(self, ..) -> Option<Vec<&str from self.rows>>` (QA12).
  - `rows_where -> Vec<&[&str from self.rows]>`: the outer `&` elides to
    `self`.
- **Borrow-sensitive test.** In `into_column_outlives_the_table` (93–104),
  `table` is moved into `into_column`. `rows` is moved out of it, which is
  legal because `Table` has no `Drop`. The result carries `text`'s loans.
  Names: `[ferox, ripgrep, zola]`; an unknown column gives `None`.
- **Other tests.** Field-count errors: line 3 (expected 2, found 1) and line
  2 (found 3). `""` gives `MissingHeader`. A trailing newline is accepted.
  `rows_where` returns the `ripgrep` and `zola` rows; stars above 14 give 2.

**Arm B: severity 0.** Passes.

- **Lifetimes.** `Table<'a>` with `parse(text: &'a str) -> Result<Self, _>`,
  `column -> Option<impl Iterator<Item = &'a str> + '_>`, `into_column ->
  Option<Vec<&'a str>>`, and `rows_where -> Vec<&[&'a str]>`.
- **Borrow-sensitive test.** The drop test (102–111) gives `[2015, 2026,
  1995]`.
- **Other tests.** `old == [["duke", "java", "1995"]]` compares
  `Vec<&[&str]>` with an array of arrays through core's `&[B] == [A; N]`.
  `"k\tv\na\t1\n\n"` fails at line 3, as the notes say.

### 3.9 logscan

**Arm A: severity 0.** Passes, given C1 and C2.

- **Origins.** None needed. `Entry { message: &str }` and `Scan { entries:
  Vec<Entry> }` are borrowing types. `parse_log(text) -> Scan` and
  `parse_line(line) -> Option<Entry>` use rule 1.
- **Capabilities.**
  - `#[pure] parse_log` makes "needs no capabilities" a check (03 §3.2,
    reference §6.6).
  - `main` attenuates (`args_only`, `read_only`) and calls `run(&Env, &Fs)
    -> u8`. Reference §6.2 asks for this shape, because tests cannot call
    `main`.
- **Borrow-sensitive test.** `messages_outlive_the_scan` (204–212), beyond
  the task, passes.
- **Parsing test.** Five malformed lines: `x`, `NOTICE`, `-1`, `+3`, and the
  empty line, whose empty stamp passes the digit check but fails `parse`.
  Three entries remain, including `10 DEBUG` with message `""`.
- **Report test.** `ERROR 2, WARN 4, INFO 1, DEBUG 1, malformed 2`, then
  entries 125 and 110.
- **Simulator test.** At `--level WARN`, the output is 130, 120, 105.
  `stderr` is empty.
- **Argument tests.** Each bad argument list gives `None`. `LOUD` exits 2
  with exactly `[USAGE]` on stderr, and a missing file exits 1 with the
  prefix `logscan: cannot read /logs/missing.log`.
- **Contingent.** C1 (`split_sinks`, 174–185), C2 (249, 272, 282).

**Arm B: severity 0.** Passes, given C1 and C2.

- **Lifetimes.** `Entry<'a>`, `Scan<'a>` (with `Default`), `Scan<'_>`, and
  `Entry<'_>`.
- **Parsing test.** `split_once` twice. Five malformed lines: `not a line`,
  `-3`, `TRACE`, the empty line, and the 23-digit stamp, which overflows
  `u64`. `"slow  reply"` keeps its inner double space.
- **Simulator test.** `ERROR 4, WARN 1, INFO 1, DEBUG 0, malformed 1`, then
  `9 d`, `7 e`, `5 b`.
- **Exit codes.** Exits 2 and 1 as specified.
- **Contingent.** C1 (197–202, 211, 219–222), C2 (195, 209, 217).

### 3.10 kv_cursor

**Arm A: severity 0.** Passes, given C3.

- **Origins.**
  - `Cursor { src: &str, pos: usize }`, `new(src) -> Cursor`.
  - `eat_while(&mut self, ..) -> &str from self.src` copies `self.src` out
    before slicing (reference §3.2), so eaten text outlives the `&mut self`
    call.
  - `parse_pairs(src: &str, out: &mut Vec<(&str, &str) from src>)` is the
    table's `add` form. It identifies the invariant element origin with
    `src`'s and leaves the outer `&mut` borrow separate. That avoids Rust's
    `&'a mut Vec<&'a str>` trap. `append_pairs` has the same form.
- **Borrow-sensitive tests.**
  - `appends_to_a_vector_holding_a_pair_from_another_string` (146–163):
    each call identifies `pairs`' one element origin with that call's `src`
    origin. The two string loans coerce into it, since both strings outlive
    every use of `pairs`. Result: `lang=tide`, `name="kv cursor"`,
    `level=3`, `empty=`, `last=x y`. That is 1 pair, then 4 (traced byte by
    byte).
  - `eaten_text_outlives_its_cursor` (191–203): `"héllo"` outlives the
    cursor.
- **Error tests.** `EmptyKey`@5, `MissingEquals`@9, `UnterminatedQuote`@2
  twice (end of input, and a newline inside the quote). `pairs` is truncated
  back to `[("kept", "yes")]` each time. Display: `"empty key at byte 5"`.
- **Contract.** The `ensures` matches on `result`, which is `Copy`, so that
  is allowed (reference §10.1). It holds on every path.
- **N3 (nit, other).** After a closing quote, A accepts the next key without
  a `;` or newline (`a="x" b=2` is two pairs). That is looser than "pairs
  separated by `;` or newlines". Disclosed.
- **Contingent.** C3 (`parse_pairs`' generated test).

**Arm B: severity 0.** Passes.

- **Lifetimes.** `Cursor<'a>`, `new(src: &'a str) -> Self`, `eat_while ->
  &'a str`, and `parse_pairs<'a>(src: &'a str, out: &mut Vec<(&'a str, &'a
  str)>)`, which keeps the `&mut` lifetime elided and separate. The
  scrutinee temporary `&mut Cursor::new(src)` lives to the end of the
  `match` without conflicting with `out`.
- **Borrow-sensitive tests.**
  - `appends_to_pairs_from_another_string` (139–150): 1 pair, then 4. The
    quoted value `"a; b"` keeps its `;`, and `last=1\r` is trimmed.
  - `eaten_text_outlives_the_cursor` (168–176) passes.
- **Error tests.** Offsets 5, 8, 3, 4, and 6 for the five kinds, with
  `pairs` untouched.
- **N2 (nit, other).** A fourth error kind, `ExpectedSeparator` (68,
  123–126), beyond the three the task lists, for text after a closing quote,
  which the task leaves undefined. Disclosed in the notes. Arm A takes the
  opposite liberty (N3).

## 4. Comparison

**Borrow-related definite errors: A 0, B 0** (0 of 10 programs in each
arm). Other definite errors: 0 and 0. First-check pass rate 10/10 in both,
and 1.0 runs per program in both.

**Sample size.** One writer per arm, ten programs each, and a model as the
grader rather than a compiler.

- A 0/10 result bounds each arm's per-program borrow-error rate below about
  26% (one-sided 95%).
- The trial cannot rank the spellings. With ten programs per arm, Fisher's
  exact test needs a gap of at least five programs to separate the arms at
  p < 0.05: 5 against 0 gives p = 0.03, while 4 against 0 gives 0.09 and 6
  against 1 gives 0.06.
- Writer and arm are confounded.

So the error rates, pass rates, run counts, and the question counts below
are all within noise.

**The tasks did test borrowing.** Neither arm passed because the tasks were
easy:

- In both arms, every program but two needed explicit origins in at least
  one signature (22 signatures in A, 23 in B).
- Each arm has 11–12 drop-the-container tests, and all of them pass.
- The tasks covered a `&mut Vec` shared across sources, two independent
  origins in one value, closures stored in a struct, and a container
  consumed by value.

The zero comes from a strong writer, not from easy tasks. Notably, D2's
predicted "`'a` error class" did not appear in arm B.

**What differed.**

1. **Annotation burden. This difference is structural, not noise.**
   - A wrote 28 `from` clauses. B wrote 99 named lifetime tokens plus 11
     `'_`, about 3.2 to 3.5 times as many (§2.4).
   - Both needed about the same number of annotated signatures. The gap is
     B's 13 declarations on 11 types and 5 impl headers, and B repeating
     each name at every use.
   - Two arm-A programs needed no annotation at all; no arm-B program did.
   - Any writer of these programs in arm B pays the declaration cost.
     Whether that cost produces errors is not measured here.
2. **Writer questions. These come from reference coverage, not from the
   difficulty of the spelling.**
   - A's notes raise 14 origin questions (9 explicit), and the writer rated
     five borrow-heavy programs "medium". B's notes raise 5 (3 explicit).
   - Six of A's questions are forms that 02 defines but arm-A's reference
     never shows: an owned-collection field as a place, `from` on an
     argument `impl`, `self.field` in a parameter, and an inner and an
     outer `from` on one return type. Writer A derived each correctly from
     the reference's general rules.
   - B's questions sit where arm-B's reference describes Tide by
     subtracting from Rust: whether an implied outlives relation counts as
     a banned bound, whether `+ 'a` on an argument `impl` is allowed, and
     what a returned `impl Trait` captures. Each is one sentence the
     reference could state.
   - Neither arm hit a genuine design gap in its borrow semantics. The
     design-text gaps found (§5.4, D-1 and D-2) changed no verdict.

**Bottom line.** Both arms are equal on correctness. On the sample, `from`
needs about a third of the annotation, while the reference covers Rust's
spelling better. The next round should do three things:

- fix RA1–RA5 and RB1–RB3 first, so that coverage stops being a variable;
- use several writers per arm, or a weaker model, so that errors occur at
  all;
- record diagnostics-driven repair. Nothing failed here, so 02 §12 item 9
  (whether agents find the right `from` from diagnostics that name `*out`
  or `lx.src`) is still untested.

## 5. Reference and design gaps

### 5.1 Arm A reference (`design/reference.md` §3)

- **RA1. §3.3, owned-collection fields.** Add: "`words: Vec<&str>` has
  origin `words`; write `from self.words` (or `from *self` when the type has
  one origin)." Source: 02 §3.2 and §2.2. Hit by table, highlight, and
  filters (QA1).
- **RA2. §3.5 and §3.3, parameter-side `from`.** Add the
  stored-closure example:
  `fn add(&mut self, f: impl Fn(&str) -> bool from self.filters)` on a
  struct whose field is `Vec<Box<dyn Fn(&str) -> bool + '_>>`. State that a
  parameter may name `self.field`. Source: 02 §2.4 item 3, §10.5. Hit by
  filters (QA2, QA3).
- **RA3. §3.2, composition.** Add one rule and two examples. The rule: "An
  inner `from` fixes its positions; a trailing `from` fills the elided ones
  left." The examples: `-> Option<(char, &[&str from *groups])> from
  groups` and `-> impl Iterator<Item = &str from *self> from self`. Source:
  02 §2.3. Hit by ini, table, and group_words (QA4, QA5).
- **RA4. §3.3, constructors.** Add: "A constructor with no borrowed input
  returns `-> Self`; `-> Pipeline` has nothing to elide from" (QA11).
- **RA5. §3.2, precedence.** State that `from` after an `impl` or `dyn`
  type, including one ending in `Fn(..) -> R`, applies to the whole `impl`
  or `dyn` type. The `over` example already assumes this. Pairs with D-2.
- **RA6. §3.5, captures.** Change "what the elided positions in its bounds
  name plus its `from` clause" to "every origin its bounds mention, elided
  or named, plus its `from` clause". Pairs with D-1.

### 5.2 Arm B reference (trial copy only)

- **RB1. §3.2, implied relations.** Add: "Relations that well-formed
  argument types imply are allowed: `&'m BTreeMap<_, Vec<&'t str>>` lets a
  result be `&'m [&'t str]`. Only written outlives bounds are `FX0201`."
  Source: 02 §2.1, §2.6, §2.7. Hit by group_words (QB1).
- **RB2. §3.2 and §3.4, argument-position `+ 'a`.** Add: "`impl Trait +
  'a` in argument position is allowed; it replaces `T: 'a`" (QB3).
- **RB3. §3.5, captures.** Change "the elided positions in its bounds ...
  plus its `+ 'a` bounds" to "every lifetime its bounds name, elided or
  written, plus its `+ 'a` bounds", which is Rust 2018's rule as the section
  already says. Hit by ini and table (QB2).
- **RB4. §13.** Delete "rewrites legacy lifetime spellings to `from`" from
  the `tide fmt` row before any rerun. This is the protocol deviation.

### 5.3 Both references

- **RS1. §6.8, simulator output and setup.** State that each printed line
  is one `Record`, give `Record`'s fields and `Sink`'s variants, and give
  `sim.world.args`' parameter and return types. Name `sim.env`. Both
  writers guessed identically (C1, C2), and both avoided two `SimRoot`s in
  one test because nothing says which one receives the body's output. Say
  that too.
- **RS2. §10.2, generated tests over borrowed parameters.** Say how
  `&[&str]`, `&mut Vec<&str>`, and `&mut Vec<(&str, &str) from src>` are
  generated, or list them among the kinds that are skipped as unchecked
  (C3; D-3).
- **RS3. §9 (low priority).** Both writers asked whether anything but
  `Result` is must-use. Say whether Rust std's `#[must_use]` methods (for
  example `str::trim`) are errors, lints, or ignored.
- **RS4. §6.2 (low priority).** Say whether `ExitCode` is `PartialEq`.
  Both writers returned `u8` from `run` to avoid comparing `ExitCode`s.

### 5.4 Normative chapters

- **D-1. 02 §5.2 and §11.1, what a returned `impl Trait` captures.** "Exactly
  those its `from` clause names plus the elided positions inside its bounds"
  omits origins a bound names through `from`, as in
  `impl Iterator<Item = &str from *self>`. Make the capture set every
  origin the bounds mention, and make §11.1's `use<'p…>` list them. No
  program depended on the strict reading, because arm A's two uses also
  carry a covering trailing `from self`.
- **D-2. 02 §2.3, `from` precedence.** The grammar says only that `from`
  binds "more loosely than `&`, `&mut` and `dyn`". State the same for
  `impl`, and that inside `impl Fn(A) -> R from P` or `dyn Fn(A) -> R from
  P` the `from` attaches to the `impl` or `dyn` type, not to `R`. The
  reference's `over` example and arm A's filters rely on it.
- **D-3. 04 §2.4 rule 1, generators for types that hold borrows.** "Structural
  types derive a generator" does not say where the referents of a
  `Vec<&str>` element come from, or how an element origin identified with
  another parameter (`from src`) is honored (C3).
- **D-4. 02 §3.2, names of nested origins.** The names table covers
  `lx: Lexer` (`lx.src`) but not a borrowing type inside a container
  (`sections: Vec<Section>`). That name is needed for per-origin binding and
  diagnostics. Arm A sidestepped it with `from *self`.
- **D-5. std, testing "is a slice of" (low priority).** Both writers noted
  that the tasks' "a slice of `text`" contract, and merge stability, cannot
  be tested without the unsafe grant, because comparing addresses is a trust
  hole (reference §11). A safe query returning a subslice's offset, like
  Rust's unstable `str::substr_range`, would make the contract testable.
