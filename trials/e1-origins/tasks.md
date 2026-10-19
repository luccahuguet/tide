# Tasks

Write each program below in Tide. Each is one `.tide` file; start it with a
comment naming the task and the path the file would have in a package
(`src/lib.tide` for a library, `src/main.tide` for a binary). Libraries need
tests but no `main`. Unless a task says otherwise, use only std, avoid
allocating copies of input text where the task says to return slices, and
include the tests the task lists plus any you find useful.

"A slice of `text`" means a `&str` that points into `text` itself, not a
copy. "Must stay usable after X is dropped" means a test drops X (for
example by creating it inside an inner block) and then uses the value, and
the program must pass `tide check` with that test.

## 1. ini

Parse INI text. `parse(src)` returns `Result<Ini, IniError>`.

- Blank lines and lines whose first non-space character is `;` or `#` are
  ignored. `[name]` starts a section; `key = value` lines belong to the
  current section, with key and value trimmed. Entries before any header
  belong to a section named `""`.
- Errors carry the 1-based line number: an unclosed `[`, a line without
  `=`, an empty key.
- `Ini` stores section names, keys, and values as slices of `src`.
- `get(&self, section, key)` returns the value as a slice of `src`;
  `section_names(&self)` returns an iterator over the names in order;
  `entries(&self, section)` returns that section's `(key, value)` pairs.
- Tests: a sample with two sections and a header-less entry; each error;
  a value from `get` must stay usable after the `Ini` is dropped.

## 2. path_norm

Lexical path handling for `/`-separated strings.

- `components(path)` returns an iterator of `Component`, an enum with
  variants `Root` (a leading `/`), `CurDir` (`.`), `ParentDir` (`..`), and
  `Normal` holding the name as a slice of `path`. Repeated slashes collapse;
  a trailing slash adds nothing. `Component` derives `Debug`, `Clone`,
  `Copy`, `PartialEq`.
- `normalize(path)` returns the normalized names as a `Vec` of slices of
  `path`: `.` is dropped; `..` removes the previous name; a leading `..` in
  a relative path is kept; `..` directly after the root is dropped.
- Tests: the components of `"/a//b/./../c/"`; `normalize` on absolute and
  relative inputs, including `"../../x"` and `"/.."`; components collected
  from an iterator that is created inside a block must stay usable after
  the block.

## 3. diff_lines

Helpers comparing two texts line by line.

- `first_difference(a, b)` returns `None` if the texts have identical lines,
  else the 1-based line number and the two differing lines: one slice of
  `a` and one slice of `b`, where a missing line is `""`.
- `only_in_first(a, b)` returns the lines of `a` that occur nowhere in `b`,
  as slices of `a`, in order. The result must stay usable after `b` is
  dropped.
- `longest_common_line(a, b)` returns the longest line that occurs in both
  texts (the first such line of `a` on ties), or `None`.
- Tests for each, including the drop-`b` case.

## 4. merge_words

- `sorted_words(text)` returns the whitespace-separated words of `text` as a
  sorted `Vec` of slices of `text`.
- `merge_sorted(a, b)` takes two sorted slices of string slices (`&[&str]`)
  and returns one sorted `Vec` holding the same string slices (no copies),
  stable: on equal words, `a`'s come first.
- `dedup_adjacent(words)` removes adjacent duplicates in place from a
  `Vec` of string slices.
- Tests: merge the words of two texts, then drop both intermediate `Vec`s
  and use the merged result while the texts are still alive; a property
  test that merging two sorted vectors gives a sorted vector whose length is
  the sum.

## 5. highlight

Find keyword occurrences in a text.

- `Highlighter::new(keywords, case_sensitive)` takes a slice of keyword
  string slices (`&[&str]`) that typically point into a configuration
  string.
- `spans(&self, text)` returns a `Vec<Span>` for every word of `text` that
  equals a keyword (ignoring ASCII case when not case-sensitive). A word is
  a maximal run of alphanumeric characters or `_`. Each `Span` holds the
  word as a slice of `text`, the matched keyword as the keyword's own
  string slice (not a copy), and the word's byte offset.
- The text and the keywords come from unrelated sources. Tests: keywords
  point into a long-lived configuration `String`; several texts are
  created and dropped in a loop, and each text's spans are used inside the
  loop; spans must stay usable after the `Highlighter` is dropped.

## 6. filters

Closures that capture borrows.

- `not_banned(banned)` takes `&[String]` and returns a closure
  `Fn(&str) -> bool` that is true when the word is not in `banned`. It must
  borrow `banned`, not clone it.
- `min_len(n)` returns a closure that is true for words of at least `n`
  characters.
- `Pipeline` holds a list of boxed filter closures, which may borrow data
  the caller owns. `Pipeline::new()`, `add(&mut self, filter)`, and
  `apply(&self, words)` where `words` is `&[&str]`: `apply` returns, in
  order, the words that pass every filter, as the same string slices.
- Tests: a pipeline with both filters over the words of a text, where the
  banned list is a local `Vec<String>`; the result of `apply` must stay
  usable after the `Pipeline` is dropped.

## 7. group_words

- `group_by_initial(text)` returns a `BTreeMap<char, Vec<..>>` from each
  word's lowercased first character to the words in order, as slices of
  `text`. Words are whitespace-separated, with `.,;:!?` trimmed from both
  ends; empty words are skipped.
- `largest_group(groups)` returns the key and the group as a slice of the
  map's vector (`&[&str]`), the smallest key on ties, or `None` for an
  empty map.
- `words_longer_than(groups, n)` returns every word longer than `n`
  characters, in key order, as slices of the original text. The result
  must stay usable after the map is dropped.
- Tests for each, including the drop-map case.

## 8. table

Tab-separated text with a header line.

- `Table::parse(text)` returns `Result<Table, TableError>`. The first line
  is the header; every row must have as many fields as the header, else
  `TableError` carries the 1-based line number. Header and cells are slices
  of `text`. A trailing newline is allowed.
- `column(&self, name)` returns `None` for an unknown column, else an
  iterator over that column's cells.
- `into_column(self, name)` consumes the table and returns that column's
  cells as a `Vec`, which must stay usable after the table is gone.
- `rows_where(&self, pred)` takes a closure over a row (`&[&str]`) and
  returns the matching rows as a `Vec` of row references.
- Tests for each, including `into_column` outliving the table.

## 9. logscan

A binary: `logscan <path> [--level LEVEL]`.

- Read the file at `path`. Each line is `<timestamp> <LEVEL> <message>`:
  an unsigned integer, one of `ERROR`, `WARN`, `INFO`, `DEBUG`, and the
  rest of the line. Malformed lines are counted and skipped.
- Parse lines into `Entry` values whose message is a slice of the file
  text, in a function that takes the text and needs no capabilities.
- Print the count per level, in the order above, then the number of
  malformed lines, then the three entries at `LEVEL` (default `ERROR`)
  with the largest timestamps, newest first, as `<timestamp> <message>`.
- Exit with code 2 and a usage message on stderr for bad arguments, and
  with code 1 and a message on stderr if the file cannot be read.
- Tests: the parsing function on good and malformed lines; one test that
  runs the whole program in the simulator against a simulated file.

## 10. kv_cursor

- A `Cursor` over a `&str` with `peek()`, `bump()`, `at_end()`,
  `skip_ws()`, `offset()`, and `eat_while(pred)`, which returns the eaten
  text as a slice of the source that must stay usable after the cursor is
  dropped.
- `parse_pairs(src, out)` parses `key=value` pairs separated by `;` or
  newlines, with optional spaces around keys, values, and separators. A
  value may be double-quoted (returned without the quotes; no escapes). It
  appends `(key, value)` slices of `src` to the caller's `&mut Vec`, which
  may already hold pairs from other strings, and returns the number
  appended, or a `ParseError` with the byte offset and a kind: empty key,
  missing `=`, or unterminated quote.
- Tests: pairs appended to a vector that already holds a pair from another
  string, all used together afterwards; each error kind; a slice from
  `eat_while` used after its cursor is dropped.
