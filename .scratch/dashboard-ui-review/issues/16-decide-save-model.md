# Decide one save model for panel forms

Type: grilling (HITL)
Status: resolved
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

## Answer

**Option E — hybrid by control type.** A single-choice control or a command
applies at once with visible feedback; a text field or a JSON blob commits only
on its form's Save. This is what the sister project Marinara-Engine does
(preference switches instant and debounced; record editors with an explicit Save
and an unsaved-changes guard). Plain "everything immediate" (option C) was
rejected: the World form edits Map and Scenarios JSON in plain textareas, which
cannot save on every keystroke.

Applied to the dashboard:

- **Instant:** connection *Set as Narrator* / *Set as Quantifier* / *Delete*;
  Text Check mode and *check before sending*; World Narrator Mode / Perspective /
  Tense. All were already instant except Text Check's Save.
- **On Save:** connection Add and Edit (already so); World details — Name,
  Description, Global Rules, Default Room Image, `options_always_on`, Map JSON,
  Scenarios JSON.
- **Text Check is coupled.** Mode "Disabled" disables and clears "check before
  sending", so the two controls cannot contradict each other (finding 4.3). The
  Save button and the `#settings-status` "saved!" line go.
- **The World posture selects keep auto-saving**, but move into their own
  labelled group ("Posture — saves automatically") with their own status,
  visually separate from the details form. The page states that Cancel applies
  to the details form only. This is how finding 4.4 is resolved: the mix is
  deliberate and visible, not removed.
- **The stale-status bug (05.F8) is fixed by scope:** a submitted details form
  clears the posture group's "Saved", so a failed Update World cannot sit under a
  green confirmation.
- **`POST /settings` is deleted** together with `save_settings_handler` and the
  tests that cover only it. No UI control posts to it; the live path is *Set as
  Narrator/Quantifier*. (Q1b: it would only have had a purpose under option B.)
- **No spec change is needed for the posture auto-save itself.** The world and
  game posture specs (`worlds.md` 25.5, `browser_worlds.md` 29.2, `games.md`
  20.8) still describe what the code does. A scenario changes only if it asserts
  the removed Text Check Save button or the `#settings-status` line.

### Graduated

[Make the dashboard panels consistent: save model and layout](58-panel-consistency.md),
shared with the layout decision.
