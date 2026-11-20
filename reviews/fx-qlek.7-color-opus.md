Model: Claude Opus 5.5 · Date: 2026-11-19 · Candidate f7c2a9c (tide-admit --color, Opus review)

## Verdict
APPROVE. No blocking findings. Read-only review of `git diff dc0997f f7c2a9c -- . ':!.beads'`
plus the surrounding parser, re-exec, cache and emitter code. Nothing was built or run (per brief).

## What I checked (evidence)
- Parsing (crates/tide-admit/src/invocation.rs:308-354). `--color` joins the existing value-flag
  arm. So `--color V` and `--color=V` both work. A missing value gives "missing value". An empty
  or `-`-leading value is "invalid value". A value outside auto/always/never, a repeated
  `--color`, and `--color=always=x` are all "unsupported argument", exit 2, like the other
  single-valued options. Because no value may start with `-`, the token `--color` can never be
  another flag's value. That makes `without_color` (:235-246), which drops `--color` and the next
  token, agree exactly with parse.
- Process chain. The outer process re-execs with `.status()`, so stderr is inherited and
  `colored()` sees the real tty (:1244-1252). The cached path resolves auto once (:1311), keys on
  it, and passes the compile child exactly one `--color=always|never` (:1351-1353). The child's
  own `colored()` then equals the parent's bool, so the stage-key check
  (`compiler snapshot changed`) still matches. `--tide-test-library` compiles in the
  re-exec'd process on the inherited stderr, and `color` is copied to the paired library (:683).
- No identity change. `--color` is kept out of `rustc_args`, so the build identity, the receipts
  and the REPLAY-2 options are unchanged. In a recorded manifest, `options` hold only rustc args
  and the `admission:` string. Only the cache key gains the `color` record (:1310-1312).
- Byte identity. With color off, the emitter gets `ColorConfig::Never`, exactly as before
  (diag/labels.rs:218-223). Every piped caller resolves to Never: the conformance runner
  (`.output()`), tests/examples.rs, and the paper's recorded transcripts. The TD text, codes,
  labels and exit statuses are unchanged. The Labels wrapper sees only `DiagInner`, never
  rendered bytes.
- The cache keeps color modes apart. The key gets the resolved bool, so auto-on-a-pipe and never
  share an entry, which is correct because they produce the same bytes.
  `cached_warnings_keep_the_color_asked_for` pins this, and the builder's falsifier showed it
  fails without the key record.
- Rustc's own color. The session's `opts.color` stays at rustc's default, but `psess_created`
  replaces the dcx emitter. Only rustc's early diagnostics (bad CLI args) use opts.color, and the
  driver builds those args itself. So there is no forwarded-diagnostic interplay to worry about.
- Admission filter. This is not a substantial toolchain addition. It is a presentation-only flag
  with rustc's spelling and three values. It touches no language rule, no build identity and no
  admission verdict. The filter does not apply.

## Blocking findings
None.

## Non-blocking
1. `--color=always` does not color the eprintln!-printed rejections. Guard::reject TD0911
   (invocation.rs:1030-1039), TD0520 (:1006) and TD0915 (:779-782) bypass the emitter and stay
   plain. Under a tty, one run can show a colored `error` for dcx diagnostics and a plain one for
   these. docs/driver.md says `--color` "only styles diagnostics", which reads as all of them.
   Minimum fix: either say in the doc that the walk's early file and build-script rejections stay
   uncolored, or route these through a small helper that bolds/reds `error` when `colored()`
   holds. The doc sentence is cheaper. The clip's TD0206 goes through `diags.report`, so the clip
   is unaffected.
2. Auto ignores NO_COLOR and TERM=dumb. The process that decides runs after `env_clear`, so it
   cannot see them. rustc's auto (anstream/termcolor) honors NO_COLOR, so "rustc's spelling"
   matches rustc's flag but not its auto semantics. This is not a regression, since the old
   output was never colored. The doc is honest because it claims only the tty test. Minimum
   follow-up: resolve auto in the outermost process (before the re-exec clears the environment)
   to tty && NO_COLOR unset/empty && TERM != dumb, and add one doc clause. Alternatively, add a
   doc clause now that NO_COLOR is not read.
3. Test coverage gaps: the tty branch of auto and the `--tide-test-library` color path have no
   automated test. The tty branch is covered only by the builder's script(1) probe. A pty test
   costs more than it proves here, so these are acceptable.
4. Nit: invocation.rs:1350's comment says auto "is decided here". It is decided at :1311. Harmless.
5. Nit: `--help` (:1238) lists `--color auto|always|never` among options the summary line calls
   "fixed build options". Fine as is.

DONE
