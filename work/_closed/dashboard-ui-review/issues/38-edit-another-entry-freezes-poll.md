# Stop editing a second entry from freezing the story-log poll

Type: grilling
Status: resolved
Blocked by: —

## Question

While one entry is being edited, clicking Edit on a different entry runs `showEditForm` again without reverting the first. `pausePolling` then overwrites `originalTrigger` with `"none"`, so the resumed poll never restores the real trigger and the story log stops updating. The first entry also stays in edit mode. What should happen: revert the first edit, ignore the second click, or lock every other Edit button while editing?

## Context

- Found by the ticket 11 implementer; pre-existing. Ticket 11 now locks the edited entry's own controls but not the other entries' Edit buttons.
- Edit state lives in `editState` in `assets/index.html`; the revert path is `revertEdit()`; polling is paused and resumed by `pausePolling()` / `resumePolling()`.
- Ticket 27 owns failed-save recovery, ticket 11 the lock-down of one entry's controls.

## Done when

- The behaviour is decided and a tier 2 stub browser test drives it (edit one entry, click Edit on another, assert polling resumes after the first is finished).
- `python build.py` is green.

## Answer

Decided: **Option C — lock every other entry's Edit button while an edit is open.**

When `showEditForm` opens an edit, every other `.edit-btn` in the story log is disabled. The open entry keeps its ✓/✗ cluster (ticket 11). The user cancels or saves the open edit to unlock the rest. The lock outlives a successful save: it holds until the resumed poll re-renders the log with fresh, enabled buttons, because releasing it earlier lets a second editor open on an entry that still shows a textarea.

Root cause fixed too, so the freeze cannot return through any other caller:

- `pausePolling` saves `hx-trigger` only when `originalTrigger` is null. A second pause keeps the real value (`load, every 2s`), and `resumePolling` restores it once.
- `showEditForm` returns early when `editState` is set. The UI lock makes this unreachable; the guard is defense in depth.

Rejected:

- **Move the editor** (revert the first, open the second). One-click switching, but it silently drops the first draft, and it needs a token check in `submitEdit` for the save-in-flight race.
- **Refuse the second click.** The same outcome as the lock, but a click that does nothing reads as a dead control.
- **`title` on the disabled buttons / `aria-disabled` no-op.** Most browsers do not show a tooltip on a disabled control, and the repo's lock pattern (ticket 11) uses plain `disabled`.

Test tier: **tier 2** (stub browser) per `tests/STRATEGY.md`. The behaviour is pure client JS; faking the server does not change it. New scenario in `docs/specs/browser_story_log.md`: the `#story-log` poll trigger never stays `"none"`, and the other `.edit-btn`s are disabled during an edit and enabled after. The old "Done when" wording (click Edit on another, assert the poll resumes) does not fit the lock, because the second button is disabled; the new scenario replaces it.

Graduated [Lock every other entry's Edit button while an edit is open](62-lock-other-edit-buttons.md).
