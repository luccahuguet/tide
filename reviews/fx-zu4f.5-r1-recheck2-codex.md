Model: gpt-6.1-sol (high) · Date: 2026-11-18 · Re-check 4289279

## Verdict

approve. Open items 5 and 6 are resolved. No new blocking defect was found in their fixes. Findings 1–4 remain closed and were not re-reviewed.

## Scope and identity

Second finder-only recheck: the diff from `5dc6b40` to `4289279`, the recorded replacement admission proof, and the narrowed example-proof statement. Classification, bookkeeping and diagnostic evidence only. This report is the only write.

- Candidate: `42892798344ce5f9513d325eecab7932b891d92b`; base `cdf47ec`.
- Round diff SHA-256 (`git diff 5dc6b40..4289279`): `c0e330a115f95f175647e91aa1c1a369a10ec83f0103c4a1e29d52d0b8d6d68a`.
- Whole candidate diff SHA-256 (`git diff cdf47ec..4289279`): `67a04be72d5cda0bf5c32f78b1802ca6d0c2c505d6b43e1d42a8d9043c20c179`.
- Only `research/37-external-admission-search.md` changed; tracked working tree clean. Embedded protocol still matches the registered file byte for byte.

## Item 5 — resolved

`research/37-external-admission-search.md:157` explicitly preserves the fact that the original commands were not retained and substitutes the r1b rerun as this record's proof. It does not reconstruct or falsely attribute exact arguments to the original executions.

The retained script and log match the document's SHA-256 values:

- `target/work/ports/rerun.sh`: `1abd8e5f80b5f7e69e94adc548e8e90cbeef147f6046fd378b9fd9e4ff9e41cc`.
- `target/work/ports/rerun.log`: `f20f563ebdc0f9ac97ed5a66e37af826e5bff3f66d96755d3749599ed91ff32b`.

The log records HEAD `5dc6b40e0254d5dd83ab83b0f2d5a1dd2c2ec6dc`, toolchain/compiler/cargo identity, driver and std identities, port hashes, working directory, explicit environment, load, full shell-escaped commands, start/end times and exit status. Both build commands use heavy.sh and timeout 1200; each of the six direct-driver commands uses heavy.sh, the declared environment, timeout 600 and closed stdin. All dependency-directory and output-path arguments are present. Local git comparison independently confirms that `crates/`, Cargo.toml, Cargo.lock and rust-toolchain.toml at this candidate equal the original probe base `33085fd`; the intervening candidate changes are documentation.

All six port files' current byte hashes match the before/after hashes in the log, without inspecting their bodies. All six admission commands exit 1. Counting diagnostic records, rather than repeated excerpts inside the runner's failure message, gives exact agreement with the original logs:

| Port | Errors |
| --- | --- |
| amaperf buggy | TD0501 ×3, TD0103 ×1, E0433 ×5, E0425 ×2, E0599 ×2 |
| amaperf capability | TD0103 ×1, E0433 ×5, E0425 ×1 |
| chrono_anchor buggy | TD0602 ×2, TD0501 ×3, E0433 ×5 |
| chrono_anchor capability | E0599 ×2 |
| finch_rust buggy | E0433 ×1 |
| uniswap_utils buggy | E0433 ×2 |

The replacement checks use `--crate-type=lib --emit=metadata`. The record expressly distinguishes that from the original capability ports' test-build declarations. Since both capability counterparts already fail during compilation with the same errors, this distinction does not hide a passing admission or change the negative classification. The recorded commands run the admission driver, not the port programs. The script contains no command to execute a generated port artifact. It reads port sources inside target/work and places build outputs in the expressly documented build-target location; it does not recreate the temporary examples directories.

The chrono_anchor buggy-file modification time, 12:38:25Z, is between the original run3 and run4 log times, as now disclosed. The original logs support the described declaration change: run3 expects TD0602/TD0501 and fails on additional E0433; run4 expects all three codes. The direct driver does not interpret those runner declaration comments. Modification times alone cannot prove the entire historical edit was limited to that declaration, and I did not inspect port bodies. That historical limitation is no longer carrying the proof: the replacement run identifies the retained current file by hash and reproduces the diagnostic result directly.

The missing-invocation gap is therefore supplied by new recorded proof, rather than waived. The original execution history remains explicitly limited. Together with the already-closed screening and bookkeeping findings, no open item in this recheck prevents the documented completed-search negative. No catch is inferred: chrono_anchor's counterpart still fails on E0599.

## Item 6 — resolved

`research/37-external-admission-search.md:244` now limits the example claim to landed variants reached before the failing probe and explicitly says later variants were not checked. This matches the original runner's sorted sequential traversal, its panic on the first mismatch, and the saved logs' stopping points. The fix removes the unsupported claim about all landed examples without requiring another suite.

## New blocking defects

None in the fixes examined.

## Proof limits

This is a review of retained evidence, not an independent execution of the rerun. Exact commands, hashes and diagnostic results are verifiable in the artifacts; unrecorded historical activity and the original pre-rerun file contents cannot be established from modification times alone. The unexplained longer uniswap-utils wall time is accurately disclosed and its recorded exit/diagnostics agree with the stated result.

Skipped closed findings 1–4 and unaffected candidate content. No network access, source fetching, malicious-code or port-body inspection, Rust compilation, admission rerun, or crate/payload execution was performed. Remaining limits are the disclosed historical-proof limits above; no open blocker was found within this recheck's scope.

DONE
