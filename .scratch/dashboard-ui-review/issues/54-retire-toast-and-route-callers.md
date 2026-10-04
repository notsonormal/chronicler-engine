# Retire the toast and route its callers

Type: task (AFK)
Status: open
Blocked by: 63, 64

## Question

`#error-notification` is retired: every failure the toast carries today has a routed surface. What changes across `assets/index.html`, the stylesheet and the specs?

## Context

- Decided in [Decide how the dashboard shows each kind of failure](08-decide-failure-display.md), and sequenced last on purpose: retiring it before the banner (51), the inline slots (25, 26) and the clamped status display (53) exist would make failures go dark.
- Eight `showError(...)` call sites in `assets/index.html`: the status-text path, `htmx:beforeSwap` for htmx responses, `submitEdit`, `submitGenerationRequest` (new swipe and retrigger), `switchSwipe`, `deleteMessage`, and the delete path that shows the server's own text.
- Scenario 16.7 ("Action failure renders an error toast") and 16.8 ("A newer error is not hidden by an older error's timer") describe the toast and must be replaced or removed with it. 16.9 acknowledges the toast as the route for a confirm-time error; check it. 16.10 names it too (ticket 53).
- Remove the element, the `showError` helper, its hide timer and the `.error-notification` styles.
- `docs/diataxis/reference/frontend/dashboard.md` describes the error banner as the error path (finding 6, doc drift). Ticket 23 aligns that doc; this ticket should not leave it describing a removed element.
- Ticket 34 replaced weak tests in this area and ticket 40 drives the retrigger control. Check that removing the toast scenarios moves their coverage rather than deleting it.

## Done when

- No `showError` call site remains and `#error-notification` is gone from the markup and the stylesheet.
- Every failure the toast used to carry now lands on the banner, the status display, or an inline slot, shown per call site.
- The spec scenarios that describe the toast are either replaced by the routed surface's scenarios or removed with the reason recorded in the answer.
- `python build.py` is green, the user reviews the diff, then commit through `/commit-and-push`.
