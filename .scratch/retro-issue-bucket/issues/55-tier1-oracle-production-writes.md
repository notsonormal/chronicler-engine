# Check that a tier-1 assertion reads state the code under test writes

Type: grilling (HITL)
Status: resolved
Blocked by: —

## Question

In C5 the Impersonate arm of the stop matrix in `tests/http/races.rs` read the room through a read seam over the Snapshot, which the Job never wrote after C4. The arm could not fail. The spec-axis review found it; a flip-the-expectation check then proved the reworked arm. The same class of fault appeared in P5 (late-answer assertions that ran before the late answer could land).

Which guard fits?

- **A standard for review.** `tests/STRATEGY.md`: a tier-1 assertion reads a seam that the code under test writes; a reviewer checks the write path of each new assertion.
- **A process step.** A test that turns green in a step that changes its subject is checked once against a wrong expectation (flip the expectation, watch it fail).
- **Leave it.** C6 deleted the Snapshot, so this instance cannot recur.

## Context

- Source: `/retro` 2026-10-19 on the core step of the Rust rebuild (ticket 07, "C5 fix pass 2"; ticket 06 retro for P5).

## Answer

2026-10-21. **Dropped (stale); the rule folds into [23](23-unconditional-mutation-check.md).** Triage pass 2 (`../triage.md`), HEAD `511e28dd`; the owner approved the close.

- C6 deleted the Snapshot (`plumbing.rs` migration drops its tables), so the instance cannot recur.
- The general rule is 23's break-and-watch. Widen 23's trigger to "a test that turns green in a step that changes its subject".
