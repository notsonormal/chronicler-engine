# Task: Options UI — input-surface dock (Use / Edit / regenerate) + /options slash-menu item

Type: task
Status: pending
Blocked by: 10, 14

## Question

Implement the rendering and selection flow for options-autogeneration as decided in ticket 04 (decisions 7–8): a vertical stack of full-width option buttons docked above the input box — options are part of the INPUT surface, not the last message. This is implementation per the map's Notes override.

### Scope

1. **Options dock template + view model.** A new template fragment (mirroring the swipe-controls pattern in `src/adapters/driving/http/templates.rs:25`) rendered ABOVE the command form inside the action area (`templates.rs:76`), driven by a new view-model in `view_models.rs`. The dock renders the CURRENT options set from game state, so it survives a page reload mid-turn (ticket 04 decision 8). Visual reference: the approved prototype `tmp/prototype-options-ui/index.html` variant V4 (vertical full-width buttons, cyan-left-border style, label + regenerate control on top) — port its CSS into `assets/styles.css` with proper token names, then DELETE the prototype directory.

2. **Use.** Clicking an option button submits its text as the player's input through the normal `/action/check` path — it becomes `MessageType::Input` and runs the normal narration pipeline (ticket 04 decision 3). The dock clears when the turn advances.

3. **Edit.** A per-option ✎ button copies the option text into the command input client-side, so the player can modify it or prepend `/guide` before sending (ticket 04 decision 4 — this is what makes options compose with guide steering).

4. **Regenerate.** A ♻ control re-runs options generation (ticket 10's path) against the same scene and swaps the dock in place via HTMX — re-run-and-replace, no option-set history (ticket 04 decision 9).

5. **`/options` slash-menu item.** Add to the slash menu alongside `/impersonate` (the user-facing entry for the on-demand trigger). Confirm where the slash menu is defined during implementation and record the path.

6. **Always-on refresh.** In always-on games, the dock refreshes after each narration-producing turn (HTMX swap of the action-area/dock fragment alongside the story-log swap). After impersonate turns the dock is left as-is (no auto-fire, ticket 04 decision 4).

## Notes for the session

- Read before implementing: ticket 04's Answer; `src/adapters/driving/http/templates.rs` (story-log + action-area templates); `src/adapters/driving/http/view_models.rs`; `src/adapters/driving/http/chat_window/` (the action endpoints); `assets/styles.css` (swipe-controls + action-btn tokens to reuse); the prototype file above.
- No per-message option state exists — do NOT anchor options to story-log entries (that was the rejected V1/V2 shape). The dock reads only current game state.
- Build-green check: `python build.py` (fast suite), then verify the dock manually against a running game (`cargo run -- --world redmist_estate --port 3000`) — generate, use, edit, regenerate, reload-page persistence.
- Skills: `/domain-modeling`.
