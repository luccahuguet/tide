# Notes

## Sources used

1. `reference.md`, read in full first.
2. `tasks.md`, read in full.
3. `reference-companion.md`. I listed its headings with grep, then read only these sections:
   - §15 (15.1–15.4), the example programs. 15.1 shows how to test a CLI when tests cannot call `main` (dedupe). 15.2 shows `sim.world.http`, `Reply`, and deadline handling (retry, longest_lines). 15.4 shows a durable-write pattern and a node crash test (backup).
   - §16, the readings taken where the documents disagree (relative paths, `with_all`).
   - §17, simulation faults, nodes, and crashes. I needed `sim.node`, `node.start`, `sim.faults.at`, `Fault::crash`, `sim.run_for`, and `node.fs()` for backup's crash test.
   - Appendix A, to confirm nothing I used is v1 or later (e.g. `par_reduce`, closure contracts).

   I did not read §14 (habit tables).

## Cross-cutting

1. **`Rules<T>` needs a lifetime parameter.** The predicates are boxed closures that may borrow caller locals, so they have type `Box<dyn Fn(&T) -> bool + 'a>` (3.4: a bare `Box<dyn ..>` in a field means `'static`). I wrote `Rules<'a, T>`. `at_least` bounds `T: 'a` because the closure owns a `T`. Affects: rules.
2. **Where the scope lifetime comes from.** I read `scope`'s body as `for<'s> FnOnce(Scope<'s, 'env, E>)`, with the implied bound `'env: 's` from `Scope`'s well-formedness, as in `std::thread::scope`. The 7.1 signature does not write that bound. 7.1's prose ("children may borrow anything that outlives the `scope` call"; `cfg: &'s Config` for a passed-down scope) implies it. I relied on it for `&'d str` results flowing out of children (longest_lines), `lines: &'s [String]` in `spawn_parser` (pipeline), and `&Tally` captures (tally).
3. **How a scope picks the error it reports.** In every scope I wrote, the body fails through `join()?` (or `collect` over joins) after a child fails. Per 7.1, the result is then the earliest-spawned *failing* child's error, not the body's `Cancelled`. Tests rely on that: `Nul(1)`, `BadLine(3)`, `Overflow`. Affects: longest_lines, pipeline.
4. **Channel errors versus the echo rule.** 7.1 defines an echo only as an error returned *after seeing a cancellation*. Two things are open: whether a send to a dropped receiver counts as an echo, and whether `send` on a non-full `sync_channel` is a checkpoint (7.3 says only "on a full `sync_channel`"). My reading:
   - The parser treats any send error as "downstream is gone", stops, and returns `Ok(sent)`. It then never competes with the summer's `Overflow` as the earliest-spawned failing child.
   - The summer splits `recv` errors with `e.is_cancelled()`. I assumed `RecvError` has that method, from 7.2's "channel errors whose `is_cancelled()` is true". A cancellation becomes an echo `Err`; a disconnect is end of input.

   Affects: pipeline.
5. **Inferring closure return types around `?`.** Inside `Shared::with`/`with2` and a scope body whose result is the operand of `?`, the closure's error type is constrained only later. The reference's `transfer` example relies on that inference. I annotated those closures (`|m| -> Result<(), TallyError> { .. }`) to be safe. Scopes that are a function's tail expression get their types from the return type, so I left them unannotated. Affects: tally.
6. **Can `+ use<'a>` leave out the elided `&self` lifetime?** I wrote `numbered`/`containing` as `impl Iterator<..> + use<'a>` / `use<'a, 'w>`, so the iterators borrow the text rather than the `Lines` value. The reference mentions `+ use<'a>` (3.5) but gives no example on a method. For `positions` I returned a named `Positions<'h>` struct, so the `Needle`-drop requirement does not depend on capture rules. Affects: scan.
7. **Reading `Stdin`.** 6.1 lists `read_to_string` and `read_to_end` "as io::Stdin's". In Rust those are `Read` trait methods, which would need `use std::io::Read`. If Tide's capability makes them inherent, that import is an unused-import error (`FX0913`). I used `read_line` in a loop instead, because it is inherent in Rust, so no import is in question. Affects: dedupe.
8. **"The whole program" in a test.** Tests cannot call `main` (`FX0506`) or name `Root` outside it. Following 15.1, I put everything except printing and the exit code into `run(env, fs, stdin)`. The sim tests set `sim.world.args`/`stdin`/`file` and call `run` with the sim capabilities. Exit codes are tested through `CliError::exit_code()`. I assumed `sim.world.stdin` accepts a `&str` (the table says `stdin(bytes)`). Affects: dedupe.
9. **"Only able to read below `src` and write below `dst`" with a `fs: &Fs` signature.** `backup` attenuates at once: `fs.dir(src, Mode::Read)` comes first, so a missing `src` fails before anything is written. The copying code holds only the read-only `Fs` below `src` and the read-write `Fs` below `dst`. The full `fs` is used only to create `dst` and its missing ancestors and to sync their entries. A stricter reading would need a different signature. Affects: backup.
10. **Directory listing APIs.** `Fs::read_dir` is "as std::fs". I assumed `DirEntry::file_name()` is an admitted pure accessor. I avoided `DirEntry::metadata`/`file_type` because 6.9 bans I/O methods on std types, and used `from.metadata(name)?.is_file()` instead. That follows symlinks; a symlink that leaves `src` should fail with `PermissionDenied`. Non-UTF-8 names are skipped. Affects: backup.
11. **Atomic replace and durability.** The order is: `write_synced` a temporary, `rename` it over the target, then `sync_dir(".")` once at the end. I assumed the simulator treats an unsynced rename as either kept or lost, never torn, since 17 says entries "may be lost or torn" without detail. Each newly created ancestor of `dst` gets its entry synced in its parent. The temporary name is `<name>.backup-tmp`, with `~` appended while it collides with a name being copied. An unrelated existing file of that name in `dst` would be overwritten, and a crash can leave a temporary behind. Affects: backup.
12. **Contract clauses calling my own methods.** A `pub` fn's contract may call only `pub` fns (10.1), and the reference says nothing about private fields in clauses. So the ring clauses use `self.len()` and `self.capacity()`, not fields. That makes them *sampled* rather than *checked*, which still runs on every call under `tide test`. `is_full`'s `ensures` calls `len`, whose own `ensures` calls `capacity`: contracts nest, but nothing recurses. I avoided `a ==> b` where `b` could trap if evaluated eagerly; `pop` uses `old(self.len()).saturating_sub(1)`. Affects: ring.
13. **Where `invariant!` is checked in a `while let`.** For a loop whose condition has side effects (`while let Some(x) = self.pop()`), it is unclear whether "loop entry" falls before or after the condition. `drain_into` uses `for moved in 0..n`, where 10.1 pins the index semantics: `moved == n` at exit. Affects: ring.
14. **Generated contract tests and resource use.** `with_capacity` has an `ensures`, so `tide test` generates `n` from `n >= 1` with no upper bound. An eager `vec![0; n]` could abort on allocation. Storage therefore grows lazily up to `cap`. Affects: ring.
15. **Violation message.** `#[should_panic(expected = "requires violated")]` assumes the provisional message prefix in 10.1. Affects: ring.
16. **Traits on `Id<T>`.** `Handle(Id<Node>)` derives `Clone, Copy, PartialEq, Eq, Hash, Debug`. I assumed `Id<T>`'s impls do not bound `T`, since 3.6 lists them unconditionally. `Node` derives `Debug` anyway. Affects: deps.
17. **Retry timing.**
   - Per 6.7, `Timeout` counts as transient, so a request that times out at the overall deadline would look retryable. Before sleeping, the loop gives up with the last error if `now + delay` would not land strictly before the deadline. That also stops it from sleeping into a request that is sure to time out.
   - How many packets a GET costs in simulation is unspecified, so the "about `first_delay * 3`" test accepts 300–900 ms.
   - `drop_all()` might time out immediately or at the deadline; that test accepts either and checks only the kind and the time bound.
   - The `http` peer closure is not required to be `Send` (6.8 signature), so I used `Rc<Cell<u32>>` to count calls.

   Affects: retry.
18. **NUL documents in `total_words`.** The task's error bullet sits under both functions, so I applied the NUL rule to both. Affects: longest_lines.
19. **Longest line.** Length is measured in `char`s. An empty document returns `&doc[..0]`, so even `""` is a slice of that document. Affects: longest_lines.
20. **The canon drop test.** "A word taken from a second text, which lives in an inner block that also holds the Dict" reads two ways. My test builds the second text and the `Dict` in one inner block and calls `drop(d)` before using the results, with that text still alive. The `rewrite` test also covers the outer-text, inner-`Dict` arrangement. Affects: canon.
21. **Small readings.** These are documented in the code:
   - An empty `Needle` matches nowhere.
   - `Lines` strips `\r` only before `\n`, so a bare trailing `\r` stays.
   - tally treats every non-ASCII-alphanumeric character as a separator, including non-ASCII letters.
   - dedupe treats any argument starting with `-` (other than `--count`) as an unknown flag, and repeated `--count` is allowed.

   Affects: scan, tally, dedupe.

## Per program

- **canon** (high). `canonical_or`/`rewrite` use `<'r> .. where 't: 'r` (3.2's "own lifetime" form). The only subtle point is lifetime subtyping in `self.get(word).unwrap_or(word)`, which is ordinary Rust.
- **rules** (high). See item 1. The `one_of` test declares the `Vec` before the `Rules`, so drop order is fine. Declaring it after would fail drop check, as in Rust.
- **scan** (high). Horspool traces by hand: `"aaaa"` in nine `a`s gives `[0, 4]`; `"abc"` in `"xxabcabcab"` gives `[2, 5]`; `"one"` in `"one two one"` gives `[0, 8]`. The remaining risk is item 6.
- **longest_lines** (high). `#[requires(max_parallel > 0)]` guards against `limit(0)`. The `limit` temporary lives through the `scope` call. Risks are items 2–3.
- **pipeline** (medium). The logic is sound, but the error-reporting tests depend on item 4's reading of channel errors and checkpoints, and on `RecvError::is_cancelled()`.
- **retry** (medium). The control flow is simple. Uncertain: simulated network timing (item 17), and whether `Instant::checked_add` is on the admission allowlist.
- **backup** (medium). Risks are items 9–11. The extra crash test (`seeds = 200`) uses the provisional §17 node API, and assumes a node's disk is separate from `sim.fs` and that node records survive the crash.
- **ring** (medium). The code and hand traces are solid (wraparound; extend/drain with head = 1). The contract machinery is where the uncertainty lies: items 12–15, and how `#[requires]` on a property test bounds `cap` to 1..=8.
- **deps** (high). Hand traces: the diamond gives `base, left, right, app`; the cycle walk gives `[solo]`, then `[x, y, z]` after `solo` is removed; removing `z` gives `y, x`. Stale handles return `Stale`/`None` thanks to arena generations. Item 16 is the only open point.
- **tally** (high). `par_map` closure and element bounds hold (`String: Pure + Sync`, `Counts: Send`). `Shared` closures capture only `Pure` data, and every overflow check runs before the first write. Risk: item 5, now annotated.
- **dedupe** (medium). The logic tests are certain. The sim tests depend on items 7–8 and on `sim.world.args` accepting an array of `&str`.
