# Task: Options UI — input-surface dock (Use / Edit / regenerate) + /options slash-menu item

Type: task
Status: resolved
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
- World-toggle coupling (SUPERSEDED by this ticket's shipped decision): the note below was the pre-implementation guidance; the shipped form grammar is the repo's PresetForm checkbox convention — `#[serde(default)] pub options_always_on: bool` on `WorldForm`, checkbox `value="true"`, no hidden input (an unchecked box posts nothing and serde's default yields `false`, which the absent-field-clobber concern becomes deliberately correct behaviour). An absent field on update resets the stored flag to false; the parse + handler-flip pins live in `src/adapters/driving/http/worlds/handlers/worlds_tests.rs`. The originally prescribed hidden-input `value="0"/"1"` pattern does not parse as a serde bool and was rejected (ticket 04 follow-up, user-approved Option A).
- Build-green check: `python build.py` (fast suite), then verify the dock manually against a running game (`cargo run -- --world redmist_estate --port 3000`) — generate, use, edit, regenerate, reload-page persistence.
- Skills: `/domain-modeling`.

## Answer

Shipped. The options dock renders the current offered set above the command form, Use/Edit/regenerate work end-to-end in a real browser, `/options` is in the slash menu, and the world form carries the always-on checkbox. Full gate green: 1585 passed / 0 failed / 2 skipped (LLM).

### What landed

1. **Dock render surface** (scope 1): `OptionsDockTemplate` (inline in `templates.rs`, whole body wrapped in an if-not-empty guard so an empty set renders an empty body) + `OptionsDockViewModel` + `GameViewQuery::get_current_options` + `AppState::render_options_dock` + `options_dock_fragment` endpoint. One new route: `GET /fragment/options-dock` (http_routes.md regenerated; the pinned route count in `scripts/tests/test_extract_http_routes.py` bumped 56 → 57). The dock is a self-polling `#options-dock` div (hx-get every 2s) inside `#action-area`, duplicated in the `index.html` static shell and `ActionAreaTemplate`.
2. **Use** (scope 2): fills the command input with the option text, then `form.requestSubmit()` through the normal `/action/check` path — the option becomes `MessageType::Input` and the turn pipeline runs as normal.
3. **Edit** (scope 3): per-row ✎ copies the text into the input and focuses it; no submit.
4. **Regenerate** (scope 4): an htmx mini-form inside the dock with hidden `command=/options` posting to the existing `/action/check` (engine commands bypass text-check; `add_status_swap_headers` retargets the response to `#status-display`; `updateToThinking()` wired via `hx-on::before-request`). No new mutating endpoint, no `regenerateOptions` JS — the draft input text is preserved because the command form is never touched.
5. **Slash menu** (scope 5): `/options` added to `SLASH_COMMANDS` in `assets/index.html`; both slash-menu browser tests and `docs/specs/browser.md` Scenario 17.1 updated to the three-item list.
6. **World checkbox** (scope 6 coupling): `options_always_on` on `WorldForm` + `WorldFormTemplate` checkbox row (posture group, main form — not the posture auto-save path); `into_world_card` and `update_world_handler` now carry the field. Per-game always-on override UI remains out of scope (the game column exists; no UI edits it — recorded on ticket 12's notes as the e2e recipe).

### Deviations from the approved plan

- **Regenerate transport**: the plan's dedicated `POST /options/regenerate` endpoint became the `/action/check` mini-form above (user-approved during planning) — one new route total, both GET.
- **Checkbox grammar**: Option A serde-default bool superseding the ticket note's hidden-input pattern (user-approved; the hidden-input `value="0"/"1"` body does not parse as a serde bool).
- **Two early unit pins**: `OptionsDockTemplate` render matrix (empty/escaped-rows/busy-disabled) in `templates_tests.rs` + WorldForm parse/handler-flip pins in `worlds_tests.rs`, requiring `serde_urlencoded = "0.7"` as a dev-dependency (pre-pays ticket 12's PresetForm debt item).
- **Mock LLM options branch**: `MockBackend::complete` returns canned `<suggestion>` options for the `options` agent when no prompt responses are seeded — deliberate test-infra scope addition so browser/E2E runs exercise the real generation path. Initially placed before the fail/seeded guards (broke three ticket-10 tests); re-scoped to unseeded-only with the branch priority pinned in `mock_tests.rs`.

### Verification (ticket-11 lane, per plan: build-green + manual UI)

A throwaway real-browser test (deleted after the run — ticket 12 owns shipped spec coverage) verified in Chromium against the real server: dock populates from a real options generation (V4 visual parity: vertical cyan full-width buttons, label row with ♻, ✎ per row); Use submits through the normal path and the option appears as `Input` with the narration following; Edit fills without submitting; regen preserves the draft input; the concurrency gate rejects a dock click during generation (observed live, matches `send_action`'s design); slash menu shows three items. Always-on/impersonate/reload semantics are pinned at the application layer by the existing `options_tests.rs` suite plus two new resolution pins (game-row-first: `test_world_toggle_does_not_enable_always_on_for_running_game`; world fallback: `test_unreadable_game_row_falls_back_to_world_toggle`); the snapshot round-trip carries `current_options` (`game_state_snapshot.rs`), which is the state-layer reload-persistence guarantee.

### Findings recorded for ticket 12

- **Game-row-first toggle**: flipping the WORLD row does not enable always-on for an existing game (inheritance at creation). The ticket-12 e2e must seed the fixture world before game creation, or set the game row directly (note added to ticket 12).
- **Vacuous-pass trap fixed at the code layer**: `wait_for_element_children` timed out silently (returned the count), which made the throwaway's always-on section pass empty-vs-empty; the helper now panics on timeout like its siblings, and `integration_test_standards.md` Cross-cutting 5 records the helper contract.
- **Legacy test deleted**: `tests/http/requires_migration/worlds_fragment_handlers.rs::test_update_world_preserves_options_always_on` — its premise (the form does not model the flag) is superseded by the checkbox; replacements are the `worlds_tests.rs` pins.
