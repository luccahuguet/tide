# Arm B notes

## Cross-cutting

- I wrote Rust 2024 with Tide's changes: named lifetimes where elision is not
  enough, `+ '_` on returned `impl Trait` that borrows, `Box<dyn Fn .. + 'a>`
  for borrowing closures, `#[derive(Error)]` instead of thiserror, no I/O
  outside capabilities, and `Root::sim` in the one binary's tests.
- **"Slice of X" cannot be asserted at run time.** Comparing raw pointers
  (`as_ptr` ranges, `ptr::eq`) needs the `unsafe` grant (section 11). So
  "points into `text`" is enforced only by signatures (the result's lifetime
  is the input's) and by the drop tests the tasks ask for. No test checks
  pointer identity. For the same reason, merge stability on equal words
  (task 4) cannot be observed in a test.
- **Where I avoided inference questions.** I annotated closure parameters
  where Rust may need the annotation (`|c: char|` for a `str` pattern,
  `|word: &str|` in returned `impl Fn(&str)`), and used `|&x|` patterns
  instead of relying on deref coercion at closure call sites.
- **Implied outlives bounds.** Section 3.2 bans explicit outlives bounds. I
  assumed the bounds Rust *implies* from argument types still hold (for
  example `&'m [&'t str]` in a return type, where `'t: 'm` follows from the
  argument `&'m BTreeMap<_, Vec<&'t str>>`). The reference never says either
  way. Only group_words depends on this.
- **Returned `impl Trait` with a named lifetime in `Item`.** Section 3.5 says
  a returned `impl Trait` "borrows exactly what the elided positions in its
  bounds name plus its `+ 'a` bounds". For
  `impl Iterator<Item = &'a str> + '_` I read "Rust 2018's rule" as also
  capturing the named `'a` that appears in the bounds (ini, table).
- **Must-use.** Only `Result` is must-use (section 9). I discard `Option`
  results (`names.pop()`, `cur.bump()`) and a `&str` (`eat_while` in
  `skip_ws`) without `let _ =`. If Tide extends must-use, these lines fail.
- **Unexplained mention.** Section 13 says `tide fmt` "rewrites legacy
  lifetime spellings to `from`", but `from` appears nowhere else. I used
  `'a` syntax throughout, since every other section teaches it.
- **Single-file roots.** Each file is checked as its own crate root (12.1),
  so libraries have no `main` and no `println!` (lint `FX0508`).
- **`use super::*` and parent imports.** I assumed the glob also brings in the
  parent's private `use` items (Rust 2018 behavior), for example
  `BTreeMap` in group_words' tests.

## 1. ini (`src/lib.tide`)

- Unclear: whether a header line must end in `]`. I chose: after trimming,
  the line must end in `]`, else `UnclosedHeader`. So `[a] ; note` is also
  "unclosed".
- Silent: repeated section headers. A repeated header continues the earlier
  section; names are listed once, in order of first appearance.
- Silent: duplicate keys. `get` returns the last value; `entries` returns all
  of them in order.
- Silent: whether `""` appears in `section_names` when there are no
  header-less entries. It appears only if such an entry exists. Its name is
  `&src[..0]`, so it is still a slice of `src`.
- Silent: `entries` for an unknown section. It returns an empty slice, not
  `Option`.
- `#` and `;` count as comments only at the start of a line, not inline.
- Check risks: `None => &[]` coercing in a match arm with a slice return
  type (Rust accepts this). `impl Iterator<Item = &'a str> + '_` (see
  cross-cutting).
- Confidence: **high**.

## 2. path_norm (`src/lib.tide`)

- Unclear: whether `.` in the middle of a path is a `CurDir` component (Rust's
  `Path::components` drops it). The task calls the API lexical and the test
  path contains `/./`, so I yield `CurDir` everywhere.
- Conflict: `normalize` must return slices of `path`, but `ParentDir` carries
  no slice. So `normalize` splits `path` itself rather than using
  `components`. This duplicates the splitting rule (on `/`, skipping empty
  pieces). Using `components` would have forced a `".."` literal.
- `normalize` drops the root: `"/a"` and `"a"` both give `["a"]`. The task
  asks only for names.
- The property test builds paths from generated `Vec<u8>` picks over
  `a . .. ""`, because random strings rarely contain `/`.
- Confidence: **high**.

## 3. diff_lines (`src/lib.tide`)

- Silent: what "identical lines" means for a trailing newline. I compare
  `str::lines()`, so `"a\n"` and `"a"` are identical, and `\r\n` equals `\n`.
- Silent: a missing line versus an empty line. `"a"` against `"a\n\n"`
  differs at line 2 with both sides `""`, as the spec's "a missing line is
  `""`" implies. The `""` for a missing line is a `&'static str` literal, as
  the task allows.
- Silent: what the result type looks like. I used a `Difference<'a, 'b>` struct
  with two independent lifetimes, not a tuple.
- Silent: what "longest" measures. I count characters (`chars().count()`),
  not bytes.
- `only_in_first` keeps repeated lines of `a`.
- Confidence: **high**.

## 4. merge_words (`src/lib.tide`)

- I put contracts here because the property is a contract:
  `requires a.is_sorted()`, `ensures result.is_sorted()`, the length
  `ensures`, a loop `invariant`, and a quantified `ensures` on
  `dedup_adjacent`.
- Unclear: whether the `forall` guard `1 <= i && i < words.len()` counts as a
  bounded binder. Section 10.1 gives the shape `lo <= i && i < hi`, so I
  think it does.
- Unclear: whether a generated contract test can produce `&[&str]` or
  `&mut Vec<&str>` arguments. If it cannot, section 10.2 says the function is
  skipped and reported as unchecked, which is not a failure.
- Unclear: whether `old(words.len())` on a `&mut` parameter is allowed. It is
  a `Copy` snapshot, so it should be.
- Sort order is `str`'s byte order. Stability is implemented (`b[j] < a[i]`
  takes from `a` on ties) but cannot be observed in a test (see
  cross-cutting).
- The drop test uses explicit `drop(a); drop(b);` while the texts stay alive.
- Confidence: **medium**. The code is plain; the risk is the contract and
  invariant syntax, and how generated contract tests treat these parameters.

## 5. highlight (`src/lib.tide`)

- `Span<'t, 'k>` has two lifetimes, and `spans<'t>(&self, text: &'t str)`
  returns `Vec<Span<'t, 'k>>`, so spans borrow neither the `Highlighter` nor
  the caller's keyword `Vec`. `new` copies the keyword *references*
  (`to_vec`), not the strings.
- Unclear: whether "alphanumeric" means Unicode or ASCII. I used
  `char::is_alphanumeric` (Unicode); offsets are bytes. Case folding is ASCII
  only (`eq_ignore_ascii_case`), as the task says.
- Silent: duplicate keywords. The first matching keyword in list order wins.
- Confidence: **high**.

## 6. filters (`src/lib.tide`)

- `not_banned` returns `impl Fn(&str) -> bool + '_`, the `FX0204` form from
  section 3.5. It moves the `&[String]` into the closure; nothing is cloned.
- `Pipeline<'a>` stores `Box<dyn Fn(&str) -> bool + 'a>`, and `add` takes
  `impl Fn(&str) -> bool + 'a`. Section 3.2 bans outlives bounds but allows
  `+ 'a` on `impl` and `dyn` types; I read the argument-position `impl .. + 'a`
  as allowed.
- Silent: what "characters" means for `min_len`. I count `chars()`, not bytes.
- Silent: whether `Pipeline` needs `Default`. I did not add it.
- Confidence: **high**.

## 7. group_words (`src/lib.tide`)

- `largest_group<'m, 't>(groups: &'m BTreeMap<char, Vec<&'t str>>) -> Option<(char, &'m [&'t str])>`
  relies on the implied bound `'t: 'm` (see cross-cutting). If Tide rejects
  it, the fix is one shared lifetime: `&'a BTreeMap<char, Vec<&'a str>>`,
  which covariance still allows callers to pass.
- Silent: what "lowercased first character" means when lowercasing gives
  several chars (`'İ'`). I take the first char of `to_lowercase()`.
- Silent: whether "longer than `n`" counts bytes or characters. I count
  characters.
- Ties go to the smallest key: the scan runs in key order and replaces only on
  a strictly larger group.
- Confidence: **medium** (the two-lifetime signature; everything else is
  ordinary).

## 8. table (`src/lib.tide`)

- Silent: empty text, which has no header line. I added
  `TableError::MissingHeader` (line 1) beside
  `FieldCount { line, expected, found }`, plus `TableError::line()`. The task
  names only the field-count error.
- Silent: what "a trailing newline is allowed" means. Exactly one trailing
  newline is allowed, because `str::lines()` drops one final empty line. A
  second blank line is a 1-field row and fails the field count.
- Silent: an unknown column in `into_column`. I return `Option<Vec<&'a str>>`,
  matching `column`.
- Silent: duplicate header names. The first match wins.
- `column` returns `Option<impl Iterator<Item = &'a str> + '_>`. Cells borrow
  the text (`'a`); the iterator borrows the table.
- `rows_where` takes `impl Fn(&[&str]) -> bool` and returns
  `Vec<&[&'a str]>`.
- Confidence: **high**.

## 9. logscan (`src/main.tide`)

- Tests cannot call `main` (6.2). So the whole program is
  `run(env: &Env, fs: &Fs) -> u8`. `main` only attenuates
  (`root.env.args_only()`, `root.fs.read_only()`) and returns
  `ExitCode::from(run(&env, &fs))`. The
  simulator test calls `run` with `sim.env` and `sim.fs` after
  `sim.world.file` and `sim.world.args`, then reads stdout back from
  `sim.records()` filtered by `Sink::Out`. That is my reading of "runs the
  whole program in the simulator".
- Unclear: whether one `print!` of a multi-line string makes one record or
  several. A `Record` is "one line without its newline", so I print with one
  `println!` per line.
- Unclear: which simulator receives sink output when one test body creates
  two `SimRoot`s. I gave each simulator its own test.
- `#[pure]` on `parse_log` makes "needs no capabilities" a compile-time
  check.
- The output format is not given. I chose `ERROR: 3` lines,
  `malformed: 2`, then `<timestamp> <message>`. Timestamp ties keep file order
  (stable sort).
- Choices about malformed lines:
  - A timestamp must be ASCII digits only; Rust's `u64::from_str` would also
    accept `+5`. Overflow is malformed.
  - Fields are separated by single spaces, so `"1  INFO x"` is malformed.
  - `"130 DEBUG"` with no message is an entry with message `""`.
  - Blank lines count as malformed.
- Silent: argument details. Levels are case-sensitive. `--level` may come
  before or after the path, and the last one wins. Any other argument starting
  with `-`, or a second path, is a usage error (exit 2).
- Unclear: how `ExitCode` compares in tests. I avoided the question: `run`
  returns `u8`.
- Assumed: `Sink` variants can be matched with `matches!` (PartialEq not
  stated), `sim.world.args` takes an array of `&str`, and `Env::args_only()`
  keeps `args()` working.
- Confidence: **medium**. Several of the sim APIs are marked provisional.

## 10. kv_cursor (`src/lib.tide`)

- `skip_ws` skips spaces, tabs, and `\r`, but not `\n`, because newlines
  separate pairs. The reference does not say which whitespace `skip_ws`
  means.
- Silent: text after a closing quote (`k="v" x`). None of the three listed
  error kinds fits, so I added a fourth, `ParseErrorKind::ExpectedSeparator`.
  This adds an error kind the task did not list.
- On any error, `parse_pairs` truncates `out` back to its original length, so
  the caller's earlier pairs are untouched and no partial pairs remain. The
  task does not say what happens to `out` on error.
- Empty segments (`a=1;;b=2`, blank lines) are skipped. An empty unquoted
  value is `""`. Keys may contain inner spaces (`my key = v`).
- Error offsets:
  - `EmptyKey`: where the key would start.
  - `MissingEquals`: where `=` was expected.
  - `UnterminatedQuote`: the opening quote.
- `eat_while` copies `self.src` out first (`let src = self.src;`, section
  3.2), so its result has lifetime `'a`.
- `ParseError`'s message uses `{kind:?}`; I assumed `#[error]` accepts format
  specs as thiserror does.
- Confidence: **high** that it checks. The extra error kind is a spec
  deviation, not a check risk.
