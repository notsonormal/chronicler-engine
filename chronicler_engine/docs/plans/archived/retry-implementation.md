# Plan: Implement Retry Handler Logic

## Problem Statement

The `/retry` endpoint currently has a stub implementation that returns "Retrying..." without actually regenerating the AI response. This was left as a TODO comment in `fragments.rs`.

## Requirements

### Feature: Retry Last AI Response

1. **Find last exchange**: Locate the last user input (`LogType::Input`) and its corresponding AI response (`LogType::Narration` or `LogType::Dialogue`)
2. **Build prompt context**: Clone world, room, player, NPCs, and history (excluding the AI response being retried)
3. **Call LLM**: Regenerate the AI response using the original user input
4. **Replace**: Update the existing AI response text in-place

### Critical Constraint

**The history passed to LLM MUST NOT include the AI response being retried.** This prevents the LLM from:
- Repeating/paraphrasing the old response
- Getting confused about "last response" semantics

```rust
// History must be truncated at last_ai_response_index
let last_ai_idx = state_guard.get_last_ai_response_index();
let history = if let Some(idx) = last_ai_idx {
    state_guard.narration_history[..idx].to_vec()  // Exclude old response
} else {
    state_guard.narration_history.clone()
};
```

## Solution

### 1. Add `replace_last_ai_response` method to `GameState`

Location: `src/model/state.rs`

```rust
/// Replace the AI response that follows the last user input
pub fn replace_last_ai_response(&mut self, new_text: String) -> Result<(), EngineError> {
    let input_idx = self.get_last_input_index()
        .ok_or_else(|| EngineError::Internal("No input to retry".into()))?;
    let ai_idx = self.get_last_ai_response_index()
        .ok_or_else(|| EngineError::Internal("No AI response to retry".into()))?;
    
    // Validate: AI response must come AFTER the input
    if ai_idx <= input_idx {
        return Err(EngineError::Internal(
            "AI response must be after input".into()
        ));
    }
    
    let entry = self.narration_history.get_mut(ai_idx)
        .ok_or_else(|| EngineError::Internal("AI response not found".into()))?;
    entry.text = new_text;
    Ok(())
}
```

### 2. Implement `retry_handler` in `fragments.rs`

Location: `src/server/fragments.rs` (lines 733-759)

Replace the TODO stub with:

1. Lock state and get `last_input_text` and `last_ai_response_index`
2. Clone required data (world, map, player, all_npcs, nearby_npcs, history truncated at ai_idx)
3. Spawn `thread::spawn` with `process_retry(state_clone, input_text)`
4. Return immediately with "Retrying..."
5. In thread: call LLM → call `state_guard.replace_last_ai_response(new_text)` → set `is_generating = false`

### Threading Pattern

Follow the same pattern as `process_action`:
- Clone all data BEFORE the `move` closure
- Use `std::thread::spawn` for background processing
- Return "Retrying..." immediately to browser

## Files to Modify

| File | Changes |
|------|---------|
| `src/model/state.rs` | Add `replace_last_ai_response()` method |
| `src/server/fragments.rs` | Implement full `retry_handler` logic |

## Implementation Order

1. Add `replace_last_ai_response` method to `GameState`
2. Write failing test for the method
3. Implement `retry_handler` in fragments.rs
4. Run validation (fmt, clippy, tests)
5. Test with mock backend

## Acceptance Criteria

1. Clicking retry regenerates via LLM with original input context
2. History passed to LLM does NOT contain the AI response being retried
3. Original AI response text is replaced with new one
4. Returns "Retrying..." immediately while LLM generates
5. Works with mock backend for testing
6. All 239+ tests pass

## Edge Cases

| Scenario | Expected Behavior |
|----------|-------------------|
| No input in history | Return 400 "No input to retry" |
| No AI response after input | Return 400 "No AI response to retry" |
| LLM call fails | Set error_message, return error status |
| Lock fails | Return 500 "Failed to lock state" |