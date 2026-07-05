# Test Support Reference

> API reference for `test_support::fixtures` and `TestAppBuilder`.

## TestAppBuilder

The `TestAppBuilder` in `src/test_support/test_app_builder.rs` provides a fluent builder pattern for constructing test fixtures. It handles:

- **World/Map/Persona Seeding**: Automatically seeds test world, map, and player persona into storage
- **Game Creation**: Creates an initial game and sets it as active
- **NPC Setup**: Optional NPC seeding and room assignment
- **State Mutation**: Optional log entries, generation status/phase, trigger context
- **Router or AppState**: Call `.build()` for `Router` or `.build_app_state()` for `AppState` directly

**Usage pattern:**

```rust
// For HTTP tests (returns Router)
let app = TestAppBuilder::default_test()
    .is_generating(true)
    .build();

// For unit tests needing AppState directly
let state = TestAppBuilder::default_test().build_app_state();
```

**Avoid duplication**: Never manually recreate the world-seeding + game-creation boilerplate in test files. Always use `TestAppBuilder` unless you have a specific reason to do custom setup.

## Test Fixtures

The `test_support::fixtures` module provides reusable test data builders. Prefer these over inline struct construction:

| Fixture | Methods | Use For |
|---------|---------|---------|
| `TestWorld` | `minimal()`, `with_rule(rule)` | `WorldCard` instances |
| `TestPlayer` | `standard()`, `named(name)` | `PlayerCard` instances |
| `TestNpc` | `named(id, name)`, `with_times_met_trigger(...)`, `with_room_scoped_trigger(...)` | `NpcCard` instances |
| `TestMap` | `room(id)`, `room_named(id, name)`, `single_room(id)`, `two_rooms(a, b)` | `Room` and `MapDef` instances |
| `TestGameState` | `in_room(id)`, `with_npc(...)`, `with_npcs(...)` | `GameState` instances |
| `TestStoredTriggerContext` | `standard()`, `for_npc(...)`, `named(...)`, `with_max_tokens(...)` | `StoredTriggerContext` instances |
| `TestPromptPreset` | `system(id, name)`, `system_default(id, name)` | `PromptPreset` instances |
| `TestWorldManifest` | `minimal()` | `WorldManifest` instances |
| `TestCharacterSheet` | `hero()` | `CharacterSheet` instances |
| `seed_default_game_row(pool, id)` | — | Insert a placeholder `games` row with the given id (FK target for `game_state_snapshots`/`messages` in sqlite-backed tests). Use after `Storage::new_sqlite(pool, n)` instead of relying on a seeded default game. |

### Example

```rust
use crate::test_support::{
    TestCharacterSheet, TestMap, TestNpc, TestPlayer,
    TestPromptPreset, TestStoredTriggerContext, TestWorld, TestWorldManifest,
};

let preset = TestPromptPreset::system("my_preset", "My Preset");
let trigger = TestStoredTriggerContext::standard();
let manifest = TestWorldManifest::minimal();
let world = TestWorld::minimal(); // Preferred for runtime-path (DB-backed) tests
```

## Integration Test Helpers

The `tests/helpers/fixtures.rs` module (exposed to integration tests via `tests/integration/mod.rs`) provides shared helpers that previously were duplicated per-file:

| Helper | Purpose |
|--------|---------|
| `create_test_world()`, `create_test_world_with_scenario()` | Canonical `WorldCard` builders (scenario variant has `StartingScenario`) |
| `create_test_player()`, `create_test_map()`, `create_test_npcs()` | Canonical character/map builders |
| `create_test_state()`, `create_basic_test_state()`, `create_basic_test_state_no_scenario()` | Canonical `GameState` builders |
| `seed_test_world(storage)` | Seeds a `TestWorld::minimal()` + `TestPlayer::standard()` into storage |
| `make_test_ctx(storage, state)` | Builds a `GameServiceContext` from storage + state |
| `create_test_storage(id)`, `create_test_storage_arc(id)` | Builds a sqlite-backed `Storage` with the `games` row pre-seeded (FK-safe) |

Prefer these over re-defining local copies in each integration test file. The `create_test_storage(id)` helper delegates to `test_support::seed_default_game_row` so sqlite-backed tests satisfy `game_state_snapshots.game_id` / `messages.game_id` FK constraints without relying on a migration-seeded default game.