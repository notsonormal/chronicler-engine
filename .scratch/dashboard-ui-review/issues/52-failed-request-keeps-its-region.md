# Keep a failed poll from replacing its region

Type: task (AFK)
Status: resolved
Blocked by: 08

## Question

A fragment failure returns HTTP 200 carrying `<div class="error-message">` (`render_fragment` → `render_error`), so the failing body *becomes* the element the request targeted. On a form that wipes a panel (tickets 25 and 26); on a poll it destroys last-good content, and the user sees an error fragment where the story log or the sidebar used to be. Make a failed poll change nothing in the DOM, and route the failure to its proper surface.

## Context

- Decided in [Decide how the dashboard shows each kind of failure](08-decide-failure-display.md). The rule: a failure never swaps into the region it describes.
- Polls and their intervals: `#story-log` 2s, `.visual-sidebar` 5s, `#options-dock` 2s, `#status-display` 5s, `.llm-messages-panel` 4s.
- A failed poll returns non-2xx with `HX-Reswap: none`, and the region keeps its last good content. `render_fragment` currently returns 200 for every failure, which is also why ticket 05's finding 1 (a failed add wipes the whole Settings panel) reaches the user as data loss.
- The client has one `htmx:beforeSwap` listener today: on `evt.detail.isError` it strips tags off the response body and shows the toast. That listener and ticket 51's banner listener are the same place. On a failed request, mark the banner; for an action — not a poll — write the message into the failing form's inline slot.
- `htmx:sendError` fires for a dead server with no response body at all, so the client synthesises the same fragment. That is the concrete gap in ticket 05's finding 6: with the engine down, the connection, world and game deletes do nothing.
- This ticket owns the shared short-message + anchored-popover error fragment. Tickets 25 and 26 consume it for their inline slots; do not build a second shape.
- Raw server text is not rendered inline. It lives behind the popover, and verbatim in the log.

## Done when

- A failed poll leaves its region unchanged and marks the banner.
- A failed action renders its message in the inline slot inside the form or card, never over the region.
- A dead server produces that same inline message for a pending action, instead of silence.
- Tests cover a poll failure, an action failure with the server up, and one with the server down. Tier by `tests/STRATEGY.md` and name it in the answer.
- `python build.py` is green, the user reviews the diff, then commit through `/commit-and-push`.

## Answer

Absorbed into [Redesign the error and health display](63-redesign-error-health-display.md).
The shared short-message + anchored-popover fragment and the poll non-2xx
behaviour moved into the Theme 1 redesign, which now owns the fragment the form
and card tickets consume. Closed as absorbed, not fixed.
