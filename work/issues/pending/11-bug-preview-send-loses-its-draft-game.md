# A preview send names the Game of the screen when the Action input was emptied

Source: ticket 02 of the state-machine map, the second scan (scan 2, 2026-11-09): `scan2-frontend.md`, *Action area*, fourth risk ([catalogue](../../wayfinder-maps/state-machines-next/assets/catalogue.md), row G5)

## The defect

`docs/diataxis/reference/frontend/dashboard.md:37` says: "Two drafts name the Game they were written for (`data-draft-game`): the Action input with its preview, … A request from inside one of them names that Game."

The preview forms have no attribute of their own. `draftOfRequest` (`assets/dashboard-screen.js:415-418`) maps a request from `#action-preview` to `#command-form`. But `syncCommandDraft` (`assets/dashboard-action-area.js:230-238`) removes `data-draft-game` from `#command-form` as soon as the input is empty (`:234`). The preview stays open. A send from the preview then names the Game of the screen (`dashboard-screen.js:420-427`), not the Game the draft was written for.

## Impact (inferred)

Low. The player must empty the Action input while a preview is open, and the active Game must change before the preview send. Then the send goes to the new Game instead of getting the `409` that the doc promises. When the screen Game did not change, the result is the same.

## Done when

- A preview send names the Game of its draft, also when the Action input is empty.
- A browser test opens a preview, empties the input, changes the active Game, sends the preview, and gets the `409`.
