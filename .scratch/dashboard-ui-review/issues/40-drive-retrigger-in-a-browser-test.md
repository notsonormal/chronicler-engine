# Drive the retrigger control in a browser test

Type: task (AFK)
Status: open
Blocked by: —

## Question

Nothing drives `submitRetrigger()` in a browser test: no stub fixture renders a retrigger button, and the stub's `/retrigger` 500 route was removed in ticket 36. Its failure path is the shared `submitGenerationRequest`, which scenario 30.5 already covers through `/swipe/new`, so only the retrigger URL and wiring are unobserved. Is that worth a tier-2 scenario?

## Context

- From [Recover from a failed message save instead of freezing the story log](27-recover-from-failed-message-save.md) and [round 1 follow-ups](36-follow-up-small-review-issues.md).
- It needs a retrigger button on the stub story-log fixture, a stub `/retrigger` route, and a tagged scenario in the browser story-log spec.

## Done when

- A tier-2 test clicks the retrigger control and asserts the request and the recovery, or the ticket is closed with the reason it isn't worth it.
- `python build.py` is green.
