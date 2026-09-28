# Task: Posture relocation — UI (world-editor posture dropdowns + in-game override + per-game mode-switch action)

Type: task
Status: resolved
Blocked by: 06, 14

## Question

Implement the UI half of the posture relocation: the author sets posture on the world; the player overrides posture per-game; the mode-switch action retargets presets + nudges perspective + re-renders. This is implementation per the map's Notes override.

### Scope

1. **World-editor posture dropdowns.** Add mode/perspective/tense dropdowns to the existing world edit form (`/worlds/:key/edit` → `edit_world_form_handler`; `/worlds` POST → `update_world_handler` in `src/adapters/driving/http/worlds/`). Auto-save-on-change (HTMX `hx-trigger="change"`, per ticket 01 decision 5). Follow the existing narrative-voice dropdown pattern (`src/adapters/driving/http/settings/templates/settings.rs`) for the markup.

2. **Per-mode activation + flags UI (ticket 13 Q5).** Activation is mode-targeted: each preset card gains per-mode activate actions (set as Novel default / set as IF default), shown only for modes in the preset's `allowed_modes`; the activate endpoint (ticket 14) validates the mode against the flags. The Default badge becomes per-mode. The preset editor gains `allowed_modes` checkboxes (at least one required; editor-level only — see Answer; new presets default to both). The per-game preset picker filters presets by the game's mode.

3. **In-game posture override UI (NEW).** A new fragment for per-game mode/perspective/tense override, surfaced where the player manages their game. Inherited from the world at creation; overridable here. Auto-save-on-change.

4. **Per-game mode-switch action.** The distinct action (ticket 01 decision 5): sets the game's mode, retargets the game's three preset-ids to the new mode's default bundle (from the global registry), nudges the game's perspective per the conditional rule (ticket 01 decision 4: nudge only if at the other mode's default), and re-renders the override UI. Mirrors `set_narrator_handler`'s "explicit retarget + full re-render" pattern.

## Notes for the session

- Read before implementing: `src/adapters/driving/http/worlds/` (handlers + templates), `src/adapters/driving/http/settings/handlers/settings.rs` + `templates/settings.rs` (the dropdown + save handler pattern to mirror), `src/adapters/driving/http/prompt_presets/handlers/prompt_presets.rs` (the activate handler to make mode-aware), `src/adapters/driving/http/games/` (game management routes — where the override UI lands), `src/adapters/driving/http/builders/router.rs` (route registration).
- The mode-switch action is the load-bearing part: it must retarget presets AND nudge perspective AND re-render in one POST. Do not fold it into a generic save.
- Build-green: `python build.py`. IF mode end-to-end needs ticket 02's IF preset to retarget coherently; the mechanics must build and novel mode must stay green.
- Blocked by 06 (the narration path must read from the game before the UI can drive it).
- Skills: `/prototype` (a stub of the in-game override fragment may sharpen the UX), `/domain-modeling`.

## Answer

Shipped 2026-09-07 — all four scope items plus browser behaviour tests with spec scenarios; `python build.py` fully green (fmt, clippy, guardrails, python tests, doc freshness, architecture, 1513 passed / 0 failed / 2 skipped including 23 Playwright browser tests; log `logs/build_20260907_223404.log`).

### Browser tests + specs (added per user decision — tests ship with their scenarios)

Six Playwright tests in `tests/browser/behaviour.rs` cover the new UI in a real browser; scenarios were written into the specs at the same time (validator: 116 declared, 116 covered, 0 gaps, 0 orphans). Ticket 12 verifies coverage.

- `games.md` 20.1–20.3 (new section "Per-game posture, mode, and presets"): fragment renders for the active game; tense change auto-saves and re-renders; mode switch retargets presets to the IF bundle and nudges perspective to Second.
- `browser.md` 18.1–18.2 (new section "World posture editor"): edit form renders the posture selects; a change auto-saves and reports `Saved` in `#world-posture-status`.
- `prompt_presets.md` 21.27 (new section "Editor mode flags"): editor exposes the Allowed Modes checkbox group; checking IF + saving re-renders the card with `Set Active (IF)`. Scenario 21.14's gherkin now also pins the preserve-on-omitted-flags contract, asserted in its existing HTTP test.

### Deviation from ticket 13 decision 1, part 2 (found by the browser tests)

axum 0.7's `Form` (serde_urlencoded) cannot deserialize `Vec<String>` from a checkbox group — a single checked box posts one scalar value and the extractor 422s (`invalid type: string, expected a sequence`). The real-browser test caught this; unit tests missed it because they construct `PresetForm` directly. `PresetForm` now carries two boolean checkbox fields (`allowed_mode_novel` / `allowed_mode_if`, `value="true"`, `#[serde(default)]`) and `parse_allowed_modes` is gone — invalid mode strings are rejected by the extractor before the handler. Preserve-on-omitted-flags semantics unchanged (both-false = field absent/unchecked-all).

### What landed

1. **World-editor posture dropdowns.** `WorldForm` gains `narrator_mode`/`narrative_perspective`/`narrative_tense` (Option, `parse_or_default`); `WorldFormTemplate` renders the three selects on create AND edit; in edit mode each select auto-saves on change via `hx-include="closest .posture-group"` → new `POST /worlds/:key/posture` (`update_world_posture_handler`), which patches posture only and returns a `Saved` status span. `update_world_handler` now preserves stored posture when a post omits all three fields (a legacy client could otherwise reset posture — the pre-existing `..Default::default()` latent bug). Selects sit inside the main form, so a normal Update World also carries posture.
2. **Per-mode activation + flags UI.** `preset_card_html` is now per-mode: badges `Active · Novel` / `Active · IF` and per-mode `Set Active (Novel/IF)` buttons gated by `allows_*` (the panel template already had these; the card builder now matches). The preset editor gains `allowed_modes` checkboxes in `preset_edit_form_html`; `PresetForm.allowed_modes: Vec<String>` with `#[serde(default)]`. Create with no flags → all modes (unchanged); create/update with invalid mode strings → error span.
3. **Per-game preset picker (built — it did not exist).** The games panel's Active Game section renders a new `GamePostureTemplate` fragment (embedded as pre-rendered `posture_html`, the `provider_options|safe` pattern): three preset selects (system/quantifier/impersonate) filtered by the game's mode via `allows()`; the stored selection always stays selectable, and a stored id missing from the library renders as `(missing) <id>` so the browser never silently substitutes another preset. Auto-save on change → `POST /games/:id/presets`.
4. **In-game posture override + mode-switch action.** The same fragment carries the mode/perspective/tense selects. Perspective/tense auto-save → `POST /games/:id/posture`. Mode is the distinct action → `POST /games/:id/mode` (`switch_game_mode_handler`), which per ticket 01 decisions 4–5 retargets the game's three preset-ids to the new mode's registry bundle, nudges perspective only when it sits at the other mode's default, and re-renders the fragment (never folded into a generic save). All three endpoints return the re-rendered fragment; validation failures render an error span in place (house style); storage failures stay 500s.

### Application + storage seams

- `GameCatalogue` gains `current_game()`, `set_posture(id, perspective, tense)`, `switch_mode(id, mode)` (retarget + conditional nudge; same-mode is a no-op so re-picking a mode never resets customized preset-ids), and `set_preset_selection(id, s, q, i)` — a changed id must exist and allow the game's mode; an unchanged stored id is a no-op slot that skips validation (so saving the form never fails on a stale/deleted selection). Flags gate selection surfaces only, per ticket 13 decision 6 — the narrate path never re-validates.
- `Storage::update_game_config(&Game)` (SQLite + in-memory) persists the posture triple + three preset-ids and bumps `updated_at`.
- Domain: `NarratorMode::default_perspective()` (Novel→Third, IF→Second) names the nudge rule where the modes live.

### Deviation from ticket 13 decision 1 (recorded)

"Editor requires ≥1 allowed mode" is editor-level only. Urlencoded forms cannot distinguish "all unchecked" from "field absent", so the update path treats empty `allowed_modes` as field-absent and preserves the stored flags (restoring ticket 14's preserve behavior). A server-side ≥1 hard error would break spec scenario 21.14 (update without mode flags returns the card). Unchecking all + saving simply re-renders the unchanged flags.

### Discovered edge (known, not fixed)

`delete_preset_handler` refuses presets referenced by the registry (any mode default) but NOT presets referenced as a game's active id — deleting such a preset leaves that game's stored id dangling. The picker renders it as `(missing) <id>` and the endpoint treats it as a no-op, so nothing breaks silently; the fix (a per-game reference check on delete) is a small follow-up if wanted.

`switch_mode` retarget copies the new mode's registry bundle ids into the game without an `allows(mode)` check (decision 5-C). The invariant "a mode's active preset allows that mode" is enforced at activation (`activate_preset_handler` refuses a disallowed mode), but a non-default active preset can have its flags narrowed via the editor after activation — narrowing IF off a preset that is the IF bundle's active default leaves that slot pointing at a preset that no longer allows IF. Reachable only through that post-activation narrowing path; the game degrades gracefully (the picker renders the stored id, `set_preset_selection` treats a disallowed stored id as a no-op slot, and the narrate path never re-validates per decision 6). Discarded as a known edge; the principled fix is to refuse a flag edit that strips a mode where the preset is an active bundle slot.

### Tests

Unit: catalogue (posture save, retarget+nudge, deliberate-perspective keep, same-mode no-op, selection accept/reject/no-op-slot), preset builders (per-mode card badges/buttons, disallowed-mode button suppression), preset handlers (flags from form as booleans, preserve-on-omit, create defaults to both), games handlers (mode retarget+nudge, invalid mode error, posture save, invalid posture, presets save, unknown preset). Browser: six Playwright behaviour tests (above) + the 23-test browser suite green. HTTP E2E: `http_routes.md` regenerated (52 → 56 routes; count ratchet in `test_extract_http_routes.py` bumped), prompt_presets scenarios 21.x green. Ticket 12 remains the owner of the remaining spec surface: worlds create/update contract, HTTP-contract scenarios for the new endpoints (invalid mode/preset error spans), and any further browser invariants.
