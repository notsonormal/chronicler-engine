# chronicler_engine/src/model/

## Responsibility

Data structures for the game world: characters, maps, triggers, game state, and scenarios. All types are JSON-serializable DTOs loaded from data files at startup.

## Design

**Hierarchy:**
- `WorldCard` / `WorldManifest` (`world.rs`) — Top-level world metadata, global rules, default images. `WorldManifest` is the full JSON config; `WorldCard` is the runtime subset.
- `MapDef` → `Overworld` → `Region` → `Room` (`map.rs`) — Nested map structure. `Room` has exits (`HashMap<Direction, String>`), items, NPCs, image paths.
- `CharacterSheet` (`character.rs`) — Shared character data (name, description, personality, scenario, images). Used by both `PlayerCard` and `NpcCard`.
- `PlayerCard` / `NpcCard` (`character.rs`) — Character instances with inventory. NPCs additionally carry `Vec<Trigger>`.
- `GameState` (`state.rs`) — Runtime state: world/map/player (Arc-wrapped), NPC map, current room, narration history (capped at 1000 entries), next_log_id for unique log entries, NPCs in area, generation state (input buffer, cursor, scroll, error_message), dynamic rooms, character state.
- `GenerationState` (`state.rs`) — TUI input buffer with push_char/pop_char/clear_input methods. `GeneratingGuard` is an RAII guard that sets `is_generating = true` on construction and resets on drop. Includes `error_message: Option<String>` for LLM error handling.
- `LogEntry` (`state.rs`) — Timestamped log with id, sender, text, `LogType` (Narration/Dialogue/System/Input), and timestamp. Supports edit by ID.
- `LogType` (`state.rs`) — Enum with PartialEq: Narration, Dialogue, System, Input.
- `Trigger` / `TriggerCondition` / `TriggerAction` (`trigger.rs`) — Trigger system: `TimesMet(ComparisonOperator, u32)` condition, `narration_prompt` action, repeat flag.
- `NpcEncounterState` (`trigger.rs`) — Per-NPC state: `times_met` counter, `trigger_fired: HashMap<usize, bool>` for tracking which triggers have fired, `currently_meeting` flag.
- `CharacterState` (`trigger.rs`) — Tracks per-NPC `NpcEncounterState` in a HashMap.
- `StartingScenario` (`scenario.rs`) — Boot configuration: id, name, description, starting room, optional intro text.

**Patterns:**
- All types derive `Serialize`, `Deserialize`, `Debug`, `Clone`
- `Arc` wrapping for shared immutable data (world, map, player)
- `#[serde(default)]` for optional fields
- `#[serde(flatten)]` for `CharacterSheet` embedding in cards

## Flow

1. JSON data files → `serde` deserialization → typed structs
2. `GameState::new()` assembles world, map, player, NPCs into runtime state; initializes `times_met = 1` and `currently_meeting = true` for NPCs in the starting room
3. `CharacterState` / `NpcEncounterState` accumulate encounter counts during gameplay
4. `GenerationState` tracks TUI input between frames
5. Log methods: `add_log` (auto-capped at 1000), `edit_log`, `get_log`, `get_last_ai_response_index`, `get_last_input_index`, `get_history_context`, `get_history_context_for_retry`, `replace_last_ai_response`

## Integration

- **Consumed by**: All modules — engine (navigation, triggers), narrative (prompt building), server (state rendering)
- **Depends on**: `serde`, `chrono` (timestamps), `std::collections::HashMap`
