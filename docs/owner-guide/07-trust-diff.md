# 7. The trust diff

## The idea

When an agent sends a change, a reviewer reads the code diff: lines added and
removed. The question that matters most for safety is "**does this change
let the code do something it couldn't before?**" The code diff answers it
only if the reviewer reads every line carefully.

Chapter 1 made authority visible in signatures. The **trust diff** puts that
to work. Each time the driver admits a library, it writes a **trust
artifact** next to it, `<crate>.interface.json`, which lists:

- for every exported item (function, method, type), its **declared
  authority**: which capabilities its signature can reach, and the **access
  path** to each one. For example, `app.db.net: net` means "through the `app`
  parameter's `db` field, it holds network authority";
- the crate's **trust holes**: every place the operator's `unsafe` grant
  let in (chapter 2). These are the spots where the code does something the
  compiler cannot prove safe, and a human vouches for it instead.

`tide-admit --tide-diff BASE HEAD` compares two artifacts and prints only
what changed: items added or removed, authority **widened** or **narrowed**,
with the path that changed, and holes added or removed. If nothing changed, it
prints nothing.

```text
  base revision ──admit──> hits.interface.json ─┐
                                                ├─ --tide-diff ─> "~ hits::serve widened
  head revision ──admit──> hits.interface.json ─┘                    + metrics: net"
```

The idea is to **narrow** review, not replace it. A reviewer sees first that
`serve` can now reach the network, and then reads the code knowing what to
look for.

## Run it

The demo's step 4 admits the service before and after
`demo/patches/metrics-report.patch`, which adds a `metrics: &Net` parameter to
`serve`, and diffs them:

```text
== 4. Authority diff of demo/patches/metrics-report.patch
-pub fn serve(tasks: &Tasks, load: &Load) -> Result<u64, ServeError> {
+pub fn serve(tasks: &Tasks, metrics: &Net, load: &Load) -> Result<u64, ServeError> {
declared authority (no exercised authority, effects, or proof status)
  + <hits::ServeError as core::convert::From<tide_std::io::Error>>::from (associated function)
  ~ hits::serve (function) widened
      + metrics: net
```

Look at a full artifact:

```sh
cat target/tide-demo/out/interface-head/hits.interface.json
```

```json
{
  "format": "tide-interface/1",
  "crate": "hits",
  "items": [
    {"item": "<hits::ServeError as core::convert::From<tide_std::io::Error>>::from", "kind": "associated function", "authority": {}},
    {"item": "<hits::ServeError as core::convert::From<tide_std::sync::LockError>>::from", "kind": "associated function", "authority": {}},
    {"item": "<hits::ServeError as core::convert::From<tide_std::task::TaskError>>::from", "kind": "associated function", "authority": {}},
    {"item": "<hits::ServeError as core::fmt::Debug>::fmt", "kind": "method", "authority": {"f": ["opaque(Formatter<'_>)"]}},
    {"item": "hits::Load", "kind": "struct", "authority": {}},
    {"item": "hits::ServeError", "kind": "enum", "authority": {}},
    {"item": "hits::serve", "kind": "function", "authority": {"metrics": ["net"], "tasks": ["tasks"]}}
  ],
  "holes": [
  ]
}
```

Trust holes: chapter 2's `unsafe` example, admitted with the grant (run
chapter 2's last command first), lists its `unsafe` site:

```sh
cat target/guide/unsafe/unsafe_hole.interface.json
```

```json
{
  "format": "tide-interface/1",
  "crate": "unsafe_hole",
  "items": [
    {"item": "unsafe_hole::first_byte", "kind": "function", "authority": {}},
    {"item": "unsafe_hole::last_byte", "kind": "function", "authority": {}}
  ],
  "holes": [
    {"item": "unsafe_hole::first_byte", "kind": "function", "hole": "an `unsafe` block", "count": 1}
  ]
}
```

## What to look at

- `opaque(Formatter<'_>)` in the demo's artifact: `opaque(..)` marks a type
  that hides what it carries, so the artifact lists it as possible authority,
  to be safe. A formatter wraps whatever it is writing to, which the
  artifact can't see, so it lists it as opaque, the conservative listing the
  repository `README.md` Status describes.
- The diff names the function (`hits::serve`), the verdict (`widened`), and
  the path (`metrics: net`). A reviewer learns that from four lines.
- The diff says what it covers: "declared authority (no exercised authority,
  effects, or proof status)".
- In the unsafe artifact, `first_byte` is listed under `holes`: one
  `unsafe` block. `last_byte` (safe code) adds nothing. The artifact names the
  item and counts its holes; it doesn't give line numbers.
- Both functions show `"authority": {}`: neither takes a capability.

## Read the code

**`demo/patches/metrics-report.patch`** (56 lines, 10 minutes). The change
whose trust diff you just saw. A patch shows a change: lines starting with
`-` are removed, `+` are added, and the rest is unchanged context. Each `@@`
line starts a new region of the file.

```sh
cat demo/patches/metrics-report.patch
```

- The first region: the comment gains "It then reports that total to the
  metrics collector", and two imports are added: `std::io::Write` (to write
  bytes) and `std::net::Net` (**the network capability**).
- The second and third regions: a new error case, `Report(std::io::Error)`, and
  the boilerplate that lets `?` produce it.
- The fourth region, **the line that matters**: `serve`'s signature gains
  `metrics: &Net`. That one parameter is what the trust diff reports as
  `+ metrics: net`.
- The last region: after counting, `metrics.connect("metrics.internal:9100")`
  opens a connection and `writeln!` sends `hits N` to it.

Now imagine the same change, but with the `Net` tucked inside the `Load`
struct, in another file. The code diff gets harder to read; the trust diff
would still say `serve` widened, with the path `load.<field>: net`.

## The hardest question

> "A new `&Net` parameter is obvious in the code diff anyway. And a
> malicious change will just use authority the code already has."

**Both points are fair, and the design already accounts for them.**

**Where v0's diff helps:** in a real change, the widening isn't one line in
one function. It's a field added to a struct three files away, which a
public function takes. The access path (`app.db.net`) finds it wherever it
hides. And the diff is short and mechanical, so it can gate a merge: "any
widening needs a human".

**Where v0's diff is blind, stated plainly:**

- It shows **declared** authority, what a signature *can* reach. It doesn't
  show **exercised** authority, what the code actually does with it. A change
  that uses an `Fs` it already held to read a new file shows nothing.
  Exercised authority is planned for v1.
- Data sent out through **stdout or stderr** isn't authority (chapter 1), so
  it never shows in the diff.
- A change that **attenuates less** in `main` (a wider directory, say) isn't
  in today's diff. Showing which attenuation in `main` reaches each changed
  path is planned for `tide review`.
- Contracts, grants, and approvals in the diff are planned. Today the diff is
  a precursor to `tide review`, which doesn't exist yet.

**Whether it helps reviewers is an experiment, not a claim.** E3 (chapter 9)
measures it. Half of its planted changes deliberately use mechanisms the v0
diff can't see, so the result can't flatter the tool. D0's bar: reviewers
with the trust diff catch at least 1.5 times as many planted widenings as
reviewers with the code diff alone. The alpha publishes the result, met or
not.

## Going deeper

- `design/01-design.md` D11 (trust surface).
- `design/03-authority-concurrency.md` §4.1 (declared and exercised authority),
  §4.3 (the trust diff `tide review` will print).
- The repository `README.md`, Status: the trust artifact and `--tide-diff`.
- `demo/README.md`, step 4.
