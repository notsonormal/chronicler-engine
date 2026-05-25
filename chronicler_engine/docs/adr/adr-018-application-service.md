# ADR-018: Application Service Layer

## Status

Accepted

## Context

The server layer (`crate::server`) was leaking business logic directly into Axum handlers. Handlers were:
- Loading state from `snapshot_storage`
- Saving snapshots via `snapshot_storage.save()`
- Loading messages from `message_storage`
- Constructing `GameStateSnapshot` from `GameState`
- Spawning LLM generation tasks

This made handlers difficult to test, tightly coupled to storage internals, and prone to duplication. The `GameService` trait existed but was focused on LLM backend orchestration, not HTTP-request-level orchestration.

## Decision

Create a dedicated `ApplicationService` trait and `DefaultApplicationService` implementation in `crate::application` that acts as a **logic firewall** between HTTP handlers and the domain.

### Responsibilities

- Load/save state and messages
- Validate preconditions (e.g., "generation already in progress")
- Set/clear `is_generating` flag
- Spawn `GameService` calls
- Return raw data / `Result` (never rendered HTML)

### Trait Design

```rust
pub trait ApplicationService: Send + Sync {
    fn process_action(&self, ctx: GameServiceContext, input: String)
        -> Result<ProcessActionResult, EngineError>;
    fn retry(&self, ctx: GameServiceContext) -> Result<(), ApplicationError>;
    fn retrigger(&self, ctx: GameServiceContext) -> Result<(), ApplicationError>;
    fn reset(&self, ctx: GameServiceContext) -> Result<(), ApplicationError>;
    fn switch_swipe(&self, ctx: GameServiceContext, message_id: u64, swipe_index: usize)
        -> Result<(), ApplicationError>;
    fn edit_history(&self, ctx: GameServiceContext, id: u64, text: String)
        -> Result<(), ApplicationError>;
    fn delete_last(&self, ctx: GameServiceContext) -> Result<(), ApplicationError>;
    fn create_game(&self, ctx: GameServiceContext) -> Result<u64, ApplicationError>;
    fn switch_game(&self, ctx: GameServiceContext, id: u64) -> Result<(), ApplicationError>;
    fn delete_game(&self, ctx: GameServiceContext, id: u64) -> Result<(), ApplicationError>;
    fn list_games(&self, ctx: GameServiceContext) -> Result<Vec<Game>, ApplicationError>;
    fn current_game_id(&self, ctx: GameServiceContext) -> Result<u64, ApplicationError>;
    fn get_current_game_name(&self, ctx: GameServiceContext) -> Result<String, ApplicationError>;
    fn get_story_log_entries(&self, ctx: GameServiceContext)
        -> Result<Vec<LogEntry>, ApplicationError>;
    fn get_input_status(&self, ctx: GameServiceContext)
        -> Result<(GenerationStatus, GenerationPhase), ApplicationError>;
    fn get_current_room_view(&self, ctx: GameServiceContext)
        -> Result<CurrentRoomView, ApplicationError>;
    fn get_npc_headshots(&self, ctx: GameServiceContext)
        -> Result<Vec<NpcPortraitView>, ApplicationError>;
    fn get_debug_state(&self, ctx: GameServiceContext)
        -> Result<GameStateDebugView, ApplicationError>;
    fn list_latest_llm_messages(&self, ctx: GameServiceContext, limit: usize)
        -> Result<Vec<LlmMessage>, ApplicationError>;
    fn get_generating_status(&self, ctx: GameServiceContext)
        -> Result<(GenerationStatus, GenerationPhase), ApplicationError>;
    fn reset_generating_status(&self, ctx: GameServiceContext) -> Result<(), ApplicationError>;
}
```

### Error Type

`ApplicationError` distinguishes:
- `Validation(String)` → 400 Bad Request
- `Engine(EngineError)` → 500 Internal Server Error
- `ShuttingDown` → 503 Service Unavailable
- `ConcurrentGeneration` → 503 Service Unavailable

### Wire-Up

`AppState` holds `application_service: Arc<dyn ApplicationService>`. All test constructors create both `game_service` and `application_service`.

## Consequences

### Positive

- Handlers are reduced to request parsing + delegation + HTTP response mapping
- No handler directly touches storage traits
- `ApplicationService` methods are unit-testable without an HTTP stack
- Storage imports can be banned from the server layer via arch-lint

### Negative

- One more layer of indirection
- `AppState` construction in tests requires both `game_service` and `application_service`

## Related

- `docs/architecture/system.md` (Application Tier)
- `docs/architecture/guardrails.md` (arch-lint `server -> storage` rule)
- `src/application/application_service.rs`
