# ADR-018: Application Service Layer

## Status

Partially Supplemented (2026-05-29)

## Context

Originally, the server layer (`crate::server`) was leaking business logic directly into Axum handlers. The `ApplicationService` trait was created to act as a **logic firewall** between HTTP handlers and the domain.

## Changes (2026-05-29)

The application service layer has been refactored:

1. **Split into verb-based submodules**:
   - `game_lifecycle.rs`: Game lifecycle - `create_game`, `switch_game`, `delete_game`, `list_games`, `current_game_id`, `reset`
   - `message_editing.rs`: Message editing - `switch_swipe`, `edit_history`, `delete_last`, `retry`, `retrigger`
   - `query_handlers.rs`: Read-only queries - `get_generating_status`, `get_current_game_name`, `list_latest_llm_messages`, `get_story_log_entries`, `get_input_status`, `get_current_room_view`, `get_npc_headshots`, `get_debug_state`

2. **Trait deleted**: `ApplicationService` trait was removed (single implementation). Uses concrete `DefaultApplicationService` struct.

3. **Layer violation fixed**: `build_fresh_initial_state` moved from application tier to `bootstrap/state.rs`.

### Current Architecture

```rust
// Thin orchestrator - delegates to submodules
pub struct DefaultApplicationService {
    game_service: Arc<DefaultGameService>,
    lifecycle: GameLifecycleService,
    editing: MessageEditingService,
    queries: QueryHandlers,
}
```

### Wire-Up

`AppState` holds `application_service: Arc<DefaultApplicationService>`. All test constructors create both `game_service` and `application_service` with concrete types.

## Original Decision (Preserved)

The responsibilities remain unchanged:
- Load/save state and messages
- Validate preconditions (e.g., "generation already in progress")
- Set/clear `is_generating` flag
- Spawn game service calls
- Return raw data / `Result` (never rendered HTML)

## Related

- `docs/architecture/system.md` (Application Tier)
- `docs/architecture/guardrails.md` (arch-lint `server -> storage` rule)
- `src/application/application_service.rs`
