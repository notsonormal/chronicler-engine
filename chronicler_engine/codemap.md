# chronicler_engine/

## Responsibility

Rust game engine for interactive fiction/text adventures. Provides HTTP/WebSocket server with HTMX dashboard, LLM-powered narrative generation, and data-driven game state from JSON configs.

## Design

- **Edition**: Rust 2024 (requires 1.85+)
- **Structure**: Single crate (binary + library)
- **Key modules**: engine/, model/, narrative/, server/, ui/
- **Data-driven**: World, map, character, triggers defined in JSON

## Flow

1. Load JSON world data → `model/` types
2. Parse player actions → `engine/` actions
3. Evaluate triggers → execute logic
4. Generate narrative via `narrative/` (LLM)
5. Serve via `server/` (HTTP/WebSocket)

## Module Structure

### engine/ (Game Logic)

| File | Responsibility |
|------|--------------|
| `mod.rs` | Module exports |
| `action.rs` | Action enum (WalkTo, Look, Talk, FreeAction, Inventory, Quit) |
| `action_processing.rs` | Pure functions for NPC/event handling (get_static_npcs, handle_movement, apply_npc_events, evaluate_and_narrate_triggers) |
| `game_service.rs` | Game orchestration (GameService trait, DefaultGameService - handles thread spawning for async LLM calls) |
| `logic.rs` | Core game logic (attempt_semantic_walk, get_current_room, find_room_in_map, create_dynamic_room) |
| `parser.rs` | Command parsing (parse_command → Action enum) |
| `trigger_eval.rs` | Trigger evaluation (evaluate_triggers, mark_trigger_fired, increment_times_met, is_currently_meeting) |

### model/ (Data Structures)

| File | Responsibility |
|------|--------------|
| `mod.rs` | Module exports |
| `character.rs` | CharacterSheet, NpcCard, PlayerCard |
| `map.rs` | MapDef, Overworld, Region, Room, Direction |
| `scenario.rs` | Scenario data |
| `state.rs` | GameState (core game state including narration_history, character_state, generation_state) |
| `trigger.rs` | Trigger, TriggerAction |
| `world.rs` | WorldCard |

### narrative/ (LLM Integration)

| File | Responsibility |
|------|--------------|
| `mod.rs` | Module exports |
| `llm.rs` | LlmBackend trait, get_llm_backend() factory |
| `openrouter_client.rs` | OpenRouter API client |
| `prompt.rs` | PromptBuilder, PromptContext, PhiMode |
| `quantifier.rs` | NPC presence detection (QuantifierBackend, NpcEvent, compute_npc_events) |

### server/ (HTTP/WebSocket)

| File | Responsibility |
|------|--------------|
| `mod.rs` | AppState, ServerConfig, run_server() |
| `fragments.rs` | HTMX fragment handlers |
| `templates.rs` | HTML templates |

## Integration

- **Parent**: mrn-general workspace
- **Downstream**: Docker service (`docker-compose.yml`)