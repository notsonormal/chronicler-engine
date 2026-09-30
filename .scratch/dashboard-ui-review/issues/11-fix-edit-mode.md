# Fix edit mode: size, focus, keys and locked controls

Type: task (AFK)
Status: open
Blocked by: —

## Question

Edit mode on a log entry has four problems (finding 2.5, screenshot 05):

- `#edit-textarea` is fixed at 144px, with an inner scrollbar.
- Focus does not move into the textarea.
- Escape and Ctrl+Enter do nothing.
- The entry's swipe and Retry controls stay active while editing.

Make the textarea fit its content and take focus. Escape cancels, Ctrl+Enter saves, and the other entry controls are disabled while editing.

## Context

- `docs/specs/browser_story_log.md` scenario 30.2 covers edit activation. Check whether the spec describes keys and locked controls. Add scenarios only if it is silent or wrong.
- Save goes through `submitEdit`, which has no failure handling (finding 1.7). Do not fix that here. It belongs to the Theme 1 work.
- Added from [Recover from a failed message save instead of freezing the story log](27-recover-failed-message-save.md): after a failed save, `revertEdit()` puts back only the entry's text. The ✓/✗ buttons stay and do nothing until the next poll re-renders the entry (≤2s). When you lock and restore the entry's controls here, make revert (cancel, Escape, failed save) put back the pre-edit actions straight away. Scenario 30.4 covers the failed save.

## Done when

- The four problems are fixed.
- A test is placed by `tests/STRATEGY.md` (likely tier 2).
- `python build.py` is green. Commit after user approval.
