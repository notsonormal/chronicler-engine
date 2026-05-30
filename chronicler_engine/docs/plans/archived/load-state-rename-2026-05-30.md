# Plan: Rename `try_load_state` / `load_state` to Clearer Names

## Problem
Two functions with similar names have opposite failure semantics — easy to use wrong one:
- `try_load_state(ctx)` → `Result<GameState, EngineError>` (explicit)
- `load_state(ctx)` → `GameState` (graceful fallback)
- `ctx.load_state()` → `GameState` (test helper, panics if no snapshot)

## Approach - DONE
Rename to convey behavior explicitly:

| Old | New | Behavior |
|-----|-----|----------|
| `try_load_state` | `load_expecting_valid_state` | Returns `Result<GameState, EngineError>. Callers decide recovery. |
| `load_state` | `load_or_fresh` | Returns `GameState`. Logs warning, returns fresh state on failure. Safe default. |
| `ctx.load_state()` | `ctx.load_state_for_test()` | Test-only. Panics if no snapshot. |

## Files Modified
- `src/bootstrap/run.rs` — `try_load_state` → `load_expecting_valid_state`
- `src/server/fragments/endpoints.rs` — `try_load_state` → `load_expecting_valid_state`
- `src/application/context.rs` — function renames (already done from prior work)
- `src/application/action_pipeline/actions.rs` — import + use renamed
- `src/application/action_pipeline/pipeline.rs` — import + use renamed
- `src/application/action_pipeline/retry.rs` — import + use renamed
- `src/application/action_pipeline/actions_tests.rs` — ctx method renamed
- `src/application/action_pipeline/pipeline_tests.rs` — ctx method renamed
- `src/application/action_pipeline/retry_tests.rs` — ctx method + imports renamed
- `src/application/application_service.rs` — import + use renamed
- `src/application/message_editing.rs` — import + use renamed
- `src/application/query_handlers.rs` — import + use renamed
- `src/application/context_tests.rs` — function references renamed

## Verification
- ✅ `cargo fmt` passes
- ✅ `cargo clippy` passes
- ✅ Architecture guardrails pass
- ✅ Custom guardrails pass
- ✅ `cargo build` succeeds (debug)
- ✅ All 939 tests pass (2 skipped — LLM tests)
- ✅ No remaining `load_state` identifiers in src/

## Notes
The test helper method `ctx.load_state_for_test()` on `GameServiceContext` remains unchanged in behavior (panics if no snapshot). Only the name was clarified to indicate test-only usage.
production code should use `load_or_fresh` (graceful) or `load_expecting_valid_state` (strict).
