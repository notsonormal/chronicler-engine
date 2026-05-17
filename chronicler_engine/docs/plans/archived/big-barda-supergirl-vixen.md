# Plan: Fix Pipeline Cancellation Blindness

## Verification Result

The comment is **100% accurate**.

### Evidence

1. **`context.rs:28`** — `GameServiceContext` carries `pub cancel_token: CancellationToken`.
2. **`actions.rs`** — `execute_freeaction_pipeline` (lines 218–392) never checks `ctx.cancel_token.is_cancelled()`:
   - Grep for `is_cancelled` or `cancel_token` inside `actions.rs` returned **zero matches**.
3. **Main LLM call** — `backend.narrate_action(...)` at `actions.rs:254`.
4. **Trigger continuation LLM call** — `backend.complete(...)` at `actions.rs:326`.
5. **No checkpoints between stages** — After the main narration returns (line 260), the code proceeds directly to quantification, trigger building, and then the second LLM call without any cancellation check.
6. **Backend implementations do not check either** — Grep inside `chronicler_engine/src/narrative/llm/` for `is_cancelled` or `spawn_blocking` returned **zero matches**, so the invariant doc’s claim that "spawn closures check `CancellationToken::is_cancelled()`" is not actually implemented in the current code.

### Impact

If a user cancels generation during the first heavy LLM call, the pipeline will still proceed to:
- Run the quantifier agents (`run_post_generation_agents` at line 272)
- Build trigger request (`build_trigger_request` at line 302)
- Spawn the **second** heavy LLM call (`backend.complete` at line 326)

This wastes API tokens and CPU cycles.

## Proposed Fix

Add explicit cancellation checkpoints in `execute_freeaction_pipeline` (`actions.rs`):

1. **After the main narration LLM call** (line 260) — return early if cancelled before proceeding to quantification.
2. **Before committing the pre-event snapshot** (line 321) — return early if cancelled before the second LLM call.
3. **After the trigger continuation LLM call** (line 347) — return early if cancelled before committing trigger narration.

At each checkpoint, if `ctx.cancel_token.is_cancelled()`:
- Log a warning
- Reset `state.narrative.input_buffer.status = GenerationStatus::Idle`
- Save state (so the UI reflects cancellation)
- Return early

## Files to Modify

- `chronicler_engine/src/application/game_service/actions.rs`

## Success Criteria

- [ ] `execute_freeaction_pipeline` checks `ctx.cancel_token.is_cancelled()` at the three checkpoints above.
- [ ] When cancelled mid-pipeline, status resets to `Idle` and state is persisted.
- [ ] Existing tests pass (`cd chronicler_engine && python build.py`).
