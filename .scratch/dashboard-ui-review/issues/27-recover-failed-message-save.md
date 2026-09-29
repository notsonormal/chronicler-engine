# Recover from a failed message save instead of freezing the story log

Type: task (AFK)
Status: open
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
