# Make the dashboard panels consistent: save model and layout

Type: task (AFK)
Status: resolved
Blocked by: —

## Question

Implements the decisions in [Decide one save model for panel forms](16-decide-save-model.md)
and [Decide the panel layout convention and supported viewports](19-decide-layout-convention.md).
Theme 4 of the review, done as one sweep.

### Save model — option E, hybrid by control type

- **Instant**, with visible feedback: connection *Set as Narrator* / *Set as
  Quantifier* / *Delete*; Text Check mode and *check before sending*; World
  Narrator Mode / Perspective / Tense.
- **On Save:** connection Add and Edit (already so); World details — Name,
  Description, Global Rules, Default Room Image, `options_always_on`, Map JSON,
  Scenarios JSON.
- **Couple the Text Check pair.** Mode "Disabled" disables and clears "check
  before sending". Remove its Save button and the `#settings-status` "saved!"
  line (finding 4.3).
- **World posture group.** Keep the selects auto-saving, but move them into a
  labelled group ("Posture — saves automatically") with their own status,
  visually separate from the details form. State on the page that Cancel applies
  to the details form only, so the mix is visible rather than confusing
  (finding 4.4).
- **Fix the stale status (finding 05.F8).** Submitting the details form clears
  the posture group's green "Saved", so a failed Update World cannot sit under a
  green confirmation.
- **Delete `POST /settings`** and `save_settings_handler`, plus the tests that
  cover only that route.

### Layout — option A1

- One centered content column for every panel: `max-width` 960px, 24px padding.
  Replaces Settings/Games at 800px, Prompt Presets at 900px, Worlds with no cap.
- The tab body is the only scroll region. Remove the inner scroll boxes from
  Games, Prompt Presets, and Worlds; Settings already scrolls the whole tab.
- Worlds: give the form view the same card frame as the list view, or remove the
  card inset in both, so the text inset does not change between views.
- The tab bar scrolls horizontally below 1024px, so it cannot overflow.
- Declare the viewports in `docs/diataxis/reference/frontend/ui_design.md`:
  ≥1024×700 fully supported; 768–1024 best-effort; phone out of scope.

## Checks before starting

- `git status` for overlapping edits. This ticket shares the Prompt Presets and
  Worlds templates with [Expose Options presets in the UI](60-options-presets-ui.md).
- The Settings-panel internals also appear in
  [Settings panel: roles, buttons and text-check controls](15-settings-panel-prototype.md).
  Coordinate: 15 owns the Settings look; this ticket owns the Text Check
  save behaviour. If 15 resolves first, apply its design here.
- Check the posture specs (`docs/specs/worlds.md` 25.5,
  `docs/specs/browser_worlds.md` 29.2, `docs/specs/games.md` 20.8). They stay
  true; a scenario changes only if it asserts the removed Text Check Save button
  or the `#settings-status` line.

## Done when

- `python build.py` is green.
- The save-model and layout rules are stated in the reference docs.
- The user has reviewed the diff.

## Answer

Absorbed into [Panel consistency and Options presets](67-panel-consistency-and-options-presets.md).
The save-model and layout sweep moved there, since both it and the Options-preset
work rewrite the Prompt Presets and Games templates and handlers. Closed as
absorbed, not fixed.
