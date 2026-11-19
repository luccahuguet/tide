# NN: fx-zu4f.15, chrono_anchor re-tested after the std conversion fix

Bead fx-zu4f.15. The registered protocol is
`research/protocols/fx-zu4f.15-protocol.md`, reproduced verbatim in
section 10. This is a new pre-registered test. It does not revise, rerun or
replace fx-zu4f.8 (`research/47-malware-rescreen.md`), whose documented
negative stands as published, or fx-zu4f.5 (`research/37`). Both are cited
here as prior work, and neither is edited. The run was done by one Claude
Opus run builder under tide-lead's `go`.

Terms used below:

- **RUN**: the builder that ran this test after `go`. **RC**: Tide's
  release candidate.
- **R'**: the revision of `main` that every port was checked against,
  `b1f0447` (section 4). fx-zu4f.8's revision was R.
- **TD codes**: Tide admission diagnostics. Codes counted for leg 1 include
  `TD0501`, an ambient std call that needs a capability, and `TD0602`, a
  Rust thread spawn in place of a Tasks scope.
- **In-class operation**: an operation in one of the screen's classes:
  ambient process, network, filesystem, environment or clock use; a detached
  spawn; global mutable state; or unsafe without a grant.
- **Caught rule** (fx-zu4f.8's, unchanged): a row is caught when both legs
  hold. **Leg 1**: the faithful port of the malicious code (the buggy port)
  draws a counted TD code on an in-class operation of an affected function.
  **Leg 2**: the capability-passing version, which receives the authority it
  uses as explicit parameters, is admitted cleanly (driver exit 0, no error
  diagnostic).

## 1. Disclosure: why this pool, and when it was chosen

- **The pool was chosen after fx-zu4f.8 saw the rows.** fx-zu4f.8 screened 87
  malicious-crate advisories and caught none. Its nearest miss was
  chrono_anchor (RUSTSEC-2026-0039). The buggy port drew counted TD codes.
  The capability-passing version failed only on rustc `E0277` at the table
  form `clock.sleep(d)?`, because Tide's std at that revision had no
  conversion from `Cancelled` into `TaskError`.
- **Why it was re-tested.** Tide's std had a gap: the documented idiom
  `clock.sleep(d)?` did not compile inside a task body. The fx-zu4f.8 screen
  exposed it. The owner chose to fix the std (fx-qlek.18, an ergonomics
  fix) and to re-test under a new registration. This is a relayed owner
  decision: Lucca via tide-desk, 2026-11-19, "1".
  - **The fix reverses an earlier removal.** fx-nkv.3's review had removed
    the conversion that fx-qlek.18 adds, on purpose
    (`reviews/fx-nkv.3-r1-codex.md` and `reviews/fx-nkv.3-r1-opus.md`, B1).
    The reason was that it breaks type inference for an unannotated
    `tasks.scope(..)?` (rustc E0283). The owner's 2026-11-19 decision
    reversed that removal. The cost now falls on users: 17 scope sites name
    their error type (design/03 §6.1).
- **What the result can and cannot show.** The outcome at R' was close to
  predictable. fx-qlek.18 adds a case proving that the idiom compiles in a
  task body. fx-zu4f.8 also disclosed that two non-registered variants of
  this same port were admitted. The catch below shows that the re-tested
  row meets the caught rule at R'. It does not show that Tide caught the
  row the first time. fx-zu4f.8's result stays "0 caught of 87".
- Whether this catch counts as the RC's public catch is decided by
  fx-qlek.7's rule and its cold-reader pass, not by this test.

## 2. Result: one catch in a one-row pool

**RUSTSEC-2026-0039 (chrono_anchor) meets the caught rule at R'.** The rule
is fx-zu4f.8's, unchanged. The test ran admission and type-checking only:
no port was executed.

- **Leg 1.** The faithful buggy port draws counted TD codes on in-class
  operations of its affected functions: `TD0602` ×2 (`thread::spawn`, 15:5
  and 29:5) and `TD0501` ×3 (`Command::new`, 17:17 and 33:17;
  `thread::sleep`, 30:9).
- **Leg 2.** The capability-passing version is admitted cleanly: driver exit
  0, no diagnostic. Its trust artifact declares the authority `bg`, `clock`
  and `proc` for `run_optional_syncs`.

The port files are fx-zu4f.8's final files, byte for byte. No port was
changed and no fix was needed. The leg-1 codes are the ones fx-zu4f.8
recorded. At R', the table form `clock.sleep(Duration::from_secs(1))?` now
type-checks, because fx-qlek.18 added `From<Cancelled> for TaskError`.

The pool is one row, so the counts (section 6) describe that row only. They
are not a rate.

## 3. Registration

Registered on branch fx-zu4f.15 in commit
`e2fba815b56a9d079f7237b7c0d158f3f57e7063` (2026-11-19T05:24:05Z, parent
`04dde61`). By git history and tide-lead's bead comment of 05:25 UTC, that
was before fx-qlek.18 was implemented or landed, and before any port of this
test was copied or run. These are Tide's own records. The ordering bullet
below says what the timestamp does and does not add.

| File | SHA-256 | OpenTimestamps |
| --- | --- | --- |
| `research/protocols/fx-zu4f.15-protocol.md` | `a15c9236c1bd0fd0beeb31457571f558d9ce9e02d686f2cf6f1e7f4ab0848065` | `.ots` beside it, Bitcoin block 976536 |
| `research/protocols/fx-zu4f.15-pool/select.py` | `0c20f29fceb30094b02ade623ca9095ed0c12866b5e8e0e583c4043aae111c44` | `.ots` beside it, Bitcoin block 976536 |
| `research/protocols/fx-zu4f.15-pool/select-output.txt` | `0661f28bffb5090b650efc2816979a0204b2afa804d907a63ce12d7594d4b1ff` | `.ots` beside it, Bitcoin block 976536 |

- Proofs stamped 2026-11-19T05:25Z, with the pending proofs committed as
  `2b7e0dd` (05:25:14Z). They were upgraded in `442f8a9` (06:34:23Z). The
  proof files' SHA-256s are `e21b3303…2105` (protocol),
  `b5a76c93…e975` (`select.py`) and `ff3972a8…2081` (`select-output.txt`).
- **Ordering: anchoring before the fix landed.** Protocol §6.2 requires
  that the anchoring block precede fx-qlek.18's landing commit `4c5b8c0`.
  Git's committer times put the upgrade (`442f8a9`, 06:34:23Z) before the
  landing (06:35:00Z), but those times are self-reported. The checks below
  are consistent with that chronology. They do not establish independently
  that the anchoring preceded the landing. For that order, a reader relies
  on the block's timestamp, within its tolerance, and on Tide's own records
  (git times and the landing commit's message).
  - **(a) Git link.** `4c5b8c0`'s message (lines 41 to 43 of the commit
    message, subject line included) reads: "Ordering: the fx-zu4f.15 registration (e2fba81) and its
    OpenTimestamps proofs (2b7e0dd, upgraded 442f8a9, Bitcoin block 976536)
    precede this landing." The abbreviation `442f8a9` resolves to exactly
    one object in the repository, the commit
    `442f8a99d4e44c52c2aa31f7e47cd48daeaa1316`. That commit's tree holds the
    three upgraded proofs, whose operation paths end in block 976536's
    Merkle root (part b). If the abbreviation was not forged (limits below),
    `4c5b8c0` was written after `442f8a9` existed, because its hashed
    message names `442f8a9`. This link says nothing about when block 976536
    was mined. A Merkle inclusion path can be computed from a block template
    before the block is mined, so proofs that reach the block's Merkle root
    do not by themselves show that the block came before `442f8a9` or
    `4c5b8c0`. A reader checks the link with:

    ```sh
    git cat-file -p 4c5b8c0          # the message names 442f8a9 and block 976536
    git rev-parse 442f8a9            # 442f8a99d4e44c52c2aa31f7e47cd48daeaa1316
    git ls-tree 442f8a9 research/protocols/ research/protocols/fx-zu4f.15-pool/
    ```

    Limits:
    - The message names `442f8a9` by a 7-hex-digit (28-bit) abbreviation,
      not the full hash. Someone who wrote `4c5b8c0` first could later
      search for a commit whose hash starts with `442f8a9`. That is a
      feasible amount of work, so the link does not hold against a
      deliberate forger. It does rule out an accidental or unnoticed
      reordering.
    - `442f8a9` is on the bead branch and is not an ancestor of `4c5b8c0`.
      The argument rests on the commit objects' hashes and on that message,
      not on ancestry.
    - Nothing here dates `4c5b8c0` or `442f8a9`. The link orders the two
      commits only.
  - **(b) Authenticated block header.** The proofs' block header was
    fetched on 2026-11-19, by read-only GET with no request body and nothing
    about Tide sent, from two independent public sources:
    - blockstream.info: `/api/block-height/976536` (06:55:13Z), then
      `/api/block/<hash>/header` (06:55:14Z);
    - mempool.space: the same two paths (06:55:14Z and 06:55:15Z).

    This fetch concerns only the timestamp. It is not a source for the
    screened row, whose sources stay fx-zu4f.8's (section 5). Both sources
    returned the same block hash,
    `000000000000000000018b8c855c551be475ce9e909b00df34ab95a991324d56`, and
    the same 80-byte header:

    ```text
    00c00a20431a7a919206221f205c3569aa560205e68718caec4500000000000000000000afd6390e1abae701242660367c3ea29e708e3ada82e826ab85a8ceb2bf8518dd4c8afe6a73160217bf69ffa6
    ```

    The standard-library script printed below (`python3 -I`, SHA-256
    `b53f6c49568980c8058cf4d9d0fcde0557358d49ec4c6ea0e6151042d16caa6e`)
    checks the following:
    - double SHA-256 of the header, in display order, equals the block
      hash;
    - each proof's file digest equals its registered file's SHA-256;
    - evaluating each proof's operations reaches a Bitcoin attestation at
      height 976536 whose message,
      `afd6390e1abae701242660367c3ea29e708e3ada82e826ab85a8ceb2bf8518dd`,
      equals the header's Merkle root (header bytes 36 to 68).

    All three proofs match. The header's timestamp is 1795066444,
    **2026-11-19T05:34:04Z**. A Bitcoin header's time is accurate only to
    about two hours. It precedes the landing's self-reported time
    (06:35:00Z) by 61 minutes, which is inside that tolerance, so on its own
    it does not order the anchoring before the landing. The check trusts the
    two explorers to report the chain's real block at that height. RUN did
    not consult a full node. The check establishes inclusion: each
    registered file existed by the attestation time, the time of block
    976536.

    ```python
    # ots-verify.py: python3 -I ots-verify.py <header-hex-file> <block-hash> <file.ots>...
    import hashlib, sys
    from datetime import datetime, timezone
    MAGIC = bytes.fromhex("004f70656e54696d657374616d7073000050726f6f6600bf89e2e884e89294")
    BITCOIN = bytes.fromhex("0588960d73d71901")
    class Reader:
        def __init__(self, data): self.data, self.pos = data, 0
        def byte(self):
            b = self.data[self.pos]; self.pos += 1; return b
        def take(self, n):
            out = self.data[self.pos:self.pos + n]
            if len(out) != n: raise ValueError("truncated proof")
            self.pos += n; return out
        def varuint(self):
            value, shift = 0, 0
            while True:
                b = self.byte(); value |= (b & 0x7F) << shift; shift += 7
                if b < 0x80: return value
        def varbytes(self): return self.take(self.varuint())
    def apply(tag, r, msg):
        if tag == 0x08: return hashlib.sha256(msg).digest()
        if tag == 0x02: return hashlib.sha1(msg).digest()
        if tag == 0x03: return hashlib.new("ripemd160", msg).digest()
        if tag == 0xF0: return msg + r.varbytes()
        if tag == 0xF1: return r.varbytes() + msg
        if tag == 0xF2: return msg[::-1]
        if tag == 0xF3: return msg.hex().encode()
        raise ValueError(f"unknown op 0x{tag:02x}")
    def walk(r, msg, found):
        def one(tag):
            if tag == 0x00: found.append((r.take(8), r.varbytes(), msg))
            else: walk(r, apply(tag, r, msg), found)
        tag = r.byte()
        while tag == 0xFF: one(r.byte()); tag = r.byte()
        one(tag)
    header = bytes.fromhex(open(sys.argv[1]).read().strip()); assert len(header) == 80
    block_hash = hashlib.sha256(hashlib.sha256(header).digest()).digest()[::-1].hex()
    merkle = header[36:68]; t = int.from_bytes(header[68:72], "little")
    print("hash ok:", block_hash == sys.argv[2], "time:",
          datetime.fromtimestamp(t, timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ"))
    ok = block_hash == sys.argv[2]
    for path in sys.argv[3:]:
        r = Reader(open(path, "rb").read())
        assert r.take(len(MAGIC)) == MAGIC and r.varuint() == 1 and r.byte() == 0x08
        digest = r.take(32); found = []; walk(r, digest, found); assert r.pos == len(r.data)
        ok &= digest == hashlib.sha256(open(path[:-4], "rb").read()).digest()
        btc = [(Reader(p).varuint(), m) for a, p, m in found if a == BITCOIN]
        ok &= bool(btc) and all(m == merkle for _, m in btc)
        print(path, [(h, m.hex()) for h, m in btc])
    print("ALL OK" if ok else "MISMATCH"); sys.exit(0 if ok else 1)
    ```

    RUN ran this exact listing from the repository root, on mempool.space's
    header and the three committed `.ots` files. It printed `hash ok: True
    time: 2026-11-19T05:34:04Z`, then height 976536 with message
    `afd6390e…18dd` for each proof, then `ALL OK`, and exited 0. A longer
    version with the same checks and more output (SHA-256 `1b136049…f8ee`)
    gave the same result on blockstream.info's header.
  - **What this supports.** Every check is consistent with the reported
    chronology:
    - the registration (`e2fba81`) and the stamping (05:25Z);
    - block 976536, whose header time is 05:34:04Z;
    - the upgrade commit (`442f8a9`, 06:34:23Z, self-reported);
    - the landing (`4c5b8c0`, 06:35:00Z, self-reported).

    The registered files existed by the attestation time. Independent proof
    that the anchoring preceded fx-qlek.18's landing is **not established**.
    That order rests on the block's timestamp, within its roughly two-hour
    tolerance, and on Tide's own records. No check found the order reversed.
- tide-lead recorded the three SHA-256s on bead fx-zu4f.15 (comment of
  2026-11-19 05:25 UTC).
- RUN checked these digests before first use, and all matched:
  - the three registered files above;
  - every item that protocol section 2 incorporates:
    - fx-zu4f.8's protocol `4e2016df…7e26` and its proof file
      `ed68e64a…6146`;
    - fx-zu4f.8's frozen criteria `f20b5667…a603`, and the caught-rule
      block alone, `7cb2479c…9d57`;
    - RULING-0039 `794f6885…d2`;
    - RULING-0175 with its correction `ff5992e1…c0`;
    - tide-desk's archive condition, comment 1106, `7ad11899…e836`;
    - its confirmation, comment 1312, `cd8d3ba0…71d0`;
    - research/47 at `fc43b37` `6c6111b7…896e`;
  - the three port files of protocol section 5.5 (section 8);
  - the harness `screen.sh` of fx-zu4f.8, `0b45dcbc…a043`.
- **POOL check (protocol §5.1).** `python3 -I
  research/protocols/fx-zu4f.15-pool/select.py` was rerun from the
  repository root, and it exited 0. Its output equals the committed
  `select-output.txt` byte for byte (`cmp`; SHA-256 `0661f28b…b1ff`):
  POOL = { RUSTSEC-2026-0039 }.

## 4. The run revision R' and the capability set at R'

- **R' = `b1f044744dea32e69cf0e6a92c4542496070f8ed`**. This was main's HEAD
  when tide-lead sent `go`. fx-qlek.18 landed as
  `4c5b8c05bc8e55f78cb805a52d2cbf455257b1a8`, an ancestor of R', with its
  reviews in `9ec3b4e`. After that, main moved only by those reviews and the
  ledger commit `b1f0447`. So, under protocol §4, R' is that later HEAD, not
  the landing commit. No file under `crates/` changed between `4c5b8c0` and
  R'. tide-lead chose R' before this run and without any result of it.
- **Tree identity and clean tree.** The branch merged R' in `def3a3f`. The
  final pass recorded `git diff --quiet R' HEAD -- crates Cargo.toml
  Cargo.lock rust-toolchain.toml` as holding and the tracked tree as clean.
  R' was frozen from the first port copy (2026-11-19T06:38:26Z).
- **The std conversion fx-qlek.18 added** (`crates/tide-std/src/task.rs` at
  R'):

  ```rust
  // The cancellation a checkpoint observed, returned as the task's error (03 §6.1).
  impl From<Cancelled> for TaskError {
      fn from(_: Cancelled) -> Self {
          TaskError::Cancelled
      }
  }
  ```

  The commit also added `Cancelled::is_cancelled()`. It turbofished 17
  `tasks.scope(..)` sites that the second conversion made ambiguous (E0283),
  and documented the conversions in design/03 §6.1. That inference cost is
  why fx-nkv.3's review had removed the conversion; section 1 discloses the
  reversal.
- **The table form at R'.** Companion §14.2's `thread::sleep` row at R'
  reads: "`clock.sleep(d)?`; in a scope with your own error enum,
  `clock.sleep(d).map_err(TaskError::from)?` (7.1)". At registration (and at
  fx-zu4f.8's R) the row read only `clock.sleep(d)?`. The added clause
  covers a scope whose error is the user's own enum. This port's scope error
  is `TaskError`, so its table form is still `clock.sleep(d)?`, as at
  registration (RULING-0039 item 1, read at R').
- **Capability set at R'** for the operations this row maps
  (`design/reference.md` §6.1 at R'):
  - `Proc`: "`command(prog) -> Command` (a handle; the child has ambient
    authority: companion 20); planned: `exit(code: i32) -> !`". Present:
    `command` is listed without "planned", and its owning bead fx-ayh2.1
    landed as `e0b0d62`, an ancestor of R'.
  - `Clock`: "`now() -> Instant`, `system_time() -> SystemTime`, `sleep(d)`,
    `sleep_until(t)`, both `-> Result<(), Cancelled>`". v0, present.
  - `Tasks`/`Scope`: `scope`, `spawn_isolated`, `shield`, `checkpoint`,
    `yield_now`, with `Scope::spawn`. v0, present.
  - For completeness, `Net` at R' lists `connect`, `bind`, `get` and
    `resolve` (fx-ff13 `8ac1317`; fx-6d50 `d378a7d`, both ancestors of R');
    `bind_udp` is planned. This row uses no `Net` operation.
- `conformance/rules.md` at R' keeps ambient `Command::new` at `TD0501`
  (CAP-1, PROC-2). No separate code exists for it.

## 5. Method, with the protocol's narrowings

fx-zu4f.8's registered protocol (`research/protocols/fx-zu4f.8-protocol.md`,
SHA-256 `4e2016dff7838f9bdcccf4b2cedbabf7051ab998e2a19af77f6e7347cfdc7e26`) binds
this test, with its caught rule, its §6 closed list of allowed diff lines,
RULING-0039, RULING-0175 with its correction, and tide-desk's archive
condition (2026-11-18 14:04, confirmed 2026-11-19 04:20). It is cited, not
reproduced again. This test's protocol (section 10) changes only names, the
pool and the revision, and it narrows the method as follows:

- **No new search and no new fetch.** chrono_anchor's sources are those
  fx-zu4f.8 collected: S020, S469, S470, S471, S473, S476, and archive copy
  socket-3, whose quote of the code is the port's basis. Nothing was fetched
  again, so no evidence entered after the result was known.
- **Steps 1 to 5 carry over** unchanged from research/47's row: advisory,
  sources, class, the faithful port on paper, and the safety gate. **Steps 6
  to 8 ran again at R'**: the buggy port's run (leg 1), the mapping on paper
  at R', and the capability-passing version's run (leg 2).
- **Ports reused byte for byte.** The three files were copied from
  fx-zu4f.8's work folder into `target/work/ports/RUSTSEC-2026-0039/`, and
  their SHA-256s were checked against research/47 §8 (section 8). No port was
  rewritten. §6's allowed fixes stayed available for a rustc error that R'
  itself caused. None arose, so none was applied.
- **Diff recheck (§6).** Before the final pass, RUN regenerated
  `diff -u buggy.rs capability.rs` and confirmed that its body equals the
  copied `capability.diff` byte for byte (the two `---`/`+++` header lines
  differ only by path and file time). Then RUN checked each changed line
  against §6's closed list:
  - three `use` lines (`Proc`; `Scope`, `TaskError`; `Clock`, `Duration`);
  - three signatures that gain capability parameters (`fetch_ref_background`,
    `run_optional_syncs`, `submit_snapshot`), with the two call sites in
    `run_optional_syncs` passing the threaded parameters;
  - mapped operations in their table forms: `bg.spawn(move || ..)` ×2,
    `proc.command(..)` ×2, and `clock.sleep(Duration::from_secs(1))?` (the
    R' form above);
  - the listed adaptation: the spawned task's `Ok(())` ×2.

  No line falls outside the list. No `let _ =` discard or `.map_err` was
  added (RULING-0039).
- **Harness.** `target/work/ports/screen.sh` is fx-zu4f.8's `screen.sh`
  (`0b45dcbc…a043`) with these changes only:
  - the header comment and the two labels, which now print `R'`;
  - the worktree, build target and revision values (the name and revision
    substitutions of §5.1 and §4), from which the log path and output root
    are derived;
  - the run list, chrono_anchor's two ports.

  Its log path (`ports/final.log`) and output root (`screen-out/final`) keep
  the same form under the new names. The development runs used fx-zu4f.8's
  `dev.sh` with the same name substitutions (SHA-256 `d4c8bfd6…b42f`).
- **Safety and the archive condition.** chrono_anchor's ports derive from
  archive copy socket-3. RUN checked them against the condition's four
  limits:
  1. Every endpoint is `format!` of the hard-coded constants `"placeholder"`
     and `"/<path>"` into an RFC 6761 `.invalid` URL. There is no IP
     address, and nothing is decoded.
  2. Nothing archive-derived was executed. Only admission runs ran
     (`--crate-type=lib --emit=metadata`), with no forbidden flag.
  3. Only the authority shape is ported. The process calls keep
     `"<program>"` and `"<arg>"`, with the `.env` path that the write-up names
     as one argument.
  4. All work stayed in `target/work`, and driver output went to the build
     target.

  The admitted runs left build artifacts: `libcapability.rmeta` and the
  `interface.json`, each with a cache copy, in the development and final
  output folders. Section 8 lists them by SHA-256, as does
  `target/work/ports/undeleted-artifacts.sha256`. None was used. After
  tide-lead answered the caught-candidate report, RUN checked all eight
  files against that list (`sha256sum -c`, all OK) and deleted them unused
  at 2026-11-19T06:43:25Z. The record keeps the `interface.json`'s SHA-256
  and declared authority, which were read before the deletion.

## 6. Counts

These counts describe this registered one-row pool only. They are not a
rate.

- Pool: 1 advisory (RUSTSEC-2026-0039).
  - caught: 1
  - not caught (no portable source): 0
  - out of class: 0
  - not caught (port incomplete): 0
  - not caught (no TD code): 0
  - not caught (capability absent): 0
  - not caught (counterpart rejected): 0
  - unresolved: 0
  - Sum: 1.
- Searches: 0 (narrowed: none run). Fetches: 0. Archive copies used: 1
  (socket-3, carried over and not reread for the port).
- Ports written: 0 new. 2 ports were reused byte for byte (`buggy.rs`,
  `capability.rs`). Allowed fixes applied: 0.
- Runs: 2 in the final pass, plus 2 development admissions in one
  development run (`dev.log`), with the same results.
- Selection, for context: of fx-zu4f.8's 87 rows, 8 recorded a
  capability-passing run. 7 of those (amaperf) fail criterion C2, and 1
  (chrono_anchor) is selected (`select-output.txt`). The other 79 rows never
  reached such a run. They include chrono_anchor's four siblings in the same
  campaign (time_calibrator, time_calibrators, dnp3times, time-sync), all
  four not caught (no portable source) in research/47 §7.

## 7. The row

The row is in fx-zu4f.8's §9 format. Fields up to `affected_functions` carry
over from research/47 §7 (line 450 at `fc43b37`). The rest are from this
run's final pass at R'.

| id | crate | incident | stated_effect | class | sources_read | code_quoted | affected_functions | buggy_codes | mapped_ops | capability_run | outcome | gaps | reason |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| [RUSTSEC-2026-0039](https://github.com/rustsec/advisory-db/blob/233155b4025b854c68a4b199c0e32d7fd53291a2/crates/chrono_anchor/RUSTSEC-2026-0039.md) | chrono_anchor | time-utility .env exfiltration (2026) | The `chrono_anchor` crate attempted to exfiltrate `.env` files to a server that was in turn impersonating the legitimate `timeapi.io` service. | fs, network (per advisory); process, spawn, clock (per source: socket-3) | S020, S469, S470, S471, S473, S476, socket-3 (carried over from research/47; nothing fetched again) | y: socket-3 (archive) | run_optional_syncs, fetch_ref_background, submit_snapshot | TD0602 x2 (15:5, 29:5 thread::spawn), TD0501 x3 (17:17, 33:17 Command::new; 30:9 thread::sleep); other: E0433 x3, E0599 x2 | bg.spawn(move \|\| {..; Ok(())}) x2, proc.command x2, clock.sleep(d)? | exit 0; no diagnostic; interface.json dbc20e74…1f43 declares bg, clock, proc | caught (at R'; re-test under a new registration, see section 1) | no detached spawn in Tide (task ends within the caller's scope); the child of proc.command has ambient authority (companion 20); fx-zu4f.8's From<Cancelled> gap is closed at R' by fx-qlek.18 | leg 1 passes (TD0602, TD0501); leg 2 passes at R': the table form clock.sleep(d)? now converts Cancelled into TaskError |

## 8. Port proof

All port files are in `target/work/ports/` (git-ignored, never committed).
Each was checked against research/47 §8 when it was copied, before first
use. The final pass hashed the two `.rs` files before the runs and after each
run, and they were unchanged. `capability.diff` is not in those hashes. It
was checked when copied and again at the diff recheck (section 5), and its
SHA-256 still matches.

| Port file | SHA-256 |
| --- | --- |
| `RUSTSEC-2026-0039/buggy.rs` | `3c296fb448ab2742e979d8cdc5373171b7a9b4a37e88da2ce3d933fcd2e4a110` |
| `RUSTSEC-2026-0039/capability.rs` | `51f348513fc80a727a35ce416fe304a0e4bf6ccfa8c5067d7207887c69d179a4` |
| `RUSTSEC-2026-0039/capability.diff` | `c8a002fb329e6695b179556943bbf85cef4dec2bfbbc8b6c3be40429d46dbdf7` |

**Port, in prose.** The port follows Socket's three quoted functions
(archive copy socket-3). `run_optional_syncs` calls `fetch_ref_background`
and `submit_snapshot`. Each of those spawns a detached thread that runs one
process. `submit_snapshot`'s thread first sleeps one second, and its process
arguments carry the `.env` path. The `diff -U0` hunks, changed lines only,
are the same as research/47 §8's:

```diff
+use std::process::Proc;
+use std::task::{Scope, TaskError};
+use std::time::{Clock, Duration};
-fn fetch_ref_background() {
-    std::thread::spawn(|| {
+fn fetch_ref_background<'s>(bg: Scope<'s, '_, TaskError>, proc: &'s Proc) {
+    bg.spawn(move || {
-        let _ = std::process::Command::new("<program>")
+        let _ = proc.command("<program>")
+        Ok(())
-pub fn run_optional_syncs() {
-    fetch_ref_background();
-    submit_snapshot();
+pub fn run_optional_syncs<'s>(bg: Scope<'s, '_, TaskError>, clock: &'s Clock, proc: &'s Proc) {
+    fetch_ref_background(bg, proc);
+    submit_snapshot(bg, clock, proc);
-fn submit_snapshot() {
-    std::thread::spawn(|| {
-        std::thread::sleep(std::time::Duration::from_secs(1));
+fn submit_snapshot<'s>(bg: Scope<'s, '_, TaskError>, clock: &'s Clock, proc: &'s Proc) {
+    bg.spawn(move || {
+        clock.sleep(Duration::from_secs(1))?;
-        let _ = std::process::Command::new("<program>")
+        let _ = proc.command("<program>")
+        Ok(())
```

**The final pass.** `target/work/ports/screen.sh` (SHA-256
`b3c97fcb37ad60678587defcda6474b7162387b3825b2cac7451f5d9acfa7637`) ran once,
2026-11-19T06:40:02Z–06:40:08Z. Its log, `target/work/ports/final.log` (made
read-only afterwards), has SHA-256
`e0029ced4c273c27d11f3890b8dacab64d9c9539c9c975cc07b098f729aa6c36`. It
records:

- R' `b1f044744dea32e69cf0e6a92c4542496070f8ed`; WT HEAD
  `def3a3fec75f1de2f5acd698734ca5e9a0917300`; tree identity with R': yes;
  clean tracked tree: yes;
- the toolchain: `rust-toolchain.toml` channel `nightly-2026-08-09`, rustc
  1.99.0-nightly (`771916f90` 2026-08-08, LLVM 23.1.0), cargo 1.99.0-nightly
  (`b07e5a086` 2026-08-07), host `x86_64-unknown-linux-gnu` (Linux
  7.2.6);
- heavy.sh SHA-256
  `cb2498c311b02f5985f51ac38e301abaa43196c3070606a096be10eac54c3080`
  (unchanged from fx-zu4f.8's final pass); environment
  `CARGO_BUILD_JOBS=4 CARGO_TARGET_DIR=<target> TMPDIR=<target>/tmp`, where
  `<target>` is `/mnt/build/tide-agents/fx-zu4f.15-target`;
- the builds, `cargo build --locked -p tide-admit` and
  `cargo build --locked -p tide_std --message-format=json`, both exit 0:
  driver `tide-admit` SHA-256
  `23fc1b295df4393ff27875acdb998c971f3162adadbe66e40a8ea7e3a22a90f8`,
  `libtide_std.rlib`
  `64a6bab7ef6c8debdc5fa434864771cb09de3b55764727ccd25614f4a3ede713`;
- the load before each command, 3.53 to 3.66;
- each port file's SHA-256 before the runs and after each run, unchanged.

Both admission runs, exactly as `final.log` records them (lines 48 and 73,
in `printf %q` form, wrapped here with `\` line continuations; every
argument is kept). The working directory of both was
`/mnt/build/eon-agents/worktrees/tide-fx-zu4f.15`.

RUSTSEC-2026-0039 buggy:

```sh
bash /home/lucca/.cache/eon-agents/eon-overseer/heavy.sh \
  env CARGO_BUILD_JOBS=4 CARGO_TARGET_DIR=/mnt/build/tide-agents/fx-zu4f.15-target TMPDIR=/mnt/build/tide-agents/fx-zu4f.15-target/tmp \
  timeout 600 \
  /mnt/build/tide-agents/fx-zu4f.15-target/debug/tide-admit \
  --crate-type=lib \
  --emit=metadata \
  --tide-std /mnt/build/tide-agents/fx-zu4f.15-target/debug/libtide_std.rlib \
  --tide-dep-dir /mnt/build/tide-agents/fx-zu4f.15-target/debug/build/shlex/210ed138669455eb/out \
  --tide-dep-dir /mnt/build/tide-agents/fx-zu4f.15-target/debug/build/find-msvc-tools/be59c2deb9ca90e4/out \
  --tide-dep-dir /mnt/build/tide-agents/fx-zu4f.15-target/debug/build/zeroize/1a1fb824db4ff4c4/out \
  --tide-dep-dir /mnt/build/tide-agents/fx-zu4f.15-target/debug/build/cc/f47325fd35ce8143/out \
  --tide-dep-dir /mnt/build/tide-agents/fx-zu4f.15-target/debug/build/cfg-if/7f02b23b970d439a/out \
  --tide-dep-dir /mnt/build/tide-agents/fx-zu4f.15-target/debug/build/libc/4c49c219f395f0df/out \
  --tide-dep-dir /mnt/build/tide-agents/fx-zu4f.15-target/debug/build/unicode-ident/debd0243e1d0e647/out \
  --tide-dep-dir /mnt/build/tide-agents/fx-zu4f.15-target/debug/build/getrandom/b3dcb6500f9fca6e/out \
  --tide-dep-dir /mnt/build/tide-agents/fx-zu4f.15-target/debug/build/proc-macro2/1f010a4d5cf75eaa/out \
  --tide-dep-dir /mnt/build/tide-agents/fx-zu4f.15-target/debug/build/rustls-pki-types/1a3718ff5f28605d/out \
  --tide-dep-dir /mnt/build/tide-agents/fx-zu4f.15-target/debug/build/utf_types/cc738870998629bc/out \
  --tide-dep-dir /mnt/build/tide-agents/fx-zu4f.15-target/debug/build/untrusted/9c1a8c0c452596ad/out \
  --tide-dep-dir /mnt/build/tide-agents/fx-zu4f.15-target/debug/build/quote/9bd8aac8db580ea4/out \
  --tide-dep-dir /mnt/build/tide-agents/fx-zu4f.15-target/debug/build/ring/1c82005960b2faa8/out \
  --tide-dep-dir /mnt/build/tide-agents/fx-zu4f.15-target/debug/build/utf8_iter/07626a5b00c33664/out \
  --tide-dep-dir /mnt/build/tide-agents/fx-zu4f.15-target/debug/build/percent-encoding/d0ddde42e654829c/out \
  --tide-dep-dir /mnt/build/tide-agents/fx-zu4f.15-target/debug/build/smallvec/33c5fbcfb79f35dd/out \
  --tide-dep-dir /mnt/build/tide-agents/fx-zu4f.15-target/debug/build/idna_adapter/25b3f71ecdfcf02b/out \
  --tide-dep-dir /mnt/build/tide-agents/fx-zu4f.15-target/debug/build/idna/37ec807e93d1a766/out \
  --tide-dep-dir /mnt/build/tide-agents/fx-zu4f.15-target/debug/build/form_urlencoded/4a42b89f31c459ce/out \
  --tide-dep-dir /mnt/build/tide-agents/fx-zu4f.15-target/debug/build/rustls-webpki/b05ff942139a2ca5/out \
  --tide-dep-dir /mnt/build/tide-agents/fx-zu4f.15-target/debug/build/syn/d41f88b89dfd81d6/out \
  --tide-dep-dir /mnt/build/tide-agents/fx-zu4f.15-target/debug/build/once_cell/34f91f5baba797d6/out \
  --tide-dep-dir /mnt/build/tide-agents/fx-zu4f.15-target/debug/build/bitflags/21113d36d14db8ae/out \
  --tide-dep-dir /mnt/build/tide-agents/fx-zu4f.15-target/debug/build/linux-raw-sys/4e3c60692274f452/out \
  --tide-dep-dir /mnt/build/tide-agents/fx-zu4f.15-target/debug/build/subtle/a7c30e9150b6c9d8/out \
  --tide-dep-dir /mnt/build/tide-agents/fx-zu4f.15-target/debug/build/tide-macros/d248bbce74dfe775/out \
  --tide-dep-dir /mnt/build/tide-agents/fx-zu4f.15-target/debug/build/rustix/b671a2e43dfdfa9d/out \
  --tide-dep-dir /mnt/build/tide-agents/fx-zu4f.15-target/debug/build/url/c2643ebfa5304869/out \
  --tide-dep-dir /mnt/build/tide-agents/fx-zu4f.15-target/debug/build/rustls/3e8ed9e94ce89229/out \
  --tide-dep-dir /mnt/build/tide-agents/fx-zu4f.15-target/debug/build/httparse/c8df9207061df7e1/out \
  --tide-dep-dir /mnt/build/tide-agents/fx-zu4f.15-target/debug/build/libm/f1d70f6c6e02e43b/out \
  --tide-dep-dir /mnt/build/tide-agents/fx-zu4f.15-target/debug/build/webpki-roots/4acd37ba3fda627c/out \
  --tide-dep-dir /mnt/build/tide-agents/fx-zu4f.15-target/debug \
  --out-dir /mnt/build/tide-agents/fx-zu4f.15-target/screen-out/final/RUSTSEC-2026-0039-buggy \
  /mnt/build/eon-agents/worktrees/tide-fx-zu4f.15/target/work/ports/RUSTSEC-2026-0039/buggy.rs \
  </dev/null
```

RUSTSEC-2026-0039 capability:

```sh
bash /home/lucca/.cache/eon-agents/eon-overseer/heavy.sh \
  env CARGO_BUILD_JOBS=4 CARGO_TARGET_DIR=/mnt/build/tide-agents/fx-zu4f.15-target TMPDIR=/mnt/build/tide-agents/fx-zu4f.15-target/tmp \
  timeout 600 \
  /mnt/build/tide-agents/fx-zu4f.15-target/debug/tide-admit \
  --crate-type=lib \
  --emit=metadata \
  --tide-std /mnt/build/tide-agents/fx-zu4f.15-target/debug/libtide_std.rlib \
  --tide-dep-dir /mnt/build/tide-agents/fx-zu4f.15-target/debug/build/shlex/210ed138669455eb/out \
  --tide-dep-dir /mnt/build/tide-agents/fx-zu4f.15-target/debug/build/find-msvc-tools/be59c2deb9ca90e4/out \
  --tide-dep-dir /mnt/build/tide-agents/fx-zu4f.15-target/debug/build/zeroize/1a1fb824db4ff4c4/out \
  --tide-dep-dir /mnt/build/tide-agents/fx-zu4f.15-target/debug/build/cc/f47325fd35ce8143/out \
  --tide-dep-dir /mnt/build/tide-agents/fx-zu4f.15-target/debug/build/cfg-if/7f02b23b970d439a/out \
  --tide-dep-dir /mnt/build/tide-agents/fx-zu4f.15-target/debug/build/libc/4c49c219f395f0df/out \
  --tide-dep-dir /mnt/build/tide-agents/fx-zu4f.15-target/debug/build/unicode-ident/debd0243e1d0e647/out \
  --tide-dep-dir /mnt/build/tide-agents/fx-zu4f.15-target/debug/build/getrandom/b3dcb6500f9fca6e/out \
  --tide-dep-dir /mnt/build/tide-agents/fx-zu4f.15-target/debug/build/proc-macro2/1f010a4d5cf75eaa/out \
  --tide-dep-dir /mnt/build/tide-agents/fx-zu4f.15-target/debug/build/rustls-pki-types/1a3718ff5f28605d/out \
  --tide-dep-dir /mnt/build/tide-agents/fx-zu4f.15-target/debug/build/utf_types/cc738870998629bc/out \
  --tide-dep-dir /mnt/build/tide-agents/fx-zu4f.15-target/debug/build/untrusted/9c1a8c0c452596ad/out \
  --tide-dep-dir /mnt/build/tide-agents/fx-zu4f.15-target/debug/build/quote/9bd8aac8db580ea4/out \
  --tide-dep-dir /mnt/build/tide-agents/fx-zu4f.15-target/debug/build/ring/1c82005960b2faa8/out \
  --tide-dep-dir /mnt/build/tide-agents/fx-zu4f.15-target/debug/build/utf8_iter/07626a5b00c33664/out \
  --tide-dep-dir /mnt/build/tide-agents/fx-zu4f.15-target/debug/build/percent-encoding/d0ddde42e654829c/out \
  --tide-dep-dir /mnt/build/tide-agents/fx-zu4f.15-target/debug/build/smallvec/33c5fbcfb79f35dd/out \
  --tide-dep-dir /mnt/build/tide-agents/fx-zu4f.15-target/debug/build/idna_adapter/25b3f71ecdfcf02b/out \
  --tide-dep-dir /mnt/build/tide-agents/fx-zu4f.15-target/debug/build/idna/37ec807e93d1a766/out \
  --tide-dep-dir /mnt/build/tide-agents/fx-zu4f.15-target/debug/build/form_urlencoded/4a42b89f31c459ce/out \
  --tide-dep-dir /mnt/build/tide-agents/fx-zu4f.15-target/debug/build/rustls-webpki/b05ff942139a2ca5/out \
  --tide-dep-dir /mnt/build/tide-agents/fx-zu4f.15-target/debug/build/syn/d41f88b89dfd81d6/out \
  --tide-dep-dir /mnt/build/tide-agents/fx-zu4f.15-target/debug/build/once_cell/34f91f5baba797d6/out \
  --tide-dep-dir /mnt/build/tide-agents/fx-zu4f.15-target/debug/build/bitflags/21113d36d14db8ae/out \
  --tide-dep-dir /mnt/build/tide-agents/fx-zu4f.15-target/debug/build/linux-raw-sys/4e3c60692274f452/out \
  --tide-dep-dir /mnt/build/tide-agents/fx-zu4f.15-target/debug/build/subtle/a7c30e9150b6c9d8/out \
  --tide-dep-dir /mnt/build/tide-agents/fx-zu4f.15-target/debug/build/tide-macros/d248bbce74dfe775/out \
  --tide-dep-dir /mnt/build/tide-agents/fx-zu4f.15-target/debug/build/rustix/b671a2e43dfdfa9d/out \
  --tide-dep-dir /mnt/build/tide-agents/fx-zu4f.15-target/debug/build/url/c2643ebfa5304869/out \
  --tide-dep-dir /mnt/build/tide-agents/fx-zu4f.15-target/debug/build/rustls/3e8ed9e94ce89229/out \
  --tide-dep-dir /mnt/build/tide-agents/fx-zu4f.15-target/debug/build/httparse/c8df9207061df7e1/out \
  --tide-dep-dir /mnt/build/tide-agents/fx-zu4f.15-target/debug/build/libm/f1d70f6c6e02e43b/out \
  --tide-dep-dir /mnt/build/tide-agents/fx-zu4f.15-target/debug/build/webpki-roots/4acd37ba3fda627c/out \
  --tide-dep-dir /mnt/build/tide-agents/fx-zu4f.15-target/debug \
  --out-dir /mnt/build/tide-agents/fx-zu4f.15-target/screen-out/final/RUSTSEC-2026-0039-capability \
  /mnt/build/eon-agents/worktrees/tide-fx-zu4f.15/target/work/ports/RUSTSEC-2026-0039/capability.rs \
  </dev/null
```

| Run | Start–end (UTC) | Exit | Counted TD codes (primary span) | Other diagnostics | Diagnostics SHA-256 |
| --- | --- | --- | --- | --- | --- |
| RUSTSEC-2026-0039 buggy | 06:40:03–06:40:05 | 1 | `TD0602` ×2 (`thread::spawn` 15:5, 29:5), `TD0501` ×3 (`Command::new` 17:17, 33:17; `thread::sleep` 30:9) | `E0433` ×3 (`thread` in `std`, 15:10, 29:10, 30:14), `E0599` ×2 (`Command::new`, 17:40, 33:40) | `222eb45e…f1f3` |
| RUSTSEC-2026-0039 capability | 06:40:05–06:40:08 | 0 | none | none | `1ba5545b…df2b` |

The buggy port's diagnostics, as the driver printed them (path shortened):

```text
buggy.rs:15:10: error[E0433]: cannot find `thread` in `std`: could not find `thread` in `std`
buggy.rs:29:10: error[E0433]: cannot find `thread` in `std`: could not find `thread` in `std`
buggy.rs:30:14: error[E0433]: cannot find `thread` in `std`: could not find `thread` in `std`
buggy.rs:17:40: error[E0599]: no associated function or constant named `new` found for struct `tide_std::process::Command` in the current scope: associated function or constant not found in `tide_std::process::Command`
buggy.rs:33:40: error[E0599]: no associated function or constant named `new` found for struct `tide_std::process::Command` in the current scope: associated function or constant not found in `tide_std::process::Command`
buggy.rs:15:5: error: TD0602: `std::thread::functions::spawn` is not admitted (Tide's std does not export it)
buggy.rs:17:17: error: TD0501: `std::process::Command::new` is not admitted
buggy.rs:29:5: error: TD0602: `std::thread::functions::spawn` is not admitted (Tide's std does not export it)
buggy.rs:30:9: error: TD0501: `std::thread::functions::sleep` is not admitted
buggy.rs:33:17: error: TD0501: `std::process::Command::new` is not admitted
error: aborting due to 10 previous errors
```

The capability-passing version printed nothing, and the driver exited 0.

**Consistency check on the buggy port (§6).** Every rustc error in the buggy
port has its primary span on a line that the capability diff changes: the
`std::thread` paths on lines 15, 29 and 30, and `Command::new` on lines 17
and 33.

**Trust artifact.** The admitted version's `capability.interface.json`
(SHA-256 `dbc20e74360d613ed6b0a1f5c7e4db955e94a9416ab04d2977ca592464371f43`;
read before any deletion):

```json
{
  "format": "tide-interface/1",
  "crate": "capability",
  "items": [
    {"item": "capability::run_optional_syncs", "kind": "function", "authority": {"bg": ["sync"], "clock": ["clock"], "proc": ["proc"]}}
  ],
  "holes": [
  ]
}
```

These are the same bytes as the trust artifact of fx-zu4f.8's two disclosed,
non-registered variants (research/47 §8, `dbc20e74…1f43`). That fits
section 1: the registered table form now reaches the result those variants
reached.

**Development run.** `dev.sh` ran once, 2026-11-19T06:39:18Z–06:39:48Z, with
the same command form and the same driver and `libtide_std.rlib` hashes.
Results: buggy exit 1 with the same codes and spans; capability exit 0 with
no diagnostic. Its log, `target/work/ports/dev.log`, has SHA-256
`0451107cfc7f21fe13c9d6010c4af2f49a69b6465813b6bf2680b1dab8976e53`. No fix
followed it.

**Build artifacts of the admitted runs**
(`target/work/ports/undeleted-artifacts.sha256`, paths under
`<target>/screen-out/`; all deleted unused at 2026-11-19T06:43:25Z, after a
hash check):

| File | SHA-256 |
| --- | --- |
| `dev-20261119T063918Z/RUSTSEC-2026-0039-capability/libcapability.rmeta` and its `.tide-cache/49a0b5e7…/` copy | `3d47c1f1deaabf793b849dece6feec03584155f00248ea37391bb56567c40a65` |
| `dev-20261119T063918Z/RUSTSEC-2026-0039-capability/capability.interface.json` and its `.tide-cache/49a0b5e7…/` copy | `dbc20e74360d613ed6b0a1f5c7e4db955e94a9416ab04d2977ca592464371f43` |
| `final/RUSTSEC-2026-0039-capability/libcapability.rmeta` and its `.tide-cache/57bffb5a…/` copy | `3d47c1f1deaabf793b849dece6feec03584155f00248ea37391bb56567c40a65` |
| `final/RUSTSEC-2026-0039-capability/capability.interface.json` and its `.tide-cache/57bffb5a…/` copy | `dbc20e74360d613ed6b0a1f5c7e4db955e94a9416ab04d2977ca592464371f43` |

## 9. The catch

This catch comes from a re-test. Tide's std had a gap: the documented idiom
`clock.sleep(d)?` did not compile inside a task body. The fx-zu4f.8 screen
exposed it. We fixed it (fx-qlek.18, `4c5b8c0`) and re-tested under a new
registration: this one, `e2fba81`. Its files existed by the time of
Bitcoin block 976536, whose header time (05:34:04Z) precedes the fix's
self-reported landing (06:35:00Z) by 61 minutes, within the timestamp's
tolerance. That the anchoring preceded the landing rests on that timestamp
and on Tide's own records; it is not independently established
(section 3). fx-zu4f.8 recorded this row as **not caught (port incomplete)**
(access failures: 3): leg 1 passed, and leg 2 failed on rustc `E0277` at the
table form `clock.sleep(d)?` (research/47 §7). That outcome stands, and so
does fx-zu4f.8's result of 0 caught of 87. Tide did not catch this row the
first time. The rest of section 1's disclosure applies here in full: the
pool was chosen after the rows were seen, and the outcome at R' was close to
predictable.

There is one catch, so fx-zu4f.8's write-up selection rule picks it without
a tie. Its `buggy.rs` has 27 lines without comments. Its parts:

- **The advisory.** RUSTSEC-2026-0039: "The `chrono_anchor` crate attempted
  to exfiltrate `.env` files to a server that was in turn impersonating the
  legitimate `timeapi.io` service."
- **TD codes on the buggy port.** `TD0602` ×2 on the two detached
  `thread::spawn` calls. `TD0501` ×3, on the two ambient `Command::new` calls
  and on the ambient `thread::sleep`.
- **The capability-passing version.** Its diff hunks are in section 8. Its
  declared authority is `bg` (sync), `clock` and `proc`, on
  `run_optional_syncs`.
- **Proof.** Both runs' command lines, environment, R', toolchain, file
  hashes, exit statuses and diagnostics are in section 8, from the final
  pass.
- **Negatives:**
  - Tide found nothing unaided. The code was already known to be malicious,
    and the port was written from Socket's quote of it.
  - The pool was chosen after the rows were seen, the std was changed
    because this screen exposed the gap, and the outcome was close to
    predictable. The change also reverses an earlier removal made for its
    inference cost (section 1).
  - Admission does not judge intent. The admitted version still runs the
    same processes with the same arguments, including the `.env` path. The
    difference is that its authority is now declared: `bg`, `clock` and
    `proc` in the trust artifact.
  - This is no malware-proofing claim. Malicious code can still try rustc
    soundness holes (`reviews/2026-11-17-red-team-opus.md`).
  - v0's limits:
    - Tide has no detached spawn, so the mapped `bg.spawn` tasks end within
      the caller's scope, while the source's threads were detached.
    - The child a `Command` starts has ambient authority (companion 20). The
      `proc` grant says that the code runs processes, not what those
      processes do.
  - rustc errors remained in the buggy port: `E0433` ×3 (`thread` in `std`)
    and `E0599` ×2 (`Command::new`). Each lies on a line the capability diff
    changes. The capability-passing version had none.
- This section contains no malicious logic. Any further write-up repeats
  section 1's disclosure and follows fx-zu4f.8's §9 rules.

## 10. The protocol, verbatim

`research/protocols/fx-zu4f.15-protocol.md`, SHA-256 `a15c9236c1bd0fd0beeb31457571f558d9ce9e02d686f2cf6f1e7f4ab0848065`:

# fx-zu4f.15 protocol: chrono_anchor and same-class rows, re-tested after the std conversion fix

Written on 2026-11-19 by the fx-zu4f.15 protocol builder (Claude Opus), in
worktree branch `fx-zu4f.15` from main `04dde61`, before fx-qlek.18 (the std
conversion fix) landed on main and before any port of this test was copied
or run. The "Frozen criteria" on bead fx-zu4f.15 stay binding. This file adds
operational detail and narrows them where it says so. It never widens them,
and it leaves fx-zu4f.8's caught rule unchanged.

This is a new pre-registered test. It does not revise, rerun or replace
fx-zu4f.8 (`research/47-malware-rescreen.md`), whose documented negative
stands as published, or fx-zu4f.5 (`research/37`). The record cites both as
prior work and edits neither.

## 1. Disclosure: why this pool, and when it was chosen

This text, or text with the same meaning, opens the record and any write-up.

- **The pool was chosen after fx-zu4f.8 saw the rows.** fx-zu4f.8 screened 87
  malicious-crate advisories and caught none. Its nearest miss was
  chrono_anchor (RUSTSEC-2026-0039): the buggy port drew counted TD codes, and
  the capability-passing version failed only on rustc `E0277` at the table
  form `clock.sleep(d)?`, because Tide's std at that revision had no
  conversion from `Cancelled` into `TaskError`.
- **Why it is re-tested.** Tide's std had a gap: the documented idiom
  `clock.sleep(d)?` did not compile inside a task body. The screen exposed
  it. The owner chose to fix the std (fx-qlek.18, an ergonomics fix on its own
  merits) and to re-test under a new registration (relayed owner decision,
  Lucca via tide-desk, 2026-11-19: "1").
- **What the result can and cannot show.** The outcome at R' is close to
  predictable: fx-qlek.18 adds a case proving the idiom compiles in a task
  body, and fx-zu4f.8 disclosed that two non-registered variants of this
  very port were admitted. A catch here shows that the re-tested row meets
  the caught rule at R'. It never shows that Tide caught the row the first
  time, and no text may say or imply that. fx-zu4f.8's result stays "0
  caught of 87".
- Whether a catch here counts as the RC's public catch is decided by
  fx-qlek.7's rule and its cold-reader pass, not by this test.

## 2. Incorporated by reference, unchanged

Each item below binds this test exactly as it bound fx-zu4f.8. RUN checks
every SHA-256 before first use and stops if one differs.

| Item | Where | SHA-256 |
| --- | --- | --- |
| fx-zu4f.8's registered protocol (sections 0 to 9: names, registration, R, sources, order of determination, the faithful port and its mapping table, the capability-passing version with §6's closed list of allowed diff lines and allowed fixes, commands, safety terms, the record) | `research/protocols/fx-zu4f.8-protocol.md`, committed `f04beb8`, OpenTimestamps proof `research/protocols/fx-zu4f.8-protocol.md.ots` (anchored in Bitcoin block 976452) | `4e2016dff7838f9bdcccf4b2cedbabf7051ab998e2a19af77f6e7347cfdc7e26`; proof file `ed68e64aed7841562106f75ec1c3fd1b0ce456597570e1af055f7441be4c6146` |
| fx-zu4f.8's frozen criteria, including the caught rule (fx-zu4f.5's, unchanged) | bead fx-zu4f.8, `description` field (closed 2026-11-19T04:50:38Z) | whole field `f20b56674098188f148c9270893ac11b3136dc72cfaf48e53c0ea7f12cb8a603`; the "**Caught rule**" block alone `7cb2479cf213cea88c8727a96d73cb99ef8173ac30453ec312ef43b8b3b89d57` |
| RULING-0039 (tide-lead, 2026-11-19 03:54Z): the table form of `thread::sleep(d)` is `clock.sleep(d)?`; `let _ =` is not an allowed diff line; `.map_err(..)` only for a `?` the source already had | `~/.cache/eon-agents/tide-lead/fx-zu4f.8-work/RULING-0039.md`; stated in `research/47` §2 | `794f6885e978cec817e02eef50fc5dc374c73c6ca9b1931c21ca43837b2fd8d2` |
| RULING-0175 with its correction of 2026-11-19 (the definition of "affected function"; item 3 corrected) | `~/.cache/eon-agents/tide-lead/fx-zu4f.8-work/RULING-0175.md`; stated in `research/47` §2 and deviation D2 | `ff5992e1effd5fbfc4c38734d1ade55b95491477f829564abc1d74985b5882c0` |
| tide-desk's archive release condition, amended, 2026-11-18 14:04 (its four limits) | bead fx-zu4f.8, comment 1106, 2026-11-18T14:04:39Z | `7ad118994870800cec644936f2bd77cfafc4f229e19ad98d3460ec8648e5d836` |
| tide-desk's confirmation of how that condition applies (it binds over port fidelity; a row that cannot meet it without changing its effect-carrying calls is "unresolved (archive condition)") | bead fx-zu4f.8, comment 1312, 2026-11-19T04:20:42Z | `cd8d3ba0c32d326434ba485122e510f95efa9c3a8a8d05ccffe40ad66d9871d0` |
| fx-zu4f.8's committed record, source of the pool and of the carried-over determinations (section 5) | `research/47-malware-rescreen.md` at `fc43b37a8852092eecf9fe832e5e06d1a26d10e9` | `6c6111b7789d0fedd34a4439c403c103b2f8a57442360afe9083660ffc3b896e` |

How the bead-field digests are taken: the SHA-256 of the field's UTF-8 text
exactly as `br --no-db show fx-zu4f.8 --json` (`description`) or
`br --no-db comments fx-zu4f.8 --json` (`text` of the comment with that
`id`) returns it, with no trailing newline added. The caught-rule block is
the description's text from `**Caught rule**` up to, not including,
`**Safety terms**`, with trailing newlines removed.

The caught rule, quoted from that block for the reader (the block above
binds): a candidate is caught when a named TD code fires on a faithful Tide
port of the malicious code, and the capability-passing version, which
receives the authority it uses explicitly, is admitted. fx-zu4f.8's protocol
§4 makes it operational: leg 1, a counted TD code on an in-class operation of
an affected function of the buggy port; leg 2, a clean admission (driver exit
0, no error diagnostic) of the capability-passing version.

Not carried over: fx-zu4f.8's post-registration UDP and resolve reporting
addition (bead fx-zu4f.8, 2026-11-19T00:14:56Z). It added outputs to that
round only.

## 3. The pool and the selection rule

**Rule.** The pool is chrono_anchor (RUSTSEC-2026-0039), which the bead
names, plus every row of fx-zu4f.8's committed record whose leg-2 failure
was the same conversion class. A row of `research/47` §7 (at `fc43b37`) is
in that class when all of these hold:

- C1. Its `capability_run` cell records a run of the capability-passing
  version that exited non-zero (`exit N; ...`, N not 0). Leg 1 passed, and
  leg 2 failed on a run.
- C2. Every diagnostic that cell lists is rustc `E0277`. No other rustc
  error and no TD code remains.
- C3. Each `E0277` is annotated as a `?` that cannot convert a type X into a
  type Y (the enclosing error type).
- C4. Each such X is a type that Tide's std defines at fx-zu4f.8's run
  revision R = `972aba0ee5917a0a70e59f6b2e7429e6e2378e74` (`pub struct X` or
  `pub enum X` under `crates/tide-std/src`): a std error type.
- C5. Record §2's RULING-0039 list of rows "where a table form brings in a
  `?` the source lacked" names the row's crate, or the first word of its
  incident. So the failing `?` is a table form's, not one the source had
  (for which §6 would have allowed a `.map_err`).

**Script.** `research/protocols/fx-zu4f.15-pool/select.py` applies the rule
mechanically. It runs from the repository root as
`python3 -I research/protocols/fx-zu4f.15-pool/select.py`, reads the record
at commit `fc43b37a8852092eecf9fe832e5e06d1a26d10e9` with `git show`, checks
its SHA-256, looks up C4's types with `git grep` at R, and prints every row that recorded
a capability-passing run, with each criterion's verdict and the row's
evidence lines in the record, then the pool. It compiles, runs and fetches
nothing else, and prints no time, so its output is byte-stable. Its output
is committed beside it as `select-output.txt`.

**Result at registration.**

- 8 of the 87 rows recorded a capability-passing run (C1). The other 79 did
  not: 76 never reached a capability-passing run (no port, or stopped on
  paper), 2 failed leg 1 (uniswap-utils,
  finch-rust), 1 stopped at capability absent on paper (evm-units).
- amaperf, checked by the rule and not by hand: its seven rows
  (RUSTSEC-2023-0097 to -0103, one shared port) pass C1 and C5 (amaperf is
  in RULING-0039's list for `Url::parse(..)?`, which converts without
  error), and fail C2: their only errors are `E0433` ×2 on `serde_json`, a
  third-party non-authority crate. Not selected.
- chrono_anchor passes C1 to C5. Its cell reads "exit 1; E0277 x1 (34:44:
  `?` cannot convert Cancelled to TaskError)"; `Cancelled` is defined at R in
  `crates/tide-std/src/task.rs:1871`, named in RULING-0039's list. Record
  lines 450 (§7 row) and 618 (§8 final pass).
- **POOL = { RUSTSEC-2026-0039 }** (1 row). The bead names it, and the rule
  selects it on its own.

**Cross-check against fx-zu4f.8's work evidence** (private, not committed;
done by the protocol builder before registration). No disagreement:

- `fx-zu4f.8-work/screen.tsv` (SHA-256
  `ed96cd00da3cfc88c140eb4cef90638c64ebeb486b7fe486724ae3e40c7f28aa`): 87 rows; the
  `capability_run`, `buggy_codes` and `outcome` cells equal the committed
  table's for all 87 (the record's outcome cells add the access-failure
  note). The same 8 rows have a run; the same values.
- `fx-zu4f.8-work/ports/final.log` (SHA-256
  `f23eb58e817685abf12e7c00214944f72564748dde5e742d91e06ccfda1298f6`, as
  research/47 §8 records): the only capability-passing runs are RUSTSEC-2023-0097
  (`exit 1; E0433 x2`, `serde_json` at 29:15 and 29:35) and
  RUSTSEC-2026-0039 (`exit 1; E0277 x1`, at 34:44: "`?` couldn't convert
  the error to `tide_std::task::TaskError`: the trait
  `tide_std::convert::From<tide_std::task::Cancelled>` is not implemented
  for `tide_std::task::TaskError`"). Line 34 of the
  chrono_anchor capability port is the table form `clock.sleep(..)?`.
- `fx-zu4f.8-work/ports/r1.log` (SHA-256
  `58bc8c019ffb844dbcdbf343f3b36d32ccdaee7ca5f8f9499caa02aef680afe2`): no
  capability-passing run. `ports/dev.log`'s one `E0277` is chrono_anchor's
  development run of the same port, not an outcome.
- The port files' SHA-256s in fx-zu4f.8's work copy equal research/47 §8's
  table (section 5 below).

## 4. The run revision R'

- **Choice.** R' is main's HEAD at the moment tide-lead sends `go`.
  tide-lead sends `go` only after fx-qlek.18 is closed and its landing
  commit is an ancestor of that HEAD, and only after the registration steps
  of section 6. R' is that landing commit when main has not moved since;
  otherwise the later HEAD, and the record says which. The `go` message gives
  R''s full SHA, and RUN copies it into the record. tide-lead chooses R'
  without any result of this test.
- If fx-qlek.18 is closed without landing a conversion that makes the
  documented idiom compile in a task body, this test is not run, and the
  bead records why.
- Everything fx-zu4f.8's §2 says about R applies to R': frozen once the
  first port file is copied; the capability set at R' (present when
  `design/reference.md` at R' lists the operation without "planned" or
  "partial", and its owning bead's landing commit is an ancestor of R');
  tree identity of `crates`, `Cargo.toml`, `Cargo.lock` and
  `rust-toolchain.toml` with R', and a clean tracked tree. The record names
  fx-qlek.18's landing commit and quotes the std conversion it added.
- **Table forms at R'.** The `thread::sleep(d)` row's form is the one
  companion §14.2 gives at R' (RULING-0039 item 1, read at R'). At
  registration it is `clock.sleep(d)?`. If R' gives a different form, the
  record says so, and that form is the table form.

## 5. How fx-zu4f.8's protocol is read for this test

Only names, the pool, the revision and the places listed here differ. Each
item substitutes a name or narrows; none widens.

1. **Names (§0).** RUN is this test's run builder. WT is its worktree;
   WORK = `WT/target/work`; TARGET = `/mnt/build/tide-agents/fx-zu4f.15-target`.
   PRIOR = `/home/lucca/.cache/eon-agents/tide-lead/fx-zu4f.8-work/`, read
   only; nothing in it is executed. POOL is section 3's pool. RUN checks POOL
   at start by rerunning `select.py` and comparing its output byte for byte
   with the committed `select-output.txt`, and stops if they differ.
2. **Registration (§1).** tide-lead records this file's SHA-256 (and the
   pool files') on fx-zu4f.15. RUN starts only after section 6's steps and
   `go`.
3. **Sources (§3), narrowed.** No new search and no new fetch. Each pool
   row's sources are those fx-zu4f.8 collected and recorded for it
   (research/47 §7: chrono_anchor reads S020, S469, S470, S471, S473, S476 and
   the archive copy socket-3; code quoted by socket-3). Nothing is fetched
   again, so no evidence can enter after the result is known.
4. **Order of determination (§4).** Steps 1 to 5 (advisory, sources, class,
   faithful port on paper, safety gate) carry over from fx-zu4f.8's record
   for each pool row, unchanged. Steps 6 to 8 run again at R': the buggy
   port's run (leg 1), the mapping on paper at R', and the
   capability-passing version's run (leg 2). The outcome labels and the
   counted TD codes are §4's, read at R' (including a code R''s
   `conformance/rules.md` may give ambient `Command::new`).
5. **The ports (§5, §6), narrowed.** RUN uses fx-zu4f.8's final port files
   byte for byte, copied from PRIOR into `WORK/ports/<ID>/` and checked
   against research/47 §8:
   - `RUSTSEC-2026-0039/buggy.rs` `3c296fb448ab2742e979d8cdc5373171b7a9b4a37e88da2ce3d933fcd2e4a110`;
   - `RUSTSEC-2026-0039/capability.rs` `51f348513fc80a727a35ce416fe304a0e4bf6ccfa8c5067d7207887c69d179a4`;
   - `RUSTSEC-2026-0039/capability.diff` `c8a002fb329e6695b179556943bbf85cef4dec2bfbbc8b6c3be40429d46dbdf7`.

   No port is rewritten. §6's allowed fixes stay available, unchanged, for
   a rustc error that R' itself causes; each one is recorded as a numbered
   diff against these hashes with the run that prompted it. RUN rechecks
   the capability diff line by line against §6's closed list before the
   final pass, as §6 requires.
6. **Commands (§7).** The harness is fx-zu4f.8's `PRIOR/ports/screen.sh`
   (SHA-256
   `0b45dcbc5a4231059ae794b669ece349a0a3ed7cb2f723586e9cee446ffca043`,
   matching research/47 §8's `0b45dcbc…a043`), copied to
   WORK with only its log path, output root, run list and labels changed, as
   fx-zu4f.8's `screen-r1.sh` was. Every other §7 term applies: heavy.sh,
   the load check, timeouts, the admission command and its forbidden flags,
   the log's contents, development runs in `WORK/ports/dev.log`, and one
   final scripted pass whose log is never overwritten.
7. **Safety (§8) and the archive condition.** chrono_anchor's ports derive
   from archive copy socket-3. The condition of section 2 binds them: hard-
   coded RFC 5737 or 6761 endpoints; nothing archive-derived is executed
   (admission runs only, `--emit=metadata`); authority shape only; work in
   `target/work`. Any build artifact an admitted run leaves (`.rmeta` and
   its cache copies) is listed with its SHA-256 in
   `WORK/ports/undeleted-artifacts.sha256` and deleted unused, by tide-lead
   if RUN's tools refuse. For an admitted version, the record keeps the
   `interface.json`'s SHA-256 and declared authority (§6's trust artifact),
   read before deletion.
8. **The record (§9).** A new file in `research/`, numbered when it lands,
   reported as a new pre-registered test. It contains, in order: section 1's
   disclosure; the result; the registration digests and OpenTimestamps
   anchors; R' and the capability set at R', with fx-qlek.18's landing
   commit; the method, with section 5's narrowings; the counts over this
   pool (never a rate); the pool's rows in §9's row format; the port proof
   (both runs' command lines, environment, toolchain, file hashes, exit
   statuses and diagnostics, from the final pass); the catch or the
   documented negative; and this protocol verbatim. fx-zu4f.8's protocol is
   cited by path and digest, not reproduced again. A catch's write-up, if
   any, follows §9's rules and negatives and repeats section 1's
   disclosure. No port body, payload or archive-derived code is committed;
   §9's `diff -U0` hunks (changed lines only) may be shown.

## 6. Registration and stop

1. The protocol builder commits this file and
   `research/protocols/fx-zu4f.15-pool/` (`select.py`, `select-output.txt`)
   on branch fx-zu4f.15, byte-identical to WORK, writes
   `WORK/PROTOCOL-READY` with every SHA-256, and stops.
2. tide-lead stamps the files with OpenTimestamps, records their SHA-256s on
   fx-zu4f.15, and upgrades the proofs once anchored, all before fx-qlek.18
   lands, so the order can be checked: the anchoring block's time precedes
   fx-qlek.18's landing commit.
3. tide-lead sends `go`, naming R', only after fx-qlek.18 lands. RUN checks
   every digest in sections 2 and 5 and this file's registered digest
   before first use, and stops on any mismatch.
4. The branch lands by `git merge --no-ff` after GOVERNANCE.md's Merge Rule
   for docs (Codex pass, then an Opus review round). Nothing archive-derived
   or private is committed.
