Model: `gpt-6.1-sol`. Effort: `xhigh`. Date: 2026-10-20.

## Verdict

**request_changes — high confidence (0.94).**

The direction is sound and proportionate. One build-time read path bypasses RUN-2’s stated mechanism and needs an explicit enforcement requirement and conformance case.

## Scope

- HEAD: `87f9cd37f824b03d6eefac66e84850d02db9666f`
- Diff: `git diff -- design/ README.md`
- Diff SHA-256: `7d87d98c44fa2e5ef7a9d9e9bc023796f475e38706931c74fdb803959827847f`

Reviewed GOVERNANCE.md, AGENTS.md, fx-arx’s goal, acceptance and decision comments, the complete diff, rounds 54–61, and the specified surrounding design. The bead was read using `br --no-db show fx-arx` because ordinary `br show` could not acquire its lock in the read-only environment.

The diff identity remained unchanged through review.

## Blocking findings

### B1. LLVM’s assembler can read host files outside rustc’s source loader

**Where:** [design/03-authority-concurrency.md:450](/home/lucca/pjs/lab/tide/design/03-authority-concurrency.md:450), particularly the file-loader mechanism at line 452 and the bypass rejection at line 457; admission of granted assembly at [line 410](/home/lucca/pjs/lab/tide/design/03-authority-concurrency.md:410).

RUN-2 confines Rust source reads through the driver’s file loader. LLVM’s assembler has its own file-reading machinery: `.include` loads assembly text and `.incbin` embeds file contents. Neither uses rustc’s source loader. LLVM implements these through its own `SourceMgr` file operations. [LLVM assembler source](https://github.com/llvm/llvm-project/blob/main/llvm/lib/MC/MCParser/AsmParser.cpp)

A package with the permitted `unsafe` grant can therefore submit:

```rust
core::arch::global_asm!(r#"
.section .rodata
.incbin "/etc/hostname"
"#);
```

I reproduced this with rustc `1.96.0 (ac68faa20 2026-05-25)`, compiling stdin with:

```text
rustc --crate-name run2_probe --crate-type=lib --edition=2024 \
  --emit=obj -Ccodegen-units=1 -o /dev/stdout -
```

The captured 704-byte object contained the external file’s contents. An inline `asm!` variant also embedded them. No binary was executed and no file was created.

This happens during code generation, before RUN-3’s sandbox exists. A private file can consequently enter a build artifact or cache without a compiler bug.

`global_asm!` is already a listed RUN-1 hole; this is **not an unlisted safe-code escape**. The defect is narrower: RUN-4 exempts RUN-1, while RUN-2 independently promises build-read confinement. Naming assembly as a trust hole does not supply the missing enforcement. Freezing granted source files under RUN-5 also does not prevent an unchanged directive from opening a host path.

**Minimal patch:** Extend RUN-2 to require that assembler-requested file reads, including `.include` and `.incbin` through global, inline, or naked assembly, are denied before the backend opens the path, including under an `unsafe` grant. Extend the RUN-2 conformance row with granted assembly naming outside canaries; require no access and no canary bytes in diagnostics, artifacts, or caches.

This needs a reliable pre-read enforcement mechanism. A literal substring filter is insufficient because assembler macros can construct directives.

## Non-blocking improvements

**Exercise piped core collectors.** At [design/03-authority-concurrency.md:480](/home/lucca/pjs/lab/tide/design/03-authority-concurrency.md:480) and the RUN-3 conformance row, include a host configured to pipe core dumps to a collector. Linux ignores `RLIMIT_CORE` for piped dumps, so setting that limit alone does not establish the required “core dumps off” outcome. The existing outcome is sufficient; this strengthens its implementation check. [Linux core-dump documentation](https://man7.org/linux/man-pages/man5/core.5.html)

## Alternatives

Prefer closing assembler file reads within the existing admission/build boundary. This preserves granted machine code without adding language semantics.

If the pinned backend cannot support reliable rejection, a separate build sandbox can enforce the read boundary around rustc and its backend. That costs more infrastructure. Merely printing `run: not covered` for assembly does not repair RUN-2’s unconditional claim.

## Reasoning

The remaining guarantee is stated honestly. It distinguishes authority confinement from availability, metadata disclosure, side channels, compiler and kernel bugs, and result forgery. The reviewed-base requirement, full snapshot comparison before test execution, package-wide transitive protection, inline replay inputs, and harness-owned printed commands address the earlier concrete attacks. I found no additional runtime escape outside the listed holes.

The Linux runtime sandbox is implementable in principle. Installing it before exec covers constructors; Landlock restrictions persist into descendants; seccomp can inspect scalar `clone` flags while refusing `clone3` with `ENOSYS`. glibc provides the stated fallback. The syscall categories also leave room for Rust startup, signal handling, allocation, polling, and threading. These are design-feasibility conclusions, not demonstrated Tide behavior. [Linux Landlock documentation](https://docs.kernel.org/userspace-api/landlock.html), [glibc clone fallback](https://sourceware.org/pipermail/glibc-cvs/2021q3/073721.html)

The addition passes the design admission rule: it strengthens safe validation through tooling, reuses the existing child-process and simulated-capability architecture, and adds no competing language mechanism. B1 should be corrected within that scope. A separate narrow independent review confirmed the distinction between RUN-1’s assembly exception and RUN-2’s build boundary.

`git diff --check -- design/ README.md` passed. The current diff addresses round 61’s snapshot, example, and copy-path residuals; those settled points are not reopened.

Not checked: token counts or an implemented Tide sandbox; no files were modified.  
Residual risk: runtime enforcement remains unproven until the conformance cases run against the actual toolchain.