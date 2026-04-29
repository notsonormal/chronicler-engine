# Plan: History Edit & Retry Feature

## Problem Statement

Currently, the Chronicler Engine provides no way to:
1. **Edit** past conversation entries (both user inputs and AI responses)
2. **Retry** the last AI response to regenerate it with fresh LLM call

Users expect this functionality from Silly Tavern, where they can edit any message in place and regenerate the last AI response.

## Requirements

### Feature 1: Edit History

- Edit any entry in `narration_history` by index
- Both `LogType::Input` (user messages) and `LogType::Narration/Dialogue` (AI responses) are editable
- Edit replaces the `text` field in place
- Timestamp remains unchanged (or optionally updates)
- Subsequent entries are unaffected

### Feature 2: Retry Last Response

- Regenerate the immediately last AI response (Narration or Dialogue)
- Finds the last user input (`LogType::Input`) and regenerates its corresponding AI response
- Replaces the existing AI response with the new one
- Only works on the last exchange, not arbitrary history points

### Constraints

- In-memory only (no persistence)
- No modification to history entries after the edited/retry point

## Proposed Solution

### Model Changes (`src/model/state.rs`)

Add methods to `GameState`:

```rust
/// Edit a log entry by index
pub fn edit_log(&mut self, index: usize, new_text: String) -> Result<(), EngineError>

/// Get the last AI response index to retry (for retry button visibility)
pub fn get_last_ai_response_index(&self) -> Option<usize>

/// Get the last user input for context
pub fn get_last_input_index(&self) -> Option<usize>
```

Add ID field to `LogEntry`:
```rust
pub struct LogEntry {
    pub id: u64,           // NEW: auto-incrementing unique ID
    pub sender: Option<String>,
    pub text: String,
    pub log_type: LogType,
    pub timestamp: DateTime<Utc>,
}
```

### Server Changes

Add endpoints:

| Method | Path | Description |
|--------|-----|-------------|
| `PUT` | `/api/history/{id}` | Edit entry text |
| `POST` | `/api/retry` | Retry last AI response |

Request/Response formats use JSON.

### UI Changes (`assets/index.html`)

- Each log entry displays edit button on hover (pencil icon)
- "Retry" button appears near the last AI response (refresh icon)
- Inline or modal editing interface
- Visual feedback during regeneration (loading state)

## Files to Modify

| File | Changes |
|------|---------|
| `src/model/state.rs` | Add ID to LogEntry, edit methods |
| `src/error.rs` | Add HistoryError variant if needed |
| `src/server/mod.rs` | Add edit/retry routes |
| `src/server/fragments.rs` | Add edit/retry handlers |
| `src/server/templates.rs` | Add edit form templates |
| `src/narrative/prompt.rs` | Allow regenerating with history context |
| `assets/index.html` | Add edit/retry UI |
| `docs/architecture/system.md` | Document new features |

## Implementation Sequence

1. Add ID field to `LogEntry` with auto-increment
2. Add edit methods to `GameState`
3. Add server endpoints
4. Update UI templates
5. Add edit/retry buttons to HTML
6. Run validation tests

## Acceptance Criteria

1. User can click edit on any history entry and change its text
2. User can click retry to regenerate the last AI response
3. Both features work without breaking existing chat flow
4. Build passes (`cargo fmt`, `clippy`, `test`)
5. Visual verification in browser shows edit controls on hover