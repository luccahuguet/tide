# Contracts: one spec, checked at run time and turned into tests

**Feature:** executable contracts and the tests generated from them
(`design/01-design.md` D10; `design/04-contracts.md`). `#[requires]` and
`#[ensures]` state a function's contract once. The test build checks the
clauses, and it also generates inputs that meet the `requires` and checks the
`ensures` on each, with no hand-written test.

| File | What it is | Declared outcome |
| --- | --- | --- |
| `sorted_insert.rs` | `insert_sorted(xs, x)`: given a sorted vector, it leaves it sorted, one element longer, holding `x`. `main` inserts two values. | runs and prints `[3, 7, 9, 14, 20]`; its generated contract test passes |
| `off_by_one.rs` | The same function with an off-by-one loop bound: an `x` larger than every element goes in before the last one. | its generated contract test fails with `ensures violated: xs.is_sorted()` |

Neither file has a hand-written test. In `off_by_one.rs`, the failing test
is the one the test build generated from the contract; it reports the
violated clause, the seed, and the generated input that broke it.

## What it shows

- The contract is code: each clause is an ordinary Rust expression, plus
  `old(..)` for entry values, checked where the function returns.
- The bug is caught from the contract alone: no test in either file calls
  `insert_sorted`. The generated test only feeds it sorted vectors, as the
  `requires` demands, and shrinks the failure to a one-element vector.
- The same contract holds for the fixed function, whose generated test
  passes.

## What the Rust version would need

Stable Rust has no contract attributes. Nightly's experimental `contracts`
feature and crates such as `contracts` check clauses at run time, but neither
generates tests from them. To get the same checks:

- state the clauses once, with a crate such as `contracts` (its `requires`,
  `ensures`, and `old(..)`) or as hand-written assertions in the function;
- write a property test with a crate such as `proptest` or `quickcheck` (a
  dev-dependency) that generates sorted vectors, or filters out unsorted
  ones, and calls the function. A violated clause panics, which fails the
  property and shrinks its input, so the test need not restate the clauses.

What Tide adds is that second step: the test build generates that test, and
its inputs follow the `requires`, for every function with an `ensures` whose
parameters it can generate.

## Run it

`cargo test --workspace` checks every declared outcome, through the examples
runner in `crates/tide-admit/tests/examples.rs`; to run only the examples:

```sh
cargo test -p tide-admit --test examples
```
