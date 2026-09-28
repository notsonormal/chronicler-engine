# Task: Posture relocation — data layer (domain model + storage migration + per-mode preset registry)

Type: task
Status: resolved

## Session notes

Claimed 2026-08-27. Breakdown for this session (ticket is 8+ points): (1) domain NarratorMode + per-mode preset registry on AppSettings; (2) posture fields on WorldCard/WorldManifest/Game; (3) migration v19 + DB models + storage ops; (4) consumer rewiring for build-green; (5) tests + build.py; (6) resolution recording.
Blocked by: (none)

## Question

Implement the data-layer half of the posture relocation decided in ticket 01. This is implementation per the map's Notes override (decide, then implement, then build-green).

### Scope

1. **`NarratorMode` enum.** Add to `src/domain/model/settings.rs` alongside `NarrativePerspective`/`NarrativeTense`: variants `Novel` / `InteractiveFiction`, `#[serde(rename_all = "snake_case")]`, `as_str()`, `parse_or_default()` (→`Novel` on bad data), `FromStr`. Add `default_narrator_mode()` to `src/domain/model/utils/settings_defaults.rs`.

2. **World posture fields.** Add `narrator_mode`, `narrative_perspective`, `narrative_tense` to `WorldCard` and `WorldManifest` (`src/domain/model/world.rs`), with serde defaults (Novel/third/past). Add default-fn-pointers to `src/domain/model/utils/world_defaults.rs`.

3. **Game posture fields + preset-ids.** Add the posture triple AND the three `active_*_prompt_preset_id` (system/quantifier/impersonate) to the `Game` row (`src/domain/model/game.rs`) — posture is config, not mutable state, so it lives on `Game` not `GameState`. At creation, a game inherits posture from its world and preset-ids from the global per-mode registry (ticket 01 decision 7).

4. **Global per-mode preset registry.** Replace the three single `active_*_prompt_preset_id` on `AppSettings` with TWO mode-keyed bundles (2×3 ids): Novel → (`system_default`, `quantifier_default`, `impersonate_default`); InteractiveFiction → (`system_if_default`, `quantifier_default`, `impersonate_default`) [IF quantifier/impersonate initially same as Novel per ticket 01 decision 8; ticket 02 may diverge them]. Provide a lookup: given a `NarratorMode`, return the three default preset-ids. `AppSettings` LOSES `narrative_perspective` and `narrative_tense` (moved to world/game).

5. **Storage migration v19.** Add posture columns to `worlds` and `games` tables; backfill existing worlds with defaults (novel/third/past); backfill existing games with their world's posture (or current global settings values — decide during implementation, record the choice). Add the per-mode registry to `settings` (columns or a json blob — decide during implementation). Deprecate/drop the `narrative_perspective`/`narrative_tense` columns from `settings` (added in v18). Bump `user_version` to 19. Follow the `if version < N { ... }` pattern in `src/adapters/driven/storage/utils/plumbing.rs`.

## Notes for the session

- Read before implementing: `src/domain/model/world.rs` (WorldCard/WorldManifest), `src/domain/model/game.rs` (Game row), `src/domain/model/settings.rs:28-120` (the enum pattern to mirror), `src/domain/model/utils/settings_defaults.rs` + `world_defaults.rs` (default-fn pattern), `src/adapters/driven/storage/utils/plumbing.rs` (migration pattern; v18 is current), `src/adapters/driven/storage/models/` (DB model round-trip), `src/adapters/driven/storage/settings.rs` + `games.rs` + `worlds.rs` (storage ops).
- The `NarratorMode` enum lives in `settings.rs` (where the other posture enums live) even though it is used on world/game, not `AppSettings`.
- This ticket is large (likely 8+ story points). If it exceeds one session, break it into subtasks per AGENTS.md and record the breakdown in a comment.
- Build-green check: `python build.py` (fast suite). IF mode end-to-end is NOT green yet (needs ticket 02's IF preset); the mechanics must build and existing novel mode must stay green.
- Skills: `/domain-modeling` (the enum + field names are domain vocabulary).

## Answer

Implemented the data-layer half of posture relocation. `python build.py` is green (1453 passed, 2 LLM skipped, 0 failed).

### What landed

- **`NarratorMode` enum** (`src/domain/model/settings.rs`): variants `Novel`/`InteractiveFiction`, `#[serde(rename_all = "snake_case")]` (`"novel"`/`"interactive_fiction"`), `as_str()`, `parse_or_default()` (→Novel), `FromStr`, `Default`→Novel. `default_narrator_mode()` in `settings_defaults.rs`.
- **World posture fields**: `narrator_mode`, `narrative_perspective`, `narrative_tense` on `WorldCard` and `WorldManifest` (`world.rs`), serde defaults from `world_defaults.rs`, pass-through in `From<WorldManifest>`.
- **Game posture + preset-ids** (`game.rs`): `Game` gains the posture triple and `active_system/quantifier/impersonate_prompt_preset_id` (config, not mutable state). New `NewGame` request struct carries inherited posture + the registry bundle into `Storage::create_game_with_posture`.
- **Per-mode preset registry**: `AppSettings` LOSES `narrative_perspective`, `narrative_tense`, and the three `active_*_prompt_preset_id`. It gains `mode_preset_registry: ModePresetRegistry` (`ModePresetBundle` ×2 with `bundle_for(mode)`). Defaults: Novel→(`system_default`, `quantifier_default`, `impersonate_default`); IF→(`system_if_default`, shared quantifier/impersonate). `settings_defaults.rs` holds the bundle/registry default-fns.
- **Migration v19** (`plumbing.rs`): worlds +3 posture cols (defaults novel/third/past); games +6 cols (posture backfilled from the game's world via `UPDATE ... SELECT`); settings gains `mode_preset_registry` JSON (backfilled from the legacy per-type columns into the Novel bundle, then IF quantifier/impersonate mirrored from Novel), then the five legacy settings columns are dropped. `user_version`→19.

### Decisions made during implementation (recorded per ticket)

- **Games backfill posture from their world**, not the global settings. Worlds are backfilled with novel/third/past defaults per the ticket. A user who had set a non-default global perspective/tense in v18 loses that on existing games (acceptable — pre-release, single user; worlds get defaults per scope). This keeps inheritance semantics consistent (one source of truth: the world).
- **Registry stored as a JSON blob column** (`mode_preset_registry TEXT`), mirroring the `text_check`/`agents` JSON-column pattern, rather than six flat columns. Keeps the 2×3 bundle a single serde structure with centralized defaults.
- **Two `create_game` paths**: `create_game` (5-arg, DB defaults) kept for test convenience; `create_game_with_posture(&NewGame)` is the production path the `GameCatalogue` uses to inherit world posture + the registry bundle. A `NewGame` struct avoids clippy's too-many-arguments lint.
- **Per-game preset-ids are not backfilled in migration v19** (Scope 5 decision, recorded). The migration migrates the legacy global `active_*` settings columns into `mode_preset_registry` (preserving the user's prior selections in the Novel bundle), then drops them — but the new per-game `active_*` columns are added with hardcoded DB defaults (`system_default`/`quantifier_default`/`impersonate_default`) and receive no `UPDATE`. Rationale: posture has a per-world source to backfill from; preset-ids do not (the world ships no rule-set selection), and the project is pre-release with a single user who disposes of old DBs. Existing games therefore reset to the Novel default bundle rather than inheriting the migrated registry values. Consistent with the posture-backfill decision's pre-release/single-user rationale.

### Consumer rewiring (build-green)

Preset/posture reads moved off `AppSettings`:
- Pipeline (`pipeline_run.rs`), impersonate action (`action.rs`), quantifier agent (`agent.rs`), game view (`view_query.rs`), and arrival spawn (`init_game.rs`) read preset-ids from the current `Game` row, falling back to the registry's Novel bundle if the game is absent.
- Assembler (`assembler.rs`) now reads posture from `context.world` (which carries posture); arrival service reads posture from the current game, falling back to the world. The `AppSettings`-based voice override is gone.
- `GameCatalogue` gained `settings: Arc<RwLock<AppSettings>>` (wired in `wiring.rs`) to look up the registry bundle for the world's mode at creation.

### UI changes (global posture UI removed; world/game UI is ticket 07)

The global settings panel's "Narrative Voice" section, the `POST /settings/narrative-voice` route, `save_narrative_voice_handler`, and `NarrativeVoiceForm` are removed — posture is no longer a global setting. The prompt-presets panel's "active" markers now read/write the Novel bundle of the registry (the IF bundle is edited via the world/game UI in ticket 07). `docs/specs/settings.md` scenarios 20.8–20.10 dropped, 20.1 trimmed; `http_routes.md` regenerated (52 routes).

### Forward notes for downstream tickets

- Ticket 06 (narration pipeline): the pipeline already reads game posture/presets now; 06 should ensure per-game mode-switch posture (the nudge) routes into `PromptContext` rather than reading the world's posture, and that the assembler uses game posture not world posture.
- Ticket 07 (UI): add world-editor posture dropdowns + in-game override + per-game mode-switch; expose the IF bundle in the preset panel.
- `NarratorMode`/`NarrativePerspective`/`NarrativeTense` now derive `Default`.
