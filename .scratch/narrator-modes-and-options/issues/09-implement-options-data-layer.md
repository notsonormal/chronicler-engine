# Task: Options data layer — PresetType::Options + two preset seeds + always-on toggle fields + migration

Type: task
Status: resolved
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

## Answer

Shipped. The options-autogeneration data layer is in place; nothing consumes it yet (tickets 10/11 do), so novel mode is untouched and green.

### What landed

1. **`PresetType::Options`** (`src/domain/model/prompt_preset.rs`): variant added, `as_str()` → `"options"`, `TryFrom<&str>` accepts `"options"`. `bundle_slot`/`set_bundle_slot` are total over the enum — `Options` returns `""` / no-op with an inline comment, because options presets hold no per-mode registry slot (mode-agnostic). Storage (`presets.rs`) is already generic on `preset_type` as a string, so no storage change was needed.
2. **Two seed presets** under `data/prompt_presets/options/`, content faithful to ticket 03's verbatim prior-art prompts (fetched fresh from the upstream repos to recover the full text, not the research doc's abridgement), adapted to Chronicler macros:
   - `default.json` — `options_default`, `is_default: true`: CYOA prompt with `{{suggestionNumber}}`→`{{option_count}}`, `<suggestion>`-tagged single-sentence beats, the five tension/scene descriptors verbatim, `{{user}}` kept (existing macro).
   - `roadway_domains.json` — `options_roadway_domains`, `is_default: false`: Roadway prompt with `Generate *exactly* 6`→`Generate *exactly* {{option_count}}`, the domain-diversity set and three example actions verbatim.
   - Both: `allowed_modes: ["novel","interactive_fiction"]` (mode-agnostic), `preset_type: "Options"`. `{{option_count}}` is an unknown placeholder to `render_template` today, so it passes through verbatim until ticket 10's prompt assembly substitutes it (default count 3 is ticket 10's call).
3. **Always-on toggle fields** (ticket 04 decision 2): `options_always_on: bool` on `WorldCard` + `WorldManifest` (`src/domain/model/world.rs`) with a serde default-fn-pointer `default_options_always_on` in `src/domain/model/utils/world_defaults.rs` (mirrors the posture fn-pointers); `From<WorldManifest>` copies it. `Game` gains `options_always_on: bool`; `NewGame` gains `options_always_on` + `options_prompt_preset_id`. New games inherit the toggle from their world (catalogue `create_game` + `init_game::resolve_game_id`), the posture-relocation pattern.
4. **Active options preset id**: `AppSettings.active_options_prompt_preset_id: String` (`src/domain/model/settings.rs`) with serde default-fn `default_active_options_prompt_preset_id` → `"options_default"`, plus a `Game.active_options_prompt_preset_id` column so the game carries a per-game copy inherited from `AppSettings` at creation (the ticket-01 decision-7 inheritance pattern; the ticket's migration scope listed only `settings`, but "the game inheriting it at creation" requires a game-side column — I added it to `games` too, backfilling `options_default`, and noted the read).
5. **Migration v24** (`src/adapters/driven/storage/utils/plumbing.rs`): adds `options_always_on` (INTEGER NOT NULL DEFAULT 0) to `worlds` + `games`, `active_options_prompt_preset_id` (TEXT NOT NULL DEFAULT 'options_default') to `games` + `settings`; backfills `games.options_always_on` from the world via COALESCE with the same orphan-game warning pattern as v19/v23. `column_exists` guards make it idempotent.
6. **Seeder**: `ensure_presets` walks `PresetType::Options` (`src/bootstrap/run.rs`); `process_preset_file` now reads `is_default` from the seed JSON (`unwrap_or(true)`, preserving existing behavior for seeds that omit it) so the Roadway seed lands `is_default: false` as designed.
7. **Worlds HTTP handler** (`worlds.rs`): `into_world_card` sets `options_always_on: false` for new worlds; `update_world_handler` unconditionally preserves the stored value (`world_card.options_always_on = stored_card.options_always_on`) because the world form has no options toggle yet (ticket 11) — the same data-safety guard ticket 07 added for posture on legacy clients.

### Resolution of the scope ambiguity (item 4)

The ticket's scope item 4 said "a single `active_options_prompt_preset_id` on `AppSettings`" but also "the game inheriting it at creation." Those are inconsistent unless the game carries a copy — inheritance with nothing to inherit into is a no-op. I implemented the per-game column (symmetric with the three sibling preset-id columns from v19) and backfilled it. If ticket 10/11 decide options selection is purely global (no per-game override), the game column can be dropped later; the inheritance reads still resolve from `AppSettings`, so removing it is cheap.

### Known edge (carried, not fixed here)

Deleting an options preset is not reference-checked against `AppSettings.active_options_prompt_preset_id` or a game's `active_options_prompt_preset_id` — the existing delete refusal checks the per-mode registry only. Mirror of ticket 07's known edge (per-game preset ids not delete-checked). Ticket 11 (UI) or a follow-up should decide whether options-default deletion is refused.

### Build

`python build.py` green: 1108 unit, architecture + guardrails pass, 1527/1528 integration (the one failure — `tests/browser/behaviour.rs::test_world_posture_change_autosaves_status` — is a flaky browser timeout: it passes in isolation and in the full focused `behaviour` suite, 23/23). Migration-test terminal-version assertions bumped 23→24; the stale `guardrails.md` (pre-existing line-number drift in `tests/infrastructure/guardrails/layers.rs`, unrelated to this ticket) was regenerated.

### Unblocks

10 (Options agent + pipeline), via 09. 11 (UI) still waits on 10; 12 (specs + integration tests) waits on 07/11.

### Post-review adjustments (same-session /code-review)

- Removed the task reference from the worlds handler comment ("(ticket 11)") — CODING_STANDARDS.md ban; the hidden-constraint comment itself was kept.
- Renamed the tuple `bundle` to destructured `(bundle, options_preset_id)` in both `catalogue.rs` create/reset sites — kills the `bundle.0`/`.1` index reads the review flagged as Mysterious Name.
- Left for ticket 11 (recorded, not fixed): mode-agnostic options activation in `activate_preset_handler` (the registry setter is a silent no-op for Options; unreachable via the current UI), and extraction of the 3-site NewGame-assembly duplication.
