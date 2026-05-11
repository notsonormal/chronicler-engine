# Plan: Restrict Message Deletion & Rethink Location/Event Headers

## Problem

1. `delete_log(id)` can delete **any** message by ID from the middle of history, breaking chronological integrity.
2. Location headers (`sender=Some(room)`, `text=""`, `Narration`) and event headers (`sender=Some(event_name)`, `text=""`, `Event`) are stored as **separate `LogEntry` records** from the narration they annotate. Deleting one leaves orphans.
3. The underlying game state (e.g. `movement.current_room_id`, `trigger_fired`) is **not** reverted when history entries are deleted, so state and log diverge.

## Root Cause

Location and event metadata were modelled as first-class history entries rather than as **visual annotations on the narration they describe**. This forces cascade-deletion logic and makes "delete last message" semantics impossible to implement cleanly.

---

## What the user agrees on

- **Events**: Should be tied to the log entry itself (inline metadata).
- **Deletion**: Should only be possible for the **last** message.

## Open question: how to handle location headers

Two options for location. Both keep the "last message only" deletion rule and solve the orphan-header problem.

---

### Option A: Inline `location_header` via `pending_location` (Recommended)

Location is treated the same as events: an optional field on `LogEntry` that is only populated on the **first log entry created after a room change**. It is NOT present on every message in the room.

**How it works:**
- `handle_movement` sets `state.narrative.pending_location = Some(room_name)` **instead of** calling `add_log`.
- The next call to `add_log` (always the main narration) consumes the pending location into `LogEntry.location_header` and clears the flag.
- Subsequent narrations, inputs, etc. in the same room do **not** get a location header.

**Visual result:** Identical to today. The template renders the location header line inside the same div as the narration that follows it:
```html
<div class="log-entry narration">
  <span class="location-header">Entrance Hall</span>
  <span class="text">You walk into the hall...</span>
</div>
```
CSS can style the `.location-header` exactly like the current standalone location entry, so the player sees no visual difference.

**Pros:**
- Zero orphan entries; deleting the narration deletes its location header automatically.
- Minimal storage: `Option<String>` is only `Some` on the entry immediately after movement.
- Simple implementation: same pattern as events.

**Cons:**
- Slightly different DOM structure (one div instead of two). Requires minor CSS adjustment to maintain identical visuals.
- If the main narration is somehow skipped, the pending location is lost. (In practice this never happens—the pipeline always logs narration.)

---

### Option B: Room context tracking with render-time derivation

Every `LogEntry` stores the `room_id` that was current when it was created. Location headers are derived at render time by comparing `room_id` between consecutive **visible** entries.

**How it works:**
- `add_log` sets `LogEntry.room_id = Some(state.movement.current_room_id.clone())` on every entry.
- The template/renderer scans the history and inserts a visual location header whenever `room_id` changes from the previous visible entry.
- No separate location records exist; location is purely derived.

**Pros:**
- Every entry permanently knows which room it was created in. Useful for future features (e.g. filtering history by room).
- No "pending" state to manage; `handle_movement` doesn't touch the narrative at all.

**Cons:**
- Heavier: every `LogEntry` carries a `room_id` string (or at least an `Option<String>`).
- Renderer logic is more complex: must compare adjacent entries and decide which entry "owns" the header.
- Edge cases: if a room change happens but the next visible entry is a System message or Input, should the location header appear on that entry or wait for the next Narration? Current behaviour always shows it immediately.

---

## Recommended choice

**Option A** is strongly recommended.

- It solves the orphan problem with the simplest possible mechanism.
- It preserves the existing visual rhythm `[LOCATION] [TEXT] [TEXT]` without complexity.
- The "heavy" concern is addressed: `location_header` is `None` on ~95% of entries (only `Some` immediately after movement).
- Events and locations use the same pattern, keeping the codebase consistent.

---

## Full data model (with Option A)

```rust
// src/model/state.rs
pub struct LogEntry {
    pub id: u64,
    pub sender: Option<String>,
    pub text: String,
    pub log_type: LogType,
    pub timestamp: DateTime<Utc>,
    pub location_header: Option<String>,  // NEW: only Some on first log after movement
    pub event_header: Option<String>,     // NEW: only Some on event continuation narrations
}

pub struct NarrativeState {
    pub history: Vec<LogEntry>,
    pub next_log_id: u64,
    pub generation: GenerationState,
    pub last_trigger: Option<StoredTriggerContext>,
    pub pending_location: Option<String>,  // NEW
    pub pending_event: Option<String>,     // NEW
}
```

---

## Files to change

| File | Change |
|------|--------|
| `src/model/state.rs` | Add `location_header`/`event_header` to `LogEntry`; add `pending_location`/`pending_event` to `NarrativeState`; update `add_log` to consume pending metadata; replace `delete_log(id)` with `delete_last_log()`; simplify `is_last_ai_response_event_continuation` |
| `src/model/state_snapshot.rs` | Add `#[serde(default)]` on new fields for backward compat |
| `src/engine/action_processing.rs` | `handle_movement`: set `pending_location` instead of `add_log`; `commit_trigger_narration`: set `pending_event` instead of `LogType::Event` entry; `evaluate_and_narrate_triggers`: same |
| `src/engine/action_processing_tests.rs` | Update assertions: single entries with metadata instead of separate header+narration pairs |
| `src/server/templates.rs` | Update `LogEntryView::from` and Askama template to render `location_header`/`event_header` inside the narration div; remove `is_location`/`is_event` heuristics |
| `src/server/templates_tests.rs` | Update tests for new rendering model |
| `src/server/fragments/history.rs` | Change handler to call `delete_last_log()`; return 400 if history is empty; remove `id` from URL path (becomes `/history/delete`) |
| `src/server/mod.rs` | Update route registration |
| `assets/index.html` | Update `deleteMessage()` JS: remove `id` parameter, call new endpoint |
| `src/model/state_tests.rs` | Replace `test_delete_log` with `test_delete_last_log`; add tests for pending metadata absorption |
| `tests/components/fragment.rs` | Update component tests for new endpoint and behaviour |

---

## Behaviour changes

1. **Movement** (`handle_movement`): sets `state.narrative.pending_location = Some(room_name)`. No log is added.
2. **Main narration** (`execute_freeaction_impl`): calls `add_log(text, None, Narration)`. `add_log` sees the pending location, moves it into the new `LogEntry.location_header` field, and clears the pending flag.
3. **Trigger** (`commit_trigger_narration`): sets `pending_event = Some(trigger_name)`, then calls `add_log(continuation_text, None, Narration)`. The event header is stored in `LogEntry.event_header`.
4. **Rendering** (`StoryLogTemplate`): For each entry, if `location_header` is set, render a location header line inside the same div. If `event_header` is set, render an event header line inside the same div.
5. **Deletion** (`delete_last_log`): pops the last entry from `history`. No cascade logic needed. Returns error if history is empty.
6. **Retry detection** (`is_last_ai_response_event_continuation`): check `history[last_ai_idx].event_header.is_some()`.

---

## Backward compatibility

New fields use `#[serde(default)]` so old snapshots deserialize safely. On first load, a lightweight migration runs:

- Scan `history` for legacy location entries (`sender=Some(x)`, `text=""`, `Narration`). Merge the `sender` value into the **next** entry's `location_header` field, then remove the legacy entry.
- Scan for legacy event entries (`log_type == Event`, `text=""`). Merge the `sender` value into the **next** entry's `event_header` field, then remove the legacy entry.

This preserves visual appearance while upgrading the data model in-memory. The next snapshot save writes the new format.

---

## Testing strategy

- Unit tests in `state_tests.rs`: `add_log` absorbs pending metadata; `delete_last_log` rejects empty history; migration merges legacy entries.
- Engine tests in `action_processing_tests.rs`: `handle_movement` sets pending location (no immediate log); `commit_trigger_narration` creates one entry with `event_header` metadata.
- Template tests in `templates_tests.rs`: location and event headers render inside the narration div.
- Component tests in `tests/components/fragment.rs`: `POST /history/delete` removes the last entry; returns 400 when empty.

---

## Out of scope

- **State reversion on delete**: Deleting the last log does NOT revert `movement.current_room_id` or un-fire triggers. The user asked about this complexity and concluded the data model was the real problem. Fixing the data model (no orphan headers) is the scope of this plan. True "undo turn" state reversion would require loading a pre-turn snapshot and is left for future work.
- **Edit handler**: `edit_log(id)` remains unchanged; it can still edit any entry's text.
