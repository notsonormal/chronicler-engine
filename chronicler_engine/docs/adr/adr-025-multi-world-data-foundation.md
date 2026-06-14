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

### Implementation Pattern

```rust
// AppState construction — NO world singletons
pub struct AppState {
    pub storage: Arc<Storage>,
    pub game_service: Arc<GameService>,
    // ... other fields
}

// Handler pattern — use appropriate method based on criticality
pub async fn action_handler(state: &AppState) -> Result<_, _> {
    let ctx = state.as_game_service_context()?; // Critical — error if missing
    // ... use ctx.world, ctx.map, etc.
}

pub async fn render_fragment(state: &AppState) -> Html<String> {
    let ctx = state.as_game_service_context_or_default(); // Non-critical — degrade gracefully
    // ... render with empty defaults if world not found
}
```

## Consequences

### Positive
- ✅ **Multi-world support**: Games from different worlds can coexist in same server instance
- ✅ **Cross-world switching**: Players can switch between games from different worlds
- ✅ **Memory efficiency**: World data loaded per-request, not held in singleton
- ✅ **Flexible bootstrap**: Server starts even if requested world missing
- ✅ **Testability**: Tests can seed different worlds per context

### Negative
- ⚠️ **Performance**: Additional DB queries per-request (3-4: get_game → get_world → get_persona → list_characters)
  - Mitigation: SQLite is fast for single-user local use; caching layer can be added later
- ⚠️ **Complexity**: Handlers must handle `Result` from `as_game_service_context()`
  - Mitigation: `as_game_service_context_or_default()` for non-critical paths

### Neutral
- **Migration required**: Existing databases need v12 migration (automatic on first startup)
- **Test updates**: Unit tests need to seed worlds into storage (already done)

## Architecture Impact

### Modified Modules

| Module | Change |
|--------|--------|
| `src/storage/db.rs` | Migration v12 block |
| `src/model/game.rs` | Add `world_key` field |
| `src/storage/models/game.rs` | Add `world_key` to DbGame |
| `src/storage/backend/games.rs` | Update all CRUD to handle `world_key` |
| `src/storage/backend/worlds.rs` | Add `create_world`, `update_world`, `get_world_by_id` |
| `src/server/app_state.rs` | Remove world singletons, change `as_game_service_context()` |
| `src/application/game_lifecycle.rs` | Remove cross-world validation, update `create_game()` |
| `src/bootstrap/run.rs` | Add fallback for missing world, update INSERT |
| All `src/server/fragments/*.rs` | Update callsites to handle Result or use `…_or_default()` |

### Files Changed

- **Core implementation**: ~36 files modified
- **Tests**: 50+ test functions updated to seed worlds and handle Result types
- **Documentation**: Architecture, system docs, CHANGELOG updated

### Verification

- **Build**: ✅ 1180 tests pass, clippy clean
- **Coverage**: ✅ 85.9% (meets 80% threshold)
- **Migration tested**: ✅ Migration v12 applies successfully with backfill
- **Cross-world switch tested**: ✅ Validation removed, switching works across worlds
- **Fallback tested**: ✅ Server starts with first world if `--world` not found

## Related

- ADR-024: Migrate Game Data to SQLite with Seed Pattern (prerequisite)
- Plan: `multi-world-data-foundation.md` (archived in `docs/plans/archived/`)
