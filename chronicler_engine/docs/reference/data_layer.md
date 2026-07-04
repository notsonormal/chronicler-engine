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
| `world_key` | TEXT    | Key of the loaded world (ADR-025)         |
| `world_name`| TEXT    | Name of the loaded world                  |
| `persona_key`| TEXT   | Foreign key into `personas.key` (ADR-026) |
| `persona_name`| TEXT | Denormalized persona name for list queries |
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
| `message_type` | TEXT | JSON: `MessageType` enum |
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

## Migration Policy

Migrations run on first access via `run_migrations`; see `src/adapters/driven/storage/db.rs`.
