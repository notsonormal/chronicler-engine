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

Schema-only migration: add `persona_key`/`persona_name` columns to `games`, drop `player_key` from `worlds`. No data backfill — fresh DBs start clean, existing DBs get empty persona fields on game rows resolved on next boot. The v9 default-game `INSERT` (placeholder row with `persona_key='player'` pointing at a nonexistent persona file) is removed; fresh DBs rely on `resolve_game_id` auto-create.

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

- Clean world-vs-game boundary. Worlds are templates; games are concrete playthroughs with per-game persona choice. Each surface has one job.
- Per-game persona flexibility. Two games on the same world can use different personas without editing the world row.
- Simpler world form. The Worlds-tab form loses a control it was never the right home for; the Games-tab form gains the control it should have had.
- Symmetric seeding. `worlds/` scans `world.json`; `personas/` scans `<key>.json`. One pattern, not two.
- Consistent UI gating. Empty-personas gate mirrors the existing empty-worlds gate pattern.

### Negative

- Migration is schema-only. Existing `games` rows get empty `persona_key`/`persona_name` fields; a new game is auto-created on next boot with the CLI persona. Existing in-progress games on upgrading DBs lose their persona association (empty string) — acceptable because the `games` table holds DB-only local play state, and `build.py --cleanup` is the supported reset path.
- Persona is now required at game creation. Users cannot create a game without picking a persona — but they cannot play one without a persona either, so this surfaces a real precondition at the right moment (creation) rather than a broken moment (first play).

### Trade-offs

- `data/worlds/redmist_estate/world.json` and `data/worlds/test/world.json` lose their `player_file` line. The persona files themselves (`data/personas/julian.json`, `data/personas/test_player.json`) are untouched.
- `data/schemas/world.schema.json` updated to drop `player_file` — schema stays as truth for what the engine reads.

## Related

- ADR-025: Multi-World Data Foundation (established `world_key` on `games` and the denormalization pattern this ADR extends)
- ADR-024: Migrate Game Data to SQLite with Seed Pattern (established the seeding pattern this ADR extends)
