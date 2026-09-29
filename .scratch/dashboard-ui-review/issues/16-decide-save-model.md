# Decide one save model for panel forms

Type: grilling (HITL)
Status: open
Blocked by: —

## Question

Panels save in three different ways:

- Connection buttons act at once.
- Text Check needs a Save button.
- World edit saves Narrator Mode, Perspective, and Tense on change, but saves the other fields only on "Update World". So Cancel does not undo a posture change.

Which save model should the panels share?

## Context

- Finding 4.4. Screenshot 15.
- Posture auto-save is specified: `docs/specs/games.md` 20.8 and `docs/specs/browser_worlds.md` 29.2. A different model means changing those specs, not only the code.
- A failed "Update World" with an invalid scenarios JSON shows the error toast, but the green "Saved" status from an earlier posture auto-save stays under the form, reading like confirmation (finding 05.F8, prior screenshot 42). The save model decision should say what that status means and when it clears.

## Done when

- The decision is in the ticket answer. Implementation tickets are created.
