# Clamp the status-display error and move the raw text into a popover

Type: task (AFK)
Status: resolved
Blocked by: 08

## Question

`generating_status_handler` renders `GenerationStatus::Error(msg)` as `<span class="status error">Error: {msg}</span>` at full length. The error string pushed the command input from about 1020px down to 235px (finding 1.5), and the string is raw transport text (finding 1.4). Clamp it to one line and move the raw text behind the agreed disclosure.

## Context

- Findings 1.4 and 1.5. Screenshot 04.
- Decided in [Decide how the dashboard shows each kind of failure](08-decide-failure-display.md): one clamped line carrying a short user-facing message, with the raw text behind an **anchored popover** — not an inline `<details>`, which grew the action area from 64px to about 100px in the mock (`tmp/show-me-shots/D-option4-table-status-480.png`, local only).
- The popover is the same pattern ticket 51's banner Details control uses: one pattern, two call sites. Escape and outside-click dismissal are part of it.
- Raw text goes verbatim to the log; what reaches the user is clamped and mapped.
- A generation error clears on retry, on the reset route, or on dismissal. No timer.
- Ticket 47 owns announcing it.
- Scenario 16.10 in `docs/specs/browser_dashboard.md` ("Status errors still reach the toast after confirming a preview") names the toast and needs updating with ticket 54.

## Done when

- A generation error renders as one clamped line, with the raw text reachable in the popover and nowhere inline.
- The action area's height and the command input's width do not change when the error appears.
- A test covers the clamped line and the popover. Tier by `tests/STRATEGY.md` and name it in the answer.
- `python build.py` is green, the user reviews the diff, then commit through `/commit-and-push`.

## Answer

Absorbed into [Redesign the error and health display](63-redesign-error-health-display.md).
The clamped status error moved into the Theme 1 redesign, where it reuses that
ticket's anchored popover instead of building its own. Closed as absorbed, not
fixed.
