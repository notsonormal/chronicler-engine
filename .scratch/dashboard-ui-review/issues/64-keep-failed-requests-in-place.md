# Keep failed forms and cards in place

Type: task (AFK)
Status: open
Blocked by: 63

## Question

One class of failure, two surfaces: a failed add, edit or refused delete must
not swap into the region it describes. Today it does, and the panel or card
disappears with only the error text left.

- **Whole panels** (finding 05.F1, P1). An invalid provider on Settings → Add
  connection replaces all of `.settings-panel`'s inner HTML (every connection
  card, the add form, Text Check); an invalid `preset_type` on Prompt Presets
  replaces `.prompt-presets-panel` with the text `Invalid preset type`. Both are
  HTTP 200 bodies, so no toast fires.
- **Entity cards** (finding 05.F2, P1). A failed connection edit replaces the
  connection's own card (`hx-target="closest .connection-edit-form"`,
  `outerHTML`), so the connection leaves the list. Deleting an active preset is
  correctly refused, but the refusal body replaces the preset card
  (`hx-target="closest .preset-card"`), so the card vanishes and only the error
  text is left.
- A failed preset **edit** (`update_preset_handler`) replaces the card with a
  bare error span and loses in-progress edits (added from ticket 17).

Both recover only on a page reload.

## Context

- Findings 05.F1 and 05.F2; screenshots 54 (empty Settings panel holding only the
  error), 51 (Prompt Presets showing only "Invalid preset type"), 56 (connection
  card replaced by `Error: Configuration error: Unknown LLM backend
  'bogus_provider'`) and 52 (bare `Preset is a mode default; change the default
  before deleting` where the card was); ticket 05 answer.
- Mechanism: handlers return `Html(render_error(..))` / `<span class='error'>`
  with status 200, and the forms target the whole panel
  (`hx-target=".settings-panel" hx-swap="innerHTML"` for connections,
  `hx-target=".prompt-presets-panel" hx-swap="outerHTML"` for presets).
- Where the message appears is decided in
  [Decide how the dashboard shows each kind of failure](08-decide-failure-display.md);
  this ticket is the implementation for panels and cards. A failure never swaps
  into the region it describes. The message renders in an inline slot inside the
  form or card — a short user sentence, with the raw server text behind an
  anchored popover.
- [Redesign the error and health display](63-redesign-error-health-display.md)
  owns the shared short-message + anchored-popover error fragment and the
  `beforeSwap` behaviour. Consume it here rather than building a second shape.
- The same bodies are also returned for storage failures (`Save failed: ..`,
  `Load failed: ..`), so this is not only reachable through tampered form values.
- The refusal logic itself is fine and should stay: default presets and
  mode-default references must not be deleted.

## Done when

- A failed add (connection or preset), a failed connection edit, a failed preset
  edit, and a refused preset delete all leave the panel and its other controls,
  or the card, intact and usable, with the message shown nearby.
- Tests cover the failed add path and both card paths; tier by
  `tests/STRATEGY.md`.
- `python build.py` is green. Commit after user approval.
