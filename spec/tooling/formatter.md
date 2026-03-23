# Ferox Tooling Reference: Formatter

This document defines the canonical formatter and formatting-related tooling guarantees.

## Formatting Policy

Ferox defines a **single canonical source format**.

Canonical formatting is part of the standard workflow, but formatting is not a prerequisite for executing otherwise valid code.

If a file is syntactically valid but **not canonically formatted**:

* `ferox build`
* `ferox run`
* `ferox test`

must still operate on the program normally.

Formatting compliance is enforced by:

* `ferox fmt`
* `ferox fmt --check`
* lint or CI policy

Formatting is enforced by the official formatter, not by making ordinary execution commands reject valid code.

Example command:

ferox fmt

Goals:

* eliminate style ambiguity
* improve mechanically safer change
* reduce review noise
* stabilize refactoring and code generation
* ensure uniform codebases

## Canonical Formatting Rules

Ferox has one official formatter and one canonical formatting outcome for any supported source form.

Core rule:

* if two source files have the same syntax tree and both are fully formatable, the official formatter should produce the same formatted result

Initial canonical rules:

* declarations appear in a normalized block-based layout rather than a compact one-line style for multi-part constructs
* opening braces stay on the same line as the declaration or control form they belong to
* closing braces appear on their own line aligned with the start of the construct
* statements end with `;` where required by the language and are emitted one per line
* a single ASCII space is used around infix operators and after commas
* there is no alignment-by-columns formatting
* unnecessary blank lines are removed, while top-level items are separated clearly
* the formatter prefers stable readability over preserving user-written local spacing quirks

Example intent:

```ferox
fn add(a: Int, b: Int) -> Int {
    a + b
}

let result = match value {
    some(inner) => inner,
    none => fallback,
};
```

### Design Intent

Canonical formatting should make Ferox source predictable to read and cheap to rewrite automatically.

Goals:

* produce one obvious formatting result
* keep declarations and control flow visually regular
* avoid style choices that depend on manual alignment
* support formatter-driven refactors and code generation

This is part of the broader Ferox goal of reducing semantic entropy and making automated change safer.

## Import Ordering And Grouping

Imports have one canonical ordering and grouping style.

Initial rules:

* all `use` declarations in a file are kept in one contiguous import block near the top of the file after any file-level doc comments
* import declarations are sorted lexicographically by their full canonical path
* grouped imports are sorted lexicographically inside their braces
* aliased imports are ordered by their underlying import path, not by the alias
* the formatter does not create semantic subgroups separated by blank lines in v1

Examples:

```ferox
use core.hash.HashMap as Map;
use core.io.Writer;
use net.http;
use net.http.{Client, Request, Response};
```

### Canonical Grouping

Rules:

* grouped imports are preferred when multiple items are imported from the same module path
* duplicate imports are collapsed into one canonical form where possible
* wildcard imports remain forbidden and therefore never appear in formatted output
* the formatter may normalize repeated imports from the same module into a grouped import when that does not change meaning

### Design Intent

Import layout should be deterministic without requiring style debates or semantic guesswork.

Goals:

* make import diffs stable
* keep import scanning fast for humans and tools
* avoid subjective grouping conventions in v1
* preserve a single obvious formatter result

## Line Breaking, Indentation, And Trailing Commas

The formatter uses a small set of fixed layout rules for multiline constructs.

### Indentation

Initial rules:

* one indentation level is `4` spaces
* tabs are not used for indentation in canonical output
* nested blocks and multiline constructs increase indentation by exactly one level

### Line Breaking

Initial direction:

* short single-line constructs may remain on one line when they fit the formatter's basic readability rules
* multiline blocks place each contained statement or arm on its own line
* multiline parameter lists, argument lists, tuple literals, array literals, map literals, and grouped imports place one item per line
* closing delimiters align with the start of the construct that opened them
* line breaking is structure-driven rather than dependent on manual alignment choices

Example intent:

```ferox
fn build(
    host: String,
    port: Int,
    secure: Bool,
) -> Config {
    Config {
        host: host,
        port: port,
        secure: secure,
    }
}
```

### Trailing Commas

Rules:

* multiline comma-separated constructs use trailing commas
* single-line comma-separated constructs do not use trailing commas unless required by the syntax of a singleton tuple or similar construct
* trailing commas are part of the canonical formatter output for multiline lists because they stabilize diffs and simplify automated rewriting

### Design Intent

These rules should make multiline Ferox code predictable before any sophisticated width-based formatting policy exists.

Goals:

* produce stable diffs for lists and blocks
* keep indentation mechanical and easy to parse
* favor structure over clever width-sensitive layout
* support code generation and refactoring tools without custom style logic

## Formatter Behavior For Partially Invalid Files

The formatter may attempt limited recovery on partially invalid files, but correctness and trust take priority over aggressive rewriting.

Core rule:

* `ferox fmt` must not silently rewrite a file when the parser cannot establish a sufficiently reliable structural understanding of the source

Initial direction:

* if a file is fully parseable, the formatter produces canonical output
* if a file has minor recoverable syntax damage, the formatter may format only the confidently parsed regions and emit diagnostics for the invalid regions
* if structural recovery is too uncertain, the formatter fails with diagnostics and leaves the file unchanged

### Recovery Expectations

Examples of potentially recoverable situations:

* an incomplete trailing expression at end of file
* a missing closing delimiter in a region where the parser can still identify surrounding top-level structure
* a damaged statement inside an otherwise recognizable block

Rules:

* recovery should be local and conservative
* the formatter must not invent or guess missing code
* any partially formatted result must preserve the original source for regions the formatter could not confidently normalize
* diagnostics should identify why full formatting was not possible

### Design Intent

The formatter should still be useful during editing without becoming untrustworthy.

Goals:

* support editor workflows on in-progress files
* preserve confidence that formatted code still reflects the user's program
* avoid destructive rewrites when syntax is badly damaged
* keep formatting failure modes explicit and diagnosable

## Formatter Stability Guarantees

The formatter is part of the compatibility surface of the official toolchain.

Core rule:

* formatter output should not change arbitrarily between routine toolchain updates

Initial stability policy:

* patch releases must not intentionally change canonical formatting for already supported syntax
* minor releases should avoid formatter churn unless the change fixes a correctness issue, supports newly added syntax, or resolves a clearly justified formatting defect
* large formatting changes should be rare and explicitly called out in release notes

### Compatibility Expectations

Rules:

* formatting changes that would rewrite large volumes of existing code should not appear silently in ordinary maintenance updates
* newly introduced syntax may define new canonical formatting without violating stability for older syntax
* formatter changes tied to language-edition boundaries may be allowed when explicitly documented
* `ferox fmt --check` should remain a reliable CI signal across a pinned toolchain version

### Design Intent

Formatter stability should reduce repository churn and make the official style a dependable automation target.

Goals:

* keep formatting predictable across upgrades
* avoid unnecessary mass rewrites
* preserve trust in formatter-driven workflows and CI
* allow carefully justified evolution without treating formatting as frozen forever
