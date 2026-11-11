The round-5 fixes check out; one minor leftover remains, which I record below as non-blocking.

Opus 5.5 (`claude-opus-5-5`) · 2026-11-11

**Verdict: approve.** No finding blocks landing.

I reviewed candidate `04f6378` on base `10991bd`. The SHA-256 of `git diff 10991bd HEAD` is `b198d5be…87567`, which matches the brief. I read the fix delta `git diff b0c3854 04f6378` in full and worked read-only. I ran five driver probes with the candidate's `target/debug/tide-admit` under `nice -n 19` and `CARGO_BUILD_JOBS=4`. Load was about 3.2 with 16 GB available, and the quiet window had ended at 13:25 local. No file under `crates/` is newer than the binary. I did not open `trials/e1-checked/`.

**Finding 1 (blocking, FX0210 false labels): resolved.**
- **Returned view probe:** `View{s:&t}` returned from `make` now gives E0515 alone, with no FX0210.
- **Vec push probe:** pushing `View{s:&t}` into `Vec<View<'static>>` now gives E0597 alone, with no FX0210.
- **Real self-borrow:** `Doc{word:&text, text}` still gets FX0210 on both its E0515 and its E0505. `self_borrow_reject.rs` keeps its FX0210 mark.
- **False marks removed:** `drop_dc_reject.rs` no longer has REF-0210 or FX0210. The four `field_*` rows in `provenance_matrix_0597_reject.rs` no longer have FX0210. Neither probe's error is labelled any more.
- **New rule (`surface.rs` `self_borrow_literal`, `borrow_owner`, `retains_loan_root`):** it requires an unadjusted, non-Copy whole local in one field, plus a certain `&`, `&mut` or receiver-tied borrow of that same local in another field. Uncertain cases get no label: borrows through stored references, raw-pointer results, and Copy values. The new evidence and raw-pointer fixtures cover these cases, each with an accepted twin.
- **FX0203 unaffected:** the `direct_borrow` extraction keeps FX0203's chain steps the same.
- **Docs:** `conformance/README.md`, `README.md`, the REF-0210 row in `conformance/rules.md` and the `diag.rs` comment now describe the certain-only rule.

**Finding 2 (non-blocking, research/27): resolved.** `research/27-fx-rule-enforcement.md:112-114` adds the "Adopted with changes (fx-u65.2)" note, which covers the four required points and links to `conformance/README.md`. Line 4's date now matches line 2 (2026-11-11), and the trailing `DONE` is gone. Nothing else in the report changed.

**Findings 3, 4 and 8:** no change here, as you decided. They are recorded on fx-u65.3, fx-djp and fx-u65.5.

**Nits 5, 6 and 7: done.**
- **Nit 5:** the comment at `rust-toolchain.toml:3-4` now says the label layer's message and span matching are reviewed whenever the pin moves.
- **Nit 6:** `diag.rs:74` uses the requested wording.
- **Nit 7:** `surface.rs:127-130` reports FX0201 at the first lifetime parameter. `tests/invocation.rs:36` asserts the error is at `:2:17`, the `'a`.

**Question 4 (did a fix add a false label or a regression?):** The fixes added no new false label. Nothing regressed in FX0201, FX0203 or the documentation. The author's suites on `04f6378` report workspace 143 passed and conformance 253 PASS / 19 AWAITS / 0 FAIL. I did not rerun the suites.

**New finding (minor, non-blocking): a real self-borrow literal also labels other locals' errors.** This is at `crates/tide-admit/src/diag/labels.rs:121-126`, which labels any E0505, E0515 or E0597 whose primary span lies inside a recorded literal.
- **Probe:** `v.push(Doc{word:&text, other:&y, text})` into `Vec<Doc<'static>>` gives FX0210 on three errors. Two are correct: the E0597 for `text` and the E0505. The third is wrong: the E0597 for `` `y` does not live long enough `` (`e.rs:2:121`).
- **Returned version:** the same literal returned from a function also labels the E0515 for `y`.
- **Why it matters:** the label's sentence is true of the literal, but it is attached to an error with a different cause. After the writer fixes the self-borrow, the `y` error stays.
- **Not new:** the old heuristic labelled this too. It is narrower now, because it needs a real self-borrow in the same literal.
- **Cheap fix:** label only the E0505 whose primary span is the owner field. A self-borrow always produces that E0505, because the stored borrow is used when the struct is built, which comes after the move. Alternatively, require E0597 spans to fall inside the recorded borrow field and drop E0515.
- **Where:** fix it here if that is cheap. Otherwise record it on fx-u65.3 with the table-driven label seam.
