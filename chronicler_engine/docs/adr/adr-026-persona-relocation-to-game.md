# ADR-026: Relocate Persona Binding from World to Game

**Date:** 2026-06-23
**Status:** Accepted
**Drivers:** Clarify world-vs-game boundary; enable per-game persona choice on a shared world

## Problem Statement

Prior to this decision, persona binding lived on the world: `WorldCard.player_key` resolved the active persona at game-start (`app_state.rs:54-58`), and the on-disk `world.json` declared a `player_file` that both (a) told the seeder which persona to seed and (b) was the runtime binding source. These are two different concerns collapsed into one field.

Three problems flowed from this:

1. **Fuzzy boundary.** The world appeared to "own" a persona. But the world is a template (map, rules, scenarios); the persona is a concrete playthrough choice. Two games on the same world could not use different personas without editing the world row.
2. **Overloaded field.** `WorldManifest.player_file` did double duty: it was a seeding pointer (which file to load) *and* a runtime foreign key (which persona this world plays as). Removing either concern required touching both.
3. **UI surface mismatch.** The persona selector lived on the Worlds-tab form, implying "persona is a world property." It isn't — it's a per-game choice made at game-creation time.

## Decision

Move the persona binding from the world to the game. Personas become global, world-independent entities; each game carries its own `persona_key`.

### Key Changes

#### 1. Manifest: drop `player_file`

`WorldManifest.player_file` is removed. On-disk `world.json` files no longer declare a persona file. The world no longer references a persona at all.

#### 2. Seeding: scan `data/personas/` directly

`bootstrap/load.rs::seed_game_data` now scans `data/personas/*.json` directly and seeds every file it finds, independent of any world manifest. Personas are a top-level, world-independent directory — symmetric with `worlds/`.

#### 3. Game: add `persona_key` and `persona_name`

`games` table and `Game` struct get two new fields, both denormalized at creation time (mirroring the `world_name` pattern from ADR-025):

- `persona_key TEXT NOT NULL DEFAULT ''` — stable foreign key into `personas.key`
- `persona_name TEXT NOT NULL DEFAULT ''` — denormalized display name

The chosen persona is immutable for the life of the game row. Editing a game's persona is out of scope — a different game is created instead.

#### 4. Bootstrap migration v13

- `ALTER TABLE games ADD COLUMN persona_key TEXT NOT NULL DEFAULT ''`
- `ALTER TABLE games ADD COLUMN persona_name TEXT NOT NULL DEFAULT ''`
- `DROP COLUMN player_key` from `worlds` (or table rebuild on SQLite < 3.35)
- Schema-only — no data backfill. Fresh DBs start clean; existing DBs get empty persona fields on game rows, resolved on next boot via the `--persona` CLI flag (see Section 8). The v9 default-game `INSERT` (which seeded a placeholder row with `persona_key='player'` pointing at a nonexistent persona file) is also removed; fresh DBs rely on `resolve_game_id` auto-create (Section 8).

#### 5. Start-up resolver: `game.persona_key` → persona

`app_state.rs` replaces `world.player_key → get_persona()` with `game.persona_key → get_persona()`. Resolution path:

```
Game (game.persona_key) → storage.get_persona() → PlayerCard
```

#### 6. UI changes

- **Games tab** (New Game form) gains `<select name="persona_key" required>`. Empty `personas` ⇒ the form renders the world select but replaces the persona select with `<div class="games-empty">No personas available. Create a persona first.</div>` and disables submit. Form layout mirrors the Worlds tab (stacked `.form-row` blocks).
- **Games tab** (saved-games list + active-game panel) renders `<span class="persona-badge">{{ game.persona_name }}</span>`. `GameRowView` gains `persona_name`.
- **Worlds tab** form: the "Player Persona" `<select>` is removed entirely. The `personas: &[PlayerCard]` parameter, the `list_personas` call in worlds_fragment handlers, and the `PersonasOption`/`personas` field on `WorldFormTemplate` are all deleted.

#### 7. Submit-time validation

`create_game_handler` looks up `persona_key` via `storage.get_persona()` before insert. If `None`, returns an error response and does not create the game row. Same lookup-or-error pattern as the start-up persona resolver in `bootstrap/run.rs` (`get_persona(&args.persona)?.ok_or_else(...)`) and the runtime `context_for_world` resolver in `app_state.rs`. This prevents broken `games` rows that would hard-error on first play.

#### 8. CLI `--persona` flag

The CLI gains `--persona <key>` (default `julian`), mirroring `--world <key>` (default `redmist_estate`). When `resolve_game_id` finds no existing game for the requested world, it auto-creates one using the CLI-provided persona and a denormalized `persona_name` resolved via `storage.get_persona()`. This restores the pre-ADR-026 auto-create behavior with explicit persona selection at startup.

The Games-tab New Game form remains the primary creation path for interactive use; the CLI flag is a headless/first-boot convenience so a fresh DB boots directly into active play rather than the limbo state originally specified by Sub-decision B (which was reverted because it broke the 22 browser tests that assume auto-active-play at startup). If `--persona <key>` does not match any persona in the DB, boot hard-errors with `EngineError::Config("Persona '<key>' not found")` — no silent fallback.

### What stays unchanged

- The `personas` table schema, `DbPersona` model, `storage::list_personas` / `get_persona` / `seed_persona` — all unchanged. Personas were already global in storage; only the world was pretending otherwise.
- `data/personas/*.json` file format and contents — unchanged.
- The `WorldCard` struct's other fields (map, scenarios, global_rules, starting_room_id, default_scenario_id, default_room_image) — unchanged.

## Alternatives Considered

### Alternative A: Keep `player_file` on the manifest as pure seeding pointer

Retain `WorldManifest.player_file` for seeding only; drop only the runtime `WorldCard.player_key`. The world would still name a persona file in its manifest, but would not bind it at runtime.

**Rejected because:** this preserves a residual world→persona coupling ("this world ships with this persona") under a weaker name. It conflates the packaging concern (which personas ship with this world) with the seeding concern (which personas to load), and requires the seeder to still iterate world manifests to find persona files — defeating the goal of making personas world-independent. The world ends up "owning" personas in a weaker form, which is muddier than the clean cut.

### Alternative B: Add a "default persona" field on the world

Keep a world-level `default_persona: Option<String>` as the `<select>`'s pre-selected value on the New Game form. Runtime binding still moves to the game; the world just suggests a default.

**Rejected because:** "default" only has meaning relative to something, and the only consumer is the New Game form's `<select>`. The user explicitly rejected this concept during grilling: adding it back under a softer name resurrects the world→persona coupling with worse odds of being understood later. The form's `required` constraint (no submission without an explicit pick) replaces the default entirely.

### Alternative C: Don't denormalize `persona_name`

Look up `persona_name` via JOIN at list time in `list_games`.

**Rejected because:** ADR-025 already established the denormalization pattern for `world_name` (`game.rs:10` comment: "Display name of the world (denormalized for performance - avoids JOIN in queries"). `persona_name` should follow the same pattern for consistency. Introducing a second convention alongside an existing one is prohibited by project convention.

## Consequences

### Positive

- ✅ **Clean world-vs-game boundary.** Worlds are templates; games are concrete playthroughs with per-game persona choice. Each surface has one job.
- ✅ **Per-game persona flexibility.** Two games on the same world can use different personas without editing the world row.
- ✅ **Simpler world form.** The Worlds-tab form loses a control it was never the right home for; the Games-tab form gains the control it should have had.
- ✅ **Symmetric seeding.** `worlds/` scans `world.json`; `personas/` scans `<key>.json`. One pattern, not two.
- ✅ **Consistent UI gating.** Empty-personas gate mirrors the existing empty-worlds gate pattern.

### Negative

- ⚠️ **Migration is schema-only.** Existing `games` rows get empty `persona_key`/`persona_name` fields; a new game is auto-created on next boot with the CLI persona. The v9 default-game INSERT is removed, so fresh DBs start with zero games and rely on `resolve_game_id` auto-create. Existing in-progress games on upgrading DBs lose their persona association (empty string) — acceptable because the `games` table holds DB-only local play state, and `build.py --cleanup` is the supported reset path.
- ⚠️ **Persona is now required at game creation.** Users cannot create a game without picking a persona — but they cannot play one without a persona either, so this surfaces a real precondition at the right moment (creation) rather than a broken moment (first play).

### Neutral

- **Bootstrap seed data.** `data/worlds/redmist_estate/world.json` and `data/worlds/test/world.json` lose their `player_file` line. The persona files themselves (`data/personas/julian.json`, `data/personas/test_player.json`) are untouched.
- **`data/schemas/world.schema.json`.** Updated to drop `player_file` — schema stays as truth for what the engine reads.

## Architecture Impact

### Modified Modules

| Module | Change |
|--------|--------|
| `src/storage/db.rs` | Add migration v13 block (schema-only: add `games.persona_key` + `persona_name`, drop `worlds.player_key`; no data backfill, no `DELETE FROM games`) |
| `src/model/world.rs` | Remove `player_key` from `WorldCard`, `player_file` from `WorldManifest`, `derive_player_key` |
| `src/model/game.rs` | Add `persona_key` + `persona_name` fields to `Game` |
| `src/storage/models/world.rs` | Remove `player_key` from `DbWorld` and `from_row` |
| `src/storage/models/game.rs` | Add `persona_key` + `persona_name` to `DbGame` (row mapping stays as inline closures in `backend/games.rs`; `DbGame` does not implement `from_row`) |
| `src/storage/backend/worlds.rs` | Remove `player_key` from INSERT / UPDATE / SELECT; update `worlds_tests.rs` |
| `src/storage/backend/games.rs` | Add `persona_key` + `persona_name` to INSERT / UPDATE / SELECT; update `games_tests.rs` |
| `src/bootstrap/load.rs` | Replace manifest-driven persona seeding with direct `data/personas/*.json` scan; update `load_tests.rs` |
| `src/server/app_state.rs` | Replace `world.player_key → get_persona` with `game.persona_key → get_persona` |
| `src/server/games_fragment/handlers.rs` | `CreateGameForm` gains `persona_key`; `create_game_handler` validates via `get_persona` and denormalizes `persona_name` |
| `src/server/games_fragment/template.rs` | `GameRowView` gains `persona_name`; `GamesPanelTemplate` gains `personas: Vec<PlayerCard>` for the New Game select; render persona badge + stacked form rows + empty gate |
| `src/server/worlds_fragment/template.rs` | Remove "Player Persona" `<select>` and `personas` field from `WorldFormTemplate` |
| `src/server/worlds_fragment/handlers.rs` | Remove `list_personas` calls and `personas` parameter from `render_world_edit_form` |
| `src/server/worlds_fragment/fragments.rs` | Remove `personas` parameter |
| `data/schemas/world.schema.json` | Remove `player_file` property |
| `data/worlds/redmist_estate/world.json` | Remove `"player_file"` line |
| `data/worlds/test/world.json` | Remove `"player_file"` line |
| `src/test_support/test_app_builder.rs` | Update test setup — no `player_file` in world manifests |
| `docs/system/worlds.md`, `docs/system/game_flow.md` | Update prose to reflect new boundary |

### Verification

Implementation complete in working tree; verification plan:

- **Build**: `cargo build` clean; `cargo clippy` clean
- **Tests**: existing tests updated (world_tests, worlds_tests, load_tests, game_service_tests, games_fragment_handlers_tests). New tests for v13 migration, submit-time validation, empty-personas gating.
- **End-to-end**: `python build.py` (fmt + clippy + tests + coverage) passes; 80% coverage threshold met.
- **UI smoke**: New Game form renders persona select; submitting creates a game whose `persona_key` resolves to the chosen persona; worlds-tab form no longer has persona select. (Verification pending screenshot review)

## Related

- ADR-025: Multi-World Data Foundation (established `world_key` on `games` and the denormalization pattern this ADR extends)
- ADR-024: Migrate Game Data to SQLite with Seed Pattern (established the seeding pattern this ADR extends)
