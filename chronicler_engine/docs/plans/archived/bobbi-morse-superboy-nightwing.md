# Plan: Fix Swipe Code Review Issues

## Context
Fix 5 issues identified in code review of the message swipe feature:
1. `post_retry_swipe_migration` is not atomic — partial failure corrupts swipe indices
2. `Message::set_active_swipe` has no bounds check
3. `load_messages` silently leaves empty text when `active_swipe_index` is out of bounds
4. `switch_swipe_handler` unnecessarily reconstructs `game_state` from snapshot then rebuilds snapshot
5. Misleading comment in `switch_swipe_handler` about "updated active index"

## Approach

### Step 1: Atomic swipe migration
Add a single `migrate_swipes` method to `MessageStorage` that wraps the entire migration (shift → insert pending → set active → purge soft-deleted) in one atomic operation.

- **Trait** (`src/storage/message_storage.rs`): add `fn migrate_swipes(&self, message_id: u64, pending_swipes: &[Swipe], new_active_index: usize, to_delete: &[u64]) -> Result<(), EngineError>`
- **SQLite** (`src/storage/snapshot_storage.rs`): implement with `conn.execute("BEGIN")` / `COMMIT` wrapping all 4 operations
- **InMemory** (`src/test_support/in_memory_storage.rs`): implement by holding the mutex across all operations
- **Mocks** (`retry_tests.rs`, `mod_tests.rs`, `tests/components/fragment.rs`): add stub implementations
- **Call site** (`src/application/action_pipeline/retry.rs`): replace the 4 separate calls in `post_retry_swipe_migration` with one `migrate_swipes` call

### Step 2: Bounds check in `set_active_swipe`
In `src/model/message.rs`, guard the index:
```rust
pub fn set_active_swipe(&mut self, index: usize) {
    if index >= self.swipes.len() {
        return;
    }
    // existing logic
}
```

### Step 3: Fallback hydration in `load_messages`
In `src/storage/snapshot_storage.rs`, change the hydration loop to fallback to swipe 0 when `active_swipe_index` is out of bounds:
```rust
for msg in &mut messages {
    let target = msg.swipes.get(msg.active_swipe_index).or(msg.swipes.first());
    if let Some(swipe) = target {
        msg.text = swipe.text.clone();
        // ... etc
    }
}
```

### Step 4: Simplify `switch_swipe_handler`
In `src/server/fragments/misc.rs`, remove the `GameState::from_snapshot`, `replace(messages)`, and `GameStateSnapshot::from_game_state` reconstruction. Instead, load the target snapshot and save it directly:
```rust
let snapshot = state.snapshot_storage.load_by_id(snapshot_id)?;
state.snapshot_storage.save(&snapshot)?;
```
This is equivalent (the snapshot already contains the correct world state) and removes the stale-messages problem entirely.

### Step 5: Fix comment
Update the comment in `switch_swipe_handler` step 6 to accurately describe what the code does, or remove the unnecessary step since we're no longer building `game_state`.

## Verification
- `cd chronicler_engine && cargo nextest run` — all tests pass
- `cd chronicler_engine && cargo clippy` — clean
- `cd chronicler_engine && python build.py` — full validation passes

## Files touched
- `src/storage/message_storage.rs`
- `src/storage/snapshot_storage.rs`
- `src/test_support/in_memory_storage.rs`
- `src/application/action_pipeline/retry.rs`
- `src/model/message.rs`
- `src/server/fragments/misc.rs`
- `src/application/action_pipeline/retry_tests.rs`
- `src/server/mod_tests.rs`
- `tests/components/fragment.rs`
