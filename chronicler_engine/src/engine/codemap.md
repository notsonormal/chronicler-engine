# chronicler_engine/src/engine/

## Responsibility

Game action parsing, command evaluation, navigation logic, trigger system, and game orchestration. Translates player text input into structured `Action` enums, resolves room lookups, processes movement, evaluates NPC triggers, and orchestrates the full action pipeline including LLM calls and NPC event handling.

## Design

**Key types:**
- `Action` enum (`action.rs`) — `Look`, `Inventory`, `Talk(String, Option<String>)`, `FreeAction(String)`, `Quit`
- `parse_command()` (`parser.rs`) — Tokenizes input, handles quoted messages for `talk`, case-insensitive matching
- `get_current_room()` (`logic.rs`) — Looks up room in static map first, then dynamic rooms, returns `Result<&Room>`
- `find_room_in_map()` (`logic.rs`) — Finds room in MapDef without checking dynamic_rooms (used for retry)
- `process_directional_movement()` (`logic.rs`) — Matches direction or semantic room name against exits
- `attempt_semantic_walk()` (`logic.rs`) — Direct room ID teleport by name lookup
- `create_dynamic_room()` (`logic.rs`) — Runtime room creation with timestamp-based ID
- `evaluate_triggers()` (`trigger_eval.rs`) — Iterates ALL NPCs in `state.npcs`, checks `TriggerCondition::TimesMet` against `CharacterState`
- `check_condition()` (`trigger_eval.rs`) — Evaluates comparison operators (`Eq`, `Lt`, `Gte`)
- `is_currently_meeting()` / `set_currently_meeting()` (`trigger_eval.rs`) — Tracks if player is currently with an NPC
- `get_times_met()` / `increment_times_met()` (`trigger_eval.rs`) — Counter for NPC encounter tracking
- `mark_trigger_fired()` / `is_trigger_fired()` (`trigger_eval.rs`) — Tracks non-repeatable trigger execution
- `get_static_npcs()` (`action_processing.rs`) — Filters NPCs from state by ID list
- `handle_movement()` (`action_processing.rs`) — Processes movement with dynamic room fallback, sets `currently_meeting` flags
- `apply_npc_events()` (`action_processing.rs`) — Applies `NpcEvent::Entered`/`Left` events to character state
- `evaluate_and_narrate_triggers()` (`action_processing.rs`) — Evaluates triggers and generates continuation narration via LLM
- `GameService` trait (`game_service.rs`) — Trait for game orchestration with `execute_action()` and `retry_last_response()`
- `DefaultGameService` (`game_service.rs`) — Default implementation handling full action pipeline

**Patterns:**
- Command parsing uses keyword matching with fallback to `FreeAction`
- Navigation supports both cardinal directions and semantic room name matching
- Trigger evaluation iterates ALL NPCs (not just `npcs_in_area`) to catch triggers for NPCs not currently in view
- Game orchestration uses trait-based design for testability, spawning threads for async LLM processing
- NPC event handling separates `Entered` (increments `times_met`) from movement-only updates (sets `currently_meeting` without increment)

## Flow

1. Player input → `parse_command()` → `Action`
2. Server dispatches to `GameService::execute_action()` → matches `Action` variant
3. For `FreeAction`: spawns thread → calls LLM → runs quantifier → `handle_movement()` → `evaluate_and_narrate_triggers()` → `apply_npc_events()`
4. For movement: `process_directional_movement()` or `attempt_semantic_walk()` updates `state.current_room_id`
5. For talk: `Action::Talk(npc, msg)` triggers dialogue generation
6. After action: `evaluate_triggers()` checks ALL NPCs, fires matching triggers with continuation narration
7. Trigger fires → `increment_times_met()` + `mark_trigger_fired()` (if non-repeatable) → continuation narration

## Integration

- **Consumes**: `model/` types (`GameState`, `Room`, `NpcCard`, `Trigger`, `CharacterState`, `NpcEvent`)
- **Produces**: `Action` variants, movement results, trigger pairs, narration text
- **Consumed by**: `server/` (action dispatch), `narrative/` (trigger continuations, quantifier)
- **Module exports**: `pub mod action; pub mod action_processing; pub mod game_service; pub mod logic; pub mod parser; pub mod trigger_eval;`
