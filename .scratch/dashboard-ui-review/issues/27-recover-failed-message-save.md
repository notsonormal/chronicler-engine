# Recover from a failed message save instead of freezing the story log

Type: task (AFK)
Status: resolved
Blocked by: —

## Question

`submitEdit()` in `assets/index.html` has no `response.ok` check and no `.catch`. When the save request fails (engine unreachable, or any network error), `resumePolling()` — called only inside `.then()` — never runs, so `#story-log` keeps `hx-trigger="none"` forever; the entry keeps its textarea; and the ✓/✗ buttons are dead because `currentEditId` and `originalText` are cleared before the response arrives. The user sees no error and cannot save, cancel, delete, or swipe that entry again until a full page reload. `submitNewSwipe()` has the same missing-check pattern and leaves the status stuck on "Thinking..." with a disabled "■ Stop". What change makes a failed save/cancel/retry report the failure and restore the previous state?

## Context

- Finding 05.F3 (P1). Screenshots 63 (stale textarea, "Ready", "Connected", no error) and 64 (stuck "Thinking..."); DOM probe: `hx-trigger="none"` after the failure, `"load, every 2s"` after reload.
- Related but distinct: finding 1.7 (silent failures) and ticket 10 (story-log poll swap). This is the *recovery* side — polling must be restored in a `finally`/`catch`, not only on success.
- Where the failure message is shown is subject to [Decide how the dashboard shows each kind of failure](08-decide-failure-display.md); the state restoration is independent of that decision.

## Done when

- A failed edit save: the error is visible, the entry returns to its pre-edit actions (or offers a working cancel), and polling resumes.
- A failed retry/new swipe: the status leaves "Thinking..." and the Send button is re-enabled.
- A test covers the failed-save recovery; tier by `tests/STRATEGY.md`.
- `python build.py` is green. Commit after user approval.

## Answer

**Recovery only, in `assets/index.html`.** Failures show through the existing `#error-notification` toast (`showError`). Where failures display long-term is [Decide how the dashboard shows each kind of failure](08-decide-failure-display.md).

- `submitEdit()` checks `response.ok` and keeps `currentEditId`/`originalText` until the outcome is known. On a non-ok response or network error it shows the toast and calls a new `revertEdit()`, which puts back the pre-edit text and clears edit state. `resumePolling()` runs in `.finally`, so `#story-log` never stays on `hx-trigger="none"`. `cancelEdit()` now uses `revertEdit()` too, with no behaviour change.
- `submitNewSwipe()` and `submitRetrigger()` share `submitGenerationRequest(url, failureMessage)`. On failure it shows the toast and calls `resetStatusToReady()`, which puts back the Ready span and re-enables Send. `submitRetrigger` had the same stuck-status bug; the reviewer judged the one-line extension justified, not scope creep.

**Tests (tier 2, stub browser, `tests/browser/stub/story_log.rs`, driving shipped clicks).**
- `test_failed_save_restores_entry_and_resumes_polling`, scenario 30.4 (`docs/specs/browser_story_log.md`): edit, save (stub 500), toast visible, textarea gone, original text back, `hx-trigger` back to `load, every 2s`.
- `test_failed_retry_clears_pending_status_and_re_enables_send`, scenario 30.5: retry (stub 500), toast visible, status Ready, `#submit-btn` enabled.
- Both fail on the pre-fix shell at the toast wait [reported by the implementer]. The stub server answers 500 on `POST /history/:id`, `/swipe/new` and `/retrigger`.

**Gate:** worktree on `9c446198`: `nextest: 1639 passed, 0 failed, 2 skipped`, browser 26 passed (`build_20260930_201452.log`). The coordinator then changed two comments only.

**Code review** (`/code-review`, verdict ISSUES → fixed): a comment in `stub_server.rs` named the ticket, which `CODING_STANDARDS.md` forbids; removed, and the file header now lists the 500 routes. Left open:
- **Partial on "returns to its pre-edit actions".** `revertEdit()` puts back only `.text`. The ✓/✗ buttons stay and do nothing, and Edit/Delete/Swipe return only when the resumed poll re-renders (≤2s). Handed to [Fix edit mode: size, focus, keys and locked controls](11-fix-edit-mode.md), which owns the entry's controls in edit mode.
- Other judgement calls went to [Follow up on small issues found during review](36-follow-up-small-review-issues.md).
- `resetStatusToReady()` does not meaningfully race the status poll: after a non-2xx no generation started, and the ≤5s poll corrects any stale state. [reviewer]

