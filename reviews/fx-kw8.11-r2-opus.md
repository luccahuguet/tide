# Tide review, fx-kw8.11 round 2 (Opus): converge confirmation round

Candidate: branch fx-kw8.11 at 9b573c4 on base 7d97833, diff SHA-256
`6bfd08a1f9fed21eb94ddb2c1a8017a1b4fb952a68eebd141455450202f09e82`.

Model: claude-opus-5-5 · Date: 2026-11-03

## Verdict

Clean: no blocking findings. Every round-1 finding is fixed correctly
(Opus B1-B3 and N1-N5; Codex 1-5). The fixes introduce no regression. Two
nits remain, and neither blocks.

## Findings

N1 (nit). `shown` (diff.rs:99-109) escapes only `char::is_control`
(category Cc). U+2028/U+2029 (Zl/Zp) and the bidi controls
(U+202A-U+202E, U+2066-U+2069, U+200E/F) pass through raw. Failing
input: an item named `"lib::f   - lib::g (function)‮"` is
written to stdout as the raw bytes e2 80 a8 and e2 80 ae. Viewers that
apply UAX #14 (browsers, some editors) treat U+2028 as a mandatory line
break, so the doc claim "none can forge a line" (diff.rs:10-11) is wider
than what the code guarantees. Terminals tested in practice keep it on
one line, and the writer never emits these characters. Fix: escape those
code points as well, e.g. `c.is_control() || matches!(c, '\u{2028}' |
'\u{2029}' | '\u{200e}' | '\u{200f}' | '\u{202a}'..='\u{202e}' |
'\u{2066}'..='\u{2069}')`, or narrow the doc claim to control characters.

N2 (nit). README.md:70 is 100 columns. It was left unwrapped after the
"builds the driver" edit; the neighbouring prose lines are at most 86.
Fix: rewrap the paragraph.

Not a finding: kinds are joined with ", " without quoting, so a forged
single kind `"fs, net"` prints the same as the two kinds `fs` and `net`.
Real kinds such as `opaque(Foo<A, B>)` also contain ", ", but they cannot
collide with any set the writer produces, and the ambiguity stays within
one line. Escaping kinds alongside N1 would close it if that is wanted.

## Checked

- `cargo test --workspace` (fresh build of 9b573c4): exit 0. All three
  diff unit tests and `the_trust_diff_reports_authority_and_hole_changes`
  pass. `conformance/run.sh`: exit 0, 190 PASS, no FAIL. Worktree clean.
- Round-1 failing inputs, re-run with the r1 probes and the built binary:
  - 10k and 100k nested arrays and 20k nested objects: exit 2, "nesting
    too deep".
  - Duplicate `format`, `items`, `item`, and authority path keys, an
    escaped duplicate (`"item"`), and a duplicate in a nested
    unknown object: exit 2, "a key given twice".
  - Counts `007` and `00`: exit 2, "a leading zero". `\u+041`, `\u-041`,
    `\u 041`, and a non-ASCII digit: exit 2.
  - Escaped surrogate pair vs raw UTF-8: same item, and the unit test
    asserts an empty diff. A lone high surrogate, a lone low, high+high,
    high+BMP, high+`\n`, and high+`\\u..`: exit 2.
  - Empty kinds: exit 2. A name with a newline prints escaped on one line.
- Depth: 15 and 16 containers are accepted and 17 is rejected; the
  artifact needs 5 (top, items, item, authority, kinds), as the comment
  says. Errors propagate and drop the reader, so the depth left
  undecremented on an error path is harmless.
- Escapes at the buffer end (`\`, `\u00`, `\ud801\u`, `\ud801\udc0`,
  `\ud801\`, `\ud801`): all exit 2 without a panic. `get(..)` bounds
  every slice.
- `shown`/`shown_path`: ESC, DEL, NEL, NUL, `\r`, `\t`, and `\` are
  escaped. The paths `(value)`, `a b`, `"`, `\`, and `p\0` are quoted and
  stay unambiguous: plain paths cannot contain `"`. A non-ASCII
  alphanumeric path (`é_ü`) stays plain, which is harmless.
- No other leftovers: the per-path duplicate check is gone, unit tests
  parse in memory, and this run created no new `/tmp/tide-diff-*`
  directories (the 33 present date from round 1). README now lists
  added and removed items and names the driver.
- Rebuttal accepted: with the limit, recursion is at most 16 frames.
  Without it, the 10k-deep unit test would overflow the 2 MB test-thread
  stack and abort the test binary, so a regression cannot pass silently.
- Scaling is linear: 100k, 200k, and 400k authority keys take 1.3, 2.6,
  and 5.1 s (debug build).
