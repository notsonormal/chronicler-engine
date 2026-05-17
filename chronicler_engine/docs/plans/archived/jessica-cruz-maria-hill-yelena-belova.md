# Plan: Separate Domain Models from DB Models in Chronicler Engine

## Objective

Refactor the storage layer so that **domain models** (used by engine, application, and server tiers) live in `src/model/` and **database models** (table-row representations) live in `src/storage/models/`. Mappers sit between them inside the storage tier. This removes the current confusion where `src/model/storage/` contains domain models that are also used as DB DTOs.

After this refactor:
- `src/model/` contains pure domain structs: `Message`, `Checkpoint`, `LlmMessage`, `GameStateSnapshot`, `NarrativeSnapshot`.
- `src/storage/models/` contains DB structs: `DbMessage`, `DbCheckpoint`, `DbLlmMessage`, `DbGameStateSnapshot`, `DbGame`.
- `src/storage/mappers/` contains conversion logic between the two worlds.
- Storage implementations (`SqliteGameStorage`, `SqliteLlmMessageStorage`) use DB models internally and map at the boundary.
- Public storage traits (`SnapshotStorage`, `MessageStorage`, `LlmMessageStorage`) continue to speak domain models.

## Current State

- `src/model/storage/` holds what are effectively domain models (`Message`, `Checkpoint`, `LlmMessage`, `GameStateSnapshot`, `NarrativeSnapshot`).
- `src/model/checkpoint.rs`, `message.rs`, `llm_message.rs`, `state_snapshot.rs` are thin re-export stubs pointing into `src/model/storage/`.
- `src/storage/snapshot_storage.rs` and `src/storage/llm_message_storage.rs` do inline row-mapping directly into domain structs (no DB model layer).
- `src/model/state.rs` imports `crate::model::storage::Message` and `crate::model::storage::UNPERSISTED_ID`.
- `src/bootstrap/run.rs` and `src/application/game_service/helpers.rs` reference `crate::model::storage::UNPERSISTED_ID`.
- Several test files import via `crate::model::storage::*` paths.

## Target Architecture

```
src/
  model/
    mod.rs                    # remove `pub mod storage`, keep direct modules
    checkpoint.rs             # moved from model/storage/checkpoint.rs
    llm_message.rs            # moved from model/storage/llm_message.rs
    message.rs                # moved from model/storage/message.rs
    state_snapshot.rs         # moved from model/storage/state_snapshot.rs
    state_snapshot_tests.rs   # moved from model/storage/state_snapshot_tests.rs
  storage/
    models/
      mod.rs
      game.rs                 # DbGame (id, world_name, created_at, updated_at)
      game_state_snapshot.rs  # DbGameStateSnapshot (flat, JSON columns as String)
      checkpoint.rs           # DbCheckpoint
      message.rs              # DbMessage (log_type as JSON String, timestamp as String)
      llm_message.rs          # DbLlmMessage (timestamp as String)
    mappers/
      mod.rs
      checkpoint.rs           # DbCheckpoint <-> Checkpoint
      message.rs              # DbMessage <-> Message
      llm_message.rs          # DbLlmMessage <-> LlmMessage
      state_snapshot.rs       # DbGameStateSnapshot <-> GameStateSnapshot
    db.rs                     # unchanged
    llm_message_storage.rs    # uses DbLlmMessage + mapper internally
    message_storage.rs        # trait unchanged
    snapshot_storage.rs       # uses Db* + mappers internally
    mod.rs                    # add `pub mod models; pub mod mappers;`
```

## Detailed Changes

### 1. Create DB Models (`src/storage/models/`)

Each DB model is a plain struct whose fields map 1-to-1 to SQLite columns, using raw types (`String` for JSON, `String` for RFC 3339 timestamps, `i64` for integer IDs). No serde derives needed unless useful for testing.

**`src/storage/models/game.rs`**
```rust
pub struct DbGame {
    pub id: i64,
    pub world_name: String,
    pub created_at: String,
    pub updated_at: String,
}
```

**`src/storage/models/game_state_snapshot.rs`**
```rust
pub struct DbGameStateSnapshot {
    pub id: i64,
    pub game_id: i64,
    pub movement_json: String,
    pub narrative_json: String,
    pub scene_json: String,
    pub character_state_json: String,
    pub committed: i32,
    pub created_at: String,
}
```

**`src/storage/models/checkpoint.rs`**
```rust
pub struct DbCheckpoint {
    pub id: String,
    pub snapshot_id: i64,
    pub name: String,
    pub created_at: String,
}
```

**`src/storage/models/message.rs`**
```rust
pub struct DbMessage {
    pub id: i64,
    pub game_id: i64,
    pub sender: Option<String>,
    pub text: String,
    pub log_type_json: String,
    pub timestamp: String,
    pub location_header: Option<String>,
    pub event_header: Option<String>,
    pub snapshot_id: Option<i64>,
}
```

**`src/storage/models/llm_message.rs`**
```rust
pub struct DbLlmMessage {
    pub id: i64,
    pub agent_name: String,
    pub backend_name: String,
    pub model_name: String,
    pub system_prompt: String,
    pub user_prompt: String,
    pub raw_request_json: String,
    pub raw_response_json: String,
    pub parsed_response: String,
    pub error_message: Option<String>,
    pub created_at: String,
}
```

### 2. Create Mappers (`src/storage/mappers/`)

Mappers are infallible or fallible conversions. Use `TryFrom`/`From` where possible; use free functions when extra context (e.g. `game_id`) is required.

**`src/storage/mappers/checkpoint.rs`**
- `impl From<&DbCheckpoint> for Checkpoint`
- `impl From<&Checkpoint> for DbCheckpoint`

**`src/storage/mappers/message.rs`**
- `impl TryFrom<&DbMessage> for Message` — parses `log_type_json` and `timestamp`
- `impl TryFrom<&Message> for DbMessage` — serializes `log_type`, formats `timestamp`; takes `game_id: i64` as parameter (free function `message_to_db`)

**`src/storage/mappers/llm_message.rs`**
- `impl TryFrom<&DbLlmMessage> for LlmMessage` — parses `created_at`
- `impl From<&LlmMessage> for DbLlmMessage` — formats `created_at`; note `id` may be `0` for unpersisted

**`src/storage/mappers/state_snapshot.rs`**
- `impl TryFrom<&DbGameStateSnapshot> for GameStateSnapshot` — parses JSON columns and `created_at`
- `impl TryFrom<&GameStateSnapshot> for DbGameStateSnapshot` — serializes JSON columns, formats `created_at`; takes `game_id: i64` as parameter (free function `snapshot_to_db`)

### 3. Move Domain Models (`src/model/storage/*` -> `src/model/*`)

Overwrite the existing re-export stubs with the actual struct definitions:

- `src/model/checkpoint.rs` ← content from `src/model/storage/checkpoint.rs`
- `src/model/llm_message.rs` ← content from `src/model/storage/llm_message.rs`
- `src/model/message.rs` ← content from `src/model/storage/message.rs`
- `src/model/state_snapshot.rs` ← content from `src/model/storage/state_snapshot.rs`
- `src/model/state_snapshot_tests.rs` ← content from `src/model/storage/state_snapshot_tests.rs`

Delete `src/model/storage/` directory entirely.

Update `src/model/mod.rs`:
- Remove `pub mod storage;`
- Keep existing `pub mod checkpoint; pub mod llm_message; pub mod message; pub mod state_snapshot;`
- Move `#[cfg(test)] mod state_snapshot_tests;` to the top level of `model/mod.rs`.

### 4. Update Storage Implementations

**`src/storage/snapshot_storage.rs`**
- Replace inline `row_to_snapshot` and `row_to_checkpoint` logic with mapper calls.
- In `save()`, build a `DbGameStateSnapshot` via mapper, then execute INSERT.
- In `load_latest()`, `load_by_id()`, `load_checkpoint()`, `list_checkpoints()`: query rows into `Db*` structs, then map to domain structs.
- In `save_checkpoint()`, convert `Checkpoint` to `DbCheckpoint` before INSERT.
- In `insert_message()` / `update_message()` / `load_messages()`: use `DbMessage` internally.
- Remove `parse_json` and `parse_datetime` helpers (mappers handle this).

**`src/storage/llm_message_storage.rs`**
- Replace inline `row_to_message` with mapper.
- In `save()`, convert `LlmMessage` to `DbLlmMessage` before INSERT.
- In `list_latest()`, map `DbLlmMessage` rows to `LlmMessage`.

### 5. Update Import Paths Across the Codebase

| File | Current Import | New Import |
|------|---------------|------------|
| `src/model/state.rs` | `crate::model::storage::Message` | `crate::model::Message` |
| `src/model/state.rs` | `crate::model::storage::UNPERSISTED_ID` | `crate::model::UNPERSISTED_ID` |
| `src/model/state.rs` | `crate::model::storage::NarrativeSnapshot` | `crate::model::NarrativeSnapshot` |
| `src/bootstrap/run.rs` | `crate::model::storage::UNPERSISTED_ID` | `crate::model::UNPERSISTED_ID` |
| `src/application/game_service/helpers.rs` | `crate::model::storage::UNPERSISTED_ID` | `crate::model::UNPERSISTED_ID` |
| `src/application/game_service/helpers_tests.rs` | `crate::model::storage::message::Message` | `crate::model::Message` |
| `src/storage/snapshot_storage_tests.rs` | `crate::model::storage::NarrativeSnapshot` | `crate::model::NarrativeSnapshot` |
| `src/storage/snapshot_storage_tests.rs` | `crate::model::storage::state_snapshot::GameStateSnapshot` | `crate::model::GameStateSnapshot` |
| `src/storage/snapshot_storage_tests.rs` | `crate::model::storage::message::Message` | `crate::model::Message` |
| `src/test_support/in_memory_storage_tests.rs` | `crate::model::storage::NarrativeSnapshot` | `crate::model::NarrativeSnapshot` |
| `src/test_support/in_memory_storage_tests.rs` | `crate::model::storage::state_snapshot::GameStateSnapshot` | `crate::model::GameStateSnapshot` |

### 6. Add Arch-Lint Guardrails

Update `chronicler_engine/arch-lint.toml` to enforce the boundary programmatically:

```toml
[[scopes]]
name = "storage"
paths = ["src/storage/**"]

[[scopes]]
name = "storage_models"
paths = ["src/storage/models/**", "src/storage/mappers/**"]

[[scopes]]
name = "bootstrap"
paths = ["src/bootstrap/**"]

[[scopes]]
name = "test_support"
paths = ["src/test_support/**"]

# DB models and mappers are an internal storage concern.
[[deny-scope-dep]]
from = "model"
to = ["storage_models"]
message = "Model layer must not depend on storage DB models/mappers."

[[deny-scope-dep]]
from = "engine"
to = ["storage_models"]
message = "Engine layer must not depend on storage DB models/mappers."

[[deny-scope-dep]]
from = "narrative"
to = ["storage_models"]
message = "Narrative layer must not depend on storage DB models/mappers."

[[deny-scope-dep]]
from = "application"
to = ["storage_models"]
message = "Application layer must not depend on storage DB models/mappers."

[[deny-scope-dep]]
from = "server"
to = ["storage_models"]
message = "Server layer must not depend on storage DB models/mappers."

[[deny-scope-dep]]
from = "bootstrap"
to = ["storage_models"]
message = "Bootstrap layer must not depend on storage DB models/mappers."

[[deny-scope-dep]]
from = "test_support"
to = ["storage_models"]
message = "Test support must not depend on storage DB models/mappers."
```

Verify with `cargo nextest run --test architecture` after the refactor.

### 7. Update Documentation

- Update `chronicler_engine/docs/architecture/system.md` §7 (Storage Tier) to mention the DB model / mapper / domain model separation.
- Update `chronicler_engine/AGENTS.md` structure diagram: `model/storage/` becomes `storage/models/` and `storage/mappers/`.

## Implementation Order

1. **Create DB models** (`src/storage/models/*`) — no dependencies, pure data structs.
2. **Create mappers** (`src/storage/mappers/*`) — depends on DB models + existing domain models.
3. **Move domain models** (`src/model/storage/*` → `src/model/*`) — overwrite stubs, delete `model/storage/`, update `model/mod.rs`.
4. **Refactor storage implementations** — replace inline mapping with mapper calls.
5. **Fix imports** across `bootstrap/`, `application/`, `model/`, `storage/`, `test_support/`.
6. **Add arch-lint rules** in `arch-lint.toml` to enforce the DB-model boundary.
7. **Run build gate**: `python build.py` (fmt + clippy + tests + guardrails).
8. **Update docs**: `system.md` and `AGENTS.md`.

## Verification Criteria

- [ ] `cargo build` succeeds with zero warnings.
- [ ] `cargo clippy --all-targets -- -D warnings` passes.
- [ ] `cargo nextest run --tests` passes (all integration tests).
- [ ] `cargo nextest run --test architecture` passes (arch-lint guardrails).
- [ ] `src/model/storage/` no longer exists.
- [ ] `src/storage/models/` exists with one struct per DB table.
- [ ] `src/storage/mappers/` exists with conversion logic.
- [ ] `cargo nextest run --test architecture` passes with the new `storage_models` scope rules.
- [ ] No file outside `src/storage/` imports from `src/storage/models/` or `src/storage/mappers/`.
- [ ] Domain models in `src/model/` have no knowledge of `rusqlite`, JSON serialization strategy, or RFC 3339 formatting.

## Risks & Mitigations

| Risk | Mitigation |
|------|-----------|
| Missing import updates cause compile errors | Compile after each batch; grep for remaining `model::storage` references. |
| Mapper `TryFrom` failures change error messages | Keep error mapping equivalent (same `EngineError::Config(...)` wrapping). |
| `state_snapshot_tests.rs` path change breaks test discovery | Ensure `model/mod.rs` registers `#[cfg(test)] mod state_snapshot_tests;` at the new path. |
| `arch-lint` layer violations if model imports storage | Model must NOT import `storage::models` or `storage::mappers`; verify with `cargo nextest run --test architecture`. |
