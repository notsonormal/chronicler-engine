# Task: Options data layer — PresetType::Options + two preset seeds + always-on toggle fields + migration

Type: task
Status: pending
Blocked by: 05, 14

## Question

Implement the data-layer half of options-autogeneration as decided in ticket 04. This is implementation per the map's Notes override (decide, then implement, then build-green).

Blocked by ticket 05 because both touch the same storage migration and the world/game field pattern — serialize the migrations (05 shipped v19; ticket 14 shipped v20; ticket 06 shipped v23; this ticket ships the next free version) and reuse the posture-relocation field pattern rather than racing it.

### Scope

1. **`PresetType::Options`.** Add the variant to the preset type enum (`src/domain/model/prompt_preset.rs`, mirroring `PresetType::Impersonate`), wire it through preset storage ops and the seeding loader (`src/bootstrap/run.rs` `process_preset_file` already walks `data/prompt_presets/<type>/`; confirm the new `options/` subdir is picked up).

2. **Two seed presets** under `data/prompt_presets/options/`:
   - `default.json` — id `options_default`, `is_default: true`, the DEFAULT: CYOA-style prompt asking for `{{option_count}}` brief distinct single-sentence suggestions for the player's next action, each wrapped in `<suggestion>` tags, output nothing else (mirror ticket 03's verbatim CYOA default prompt, adapted to Chronicler macros).
   - `roadway_domains.json` — id `options_roadway_domains`, `is_default: false`: Roadway-style prompt asking for a numbered list of exactly `{{option_count}}` possible player actions with an explicit domain-diversity instruction (Observation/Dialogue/Stealth/Combat/Movement/Knowledge/…), output only the list (mirror ticket 03's verbatim Roadway default prompt).
   - Both carry a `{{option_count}}` placeholder; the default count is 3 (substitution lands in ticket 10's prompt assembly).

3. **Always-on toggle fields (ticket 04 decision 2).** Add `options_always_on: bool` to `WorldCard` and `WorldManifest` (`src/domain/model/world.rs`, author default, serde default `false`) with a default-fn-pointer in `src/domain/model/utils/world_defaults.rs`; add `options_always_on: bool` to the `Game` row (`src/domain/model/game.rs`, per-game override). At game creation the game inherits from its world — same pattern as the posture triple in ticket 05.

4. **Active options preset id.** Options prompts are mode-AGNOSTIC (ticket 04: options serve both modes), so a single `active_options_prompt_preset_id` on `AppSettings` (default `options_default`), with the game inheriting it at creation per the ticket-01 decision-7 pattern. It does NOT join the per-mode registry (options are mode-agnostic; the registry is a mode-tagged list per ticket 13).

5. **Storage migration (next free version).** Add `options_always_on` to `worlds` and `games` (backfill `false`), add the options preset-id to `settings` (backfill `options_default`). Bump `user_version` to the next free number — v21–v23 have since shipped to other tickets; re-check `plumbing.rs` at execution time. Follow the `if version < N { ... }` pattern in `src/adapters/driven/storage/utils/plumbing.rs`.

## Notes for the session

- Read before implementing: ticket 04's Answer (the design), `research/03-option-generation-prior-art.md` §1b (the verbatim CYOA and Roadway prompts to adapt), `src/domain/model/prompt_preset.rs`, `src/bootstrap/run.rs` (`process_preset_file`), and everything ticket 05's read-list names (same files, same patterns).
- The seeds' prompt CONTENT is decided (ticket 04 decision 6). Adapt macros, do not redesign; if the content needs changes, reopen ticket 04.
- Build-green check: `python build.py` (fast suite). Nothing consumes the new fields yet — novel mode must stay green.
- Skills: `/domain-modeling` (field and preset-id names are domain vocabulary).
