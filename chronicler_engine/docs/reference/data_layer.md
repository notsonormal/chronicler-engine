# Data Layer Reference

## Overview

The Chronicler Engine uses SQLite for persistence. The database is created automatically on first access and is scoped to a single file per server instance (e.g. `chronicler_3000.db`).

All game state data is scoped to a **game** record. Multiple games can exist in the same database and are switched at runtime via `set_game_id`.

## Tables

### `games`

Top-level game session record. Every snapshot and message belongs to a game.

| Column      | Type    | Notes                                      |
|-------------|---------|-------------------------------------------|
| `id`        | INTEGER | PRIMARY KEY AUTOINCREMENT                 |
| `world_name`| TEXT    | Name of the loaded world                  |
| `name`      | TEXT    | Display name (e.g. `Redmist_2026-05-21_1`)|
| `created_at`| TEXT    | ISO 8601 timestamp (RFC 3339)             |
| `updated_at`| TEXT    | ISO 8601 timestamp (RFC 3339)             |

**Current behaviour:** On first startup for a world, a new game row is auto-created with a generated name. The active game can be switched via the UI or `SnapshotStorage::set_game_id`.

---

### `game_state_snapshots`

Frozen point-in-time captures of the mutable game state. Used for:
- Loading the latest state on server startup
- Retry (loading snapshots via message `snapshot_id`)

| Column             | Type    | Notes                                         |
|--------------------|---------|----------------------------------------------|
| `id`               | INTEGER | PRIMARY KEY AUTOINCREMENT                    |
| `game_id`          | INTEGER | NOT NULL DEFAULT 1 — foreign key to `games`  |
| `movement`         | TEXT    | JSON: `MovementState`                        |
| `narrative`        | TEXT    | JSON: `NarrativeSnapshot` (no messages)      |
| `scene`            | TEXT    | JSON: `SceneState`                           |
| `npc_encounter_log`  | TEXT    | JSON: `NpcEncounterLog`                       |

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
| `sender`            | TEXT    | Player name, NPC name, or NULL for narrator  |
| `log_type`          | TEXT    | JSON: `LogType` enum                         |
| `timestamp`         | TEXT    | ISO 8601 timestamp                           |
| `active_swipe_index`| INTEGER | Index of currently active swipe version      |
| `is_deleted`        | INTEGER | 0 or 1 — soft delete flag                    |

**Storage contract:** Messages are persisted incrementally. New messages are inserted individually. The active swipe text and snapshot are stored in the `message_swipes` table. There is only ever one message history per game.

**Index:** `idx_messages_game_id(game_id, id)`

---

### `message_swipes`

Per-message swipe versions. Each row is one alternative generation for a message. Cascades on message delete.

| Column           | Type    | Notes                                         |
|------------------|---------|----------------------------------------------|
| `id`             | INTEGER | PRIMARY KEY AUTOINCREMENT                    |
| `message_id`     | INTEGER | FOREIGN KEY -> messages(id) ON DELETE CASCADE|
| `swipe_index`    | INTEGER | Version index (0, 1, 2...)                   |
| `text`           | TEXT    | Message content for this swipe               |
| `snapshot_id`    | INTEGER | REFERENCES game_state_snapshots(id)          |
| `location_header`| TEXT    | Optional room header prefix                  |
| `event_header`   | TEXT    | Optional event header prefix                 |

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
  │     └── messages (?)     [via message_swipes.snapshot_id — optional, not FK]
  └── messages (*)
      └── message_swipes (*) [cascades on message delete]

llm_messages (*)  [independent]
```

## Code Mapping

The Rust code maps to the database as follows:

- **`src/storage/models/`** — One DB model struct per table (`DbGame`, `DbGameStateSnapshot`, `DbMessage`, `DbSwipe`, `DbLlmMessage`). These use raw SQLite types (`String` for JSON and timestamps, `i64` for IDs).
- **`src/storage/mappers/`** — Conversion logic between DB models and domain models. Mappers handle JSON serialization, RFC 3339 parsing, and integer↔unsigned mapping.
- **`src/storage/snapshot_storage.rs`** — `SqliteSnapshotRepository` uses `DbGameStateSnapshot` internally and maps to/from domain models at the trait boundary. Includes game CRUD (`list_games`, `create_game`, `delete_game`, `get_game`) and `set_game_id` for runtime switching. `delete_game` is transactional: all cascading deletes execute inside a SQLite `Transaction`.
- **`src/storage/message_storage.rs`** — `SqliteMessageRepository` handles message persistence and swipe management. Uses `DbMessage`/`DbSwipe` internally and maps at the trait boundary.
- **`src/storage/llm_message_storage.rs`** — `SqliteLlmMessageStorage` uses `DbLlmMessage` internally.
- **`src/model/`** — Domain models (`Message`, `Game`, `LlmMessage`, `GameStateSnapshot`, `NarrativeSnapshot`) have no knowledge of `rusqlite`, JSON strategy, or timestamp formatting.

## Migration Policy

Migration v1 is **breaking** — it drops and recreates tables. Subsequent migrations (v2+) are **incremental** and preserve data where possible (e.g., v6 migrates old `messages.text` into `message_swipes` before dropping the column). This is acceptable because Chronicler is pre-release, but we no longer discard all data on every schema change.

## Future Work

- **Message versioning:** Not implemented; retry creates new messages via snapshot rollback.
- **Snapshot pruning:** Delete old snapshots to limit database growth. With immediate persistence every message has exactly one snapshot, so the table grows linearly with turns.
