# Tide second-model confirmation, round 7

## Verdict

**approve**

Reviewed HEAD `c98fde8977ef1e5b5c7882171993313101ba8692` (`c98fde8`) on
2026-10-19. Approval covers R3 and new v0-blocking contradictions in the
requested sections; the evidence and scenario deductions follow.

## R3 status

**Resolved (review inference).** The decision record owns semantics
([01, lines 3–6](../design/01-design.md#L3-L6)). Its scope-result rule and
the chapter and reference now give the same outcome selection:

1. A body panic continues unwinding after joining the children.
2. Otherwise, the earliest-spawned panicking child supplies the panic.
3. Otherwise, the earliest-spawned failing child, excluding echoes, supplies
   the error, with the scope's own deadline reported as `DeadlineExceeded`.
4. Otherwise, the scope returns the body's result, `Ok` or `Err`.

Evidence: [D8, lines 435–446](../design/01-design.md#L435-L446),
[TASK-1/2, lines 615–645](../design/03-authority-concurrency.md#L615-L645),
and reference §7.1's [Failure, lines 865–874](../design/reference.md#L865-L874)
and [Panics propagate, lines 884–887](../design/reference.md#L884-L887).
The reference's body-panic bullet supplies the exception to its Failure
bullet, matching D8 and TASK-1/2 (inference from those passages).

Evidence: the echo definition includes cancellation caused by the body's
`Err` or a sibling's failure in [D8, lines 437–439](../design/01-design.md#L437-L439)
and [reference, lines 871–874](../design/reference.md#L871-L874).
[TASK-2, lines 636–645](../design/03-authority-concurrency.md#L636-L645)
also applies both causes to the per-child observation bit, requires no
inspection of `E`, and places a child failure without that observation
ahead of the body's `Err`. The body-error explanation now applies when
children only echo it, rather than overriding the ordered selection
(inference from these passages).

**Scenario analysis (inference, not execution).** Only the stated failures
and panics occur; (a)–(d) each have one child. Source links identify the
HEAD text from which each outcome follows. `DE` below means
`Err(E::from(TaskError::DeadlineExceeded))` as specified by
[TASK-2/4, lines 630–633 and 653–661](../design/03-authority-concurrency.md#L630-L661).

| Scenario | Decision record: D8 | Concurrency chapter | Reference §7.1 | Agree? |
| --- | --- | --- | --- | --- |
| (a) Body returns `Err(Invalid)`; its cancellation reaches the child's checkpoint; child returns that cancellation error. | `Err(Invalid)`; exclude the echo ([437–444](../design/01-design.md#L437-L444)). | `Err(Invalid)`; the checkpoint sets the body-caused echo bit, then rule 3 applies ([634–644](../design/03-authority-concurrency.md#L634-L644)). | `Err(Invalid)`; children only echo the body ([869–874](../design/reference.md#L869-L874)). | Yes |
| (b) Body returns `Err(Invalid)`; child computes without a checkpoint, then returns `Err(Independent)`. | `Err(Independent)`; the child did not observe cancellation ([437–445](../design/01-design.md#L437-L445)). | `Err(Independent)`; rule 2 counts this independent failure ([630–642](../design/03-authority-concurrency.md#L630-L642)); cancellation cannot interrupt that computation ([TASK-5, 667–669](../design/03-authority-concurrency.md#L667-L669)). | `Err(Independent)`; the child error precedes the body result and does not qualify as an echo ([868–874](../design/reference.md#L868-L874)). | Yes |
| (c) Child `x` returns `Err(Independent)` without observing cancellation; body does `x.join()?`. | `Err(Independent)`, as D8 states for this form ([440–446](../design/01-design.md#L440-L446)). | The join yields converted `Cancelled`; rule 2 reports `x`'s `Independent` ([JOIN-2, 684–688](../design/03-authority-concurrency.md#L684-L688); [TASK-2, 641–645](../design/03-authority-concurrency.md#L641-L645)). | The join yields converted `Cancelled`, but the scope reports `Independent` ([871–882](../design/reference.md#L871-L882)). | Yes |
| (d) The scope's own deadline cancels it first; child then returns `Err`, and body also returns `Err`. | `DE`; the deadline-cancelled failing child precedes the body result ([440–444](../design/01-design.md#L440-L444)). | `DE`; rule 2 selects the child and TASK-4 maps its error. TASK-4 does not rewrite the body's error or give it precedence ([630–634](../design/03-authority-concurrency.md#L630-L634); [653–661](../design/03-authority-concurrency.md#L653-L661)). | `DE`; the selected child's deadline error precedes the body result; deadline-origin cancellation is outside the stated echo causes ([868–874](../design/reference.md#L868-L874)). | Yes |
| (e) Body panics while a child has returned `Err`. | No `Result`; join children and continue the body's panic ([435–437](../design/01-design.md#L435-L437)). | No `Result`; TASK-1 continues body unwinding and TASK-2 excludes this case from result selection ([615–627](../design/03-authority-concurrency.md#L615-L627)). | No `Result`; continue the body's panic after joining ([884–887](../design/reference.md#L884-L887)). | Yes |
| (f) §12.2 fetcher: children wrap per-URL errors in `Ok`, including the slow request's deadline cancellation ([1218–1221, 1238–1242](../design/03-authority-concurrency.md#L1218-L1242)). | `Ok(partial_results)`; expiry alone does not fail the scope and no child failed ([417–421](../design/01-design.md#L417-L421); [440–444](../design/01-design.md#L440-L444)). | `Ok(partial_results)`, including the inner `Err(NetError::Cancelled)`; successful joins preserve these values ([TASK-4, 661–663](../design/03-authority-concurrency.md#L661-L663); [JOIN-2, 684–686](../design/03-authority-concurrency.md#L684-L686); [example, 1231–1242](../design/03-authority-concurrency.md#L1231-L1242)). | `Ok(partial_results)`; the child errors are data inside successful results ([870–878](../design/reference.md#L870-L878); [880–882](../design/reference.md#L880-L882)). | Yes |
| (f) §12.3 partition: `put` has no children; a body operation returns a cancellation error through `?` ([1277–1285, 1311–1314](../design/03-authority-concurrency.md#L1277-L1314)). | Inner scope returns the body's cancellation `Err` unchanged ([440–444](../design/01-design.md#L440-L444)). | Same `Err`; TASK-4 never rewrites the body error, and the example states this result ([634](../design/03-authority-concurrency.md#L634); [660–661](../design/03-authority-concurrency.md#L660-L661); [1311–1315](../design/03-authority-concurrency.md#L1311-L1315)). | Same body `Err`; there is no child error to select ([868–874](../design/reference.md#L868-L874)). | Yes |
| (f) §12.3 variant: `handle` panics in a server child ([1253–1258, 1315–1317](../design/03-authority-concurrency.md#L1253-L1317)). | Server scope propagates the child panic ([439–440](../design/01-design.md#L439-L440)). | Server scope propagates that panic ([TASK-1/2, 615–629](../design/03-authority-concurrency.md#L615-L629); [example, 1315–1317](../design/03-authority-concurrency.md#L1315-L1317)). | Server scope propagates that panic ([884–887](../design/reference.md#L884-L887)). | Yes |

For §12.3's ordinary partition path, `handle` converts `put`'s `Err` into
`Msg::Fail`, and the server child handles a returned `handle` error and
returns `Ok(())` ([1253–1258](../design/03-authority-concurrency.md#L1253-L1258),
[1263–1268](../design/03-authority-concurrency.md#L1263-L1268)). Thus that
path does not itself end or fail the outer server scope, whose body keeps
accepting connections ([1251–1259](../design/03-authority-concurrency.md#L1251-L1259));
this is an inference under the same rules cited above.

## New blocking problems

**none**

Review inference: the cited rules and scenario outcomes expose no new
v0-blocking contradiction in the requested sections. This review does not
reopen unrelated non-blocking notes or later tiers.

## Ready to implement v0

**Yes, within this review's scope (review inference).** R3 no longer
requires a scope-result design choice: D8, TASK-1/2/4, JOIN-2, and the
reference agree on the tested outcomes, including body-error echoes,
independent child failures, and the worked examples (citations above).
The v0 tier includes structured tasks and the simulator
([01, lines 66–74](../design/01-design.md#L66-L74)); this bounded confirmation
does not establish readiness of unreviewed parts of that tier.

Validation: read-only document analysis and scenario comparison of HEAD
`c98fde8977ef1e5b5c7882171993313101ba8692` on 2026-10-19. No checker,
interpreter, or runtime was run; implementation behavior remains unproven.
Only `reviews/round7-codex.md` was written; no staging or commit operations
were performed.
