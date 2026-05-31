# Plan: Enforced Message Consistency via Private Fields

## Problem

`Message.text`, `Message.location_header`, `Message.event_header`, and `Message.snapshot_id` are `pub` fields, allowing direct mutation that bypasses `set_active_swipe()`. This creates inconsistency between the active swipe and the runtime-mirrored fields.

**Current:**
```rust
pub struct Message {
    pub text: String,                    // writable directly - bypasses swipe tracking
    pub location_header: Option<String>, // same problem
    pub event_header: Option<String>,    // same problem
    pub snapshot_id: Option<u64>,         // same problem
    pub swipes: Vec<Swipe>,
    pub active_swipe_index: usize,
    // ...
}
```

Direct writes at:
- `message_history.rs:34` — `msg.text = new_text.clone()`
- `state.rs:332-334` — `target.text = text; target.location_header = ...; target.event_header = ...`
- `context.rs:126-129` — `msg.text = ...; msg.location_header = ...; msg.event_header = ...; msg.snapshot_id = ...`
- `mappers/message.rs:52-55` — `message.text = ...; message.location_header = ...; etc.`

## Goal

Make runtime-mirrored fields private. Add controlled mutation methods. Compiler enforces consistency — no way to bypass `set_active_swipe()`.

## Approach

Private fields + read-only accessors + controlled mutation methods. One atomic operation for each mutation category.

## Changes

### 1. `src/model/message.rs` — Core struct and methods

**After:**
```rust
pub struct Message {
    pub id: u64,
    pub sender: Option<String>,
    text: String,                        // private
    pub message_type: MessageType,
    pub timestamp: DateTime<Utc>,
    location_header: Option<String>,     // private
    event_header: Option<String>,        // private
    snapshot_id: Option<u64>,           // private
    pub active_swipe_index: usize,
    pub swipes: Vec<Swipe>,
    pub is_deleted: bool,
}

impl Message {
    // existing: new(), is_unpersisted(), swipe_count()

    pub fn text(&self) -> &str { &self.text }
    pub fn location_header(&self) -> Option<&String> { self.location_header.as_ref() }
    pub fn event_header(&self) -> Option<&String> { self.event_header.as_ref() }
    pub fn snapshot_id(&self) -> Option<u64> { self.snapshot_id }

    pub fn set_active_swipe(&mut self, index: usize) { /* existing - already correct */ }

    /// Updates text of the active swipe. Maintains msg↔swipe consistency.
    pub fn update_active_swipe_text(&mut self, new_text: impl Into<String>) {
        let new_text = new_text.into();
        self.text = new_text.clone();
        if let Some(swipe) = self.swipes.get_mut(self.active_swipe_index) {
            swipe.text = new_text;
        }
    }

    /// Sets event_header directly (used by tests to simulate event messages).
    pub fn set_event_header(&mut self, header: Option<String>) {
        self.event_header = header;
    }
}
```

### 2. `src/model/message_history.rs` — Uses `update_active_swipe_text()`

**Before (`edit` method):**
```rust
msg.text = new_text.clone();
let idx = msg.active_swipe_index;
if let Some(swipe) = msg.swipes.get_mut(idx) {
    swipe.text = new_text;
}
```

**After:**
```rust
msg.update_active_swipe_text(new_text);
```

### 3. `src/model/state.rs` — Uses `set_active_swipe()`

**Before (`push_message` retry interception, lines 324-335):**
```rust
target.swipes.push(swipe);
target.active_swipe_index = target.swipes.len() - 1;
target.text = text;
target.location_header = location_header;
target.event_header = event_header;
```

**After:**
```rust
target.swipes.push(swipe);
target.set_active_swipe(target.swipes.len() - 1);
```

### 4. `src/application/context.rs` — Uses `set_active_swipe()`

**Before (`load_messages`, lines 126-129):**
```rust
msg.text = swipe.text.clone();
msg.location_header = swipe.location_header.clone();
msg.event_header = swipe.event_header.clone();
msg.snapshot_id = swipe.snapshot_id;
```

**After:**
```rust
msg.set_active_swipe(msg.active_swipe_index);
```

### 5. `src/storage/mappers/message.rs` — Uses `set_active_swipe()`

**Before (`db_message_to_model`, lines 52-55):**
```rust
message.text = swipe.text.clone();
message.location_header = swipe.location_header.clone();
message.event_header = swipe.event_header.clone();
message.snapshot_id = swipe.snapshot_id;
```

**After:**
```rust
message.set_active_swipe(message.active_swipe_index);
```

### 6. Test files — Use new API

| File | Line | Change |
|------|------|--------|
| `src/model/message_tests.rs` | 31 | `msg.text = "Updated".to_string()` → `msg.update_active_swipe_text("Updated")` |
| `src/model/message_tests.rs` | 32 | `assert_eq!(msg.text, "Updated")` → `assert_eq!(msg.text(), "Updated")` |
| `src/model/message_history_tests.rs` | 93 | `history.last_mut().unwrap().text = "y".to_string()` → `history.last_mut().unwrap().update_active_swipe_text("y")` |
| `src/model/message_history_tests.rs` | 94 | `assert_eq!(history.last().unwrap().text, "y")` → `assert_eq!(history.last().unwrap().text(), "y")` |
| `src/model/message_history_tests.rs` | 183 | `msg.event_header = Some("Event".to_string())` → `msg.set_event_header(Some("Event".to_string()))` |
| `src/application/context_tests.rs` | 194 | `target.text = "Retried narration".to_string()` → `target.update_active_swipe_text("Retried narration")` |

## Files to Modify

1. `chronicler_engine/src/model/message.rs` — private fields + getters + `update_active_swipe_text()` + `set_event_header()`
2. `chronicler_engine/src/model/message_history.rs` — `edit()` uses new method
3. `chronicler_engine/src/model/state.rs` — `push_message()` uses `set_active_swipe()`
4. `chronicler_engine/src/application/context.rs` — `load_messages()` uses `set_active_swipe()`
5. `chronicler_engine/src/storage/mappers/message.rs` — `db_message_to_model()` uses `set_active_swipe()`
6. `chronicler_engine/src/model/message_tests.rs` — test uses new method
7. `chronicler_engine/src/model/message_history_tests.rs` — test uses new method
8. `chronicler_engine/src/application/context_tests.rs` — test uses new method

## Verification

1. `cargo build` — compiles without errors
2. `cargo test` — all tests pass
3. Confirm no remaining direct `.text =` writes on `Message` in production code

## Notes

- `snapshot_id` writes in `context.rs:216`, `game_lifecycle.rs:48,127`, `run.rs:245` are for **persisting** the active swipe's snapshot_id when saving a new message — these are correct as-is (they mirror the swipe, not bypass it).
- `Message::new()` still initializes `text`, `location_header`, `event_header` directly — this is fine since the message is new and has exactly one swipe at index 0.
