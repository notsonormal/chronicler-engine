# Re-review the dashboard after the phase-2 fixes

Type: task (AFK)
Status: open
Blocked by: 09, 10, 11, 12, 13, 15

## Question

After the fixes for the [2026-10-11 re-review](../re-review-2026-10-11.md) are resolved, does a fresh review of the dashboard find any new P1 or P2?

## Context

- Method: the same as [Re-review the dashboard after phase 2](07-re-review-after-phase-2.md) and its report. Use a probe server on its own port and database, the real connections, headless Chrome at 1280 wide plus one 1024x700 check, and tag each finding [known] or [inferred] on the P1/P2/P3 scale.
- Check that S1–S8 are fixed or ruled out in the running app, not only in tests. S6 is fixed by [Put the room name on the Narration that arrives in the room](15-room-name-on-arriving-narration.md). F10-1 and F10-2 (trailing `System` notices) are out of scope; the map names their owner.
- Force a trigger again (the Quantifier needs a working connection; Ollama does not run in the container). Exercise event Retry after a normal trigger turn and after a Retrigger, and a failed trigger continuation. Check that a failed retry leaves the room, the trigger flags and the history unchanged.
- Spot-check the phase-2 fixes the 2026-10-11 report lists as fixed.
- Write the report next to the map as `re-review-<date>.md`.

## Done when

- The fresh review finds no new P1/P2. Any new P3 is fixed or ruled out of scope with a reason.
- `python build.py` passes.
- If the review finds new P1/P2s, record them as tickets, add them to this ticket's `Blocked by:`, and leave this ticket open.
