# Test Support Reference

Builders for `test_support::fixtures` and `TestAppBuilder`. Prefer these over inline struct construction.

## TestAppBuilder

Fluent builder for test fixtures (`src/test_support/test_app_builder.rs`). Seeds world/map/persona, creates the initial game, optionally adds NPCs, log entries, generation status/phase, and trigger context. `.build()` returns a `Router` for HTTP tests; `.build_app_state()` returns `AppState` directly.

```rust
let app = TestAppBuilder::default_test().is_generating(true).build();
let state = TestAppBuilder::default_test().build_app_state();
```

## Fixtures

| Fixture | Methods | Use For |
|---------|---------|---------|
| `TestWorld` | `minimal()`, `with_rule(rule)` | `WorldCard` |
| `TestPlayer` | `standard()`, `named(name)` | `PlayerCard` |
| `TestNpc` | `named(id, name)`, `with_times_met_trigger(...)`, `with_room_scoped_trigger(...)` | `NpcCard` |
| `TestMap` | `room(id)`, `room_named(id, name)`, `single_room(id)`, `two_rooms(a, b)` | `Room` / `MapDef` |
| `TestGameState` | `in_room(id)`, `with_npc(...)`, `with_npcs(...)` | `GameState` |
| `TestStoredTriggerContext` | `standard()`, `for_npc(...)`, `named(...)`, `with_max_tokens(...)` | `StoredTriggerContext` |
| `TestPromptPreset` | `system(id, name)`, `system_default(id, name)` | `PromptPreset` |
| `TestWorldManifest` | `minimal()` | `WorldManifest` |
| `TestCharacterSheet` | `hero()` | `CharacterSheet` |
| `seed_default_game_row(pool, id)` | — | Pre-seed FK target (`games` row) in sqlite-backed tests |

### Example

```rust
use chronicler_engine::test_support::{
    TestCharacterSheet, TestMap, TestNpc, TestPlayer,
    TestPromptPreset, TestStoredTriggerContext, TestWorld, TestWorldManifest,
};

let preset = TestPromptPreset::system("my_preset", "My Preset");
let trigger = TestStoredTriggerContext::standard();
let manifest = TestWorldManifest::minimal();
let world = TestWorld::minimal();
```

## Recording Forensics

`LlmMessageRepository` spy impls in `src/test_support/`:

| Spy | Purpose |
|-----|---------|
| `NoopForensics` | Discards all messages; use when test does not assert on LLM call log |
| `RecordingForensics` | Records every `save_llm_message` call; exposes counters + last message + injectable error for failure-injection tests |

Both satisfy the `LlmMessageRepository` port; recorder wraps a provider + a spy via `make_test_recorder(provider)` / `make_test_recorder_with_storage(provider, storage)`.

## Integration Test Helpers

`tests/helpers/fixtures.rs` exposes shared builders to integration tests via `tests/integration/mod.rs`. Prefer these over re-defining per file:

| Helper | Purpose |
|--------|---------|
| `create_test_world()`, `create_test_world_with_scenario()` | Canonical `WorldCard` (scenario variant carries `StartingScenario`) |
| `create_test_player()`, `create_test_map()`, `create_test_npcs()` | Canonical character / map builders |
| `create_test_state()`, `create_basic_test_state()`, `create_basic_test_state_no_scenario()` | Canonical `GameState` builders |
| `seed_test_world(storage)` | Seeds `TestWorld::minimal()` + `TestPlayer::standard()` |
| `make_test_ctx(storage, state)` | Builds `GameServiceContext` |
| `create_test_storage(id)`, `create_test_storage_arc(id)` | Sqlite-backed `Storage` with `games` row pre-seeded (FK-safe) |

`create_test_storage(id)` delegates to `test_support::seed_default_game_row` so sqlite-backed tests satisfy the `game_state_snapshots.game_id` / `messages.game_id` FK constraints without relying on a migration-seeded default game.