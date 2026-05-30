# Plan: Extract Shared Retry Orchestration in MessageEditingService

## Problem
`MessageEditingService::retry()` and `MessageEditingService::retrigger()` have nearly identical orchestration patterns (~20 lines of duplication):
1. Load game state
2. Validate preconditions
3. Set status/phase to Generating/Narrating
4. Save snapshot
5. Check cancellation
6. Clone context and service
7. Spawn blocking task

This is classic spaghetti-growth that will compound if more retry-variant methods are added.

## Files to Modify
- `chronicler_engine/src/application/message_editing.rs`

## Approach
Extract shared setup logic into a private helper `prepare_retry_state()` that returns the prepared state and a cancel check. The spawn logic remains in each method since the game_service method called differs (`retry_last_response` vs `retrigger_event`).

### Changes

**`message_editing.rs`:**

1. Add new helper method:
```rust
fn prepare_retry_state(
    ctx: &GameServiceContext,
    mut game_state: GameState,
    status: GenerationStatus,
    phase: GenerationPhase,
) -> Result<(GameState, bool), ApplicationError> {
    // Returns (state, was_cancelled)
    game_state.narrative.input_buffer.status = status;
    game_state.narrative.input_buffer.phase = phase;
    let snapshot = GameStateSnapshot::from_game_state(&game_state);
    ctx.storage.save_snapshot(&snapshot)?;
    let cancelled = ctx.cancel_token.is_cancelled();
    Ok((game_state, cancelled))
}
```

2. Simplify `retry()`:
```rust
pub fn retry(&self, ctx: GameServiceContext) -> Result<(), ApplicationError> {
    let mut game_state = load_state(&ctx);
    if game_state.narrative.history.last_input_text().is_none() {
        return Err(ApplicationError::validation("No input to retry"));
    }
    let (game_state, cancelled) = Self::prepare_retry_state(
        &ctx,
        game_state,
        GenerationStatus::Generating,
        GenerationPhase::Narrating,
    )?;
    if cancelled {
        return Err(ApplicationError::ShuttingDown);
    }
    let game_service = Arc::clone(&self.game_service);
    let ctx_clone = ctx.clone();
    tokio::task::spawn_blocking(move || {
        if ctx_clone.cancel_token.is_cancelled() {
            return;
        }
        game_service.retry_last_response(ctx_clone);
    });
    Ok(())
}
```

3. Simplify `retrigger()` similarly.

## Verification
- [x] `cargo check --package chronicler_engine` passes
- [x] Existing tests pass: `cargo test --package chronicler_engine -- message_editing`
- [x] Both methods still exhibit identical behavior (verify by inspection)

## Notes
- The spawn logic itself is NOT extracted because `retry_last_response` vs `retrigger_event` are different game_service methods
- Could further unify by passing the game_service method as a closure, but that adds complexity for minimal gain