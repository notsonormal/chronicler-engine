# Storage System Specification

**Document Type:** Domain Specification  
**Module:** `crate::storage`  
**Status:** Implemented (Unified Storage with Backend enum)  
**Related ADRs:** [ADR-019](../adr/adr-019-one-table-per-storage-module.md) (superseded), [ADR-020](../adr/adr-020-storage-consolidation.md)

---

## Overview

The Chronicler Engine storage layer provides unified persistence for game sessions, narrative content, user configurations, and LLM call forensics. It uses a concrete `Storage` struct with a `Backend` enum (`Sqlite`, `InMemory`, `Test`) instead of trait-based repository patterns.

**Key Design Decisions:**

- ✅ **Unified Storage struct** — Single `Storage` struct with all CRUD operations as methods
- ✅ **Backend enum abstraction** — `Sqlite` (production), `InMemory` (dev), `Test` (testing)
- ✅ **No trait boilerplate** — No `dyn Trait`, `Arc<dyn>`, or custom mocks
- ✅ **Single table per method** — Each storage method touches exactly one table
- ✅ **Cross-table coordination in application tier** — `GameServiceContext` composes multi-table operations

---

## Architecture

```
src/storage/
├── mod.rs                # Re-exports and module structure
├── db.rs                 # Schema definitions, migrations v1–v10
├── backend/
│   ├── mod.rs           # Storage struct, Backend enum
│   ├── core.rs          # Sqlite/InMemory/Test backend implementations
│   ├── games.rs         # `games` table CRUD
│   ├── messages.rs      # `messages` table CRUD  
│   ├── swipes.rs        # `message_swipes` table CRUD
│   ├── llm_messages.rs  # `llm_messages` table CRUD
│   ├── worlds.rs        # `worlds`, `maps` tables (seeded from JSON)
│   ├── characters.rs    # `characters` table (seeded from JSON)
│   ├── personas.rs      # `personas` table (seeded from JSON)
│   ├── settings.rs      # singleton `settings` row
│   ├── presets.rs       # `prompt_presets` table
│   └── snapshots.rs     # `game_state_snapshots` table
├── models/              # Database row structs (auto-derived from schema)
│   ├── game.rs
│   ├── message.rs
│   ├── swipe.rs
│   └── ... (one per table)
└── mappers/             # Domain ↔ DB mapping (optional, if needed)
    ├── message.rs
    └── ...
```

---

## Core Tables

### Game Sessions (scoped to `game_id`)

| Table | Purpose | Key Fields |
|-------|---------|------------|
| `games` | Top-level game session record | `id`, `name`, `world_name`, `created_at`, `updated_at` |
| `game_state_snapshots` | Serialized game state metadata | `id`, `game_id`, `snapshot_data` (JSON), `created_at` |
| `messages` | Narrative message history | `id`, `game_id`, `sender`, `log_type`, `timestamp`, `active_swipe_index`, `is_deleted` |
| `message_swipes` | Per-message alternate generations | `id`, `message_id`, `swipe_index`, `text`, `snapshot_id`, `location_header`, `event_header` |

### Global Data (not game-scoped)

| Table | Purpose | Key Fields |
|-------|---------|------------|
| `llm_messages` | LLM API call logging | `id`, `agent_name`, `backend`, `model`, `request_json`, `response_json`, `timestamp` |
| `prompt_presets` | System and quantifier prompt templates | `id`, `preset_type` (system/quantifier), `role`, `instructions`, `writing_style`, `output_format`, `is_default` |

### Seeded Game Data (Migration v10)

Loaded from `data/worlds/<key>/` JSON files at startup, idempotent.

| Table | Purpose | Key Fields |
|-------|---------|------------|
| `worlds` | World definitions | `id`, `key` (unique), `name`, `description`, `global_rules[]`, `starting_room_id` |
| `maps` | World map geometry | `id`, `world_id` FK, `map_data` (JSON blob with full MapDef) |
| `personas` | Player character templates | `id`, `key` (unique), `name`, `description`, `personality`, `inventory[]` |
| `characters` | NPC templates | `id`, `key`, `world_id` FK, `name`, `triggers[]`, `relationships[]` |
| `settings` | Singleton application settings | `id=1`, `connections[]`, `narration_connection_id`, `quantifier_connection_id`, `active_preset_ids[]` |

---

## Storage API

### Constructors

```rust
impl Storage {
    // Production: SQLite database at given path
    pub fn new_sqlite(db_path: &str) -> Result<Self> { }
    
    // Development: In-memory HashMap backend
    pub fn new_in_memory() -> Self { }
    
    // Testing: In-memory with optional failure injection
    pub fn with_test_failures(self) -> (Self, TestFailureHandle) { }
}
```

### Game Lifecycle Operations

```rust
impl Storage {
    pub fn create_game(&self, name: &str, world_name: &str) -> Result<i64> { }
    pub fn list_games(&self, limit: usize) -> Result<Vec<GameSummary>> { }
    pub fn load_game_state(&self, game_id: i64) -> Result<Option<GameSnapshot>> { }
    pub fn save_game_state(&self, game_id: i64, snapshot: &GameSnapshot) -> Result<()> { }
    pub fn delete_game(&self, game_id: i64) -> Result<()> { }
}
```

### Message Operations

```rust
impl Storage {
    pub fn insert_message(&self, game_id: i64, message: &Message) -> Result<i64> { }
    pub fn load_messages_for_game(&self, game_id: i64, since_id: Option<i64>) -> Result<Vec<Message>> { }
    pub fn update_message_swipe_index(&self, message_id: i64, swipe_index: usize) -> Result<()> { }
    pub fn soft_delete_message(&self, message_id: i64) -> Result<()> { }
    
    pub fn insert_swipe(&self, message_id: i64, swipe: &Swipe) -> Result<i64> { }
    pub fn load_swipes_for_message(&self, message_id: i64) -> Result<Vec<Swipe>> { }
    pub fn count_swipes_for_message(&self, message_id: i64) -> Result<usize> { }
}
```

### LLM & Prompt Operations

```rust
impl Storage {
    pub fn log_llm_call(&self, call: &LlmMessage) -> Result<i64> { }
    pub fn list_recent_llm_calls(&self, limit: usize) -> Result<Vec<LlmMessage>> { }
    
    pub fn load_prompt_preset(&self, preset_id: i64) -> Result<Option<PromptPreset>> { }
    pub fn save_prompt_preset(&self, preset: &PromptPreset) -> Result<i64> { }
    pub fn list_prompt_presets(&self, preset_type: PresetType) -> Result<Vec<PromptPreset>> { }
    pub fn delete_prompt_preset(&self, preset_id: i64) -> Result<()> { }
}
```

### Seeded Data Operations

```rust
impl Storage {
    pub fn get_world_by_key(&self, key: &str) -> Result<Option<World>> { }
    pub fn get_character_by_key(&self, world_id: i64, key: &str) -> Result<Option<Character>> { }
    pub fn get_persona_by_key(&self, key: &str) -> Result<Option<Persona>> { }
    
    pub fn get_settings(&self) -> Result<AppSettings> { }
    pub fn save_settings(&self, settings: &AppSettings) -> Result<()> { }
}
```

---

## Seeding Pattern

Game data is seeded from JSON files at application startup via `bootstrap::ensure_defaults()`:

```rust
// Pseudo-code from bootstrap/run.rs
pub fn ensure_defaults(storage: &Storage) -> Result<()> {
    // 1. Load seed data from JSON files
    let worlds = load_worlds_from_json("data/worlds/")?;
    
    // 2. Seed idempotently (skip if key exists)
    for world in worlds {
        if storage.get_world_by_key(&world.key)?.is_none() {
            storage.seed_world(&world)?;
        }
    }
    
    // 3. Load settings (singleton row, create if missing)
    if storage.get_settings().is_err() {
        storage.save_settings(&AppSettings::default())?;
    }
    
    Ok(())
}
```

**Key Properties:**

- ✅ **Idempotent** — Skips if key already exists
- ✅ **JSON as seed templates** — Runtime source of truth is DB
- ✅ **Startup-blocking** — Ensures data exists before server starts

---

## Cross-Table Coordination

Individual storage methods touch exactly one table. Multi-table operations are composed in the application tier:

```rust
// In application/context.rs
impl GameServiceContext {
    /// Save message and snapshot atomically (two sequential statements)
    pub fn save_message_and_snapshot(
        &self,
        message: &Message,
        snapshot: &GameStateSnapshot,
    ) -> Result<()> {
        let snapshot_id = self.storage.save_snapshot(self.game_id, snapshot)?;
        let message_with_snapshot = message.with_snapshot_id(snapshot_id);
        self.storage.insert_message(self.game_id, &message_with_snapshot)?;
        Ok(())
    }
    
    /// Load messages with swipes hydrated
    pub fn load_messages_with_swipes(
        &self,
        since_id: Option<i64>,
    ) -> Result<Vec<Message>> {
        let mut messages = self.storage.load_messages_for_game(self.game_id, since_id)?;
        for msg in &mut messages {
            msg.swipes = self.storage.load_swipes_for_message(msg.id)?;
        }
        Ok(messages)
    }
}
```

**Atomicity:** Sequential SQLite statements within a single connection. Tiny window for inconsistency on crash, acceptable for non-critical data (messages can be regenerated).

---

## Migrations

Schema migrations are defined in `src/storage/db.rs`:

| Version | Purpose | Tables Affected |
|---------|---------|-----------------|
| v1 | Initial schema (2025-01) | `games`, `messages` |
| v2 | Message retry improvements | `messages` |
| v3 | Message swipes support | `message_swipes` |
| v4 | LLM call logging | `llm_messages` |
| v5 | Prompt presets | `prompt_presets` |
| v6 | Message swipes refactor | `message_swipes` schema change |
| v7 | Prompt preset sections | `prompt_presets` (add section columns) |
| v8 | Drop deprecated `prompt_text` | `prompt_presets` |
| v9 | Multi-game support | `games` FK relationships |
| **v10** | **Game data seeding** | `worlds`, `maps`, `personas`, `characters`, `settings` |

---

## Testing Strategy

### In-Memory Backend

```rust
#[test]
fn test_message_persistence() {
    let storage = Storage::new_in_memory();
    let game_id = storage.create_game("Test", "Redmist Estate")?;
    
    let message = Message::new(Sender::Player, "Hello, world!");
    let msg_id = storage.insert_message(game_id, &message)?;
    
    assert!(msg_id > 0);
    let loaded = storage.load_messages_for_game(game_id, None)?;
    assert_eq!(loaded.len(), 1);
}
```

### Failure Injection

```rust
#[test]
fn test_retry_on_db_failure() {
    let (storage, handle) = Storage::new_in_memory().with_test_failures();
    handle.fail_next("insert_message", "simulated DB error");
    
    let err = storage.insert_message(1, &test_message()).unwrap_err();
    assert!(err.to_string().contains("simulated DB error"));
}
```

---

## Architecture Rationale

### Why Unified Storage Over Trait-Based Repositories?

**Trait-Based (Before ADR-020):**

```rust
trait GameStorage { /* 5 methods */ }
trait SnapshotStorage { /* 3 methods */ }
trait MessageStorage { /* 4 methods */ }

struct SqliteGameStorage { /* 60 lines */ }
struct InMemoryGameStorage { /* 45 lines */ }
// ... × 6 traits × 2 backends = 12 structs, 600+ lines
```

**Unified Storage (After ADR-020):**

```rust
struct Storage {
    backend: Backend, // Sqlite | InMemory | Test
}

impl Storage {
    pub fn create_game(&self, ...) { /* match on backend */ }
    pub fn insert_message(&self, ...) { /* match on backend */ }
    // ... 25 methods, ~350 lines total
}
```

**Benefits:**

- **55% code reduction** (1,371 lines → ~620 lines)
- ✅ No trait boilerplate (`dyn Trait`, `Arc<dyn>`, custom mocks)
- ✅ Single `Arc<Storage>` on `GameServiceContext` instead of 5 `Arc<dyn Trait>`
- ✅ Dynamic failure injection via `(Storage, TestFailureHandle)` pair
- ✅ Easier navigation — all storage ops in one file

**Trade-offs:**

- ❌ Large `impl Storage` block (mitigated by submodule organization)
- ❌ Backend enum match in every method (minor perf cost, acceptable)

---

## Module Boundaries

### Storage Tier Must Not Be Accessed By

- ❌ **Model tier** — Pure data structs, no storage knowledge
- ❌ **Engine tier** — Game logic is storage-agnostic  
- ❌ **Narrative tier** — LLM backends operate on in-memory state

### Storage Tier May Depend On

- ✅ `crate::model::*` — Domain structs for mapping
- ✅ `crate::error::*` — Error types

### Who Accesses Storage

- ✅ **Application tier** — `GameServiceContext` composes storage ops
- ✅ **Bootstrap tier** — Seeding, settings initialization
- ✅ **Server tier** — Indirectly via `ApplicationService` trait (never direct)

---

## Security Considerations

- ✅ **No secrets in storage** — API keys live in env vars / `.env`, not DB
- ✅ **SQLite file permissions** — `data/` directory should be user-only
- ✅ **LLM messages contain prompts/responses** — consider redaction for production logging
- ✅ **No SQL injection** — All queries use parameterized bindings via `rusqlite`

---

## Performance Characteristics

| Operation | Latency (SQLite) | Memory (InMemory) |
|-----------|------------------|-------------------|
| `create_game` | ~1–2 ms | ~0.1 µs |
| `insert_message` | ~0.5 ms | ~0.1 µs |
| `load_messages(limit=50)` | ~2–5 ms | ~0.5 µs |
| `save_snapshot` | ~5–10 ms | ~1 µs |
| `load_snapshot` | ~3–8 ms | ~0.5 µs |

**Optimization Note:** Most latency is SQLite I/O. For production, consider WAL mode, connection pooling, or async I/O if needed.

---

## References

- **Implementation:** `src/storage/`
- **Schema & Migrations:** `src/storage/db.rs`
- **Backend Implementations:** `src/storage/backend/core.rs`
- **Seed Data:** `data/worlds/`, `data/settings.json`
- **ADRs:** [ADR-019](../adr/), [ADR-020](../adr/adr-020-storage-consolidation.md)
- **Architecture Spec:** [`system.md`](../architecture/system.md) — Storage Tier section

---

## Change Log

**2026-05-25:** Unified Storage implementation (ADR-020)  
**2026-05-21:** Multi-game support with `game_id` scoping  
**2026-05-19:** Prompt presets with DB seeding (v5–v8 migrations)  
**2026-05-14:** Message swipes support (v3, v6 migrations)  
**2026-05-12:** LLM call logging (v4 migration)  
**2026-01:** Initial SQLite implementation
