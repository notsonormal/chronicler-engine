# Remove Identity Wrapper Functions from GameService

**Status:** ✅ COMPLETED  
**Date:** 2026-05-31  
**Original Location:** `local://remove-identity-wrapper-functions.md`

## Problem
Three wrapper methods in `service.rs` are identity wrappers that add `#[instrument]` attributes but delegate directly to `*_impl` functions:
- `execute_action()` → `execute_action_impl()`
- `retry_last_response()` → `retry_last_response_impl()`
- `retrigger_event()` → `retrigger_event_impl()`

This is the identity-wrapper anti-pattern: indirection without earning its keep.

## Solution
Delete wrapper functions entirely. Move `#[instrument]` attributes to the `*_impl` functions where actual work happens. Update callers to invoke `*_impl` directly.

## Files Modified

1. **chronicler_engine/src/application/game_service/service.rs**
   - ✅ Removed `execute_action()` wrapper (lines 28-32)
   - ✅ Removed `retry_last_response()` wrapper (lines 34-38)
   - ✅ Removed `retrigger_event()` wrapper (lines 40-44)
   - ✅ Cleaned up unused imports

2. **chronicler_engine/src/application/action_pipeline/actions.rs**
   - ✅ Added `#[instrument]` to `execute_action_impl()` with fields: `player_name`, `input_length`

3. **chronicler_engine/src/application/action_pipeline/retry.rs**
   - ✅ Added `#[instrument]` to `retry_last_response_impl()`
   - ✅ Added `#[instrument]` to `retrigger_event_impl()`

4. **chronicler_engine/src/application/application_service.rs**
   - ✅ Updated call: `execute_action_impl(&*game_service, ...)`

5. **chronicler_engine/src/application/message_editing.rs**
   - ✅ Updated call: `retry_last_response_impl(&*game_service, ctx_clone)`
   - ✅ Updated call: `retrigger_event_impl(&*game_service, &ctx_clone)`

## Test Files Updated
- ✅ `tests/game_service.rs` - 3 call sites
- ✅ `tests/diagnostic.rs` - 1 call site
- ✅ `tests/diagnostic/scenarios.rs` - 2 call sites
- ✅ `tests/flow_mock/retry_event.rs` - 5 call sites
- ✅ `tests/flow_mock/retry_main.rs` - Multiple call sites
- ✅ `tests/flow_mock/sequence.rs` - 18 call sites

## Verification Results
✅ `cargo check` - Clean compilation  
✅ `cargo test --lib` - 648 tests passed  
✅ `cargo test --test game_service` - 5 tests passed  
✅ `cargo test --test diagnostic` - 12 tests passed  
✅ `cargo test --test flow_mock` - 21 tests passed  

## Trade-offs Considered
- **Option B (keep wrappers, move instrumentation)**: Rejected - still wasteful indirection
- **Option A (delete wrappers)**: Chosen - cleanest, removes unnecessary layer

## Notes
- `retrigger_event_impl` takes `&ctx` (reference), callers must pass `&ctx_clone` instead of `ctx_clone`
- All `*_impl` functions already exported via `mod.rs`
- Callers already have `Arc<DefaultGameService>`, need to deref for trait method calls
