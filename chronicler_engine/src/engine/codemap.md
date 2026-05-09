# chronicler_engine/src/engine/

## Responsibility
The game logic tier. Translates player intent into state mutations and narrative requests. Handles command parsing, movement resolution, trigger evaluation, action processing orchestration, and the main game service that coordinates the engine-narrative-server pipeline.

## Design Patterns
- **Command Pattern**: `Action` enum (`Look`, `Inventory`, `Talk`, `FreeAction`, `Quit`) encapsulates all player intents.
- **Strategy Pattern**: `GameService` trait with `DefaultGameService` implementation allows swapping backends for testing.
- **State Machine**: `GenerationStatus` (`Idle`/`Generating`/`Error`) and `GenerationPhase` (`Narrating`/`Quantifying`/`GeneratingEvent`) track the async generation lifecycle.
- **Three-Phase Lock/Unlock**: Trigger processing uses evaluate (lock) → LLM call (unlock) → commit (lock) to avoid blocking frontend polling.

## Data & Control Flow
```
Player Input → parse_command() → Action enum
  → GameService.execute_action() → spawn async thread
    → Phase 1: Narrating
      → LLMBackend.narrate_action_from_prompt() → narration_text
    → Phase 2: Quantifying
      → QuantifierBackend.analyze() → QuantifierResult
      → handle_movement() → update current_room_id
    → Phase 3: Trigger Evaluation (if applicable)
      → build_trigger_request() → TriggerContinuationRequest
      → Release lock → LLM call for continuation
      → Re-acquire lock → commit_trigger_narration()
    → Phase 4: Reset status → Idle
```

## Integration Points
- **Consumes**: `model/` (state, characters, map, triggers), `narrative/` (LLM, quantifier, prompts)
- **Consumed by**: `server/` (HTTP handlers call `GameService`)

## Files
| File | Purpose |
|------|---------|
| `action.rs` | `Action` enum — all supported player intents |
| `parser.rs` | `parse_command()` — natural language → `Action` |
| `logic.rs` | Movement resolution, room lookup, dynamic room creation |
| `trigger_eval.rs` | Pure trigger evaluation against `CharacterState` |
| `action_processing.rs` | `execute_freeaction_impl()`, `handle_movement()`, `commit_trigger_narration()` |
| `game_service.rs` | `GameService` trait and `DefaultGameService` — async orchestration |
| `mod.rs` | Module exports and test module declarations |
