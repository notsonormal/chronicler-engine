# Rebind action-area handles after swaps

Type: task (AFK)
Status: open
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
