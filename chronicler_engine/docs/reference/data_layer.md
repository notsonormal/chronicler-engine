# Data Layer Reference

## Overview

The Chronicler Engine uses SQLite for persistence. The database is created automatically on first access and is scoped to a single file per server instance (e.g. `chronicler_3000.db`).

All game state data is scoped to a **game** record. There is currently one default game per database; multi-game support is planned.

## Tables

### `games`

Top-level game session record. Every snapshot and message belongs to a game.

| Column      | Type    | Notes                                      |
|-------------|---------|-------------------------------------------|
| `id`        | INTEGER | PRIMARY KEY AUTOINCREMENT                 |
| `world_name`| TEXT    | Name of the loaded world                  |
| `created_at`| TEXT    | ISO 8601 timestamp (RFC 3339)             |
| `updated_at`| TEXT    | ISO 8601 timestamp (RFC 3339)             |

**Current behaviour:** A default row (`id=1, world_name='default'`) is inserted automatically by `DbPool::new` during migration. All storage operations filter by this `game_id`.

---

### `game_state_snapshots`

Frozen point-in-time captures of the mutable game state. Used for:
- Loading the latest state on server startup
- Retry (loading snapshots via message `snapshot_id`)
- Checkpoints (named restore points)

| Column             | Type    | Notes                                         |
|--------------------|---------|----------------------------------------------|
| `id`               | INTEGER | PRIMARY KEY AUTOINCREMENT                    |
| `game_id`          | INTEGER | NOT NULL DEFAULT 1 — foreign key to `games`  |
| `movement`         | TEXT    | JSON: `MovementState`                        |
| `narrative`        | TEXT    | JSON: `NarrativeSnapshot` (no messages)      |
| `scene`            | TEXT    | JSON: `SceneState`                           |
| `npc_encounter_log`  | TEXT    | JSON: `NpcEncounterLog`                       |
| `committed`        | INTEGER | 0 or 1 — marks pre-generation snapshots      |
| `created_at`       | TEXT    | ISO 8601 timestamp                           |

**Key invariant:** Messages are **not** stored in the snapshot JSON. They live in the `messages` table and are hydrated after snapshot load.

**Index:** `idx_snapshots_game_latest(game_id, created_at DESC)`

---

### `messages`

Chronological narrative history. Each row is one log entry (player input, narration, system message, dialogue).

| Column           | Type    | Notes                                         |
|------------------|---------|----------------------------------------------|
| `id`             | INTEGER | PRIMARY KEY AUTOINCREMENT                    |
| `game_id`        | INTEGER | NOT NULL DEFAULT 1 — foreign key to `games`  |
| `sender`         | TEXT    | Player name, NPC name, or NULL for narrator  |
| `text`           | TEXT    | Message content                              |
| `log_type`       | TEXT    | JSON: `LogType` enum                         |
| `timestamp`      | TEXT    | ISO 8601 timestamp                           |
| `location_header`| TEXT    | Optional room header prefix                  |
| `event_header`   | TEXT    | Optional event header prefix                 |
| `snapshot_id`    | INTEGER | REFERENCES `game_state_snapshots(id)` — snapshot saved after this message was created |

**Storage contract:** Messages are persisted incrementally. New messages are inserted individually and tagged with the `snapshot_id` of the state captured after their creation. There is only ever one message history per game.

**Index:** `idx_messages_game_id(game_id, id)`

---

### `checkpoints`

Named bookmarks that reference a specific snapshot. Users can create and restore checkpoints via the UI.

| Column       | Type    | Notes                                         |
|--------------|---------|----------------------------------------------|
| `id`         | TEXT    | PRIMARY KEY (user-defined or UUID)           |
| `snapshot_id`| INTEGER | REFERENCES `game_state_snapshots(id)`        |
| `name`       | TEXT    | User-visible label                           |
| `created_at` | TEXT    | ISO 8601 timestamp                           |

**Index:** `idx_checkpoints_snapshot(snapshot_id)`

---

### `llm_messages`

Forensics log of LLM API calls. Independent of game state — used for debugging and prompt engineering.

| Column            | Type    | Notes                            |
|-------------------|---------|---------------------------------|
| `id`              | INTEGER | PRIMARY KEY AUTOINCREMENT       |
| `agent_name`      | TEXT    | e.g. "narrator", "quantifier"   |
| `backend_name`    | TEXT    | e.g. "ollama", "openrouter"     |
| `model_name`      | TEXT    | Model identifier                |
| `system_prompt`   | TEXT    | Full system prompt sent         |
| `user_prompt`     | TEXT    | Full user prompt sent           |
| `raw_request_json`| TEXT    | Raw JSON payload                |
| `raw_response_json`| TEXT   | Raw JSON response               |
| `parsed_response` | TEXT    | Extracted text content          |
| `error_message`   | TEXT    | NULL on success                 |
| `created_at`      | TEXT    | ISO 8601 timestamp              |

**Note:** Not game-scoped. This table is pruned automatically (default cap: 50 rows).

## Relationships

```
games (1)
  ├── game_state_snapshots (*)
  │     ├── checkpoints (*)  [via snapshot_id]
  │     └── messages (?)     [via snapshot_id — optional, not FK]
  └── messages (*)

llm_messages (*)  [independent]
```

## Code Mapping

The Rust code maps to the database as follows:

- **`src/storage/models/`** — One DB model struct per table (`DbGame`, `DbGameStateSnapshot`, `DbCheckpoint`, `DbMessage`, `DbLlmMessage`). These use raw SQLite types (`String` for JSON and timestamps, `i64` for IDs).
- **`src/storage/mappers/`** — Conversion logic between DB models and domain models. Mappers handle JSON serialization, RFC 3339 parsing, and integer↔unsigned mapping.
- **`src/storage/snapshot_storage.rs`** — `SqliteGameStorage` uses `DbGameStateSnapshot`/`DbCheckpoint`/`DbMessage` internally and maps to/from domain models at the trait boundary.
- **`src/storage/llm_message_storage.rs`** — `SqliteLlmMessageStorage` uses `DbLlmMessage` internally.
- **`src/model/`** — Domain models (`Message`, `Checkpoint`, `LlmMessage`, `GameStateSnapshot`, `NarrativeSnapshot`) have no knowledge of `rusqlite`, JSON strategy, or timestamp formatting.

## Migration Policy

Schema migrations are **breaking** — old save data is discarded on schema change. The `run_migrations` function drops and recreates tables. This is acceptable because Chronicler is currently pre-release and has no backward-compatibility guarantee for save files.

## Future Work

- **Multi-game support:** Allow creating multiple `games` rows and switching between them.
- **Message versioning:** Not implemented; retry creates new messages via snapshot rollback.
- **Snapshot pruning:** Delete old uncommitted snapshots to limit database growth.
