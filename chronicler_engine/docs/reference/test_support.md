# Test Support Reference

Builders for `test_support::fixtures`, `TestDataBuilder`, `TestAppBuilder`, plus the integration-only `SqliteTestAppBuilder`. Prefer these over inline struct construction.

## TestDataBuilder (`src/test_support/test_data_builder.rs`)

Bundles world / map / persona / NPCs as a single `TestData` value. `default_test()` produces canonical defaults: world key `"test"` with scenarios populated, persona key `"test_player"`, npc id `"npc_1"`. `TestData::seed_into(&storage)` seeds world / persona / NPC character rows and returns `world_id`. `TestData` derives `Clone` to support multi-app test patterns where the same world data is needed for two `Arc<DefaultApplicationService>` instances.

Methods: `default_test()`, `world(card)`, `map(def)`, `persona(card)`, `npc(card)`, `npcs(vec)`, `room_npc(id, card)`, `build()`. `TestData` methods: `seed_into(&storage)`, `find_npc(id)`, `world_key()`, `player_name()`.

```rust
use chronicler_engine::test_support::TestDataBuilder;

let data = TestDataBuilder::default_test()
    // override only what the test cares about
    .npc(/* ... */)
    .build();
```

## TestAppBuilder (`src/test_support/test_app_builder.rs`)

Fluent in-memory builder for `Arc<DefaultApplicationService>`. Fields: `test_data: Option<TestData>`, `logs`, `last_trigger`, `generation: Option<(GenerationStatus, GenerationPhase)>` tuple, `settings`, `storage`, `game_service`, `skip_seeding`, `is_generating`. `build_service()` returns `Arc<DefaultApplicationService>`; `build_app_state()` returns the underlying `AppState` directly.

```rust
use chronicler_engine::test_support::{TestDataBuilder, TestAppBuilder};

let data = TestDataBuilder::default_test().build();
let app = TestAppBuilder::with_data(data).build_service()?;
```

**`skip_seeding(true)`** skips `test_data.seed_into`, snapshot save, and message persistence inside `build_app_state`. The transient `GameState` (with `starting_room` derivation, `room_npcs` population, `last_trigger`, `generation`, `logs`) is still constructed from `test_data` but discarded. `build_app_state` uses `default_test_preset_storage()` for the `preset_storage` field.

Methods: `default_test()`, `with_data(data)`, `data()`, `default_app()` (HTTP-test shorthand: `Self::default_test().build()`), `last_trigger(...)`, `log(...)`, `generation_status(...)`, `settings(...)`, `storage(...)`, `game_service(gs)`, `is_generating(value)`, `skip_seeding(bool)`, `build()`, `build_with_service(gs)`, `build_service()`, `from_base(base)`, `build_app_state()`.

## SqliteTestAppBuilder (`tests/helpers/sqlite_test_app_builder.rs`)

Integration-only builder (not in `src/test_support/` — lib tests cannot import from `tests/`). File carries `#![allow(dead_code)]` at module level because it is `#[path]`-included in both the `tests/integration/` and `tests/infrastructure/` binaries — methods unused from one binary's view are used from the other.

`SqliteTestAppBuilder` runs the following setup: sqlite pool, `seed_default_game_row`, `test_data.seed_into`, snapshot save → `pre_main_id`, `Input` message `snapshot_id` wiring, first-swipe `snapshot_id` wiring, message + swipe persistence, final snapshot re-save. Private `finalize_app` is duplicated locally (not re-exported from `test_support::context` to avoid making the private helper `pub`).

```rust
use chronicler_engine::test_support::TestDataBuilder;
use crate::helpers::SqliteTestAppBuilder;

let data = TestDataBuilder::default_test().build();
let app = SqliteTestAppBuilder::with_data(data)
    .game_service_fn(move |storage| {
        Arc::new(GameService::with_mock_quantifier(
            make_test_recorder_with_storage(Arc::new(MockBackend::new()), Arc::clone(storage)),
            Arc::new(MockBackend::default()),
        ))
    })
    .build_service()?;
```

Methods: `with_data(data)`, `default_test()`, `data()`, `last_trigger(...)`, `log(...)`, `message(msg)`, `messages(msgs)`, `generation_status(...)`, `settings(...)`, `is_generating(value)`, `mock_backend(F)`, `backends(F)`, `separate_backends(n, q)`, `game_service_fn(F)`, `state_mut(F)`, `build_service()`.

**`.state_mut(F: FnOnce(&mut GameState))`** escape hatch: overrides the `GameState` the builder constructs from `test_data`. Needed when a test must mutate runtime state fields not expressible via `TestData` (e.g. `state.movement.current_room_id = non_existent_room`, `state.npc_encounter_log.npcs["shopkeeper"].times_met = 0`).

**`.message(...)` / `.messages(...)`** append to `state.narrative.history` before snapshot-id wiring — preserves faithful multi-swipe test setups that the `logs`-driven path would collapse.

## Fixtures

| Fixture | Methods | Use For |
|---------|---------|---------|
| `TestWorld` | `minimal()`, `with_rule(rule)` | `WorldCard` |
| `TestPersona` | `standard()`, `named(name)` | `PersonaCard` |
| `TestNpc` | `named(id, name)`, `with_times_met_trigger(...)`, `with_room_scoped_trigger(...)` | `NpcCard` |
| `TestMap` | `room(id)`, `room_named(id, name)`, `single_room(id)`, `two_rooms(a, b)` | `Room` / `MapDef` |
| `TestGameState` | `in_room(id)` | `GameState` |
| `TestStoredTriggerContext` | `standard()`, `for_npc(...)`, `named(...)`, `with_max_tokens(...)` | `StoredTriggerContext` |
| `TestPromptPreset` | `system(id, name)`, `system_default(id, name)` | `PromptPreset` |
| `TestWorldManifest` | `minimal()` | `WorldManifest` |
| `TestCharacterSheet` | `hero()` | `CharacterSheet` |
| `TestDataBuilder` | `default_test()`, `world(...)`, `map(...)`, `persona(...)`, `npc(...)`, `npcs(...)`, `room_npc(...)`, `build()` | `TestData` (world / map / persona / NPCs bundle) |
| `seed_default_game_row(pool, id)` | — | Pre-seed FK target (`games` row) in sqlite-backed tests |

See [`## TestDataBuilder`](#testdatabuilder-src/test_support/test_data_builderrs) above for the full `TestData` API.

### Example

```rust
use chronicler_engine::test_support::{
    TestCharacterSheet, TestMap, TestNpc, TestPersona,
    TestPromptPreset, TestStoredTriggerContext, TestDataBuilder, TestWorld, TestWorldManifest,
};

let preset = TestPromptPreset::system("my_preset", "My Preset");
let trigger = TestStoredTriggerContext::standard();
let manifest = TestWorldManifest::minimal();
let world = TestWorld::minimal();
let data = TestDataBuilder::default_test().build();
```

## Recording Forensics

`LlmMessageRepository` spy impls in `src/test_support/`:

| Spy | Purpose |
|-----|---------|
| `NoopForensics` | Discards all messages; use when test does not assert on LLM call log |
| `SpyForensics` | Counts `save_llm_message` calls via atomic counter; use when a test must assert a code path routed through `LlmCallRecorder::complete` (regression guard for the quantifier forensics bypass) |
| `RecordingForensics` | Records every `save_llm_message` call; exposes counters + last message + injectable error for failure-injection tests |

All three satisfy the `LlmMessageRepository` port; recorder wraps a provider + a spy via `make_test_recorder(provider)` / `make_test_recorder_with_storage(provider, storage)` / `make_spy_recorder(provider)`.

## Integration Test Helpers

`tests/helpers/fixtures.rs` exposes shared builders to integration tests via `tests/integration/mod.rs`. Prefer these over re-defining per file:

| Helper | Purpose |
|--------|---------|
| `create_test_world()`, `create_test_world_with_scenario()` | Canonical `WorldCard` (scenario variant carries `StartingScenario`) |
| `create_test_player()`, `create_test_map()`, `create_test_npcs()` | Canonical character / map builders |
| `create_test_state()` | Canonical `GameState` builder (alias to `GameState::new("room1")`) |
| `create_test_storage(id)`, `create_test_storage_arc(id)` | Sqlite-backed `Storage` with `games` row pre-seeded (FK-safe) |

`create_test_storage(id)` delegates to `test_support::seed_default_game_row` so sqlite-backed tests satisfy the `game_state_snapshots.game_id` / `messages.game_id` FK constraints without relying on a migration-seeded default game.

## Narrow-Case Helpers (`src/test_support/context.rs`)

Three narrow-case helpers. Prefer the dedicated builders above for new tests:

| Helper | Returns | Use For |
|--------|---------|---------|
| `make_test_app(state)` | `Result<Arc<DefaultApplicationService>>` | In-memory storage + mock backend, full snapshot seeded — 1 caller `src/application/query_handlers_tests.rs:21` |
| `make_test_app_without_snapshot(state)` | `Result<Arc<DefaultApplicationService>>` | In-memory storage + mock backend, snapshot NOT seeded (forces fresh-state path) — 2 callers; snapshot-skip semantics not expressible in `TestAppBuilder` without a `skip_snapshot` method |
| `seed_test_world_into_storage(storage, state)` | `WorldId` | Seeds world / persona / NPC character rows — 1 external caller `tests/integration/flow/arrival_persistence.rs:139` (failing-storage test) + internal use by the two survivors above |

`DefaultApplicationService` accessors used by tests: `app.storage()`, `app.preset_storage()`, `app.settings()`, `app.cancel_token()` (returns the shutdown token), `app.is_shutting_down()`, `app.is_generating()`, `app.game_service()`. Domain-bound methods: `app.load_or_fresh()`, `app.load_expecting_valid_state()`, `app.save_message_and_snapshot(&mut state)`, `app.delete_and_remove_message(...)`, `app.load_messages_into_state(&mut state)`, `app.load_messages()`, `app.process_action(input)`.
