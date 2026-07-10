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
| `TestPersona` | `standard()`, `named(name)` | `PersonaCard` |
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
    TestCharacterSheet, TestMap, TestNpc, TestPersona,
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
| `SpyForensics` | Counts `save_llm_message` calls via atomic counter; use when a test must assert a code path routed through `LlmCallRecorder::complete` (regression guard for the quantifier forensics bypass) |
| `RecordingForensics` | Records every `save_llm_message` call; exposes counters + last message + injectable error for failure-injection tests |

All three satisfy the `LlmMessageRepository` port; recorder wraps a provider + a spy via `make_test_recorder(provider)` / `make_test_recorder_with_storage(provider, storage)` / `make_spy_recorder(provider)`.

## Integration Test Helpers

`tests/helpers/fixtures.rs` exposes shared builders to integration tests via `tests/integration/mod.rs`. Prefer these over re-defining per file:

| Helper | Purpose |
|--------|---------|
| `create_test_world()`, `create_test_world_with_scenario()` | Canonical `WorldCard` (scenario variant carries `StartingScenario`) |
| `create_test_player()`, `create_test_map()`, `create_test_npcs()` | Canonical character / map builders |
| `create_test_state()`, `create_basic_test_state()`, `create_basic_test_state_no_scenario()` | Canonical `GameState` builders |
| `seed_test_world(storage)` | Seeds `TestWorld::minimal()` + `TestPersona::standard()` |
| `make_test_app(state)` | Builds `Arc<DefaultApplicationService>` (in-memory storage + mock backend); returns `Result` due to lib clippy context |
| `create_test_storage(id)`, `create_test_storage_arc(id)` | Sqlite-backed `Storage` with `games` row pre-seeded (FK-safe) |

`create_test_storage(id)` delegates to `test_support::seed_default_game_row` so sqlite-backed tests satisfy the `game_state_snapshots.game_id` / `messages.game_id` FK constraints without relying on a migration-seeded default game.

### Test App Factories (`test_support::context`)

`src/test_support/context.rs` exposes factories that build `Arc<DefaultApplicationService>` (replacement for the deleted `OpContext` test factories). All factories seed world / player / NPCs and pre-populate the snapshot + messages table when applicable. Most return `Result<Arc<DefaultApplicationService>>` because the lib clippy context denies `unwrap` / `expect` / `panic` — propagate the error with `?` or `unwrap_or_else(|e| panic!(...))` at the call site:

| Factory | Returns | Use For |
|---------|---------|---------|
| `make_test_app(state)` | `Result<Arc<DefaultApplicationService>>` | Default fixture: in-memory storage + mock backend, full snapshot seeded |
| `make_test_app_without_snapshot(state)` | `Result<Arc<DefaultApplicationService>>` | In-memory storage + mock backend, snapshot NOT seeded (forces fresh-state path) |
| `make_test_app_with_sqlite(state)` | `Result<Arc<DefaultApplicationService>>` | Sqlite-backed storage + mock backend, full snapshot seeded |
| `make_test_app_with_mock_backend(state, F)` | `Result<Arc<DefaultApplicationService>>` | Sqlite-backed storage + caller-supplied `Fn() -> MockBackend` |
| `make_test_app_with_backends(state, narrator)` | `Result<Arc<DefaultApplicationService>>` | Sqlite + `GameService::with_backends` (no quantifier agent) |
| `make_test_app_with_separate_backends(state, n, q)` | `Result<Arc<DefaultApplicationService>>` | Sqlite + `with_mock_quantifier` with separate narrator / quantifier factories |
| `make_test_app_with_game_service(state, build)` | `Result<Arc<DefaultApplicationService>>` | Most flexible: caller builds the whole `GameService` given the seeded `Arc<Storage>` |
| `make_test_app_with_storage_and_service(storage, game_service)` | `Arc<DefaultApplicationService>` | Rebuild an app over EXISTING storage with a new `GameService`; preserves storage contents |

`DefaultApplicationService` accessors used by tests: `app.storage()`, `app.preset_storage()`, `app.settings()`, `app.cancel_token()` (returns the shutdown token), `app.is_shutting_down()`, `app.is_generating()`, `app.game_service()`. Domain-bound methods: `app.load_or_fresh()`, `app.load_expecting_valid_state()`, `app.save_message_and_snapshot(&mut state)`, `app.delete_and_remove_message(...)`, `app.load_messages_into_state(&mut state)`, `app.load_messages()`, `app.process_action(input)`.