# Re-review the dashboard after phase 2

Type: task (AFK)
Status: open
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
