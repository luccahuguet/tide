# Tide review, fx-kw8.11 round 1 (Opus): converge review round

Candidate: branch fx-kw8.11 at 37757e6 on base 7d97833, diff SHA-256
`347ea1c83dc9d0576e9682b51a028f86250a2cea1c50a625e03516625a50b22b`.

Model: claude-opus-5-5 · Date: 2026-11-03

## Verdict

Not clean. The comparison logic is correct and meets the bead's Acceptance.
The JSON reader breaks the contract's "exit 2 on malformed input or a
duplicate key" in three verified ways (B1-B3). Each fix is in diff.rs, which
is in the v0 tier this change edits, so all three block.

## Findings

B1 (blocking). Deep nesting crashes the process instead of exiting 2.
`Reader::value` (diff.rs:225-283) recurses once per `[` or `{` and has no
depth bound. Failing input: an artifact whose `items` is 10,000 nested
`[` then 10,000 `]`, about 20 KB. The debug build aborts with "thread
'main' has overflowed its stack" (SIGABRT, exit 134); 6,000 levels still
exit 2. Fix: add a depth counter to `value` and reject anything deeper
than the schema needs. The deepest real path is object, array, object,
object, array, string, so a limit such as 16 is ample. Add one row to
`rejects_what_is_not_an_artifact_of_this_format` for it.

B2 (blocking). Duplicate JSON object keys are accepted, and the first one
silently wins. `field` (diff.rs:154-160) returns the first match, and the
object branch (diff.rs:256-279) never checks for repeats. Failing inputs,
all exit 0:
`{"format":"tide-interface/1","items":[],"holes":[],"items":[{"item":
"lib::x","kind":"function","authority":{"":["net"]}}]}` gives an empty
report and hides the second `items`.
`{"format":"tide-interface/1","format":"tide-interface/9",...}` passes the
format check.
`{"item":"lib::f","item":"lib::evil",...}` is read as `lib::f`.
The bead's contract says a duplicate key exits 2, and a reader that keeps
the last key (jq, for example) would see different authority. Fix: in the
object branch, keep the keys already seen (a `BTreeSet`, so a hostile wide
object stays O(n log n)) and return `expected unique key` on a repeat. That
check makes the per-path duplicate check at diff.rs:130-132 unreachable, so
delete it.

B3 (blocking). Malformed escapes and numbers are accepted. At diff.rs:310,
`u32::from_str_radix` accepts a leading `+`, so `"lib::\u+041"` decodes to
`lib::A` and exits 0. At diff.rs:229-238, `"count": 007` is accepted as 7,
though JSON forbids leading zeros. Both are "malformed JSON", which the
contract says exits 2. Fix: require four ASCII hex digits
(`h.iter().all(u8::is_ascii_hexdigit)`) before parsing, and reject a number
that has a leading `0` followed by more digits.

N1 (nit). Names containing control characters are printed raw, so a forged
artifact can fake report lines. An item name of
`lib::f (function)\n      + app: net\n  ~ lib::g` prints as three
plausible report lines. The writer never produces such names, since rustc
paths, field names, and kinds contain no newlines. The unit test at
diff.rs:410 asserts that the raw newline appears in the output. Suggest
escaping control characters when displaying names (for example, reuse
`interface::string` without the quotes) and asserting the escaped form.

N2 (nit). An empty kinds array gives a bare change line. With base
`"authority":{}` and head `"authority":{"x":[]}` the report is
`~ lib::f (struct)`, with no widened or narrowed marker and no path lines.
The writer never emits an empty set (interface.rs:264, `entry().or_default()
.insert`). Suggest rejecting empty kind arrays, or skipping them when
reading.

N3 (nit). The unit tests leave temporary directories behind. `compare`
(diff.rs:339-348) creates `$TMPDIR/tide-diff-<pid>-<n>` and never removes
it; each `cargo test` run leaves 11 directories, and 33 were in /tmp after
three runs. Suggest removing the directory after `diff` returns, or
splitting `diff` into reading and comparing so the tests compare parsed
`Artifact`s without touching the filesystem.

N4 (nit). Valid JSON surrogate pairs are rejected. `"😀"` fails
with "a bad `\u` escape" because `char::from_u32` refuses surrogates. The
writer emits non-ASCII text as raw UTF-8, so its own output is unaffected.
Only the doc comment's "the JSON the artifact uses" needs to say so, if
anything.

N5 (nit). README.md:65-69 says the diff lists "the items whose declared
authority widened or narrowed" but not the items added or removed, which it
also lists. The sentences inserted before "The root Cargo workspace builds
it" also make "it" read as the diff rather than the driver. Suggest "items
added, removed, or whose declared authority widened or narrowed", and
naming the driver in the next sentence.

## Checked

- `cargo test --workspace` in the worktree: exit 0. Both diff unit tests
  and `the_trust_diff_reports_authority_and_hole_changes` pass.
- `conformance/run.sh`: exit 0, no FAIL lines.
- The Acceptance is met by the driver test, which runs over real admitted
  artifacts: the Net reached through a private field is reported on Db, on
  App as `db.net`, and on handle as `app.db.net`; a new `unsafe` block
  appears as holes; a body-only edit gives an empty report.
- The README, the module doc, and the report header make no claim about
  correct or malicious behavior, and no v1 diff is included.
- The comparison logic is correct. Item and hole keys are merged from both
  sides; changes per path are `-` and `+` set differences; the widened and
  narrowed markers are right; the empty path displays as `(value)`;
  identical artifacts give an empty report.
- Argument handling: `--tide-diff` with one argument, or with three paths,
  falls through to `Invocation::parse` and exits 2 ("unsupported
  argument"). A missing file, a BOM, a wrong format, and a duplicate item
  all exit 2. Other modes are unaffected.
- Ownership: the reader reuses `FORMAT` and `Authority` from interface.rs,
  and no other JSON reader exists in crates/. Field names are mirrored
  between the writer and the reader, and the driver test round-trips them.
- Large inputs: 200,000 items run in 2.6 s in the debug build, and an
  object with 200,000 keys in 0.1 s. Nothing grew quadratically.
- The worktree stayed clean; probes are under scratchpad/k11-r1.
