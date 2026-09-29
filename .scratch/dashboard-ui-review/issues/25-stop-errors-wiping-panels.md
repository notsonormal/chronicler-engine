# Stop server errors from wiping whole panels

Type: task (AFK)
Status: open
Blocked by: 08

## Question

Failed adds replace the entire panel with a bare error fragment: an invalid provider on Settings → Add connection replaces all of `.settings-panel`'s inner HTML (every connection card, the add form, Text Check), and an invalid `preset_type` on Prompt Presets replaces `.prompt-presets-panel` with the text `Invalid preset type` (both are HTTP 200 bodies, so no toast fires). What change makes an add failure report itself without destroying the panel?

## Context

- Finding 05.F1 (P1). Screenshots 54 (empty Settings panel holding only the error) and 51 (Prompt Presets showing only "Invalid preset type"); ticket 05 answer.
- Mechanism: handlers return `Html(render_error(..))` / `<span class='error'>` with status 200, and the forms target the whole panel (`hx-target=".settings-panel" hx-swap="innerHTML"` for connections, `hx-target=".prompt-presets-panel" hx-swap="outerHTML"` for presets). Recovery is a page reload.
- The same bodies are also returned for storage failures (`Save failed: ..`, `Load failed: ..`), so this is not only reachable through tampered form values.
- The choice of where the error should appear is [Decide how the dashboard shows each kind of failure](08-decide-failure-display.md). This ticket is the implementation side for adds; do not invent a parallel error surface.

## Done when

- A failed add (connection or preset) leaves the panel and its other controls intact, and the error is visible near the form.
- A test covers the failed add path; tier by `tests/STRATEGY.md`.
- `python build.py` is green. Commit after user approval.
