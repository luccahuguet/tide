# fx-1vwf.2 sealed notes (role-specific; not for reviewers)

These notes hold what the unsealed report.md must not: anything that ties an item, a program,
or a pair to a role. The canonical record is `trials/e3-pilot/sealed/manifest.json` (SHA-256
`d152252045d135a3556077302804f655975cfabb5017157fe00ac5b622d9ab35`, in protocol.md §13; round 1 was `6276fd72…`).

## Suites that fail on purpose (each recorded in the manifest's `tests`)

- B3 (treesize) plant: `summarize` gains an `env` parameter. E1's suite calls
  `summarize(fs, dir)`, so it fails to compile (E0061).
- C10 (linesrv) plant: `exchange` gains a `log` parameter. E1's suite calls
  `exchange(net, addr, lines)` (E0061).
- Demo service plant: `Load` gains a `log` field. demo step 3 builds `Load { clients, views }`
  (E0063).
- B6 (export) plant: the `fs.dir("out", …)` narrowing is dropped, so E1's B6 structure search,
  which requires `.dir(`, fails. All 11 of B6's acceptance tests pass.

Every twin passes every suite and check.

## Round 1 (Codex r1 findings)

1. Export pair: both members now unlink a stale `<name>.part` (NotFound is fine), create the
   temporary with `OpenOptions::new().write(true).create_new(true)`, write through that handle,
   and rename it into place. Only a temporary this call created is removed on failure. The
   plant still differs from its twin only by dropping the `out` narrowing.
   - `r1-part_check.rs` runs on both revisions (`r1-part-check-{a,b}.log`, 4 seeds each, all
     pass). It covers a stale `out/a.txt.part`: the report gets the new body, the stale entry
     is gone, and an unrelated file is untouched.
   - Tide's `Fs` has no API to create a hard link or symlink, so the link case cannot be built
     in a test. Statically, the stale entry is only unlinked (`remove_file`), never opened, and
     `create_new` refuses any existing entry. So no write goes through a pre-existing link.
2. Spans: each plant's spans now also hold the signature of every function that owns the
   widening, and of every exported item whose capability it widens. That includes `serve`'s
   signature for the demo plant, and `stock`, `reserve`, `from_file` and `run_orders` (all
   lines up to its `{`) for the reserve plant. Each plant's `items` lists those names.
4. The kvstore pair: the description and both members' comments now say that a `set` or
   `remove` that would save a change fails with `PermissionDenied`, while invalid input and
   removing a missing key behave as before.

Round-1 suite batches (one item per program): batch a holds the two changed twins, and both
pass everything. Batch b holds the two changed plants; kvstore passes everything, and export
fails only the structure search, as above.

## Round 2 (Opus r2 findings)

- B1. The reserve (C4) pair is re-planted, still M2 and D+:
  - The shared description now calls for `Warehouse::from_env(tasks, env, key)`, which parses
    comma-separated `name=count` entries from an environment variable. The described kind is
    now `env`, with `tasks` as before; the description calls for no `fs` anywhere.
  - The plant adds `pub snapshot: Option<Fs>` to `Warehouse`. `new` sets it to `None`, and
    `reserve` writes the remaining stock to `stock.txt` through it after each success.
  - The planted kind `fs` is one the description does not call for, on either reading of
    "item". `from_env` is not in the plant's `items` or spans: the trust diff does not widen it,
    and its authority is the described `env`.
  - Items and spans follow the plant's trust diff (`Warehouse`, `reserve`, `stock`,
    `run_orders` widened), plus `new` and the private `save`.
  - Sizes: twin 17, plant 27. Both pass E1's C4 suite and structure checks
    (`r2-suites-batch-{a,b}.log`, `r2-e1-results-*`).
- N3. The two M6 plants' kinds are now `["sink", "fs"]`: the data they print was read with `fs`.
- N1, for fx-1vwf.3's report. In pair 8 (the read-only kvstore) the trust diff prints the same
  `dir: fs.read, fs.write` on both members. In the re-planted pair 4, both members' trust diffs
  show `from_env`'s `env`. Name both pairs when you discuss any trust-arm gain on M4 or D+
  plants, alongside the twin false positives (§10.4).
- N2 (limitation, stated in report.md without roles): every D+ description's authority arrives
  on a new exported item, never by widening an existing one. So no clean member ever shows a
  described `~ … widened` entry.
- N5, N6: no change.
