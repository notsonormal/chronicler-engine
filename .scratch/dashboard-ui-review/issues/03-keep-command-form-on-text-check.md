# Keep the command form when a text-check result shows

Type: task (AFK)
Status: open
Blocked by: 02

## Question

`checkText()` replaces all of `#action-area` with the `/check-text` response. With text check disabled, the response is only `<span>Text check is disabled</span>`. The command input, Send button, and status display then vanish, with no way back except a reload. Where should a text-check result show so the command form survives?

## Context

- Finding 2.1 (P1). Screenshot 07.
- This is a minimal fix, as the map decided: the result gets its own element and never replaces the command form. The Theme 1 design may reshape it later.
- Do not decide here whether the ✓ buttons should show when text check is disabled. That is finding 4.3, in [Settings panel: roles, buttons and text-check controls](15-settings-panel-prototype.md).
- Blocked by 02 because both tickets edit the action-area JS.
- Stub tests 16.9 and 16.10 (`tests/browser/stub/dashboard.rs`) guard 02's fix. They set up the swap by calling `saveActionArea()` / `restoreActionArea()` directly. After this fix, check what they still guard. If the text-check flow no longer replaces `#action-area`, retire both tests and their scenarios, or rewrite them to drive a path that still replaces it. A test that still passes but no longer matches any shipped path is the case to avoid. See findings M1 and B2 in the [test audit](../assets/test-audit/test_design.md).

## Done when

- Clicking ✓ on a log entry or on the player input never removes `#command-form` or `#status-display`.
- A test is placed by `tests/STRATEGY.md` (likely tier 2).
- `python build.py` is green. Commit after user approval.
