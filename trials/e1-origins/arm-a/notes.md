# Arm A notes

Sources used: `arm-a/reference.md` and `tasks.md` only. Every program was
checked by reading it against the reference, as `tide check` and `tide test`
would. Expected test values were traced by hand.

## Cross-cutting

1. **Origin of an owned collection field.** The reference names origins for
   `src: &str` (`src`) and for elements behind a reference (`*out` for
   `out: &mut Vec<&str>`). It does not show the elements of an owned field
   such as `header: Vec<&str>`. I read `from header`, `from self.keywords`,
   and `from self.rows` as "what that field's value borrows", which is the
   general rule in 3.2 ("borrows from what the value at place `p`
   borrows"). This reading is used in table, highlight, and filters.
2. **Composed `from` forms that the reference never shows together.**
   - An inner `from` on `Item` plus an outer capture `from` on a returned
     `impl Trait`: `impl Iterator<Item = &str from *self> from self` (ini,
     table).
   - An inner `from` plus a trailing `from` on one return type:
     `Option<(char, &[&str from *groups])> from groups` (group_words).
   - `from` on an argument-position `impl Fn` (filters).

   Each one follows the documented rules, but none appears as an example.
3. **"Is a slice of the input" cannot be checked by address.** Comparing or
   printing raw pointers needs the `unsafe` grant (section 11), so no test
   checks addresses. The drop tests prove which value a result borrows, and
   equality checks prove its content. One consequence is that merge_words'
   stability (equal words from `a` come first) cannot be observed in a
   test.
4. **Elided references inside `Fn(&str) -> bool`.** I treated these as
   Rust's higher-ranked elision, not as captures. 3.5's `over` example needs
   an explicit `from min`, which supports this reading.
5. **Dead code.** The reference makes unused imports errors (`FX0913`) but
   says nothing about unused fields, functions, or variants. I left none.
6. **Must-use.** Only `Result` is stated to be must-use (section 9). I
   discard `Option` (`cur.bump();`) and `&str` (inside `skip_ws`) without
   `let _ =`.
7. **Borrowing std types without origins.** I wrote `&mut fmt::Formatter`
   with no origin, following "write `Lexer`, never `Lexer<'a>`".
8. **Contracts.** I added contracts only where they are simple
   (merge_words, kv_cursor). I don't know if generated contract tests can
   build inputs such as `&[&str]` or `&mut Vec<(&str, &str) from src>`.
   10.2 says a function that cannot be generated is skipped and reported
   as unchecked, not failed.
9. **Simulator output.** I assumed that `println!` and `eprintln!` in a test
   body that uses `Root::sim` go to that simulator's `records()`, one
   `Record` per line (6.8 marks `Record` as provisional).

## 1. ini (`src/lib.tide`): medium

- `Section { name: &str, entries: Vec<(&str, &str) from name> }` puts
  `from` after a tuple inside a generic argument. I relied on "applies to
  every elided position in the type before it" to cover both tuple
  elements.
- `get`, `entries`, and `section_names` use `from *self` ("what the `Ini`
  borrows"), from the table row `fn text(&self) -> &str from *self`. The
  `Ini`'s only origin sits two levels down (`Vec<Section>`, then
  `Section.name`). I assumed `*self` reaches it.
- `section_index(sections: &mut Vec<Section from name>, name: &str)` mirrors
  `fn add(m: &mut HashMap<&str from w, u32>, w: &str)`. It is called with
  `""` (`&'static str`), which I assumed coerces as in Rust.
- Behaviour the task leaves open:
  - A repeated `[section]` continues the earlier section.
  - For a repeated key, `get` returns the last value and `entries` returns
    all of them.
  - `[ name ]` is trimmed.
  - A header line must end in `]`, so `[a] ; note` is reported as an
    unclosed `[`.
  - Values have no inline comments.
  - `entries` of an unknown section is an empty slice, not `None`.
  - The implicit `""` section exists only if some entry comes before the
    first header. Its name is the literal `""`, not a slice of `src`.

## 2. path_norm (`src/lib.tide`): high

- `components(path: &str) -> impl Iterator<Item = Component>` relies on
  elision: `Component`'s elided origin is in the bound, so the iterator
  borrows `path`. This is the `adults` case in 3.5.
- `normalize` walks the raw names, not `Component`s. `ParentDir` carries no
  slice, and a kept `..` must be a slice of `path`. Both functions share
  one private `names` helper.
- `CurDir` is produced wherever `.` appears. Rust's `Path::components` drops
  a `.` that is not first; the task's variant list suggests keeping it.
- `normalize` returns only the names, with no absolute/relative marker.

## 3. diff_lines (`src/lib.tide`): high

- `Option<(usize, &str from a, &str from b)>` relies on "a comma ends
  `from`" inside the tuple.
- Lines come from `str::lines`, so a final newline is not an extra line.
  Comparing `"a\n\n"` with `"a\n"` reports line 2 as `("", "")`: an empty
  line against a missing one look the same.
- "Longest" is measured in characters. A tie keeps the first line of `a`.
  An empty line present in both texts counts as a common line.

## 4. merge_words (`src/lib.tide`): high

- `merge_sorted(a: &[&str], b: &[&str]) -> Vec<&str from (*a, *b)>` follows
  3.2's `from *words` and `from (a, b)` examples.
- The contracts on `merge_sorted` are: `requires` that both inputs are
  sorted, and `ensures` on length and sortedness. `dedup_adjacent` uses a
  `forall` clause with the guard written `1 <= i && i < words.len()` to
  match the recognized shape. I'm not sure whether `is_sorted` counts as a
  "std accessor" (checked) or not (sampled). Either way it runs under
  `tide test`.
- The property test takes `Vec<String>` inputs and sorts them, so it builds
  valid inputs instead of filtering for them (10.2).
- The tests cannot show stability (cross-cutting 3).

## 5. highlight (`src/lib.tide`): medium

- `new(keywords: &[&str], ..) -> Highlighter from *keywords`. Rust's elision
  fails here because the input has two origins, so the `from` is required.
  The highlighter stores `keywords.to_vec()`: it copies the references, not
  the strings. So it borrows the configuration text, not the caller's
  slice.
- `Vec<Span from { word: text, keyword: self.keywords }>` uses the
  brace-binding form inside a generic argument. I assumed the comma inside
  the braces does not end `from`.
- `from self.keywords` names the origin of an owned `Vec<&str>` field
  (cross-cutting 1). `from *self` would also work.
- If several keywords match one word, the first keyword in the list is
  reported. "Alphanumeric" is Unicode `char::is_alphanumeric`; case folding
  is ASCII only, as the task says.

## 6. filters (`src/lib.tide`): medium

- The field type `Box<dyn Fn(&str) -> bool + '_>` comes from 3.4. The
  reference does not name that origin. I referred to it through the field:
  `add(&mut self, filter: impl Fn(&str) -> bool from self.filters)`. Two
  things here are not shown in the reference:
  - `from` on an argument-position `impl Trait`;
  - naming an impl origin through `self.field` in parameter position.
    3.3 shows `Self.src` for a constructor parameter.
- `Pipeline::new() -> Self`. With no inputs, `-> Pipeline` has nothing to
  elide from, and 3.3 says `-> Self` carries the impl's origins.
- `not_banned` compares `&String == &str`. `min_len` counts characters.
  `Pipeline` has no `Default` impl.

## 7. group_words (`src/lib.tide`): medium

- In `largest_group`'s return type,
  `Option<(char, &[&str from *groups])> from groups`:
  - the inner `from *groups` covers the words, which borrow the text;
  - the trailing `from groups` covers the slice reference, which borrows
    the map.

  The return type is built from the two rules; the reference never shows
  this combination.
- `words_longer_than` returns `Vec<&str from *groups>`.
- Lowercasing uses full Unicode `to_lowercase` and keeps the first `char`
  of the result.
- Ties are broken with a strict `>` while iterating keys in ascending
  order.

## 8. table (`src/lib.tide`): medium

- `rows: Vec<Vec<&str from header>>` gives `Table` a single origin.
  Signatures name it as `from self.rows` (cross-cutting 1).
- `into_column(self, ..) -> Option<Vec<&str from self.rows>>` puts `from` on
  a field of a by-value `self`. I chose the field form over a bare
  `from self` because the field form is the documented one.
- `column` returns `Option<impl Iterator<Item = &str from self.rows> from
  self>`.
- Choices the task leaves open:
  - Empty text gives `TableError::MissingHeader`.
  - An empty body line is one empty field, so it is an error unless the
    header has one column.
  - `rows_where` takes `FnMut`.
  - An unknown column gives `None` from both `column` and `into_column`.

## 9. logscan (`src/main.tide`): medium

- Tests cannot call `main`. So `run(env: &Env, fs: &Fs) -> u8` is the whole
  program. `main` attenuates (`root.env.args_only()`,
  `root.fs.read_only()`) and converts with `ExitCode::from`. `run` returns
  a `u8` so tests compare plain integers; the reference does not say
  whether `ExitCode` is `PartialEq`.
- Simulator assumptions (mostly provisional in 6.8):
  - `sim.env.args()` returns exactly what `sim.world.args([...])` set.
  - Each `println!`/`eprintln!` line becomes one `Record` with `Sink::Out`
    or `Sink::Err` and no newline. I print line by line so nothing depends
    on how a multi-line `print!` is split.
  - `Sink` may have more variants, so the match has `_ => {}`.
  - Each error-path test uses exactly one simulator. With two `Root::sim`
    values in one test, it is unclear which one receives the body's output.
  - `sim.world.args` takes arrays of different lengths.
- `#[pure]` on `parse_log` turns "needs no capabilities" into a compile
  error if it is broken.
- Output and parsing choices:
  - Lines are printed as `ERROR: n` and so on, then `malformed: n`, then
    `<timestamp> <message>`.
  - Entries with equal timestamps keep file order.
  - The message is everything after the second single space. It may be
    empty or start with a space.
  - Timestamps must be ASCII digits (so no `+`) and fit in `u64`.
  - Blank lines count as malformed.
  - Level names are case-sensitive, both in the file and in `--level`.
  - Options may come in any order. A second path, an unknown `--flag`, a
    missing level value, or an unknown level is a usage error (exit 2).

## 10. kv_cursor (`src/lib.tide`): high

- `eat_while(&mut self, ..) -> &str from self.src` is the table's
  `next_word` form. I copy `self.src` into a local before slicing, as 3.2's
  note suggests.
- `parse_pairs(src: &str, out: &mut Vec<(&str, &str) from src>)` follows the
  `add(m: &mut HashMap<&str from w, u32>, w: &str)` row.
- Grammar choices the task leaves open:
  - Empty entries (`;;`, blank lines, a trailing `;`) are skipped.
  - `skip_ws` skips all whitespace except `\n`, because `\n` separates
    pairs.
  - Keys may contain inner spaces.
  - An unquoted value runs to `;` or a newline and is trimmed.
  - A quoted value cannot cross a newline.
  - Text after a closing quote is not required to be a separator; it starts
    the next pair. So `a="x"y` reports a missing `=`, and `a="x" b=2` parses
    as two pairs. The task names only three error kinds, so I did not add a
    fourth.
- Error offsets:
  - empty key: the `=`;
  - missing `=`: where the `=` was expected (the end of the key);
  - unterminated quote: the opening quote.
- On error, `out` is truncated back to its old length, so nothing is
  appended; an `ensures` states this. The task does not say what happens to
  partial appends.
- The kind enum is named `ParseErrorKind` to avoid confusion with
  `io::ErrorKind`. Its `Display` is written by hand, and `#[derive(Error)]`
  is on the `ParseError` struct.
