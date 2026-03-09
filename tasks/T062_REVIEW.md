# T062 Review Notes

Review date:

* March 8, 2026

Status:

* completed
* first decision pass completed and reflected into the live spec

## Decision

Current recommendation:

* **settle method receivers in v1 as three explicit forms only**

Recommended receiver forms:

* `self`
* `&self`
* `&mut self`

No additional receiver forms should exist in v1.

## Why

### 1. The current spec is too vague here

The spec currently allows `self` syntax but defers receiver ownership and borrowing variants.

That is not a detail. It affects:

* method ergonomics
* trait signatures
* API design norms
* ownership behavior
* mutation behavior
* diagnostics

### 2. Ferox does not need novelty here

Receiver forms are not where Ferox needs to be innovative.

The language already has enough design risk in ownership, borrowing, and concurrency. A small explicit receiver model is the pragmatic choice.

### 3. Three explicit forms are enough for v1

These forms cover the important cases:

* `self` for ownership-consuming methods
* `&self` for read-only access
* `&mut self` for in-place mutation under exclusivity

That gives Ferox a complete and teachable receiver story without adding more special cases.

## Recommended V1 Receiver Model

### Trait methods

Trait methods may use:

* `self`
* `&self`
* `&mut self`

Example:

```ferox
trait Display {
    fn render(&self) -> String;
}

trait Builder {
    fn push(&mut self, item: Item) -> Unit;
}

trait IntoBytes {
    fn into_bytes(self) -> Bytes;
}
```

### Implementation methods

Impl methods use the same three receiver forms.

Associated functions without receivers remain separate:

```ferox
impl Parser for JsonParser {
    fn parse(input: String) -> Result[Value, ParseError] {
        ...
    }
}
```

### Non-goals for v1

Do not support:

* arbitrary typed receivers beyond the three explicit forms
* implicit receiver coercions
* special ownership-smart receiver forms
* advanced receiver polymorphism

## Why This Is The Right Tradeoff

This model:

* fits the ownership system directly
* keeps mutation visible in method signatures
* avoids inventing a new method-receiver theory
* is small enough for clear diagnostics and implementation

## Consequences For The Spec

If adopted, this decision implies follow-up edits to:

* `spec/LANGUAGE.md`
  * replace the current deferred receiver language with concrete rules
  * add examples for `self`, `&self`, and `&mut self`
  * clarify how trait methods and impl methods must match receiver form exactly
* later method-call syntax sections, if expanded
  * ensure method-call lowering respects the explicit receiver model

## Open Questions

This review does **not** yet settle:

* whether Ferox should treat methods as pure syntax sugar over ordinary function calls in the spec text
* how trait objects, if they ever exist, would interact with receiver forms
* whether any additional receiver conveniences belong in post-v1 design

Those are later questions. They should not block a small concrete v1 receiver model.

## Current Recommendation

For the adversarial spec pass:

* replace the current deferred receiver semantics with the three explicit forms above
* do not expand the receiver surface further in v1
