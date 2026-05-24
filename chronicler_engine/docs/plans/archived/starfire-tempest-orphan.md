# Plan: Message Swipes for Chronicler Engine

## Goal
Replace destructive retry with swipe-aware generation. Each AI-generated message (narration or event) gets its own swipe set stored in a dedicated table. Old generations are preserved as swipes with their snapshot IDs, so switching swipes restores the exact world state that produced that text. Events are kept independent of narration swipes and can be retriggered from a restored snapshot.

## Success Criteria
- [ ] Retry on the last message preserves its old text + snapshot as a swipe, then reruns generation.
- [ ] Swipe navigation (left/right arrows + counter) appears on the **last message only** when swipes exist.
- [ ] Switching to a swipe restores that swipe's snapshot. No history truncation is needed because swiping is only allowed on the last message.
- [ ] If a restored narration snapshot contains `last_trigger` and the narration is the last message, the UI shows a **"Retrigger Event"** button that runs trigger continuation from that state.
- [ ] Event messages have their own independent swipes via event retry.
- [ ] Prompt builder (`history() → Vec<LogEntry>`) always uses the active swipe text.
- [ ] All existing tests pass; new tests cover swipe creation, snapshot restoration, and retrigger.

---

## Architecture Decision

**Per-message swipes in a dedicated `message_swipes` table, each swipe storing its own `snapshot_id`. Messages use soft deletes so retry can recover on pipeline failure.**

Narration and events remain independent (no "turn" grouping):
- **Main retry** rolls back to the input snapshot, soft-deletes messages after the anchor, preserves the old narration as a swipe, and reruns the full pipeline. On success, the soft-deleted messages are hard-deleted. On failure, they are restored.
- **Event retry** rolls back to the pre-event snapshot, soft-deletes event messages, preserves the old event as a swipe, and reruns trigger continuation.
- **Swipe navigation** is only allowed on the **last message**. Restoring a swipe's snapshot rewinds world state without deleting anything after it (there is nothing after it).
- **Retrigger Event** appears on a narration swipe when its snapshot has `last_trigger` and there are no event messages after it.

This gives Marinara-style state consistency without graph snapshots or turn grouping.

---

## Data Model Changes

### `src/model/message.rs`
```rust
pub struct Swipe {
    pub text: String,
    pub snapshot_id: u64,
    pub location_header: Option<String>,
    pub event_header: Option<String>,
}

pub struct Message {
    pub id: u64,
    pub sender: Option<String>,
    pub text: String,              // active text (from active swipe)
    pub log_type: LogType,
    pub timestamp: DateTime<Utc>,
    pub location_header: Option<String>, // active location_header (from active swipe)
    pub event_header: Option<String>,    // active event_header (from active swipe)
    pub snapshot_id: Option<u64>,        // active snapshot_id (from active swipe)
    pub active_swipe_index: usize,
    pub swipes: Vec<Swipe>,
}

impl Message {
    pub fn active_text(&self) -> &str { &self.text }
    pub fn active_snapshot_id(&self) -> Option<u64> { self.snapshot_id }
}
```

> `text`, `location_header`, `event_header`, and `snapshot_id` are always the **active** swipe's values at runtime. They are hydrated from `message_swipes` by the storage layer.

### `src/model/message_history.rs`
- `to_log_entries()` uses `msg.active_text()` instead of `msg.text` (already the case since `text` is the active text).
- Add `is_last(id: u64) -> bool` helper.

### `src/model/state.rs` (`LogEntry`)
- Add `swipe_count: usize` and `active_swipe_index: usize` so the template can render swipe controls.

---

## Storage Layer Changes

### `src/storage/db.rs`
Migration:
```sql
-- Add soft-delete flag to messages
ALTER TABLE messages ADD COLUMN is_deleted INTEGER NOT NULL DEFAULT 0;

-- Remove text, snapshot_id, location_header, event_header from messages
CREATE TABLE messages_new (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    game_id INTEGER NOT NULL DEFAULT 1,
    sender TEXT,
    log_type TEXT NOT NULL,
    timestamp TEXT NOT NULL,
    active_swipe_index INTEGER NOT NULL DEFAULT 0,
    is_deleted INTEGER NOT NULL DEFAULT 0
);

INSERT INTO messages_new (id, game_id, sender, log_type, timestamp, active_swipe_index, is_deleted)
SELECT id, game_id, sender, log_type, timestamp, 0, 0 FROM messages;

DROP TABLE messages;
ALTER TABLE messages_new RENAME TO messages;

-- New swipes table
CREATE TABLE message_swipes (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    message_id INTEGER NOT NULL REFERENCES messages(id) ON DELETE CASCADE,
    swipe_index INTEGER NOT NULL,
    text TEXT NOT NULL,
    snapshot_id INTEGER NOT NULL,
    location_header TEXT,
    event_header TEXT,
    UNIQUE(message_id, swipe_index)
);

-- Migrate existing messages: create swipe 0 from old message data
INSERT INTO message_swipes (message_id, swipe_index, text, snapshot_id, location_header, event_header)
SELECT id, 0, text, snapshot_id, location_header, event_header FROM messages;
```

> `ON DELETE CASCADE` ensures swipes are cleaned up automatically when a message is hard-deleted.

### `src/storage/models/message.rs` & `src/storage/mappers/message.rs`
- `DbMessage` drops `text`, `snapshot_id`, `location_header`, `event_header`.
- Add `DbSwipe` struct.
- `load_messages()` uses a single JOIN query:
  ```sql
  SELECT m.*, s.swipe_index, s.text, s.snapshot_id, s.location_header, s.event_header
  FROM messages m
  LEFT JOIN message_swipes s ON m.id = s.message_id
  WHERE m.game_id = ? AND m.is_deleted = 0
  ORDER BY m.id, s.swipe_index
  ```
  Then group rows by `message_id` to build `Message.swipes` and hydrate active fields.

### `src/storage/message_storage.rs`
```rust
pub trait MessageStorage: Send + Sync {
    // ... existing methods ...
    fn soft_delete_message(&self, id: u64) -> Result<(), EngineError>;
    fn restore_soft_deleted(&self, ids: &[u64]) -> Result<(), EngineError>;
    fn purge_soft_deleted(&self, ids: &[u64]) -> Result<(), EngineError>;
    fn insert_swipe(&self, message_id: u64, swipe: &Swipe, index: usize) -> Result<(), EngineError>;
    fn update_active_swipe(&self, message_id: u64, index: usize) -> Result<(), EngineError>;
}
```

- `insert_message()` is a transaction: insert into `messages`, then insert swipe 0 into `message_swipes`, then set `msg.id`.
- `update_message(id, text)` updates the active swipe's `text` in `message_swipes`.
- `delete_message(id)` performs a **hard delete** (used by user-initiated delete). CASCADE cleans up swipes.

### `src/storage/snapshot_storage.rs` & `src/test_support/in_memory_storage.rs`
- Implement new methods.

---

## Retry Logic Changes

### `src/application/action_pipeline/retry.rs`

**`retry_last_response_impl` — new flow:**

1. Load all messages.
2. Determine `is_event` from the last message.
3. Find anchor index (same as today).
4. **Extract target swipes before soft-delete:**
   - `old_target = messages.last()`.
   - `pending_swipes = old_target.swipes.clone()`.
   - `pending_swipes.push(Swipe {
       text: old_target.text.clone(),
       snapshot_id: old_target.snapshot_id.unwrap_or(0),
       location_header: old_target.location_header.clone(),
       event_header: old_target.event_header.clone(),
     })`.
5. **Soft-delete** messages after anchor in DB: `soft_delete_message(id)` for each.
6. Truncate in-memory vector after anchor.
7. Load anchor snapshot, reconstruct `GameState`, hydrate truncated history.
8. Run pipeline (`retry_main_narration` or `retry_event_continuation`).
9. **After pipeline succeeds:**
   - Reload messages from DB (soft-deleted ones are hidden).
   - `new_target = messages.last()`.
   - For each swipe in `pending_swipes`, call `insert_swipe(new_target.id, &swipe, index)`.
   - Set `new_target.active_swipe_index = pending_swipes.len()`.
   - `update_active_swipe(new_target.id, new_target.active_swipe_index)`.
   - **Hard-delete** the soft-deleted old messages: `purge_soft_deleted(&to_delete_ids)`.
     - CASCADE automatically removes their old swipes (which we already copied to the new message).
10. **If pipeline fails:**
    - `restore_soft_deleted(&to_delete_ids)`.
    - Old messages and their swipes reappear unchanged.
    - Set status to Error.

### `src/application/action_pipeline/pipeline.rs`
No changes. The pipeline appends messages normally. Swipe migration happens in `retry.rs` after the pipeline returns.

---

## Swipe Navigation & Retrigger

### New endpoint: `POST /message/:id/swipe/:index`
**`src/server/fragments/misc.rs`**

Handler logic:
1. Load current state.
2. Find message `id` in history.
3. **Validate it is the last message.** Return 400 if not.
4. Validate `index < message.swipes.len()`.
5. Get `snapshot_id = message.swipes[index].snapshot_id`.
6. Load snapshot and `snapshot.apply_to(&mut state)`.
7. Update `message.active_swipe_index = index` (and hydrate active fields from the selected swipe).
8. Save state.
9. Return updated story-log fragment.

### New endpoint: `POST /retrigger`
**`src/server/fragments/misc.rs`**

Handler logic:
1. Load current state.
2. Verify `state.narrative.last_trigger.is_some()`.
3. Verify the last message is a narration (not an event).
4. Set status to `Generating` and save generating snapshot.
5. Spawn blocking task: `pipeline.run_trigger_continuation(state, trigger, &input_text)`.
6. Return "Retriggering...".

> **When is this shown?** The `StoryLogTemplate` renders a "Retrigger Event" button on the last narration message when the current `GameState` has `last_trigger` and the last message is a narration.

### `src/server/templates.rs`
- Update `LogEntryView` with `swipe_count` and `active_swipe_index`.
- Update `StoryLogTemplate`:
  - On `loop.last` and `entry.swipe_count > 0`: left arrow, counter (`active + 1` / `swipe_count + 1`), right arrow.
  - Right arrow on latest swipe calls `submitRetry()` (generates new swipe).
  - Right arrow when not on latest calls `submitSwipe({{ entry.id }}, {{ active + 1 }})`.
  - Left arrow calls `submitSwipe({{ entry.id }}, {{ active - 1 }})`.
  - On last narration when template has `show_retrigger == true`: "Retrigger Event" button calling `submitRetrigger()`.

### `assets/index.html`
- `submitSwipe(messageId, index)` — POST to `/message/:id/swipe/:index`, refresh log.
- `submitRetrigger()` — POST to `/retrigger`, refresh log.
- `submitRetry()` stays but is now triggered by the right arrow when on the latest swipe.

### `assets/styles.css`
- Styles for `.swipe-left`, `.swipe-right`, `.swipe-counter`, `.retrigger-btn`.

---

## UI Design Notes

The retry button is replaced by swipe controls on the last message:

```
[←]  2 / 3  [→]
```

- **Left arrow (`←`)**: switches to the previous swipe. Hidden when on swipe 0.
- **Counter**: `active_index + 1` / `swipe_count + 1`.
- **Right arrow (`→`)**: if not on latest swipe, switches to next swipe. If on latest swipe, triggers a new generation (same as old retry).

This matches SillyTavern and Marinara behavior exactly.

---

## Risks & Open Issues

### 1. DB migration complexity
SQLite cannot drop columns directly. We must recreate the `messages` table. Existing data must be migrated into `message_swipes` (swipe 0 for every existing message).

**Mitigation:** The project convention accepts breaking DB changes (DBs are recreated on fresh runs). However, a proper migration script is included in the plan for completeness.

### 2. `location_header` per swipe
The user correctly noted that `location_header` is message metadata. Since different swipes can result in different locations (e.g., forest vs tavern), each swipe must store its own `location_header` and `event_header`. The plan already includes this.

### 3. Only the last message is swipeable
If events exist after narration, the user must delete the event before they can swipe on the narration. This matches current retry behavior (you can only retry the last message) and is consistent with the UI.

### 4. Soft-delete accumulation
If retry fails repeatedly, soft-deleted messages accumulate in the DB. They are restored on failure, so this is only a concern if the process crashes between soft-delete and restore/hard-delete.

**Mitigation:** On engine startup, `load_messages()` already filters `is_deleted = 0`. A periodic cleanup or startup task could purge orphaned soft-deleted messages, but this is low priority.

---

## Test Updates

### New tests
- **`tests/flow_mock/retry_main.rs`**: After main retry, assert old narration text exists as a swipe on the new narration message.
- **`tests/flow_mock/retry_event.rs`**: After event retry, assert old event text exists as a swipe on the new event message.
- **`tests/components/fragment.rs`**:
  - Swipe navigation restores correct snapshot.
  - Swipe navigation rejects non-last messages with 400.
  - Retrigger endpoint generates an event when `last_trigger` is present.
  - Retry failure restores soft-deleted messages.
- **`src/model/message_history_tests.rs`**: `to_log_entries()` uses active swipe text.
- **`src/storage/db_tests.rs`** or integration tests: `ON DELETE CASCADE` cleans up swipes on hard delete.

### Updated tests
- Retry tests that assert exact message counts or IDs need updating (old message is soft-deleted, swipes migrate to new message).
- `src/server/templates_tests.rs`: swipe rendering + retrigger button assertions.

---

## Verification Steps

1. `cd chronicler_engine && cargo test` → all pass.
2. `cd chronicler_engine && python build.py` → fmt, clippy, tests, coverage clean.
3. Manual UI check (screenshot):
   - Send input → narration + event generated.
   - Since event is now last message, swipe arrows appear on the event, not narration.
   - Delete event → narration becomes last message, swipe arrows appear on narration.
   - Hit right arrow (generate new) → counter "1/2" appears.
   - Swipe left → old narration appears. "Retrigger Event" button visible if snapshot has trigger data.
   - Click "Retrigger Event" → new event generated from restored snapshot.
   - Induce retry failure (e.g., cancel LLM) → soft-deleted messages are restored, no data loss.

---

## Files to Modify (checklist)

- [ ] `src/model/message.rs`
- [ ] `src/model/message_history.rs`
- [ ] `src/model/state.rs`
- [ ] `src/storage/db.rs`
- [ ] `src/storage/models/message.rs`
- [ ] `src/storage/mappers/message.rs`
- [ ] `src/storage/message_storage.rs`
- [ ] `src/storage/snapshot_storage.rs`
- [ ] `src/test_support/in_memory_storage.rs`
- [ ] `src/application/action_pipeline/retry.rs`
- [ ] `src/server/mod.rs` (add routes)
- [ ] `src/server/fragments/misc.rs`
- [ ] `src/server/templates.rs`
- [ ] `assets/index.html`
- [ ] `assets/styles.css`
- [ ] `docs/adr/` (update ADR-013 or create ADR-014)
- [ ] Various test files
