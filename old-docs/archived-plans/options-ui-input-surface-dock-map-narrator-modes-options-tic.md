# Options UI — input-surface dock (Map: Narrator Modes & Options, Ticket 11)

## Summary

Implement ticket `.scratch/narrator-modes-and-options/issues/11-implement-options-ui.md`: render the game's current option set as a vertical button dock *above the command form inside the action area* (ticket 04 decision V4), with Use / ✎ Edit / ♻ regenerate, a `/options` slash-menu item, and the world's `options_always_on` checkbox in `WorldForm`. Ticket 10 already ships everything server-side — `NarrativeState.current_options` (persisted, `action_pipeline/options.rs:197` clears first at every turn end), `Action::Options` (`/options` engine command through `/action/check`) — so this ticket is presentation only: **one** new route (`GET /fragment/options-dock`), no new mutating endpoint. ♻ regenerate reuses the existing `/action/check` engine-command path via an htmx mini-form (user decision). Refresh uses the repo's polling topology: `#options-dock` self-polls every 2s mirroring `#story-log`, so "dock clears when the turn advances" and reload persistence fall out of ticket 10 server behavior. World checkbox uses Option A grammar (serde-default bool, `value="true"`); two cheap unit pins (WorldForm urlencoded grammar, dock template render) ship in this ticket per review.

## Key Changes

**Dock render surface.** `view_query.rs` — `get_current_options()` (mirror `get_story_log_entries` state access). `view_models.rs` — `OptionsDockViewModel { options: Vec<String>, is_busy: bool }` (busy from existing `get_generating_status`; disables all dock buttons while generating). `templates.rs` — inline `OptionsDockTemplate`: label row ("options — pick one, or type your own" + ♻ mini-form `hx-post="/action/check"` with hidden `command=/options`, `hx-on::before-request="updateToThinking()"` — HX-Retarget headers route its status response to `#status-display`) + `.option-item` rows (full-width `.option-btn` `type="button"` + ✎ `.mini-btn` `type="button"`); empty options → empty body (container persists for polling). Askama escapes option text in markup; JS moves it via `textContent` — quote/HTML-safe end to end. `ActionAreaTemplate` gains the dock container above `#command-form`.

**Wiring (one route).** `app_state.rs` — `render_options_dock()` (propagates `Err` like `render_action_area`; the `render_fragment` glue renders error HTML on failure). `layout/handlers/endpoints.rs` — `options_dock_fragment`. `builders/router.rs` — `GET /fragment/options-dock` only.

**`assets/index.html`.** Static-shell dock container `<div id="options-dock" hx-get="/fragment/options-dock" hx-trigger="load, every 2s" hx-swap="innerHTML"></div>` above `#command-form`. `SLASH_COMMANDS` gains `{ cmd: "/options", desc: "Suggest actions (generate pickable options)" }` (slash menu definition = `assets/index.html:422`, recorded per ticket). `onStatusPoll` gains `"options": "Generating options..."` in its thinking set. Two functions: `useOption(btn)` (option textContent → input → `#command-form.requestSubmit()` — full normal path incl. auto text-check, deliberate per ticket 04 decision 3) and `editOption(btn)` (fills input + focus, enabling `/guide` prepend; draft untouched by ♻ mini-form by construction).

**CSS + cleanup.** Port prototype V4 (`.options-strip`, `.options-label`, `.option-item`, `.option-btn`, `.mini-btn`, hover/disabled states) into `assets/styles.css` on existing tokens; skip `.chosen` (V4 clears on submit by design). Delete `tmp/prototype-options-ui/`. Regenerate `docs/diataxis/reference/frontend/http_routes.md` via `scripts/extract_http_routes.py` (one new route).

**World checkbox (Option A).** `WorldForm` gains `#[serde(default)] options_always_on: bool`; `into_world_card` uses the form value instead of hardcoded `false`; `update_world_handler` (read-modify-write) patches the field — absent → `false` matches the posture fields' absent→reset contract. `WorldFormTemplate` gains a checkbox row after the posture group (label "Auto-generate options after each turn", `value="true"`, pre-checked from stored value), create + edit forms, Save-driven (no new auto-save endpoint).

**Tests (two pins, existing files).** `templates_tests.rs` — dock render matrix: empty → empty body; 3 options → 3 rows, text escaped; `is_busy` → all buttons disabled. Add `serde_urlencoded` as dev-dependency; `worlds_tests.rs` — full WorldForm body with `options_always_on=true` parses true; body without the field parses false; checkbox-on-then-off Save flips the value (grammar pin, pre-pays ticket 12 debt item 3).

## Implementation

### Phase 1: Dock render surface (5 SP)

- [ ] #### Task 1.1: Claim ticket, then dock data path + fragment (5 SP)
  - [ ] ##### SubTask 1.1.1: Set `Status: claimed` in ticket 11 (issue-tracker convention, before any work) (1 SP)
  - [ ] ##### SubTask 1.1.2: `get_current_options()` in `GameViewQuery`; `OptionsDockViewModel`; inline `OptionsDockTemplate` (incl. ♻ mini-form + onclick hooks); `AppState::render_options_dock()`; `options_dock_fragment`; `GET /fragment/options-dock` route; dock container in `ActionAreaTemplate` and the `assets/index.html` static shell (3 SP)
  - [ ] ##### SubTask 1.1.3: `python build.py fmt && python build.py check`; hit `/fragment/options-dock` on a running game (200, empty body on empty set) (1 SP)

### Phase 2: Interactions, styling, template tests (5 SP)

- [ ] #### Task 2.1: Use / Edit / slash item / status phase + CSS + render-unit (5 SP)
  - [ ] ##### SubTask 2.1.1: `useOption`, `editOption`, `SLASH_COMMANDS` entry, `onStatusPoll` `"options"` phase mapping in `assets/index.html`; dock buttons now live (2 SP)
  - [ ] ##### SubTask 2.1.2: Port V4 CSS into `assets/styles.css` on repo tokens; eyeball against V4; delete `tmp/prototype-options-ui/` (2 SP)
  - [ ] ##### SubTask 2.1.3: Dock render-unit in `templates_tests.rs` (empty / 3-options / busy matrix); `python build.py check` (1 SP)

### Phase 3: World checkbox + pins + gate + resolution (3 SP)

- [ ] #### Task 3.1: WorldForm field + handlers + checkbox row (1 SP)
  - [ ] ##### SubTask 3.1.1: `#[serde(default)] options_always_on: bool` in `WorldForm`; `into_world_card` + `update_world_handler` patch; checkbox row in `WorldFormTemplate` (create + edit, pre-checked) (1 SP)
- [ ] #### Task 3.2: Grammar pin + docs + gate (1 SP)
  - [ ] ##### SubTask 3.2.1: `serde_urlencoded` dev-dep + WorldForm checked/unchecked parse pins in `worlds_tests.rs`; `python scripts/extract_http_routes.py`; full `python build.py` green (1 SP)
- [ ] #### Task 3.3: Manual UI verification + map bookkeeping (1 SP)
  - [ ] ##### SubTask 3.3.1: Manual checklist below; then resolve ticket 11 — Answer records four decisions (poll-based dock refresh; ♻ reuses `/action/check` mini-form instead of a dedicated endpoint; Option A grammar supersedes the hidden-input note; two unit pins shipped early), map Decisions-so-far pointer, Not-yet-specified gains **per-game always-on override UI**, ticket 12 noted unblocked (1 SP)

## Test Plan

1. **Unit (new):** dock render matrix (`templates_tests.rs`); WorldForm urlencoded grammar (`worlds_tests.rs`). Spec + integration coverage of options/IF mode stays with ticket 12 as decided.
2. **Gate:** `python build.py fmt`, then full `python build.py` green.
3. **Manual UI verification** (`cargo run -- --world redmist_estate --port 3000`, mock-LLM game; chronicler-ui-investigator skill):
   - `/options` in the slash menu; sending shows "Generating options..." then 3 dock buttons (≤ ~2s via poll).
   - **Use:** text enters history as player input, narration follows; dock clears at turn end; button clicks during generation are visibly disabled.
   - **Edit:** fills input without submitting; `/guide <steer> <option>` composition sends normally.
   - **♻:** set replaced; half-typed command draft survives; concurrent-generation click shows "Still thinking...".
   - **Always-on:** world checkbox persists over Save; new game inherits; narration turn → dock refresh; impersonate turn leaves set; page reload mid-set → dock persists.
   - **Fresh game:** `/options` with no history → error notification, no panic, dock unchanged.

## Per Task/Sub Task Validation Steps

- 1.1.2 → `python build.py check`; browser: `/fragment/options-dock` returns 200 with empty body on a fresh game.
- 2.1.1 → send `/options` in a running game; dock populates without page reload.
- 2.1.2 → visual parity against V4 before deleting the prototype directory.
- 2.1.3 → `python build.py unit`.
- 3.1.1 → create world with toggle on → stored true; new game inherits; edit-save toggling off flips; unrelated edits preserve it.
- 3.2 → routes-doc diff shows exactly one new route; full gate green.
- 3.3 → manual checklist; `.scratch` edits only.

## Assumptions

- ♻ regenerate goes through `/action/check` (user-locked Finding 1): dock's mini-form posts `command=/options`; `is_engine_command()` bypasses text-check; `add_status_swap_headers` retargets the response; `EngineError::Validation` surfaces via the global htmx error handler. No dedicated endpoint, no new JS fetch function.
- Option A checkbox grammar (user-locked earlier) supersedes the ticket note's hidden-input prescription.
- Dock refresh is the self-polled fragment, not HX-Trigger push — consistent with `#story-log`; makes scope items 2 (clears on advance) and 6 (always-on refresh) consequences of ticket 10 server behavior, requiring no UI-side turn detection.
- Options receive the same auto text-check as typed input (Use submits through the normal path) — deliberate, per ticket 04 decision 3; the save/restore flow already covers the dock through a preview.
- Per-game always-on override UI is out of this ticket; becomes a fog item at resolution.
