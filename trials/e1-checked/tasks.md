# E1 checked-toolchain tasks

Thirty held-out tasks for the paired E1 run that `protocol.md` defines. A
writer never sees this file whole: the harness gives it the preface below
and one task, rendered for its arm (protocol §4.2). Lines that start with
`- Tide:` or `- Rust:`, with their indented continuation lines, belong to
that arm only; every other line goes to both arms.

The tasks are frozen with the protocol. Their names, specs, and checked
cases do not change once the first gate writer has started (protocol §2).

## Preface

Write the program that the task below describes.

- Tide: Write it in Tide. `reference.md` in this directory is the language
  reference: read it before you write code. `reference-companion.md` holds
  examples, simulator details, and habit tables; read the parts you need.
  Use Tide's std only.
- Rust: Write it in Rust 2024. Use std; you may also use `serde` (with
  `derive`), `serde_json`, and `thiserror`, which `Cargo.toml` already
  lists. No other crates.

The package is this directory. A library goes in `src/lib.rs` and a binary
in `src/main.rs`, as one file; the task says which. Put your own tests in
that file. Run `./check` to check the package and its tests without running
them, and `./test` to check it and run your tests. Each run counts toward
your budget of 20 runs and 40 minutes. Your first run is measured, so write
the whole program and its tests before it. Stop when both pass and you
believe the task is done.

The grader then runs its own acceptance tests, which do exactly what the
task's checked cases say. For a library they are a separate test file that
calls your public items, so every item the task names must be `pub` with
the name and shape the task gives; any lifetimes or generic spelling that
let the checked cases compile are fine. For a binary they run the built
program as a process in a fresh working directory with only the files,
arguments, standard input, and environment variables the case names, and
compare its standard output, the start of its standard error, and its exit
code. Where the task says work runs concurrently or in parallel, a grader
also reads your program to confirm that it does.

- Tide: Acceptance tests for a library that takes capabilities run under
  the simulator, over 20 seeds unless a case says otherwise: they create the case's files with `sim.world.file` or
  `sim.fs`, pass `&sim.fs` (and the other capabilities) to your functions,
  and read virtual time with `sim.elapsed()`. A path in a case is relative
  to the `Fs` your function receives.
- Rust: Acceptance tests for a library that touches files run in a fresh
  temporary directory and pass its path where the task names a directory.
  A path in a case is relative to that directory. Timing cases use real
  time and allow the slack they state.

"A slice of `text`" means a `&str` that points into `text` itself, not a
copy. "Stays usable after X is dropped" means a checked case drops X (for
example by creating it in an inner block) and then uses the value, and the
acceptance tests must compile and pass with that use. "Panics" means the
case expects a panic, with any message.

## Band A: core language

### A1. anagrams (library)

An index of the anagram classes in one text.

- Words are maximal runs of ASCII letters. Two words are anagrams when
  their lowercased letters form the same multiset. Words are compared
  exactly for identity: `Dusty` and `dusty` are two words.
- `AnagramIndex::new(text: &str)` builds the index.
- `groups(&self)` returns `Vec<Vec<&str>>`: every class with at least two
  distinct words, each class listing its distinct words in order of first
  occurrence, and the classes ordered by their first word's first
  occurrence. Every word is a slice of `text`.
- `partners(&self, word: &str)` returns `Vec<&str>`: the distinct words of
  the index that are anagrams of `word` but not equal to it, in order of
  first occurrence, as slices of `text`.

Checked cases:

1. `"listen silent enlist google tinsel"` groups to
   `[["listen", "silent", "enlist", "tinsel"]]`.
2. `"Dusty study, night thing; dusty"` groups to
   `[["Dusty", "study", "dusty"], ["night", "thing"]]`.
3. `""` and `"abc def"` group to `[]`.
4. A `groups` result stays usable after the index is dropped, while the text
   lives.
5. `partners` of a word read from a `String` created and dropped in an inner
   block returns `["silent", "enlist", "tinsel"]` for `"listen"` on case 1's
   text, and that result stays usable after both the `String` and the index
   are dropped.

### A2. money (library)

Decimal amounts as integer cents.

- `pub struct Cents(pub i64)`, deriving `Debug`, `Clone`, `Copy`,
  `PartialEq`, `Eq`.
- `parse_amount(s: &str) -> Result<Cents, AmountError>`: an optional `-`,
  one or more ASCII digits, and optionally `.` followed by one or two
  digits (`"1.5"` is 150 cents). Anything else is `AmountError::Invalid`;
  a well-formed amount outside `i64` cents is `AmountError::Overflow`.
  `AmountError` derives `Debug` and `PartialEq`.
- `total(items: &[(Cents, u32)]) -> Result<Cents, AmountError>`: the sum of
  price times quantity; `Overflow` if any step overflows `i64`.
- `format_amount(c: Cents) -> String`: `-` for negatives, at least one
  integer digit, and exactly two decimals.
- `split_even(c: Cents, parts: u32) -> Vec<Cents>`: `parts` shares that sum
  to `c` and differ by at most one cent, larger shares first. Panics if
  `parts` is 0.

Checked cases:

1. `"12.34"` parses to 1234, `"-0.5"` to -50, `"7"` to 700.
2. `"1.234"`, `""`, `"-"`, `".5"`, `"1."`, `"+1"`, and `"1,00"` are
   `Invalid`.
3. `"92233720368547758.07"` parses to `i64::MAX`, `"-92233720368547758.08"`
   to `i64::MIN`, and `"92233720368547758.08"` is `Overflow`.
4. `total` of `[(250, 3), (-100, 2)]` is 550; of `[(1, u32::MAX)]` is
   4294967295; of `[(i64::MAX, 2)]` is `Overflow`.
5. `format_amount` gives `"0.00"` for 0, `"-12.05"` for -1205, `"1234.50"`
   for 123450, and `"-92233720368547758.08"` for `i64::MIN`.
6. `split_even` gives `[34, 33, 33]` for (100, 3), `[-2, -3]` for (-5, 2),
   and `[0]` for (0, 1); `split_even(Cents(1), 0)` panics.

### A3. calc (library)

An integer expression evaluator.

- `eval(src: &str) -> Result<i64, CalcError>` evaluates decimal integer
  literals, binary `+ - * / %`, unary `-`, and parentheses, with spaces
  allowed between tokens. Unary minus binds tighter than `* / %`, which bind
  tighter than `+ -`; binary operators are left-associative. `/` and `%`
  truncate toward zero, as Rust's do.
- `CalcError` derives `Debug` and `PartialEq` and has the variants
  `DivByZero { pos: usize }`, `Overflow { pos: usize }`, and
  `Unexpected { pos: usize }`, where `pos` is a byte offset into `src`:
  the operator for `DivByZero` and for an operator that overflows, the
  first digit of a literal that does not fit in `i64`, and for
  `Unexpected` the first character that cannot continue a valid
  expression, or `src.len()` if the input ends too early.
- `CalcError` implements `Display`: `division by zero at {pos}`,
  `overflow at {pos}`, `unexpected input at {pos}`.

Checked cases:

1. `"1 + 2 * 3"` is 7, `"(1+2)*3"` is 9, `"-(-4) % 3"` is 1, `"7 / -2"` is
   -3, and `"- 9223372036854775807 - 1"` is `i64::MIN`.
2. `"1 / (2 - 2)"` is `DivByZero { pos: 2 }`.
3. `"9223372036854775807 + 1"` is `Overflow { pos: 20 }`,
   `"9223372036854775808"` is `Overflow { pos: 0 }`,
   `"-9223372036854775808"` is `Overflow { pos: 1 }`, and
   `"-(-9223372036854775807 - 1)"` is `Overflow { pos: 0 }`.
4. `"2 *"` is `Unexpected { pos: 3 }`, `"(1"` is `Unexpected { pos: 2 }`,
   `"1 2"` is `Unexpected { pos: 2 }`, `""` is `Unexpected { pos: 0 }`, and
   `"a"` is `Unexpected { pos: 0 }`.
5. `CalcError::DivByZero { pos: 2 }.to_string()` is
   `"division by zero at 2"`.

### A4. poly (library)

Polynomials with integer coefficients.

- `Poly` derives `Debug`, `Clone`, and `PartialEq`. `Poly::new(coeffs:
  Vec<i64>)` takes coefficients lowest degree first and drops trailing
  zeros, so the zero polynomial has no coefficients. `coeffs(&self) ->
  &[i64]` and `degree(&self) -> Option<usize>` (`None` for zero).
- `+`, `-`, and `*` work on two `Poly` values and on two `&Poly` values,
  and unary `-` on a `Poly`. Coefficient arithmetic may panic on overflow.
- `eval(&self, x: i64) -> Option<i64>`: `None` if any step overflows.
- `Display` prints terms from the highest degree down, such as
  `3x^2 - x + 5`: a coefficient of 1 or -1 is left out except on the
  constant, the degree-1 term prints as `x`, and zero prints `0`.

Checked cases:

1. `Poly::new(vec![5, -1, 3])` prints `3x^2 - x + 5`;
   `Poly::new(vec![-2, 0, 0, -1])` prints `-x^3 - 2`.
2. `Poly::new(vec![0, 0])` prints `0` and has degree `None`.
3. `(x + 1) * (x - 1)` built from owned values has coefficients
   `[-1, 0, 1]`; `&a + &b` leaves `a` and `b` usable afterwards.
4. `a.clone() - a` is the zero polynomial; `-Poly::new(vec![0, 1])` prints
   `-x`.
5. `x^2 - 1` evaluates to `Some(8)` at 3; `x^3` evaluates to `None` at
   3,000,000.

### A5. shapes (library)

Shapes behind a trait object.

- `pub trait Shape { fn area(&self) -> f64; fn perimeter(&self) -> f64;
  fn name(&self) -> &str; }`
- `Circle::new(r: f64)`, `Rect::new(w: f64, h: f64)`, and
  `Triangle::new(a: f64, b: f64, c: f64)` return `Option<Self>`: `None`
  when a length is not finite or not positive, or when the triangle's
  sides fail the strict triangle inequality. Each implements `Shape`; the
  names are `circle`, `rect`, and `triangle`; a triangle's area is Heron's
  formula.
- `largest(shapes: &[Box<dyn Shape>]) -> Option<&dyn Shape>`: the largest
  area, the first on ties.
- `sort_by_area(shapes: &mut [Box<dyn Shape>])`: ascending, stable.
- `total_area(shapes: &[Box<dyn Shape>]) -> f64`.
- `describe(s: &dyn Shape) -> String`, such as
  `rect: area 6.00, perimeter 10.00` (two decimals).

Checked cases (areas compared within 1e-9):

1. A unit circle has area π; `Triangle::new(3.0, 4.0, 5.0)` has area 6.
2. `describe` of `Rect::new(2.0, 3.0)` is
   `"rect: area 6.00, perimeter 10.00"`.
3. `Triangle::new(1.0, 2.0, 3.0)`, `Circle::new(f64::NAN)`, and
   `Rect::new(0.0, 1.0)` are `None`.
4. `largest(&[])` is `None`; for `[unit circle, rect 2×3, rect 1×6]` it
   returns the 2×3 rect (perimeter 10), the first of the two with area 6.
5. Sorting `[rect 2×3, unit circle, triangle 3-4-5]` gives names
   `circle, rect, triangle`; `total_area` of that slice is 12 + π.

### A6. lru (library)

A least-recently-used cache.

- `Lru<K, V>` for `K: Eq + Hash + Clone`. `Lru::new(capacity: usize)`
  panics if `capacity` is 0.
- `get(&mut self, k: &K) -> Option<&V>` marks `k` most recently used;
  `peek(&self, k: &K) -> Option<&V>` does not.
- `put(&mut self, k: K, v: V) -> Option<(K, V)>`: for an existing key,
  replace the value, mark it most recent, and return `None`; for a new key
  in a full cache, evict and return the least recently used entry first.
- `len(&self) -> usize`; `keys_recent_first(&self) -> Vec<&K>`.

Checked cases:

1. Capacity 2: put a→1, put b→2, get a, put c→3 returns `Some((b, 2))`;
   `keys_recent_first` is `[c, a]`.
2. Capacity 2: put a, put b, peek a, put c returns `Some((a, ..))`.
3. Putting an existing key updates its value, evicts nothing, and makes it
   most recent.
4. `Lru::<u32, u32>::new(0)` panics.
5. Capacity 100,000: 300,000 puts of distinct keys interleaved with
   300,000 gets finish within the 10-second test limit, and the final
   `len` is 100,000.

### A7. trie (library)

A set of strings in a prefix tree.

- `Trie::new()`; `insert(&mut self, w: &str) -> bool` (true if new);
  `contains(&self, w: &str) -> bool`; `remove(&mut self, w: &str) -> bool`
  (true if it was present); `len(&self) -> usize`.
- `with_prefix(&self, p: &str) -> Vec<String>`: the words starting with
  `p`, sorted by byte order.
- `longest_common_prefix(&self) -> String`: of all words; `""` when empty.
- Words are any UTF-8 strings, the empty string included.

Checked cases:

1. After inserting `car card care cat dog`, `with_prefix("car")` is
   `[car, card, care]` and `with_prefix("")` lists all five.
2. Removing `car` returns true; then `contains("car")` is false,
   `contains("card")` true, and `with_prefix("ca")` is `[card, care, cat]`.
3. Removing an absent word returns false; inserting a present one returns
   false; `len` counts each word once.
4. `longest_common_prefix` of `interview internet interval` is `inter`; of
   an empty trie, `""`; after also inserting `""`, `""`.
5. `über` and `üben` are both returned by `with_prefix("üb")`.

### A8. template (library)

String templates with named placeholders.

- `Template::parse(src: &str) -> Result<Template, TemplateError>`.
  `{name}` is a placeholder, where `name` matches `[A-Za-z_][A-Za-z0-9_]*`
  with nothing else between the braces; `{{` and `}}` are literal braces.
- `names(&self)` returns `Vec<&str>`: the distinct placeholder names in
  order of first occurrence, as slices of `src`.
- `render(&self, vars: &HashMap<&str, &str>) -> Result<String,
  TemplateError>`.
- `TemplateError` derives `Debug` and `PartialEq`: `Unclosed { pos }` for a
  `{` with no closing `}`, `BadName { pos }` for a `{` whose contents are not
  a name, `StrayBrace { pos }` for a lone `}`, all byte offsets of that
  brace, and `Missing { name: String }`. Its `Display` gives
  `unclosed placeholder at {pos}`, `bad placeholder name at {pos}`,
  `stray '}' at {pos}`, and `missing value for {name}`.

Checked cases:

1. `"Hi {name}, {{ok}} {name}!"` has names `["name"]` and renders with
   name = `Ann` to `"Hi Ann, {ok} Ann!"`.
2. `"{{{x}}}"` renders with x = `1` to `"{1}"`; `"}}"` renders to `"}"`.
3. `"{a}{b}"` with only `a` given is `Missing { name: "b" }`, whose
   `to_string()` is `"missing value for b"`.
4. `"x {y"` is `Unclosed { pos: 2 }`; `"{1x}"`, `"{}"`, and `"{ a }"` are
   `BadName { pos: 0 }`; `"a } b"` is `StrayBrace { pos: 2 }`.
5. A `names()` result stays usable after the template is dropped, while
   `src` lives.

### A9. stock (library)

An inventory read from and written to JSON.

- `Inventory { warehouse: String, items: Vec<Item> }` and `Item { sku:
  String, quantity: u32, unit_price_cents: u64, tags: Vec<String>,
  discontinued: Option<bool> }`, all fields `pub`, both deriving `Debug`,
  `Clone`, `PartialEq`. JSON keys are camelCase (`unitPriceCents`); a
  missing `tags` decodes as empty and a missing `discontinued` as `None`.
- `Change` is an enum with variants `Restock { sku: String, qty: u32 }`,
  `Sell { sku: String, qty: u32 }`, and `Discontinue { sku: String }`, in
  JSON externally tagged under those variant names:
  `{"Sell":{"sku":"a","qty":2}}`.
- `load(bytes: &[u8]) -> Result<Inventory, LoadError>`: `LoadError::Json(..)`
  holds the decoder's error, and `LoadError::Duplicate(String)` names a
  repeated sku.
- `apply(inv: &mut Inventory, changes: &[u8]) -> Result<(), ApplyError>`:
  `changes` is a JSON array of `Change`. Errors: `ApplyError::Json(..)`,
  `UnknownSku(String)`, `Insufficient(String)` (selling more than the
  quantity), `Overflow(String)` (a restock past `u32::MAX`). It applies all
  changes or none: on an error, `inv` is unchanged.
- `save(inv: &Inventory) -> Vec<u8>`: JSON that `load` reads back equal.
- Tide: Use the `Encode` and `Decode` derives and `std::json`; the `Json`
  variants hold a `json::Error`.
- Rust: Use serde and serde_json; the `Json` variants hold a
  `serde_json::Error`.

Checked cases, where `inv` is
`{"warehouse":"w1","items":[{"sku":"a","quantity":3,"unitPriceCents":250},{"sku":"b","quantity":0,"unitPriceCents":100,"tags":["x"]}]}`
and each case starts from a fresh load of it:

1. `inv` loads with `a`'s `tags` empty and both `discontinued` `None`.
2. Two items with sku `a` give `Duplicate("a")`; `{bad` gives `Json(_)`.
3. `[{"Sell":{"sku":"a","qty":2}},{"Restock":{"sku":"b","qty":5}}]` leaves
   `a` at 1 and `b` at 5.
   `[{"Restock":{"sku":"b","qty":1}},{"Sell":{"sku":"a","qty":9}}]` gives
   `Insufficient("a")` and leaves `inv` unchanged; selling `z` gives
   `UnknownSku("z")`; restocking `b` by 1 and then by 4294967295 gives
   `Overflow("b")` and leaves `inv` unchanged.
4. `[{"Discontinue":{"sku":"a"}}]` sets `discontinued` to `Some(true)`.
5. `load(&save(&inv))` equals `inv`, and the saved bytes contain
   `"unitPriceCents"`.

### A10. sentences (library)

Sentences of a text, borrowed from it.

- `Doc::new(text: &str)`. A sentence ends after `.`, `!`, or `?` that is
  followed by whitespace or the end of the text; the text after the last
  such end is a final sentence. Sentences are trimmed and empty ones
  skipped.
- `sentences(&self)` returns a lazy iterator of `&str`, slices of `text`;
  the iterator stays usable after the `Doc` is dropped.
- `longest(&self) -> Option<&str>`: the most `char`s, the first on ties.
- `words_of(&self, i: usize) -> Option<Vec<&str>>`: sentence `i` split on
  whitespace, with leading and trailing ASCII punctuation removed from each
  word and empty words skipped.
- `with_word(&self, word: &str)` returns a lazy iterator of the sentences
  containing `word` as one of their `words_of` words, compared ASCII
  case-insensitively. It stays usable after the `Doc` is dropped, while
  `word` lives.

Checked cases:

1. `"Hi there. How are you? Fine!"` gives
   `["Hi there.", "How are you?", "Fine!"]`; `"a.b. c"` gives
   `["a.b.", "c"]`; `"No end"` gives `["No end"]`; `"   "` gives `[]`.
2. `longest` of `"ab. cd. e."` is `"ab."`; `words_of(1)` of case 1's text is
   `["How", "are", "you"]`, and `words_of(9)` is `None`.
3. `with_word("HI")` on case 1's text yields `["Hi there."]`.
4. A `sentences()` iterator created in an inner block, where the `Doc` is
   dropped, is iterated afterwards; the same for `with_word`.

## Band B: authority and I/O

### B1. findtext (binary)

Print the lines of files that contain a pattern.

- `findtext [-i] PATTERN FILE...` prints every line of each file that
  contains `PATTERN` as `FILE:N:LINE`, with `N` counted from 1 and the line
  without its `\n` or `\r\n`. `-i` matches ASCII case-insensitively.
- Exit 0 if any line matched, 1 if none did, 2 if there was any error.
- A missing pattern or file, or an unknown option, prints a message
  starting `usage: findtext` to standard error and exits 2 without reading
  anything. A file that cannot be read or is not UTF-8 prints
  `findtext: FILE: ` and a reason to standard error; the other files are
  still searched.
- Tide: The binary has `#[tide::main]` and takes what it needs from `root`.

Checked cases, with `a.txt` = `"one\nTwo\nthree two\n"` and `b.txt` =
`"nothing\r\ntwo\r\n"`:

1. `two a.txt b.txt` prints `a.txt:3:three two\nb.txt:2:two\n`, exit 0.
2. `-i two a.txt` prints `a.txt:2:Two\na.txt:3:three two\n`, exit 0.
3. `zzz a.txt` prints nothing, exit 1; `"" a.txt` prints all three lines,
   exit 0.
4. `two missing.txt a.txt` prints `a.txt:3:three two\n`, standard error
   starts `findtext: missing.txt: `, exit 2.
5. `two` and `-x two a.txt` print nothing to standard output, standard
   error starts `usage: findtext`, exit 2.

### B2. colstats (binary)

Statistics of one integer column read from standard input.

- `colstats [--col N] [--sep C]`, options in any order: `N` is a 1-based
  column (default 1), `C` a single-character separator (default `,`).
- Each line that is not blank is split on `C`; field `N`, trimmed, must
  parse as `i64`. Blank lines are skipped but counted in line numbers.
- Output: `count=K sum=S min=A max=B`, or `count=0` when there are no
  values, then `\n`, exit 0.
- A line with too few fields or a non-integer field prints
  `colstats: line L: ` and a reason to standard error and exits 1 with
  nothing on standard output. A sum that overflows `i64` prints
  `colstats: overflow` and exits 1. A bad option prints a message starting
  `usage: colstats` and exits 2.
- Tide: The binary has `#[tide::main]` and takes what it needs from `root`.

Checked cases:

1. `--col 2` with `"a,1\nb,-5\n\nc,10\n"` prints
   `count=3 sum=6 min=-5 max=10\n`.
2. `--sep ; --col 1` with `"4;x\n2;y\n"` prints `count=2 sum=6 min=2 max=4\n`.
3. Empty input prints `count=0\n`.
4. `"1\nx\n"` prints nothing, standard error starts `colstats: line 2: `,
   exit 1. `"9223372036854775807\n1\n"` gives standard error starting
   `colstats: overflow`, exit 1.
5. `--col 0`, `--col` alone, and `--sep ab` exit 2 with standard error
   starting `usage: colstats`.

### B3. treesize (library)

Sizes of the files below a directory.

- `Summary { files: u64, dirs: u64, bytes: u64, largest: Option<(String,
  u64)> }`, all fields `pub`, deriving `Debug` and `PartialEq`. `dirs`
  counts directories below the root, not the root itself. Paths are
  relative to the root, joined with `/`. `largest` is the largest file, the
  smallest path on ties.
- `top_n` returns the `n` largest files as `(path, size)`, by size
  descending, then path ascending.
- A missing root directory is an `io::Error` of kind `NotFound`.
- Tide: `summarize(fs: &Fs, dir: &str) -> io::Result<Summary>` and
  `top_n(fs: &Fs, dir: &str, n: usize) -> io::Result<Vec<(String, u64)>>`,
  where `dir` is resolved by `fs`.
- Rust: `summarize(dir: &Path) -> io::Result<Summary>` and
  `top_n(dir: &Path, n: usize) -> io::Result<Vec<(String, u64)>>`.

Checked cases, with `data/a.txt` (5 bytes), `data/sub/b.bin` (10),
`data/sub/deeper/c` (10), and an empty directory `data/empty`:

1. `data` summarizes to 3 files, 3 directories, 25 bytes, largest
   `("sub/b.bin", 10)`.
2. `top_n` 2 is `[("sub/b.bin", 10), ("sub/deeper/c", 10)]`; `top_n` 0 is
   `[]`.
3. `data/empty` summarizes to zeros and `largest` `None`.
4. A missing directory gives `NotFound`.

### B4. rotlog (library)

An append-only log that rotates by size.

- The log directory holds `current.log`, `old.1.log`, and `old.2.log`.
- `append(&mut self, line: &str) -> io::Result<()>` writes `line` and `\n`
  to `current.log` in append mode. A line containing `\n` is an error of
  kind `InvalidInput`. If `current.log` is not empty and the write would
  make it larger than `max_bytes`, rotate first: `old.1.log` replaces
  `old.2.log`, `current.log` becomes `old.1.log`, and a new `current.log`
  starts. A line longer than `max_bytes` is still written.
- Opening continues an existing `current.log`, counting its size.
- Tide: `RotLog::open(fs: &Fs, max_bytes: u64) -> io::Result<RotLog>` and
  the free function `read_all(fs: &Fs) -> io::Result<Vec<String>>`, where `fs` is the log
  directory.
- Rust: `RotLog::open(dir: &Path, max_bytes: u64) -> io::Result<RotLog>`
  and the free function `read_all(dir: &Path) -> io::Result<Vec<String>>`.
- `read_all` returns the lines of `old.2.log`, `old.1.log`, and
  `current.log`, oldest first, skipping missing files.

Checked cases, with `max_bytes` 10:

1. Appending `aaaa`, `bbbb`, `cc` leaves `old.1.log` = `"aaaa\nbbbb\n"` and
   `current.log` = `"cc\n"`; `read_all` is `[aaaa, bbbb, cc]`.
2. Appending `dddddddd` and then `e` as well leaves `read_all` =
   `[cc, dddddddd, e]`, the first file having been dropped.
3. `"a\nb"` gives `InvalidInput` and changes no file.
4. A 20-byte line on an empty log is written; the next append rotates.
5. Open, append `a`, drop, open again, append `b`: `current.log` is
   `"a\nb\n"`.

### B5. envreport (binary)

Report environment variables, with defaults from a file.

- `envreport [--defaults FILE] NAME...` prints, for each `NAME` in order,
  `NAME=value` from the environment, else from the defaults file, else
  `NAME is unset`. Exit 0 if every name had a value, else 1. A variable
  whose value is not valid Unicode counts as unset.
- The defaults file has `KEY=VALUE` lines, split at the first `=`, the key
  trimmed and the value kept as is; blank lines and lines starting with `#`
  are ignored.
- No `NAME`: a message starting `usage: envreport` on standard error,
  exit 2. An unreadable defaults file: `envreport: FILE: ` and a reason,
  exit 2. A line without `=` or with an empty key: `envreport: FILE:L: `
  and a reason, exit 2. On exit 2 nothing goes to standard output.
- Tide: The binary has `#[tide::main]` and takes what it needs from `root`.

Checked cases (the environment holds only the variables named):

1. With `A=1`: `A B` prints `A=1\nB is unset\n`, exit 1.
2. With `A=1` and `d.env` = `"B=two\n# c\nA=zzz\n\nC=x=y\n"`:
   `--defaults d.env A B C` prints `A=1\nB=two\nC=x=y\n`, exit 0.
3. `--defaults nope.env A` exits 2, standard error starting
   `envreport: nope.env: `.
4. `d.env` = `"oops\n"`: `--defaults d.env A` exits 2, standard error
   starting `envreport: d.env:1: `.
5. No arguments: exit 2, standard error starting `usage: envreport`.

### B6. export (library)

Write named reports, confined to one output directory.

- `export` writes each `(name, body)` of `reports: &[(&str, &str)]` to
  `out/<name>`, creating `out` and any subdirectories in `name`, replacing
  existing files, and returns `Ok(count)`.
- A name is relative, made of `/`-separated components, each nonempty and
  neither `.` nor `..`, with no `\`. An invalid name gives
  `ExportError::BadName(name)` and a repeated name
  `ExportError::Duplicate(name)`, both carrying the name as a `String`;
  both are found before anything is
  written, so then nothing is written. An I/O failure gives
  `ExportError::Io(io::Error)`.
- Nothing outside `out` is ever created or changed.
- Tide: `export(fs: &Fs, reports: &[(&str, &str)]) -> Result<usize,
  ExportError>`. Write only through an `Fs` attenuated to `out` with
  `dir` (structure: the source contains `.dir(`).
- Rust: `export(root: &Path, reports: &[(&str, &str)]) -> Result<usize,
  ExportError>`.

Checked cases:

1. `[("a.txt", "1"), ("2026/jan.txt", "2")]` returns 2 and writes
   `out/a.txt` and `out/2026/jan.txt`; an existing `out/a.txt` is replaced.
2. `[("ok", "x"), ("../evil", "y")]` is `BadName("../evil")`; neither
   `out/ok` nor `evil` exists afterwards.
3. `/abs`, `a//b`, `a/./b`, the empty name, and `a\b` are each `BadName`.
4. `[("a", "1"), ("a", "2")]` is `Duplicate("a")`.
5. When `out` is an existing file, the result is `Io(_)`.

### B7. renameplan (library)

Plan simultaneous renames in a pure core, then apply them.

- `plan(moves: &[(&str, &str)]) -> Result<Vec<(String, String)>,
  PlanError>` returns single renames that, applied in order, realize all
  moves at once, without any rename's target existing at that moment.
  Moves to the same name are dropped. A cycle goes through a temporary
  name: one of the cycle's names followed by `.tmp`, with `.tmp` repeated
  until it differs from every name in `moves`. Two moves from one source
  give `PlanError::DuplicateSource(name)`; two to one target,
  `PlanError::DuplicateTarget(name)`. `PlanError` derives `Debug` and
  `PartialEq`.
- `apply` performs the renames in order. If a step's source neither
  exists before the first rename nor is the target of an earlier step, it
  returns an error of kind `NotFound` and renames nothing.
- `PlanError`'s variants carry the name as a `String`.
- Tide: `plan` is authority-free and carries `#[pure]` (structure:
  `#[pure]` directly before `plan`'s signature).
  `apply(fs: &Fs, steps: &[(String, String)]) -> io::Result<()>`.
- Rust: `apply(dir: &Path, steps: &[(String, String)]) -> io::Result<()>`.

Checked cases (a plan is checked by replaying it on a map from names to
contents in which each source holds a distinct content: each step's
source must exist and its target must not, and at the end each move's
target must hold its source's original content and no other name may
remain from the moves):

1. `[(a, b), (b, a)]`, `[(a, b), (b, c)]`, and `[(a, b), (b, c), (c, a)]`
   give valid plans.
2. `[(a, a)]` gives `[]`.
3. `[(a, b), (a, c)]` is `DuplicateSource("a")`; `[(a, c), (b, c)]` is
   `DuplicateTarget("c")`.
4. Applying the swap plan to files `a` = `1`, `b` = `2` leaves `a` = `2`,
   `b` = `1`.
5. Applying `[(x, y)]` without `x` gives `NotFound` and changes nothing.

### B8. usage (binary)

Total bytes per user from JSON lines on standard input.

- Each line that is not blank is a JSON object with `user` (string),
  `bytes` (`u64`), and `op` (`"read"` or `"write"`); other keys are
  ignored.
- Output, one line per user sorted by user: `USER read=R write=W`, the
  totals of that user's reads and writes. Exit 0.
- A line that does not decode prints `usage: line L: ` and a reason to
  standard error and is skipped; the program then exits 1 after printing
  the totals. A total that overflows `u64` prints `usage: overflow` to
  standard error and exits 1 with nothing on standard output.
- Tide: The binary has `#[tide::main]` and uses the `Decode` derive and
  `std::json`.
- Rust: Use serde and serde_json.

Checked cases:

1. Lines `{"user":"bo","bytes":5,"op":"read"}`,
   `{"user":"al","bytes":7,"op":"write","extra":1}`, and
   `{"user":"bo","bytes":3,"op":"write"}` print
   `al read=0 write=7\nbo read=5 write=3\n`, exit 0.
2. With `{bad` as the second of those lines (pushing the others down),
   the same totals print, standard error starts `usage: line 2: `, exit 1.
3. An `op` of `"delete"` or a `bytes` of `-1` is a line that does not decode.
4. Empty input prints nothing, exit 0.
5. Two reads of 18446744073709551615 bytes by one user: standard error
   starts `usage: overflow`, nothing on standard output, exit 1.

### B9. kvstore (library)

A small key-value store saved by atomic replace.

- `Store::open` loads `store.kv` if it exists. Its lines are
  `key\tvalue\n`, sorted by key; a line without a tab is an error of kind
  `InvalidData`. A leftover `store.kv.tmp` is ignored.
- `get(&self, key: &str) -> Option<&str>`;
  `set(&mut self, key: &str, value: &str) -> io::Result<()>`;
  `remove(&mut self, key: &str) -> io::Result<bool>`.
- Each `set` and successful `remove` saves the whole store before
  returning: it writes `store.kv.tmp` and makes it durable, renames it over
  `store.kv`, and makes the directory entry durable. Keys may not contain
  `\t` or `\n`, values may not contain `\n`; violations are errors of kind
  `InvalidInput` and change nothing.
- Tide: `Store::open(fs: &Fs) -> io::Result<Store>`, where `fs` is the
  store's directory (structure: the source contains `write_synced(` and
  `sync_dir(`).
- Rust: `Store::open(dir: &Path) -> io::Result<Store>` (structure: the
  source contains `sync_all(`).

Checked cases:

1. `set a 1`, `set b 2`, reopen: `get("a")` is `Some("1")` and `store.kv` is
   `"a\t1\nb\t2\n"`.
2. `remove a` returns true and survives a reopen; removing an absent key
   returns false.
3. Setting key `"x\ny"` gives `InvalidInput` and leaves `store.kv`
   unchanged.
4. A `store.kv` holding `"oops\n"` makes `open` fail with `InvalidData`.
5. With a garbage `store.kv.tmp` present, `open` succeeds, and after a
   `set` no `store.kv.tmp` remains.

### B10. heartbeat (library)

Probe something at a fixed rate until it keeps failing.

- `watch` calls `probe` at the start and then at each tick, every
  `interval` after the start: it sleeps until the next tick, so the time a
  probe takes does not shift later ticks. A tick that passed while a probe
  ran is skipped. It stops after `misses` consecutive `false` results;
  `misses` 0 panics.
- It returns `Report { calls: u32, last_ok: Option<u32>, elapsed: Duration
  }` (fields `pub`, deriving `Debug` and `PartialEq`): the number of probe
  calls, the 0-based index of the last call that returned `true`, and the
  time from the start to the last call.
- Tide: `watch(clock: &Clock, interval: Duration, misses: u32, probe: impl
  FnMut() -> bool) -> Result<Report, Cancelled>`.
- Rust: `watch(interval: Duration, misses: u32, probe: impl FnMut() ->
  bool) -> Report`.

Checked cases (Tide: exact virtual times; Rust: `elapsed` at least the
stated time and less than it plus 150 ms):

1. Interval 100 ms, `misses` 2, results `true, true, false, true, false,
   false`: 6 calls, `last_ok` `Some(3)`, `elapsed` 500 ms.
2. Interval 100 ms, `misses` 1, a first probe that takes 250 ms and returns
   `true`, then `false`: 2 calls, `last_ok` `Some(0)`, `elapsed` 300 ms.
3. `misses` 1 and an always-false probe: 1 call, `last_ok` `None`,
   `elapsed` zero.
4. `misses` 0 panics.

## Band C: concurrency, parallelism, contracts

### C1. histogram (library)

A histogram computed in parallel.

- `histogram(values: &[u32], edges: &[u32]) -> Vec<u64>`: bucket `i` counts
  the values `v` with `edges[i] <= v < edges[i + 1]`; other values are
  ignored. `edges` must have at least two entries and increase strictly,
  or it panics. The values are processed in parallel, in chunks of at most
  4096.
- Tide: Use `par_map`, `par_for`, or `join` (structure: the source
  contains `par_map(`, `par_for(`, or `std::par`).
- Rust: Use scoped threads (structure: the source contains
  `thread::scope(`).

Checked cases:

1. Values `[0, 1, 5, 9, 10, 99]` with edges `[0, 5, 10]` give `[2, 2]`.
2. One million values from a fixed xorshift sequence, with 17 random
   increasing edges, match a sequential count.
3. Empty values give zeros; edges `[5, 5]` and `[5]` panic.

### C2. psort (library)

A parallel merge sort.

- `sort_parallel(v: &mut [i64])` sorts ascending: while a slice is longer
  than 2048 elements, sort its two halves in parallel and merge them;
  shorter slices may use any sequential sort.
- `sort_by_key_parallel(v: &mut [T], key: F)`, with `key: Fn(&T) -> K` and
  `K: Ord`, sorts stably by key, with the same parallel structure. Choose
  the bounds the parallel calls need.
- Tide: Use `join` and not `par_sort` (structure: the source contains
  `std::par` and not `par_sort`).
- Rust: Use scoped threads (structure: the source contains
  `thread::scope(`).

Checked cases:

1. One million values from a fixed xorshift sequence come out equal to
   `slice::sort`'s result.
2. Empty, one-element, and all-equal slices.
3. 100,000 pairs `(i % 7, i)` sorted by `.0` come out ordered by `.0`, then
   by `.1`.

### C3. kmerge (library)

Merge sorted streams that arrive over bounded channels.

- `merge_streams` starts one producer per input, each sending its items in
  order through its own bounded channel of `capacity`, and merges the
  channel heads in the calling task into one ascending `Vec`. `capacity` 0
  panics.
- An input that is not ascending stops its producer when it finds an item
  smaller than the one before it, and the merge fails with
  `MergeError::Unsorted { input, index }` (`input`'s index and the
  offending item's index). With several such inputs, the smallest `input`
  is reported. `MergeError` derives `Debug` and `PartialEq`.
- Tide: `merge_streams(tasks: &Tasks, inputs: &[Vec<i64>], capacity:
  usize) -> Result<Vec<i64>, MergeError>` (structure: the source contains
  `sync_channel(`).
- Rust: `merge_streams(inputs: &[Vec<i64>], capacity: usize) ->
  Result<Vec<i64>, MergeError>` (structure: the source contains
  `sync_channel(`).

Checked cases:

1. `[[1, 4, 9], [2, 3, 10], [], [5]]` with capacity 1 give
   `[1, 2, 3, 4, 5, 9, 10]`; no inputs give `[]`; duplicates are kept.
2. Eight inputs of 600 ascending items each, capacity 2, equal their
   sorted concatenation.
3. `[[1, 2], [3, 1], [5, 0]]` give `Unsorted { input: 1, index: 1 }`.
4. Capacity 0 panics.

### C4. reserve (library)

All-or-nothing stock reservations under concurrency.

- `Warehouse` keeps each item's stock in its own lock cell.
  `reserve(&self, order: &[(&str, u32)]) -> Result<(), ReserveError>`
  deducts every line of the order or none. Lines naming the same item add
  up. `ReserveError` derives `Debug` and `PartialEq` and has
  `UnknownItem(String)` and `Insufficient(String)`, naming the first
  failing item in order of first appearance in the order. `stock(&self,
  item: &str) -> Option<u32>`.
- `run_orders` reserves the orders concurrently, at most `workers` at once,
  and returns their results in order.
- Tide: `Warehouse::new(tasks: &Tasks, stock: &[(&str, u32)])` keeps each
  item in a `Shared` cell (structure: the source contains `Shared::new(`);
  `run_orders(tasks: &Tasks, w: &Warehouse, orders: &[Vec<(&str, u32)>],
  workers: usize) -> Vec<Result<(), ReserveError>>`.
- Rust: `Warehouse::new(stock: &[(&str, u32)])` keeps each item in a
  `Mutex` (structure: the source contains `Mutex::new(`);
  `run_orders(w: &Warehouse, orders: &[Vec<(&str, u32)>], workers: usize)
  -> Vec<Result<(), ReserveError>>`.

Checked cases (Tide: under 20 seeds):

1. Stock `a` 5, `b` 2: `[(a, 2), (b, 2)]` succeeds, leaving 3 and 0; then
   `[(a, 1), (b, 1)]` is `Insufficient("b")` and leaves `a` at 3.
2. With `a` at 3, `[(a, 2), (a, 2)]` is `Insufficient("a")`.
3. `[(z, 1)]` is `UnknownItem("z")`.
4. Stock `x` 600, `y` 1000, 1000 orders of `[(x, 1), (y, 1)]` with 8
   workers: exactly 600 succeed, the other 400 are `Insufficient("x")`, and
   the stock ends at 0 and 400.

### C5. deadline (library)

Run jobs concurrently and keep what finishes before a time limit.

- `run_batch` starts every job at once. Job `i` works for `jobs[i]` (it
  sleeps that long). It returns `Vec<Outcome>` in job order, with
  `Outcome::Done` for a job that finished and `Outcome::TimedOut` for one
  still working when `limit` passed. It returns as soon as every job is
  done or the limit has passed. `Outcome` derives `Debug` and `PartialEq`.
- Tide: `run_batch(tasks: &Tasks, clock: &Clock, jobs: &[Duration], limit:
  Duration) -> Vec<Outcome>`.
- Rust: `run_batch(jobs: &[Duration], limit: Duration) -> Vec<Outcome>`.

Checked cases (Tide: exact virtual time; Rust: real time with the slack
given):

1. Jobs `[10, 500, 30]` ms, limit 100 ms: `[Done, TimedOut, Done]`;
   elapsed 100 ms (Rust: at least 100 ms and under 300 ms).
2. Jobs `[5, 6]` ms, limit 1 s: `[Done, Done]`; elapsed 6 ms (Rust: at
   least 6 ms and under 500 ms).
3. No jobs: `[]`; elapsed zero (Rust: under 100 ms).

### C6. firstmatch (library)

Search several texts concurrently and stop at the first match.

- `find_any` searches each haystack in its own concurrent task and returns
  `Some((i, pos))` for a haystack `i` that contains `needle`, with `pos` the
  byte offset of its first occurrence there, or `None`. Once one search
  finds a match, the others stop early, checking for that at least every
  4096 bytes. An empty `needle` panics.
- Tide: `find_any(tasks: &Tasks, hays: &[String], needle: &str) ->
  Option<(usize, usize)>` (structure: the source contains `checkpoint(` or
  `.cancel(`).
- Rust: `find_any(hays: &[String], needle: &str) -> Option<(usize, usize)>`
  (structure: the source contains `AtomicBool`).

Checked cases:

1. One haystack of three contains the needle: its index and offset.
2. No haystack contains it: `None`; no haystacks: `None`.
3. Two contain it: either answer, each with its correct offset.
4. Eight 1 MB haystacks, the needle only in haystack 3 at offset 999,000:
   `Some((3, 999000))`.
5. An empty needle panics.

### C7. bstack (library)

A bounded stack with stated pre- and postconditions.

- `BoundedStack<T>`: `new(capacity: usize)` requires `capacity >= 1`;
  `push(&mut self, x: T)` requires the stack not to be full; `pop(&mut
  self) -> Option<T>`; `peek(&self) -> Option<&T>`; `len`, `capacity`,
  `is_empty`, `is_full`; `drain_top(&mut self, n: usize) -> Vec<T>`
  requires `n <= len` and returns the top `n` items, top first.
- `push` leaves `len` one higher; `pop` leaves it one lower when the stack
  was not empty and unchanged when it was.
- A violated requirement panics.
- Tide: State the requirements as `#[requires]` and the `push` and `pop`
  length facts as `#[ensures]` (structure: the source contains
  `#[requires(` at least twice and `#[ensures(` at least twice).
- Rust: Check the requirements with `assert!`.

Checked cases:

1. Push 1, 2, 3 into capacity 3; `is_full`; pop gives 3, 2, 1, then `None`.
2. A push onto a full stack panics; `new(0)` panics; `drain_top(3)` on two
   items panics.
3. `drain_top(2)` on `[a, b, c]` (pushed in that order) gives `[c, b]` and
   leaves `[a]`; `peek` is then `Some(&a)`.
4. A stack of `String` works the same.

### C8. partition (library)

In-place partition and selection with loop invariants.

- `partition(v: &mut [i32], pivot: i32) -> usize` reorders `v` in place so
  that every element less than `pivot` precedes every other, keeps the
  multiset of elements, and returns how many are less. It allocates
  nothing.
- `select_kth(v: &mut [i32], k: usize) -> i32` requires `k < v.len()` and
  returns the `k`-th smallest element (0-based) by repeated `partition`;
  a violated requirement panics.
- Tide: `partition` carries `#[ensures]` clauses stating the ordering and
  the result's bound, and its loop starts with `invariant!`;
  `select_kth` states its requirement with `#[requires]` (structure: the
  source contains `#[ensures(`, `invariant!(`, and `#[requires(`).
- Rust: Check `select_kth`'s requirement with `assert!`.

Checked cases:

1. `[3, 1, 4, 1, 5, 9, 2, 6]` with pivot 4 returns 4, the first four
   elements are below 4, and the multiset is unchanged.
2. All elements at least the pivot return 0; an empty slice returns 0.
3. `select_kth` on 10,000 values from a fixed xorshift sequence equals the
   sorted value at k = 0, 1, 4999, and 9999; with many duplicates too.
4. `k == v.len()` panics.

### C9. jobpool (library)

Run jobs that may fail or panic, isolated from each other.

- `steps(n: u64) -> Result<u32, TooLong>` counts the Collatz steps from `n`
  to 1 (`n / 2` when even, `3n + 1` when odd) with plain arithmetic, so
  an overflow panics. `n == 0` panics. More than 200 steps gives
  `Err(TooLong)`. `TooLong` derives `Debug` and `PartialEq`.
- `run_jobs` runs `steps` on every input concurrently, at most `workers` at
  once, and returns `Summary { ok: u64, done: u32, too_long: u32,
  panicked: u32 }` (fields `pub`, deriving `Debug` and `PartialEq`): the
  sum of the successful step counts, and the number of jobs that
  succeeded, returned `TooLong`, and panicked. A panicking job does not
  stop the others.
- Tide: `run_jobs(tasks: &Tasks, inputs: &[u64], workers: usize) ->
  Summary`.
- Rust: `run_jobs(inputs: &[u64], workers: usize) -> Summary`.

Checked cases:

1. `steps(1)` is 0, `steps(6)` is 8, `steps(27)` is 111, `steps(6171)` is
   `Err(TooLong)`.
2. `steps(0)` and `steps(u64::MAX)` panic.
3. Inputs `[1, 6, 27, 0, u64::MAX, 6171]` with 2 workers give `ok` 119,
   `done` 3, `too_long` 1, `panicked` 2.

### C10. linesrv (library)

A concurrent line server over TCP, and its client.

- `serve` accepts exactly `clients` connections and handles them
  concurrently. For each `\n`-terminated UTF-8 line it receives, it replies
  with the line's characters reversed and `\n`; the line `QUIT` closes that
  connection without a reply, as does end of input. It returns the number
  of lines answered.
- `exchange` connects to `addr`, sends each line and reads its reply, then
  sends `QUIT`, and returns the replies.
- Tide: `serve(tasks: &Tasks, listener: &TcpListener, clients: usize) ->
  io::Result<u64>` and `exchange(net: &Net, addr: &str, lines: &[&str]) ->
  io::Result<Vec<String>>`.
- Rust: `serve(listener: &TcpListener, clients: usize) -> io::Result<u64>`
  and `exchange(addr: &str, lines: &[&str]) -> io::Result<Vec<String>>`.

Checked cases (Tide: in a simulation, under 20 seeds; Rust: on 127.0.0.1
with port 0):

1. Three clients exchange `["abc", "héllo"]`, `["x"]`, and `[]` at once:
   their replies are `["cba", "olléh"]`, `["x"]`, and `[]`, and `serve`
   returns 3.
2. A first client sends one line and holds its connection open until a
   second client's `exchange` has returned: both finish, so the
   connections are handled concurrently.
