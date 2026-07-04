# ADR-025: Multi-World Data Foundation

**Date:** 2026-06-14  
**Status:** Accepted  
**Drivers:** Multi-world game support, cross-world game switching

## Problem Statement

The Chronicler Engine bakes a single world's data into `AppState` at startup based on the `--world` CLI argument. This creates hard limitations:

1. **Single-world server**: All games must belong to one world instance
2. **Cross-world switching blocked**: `switch_game()` rejects games from different worlds
3. **Wasteful memory**: World/map/persona/NPC data duplicated across all game contexts
4. **UI inflexibility**: No ability to manage or switch between multiple worlds at runtime

## Decision

Refactor the data layer to load world context from the database per-request based on the active game's `world_key`.

### Key Changes

#### 1. Migration v12: Schema Change
- Add `world_key TEXT NOT NULL DEFAULT ''` to `games` table
- Backfill: map `world_name` to `worlds.key`, fallback to `redmist_estate` for unmatched
- No FK constraint — `world_key` is a logical reference (same as `world_name`)

#### 2. Per-Request World Loading
- Remove `Arc<WorldCard>`, `Arc<MapDef>`, `Arc<PlayerCard>`, `Arc<HashMap<String, NpcCard>>` from `AppState`
- `as_game_service_context()` now:
  - Loads active game from DB via `storage.current_game_id()`
  - Fetches world data: `storage.get_world(game.world_key)`
  - Fetches persona: `storage.get_persona(world.player_key)`
  - Fetches NPCs: `storage.list_characters(world_id)`
  - Returns populated `GameServiceContext`

#### 3. Fallback Strategy
- `as_game_service_context_or_default()` for non-critical UI rendering
- Returns empty defaults (`WorldCard::default()`, empty maps, etc.) when world not found
- Prevents UI crashes on missing data

#### 4. Cross-World Switching Enabled
- Remove `if game.world_name != ctx.world.name` validation from `switch_game()`
- World context loads fresh from DB for the target game
- Games from different worlds can now coexist and switch freely

#### 5. Bootstrap Fallback
- If `--world` world not found, fall back to first available world in DB
- Log warning but don't crash server
- Ensures server can start even if world data is corrupted

## Consequences

### Positive
- Multi-world support: Games from different worlds can coexist in same server instance
- Cross-world switching: Players can switch between games from different worlds
- Memory efficiency: World data loaded per-request, not held in singleton
- Flexible bootstrap: Server starts even if requested world missing
- Testability: Tests can seed different worlds per context

### Negative
- Performance: Additional DB queries per-request (3-4: get_game → get_world → get_persona → list_characters)
  - Mitigation: SQLite is fast for single-user local use; caching layer can be added later
- Complexity: Handlers must handle `Result` from `as_game_service_context()`
  - Mitigation: `as_game_service_context_or_default()` for non-critical paths

### Trade-offs
- **Migration required**: Existing databases need v12 migration (automatic on first startup)
- **Test updates**: Unit tests needed reseeding worlds into storage

## Related

- ADR-024: Migrate Game Data to SQLite with Seed Pattern (prerequisite)
