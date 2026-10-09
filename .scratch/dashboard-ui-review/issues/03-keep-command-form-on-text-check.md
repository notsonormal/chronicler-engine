# Keep the command form when a text-check result shows

Type: task (AFK)
Status: resolved
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

## Answer

**Approach.** The text-check result renders into its own `#text-check-result` element inside `#action-area`. `checkText()` writes the `/check-text` response there and runs `htmx.process` on it. It never touches `#command-form` or `#status-display`. `restoreActionArea()` clears the result, and reinstates the saved action area only when `#command-form` is gone, so a preview's Cancel dismisses a result that sits beside a live form. `ActionAreaTemplate` (`src/adapters/driving/http/templates.rs`) and the stub fixture also carry the element, so the `/action/confirm` and `/fragment/action-area` outerHTML swaps keep it. Any non-empty `#text-check-result` takes its own row (`assets/styles.css`).

**Scope kept.** The form-submit auto-check preview still replaces `#action-area` on purpose: it brings its own Send and Cancel. Only the ✓ path moved. Finding 4.3 is not decided here.

**Tests (tier 2, stub browser).**
- New `test_text_check_result_keeps_command_form`, scenario 16.11 (`docs/specs/browser_dashboard.md`). It asserts node identity of `#command-form` and `#status-display` before and after the log-entry ✓, and that `#text-check-result` is non-empty.
- 16.9 and 16.10 were rewritten, not retired (audit M1). `swap_action_area_via_restore` called `saveActionArea()`/`restoreActionArea()` directly. It is replaced by `swap_action_area_via_text_check_confirm`, which drives the shipped path: form submit, auto-check preview replaces `#action-area`, confirm, `/action/confirm` swaps in a fresh action area. `assert_status_display_restored` now checks node identity. The spec Givens for 16.9/16.10 follow.
- The stub server serves `/check-text` and `/action/confirm`, and `/action/check` returns a real `TextCheckPreviewTemplate` render for a canned misspelling. The stub's check hook now sits on a `log-entry input` fixture entry, matching the template's `log_type == "input"` condition.

**Gate:** worktree on `f03ee45f` + this change: `nextest: 1638 passed, 1 failed, 2 skipped`, browser 24 passed. The failure, `story_log::test_delete_last_between_actions_http` ("should have 2 Input entries"), passes in isolation. It is the parallel-load flake family that [Stop the story-log delete test flaking under parallel load](30-fix-flaky-story-log-delete-test.md) covers. Accepted on that basis.

**Code review** (`/code-review`, verdict CLEAN). Fixed before merge: fixture put `.check-btn` on a narration entry (SP2); plain result spans did not get their own row (SP3); the stub test asserted the canned body verbatim (S1); the stub server header named one dynamic endpoint (S2). Left open:
- B2 is unchanged: 16.10's `inject_status_html` still hand-writes the status span. Noted on [Fix the weak dashboard and settings tests](34-fix-weak-dashboard-and-settings-tests.md).
- `checkCurrentInput()` (`assets/index.html`) has no caller, so "✓ on the player input" has no UI today. Noted on [Settings panel: roles, buttons and text-check controls](15-settings-panel-prototype.md) with finding 4.3.

