# Task: Posture relocation — UI (world-editor posture dropdowns + in-game override + per-game mode-switch action)

Type: task
Status: pending
Blocked by: 06, 14

## Question

Implement the UI half of the posture relocation: the author sets posture on the world; the player overrides posture per-game; the mode-switch action retargets presets + nudges perspective + re-renders. This is implementation per the map's Notes override.

### Scope

1. **World-editor posture dropdowns.** Add mode/perspective/tense dropdowns to the existing world edit form (`/worlds/:key/edit` → `edit_world_form_handler`; `/worlds` POST → `update_world_handler` in `src/adapters/driving/http/worlds/`). Auto-save-on-change (HTMX `hx-trigger="change"`, per ticket 01 decision 5). Follow the existing narrative-voice dropdown pattern (`src/adapters/driving/http/settings/templates/settings.rs`) for the markup.

2. **Per-mode activation + flags UI (ticket 13 Q5).** Activation is mode-targeted: each preset card gains per-mode activate actions (set as Novel default / set as IF default), shown only for modes in the preset's `allowed_modes`; the activate endpoint (ticket 14) validates the mode against the flags. The Default badge becomes per-mode. The preset editor gains `allowed_modes` checkboxes (at least one required; new presets default to both). The per-game preset picker filters presets by the game's mode.

3. **In-game posture override UI (NEW).** A new fragment for per-game mode/perspective/tense override, surfaced where the player manages their game. Inherited from the world at creation; overridable here. Auto-save-on-change.

4. **Per-game mode-switch action.** The distinct action (ticket 01 decision 5): sets the game's mode, retargets the game's three preset-ids to the new mode's default bundle (from the global registry), nudges the game's perspective per the conditional rule (ticket 01 decision 4: nudge only if at the other mode's default), and re-renders the override UI. Mirrors `set_narrator_handler`'s "explicit retarget + full re-render" pattern.

## Notes for the session

- Read before implementing: `src/adapters/driving/http/worlds/` (handlers + templates), `src/adapters/driving/http/settings/handlers/settings.rs` + `templates/settings.rs` (the dropdown + save handler pattern to mirror), `src/adapters/driving/http/prompt_presets/handlers/prompt_presets.rs` (the activate handler to make mode-aware), `src/adapters/driving/http/games/` (game management routes — where the override UI lands), `src/adapters/driving/http/builders/router.rs` (route registration).
- The mode-switch action is the load-bearing part: it must retarget presets AND nudge perspective AND re-render in one POST. Do not fold it into a generic save.
- Build-green: `python build.py`. IF mode end-to-end needs ticket 02's IF preset to retarget coherently; the mechanics must build and novel mode must stay green.
- Blocked by 06 (the narration path must read from the game before the UI can drive it).
- Skills: `/prototype` (a stub of the in-game override fragment may sharpen the UX), `/domain-modeling`.
