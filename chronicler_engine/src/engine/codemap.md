# chronicler_engine/src/engine/

## Responsibility

Game action parsing, command evaluation, navigation logic, and trigger system. Translates player text input into structured `Action` enums, resolves room lookups, processes movement, and evaluates NPC triggers.

## Design

**Key types:**
- `Action` enum (`action.rs`) — `Look`, `Inventory`, `Talk(String, Option<String>)`, `FreeAction(String)`, `Quit`
- `parse_command()` (`parser.rs`) — Tokenizes input, handles quoted messages for `talk`, case-insensitive matching
- `get_current_room()` (`logic.rs`) — Looks up room in static map first, then dynamic rooms, returns `Result<&Room>`
- `process_directional_movement()` (`logic.rs`) — Matches direction or semantic room name against exits
- `attempt_semantic_walk()` (`logic.rs`) — Direct room ID teleport by name lookup
- `create_dynamic_room()` (`logic.rs`) — Runtime room creation with timestamp-based ID
- `evaluate_triggers()` (`trigger_eval.rs`) — Iterates NPCs in area, checks `TriggerCondition::TimesMet` against `CharacterState`
- `check_condition()` (`trigger_eval.rs`) — Evaluates comparison operators (`Eq`, `Lt`, `Gte`)
- `increment_times_met()` / `mark_trigger_fired()` — State mutation helpers

**Patterns:**
- Command parsing uses keyword matching with fallback to `FreeAction`
- Navigation supports both cardinal directions and semantic room name matching
- Trigger evaluation is stateless (reads `CharacterState`, returns matching triggers)

## Flow

1. Player input → `parse_command()` → `Action`
2. Server matches `Action` variant → calls appropriate logic function
3. For movement: `process_directional_movement()` or `attempt_semantic_walk()` updates `state.current_room_id`
4. For talk: `Action::Talk(npc, msg)` triggers dialogue generation
5. After action: `evaluate_triggers()` checks if any NPC triggers fire
6. Trigger fires → `increment_times_met()` + `mark_trigger_fired()` → continuation narration

## Integration

- **Consumes**: `model/` types (`GameState`, `Room`, `NpcCard`, `Trigger`, `CharacterState`)
- **Produces**: `Action` variants, movement results, trigger pairs
- **Consumed by**: `server/` (action dispatch), `narrative/` (trigger continuations)
