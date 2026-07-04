# T3: Service Layer Cleanup

**Parent Plan:** [abstraction-fixes-followup-superplan.md](../abstraction-fixes-followup-superplan.md)
**Status:** Done — landed 2026-06-28
**Date:** 2026-06-28
**Depends on:** none
**Blocks:** none
**Priority:** P1
**Findings owned:** B10 (closed — `spawn_pipeline_task` extracted + reused by all 3 sites), N1 (closed — 9 identity-passthroughs deleted)

---

## Summary

`GameLifecycleService` was flattened into `DefaultApplicationService` (B4 closed). But two related smears remain:

1. **N1:** `DefaultApplicationService` still has ~9 identity-passthrough methods to `super::query_handlers::*` (`application_service.rs:333-393`: `get_generating_status`, `reset_generating_status`, `get_current_game_name`, `list_latest_llm_messages`, `get_story_log_entries`, `get_input_status`, `get_current_room_view`, `get_npc_headshots`, `get_debug_state`). Service-layer sandwich relocated, not eliminated.
2. **B10 partial:** `spawn_pipeline_task` helper extracted on `DefaultApplicationService:181` for `process_action`, but `message_editing.rs:145, 189` (`retry` / `retrigger`) still inline `tokio::task::spawn_blocking` with same shape (clone Arc, clone ctx, check cancel_token, spawn).

## Architecture-Lens Reframe

Deletion test on the 9 query passthroughs: delete them → complexity vanishes (callers `use crate::application::query_handlers::*` directly). Pure pass-through; nothing hidden, zero leverage. The "move to GameService" alternative creates a **new** shallow delegate in a different file — same shape, different file. Reject.

Same shape applies to the 5 editing delegates (`retry`, `retrigger`, `switch_swipe`, `edit_history`, `delete_last`). Default: delete unless mutation-coherence justifies promoting `MessageEditingService` to a deep **HistoryMutation** module.

## Key Changes

1. Delete the 9 query-handler wrappers from `DefaultApplicationService` (`application_service.rs:333-393`); callers `use crate::application::query_handlers::*` directly.
2. Extract a single `spawn_pipeline_task<F>` helper — signature `(ctx, f: F) where F: FnOnce(&GameService, GameServiceContext) + Send + 'static`. Used by `process_action`, `retry`, `retrigger`.
3. Decide on `MessageEditingService`: delete the 5 delegates (default), or promote to HistoryMutation.

## Decisions to Lock

- Keep `MessageEditingService` (promote to HistoryMutation) or delete the 5 delegates?

## Blast Radius

`application_service.rs`, `message_editing.rs`, ~3–5 caller call sites (server fragments).

## Verification

- `python build.py` — fmt + clippy + tests + coverage must pass clean.
- Integration test coverage for the 3 spawn sites (`process_action`, `retry`, `retrigger`) — required before merge (structural track).
- Verify no callers of the deleted 9 methods remain after migration (~5 server fragments).

## Pre-Implementation Checklist

- [ ] List all callers of the 9 query-handler wrappers (grep `application_service\.\(get_generating_status\|reset_generating_status\|...\)` across `server/`).
- [ ] List all callers of the 5 editing delegates (grep `editing\.\(retry\|retrigger\|switch_swipe\|edit_history\|delete_last\)` across `server/`).
- [ ] Confirm decision on `MessageEditingService` with user before writing code.
