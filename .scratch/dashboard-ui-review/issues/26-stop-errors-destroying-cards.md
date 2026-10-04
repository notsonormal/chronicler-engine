# Stop failed edits and refused deletes from destroying the entity card

Type: task (AFK)
Status: open
Blocked by: 08

## Question

A failed connection edit replaces the connection's own card with a bare error div (`hx-target="closest .connection-edit-form"`, `outerHTML`) — the connection disappears from the list. Deleting an active preset is correctly refused, but the refusal body replaces the preset card (`hx-target="closest .preset-card"`), so the card vanishes and only the error text is left. Both recover only on reload. What change keeps the card in place while showing the error or refusal?

## Context

- Finding 05.F2 (P1). Screenshots 56 (connection card replaced by `Error: Configuration error: Unknown LLM backend 'bogus_provider'`) and 52 (bare `Preset is a mode default; change the default before deleting` where the card was); ticket 05 answer.
- The refusal logic itself is fine and should stay: default presets and mode-default references must not be deleted.
- Where the error/refusal should be shown was [Decide how the dashboard shows each kind of failure](08-decide-failure-display.md); this ticket is the implementation for entity cards. Decided: a failure never swaps into the region it describes. The card stays, and the message renders in an inline slot inside it — a short user sentence, with the raw server text behind an anchored popover. [Keep a failed poll from replacing its region](52-failed-request-keeps-its-region.md) owns that shared fragment — consume it here.

## Done when

- A failed connection edit and a refused preset delete both leave the card visible and usable, with the message shown nearby.
- Tests cover both paths; tier by `tests/STRATEGY.md`.
- `python build.py` is green. Commit after user approval.
