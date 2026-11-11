# Rebind action-area handles after swaps

Type: task (AFK)
Status: resolved
Blocked by: —

## Question

`assets/index.html` binds `statusDisplay` and `submitBtn` once, at page load. After any swap of `#action-area` (for example `restoreActionArea()`), both point at detached nodes (`isConnected === false`). So two things stop working until reload: the Send-button lock during generation, and the status observer (including the `lastStatusError` dedupe from the error-banner fix). How should the shell find these elements after a swap?

## Context

- Finding 2.2 (P1).
- Options include looking the elements up at use time, event delegation on a stable ancestor, or re-attaching the observer after each swap (`htmx:afterSwap`).

## Done when

- After an action-area swap, Send locks during generation and status errors still reach the observer.
- A test is placed by `tests/STRATEGY.md`. Likely tier 2 (stub), because this is pure client behaviour.
- `python build.py` is green. Commit after user approval.

## Answer

Chosen mechanism: a mix of the first two options, no `htmx:afterSwap` re-attach.

1. Send button — look up at use time. `setButtonState()` now resolves
   `#submit-btn` by id on every call (null-guarded), matching the shell's own
   established pattern (`commandInput()`, `saveActionArea()`,
   `updateToThinking()` already re-query by id). The page-load `const submitBtn`
   binding is gone.
2. Status observer — event delegation on a stable ancestor. The page-load
   `const statusDisplay` binding is gone; the MutationObserver now observes
   `document.body` (a node no swap replaces) and resolves the live
   `#status-display` per event, guarded by
   `statusDisplay.contains(mutation.target)` so the story-log/options-dock
   pollers stay no-ops. Delegation was chosen over re-attaching via
   `htmx:afterSwap` because two of the swap sources (`checkText()` and
   `restoreActionArea()`) are raw `innerHTML` writes that fire no htmx event,
   and because swap-mechanism-agnostic observation keeps working when ticket 03
   rewrites the text-check swap path.

The `lastStatusError` dedupe state is script-level and unchanged; the
`errorNotification` handle stays (direct body child, never swapped).

Test tier: 2 (stub browser), `tests/browser/stub/dashboard.rs`:
- `test_send_locks_and_unlocks_after_action_area_swap` (spec scenario 16.9) —
  swaps `#action-area` through the shipped save/restore path, submits, asserts
  the Send button locks with a "Stop" label and unlocks ("Send") when the
  status returns to Ready.
- `test_status_error_reaches_observer_after_action_area_swap` (spec scenario
  16.10) — injects the engine-shaped error span into the live status display
  after the swap, asserts the toast shows, then after a Ready reset asserts the
  same error toasts again (dedupe was reset, not leaked).
