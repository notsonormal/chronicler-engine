# Lock every other entry's Edit button while an edit is open

Type: task (AFK)
Status: resolved
Blocked by: —

## Question

Implement the decision in [Stop editing a second entry from freezing the story-log poll](38-edit-another-entry-freezes-poll.md). While an edit is open on a story-log entry, disable every other entry's `.edit-btn`. Also fix the root cause, so a second `showEditForm` call cannot corrupt the poll:

- `pausePolling` saves `hx-trigger` only when `originalTrigger` is null. A second pause keeps `load, every 2s`.
- `showEditForm` returns early when `editState` is set.

## Context

- Decision and rejected options: ticket 38.
- Edit state lives in `editState` in `assets/index.html`. The lock and its release belong beside `showEditForm` and `revertEdit`.
- Release the lock on cancel, Escape, and a failed save. A successful save keeps the lock until the resumed poll re-renders the log (≤2 s); releasing it earlier lets a second editor open on an entry that still shows a textarea.
- The disabled style follows the existing pattern in `docs/diataxis/reference/frontend/ui_design.md` (`opacity: 0.5; cursor: not-allowed`). `.action-btn:hover` needs a `:not(:disabled)` guard, like `.option-btn` and `.swipe-btn`.
- Spec: `docs/specs/browser_story_log.md` is silent on a second Edit click. Add a scenario in the register of 30.9. Scenarios 30.1–30.10 stay.
- Test: tier 2 stub browser, `tests/browser/stub/story_log.rs`, beside the other edit tests. Follow `tests/STRATEGY.md`.
- Ticket 11 locks the edited entry's own controls; this extends the same pattern to the rest of the log.
- Source of the finding: the ticket 11 implementer, recorded in [Follow up on small issues found during review](_resolved/36-follow-up-small-review-issues.md).

## Done when

- While an edit is open, every other `.edit-btn` is disabled. Cancel, Escape, and a failed save re-enable them in the same tick. A successful save re-enables them when the poll re-renders the log.
- `pausePolling` cannot overwrite the saved trigger, and `showEditForm` does nothing while an edit is open.
- The new spec scenario is added; a tier 2 test drives it and fails on the pre-fix shell.
- `python build.py` is green, the user reviews the diff, then commit through `/commit-and-push`.

## Answer

Absorbed into [Keep DOM state and focus through in-place swaps](65-keep-dom-state-through-swaps.md).
The edit-button lock moved into the DOM-state sweep, beside the morph and focus
work in the same file, spec and test file. Closed as absorbed, not fixed.
