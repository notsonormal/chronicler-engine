# Stop editing a second entry from freezing the story-log poll

Type: task (AFK)
Status: open
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
