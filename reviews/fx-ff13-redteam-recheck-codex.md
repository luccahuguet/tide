Model: gpt-6.1-sol (high) · Date: 2026-11-18 · Red-team re-check b772256

## Verdict

**Sound with stated limits.** F1 is fixed; F2–F4's limit texts and implementation guards are accurate. The fixes introduce no new authority hole or materially misleading statement in the reviewed surfaces. The reference proposal independently measures **24,989 tokens**, passing the requested **24,990** cap. This is the finder's scoped re-check of F1–F4 and the changes made to address them, not a fresh full red-team pass or implementation approval.

Candidate: `b772256a4454451d71a88537c345a99fd8a13ff3`, base main `d58f3e425914d717360e04d3d114a926aa7ed4f8`; previous reviewed candidate `9ad42b9c559c8313cffa0ac9192e065a83ebebf7`.

- SHA-256 of `git diff 9ad42b9..b772256`: `14c25a38d008dc0efe41ffa2cec2469be34909a43c29260a52342634dd4522b3`.
- SHA-256 of `git diff d58f3e4..HEAD`: `ab79e174c850353b0b990058d87aa09d10aea9f4dd7ae10f8db9dd2a587f99dc`.
- Updated `target/work/reference-proposal.diff` SHA-256: `98f018d29c6767c5da319a8d64583298604a94019c829d0cce331b9ae1d64e3e`.

Read the complete round diff, the updated proposal, the original findings, and the author's “Round redteam fixes” claims, then checked their cited owning text and conformance requirements. The round changes five documentation files; no implementation under `crates/` changed. The tracked reference remains byte-identical to base main.

## Finding status

### F1 — Fixed (blocking finding closed)

The real boundary is now stated in every requested teaching surface:

- `design/01-design.md:512`–520: outbound peer operations only; bind-time resolution and unlisted inbound peers remain possible; an empty list is not network silence; use `only([..]).no_bind()` for outbound-only grants.
- `design/03-authority-concurrency.md:241`–250: the same exceptions, plus explicit refusal by `no_bind` before resolution. The earlier TCP paragraph's retained bind semantics remain consistent.
- `design/reference-companion.md:726`–738: the same boundary and limits alongside the name/IP, rebinding and data-channel explanations.
- `research/42-net-get-v0.md:278`–285 and `:339`–341: the host rule and pre-lookup guarantee explicitly exclude bind-name resolution. Its trust-diff interpretation at `:693`–698 now warns that a displayed `only` without `no_bind` retains arbitrary bind DNS and unlisted inbound communication. Its local-reasoning and soundness text at `:713`–716 and `:723`–736 agree, including the authorized-handle exception.
- Updated proposal, resulting reference §6.7 at scratch-copy lines 505–509: outbound lookup, bind-name resolution even on failure, unrestricted inbound peers, empty-list limit and the `no_bind` composition all appear in the core reference.

`conformance/rules.md:103` (NET-HTTP-5) and `research/42-net-get-v0.md:822`–825 require `only([]).no_bind().bind(("unlisted.test", 8080))` to return `PermissionDenied` with **zero observed resolver calls on the real backend**. This would falsify the intended early-refusal boundary if the resolver ran before `no_bind`. It correctly refuses to use a simulated bind error as proof. It does not mistakenly require `only([]).bind(...)` to refuse, so the fix preserves the chosen bind policy rather than silently changing it.

No path by which an already-held wider capability, listener or stream could be revoked is newly promised. The fix describes the rights of the delegated Net; it does not claim that narrowing destroys other authorized handles.

### F2 — Fixed (non-blocking limit and requirements addressed)

`research/42-net-get-v0.md:641`–653, `design/reference-companion.md:721`–725 and the companion's peer table at `:438` now distinguish application-response simulation from TLS certificate validation and wire parsing. Simulated HTTPS success asserts authenticated transport without proving it. That is an accurate statement of the selected simulator.

`Request::url()` now exposes the request URL **with its fragment removed**. This changes the callback's view, without requiring a public setter or changing the original immutable `Url`. The rule is repeated in NET-HTTP-3 (`conformance/rules.md:101`), with a `#secret` witness at `research/42:817`–819. The simulated peer can no longer rely on a fragment that a real server never receives.

The 64 MiB body limit explicitly applies to **every Reply**, including `Reply::ok`; no constructor is an exemption. Research §9, companion 17/21 and NET-HTTP-3 agree. Exactly 64 MiB succeeding and one byte more returning `Io(InvalidData)` are required on simulated and real response paths (`research/42:820`–821; NET-HTTP-2/-3). Replies with no body do not create a body-limit escape. The real untrusted-certificate failure before HTTP data is also required (`research/42:813`–816; NET-HTTP-2). None of these changes bypasses the existing authority check before a peer callback.

### F3 — Fixed (non-blocking limit addressed)

`research/42-net-get-v0.md:430`–450 removes the categorical claims about cache isolation and queried-name suffixes. It distinguishes Tide-owned caches from system resolver caches and states configured alias substitution, DNS alias following and traffic continuing after cancellation. The private resolver's abandonment description at `:469`–471 now explicitly includes that continuing traffic.

`design/03-authority-concurrency.md:277`–292 and `design/reference-companion.md:697`–704 agree. These limits accurately describe the retained operator-controlled resolver mechanism. The changes neither add attacker control over resolver configuration nor replace the pre-lookup check for each outbound peer operation. They also avoid implying network quiescence when only the waiting Tide caller has stopped.

### F4 — Fixed (non-blocking guard addressed)

`research/42-net-get-v0.md:505`–512, `design/03-authority-concurrency.md:278`–280 and `design/reference-companion.md:718`–720 require `NoKeyLog`, disabled resumption, no `KeyLogFile` construction and ignored `SSLKEYLOGFILE`. They correctly distinguish this from disabling rustls's `logging` feature.

Re-read the pinned published `rustls-0.23.46` source in the existing Cargo registry cache: `src/client/builder.rs:172`–180 initializes default resumption and `NoKeyLog`; `src/key_log_file.rs:24`–43 opens an append/create file and `:89`–95 reads `SSLKEYLOGFILE` when constructed. The added account matches those sources: ordinary defaults do not automatically log keys, but Tide must override resumption and avoid installing the file logger. NET-HTTP-2 and `research/42:813`–816 require environment invariance, with neither key-log creation nor append. The selected dependencies are unchanged.

## Reference count and fix-local verification

Created a fresh scratch directory, copied the unchanged tracked reference, and successfully checked and applied only the updated proposal there. Ran:

```text
scripts/count-tokens.sh target/work/redteam-recheck-reference.x0ynMB/design/reference.md </dev/null
24989
```

Exit status: **0**. The repository's counter uses its configured `claude-opus-5-5` token-measurement calls; no Rust compilation or Tide execution is involved. Measured scratch file SHA-256:

`d0deb5148eb83e69433cdbb17f8a566632227733f96c70617ade28d2ea57e5c0`

This independently matches the author's claimed file identity and count. There is **one token** of headroom under the requested 24,990 cap and eleven under D14's 25,000; the result belongs to this exact proposal and reference input.

The proposal's compensating receiver/simulation trims preserve capability receiver rules, planned availability, deterministic simulated capabilities, setup/inspection roles, virtual-time advancement and deadlock behavior. They retain the get signature and route its timing detail to companion 17. I found no new contract loss or authority claim in those trims.

`git diff --check 9ad42b9..b772256` and `git diff --check d58f3e4..HEAD` pass. Candidate identity, diff hash and proposal hash remained unchanged during the re-check; tracked working-tree status remains clean. Only the requested report and the isolated reference scratch copy were authored.

Not checked: no implementation builds, runtime witnesses, repeated full review or Opus verdict. Stage-2 enforcement remains unproved; the planned checks are requirements, not passing runtime evidence.
DONE
