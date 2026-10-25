# Re-review the dashboard after phase 2

Type: task (AFK)
Status: resolved
Blocked by: 01, 02, 03, 04, 05, 06, 08

## Question

After every phase-2 ticket is resolved, does a fresh review of the dashboard find any new P1 or P2?

## Context

- Method: the same as the [first re-review](../../dashboard-ui-review/re-review-2026-10-09.md). Use a probe server on its own port and database, the real connections, headless Chrome at 1280 wide plus one 1024x700 check, and tag each finding [known] or [inferred] on the P1/P2/P3 scale.
- Check that each R-finding the phase-2 tickets fixed is fixed in the running app, not only in tests.
- Cover what the first re-review missed: force a trigger so the Retrigger control appears, and exercise it. Use that run to settle the fog item "Other snapshot-restore paths" on the map.
- Write the report next to the map as `re-review-<date>.md`.

## Done when

- The fresh review finds no new P1/P2. Any new P3 is fixed or ruled out of scope with a reason.
- `python build.py` passes.
- If the review finds new P1/P2s, record them as tickets, add them to this ticket's `Blocked by:`, and leave this ticket open.

## Answer

The review found **one new P1 and three new P2s**, so the "no new P1/P2" exit is not met. At the user's direction, this ticket resolves as the record of the 2026-10-11 review. The gate moves to [Re-review the dashboard after the phase-2 fixes](14-re-review-after-phase-2-fixes.md), which is blocked by the new tickets below.

[Report](../re-review-2026-10-11.md). Tree `95ebb6ee`, full gate green (`logs/build_20261011_124531.log`: integration `1610 passed, 0 failed, 2 skipped`, browser `27 passed`). 31 real LLM calls. Settings were restored and the probe server was stopped.

- **Phase-2 fixes:** every R-finding that phase 2 fixed is fixed in the running app (R1–R4, R6–R11, R13–R15, R17–R22). R6 was seen live: a real spent budget became a `TokenBudgetSpent` failure, not story text.
- **Fog item "Other snapshot-restore paths": settled.** A forced trigger (Quantifier on `gpt-4o-mini`, `front_gates` → `courtyard` → `entrance_hall`) fired "Gabriella Introduction". Event delete and Retrigger (success and failure) leave no stuck status. Event Retry is broken (S1).
- **New findings, each ticketed:**
  - S1 (P1): [Make event Retry restore the trigger context and never roll the game back](09-fix-event-retry-snapshot.md)
  - S3 (P2): [Report a failed trigger continuation without a story-log Message](10-report-failed-trigger-without-message.md)
  - S2 (P2): [Show the whole token-budget failure, remedy included](11-show-the-token-budget-remedy.md)
  - S4 (P2): [Restore focus after preset and world deletes](12-restore-focus-after-card-deletes.md)
  - S5–S8 (P3): [Follow up on small issues found in the phase-2 re-review](13-follow-up-phase-2-re-review-p3s.md)
- **Map changes:** the restore-paths fog item is removed; a fog item for a Max Tokens control is added; the Quantifier missing a trigger-introduced NPC is out of scope (backend detection quality).
