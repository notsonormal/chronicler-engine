# Plan: Phase 1 — SQLite Snapshots + Reset Game

**Date:** 2026-05-09
**Status:** Planned
**Parent Spec:** `docs/plans/multi-agent-architecture-overarching-spec.md`
**Goal:** Replace `Arc<Mutex<GameState>>` with SQLite-backed per-message snapshots. Add reset game endpoint.
**Estimated Effort:** 2–3 weeks

---

## Overview

This is Phase 1 of the agent-ready pipeline restructure. It has no dependencies and produces a working engine at every task. The other phases cannot begin until this phase is complete.

**What changes:**
- `GameState` is no longer mutated in place. Each turn produces a new snapshot of mutable sub-state saved to SQLite.
- `Arc<Mutex<GameState>>` is removed from `AppState`. State is loaded from SQLite on demand.
- Regeneration creates a new snapshot row (swipe) without destroying the original.
- A reset endpoint clears SQLite and reloads the world from JSON.

**What does NOT change:**
- Narrative quality (same LLM prompts)
- Quantifier behaviour (still runs post-generation, still produces same output)
- HTMX dashboard appearance
- World/map/character JSON formats
- `GameState` struct definition (it stays in memory during a turn)

**Critical insight from code review:** `GameState` contains `Arc<WorldCard>`, `Arc<MapDef>`, `Arc<PlayerCard>` — these are NOT serialisable. The snapshot stores only the **mutable** sub-structs (`MovementState`, `NarrativeState`, `SceneState`, `CharacterState`), which already derive `Serialize, Deserialize`. World data is cached separately and re-attached on load.

---

## Architecture Decisions

1. **Use `rusqlite` with `bundled` feature.** Embeds SQLite into the binary — no system package manager needed.
2. **Snapshots store only mutable sub-state.** `world`, `map`, `player`, `npcs` are cached in `AppState` as immutable `Arc`s. Only `movement`, `narrative`, `scene`, `character_state` are persisted.
3. **Migrations are code, not external `.sql` files.** A `run_migrations(conn)` function applies `CREATE TABLE IF NOT EXISTS`. No migration runner tool needed.
4. **Reset is hard delete.** Drops all snapshot rows and rebuilds initial state from `GameState::new`. Save/load is future work.
5. **Keep `GameState` in memory for the active turn.** Engine loads latest snapshot at turn start, builds a `GameState` from cached world data + snapshot sub-state, operates in RAM, then snapshots mutable parts at turn end.
6. **Use `uuid` for snapshot IDs.** Add `uuid` crate with `v4` feature. SQLite row IDs are strings.

---

## Task 1.1: Add rusqlite + uuid Dependencies

**Goal:** SQLite and UUID compile and link correctly.

### Steps
1. Add to `chronicler_engine/Cargo.toml`:
   ```toml
   rusqlite = { version = "0.34", features = ["bundled"] }
   uuid = { version = "1.16", features = ["v4", "serde"] }
   ```
2. Add to root `.gitignore` (repo root, not `chronicler_engine/`):
   ```
   # SQLite
   *.db
   *.db-journal
   *.db-wal
   *.db-shm
   ```
3. Verify build: `cd chronicler_engine && cargo check`.

**Files:**
- `chronicler_engine/Cargo.toml`
- `.gitignore` (repo root)

**Acceptance Criteria:**
- [ ] `cargo check` passes with no new warnings
- [ ] `cargo test` passes (new deps do not break existing tests)
- [ ] `.db` files are ignored by git

---

## Task 1.2: DB Module + Migrations

**Goal:** SQLite connection management and schema creation.

### Steps
1. Create `src/storage/mod.rs`:
   ```rust
   pub mod db;
   pub mod snapshot_storage;
   ```
2. Create `src/storage/db.rs`:
   ```rust
   use rusqlite::Connection;
   use std::sync::{Arc, Mutex};

   pub struct DbPool {
       conn: Mutex<Connection>,
   }

   impl DbPool {
       pub fn new(path: &str) -> Result<Self, crate::error::EngineError> {
           let conn = Connection::open(path)
               .map_err(|e| crate::error::EngineError::Config(format!("Failed to open DB: {e}")))?;
           run_migrations(&conn)?;
           Ok(Self { conn: Mutex::new(conn) })
       }

       pub fn conn(&self) -> std::sync::MutexGuard<Connection> {
           self.conn.lock().expect("DB connection mutex poisoned")
       }
   }

   fn run_migrations(conn: &Connection) -> Result<(), crate::error::EngineError> {
       conn.execute(
           "CREATE TABLE IF NOT EXISTS game_state_snapshots (
               id TEXT PRIMARY KEY,
               message_id TEXT NOT NULL,
               swipe_index INTEGER NOT NULL DEFAULT 0,
               movement TEXT NOT NULL,
               narrative TEXT NOT NULL,
               scene TEXT NOT NULL,
               character_state TEXT NOT NULL,
               committed INTEGER NOT NULL DEFAULT 0,
               created_at TEXT NOT NULL
           )",
           [],
       ).map_err(|e| crate::error::EngineError::Config(format!("Migration failed: {e}")))?;

       conn.execute(
           "CREATE INDEX IF NOT EXISTS idx_snapshots_message ON game_state_snapshots(message_id, swipe_index)",
           [],
       ).map_err(|e| crate::error::EngineError::Config(format!("Migration failed: {e}")))?;

       conn.execute(
           "CREATE INDEX IF NOT EXISTS idx_snapshots_latest ON game_state_snapshots(created_at DESC)",
           [],
       ).map_err(|e| crate::error::EngineError::Config(format!("Migration failed: {e}")))?;

       Ok(())
   }
   ```
3. Initialise `DbPool` in `bootstrap.rs::run()` before creating `GameState`. Default path: `data/chronicler.db`.

**Files:**
- `src/storage/mod.rs` (new)
- `src/storage/db.rs` (new)
- `src/bootstrap.rs`

**Acceptance Criteria:**
- [ ] `cargo run -- db-migrate` creates `data/chronicler.db` with correct schema
- [ ] Running twice does not error (idempotent)
- [ ] `DbPool` is accessible from `AppState`

---

## Task 1.3: Snapshot Types

**Goal:** Define the snapshot data model and conversions.

### Context from code review

`GameState` (from `model/state.rs`):
```rust
pub struct GameState {
    pub world: Arc<WorldCard>,      // NOT serialisable
    pub map: Arc<MapDef>,           // NOT serialisable
    pub player: Arc<PlayerCard>,    // NOT serialisable
    pub npcs: HashMap<String, NpcCard>, // NOT serialisable (no serde on GameState)
    pub movement: MovementState,    // HAS serde
    pub narrative: NarrativeState,  // HAS serde
    pub scene: SceneState,          // HAS serde
    pub character_state: CharacterState, // HAS serde
}
```

The mutable sub-structs all derive `Serialize, Deserialize, PartialEq`. The snapshot stores only these four fields.

### Steps
1. Create `src/model/state_snapshot.rs`:
   ```rust
   use chrono::{DateTime, Utc};
   use serde::{Serialize, Deserialize};
   use crate::model::state::{MovementState, NarrativeState, SceneState};
   use crate::model::trigger::CharacterState;

   #[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
   pub struct GameStateSnapshot {
       pub id: String,                    // uuid v4
       pub message_id: String,
       pub swipe_index: u32,
       pub movement: MovementState,
       pub narrative: NarrativeState,
       pub scene: SceneState,
       pub character_state: CharacterState,
       pub committed: bool,
       pub created_at: DateTime<Utc>,
   }

   impl GameStateSnapshot {
       pub fn from_game_state(state: &GameState, message_id: String, swipe_index: u32) -> Self {
           Self {
               id: uuid::Uuid::new_v4().to_string(),
               message_id,
               swipe_index,
               movement: state.movement.clone(),
               narrative: state.narrative.clone(),
               scene: state.scene.clone(),
               character_state: state.character_state.clone(),
               committed: false,
               created_at: Utc::now(),
           }
       }

       pub fn apply_to(&self, state: &mut GameState) {
           state.movement = self.movement.clone();
           state.narrative = self.narrative.clone();
           state.scene = self.scene.clone();
           state.character_state = self.character_state.clone();
       }
   }
   ```
2. Add module declaration in `src/model/mod.rs`.

**Files:**
- `src/model/state_snapshot.rs` (new)
- `src/model/mod.rs`

**Acceptance Criteria:**
- [ ] `GameStateSnapshot::from_game_state` creates snapshot from real `GameState`
- [ ] `apply_to` mutates target `GameState` correctly
- [ ] Round-trip test: create `GameState` → snapshot → apply to blank `GameState` → fields match
- [ ] `created_at` is within 1 second of `Utc::now()`

---

## Task 1.4: Snapshot Storage Implementation

**Goal:** CRUD operations for snapshots in SQLite.

### Steps
1. Create `src/storage/snapshot_storage.rs`:
   ```rust
   use crate::error::EngineError;
   use crate::model::state_snapshot::GameStateSnapshot;

   pub trait SnapshotStorage: Send + Sync {
       fn save(&self, snapshot: &GameStateSnapshot) -> Result<(), EngineError>;
       fn load_latest(&self, message_id: Option<&str>) -> Result<Option<GameStateSnapshot>, EngineError>;
       fn load_by_message(&self, message_id: &str, swipe_index: u32) -> Result<Option<GameStateSnapshot>, EngineError>;
       fn commit(&self, snapshot_id: &str) -> Result<(), EngineError>;
       fn reset(&self) -> Result<(), EngineError>;
   }

   pub struct SqliteSnapshotStorage {
       pool: crate::storage::db::DbPool,
   }

   impl SqliteSnapshotStorage {
       pub fn new(pool: crate::storage::db::DbPool) -> Self {
           Self { pool }
       }
   }

   impl SnapshotStorage for SqliteSnapshotStorage {
       // ... implementation using serde_json for JSON columns ...
   }
   ```
2. Implement `save`:
   - Serialise `movement`, `narrative`, `scene`, `character_state` to JSON with `serde_json::to_string`
   - Upsert on `(message_id, swipe_index)` conflict: replace existing row
3. Implement `load_latest`:
   - If `message_id` is `Some`, load latest for that message ordered by `created_at DESC`
   - If `None`, load latest across all messages
   - Deserialise JSON columns back into structs
4. Implement `commit`:
   - `UPDATE game_state_snapshots SET committed = 1 WHERE id = ?`
5. Implement `reset`:
   - `DELETE FROM game_state_snapshots`
   - `VACUUM` (optional)

**Files:**
- `src/storage/snapshot_storage.rs` (new)

**Acceptance Criteria:**
- [ ] Save + load round-trips (unit test with `SqliteSnapshotStorage`)
- [ ] Upsert: saving same `(message_id, swipe_index)` replaces old row
- [ ] `load_latest(None)` returns most recent snapshot
- [ ] `commit` sets flag
- [ ] `reset` empties table; subsequent `load_latest` returns `None`

---

## Task 1.5: Stateless Action Processing

**Goal:** `execute_freeaction_impl` no longer mutates state in place.

### Context from code review

Current signature (`src/engine/action_processing.rs:287`):
```rust
pub fn execute_freeaction_impl(
    state: &mut GameState,
    ctx: &FreeActionContext<'_>,
) -> Result<Option<TriggerContinuationRequest>, EngineError>
```

Internal helpers mutate `state`:
- `handle_movement(state, ...)` — line 50
- `apply_npc_events(state, ...)` — line 97
- `commit_trigger_narration(state, ...)` — line 115

Current caller (`src/engine/game_service.rs:245`):
```rust
let trigger_request = with_state_lock(&state_for_thread, |state| {
    execute_freeaction_impl(state, &FreeActionContext { ... })
});
```

### Steps
1. Add `TurnResult` struct to `src/engine/action_processing.rs`:
   ```rust
   pub struct TurnResult {
       pub next_state: GameState,
       pub narration: String,
       pub trigger_continuation: Option<TriggerContinuationRequest>,
   }
   ```
2. Change `execute_freeaction_impl` signature:
   ```rust
   pub fn execute_freeaction_impl(
       state: &GameState,           // immutable
       ctx: &FreeActionContext<'_>,
   ) -> Result<TurnResult, EngineError>
   ```
3. Refactor internal helpers to return new state:
   - `handle_movement(state, destination, new_npc_ids) -> GameState`
   - `apply_npc_events(state, events) -> GameState`
   - `commit_trigger_narration(state, request, text) -> GameState`
   
   Each helper clones the input state, mutates the clone, returns it.
4. Thread state through `execute_freeaction_impl`:
   ```rust
   let state = handle_movement(state, ...);
   let state = add_log_to_state(state, ...);  // new helper
   let state = apply_npc_events(state, ...);
   let state = build_trigger_request(state, ...); // may need state too
   ```
   Note: `state.add_log(...)` is a method on `GameState`. Either add a free function `add_log_to_state(state, ...) -> GameState` or clone + call method.
5. Replace all `.ok()` swallow patterns with `?` or explicit handling:
   - Line 93: `assert_state_consistency(state).ok();` → `assert_state_consistency(state)?;`
   - Line 110: `assert_state_consistency(state).ok();` → `assert_state_consistency(state)?;`
   - Line 137: `assert_state_consistency(state).ok();` → `assert_state_consistency(state)?;`
   - Line 283: `assert_state_consistency(state).ok();` → `assert_state_consistency(state)?;`
6. Update `game_service.rs::execute_action`:
   - Load latest snapshot from storage (or use current in-memory state for sync actions)
   - For `FreeAction`: build `GameState` from snapshot + cached world data, call `execute_freeaction_impl`, save resulting `next_state` as new snapshot
   - For sync actions (`Look`, `Talk`, `Inventory`, `Quit`): still operate on in-memory state for now, or load/save snapshot

**Files:**
- `src/engine/action_processing.rs`
- `src/engine/game_service.rs`

**Acceptance Criteria:**
- [ ] `cargo test --features diagnostics` passes
- [ ] `execute_freeaction_impl` takes `&GameState` (not `&mut`)
- [ ] `TurnResult` contains fully populated `next_state`
- [ ] Each turn creates a new row in SQLite (verify with test)
- [ ] No `.ok()` swallow in action processing

---

## Task 1.6: Update GameService + AppState

**Goal:** Replace `Arc<Mutex<GameState>>` with `Arc<SnapshotStorage>` + cached world data.

### Context from code review

`GameService` trait (`src/engine/game_service.rs:20`):
```rust
pub trait GameService: Send + Sync {
    fn execute_action(&self, state: Arc<Mutex<GameState>>, input: String, player_name: String);
    fn retry_last_response(&self, state: Arc<Mutex<GameState>>);
}
```

`AppState` (`src/server/mod.rs:146`):
```rust
pub struct AppState {
    pub state: Arc<std::sync::Mutex<GameState>>,
    pub game_service: Arc<dyn GameService>,
    pub settings: Arc<RwLock<AppSettings>>,
    pub cancel_token: CancellationToken,
}
```

`AppState::lock_state()` (`src/server/mod.rs:164`):
```rust
pub fn lock_state(&self) -> crate::error::Result<std::sync::MutexGuard<GameState>> {
    self.state.lock().map_err(|_| crate::error::EngineError::Config("Lock poisoned".into()))
}
```

Many fragment renderers call `state.lock_state()?` (`src/server/fragments.rs:42`, `47`, `99`, `105`, `155`, etc.).

### Steps
1. Update `AppState`:
   ```rust
   pub struct AppState {
       pub snapshot_storage: Arc<dyn crate::storage::snapshot_storage::SnapshotStorage>,
       pub world: Arc<WorldCard>,
       pub map: Arc<MapDef>,
       pub player: Arc<PlayerCard>,
       pub npcs: Arc<HashMap<String, NpcCard>>,
       pub game_service: Arc<dyn GameService>,
       pub settings: Arc<RwLock<AppSettings>>,
       pub cancel_token: CancellationToken,
   }
   ```
2. Replace `lock_state()` with `load_state()`:
   ```rust
   impl AppState {
       pub fn load_state(&self) -> crate::error::Result<GameState> {
           let snapshot = self.snapshot_storage.load_latest(None)?;
           match snapshot {
               Some(snap) => {
                   let mut state = GameState::from_snapshot(
                       snap,
                       Arc::clone(&self.world),
                       Arc::clone(&self.map),
                       Arc::clone(&self.player),
                       (*self.npcs).clone(),
                   );
                   Ok(state)
               }
               None => {
                   // No snapshots yet — create initial state
                   Ok(GameState::new(
                       Arc::clone(&self.world),
                       Arc::clone(&self.map),
                       Arc::clone(&self.player),
                       (*self.npcs).values().cloned().collect(),
                       self.map.overworld.regions[0].rooms[0].id.clone(), // or from manifest
                   ))
               }
           }
       }
   }
   ```
3. Update `GameService` trait:
   ```rust
   pub trait GameService: Send + Sync {
       fn execute_action(&self, app_state: &AppState, input: String, player_name: String);
       fn retry_last_response(&self, app_state: &AppState);
   }
   ```
   Note: Using `&AppState` instead of `Arc<Mutex<GameState>>` gives the service access to storage + cached world data.
4. Update `DefaultGameService::execute_action`:
   - Load state via `app_state.load_state()`
   - For `FreeAction`: clone world data, drop state lock, run LLM, build `TurnResult`, save snapshot
   - For sync actions: load state, mutate, save snapshot
5. Update all fragment renderers in `server/fragments.rs`:
   - Replace `state.lock_state()?` with `state.load_state()?`
   - Note: `load_state()` returns `GameState` (owned), not `MutexGuard`. Renderers that only read can use it directly.
6. Update `create_app_for_testing` and `create_app_for_testing_with_settings`:
   - Construct `AppState` with `InMemorySnapshotStorage` (HashMap-backed for tests)
   - Pass cached world data

**Files:**
- `src/server/mod.rs`
- `src/engine/game_service.rs`
- `src/server/fragments.rs`
- `src/server/debug.rs` (if it uses `lock_state`)

**Acceptance Criteria:**
- [ ] All server handlers compile
- [ ] No `std::sync::Mutex<GameState>` in `AppState`
- [ ] `load_state()` returns a `GameState` from the latest snapshot
- [ ] Tests pass with in-memory snapshot storage

---

## Task 1.7: Regeneration Support

**Goal:** Retry/regenerate creates a new snapshot (swipe) without destroying the original.

### Context from code review

Current retry (`src/engine/game_service.rs:307`):
```rust
fn retry_last_response(&self, state: Arc<Mutex<GameState>>) {
    // ... locks state, gets input text, gets history_for_retry ...
    // ... calls backend.narrate_action() ...
    // ... calls state.replace_last_ai_response(new_narration) ...
}
```

`replace_last_ai_response` (`src/model/state.rs:283`) replaces the text of the last AI response in-place. With snapshots, we need to re-run the entire turn.

### Steps
1. In `retry_last_response`:
   - Load the **committed** snapshot from before the target message
   - Get the input text from that snapshot's history
   - Build `GameState` from the committed snapshot + cached world data
   - Re-run the full `FreeAction` pipeline (narrate + quantify + apply)
   - Save result as new snapshot with `swipe_index = previous_swipe_index + 1`
2. In `server/fragments.rs`:
   - The retry handler continues to work as before (returns "Retrying..." immediately)
   - No swipe UI in Phase 1 — the display always shows the latest snapshot for a message
   - `swipe_index` is stored for future UI; `load_by_message` exists on the trait for when that UI is built
3. Handle edge case: if no earlier committed snapshot exists (first message), use initial `GameState`.

**Files:**
- `src/engine/game_service.rs`
- `src/server/fragments.rs`

**Acceptance Criteria:**
- [ ] Regenerating a message creates a new snapshot with incremented `swipe_index`
- [ ] Original snapshot remains queryable via `load_by_message`
- [ ] No SQLite constraint errors on upsert

---

## Task 1.8: Remove GeneratingGuard and Mutex

**Goal:** Eliminate `GeneratingGuard` and all mutex poison handling.

### Context from code review

`GeneratingGuard` (`src/model/state.rs:310`):
```rust
pub struct GeneratingGuard {
    state: Arc<std::sync::Mutex<GameState>>,
}
```

Used in `bootstrap.rs:225`:
```rust
let _handle = runtime.spawn_blocking(move || {
    let _guard = GeneratingGuard::new(state_for_task.clone());
    // ... arrival narration ...
});
```

With snapshots, there is no shared mutable state to guard. Generation status lives in the snapshot's `NarrativeState.generation` field.

### Steps
1. Remove `GeneratingGuard` struct and `with_lock_or_recover` function from `model/state.rs`
2. In `bootstrap.rs`:
   - Replace `GeneratingGuard::new(state_for_task.clone())` with explicit status setting:
     ```rust
     if let Ok(mut state) = state_for_task.lock() {
         state.narrative.generation.status = GenerationStatus::Generating;
     }
     // ... after narration ...
     if let Ok(mut state) = state_for_task.lock() {
         state.narrative.generation.status = GenerationStatus::Idle;
     }
     ```
   - Note: during bootstrap, we still use the mutex because the server hasn't started yet. The arrival narration runs before `run_server_with_config`. We can keep the mutex just for bootstrap, or refactor bootstrap to use snapshots too.
   - **Decision:** Keep bootstrap using in-memory `GameState` for now. The mutex is only held during bootstrap, not during server operation. Save the final bootstrap state as the first snapshot before starting the server.
3. Remove `set_phase`, `reset_generating`, `set_error_and_reset` helper functions from `game_service.rs` (they all lock the mutex). Replace with direct snapshot updates.
4. Update `game_service.rs`:
   - `execute_action` no longer needs `with_state_lock` closures
   - Status changes are done by loading snapshot, mutating `narrative.generation`, saving snapshot

**Files:**
- `src/model/state.rs`
- `src/bootstrap.rs`
- `src/engine/game_service.rs`

**Acceptance Criteria:**
- [ ] `GeneratingGuard` does not exist in codebase
- [ ] `with_lock_or_recover` does not exist in codebase
- [ ] `std::sync::Mutex` is only used in `bootstrap.rs` (not in server code)
- [ ] Generation status is set by loading/saving snapshots

---

## Task 1.9: Reset Game Endpoint

**Goal:** `POST /reset` clears all state and returns to world start.

### Steps
1. Add route in `server/mod.rs`:
   ```rust
   .route("/reset", post(fragments::reset_handler))
   ```
2. Implement `reset_handler` in `server/fragments.rs`:
   ```rust
   pub async fn reset_handler(State(state): State<AppState>) -> Html<String> {
       // Cancel any in-progress generation
       state.cancel_token.cancel();
       
       // Clear all snapshots
       if let Err(e) = state.snapshot_storage.reset() {
           log::error!("Reset failed: {e}");
           return Html(render_error(&e.to_string()));
       }
       
       // Create new initial state and save as snapshot
       let initial_state = GameState::new(
           Arc::clone(&state.world),
           Arc::clone(&state.map),
           Arc::clone(&state.player),
           (*state.npcs).values().cloned().collect(),
           // starting room from world manifest — need to cache this in AppState
           state.starting_room_id.clone(),
       );
       
       let snapshot = GameStateSnapshot::from_game_state(
           &initial_state,
           "initial".to_string(),
           0,
       );
       snapshot.committed = true;
       let _ = state.snapshot_storage.save(&snapshot);
       
       // Return HTMX fragments
       let header = render_header(&state).unwrap_or_default();
       let story = render_story_log(&state).unwrap_or_default();
       let sidebar = render_visual_sidebar(&state).unwrap_or_default();
       let action = render_action_area(&state).unwrap_or_default();
       
       Html(format!("{header}{story}{sidebar}{action}"))
   }
   ```
3. Add `starting_room_id: String` to `AppState` (needed for reset reconstruction).
4. Add reset button to dashboard:
   - Add to `assets/index.html` or template:
     ```html
     <button hx-post="/reset" hx-confirm="This will erase your current game. Continue?"
             hx-target="#app-container" hx-swap="innerHTML">
       Reset Game
     </button>
     ```

**Files:**
- `src/server/mod.rs`
- `src/server/fragments.rs`
- `src/server/mod.rs` (AppState)
- `assets/index.html` or dashboard template

**Acceptance Criteria:**
- [ ] Reset clears all rows from `game_state_snapshots`
- [ ] Player returns to starting room
- [ ] NPC encounter tracking resets (`times_met` back to initial values)
- [ ] Story log is empty
- [ ] Reset works even during generation (cancels LLM call)
- [ ] Confirmation dialog prevents accidental reset

---

## Task 1.10: Test Infrastructure Update

**Goal:** Test helpers work with snapshot-based state.

### Context from code review

`TestGameState` (`src/test_support/fixtures.rs:227`) constructs raw `GameState` directly:
```rust
pub fn in_room(room_id: &str) -> GameState {
    GameState::new(
        Arc::new(TestWorld::minimal()),
        Arc::new(TestMap::single_room(room_id)),
        Arc::new(TestPlayer::standard()),
        vec![],
        room_id.to_string(),
    )
}
```

Integration tests use `create_app_for_testing(state)` which wraps it in `Arc<Mutex<GameState>>`.

### Steps
1. Create `InMemorySnapshotStorage` for tests:
   ```rust
   pub struct InMemorySnapshotStorage {
       snapshots: Mutex<Vec<GameStateSnapshot>>,
   }

   impl SnapshotStorage for InMemorySnapshotStorage {
       // ... HashMap/Vec-backed implementation ...
   }
   ```
2. Update `create_app_for_testing`:
   ```rust
   pub fn create_app_for_testing(
       state: GameState,  // not Arc<Mutex<GameState>>
   ) -> Router {
       let snapshot = GameStateSnapshot::from_game_state(&state, "test".to_string(), 0);
       let storage = Arc::new(InMemorySnapshotStorage::new()) as Arc<dyn SnapshotStorage>;
       storage.save(&snapshot).unwrap();
       
       let app_state = AppState {
           snapshot_storage: storage,
           world: state.world.clone(),
           map: state.map.clone(),
           player: state.player.clone(),
           npcs: Arc::new(state.npcs.clone()),
           starting_room_id: state.movement.current_room_id.clone(),
           game_service: Arc::new(DefaultGameService::new()) as Arc<dyn GameService>,
           settings: Arc::new(RwLock::new(AppSettings::default())),
           cancel_token: CancellationToken::new(),
       };
       build_router(app_state)
   }
   ```
3. Update all integration tests that construct `AppState` or call `create_app_for_testing`.
4. Add snapshot-specific tests:
   - Round-trip property test
   - Upsert behaviour test
   - Reset behaviour test

**Files:**
- `src/test_support/mod.rs` (add in-memory storage)
- `src/server/mod.rs` (`create_app_for_testing`)
- `tests/` (update callers)
- `src/model/state_snapshot_tests.rs` (new)

**Acceptance Criteria:**
- [ ] All integration tests pass
- [ ] `create_app_for_testing` uses `InMemorySnapshotStorage`
- [ ] Property test: 100 random snapshot round-trips
- [ ] No test constructs `Arc<Mutex<GameState>>`

---

## Dependencies

| Task | Depends On | Blocks |
|------|-----------|--------|
| 1.1 Dependencies | — | 1.2 |
| 1.2 DB module | 1.1 | 1.3, 1.4 |
| 1.3 Snapshot types | — | 1.4, 1.5 |
| 1.4 Storage impl | 1.2, 1.3 | 1.5, 1.6, 1.7, 1.10 |
| 1.5 Stateless action | 1.3, 1.4 | 1.6, 1.7 |
| 1.6 Update AppState | 1.4, 1.5 | 1.7, 1.9, 1.10 |
| 1.7 Regeneration | 1.4, 1.5, 1.6 | — |
| 1.8 Remove Guard | 1.5, 1.6 | 1.9 |
| 1.9 Reset endpoint | 1.6, 1.8 | — |
| 1.10 Test infra | 1.4, 1.6 | — |

**Parallelisable:** Tasks 1.1, 1.2, 1.3 can start immediately. Task 1.4 needs 1.2 and 1.3. Tasks 1.5–1.10 form a chain.

---

## Risks

| Risk | Mitigation |
|------|-----------|
| `rusqlite` compilation fails on Windows | `bundled` feature includes SQLite; test early |
| Snapshot JSON is large/slow | Only 4 sub-structs are serialised; `movement` and `scene` are small; `narrative` is bounded by `MAX_LOG_ENTRIES = 1000` |
| Bootstrap mutex still needed | Acceptable — mutex is only during startup, not server runtime |
| `load_state()` called frequently | Cache latest snapshot in `AppState` with `RwLock<Option<GameStateSnapshot>>`; invalidate on save |
| Test fixtures broken | `TestGameState` still constructs `GameState`; tests just need to save it to storage before use |

---

## Success Criteria

1. `python build.py` passes (fmt + clippy + guardrails + tests).
2. `cargo test --features diagnostics` passes with no regressions.
3. Every turn creates a queryable SQLite row containing the 4 mutable sub-structs.
4. Regeneration creates a new `swipe_index` row without modifying the original (data-layer only; no swipe UI yet).
5. Reset returns the game to initial state without server restart.
6. No `Mutex<GameState>` or `GeneratingGuard` in server runtime code (outside `bootstrap.rs`).
7. `execute_freeaction_impl` signature uses `&GameState` (not `&mut`).
8. No `.ok()` swallow patterns remain in `action_processing.rs`.
9. Test-to-code ratio remains ≥ 1.5 (matching overarching spec; current ~2.04).

---

## Verification

### Automated Tests

```bash
cd chronicler_engine
python build.py
```

`python build.py` is the gate: it runs fmt, clippy, guardrails, and all tests. The following must be covered by the test suite:

| Behaviour | Test Level | How |
|-----------|-----------|-----|
| Turn creates snapshot row | Integration | Mock backend + `InMemorySnapshotStorage` — assert save called with correct sub-structs |
| Snapshot round-trip | Unit + Property | `GameStateSnapshot::from_game_state` → `apply_to` → fields match |
| SQLite upsert | Unit | Save same `(message_id, swipe_index)` twice — assert single row with latest data |
| `load_latest(None)` returns most recent | Unit | Save 3 snapshots — assert loaded is the last |
| `commit` sets flag | Unit | Save → commit → load — assert `committed = true` |
| `reset` empties table | Unit | Save → reset → load_latest — assert `None` |
| Regeneration increments `swipe_index` | Integration | Mock backend — call retry, assert 2 rows for same `message_id` with `swipe_index` 0 and 1 |
| `execute_freeaction_impl` immutability | Unit | Assert signature is `&GameState`, compile-time check |
| `.ok()` swallow removed | Unit + grep | Tests pass + `grep "\.ok()" src/engine/action_processing.rs` returns nothing |
| No `Mutex<GameState>` in server | Compile-time + grep | `grep -rn "Mutex<GameState>" src/server/` returns nothing |
| No `GeneratingGuard` outside bootstrap | Compile-time + grep | `grep -rn "GeneratingGuard" src/` only matches `bootstrap.rs` |
| Reset endpoint clears state | Integration | Post to `/reset` — assert `load_latest` returns initial state |
| Reset during generation | Integration | Start async action, post `/reset` mid-flight — assert cancellation + clean state |
| Fragment renders after refactor | E2E (Playwright) | `with_test_page` — navigate tabs, assert no panics |

### Spot-Checks

Run these manually only if `python build.py` passes but something feels wrong:

```bash
# Verify .ok() swallow is gone from action processing
grep -n "\.ok()" src/engine/action_processing.rs
# Expected: no matches (or only legitimate Option uses)

# Verify no Mutex<GameState> in server code
grep -rn "Mutex<GameState>" src/server/
# Expected: no matches

# Verify GeneratingGuard is gone (except bootstrap)
grep -rn "GeneratingGuard" src/
# Expected: only in src/bootstrap.rs
```

### AI Agent Browser Verification (Optional)

If the automated suite passes but visual confirmation is desired, the AI agent can launch the server and verify via browser using the existing Playwright e2e infrastructure (`with_test_page`). No manual human steps are required.
