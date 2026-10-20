# Tasks

Write each program below in Tide. Each is one `.rs` file; start it with a
comment naming the task and the path the file would have in a package
(`src/lib.rs` for a library, `src/main.rs` for a binary). Libraries need
tests but no `main`. Unless a task says otherwise, use only std, avoid
allocating copies of input text where the task says to return slices, and
include the tests the task lists plus any you find useful.

"A slice of `text`" means a `&str` that points into `text` itself, not a
copy. "Must stay usable after X is dropped" means a test drops X (for
example by creating it inside an inner block) and then uses the value, and
the program must pass `tide check` with that test.

## 1. canon

A dictionary of canonical spellings, built from one text and consulted with
words from others.

- `Dict::build(text)` splits `text` on whitespace and records, for each
  word compared case-insensitively (ASCII), the first spelling seen, as a
  slice of `text`.
- `get(&self, word)` returns the canonical spelling of `word` as a slice of
  the build text, or `None`.
- `canonical_or(&self, word)` returns the canonical spelling if there is
  one, else `word` itself.
- `rewrite(&self, line)` returns the words of `line` (split on whitespace),
  each replaced by its canonical spelling where one exists, as a `Vec` of
  slices.
- Tests: `get` and `canonical_or` on mixed case; a `get` result must stay
  usable after the `Dict` is dropped; a `canonical_or` result for a word
  taken from a second text, which lives in an inner block that also holds
  the `Dict`, must stay usable after the `Dict` is dropped while that text
  is still alive; the same for every element of a `rewrite` result.

## 2. rules

A list of row predicates, some owning their data and some borrowing it.

- `Rules<T>` holds predicates over `&T`, for any `T`. `matches(&self, x)`
  is true when every predicate accepts `x`; an empty list accepts
  everything.
- `at_least(&mut self, min: T)` (for `T: PartialOrd`) adds a predicate that
  owns `min`.
- `one_of(&mut self, allowed: &[T])` (for `T: PartialEq`) adds a predicate
  that borrows `allowed` without copying it.
- `custom(&mut self, f)` adds any closure the caller supplies, including one
  that borrows the caller's locals.
- `count_matching(&self, xs: &[T])` returns how many elements match.
- Tests: integers with `at_least` and `one_of` together; `&str` rows with a
  `custom` closure borrowing a local `HashSet`; a `Rules` whose `one_of`
  borrows a `Vec` created in the same block as the `Rules`.

## 3. scan

Byte scanning helpers over a haystack.

- `Needle::new(pattern: &str)` precomputes a skip table (any table you
  like) from an owned copy of the pattern.
- `positions(haystack: &[u8], needle: &Needle)` returns a lazy iterator of
  the start offsets of non-overlapping matches. The iterator must stay
  usable after the `Needle` is dropped: compute what you need from it
  before returning.
- `Lines<'a>` iterates over the lines of a `&'a str` (without line
  terminators; `\r\n` and `\n` both end a line). Give it a method
  `numbered(&self)` returning a lazy iterator of `(usize, &str)` pairs
  numbered from 1 that does not consume or move the `Lines`, and a method
  `containing(&self, word: &str)` returning a lazy iterator of the lines
  that contain `word`.
- Tests: `positions` on overlapping candidates (`"aaaa"` in `"aaaaaaaaa"`
  gives `[0, 4]`); the `Needle`-drop test; `numbered` and `containing` on a
  three-line text with `\r\n`; collecting `containing` while `word` is a
  local `String`.

## 4. longest_lines

Concurrent per-document work that borrows from the caller.

- `longest_lines(tasks: &Tasks, docs: &[&str], max_parallel: usize)`
  returns, for each document in order, its longest line as a slice of that
  document (the first on ties; `""` for an empty document). Run one child
  task per document, at most `max_parallel` at a time.
- `total_words(tasks: &Tasks, docs: &[String])` counts whitespace-separated
  words in every document, one child per document, and returns the sum.
- Errors: an error enum for the scope. A document containing a NUL byte is
  an error that names its index, and fails the whole call.
- Tests (run under the simulator): three documents; a NUL document; a
  result of `longest_lines` must stay usable after the `Vec` of `&str`
  passed as `docs` is dropped, as long as the `String`s it points into are
  alive.

## 5. pipeline

A two-stage pipeline built from helper functions that each start one child
on a scope they are given.

- `spawn_parser(s, lines: &[String], tx)` starts a child that parses each
  line as a decimal `i64` and sends it on a bounded channel, stopping at the
  first line that does not parse (that child fails with an error naming
  the 1-based line number). It returns the child's task, whose value is the
  number of values sent.
- `spawn_summer(s, rx)` starts a child that receives every value and
  returns the sum (with overflow reported as an error, not a trap).
- `sum_lines(tasks: &Tasks, lines: &[String])` creates a channel of
  capacity 4, calls both helpers on one scope, and returns `(count, sum)`.
- Tests under the simulator: ten lines; an empty input; a bad line 3, which
  must report line 3 as the error even though the summer is cancelled; an
  overflowing sum.

## 6. retry

An HTTP fetch with retries.

- `fetch_with_retry(net, clock, url, policy)` performs GET requests until
  one succeeds, a non-transient error occurs, or the attempts run out.
  `Policy` has `attempts: u32` (at least 1), `first_delay: Duration`
  (doubled after every failed attempt), and `overall: Duration`, a deadline
  for the whole call measured from its start, which also bounds each
  request and each sleep.
- On failure it returns an error carrying the last network error and the
  number of attempts made; a cancellation is returned at once, not retried.
- Tests under the simulator: a peer that answers 503 twice, then `200 ok`,
  which succeeds on attempt 3 after about `first_delay * 3` of virtual time;
  a peer that always answers 404, which fails after one attempt; a peer
  that hangs, which fails with a timeout no later than `overall` (with
  slack); a `Net` on which everything times out.

## 7. backup

Copy files from one directory to another.

- `backup(fs: &Fs, src: &str, dst: &str, suffix: &str)` copies every
  regular file directly inside `src` whose name ends in `suffix` into
  `dst` (created if missing), durably: when `backup` returns `Ok`, each
  copy and its directory entry survive a crash. It returns the copied names
  sorted. The function must only be able to read below `src` and write
  below `dst`.
- A file in `dst` with the same name is replaced, so that a crash at any
  point leaves either the old or the new contents, never a mix.
- Tests under the simulator: two matching files and one other; an existing
  file that is replaced; a missing `src` directory, which is an error.

## 8. ring

A fixed-capacity ring buffer of `i64`, with its rules written as contracts.

- `Ring::with_capacity(n)` (`n` at least 1), `push(&mut self, x)` (the
  caller must not push into a full ring), `pop(&mut self) -> Option<i64>`
  (oldest first), `len`, `is_full`, and `extend_from(&mut self, xs: &[i64])`
  (the caller must ensure the elements fit).
- Every method states what it guarantees about `len` in terms of its value
  on entry; `pop` returns `None` exactly when the ring was empty.
- `drain_into(&mut self, out: &mut Vec<i64>)` moves every element into
  `out`, oldest first, and guarantees that `out` grew by the ring's old
  length and that the ring is empty. Use a loop whose invariant relates
  the moved count to the two lengths.
- Tests: push and pop through a wraparound; a property test that a sequence
  of pushes and pops agrees with a `VecDeque`; a test that pushing into a
  full ring panics with a precondition violation.

## 9. deps

Build-order resolution over a dependency graph.

- `Graph::add(&mut self, name: &str) -> Handle` adds a node (names are
  unique; adding a name again returns the existing handle);
  `depends(&mut self, a, b)` records that `a` needs `b`;
  `remove(&mut self, h)` deletes a node and every edge to it.
- `order(&self)` returns the names in an order where every node comes
  after its dependencies (ties broken by insertion order), or an error
  naming the nodes of one cycle.
- `dependents(&self, h)` returns a lazy iterator over the names of the
  nodes that directly need `h`.
- Tests: a diamond; a cycle; removing a node breaks a cycle; a handle used
  after its node is removed is detected, not misread as another node.

## 10. tally

Count words across documents, in parallel where it is pure and in tasks
where it is shared.

- `freq(text)` returns word counts (lowercased ASCII words, split on
  non-alphanumeric characters) as a `BTreeMap<String, u64>`.
- `freq_all(docs: &[String])` computes every document's counts in parallel
  and merges them into one map.
- `Tally` is a counter that several tasks update concurrently: `add(&self,
  word, n)` and `snapshot(&self) -> BTreeMap<String, u64>`, plus
  `merge_into(&self, other: &Tally)`, which moves all of `self`'s counts
  into `other` atomically; merging a tally into itself is a no-op.
- `count_concurrently(tasks: &Tasks, docs: &[String])` spawns one child
  per document that adds its counts to a shared `Tally`, and returns the
  snapshot.
- Tests: `freq_all` equals the sequential merge; under the simulator,
  `count_concurrently` gives the same totals on every seed; merging into
  itself.

## 11. dedupe

A binary: `dedupe [--count] [FILE]` prints each distinct line of `FILE`
(or standard input when no file is given) once, in order of first
appearance. With `--count`, each line is prefixed by its number of
occurrences and a tab. Exit code 0 on success, 2 on a usage error (unknown
flag or more than one file, with a message on stderr), 1 if the file cannot
be read.

- Keep the logic in a function that takes the lines' source and returns
  the output text, and test it directly.
- Tests: the logic on repeated lines with and without counts; under the
  simulator, the whole program reading a file named in the arguments, and
  reading standard input.
