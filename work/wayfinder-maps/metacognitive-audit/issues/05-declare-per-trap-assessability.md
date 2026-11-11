# Declare per-trap transcript assessability

Type: research
Status: resolved
Blocked by: 01

## Question

For each of the 8 traps, can a chat transcript ground a verdict, or is another signal required?

Declare each trap as either "assessable from transcript" or "not assessable from transcript (reason)." This is the Q7=B standard, applied before synthesis (ticket 06) so synthesis knows which traps to attempt and which to mark not-assessable.

Established fact to use: message entries carry millisecond timestamps, so the gap between user prompts is measurable and Interruption is assessable.

Open question to resolve: the Achievement trap (band-aid vs refactor) may need code-diff history a chat transcript does not contain. Decide whether the transcript alone can ground it, or whether it must be marked not-assessable-from-transcript (which may graduate a diffs-research ticket — see the map's Not-yet-specified).

Read a sample of the flattened transcripts (ticket 02 output) to judge what signals the audit will actually see — judge from the flattened view, not the raw jsonl, since the audit works from the flattened files (e.g. thinking blocks are dropped, so they are not a signal available to later tickets). Produce an assessability table (trap -> verdict -> reason) as a linked asset.

## Answer

Asset: [`assets/assessability-table.md`](../assets/assessability-table.md).

**Method.** Judged from the flattened view (read samples of sessions 11 and 17, timestamp scans on 11 and 8), not the raw jsonl. The flattened view carries: user text + assistant text verbatim, millisecond timestamps on every entry, compaction summaries verbatim, tool results truncated head+tail (1000+1000 chars), thinking + images dropped.

**Verdicts (8/8).**

| # | Trap | Verdict |
|---|---|---|
| 1 | Forming | Assessable (verbalized-model dependent; silent wrong models not catchable) |
| 2 | Dislodging | Assessable |
| 3 | Assumption | Assessable (intent verbatim; code-narrowness confirmation secondary) |
| 4 | Location | Assessable |
| 5 | Achievement | Assessable at instance level; NOT assessable at pattern level without diffs |
| 6 | Progression (AI) | Assessable |
| 7 | Interruption (AI) | Assessable (millisecond timestamps) |
| 8 | Mislead (AI) | Assessable |

**The Achievement split verdict resolves the map's open question.** The trap has two parts: a single band-aid-vs-refactor decision (instance level — assessable from the user's verbalized choice) and an "endless/repeated" pattern (pattern level — a frequency claim over code changes that a transcript grounds only via verbalized decisions, while git diff history grounds the accumulation regardless of verbalization). So: a **positive** Achievement finding asserting a repeated band-aid pattern would require diff history to confirm — that is the case which graduates the diffs-research ticket. A **negative** finding (as ticket 03 made: the user favors refactoring and removes band-aids) is grounded at the instance level and needs no diffs.

**Consequence for this pool.** No positive Achievement pattern is in the evidence, so **no diffs-research ticket graduates.** The map's Achievement/diffs fog is resolved conditionally, not by creating a ticket.

**For ticket 06.** Attempt all 8 traps. Write none as "not assessable from transcript" (none is fully unassessable); for Achievement, write the instance-level verdict and note the pattern-level caveat only if a positive pattern is claimed (it is not). Use only the exact-passing quotes from ticket 04 (or its corrected verbatim forms); assessability does not change which quotes are usable.
