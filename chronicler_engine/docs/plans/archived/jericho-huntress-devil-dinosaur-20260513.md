# Plan: Migrate Chronicler Engine to Turn + Swipe Model

## Context

The current architecture uses a flat `Vec<LogEntry>` for narrative history with an **emergent** turn identity encoded only in snapshot `message_id`. This causes the bug where history mutation (delete/edit) breaks retry because the correlation between a turn and its pre-generation snapshots (`pre-main:{uuid}`, `pre-event:{uuid}`) is purely conventional — easy to forget, as both `delete_history_handler` and `edit_history_handler` did.

Marinara Engine solves this with a relational model (`messages` + `message_swipes` + `game_state_snapshots` tied to `(messageId, swipeIndex)`). Chronicler can adopt the same conceptual model without requiring backwards compatibility — SQLite databases are recreated on fresh runs and there is no migration requirement for existing save data.

This plan does not reduce scope for minimal-fix convenience. It addresses the root architectural fragility.

---

## Architectural Decisions

1. **Keep `LogEntry` as the atomic unit.** `Turn` and `Swipe` contain `LogEntry` items. This preserves the rendering pipeline (`StoryLogTemplate`, `LogEntryView`), prompt builder history context, and HTMX `data-id` edit handlers.

2. **Turn = one player input + all its AI responses.** Every player action (sync or async) creates a `Turn`. Sync actions create a turn with a single default swipe. Async actions create a turn and may add additional swipes on retry.

3. **Swipe = one generation attempt.** Each swipe contains the log entries produced by that attempt (narration, event continuation, system messages). The `active_swipe_index` on the turn controls which swipe is rendered.

4. **Rename `message_id` → `turn_id` everywhere.** The snapshot key becomes `pre-main:{turn_id}` and `pre-event:{turn_id}`. This makes the relationship explicit.

5. **History mutation operates on turns, not individual entries.** `delete_last_turn()` removes the entire last turn and cascades to delete its snapshots. `edit_turn_input()` edits the input text of a specific turn.

6. **Render via derived `history()` view.** All existing consumers (templates, prompt builder, status checks) continue to receive `Vec<LogEntry>` via `NarrativeState::history()`.

---

## Target Domain Model

### New: `src/model/turn.rs`

```rust
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::model::state::LogEntry;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Turn {
    pub id: String,                    // UUID — stable turn identity
    pub input: LogEntry,               // LogType::Input
    pub swipes: Vec<Swipe>,            // All generation attempts
    pub active_swipe_index: u32,       // Which swipe is displayed
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Swipe {
    pub index: u32,
    pub entries: Vec<LogEntry>,        // Narration, Dialogue, System
}

impl Turn {
    pub fn new(input: LogEntry) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            input,
            swipes: vec![Swipe { index: 0, entries: Vec::new() }],
            active_swipe_index: 0,
            created_at: Utc::now(),
        }
    }

    pub fn active_swipe(&self) -> Option<&Swipe> {
        self.swipes.get(self.active_swipe_index as usize)
    }

    pub fn active_swipe_mut(&mut self) -> Option<&mut Swipe> {
        self.swipes.get_mut(self.active_swipe_index as usize)
    }

    pub fn flattened_entries(&self) -> Vec<LogEntry> {
        let mut entries = vec![self.input.clone()];
        if let Some(swipe) = self.active_swipe() {
            entries.extend(swipe.entries.clone());
        }
        entries
    }
}
```

### Refactored: `NarrativeState` in `src/model/state.rs`

```rust
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct NarrativeState {
    pub turns: Vec<Turn>,
    pub next_log_id: u64,
    pub generation: GenerationState,
    pub last_trigger: Option<StoredTriggerContext>,
    pub pending_location: Option<String>,
    pub pending_event: Option<String>,
}
```

`history()` is a derived method that flattens all turns into the legacy `Vec<LogEntry>` view:

```rust
impl NarrativeState {
    pub fn history(&self) -> Vec<LogEntry> {
        self.turns.iter()
            .flat_map(|turn| turn.flattened_entries())
            .collect()
    }
}
```

This is the **only** view used by:
- `render_story_log` → `StoryLogTemplate::new(&entries)`
- `PromptBuilder` → `history: &[LogEntry]`
- `get_history_context_for_retry()`
- `get_last_ai_response_index()`
- `is_last_ai_response_event_continuation()`

### Renamed: `GameStateSnapshot` in `src/model/state_snapshot.rs`

```rust
pub struct GameStateSnapshot {
    pub id: String,
    pub turn_id: String,        // Was: message_id
    pub swipe_index: u32,
    pub movement: MovementState,
    pub narrative: NarrativeState,
    pub scene: SceneState,
    pub character_state: CharacterState,
    pub committed: bool,
    pub created_at: DateTime<Utc>,
}
```

Pre-generation anchors:
- `pre-main:{turn_id}`
- `pre-event:{turn_id}`

### New: `Checkpoint` in `src/model/checkpoint.rs`

```rust
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Checkpoint {
    pub id: String,
    pub turn_id: String,
    pub swipe_index: u32,
    pub name: String,
    pub created_at: DateTime<Utc>,
}
```

Stored in a dedicated SQLite `checkpoints` table (not in `NarrativeState` JSON, to avoid duplicating checkpoint data across every snapshot row).

---

## Task List

### Phase 1: Domain Model
**Scope:** Define `Turn`, `Swipe`, `Checkpoint`. Refactor `NarrativeState` to use `Vec<Turn>` instead of `Vec<LogEntry>`.

**Files:**
- `src/model/turn.rs` — **new**
- `src/model/checkpoint.rs` — **new**
- `src/model/state.rs` — **refactor** `NarrativeState`
- `src/model/mod.rs` — **update** exports

**Work:**
- Define `Turn`, `Swipe` structs with `flattened_entries()` method
- Refactor `NarrativeState`: replace `history: Vec<LogEntry>` with `turns: Vec<Turn>`
- Add `history()` derived method returning `Vec<LogEntry>`
- Update `add_log()` → add to active swipe of last turn
- Add `new_turn(input: LogEntry)` method
- Add `delete_last_turn()` method (pops last turn, returns turn ID for cascade)
- Add `edit_turn_input(turn_id: &str, new_text: String)` or keep `edit_log(id: u64)` by finding the entry in turns
- Update `get_last_ai_response_index()`, `is_last_ai_response_event_continuation()`, `get_history_context_for_retry()`, `replace_last_ai_response()` to operate on `history()` derived view
- Preserve `next_log_id` counter behavior
- Add `Checkpoint` struct

**Acceptance criteria:**
- [ ] `NarrativeState::history()` returns correct `Vec<LogEntry>` for multi-turn, multi-swipe scenarios
- [ ] `add_log` appends to the active swipe of the last turn
- [ ] `delete_last_turn` removes the entire turn
- [ ] `edit_log` still works by ID lookup across turns/swipes
- [ ] Unit tests for turn flattening, add/delete, swipe switching

**Verification:**
- [ ] `cargo test --lib model::state` passes
- [ ] `cargo test --lib model::turn` passes

---

### Phase 2: Snapshot Alignment
**Scope:** Rename `message_id` → `turn_id` across the snapshot system. Add turn-scoped delete. Update storage trait and both implementations.

**Files:**
- `src/model/state_snapshot.rs` — **rename field**
- `src/storage/snapshot_storage.rs` — **rename methods**, add `delete_turn_snapshots`
- `src/storage/db.rs` — **schema update**
- `src/test_support/in_memory_storage.rs` — **update** implementation

**Work:**
- Rename `message_id` → `turn_id` in `GameStateSnapshot`
- Rename `load_by_message()` → `load_by_turn()` in trait
- Rename `load_latest(message_id)` parameter → `turn_id`
- Add `delete_turn_snapshots(turn_id: &str)` to trait
- Update SQLite schema: `turn_id TEXT NOT NULL` (replace `message_id`)
- Implement `delete_turn_snapshots` in `SqliteSnapshotStorage` (deletes all rows where `turn_id = ? OR turn_id LIKE 'pre-main:%' OR turn_id LIKE 'pre-event:%'` — or simpler: parse prefix)
- Actually simpler: store `base_turn_id` separately? No, keep prefixed turn_id in `turn_id` column. `delete_turn_snapshots` does: `DELETE FROM game_state_snapshots WHERE turn_id = ?1 OR turn_id = ?2 OR turn_id = ?3` with `turn_id`, `pre-main:turn_id`, `pre-event:turn_id`.
- Update `InMemorySnapshotStorage` to match

**Acceptance criteria:**
- [ ] All storage CRUD tests pass with renamed fields
- [ ] `delete_turn_snapshots` removes pre-main, pre-event, and final snapshots for a turn
- [ ] Upsert behavior preserved (`ON CONFLICT(turn_id, swipe_index)`)

**Verification:**
- [ ] `cargo test snapshot_storage_tests` passes
- [ ] `cargo test state_snapshot_tests` passes

---

### Phase 3: Generation Pipeline
**Scope:** Update `execute_action_impl` and `execute_freeaction_pipeline` to create turns and save snapshots with `turn_id`.

**Files:**
- `src/engine/game_service/actions.rs` — **refactor** turn creation
- `src/engine/game_service/helpers.rs` — **rename** parameters
- `src/server/fragments/actions.rs` — **update** sync action snapshot save

**Work:**
- In `process_action` / `execute_action_impl`:
  - After adding input log, call `state.narrative.new_turn(input_entry)` (or the turn is created when input is added)
  - For sync actions: add response entries to the current turn's active swipe, save snapshot with `turn_id = turn.id`, `swipe_index = 0`
  - For async actions: pass `turn.id` as `turn_id` to `execute_freeaction_pipeline`
- In `execute_freeaction_pipeline`:
  - Save `pre-main:{turn_id}` before LLM call
  - Add main narration to `state.narrative.last_turn().active_swipe().entries`
  - If trigger fires: save `pre-event:{turn_id}`, add event continuation to same swipe
  - Save final state with `turn_id`, `swipe_index`
- Update `save_state` and `save_committed_state` helpers to use `turn_id` parameter name

**Acceptance criteria:**
- [ ] Sync actions create a turn and save snapshot with `turn_id = turn.id`
- [ ] Async pipeline saves `pre-main:{turn_id}` and `pre-event:{turn_id}`
- [ ] All entries from a generation attempt land in the same swipe

**Verification:**
- [ ] `cargo test game_service::basic` passes
- [ ] `cargo test game_service::advanced` passes

---

### Phase 4: Retry Mechanism
**Scope:** Update retry to operate on turns: increment swipe index, create new swipe, re-run pipeline.

**Files:**
- `src/engine/game_service/retry.rs` — **refactor**

**Work:**
- `retry_last_response_impl`:
  - Load latest snapshot → extract `turn_id`
  - Find the turn by ID in current state
  - Check if last AI response is event continuation (via `history()` derived view — unchanged logic)
  - `retry_main_narration`: load `pre-main:{turn_id}`, create new swipe on the turn with `index = current_swipe + 1`, set as active, run full pipeline
  - `retry_event_continuation`: load `pre-event:{turn_id}`, create new swipe with `index = current_swipe + 1`, copy main narration from previous swipe, regenerate event
- Update `save_retry_error` to save with correct `turn_id`

**Acceptance criteria:**
- [ ] Retry creates a new `Swipe` on the same `Turn`
- [ ] `swipe_index` increments correctly
- [ ] Event retry copies main narration from previous swipe
- [ ] All existing retry behavior preserved

**Verification:**
- [ ] `cargo test flow_mock::retry_main` passes
- [ ] `cargo test flow_mock::retry_event` passes

---

### Phase 5: History Mutation (The Fix)
**Scope:** Replace entry-level mutation with turn-level mutation. Fix handlers to preserve turn identity and cascade delete snapshots.

**Files:**
- `src/model/state.rs` — **add** `delete_last_turn()`, update `edit_log()`
- `src/server/fragments/history.rs` — **refactor** handlers

**Work:**
- `delete_last_turn()`:
  - Pop last turn from `turns`
  - Return the turn's `id` so caller can cascade-delete snapshots
- `edit_log(id, new_text)`:
  - Find the log entry across all turns/swipes by ID
  - If it's an input entry, edit it
  - If it's a narration/dialogue entry, edit it (preserve existing behavior)
- `delete_history_handler`:
  - Load latest snapshot to get current `turn_id` before mutation
  - Call `delete_last_turn()` → get removed turn ID
  - Call `snapshot_storage.delete_turn_snapshots(&turn_id)`
  - Save new snapshot with preserved `turn_id` from the *new* latest turn (or new UUID if no turns remain)
- `edit_history_handler`:
  - Load latest snapshot to get `turn_id`
  - Call `edit_log(id, text)`
  - Save snapshot with same `turn_id`, same `swipe_index`

**Acceptance criteria:**
- [ ] Delete removes entire turn + all its snapshots
- [ ] Edit preserves turn ID and swipe index
- [ ] Retry works after delete or edit

**Verification:**
- [ ] New regression test: execute with trigger → delete event → retry main succeeds
- [ ] New regression test: delete narration → retry with new UUID simulation fails before fix, succeeds after
- [ ] `cargo test flow_mock::sequence` passes

---

### Phase 6: Checkpoints
**Scope:** Add bookmark system for saving and restoring specific turn+swipe combinations.

**Files:**
- `src/model/checkpoint.rs` — **new**
- `src/storage/db.rs` — **update** Add `checkpoints` table
- `src/storage/snapshot_storage.rs` — **update** Add checkpoint CRUD methods
- `src/test_support/in_memory_storage.rs` — **update** Implement checkpoint methods
- `src/server/fragments/checkpoint.rs` — **new** handlers
- `src/server/fragments/mod.rs` — **register** routes

**Work:**
- Add `Checkpoint` struct
- Add checkpoint CRUD methods to `SnapshotStorage` trait:
  - `save_checkpoint(&self, checkpoint: &Checkpoint) -> Result<(), EngineError>`
  - `load_checkpoint(&self, id: &str) -> Result<Option<Checkpoint>, EngineError>`
  - `list_checkpoints(&self) -> Result<Vec<Checkpoint>, EngineError>`
  - `delete_checkpoint(&self, id: &str) -> Result<(), EngineError>`
- Implement checkpoint storage in `SqliteSnapshotStorage` (new `checkpoints` table)
- Implement checkpoint storage in `InMemorySnapshotStorage`
- Add `create_checkpoint(name: String) -> Checkpoint` helper to `GameState`
- Add `restore_checkpoint(checkpoint_id: &str)` method:
  - Load checkpoint from storage
  - Load snapshot by `(turn_id, swipe_index)`
  - Set that turn's `active_swipe_index` to the checkpoint's swipe
  - Save state
- Add routes:
  - `POST /checkpoint` — create checkpoint at current turn+swipe
  - `POST /checkpoint/:id/restore` — restore checkpoint
  - `GET /fragment/checkpoints` — list checkpoints fragment
  - `POST /checkpoint/:id/delete` — delete checkpoint

**Acceptance criteria:**
- [ ] Checkpoints save turn + swipe reference
- [ ] Restoring switches active swipe and re-saves state
- [ ] Checkpoints survive page reload

**Verification:**
- [ ] Component tests for checkpoint create/restore

---

### Phase 7: UI Rendering & Swipe Navigation
**Scope:** Update templates to show swipe navigation and checkpoint controls.

**Files:**
- `src/server/templates.rs` — **update** `StoryLogTemplate`
- `src/server/fragments/renderers.rs` — **update** if needed
- `assets/index.html` — **add** swipe nav JS, checkpoint buttons

**Work:**
- `StoryLogTemplate` already consumes `Vec<LogEntry>` via `history()` — no change needed
- Add swipe navigation UI to action area or story log:
  - Left/right arrows when `turn.swipes.len() > 1`
  - Swipe counter: "2 / 5"
  - `POST /turn/:id/swipe/:index` to switch active swipe
- Add checkpoint button to action area

**Acceptance criteria:**
- [ ] Swipe navigation appears when multiple swipes exist
- [ ] Switching swipe updates rendered history without regeneration
- [ ] Checkpoint button creates bookmark

**Verification:**
- [ ] Browser/component tests render correctly
- [ ] Manual check: multi-swipe turn shows navigation

---

### Phase 8: Documentation
**Scope:** Update all architecture docs to reflect the new model. This is spec-driven development — docs are updated as part of the plan, not after.

**Files:**
- `docs/architecture/system.md` — **update** History Management, Snapshot Storage, GameState sections
- `docs/system/game_flow.md` — **update** Retry Flow diagram, add Turn/Swipe flow
- `docs/adr/adr-009-turn-swipe-model.md` — **new** ADR
- `chronicler_engine/TODO.md` — **remove** resolved item

**Work:**
- Update `docs/architecture/system.md`:
  - Replace `LogEntry` flat history description with `Turn` + `Swipe` model
  - Document `NarrativeState::history()` derived view
  - Update snapshot key names (`turn_id` instead of `message_id`)
  - Document `delete_turn_snapshots` cascade behavior
- Update `docs/system/game_flow.md`:
  - Update Retry Flow Mermaid diagram to show swipe creation
  - Add Turn Lifecycle diagram (input → swipe 0 → retry → swipe 1)
  - Document checkpoint flow
- New ADR `adr-009-turn-swipe-model.md`:
  - Context: emergent turn identity caused retry breakage
  - Decision: first-class Turn and Swipe domain objects
  - Consequences: structural turn identity, swipe browser, checkpoint support
- Remove TODO item: "We should be able to retry to first LLM message..."

**Acceptance criteria:**
- [ ] Architecture docs accurately describe the new model
- [ ] Game flow docs show updated diagrams
- [ ] ADR explains rationale and trade-offs

**Verification:**
- [ ] Human review: docs match implementation

---

## Testing Strategy

1. **Unit tests** (`src/model/turn.rs`, `src/model/state.rs`):
   - Turn creation, flattening, swipe management
   - `history()` derived view correctness
   - `delete_last_turn`, `edit_log` across turns/swipes

2. **Storage tests** (`tests/snapshot_storage_tests.rs`):
   - CRUD with `turn_id`
   - `delete_turn_snapshots` cascade
   - Upsert behavior
   - Commit flag

3. **Flow mock tests** (`tests/flow_mock/`):
   - Update ALL existing tests to use turn-based API
   - `retry_main.rs`: add `test_delete_event_then_retry_main_regenerates`
   - `retry_main.rs`: add `test_delete_narration_then_retry_with_new_uuid_fails` (simulates old bug)
   - `retry_event.rs`: add `test_retry_creates_new_swipe`
   - `sequence.rs`: update delete tests for turn semantics

4. **Component tests** (`tests/components/`):
   - History edit/delete endpoints preserve turn ID
   - Checkpoint create/restore endpoints

5. **Full validation**:
   - `python build.py` (fmt + clippy + tests + coverage)

---

## Complete File List

| File | Change |
|------|--------|
| `src/model/turn.rs` | **New** — `Turn`, `Swipe` structs |
| `src/model/checkpoint.rs` | **New** — `Checkpoint` struct |
| `src/model/state.rs` | **Refactor** — `NarrativeState` uses `Vec<Turn>`, `history()` view |
| `src/model/mod.rs` | **Update** — Export new modules |
| `src/model/state_snapshot.rs` | **Rename** — `message_id` → `turn_id` |
| `src/storage/snapshot_storage.rs` | **Rename** — Methods use `turn_id`; add `delete_turn_snapshots` |
| `src/storage/db.rs` | **Update** — Schema: `turn_id` column |
| `src/test_support/in_memory_storage.rs` | **Update** — Match trait changes |
| `src/engine/game_service/helpers.rs` | **Rename** — Parameters `turn_id` |
| `src/engine/game_service/actions.rs` | **Refactor** — Create turns, operate on swipes |
| `src/engine/game_service/retry.rs` | **Refactor** — Create swipes on retry |
| `src/server/fragments/history.rs` | **Refactor** — Turn-level mutation, cascade delete |
| `src/server/fragments/actions.rs` | **Update** — Sync action snapshot save |
| `src/server/fragments/checkpoint.rs` | **New** — Checkpoint handlers |
| `src/server/fragments/mod.rs` | **Update** — Register checkpoint routes |
| `src/server/templates.rs` | **Update** — Swipe nav UI (if template changes needed) |
| `tests/flow_mock.rs` | **Update** — Test utilities |
| `tests/flow_mock/retry_main.rs` | **Update** — New regression tests |
| `tests/flow_mock/retry_event.rs` | **Update** — Swipe tests |
| `tests/flow_mock/sequence.rs` | **Update** — Turn-based sequence tests |
| `tests/snapshot_storage_tests.rs` | **Update** — `turn_id` tests |
| `docs/architecture/system.md` | **Update** — Document Turn + Swipe model |
| `docs/system/game_flow.md` | **Update** — Updated flow diagrams |
| `docs/adr/adr-009-turn-swipe-model.md` | **New** — ADR |
| `chronicler_engine/TODO.md` | **Update** — Remove resolved item |

---

## Risks and Mitigations

| Risk | Impact | Mitigation |
|------|--------|------------|
| `NarrativeState` JSON shape change breaks existing DBs | High | Acceptable — no migration required per project conventions; DBs are recreated on fresh runs |
| Missed `message_id` reference somewhere in src/ | Medium | Global find-replace + compiler catches remaining references |
| `history()` performance with many turns/swipes | Low | `MAX_LOG_DISPLAY = 50` limits render; prompt builder already truncates history |
| Tests manually construct `NarrativeState` with `history` field | High | Update all test constructors; compiler will flag struct literal mismatches |
| Template `data-id` edit handlers break if ID lookup changes | Medium | `edit_log(id)` preserves ID-based lookup; template uses `history()` which preserves IDs |
| Swipe navigation complicates HTMX polling | Low | Swipe switch is explicit user action (POST), not a polling concern |

---

## Open Questions

1. **Should sync actions support retry?** Currently they don't (no pre-main snapshot). With the turn model, we could save `pre-main:{turn_id}` for sync actions too, enabling retry. This is a feature decision, not a bug fix. Default: keep current behavior (sync actions not retryable).

2. **Should `edit_log` allow editing narrations, or only inputs?** Currently any entry is editable. With swipes, editing a narration means editing one swipe but not others. Simpler to preserve existing behavior: any entry editable by ID.

3. **Checkpoint storage:** ~~In `NarrativeState` JSON~~ → **SQLite table**. Storing checkpoints in `NarrativeState` would duplicate them across every snapshot row. A dedicated `checkpoints` table is normalized, queryable, and consistent with the rest of the storage layer. Add checkpoint CRUD to `SnapshotStorage` trait.
