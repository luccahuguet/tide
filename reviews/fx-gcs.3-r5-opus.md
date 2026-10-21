# Tide review, fx-gcs.3 round 5 (Opus): converge round

Candidate: commit 5b80ed0 on main 755d86e (worktree tide-fx-gcs.3). I
reviewed the whole change (`git diff 755d86e 5b80ed0`) with fresh eyes. The
check covered every authority case against the full reference, not only
the rule it targets, plus the design edits, rules.md, README, run.sh, and
the stub. This round fixes what it finds. The fixes are uncommitted in the
worktree.

## Verdict

fixed

Round 4's fixes are in the commit and hold up. The README no longer says a
case holds no grants. 03 §3.1 cites DYN-2. Both granted and ungranted
`unsafe impl` lines in `unsafe_granted.rs` and `unsafe_ungranted.rs` carry
`// SAFETY:` comments. I found one more case that round 4's `FX0902`
reading leaves exposed, plus one formatting leftover. Neither changes a
design text or a rustc verdict.

## Findings and fixes

**F1. `unsafe_impl_pure.rs` had no `// SAFETY:` comment on its granted
`unsafe impl`.** Round 4 F3 reads reference 11's `FX0902` rule ("each
block needs a `// SAFETY:` comment") as Rust's `undocumented_unsafe_blocks`
lint does, which covers `unsafe impl` too. On that reading it added the
comment to every other `unsafe impl` in the corpus, and fx-gcs.2's
`send_unsafe_impl.rs` already had one. `unsafe_impl_pure.rs` runs under
`//@ grants: unsafe`, so a checker reading `FX0902` that way would add an
unmarked `FX0902` on line 9. The case would then fail for a reason
unrelated to the overlap it tests.
*Changed:* line 9 now has `// SAFETY: present, so the overlap is the only
error.` above it, the same wording `send_unsafe_impl.rs` uses. Its verdict
is unchanged: AWAITS CHECKER, `E0119 FX0402`.

**F2 (nit). README's rule-ID paragraph had a 95-character line.** This was
left over from round 4 F4's edit ("noted where the rule stood"). Every
other prose line in the README fits in 80 characters.
*Changed:* I rewrapped the paragraph. The text is the same.

## Not changed

- **dead_code warnings** (`items`, `fs`, `hits`, `p`, and `Token.0` never
  read) in `afree_const_generic`, `pure_bound`, `pure_supertrait_impl`,
  `unsafe_granted`, `unsafe_ungranted`, and both `trusted_pure` cases.
  Reference 12.2 makes only unused imports errors (`FX0913`), and no rule
  rejects unread fields. When I compiled every authority case in both
  modes, these were the only warnings. There was no unused import and no
  unused `Result`.
- **`let _ = b.with(..)` in `lock_closure_nested.rs`** discards a
  `Result`. Reference 7.4 says "`let _ = r;` discards on purpose", so the
  line draws no `FX0702`.
- **`#[unsafe(no_mangle)]` is not in reference 2.3's kept-attribute
  list.** Reference 2.3 rejects only the attributes it names, and both
  reference 11 (`#[unsafe(..)]`) and 03 UNSAFE-1 name this spelling as a
  grant-gated hole. So `unsafe_granted.rs` accepts it correctly.
- **`fn main(root: Root)` without the attribute (`root_main_unmarked.rs`)
  draws only `E0580 FX0512`, not also `FX0503`.** Read literally, ROOT-1
  allows naming `Root` only in the attributed `main`. But the companion
  gives this exact spelling `FX0512` alone, with the safe fix that adds
  the attribute. That is the more specific text, and the fix makes the
  naming legal.
- **`admit_float.rs` exercises PAR-3's allowlist part but names only
  ADMIT-1.** Earlier rounds left PAR-3 to the tasks bead. That is a
  scope call, not a defect.
- The earlier rounds' open items still stand, all out of scope or at the
  core budget: `asm!` is `FX0103` while RUN-2 uses it under a grant;
  reference 6.2 says "only in `main`'s signature"; reference 11's
  `std::sync::raw` row gives no code; 02 §9 cites ADMIT-1 without a
  chapter.

## Checked

- **Rule text:** AGENTS.md's local rules, GOVERNANCE.md's Merge Rule, the
  bead (`br --no-db show fx-gcs.3`), conformance/README.md, and the
  reviews from rounds 1 to 4.
- **Every authority case against the full reference:**
  - unused imports (`FX0913`) and orphan placement (`FX0401`, the user
    `Into` impl sits in `Key`'s module);
  - overlaps (`FX0402`, in `pure_overlap` and `unsafe_impl_pure`) and
    `// SAFETY:` comments (`FX0902`, which gave F1);
  - lossy casts (`FX0301`: `u8 as u32` in `pure_deep` widens; the
    reference-to-pointer casts are not numeric);
  - naming `Root` (ROOT-1: `Root::sim` is allowed, and `root_forge` forges
    `Fs`);
  - the macro list (reference 2.1, including `println!`, `dbg!`, and
    `global_asm!`), and `let _ =` discards (7.4);
  - glob imports (`use super::*` is fine, per 12.2), and the kept
    attributes (2.3).
- **Design consistency:** I grepped design/ for FX0402, FX0502, FX0503,
  FX0517, FX0518, FX0520, FX0901, FX0902, `thread_local`, `lazy_static`,
  `const fn`, `Into`, and SEND-2. 01 D7/D18, 02 §8.1/§8.4/§9, 03 §1.2,
  §2.1, §2.2, RUN-4, §3.1, §3.2, and §3.5, reference 2.1, 2.3, 6.4, 6.6,
  and 11, and the companion all agree. SEND-2 survives only in 02 §8.1's
  retirement note.
- **README and run.sh agree:** the `//@` directives (`grants: unsafe`,
  `profile: release`, any other rejected), `-Zallow-features=` under
  `RUSTC_BOOTSTRAP`, bin detection, and the exit codes.
- **Coverage:** `run.sh --trace` gives an accepting case and a rejecting
  case for every in-scope ID: CAP-1..3, ROOT-1..5, GLOBAL-1..4 and 6,
  SINK-8 and 10, ADMIT-1 and 2, UNSAFE-1, AFREE-1 and 2, LOCK-1, 4, and 5,
  and PURE-1 and 2.
- **Runs:**
  - On 5b80ed0 (load 8.7 on 16 cores), `conformance/run.sh` exited 0 with
    107 PASS, 56 AWAITS CHECKER, and no FAIL.
  - After the fixes (load 11.3), it exited 0 with the same 107 PASS, 56
    AWAITS CHECKER, and no FAIL. `unsafe_impl_pure` still awaits the
    checker on `FX0402`.
  - `--trace` exited 0 both times.
  - `git diff --check` is clean, and no authority case line is over 100
    characters.
- **Budget:** `scripts/count-tokens.sh design/reference.md` gives 24,992.
  This round does not edit the core, and the companion's count line
  matches. rules.md's snapshot line (`a3b0177`) is untouched.
