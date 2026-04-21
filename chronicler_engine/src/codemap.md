# Chronicler Engine Source Code Map

**Directory**: `chronicler_engine/src/`
**Purpose**: Interactive fiction/text adventure game engine with LLM-powered narrative generation and HTMX web dashboard

---

## Architecture Overview

```
src/
├── lib.rs           # Library root - exports public API
├── main.rs          # Binary entry point - CLI + server startup
├── error.rs         # EngineError enum and Result type
├── engine/          # Game logic (action parsing, navigation, triggers)
├── model/           # Data structures (world, map, character, state)
├── narrative/       # LLM integration (prompts, clients, quantifier)
└── server/          # Axum HTTP server + HTMX templates
```

---

## Root Level

### `lib.rs`

**Responsibility**: Library root that re-exports public modules and types for external consumers.

**Design**:
- Module declarations: `engine`, `error`, `model`, `narrative`, `server`
- Public re-exports: `EngineError`, `Result`, `AppState`, `create_app_for_testing`

**Integration**:
- Consumers: `main.rs` (binary), tests, external crates

---

### `main.rs`

**Responsibility**: Binary entry point that handles CLI arguments, world loading, game state initialization, and server startup.

**Design**:
- CLI parser using `clap`: `--world`, `--list-worlds`, `--port`
- World loading from JSON files in `data/worlds/<world_id>/`
- `GameState` initialization with world, map, player, NPCs
- Spawns background thread for initial arrival narration (if no default scenario)
- Async server runtime using `tokio`

**Flow**:
1. Parse CLI args → load world manifest → load map.json, player.json, NPCs
2. Create `GameState` with starting room
3. Optionally add default scenario narration
4. Spawn background thread for arrival narration (LLM)
5. Start Axum HTTP server with game state

**Integration**:
- Dependencies: `engine`, `model`, `narrative`, `server` modules
- External: `clap`, `tokio`, `dotenv`, `env_logger`

---

### `error.rs`

**Responsibility**: Central error type for the entire engine.

**Design**:
- `EngineError` enum with variants: `Io`, `Serde`, `Navigation`, `LLM`, `Narrative`, `RoomNotFound`, `NpcNotFound`, `WorldNotFound`, `Config`, `Template`, `Internal`, `DataLoad`, `ContextOverflow`
- Uses `thiserror` for derive
- `Result<T> = std::result::Result<T, EngineError>`

**Integration**:
- Used throughout all modules for error propagation

---

## `engine/` Module

**Responsibility**: Core game logic - action parsing, navigation, room management, trigger evaluation

### `engine/mod.rs`

**Design**: Re-exports submodules: `action`, `logic`, `parser`, `trigger_eval`

---

### `engine/action.rs`

**Responsibility**: Player action enumeration

**Design**:
- `Action` enum: `Look`, `Inventory`, `Talk(String, Option<String>)`, `FreeAction(String)`, `Quit`
- Represents parsed player commands

**Integration**:
- Consumed by: `server/fragments.rs` (action handling)

---

### `engine/logic.rs`

**Responsibility**: Navigation and room management functions

**Design**:
- `find_room_in_world_map(state, target_id) -> Option<&Room>` - searches all regions
- `get_current_room(state) -> Result<&Room>` - gets current room (static or dynamic)
- `get_available_exits(state) -> Vec<String>` - lists exits from current room
- `process_directional_movement(state, target) -> Result<String>` - handles "north", "kitchen", etc.
- `attempt_semantic_walk(state, room_id) -> Result<String>` - direct room ID navigation
- `create_dynamic_room(name, description) -> Room` - creates runtime rooms

**Flow**:
- Navigation checks exits for direction or semantic match (room name contains target)
- Falls back to dynamic room creation if destination not found

**Integration**:
- Consumers: `server/fragments.rs` (movement handling)

---

### `engine/parser.rs`

**Responsibility**: Command parsing from user input

**Design**:
- `parse_command(input: &str) -> Action`
- Handles quoted strings for dialogue: `talk carla "Hello there"`
- Aliases: `l`/`look`, `i`/`inv`/`inventory`, `t`/`talk`, `q`/`quit`/`exit`
- Case-insensitive matching
- Unknown commands → `FreeAction` (sent to LLM)

**Flow**:
- Input → tokenize → match first token → extract arguments → return Action

**Integration**:
- Consumers: `server/fragments.rs` (action_handler)

---

### `engine/trigger_eval.rs`

**Responsibility**: Trigger condition evaluation and execution

**Design**:
- `evaluate_triggers(state) -> Vec<(NpcCard, Trigger)>` - finds matching triggers for NPCs in area
- `check_condition(character_state, npc_id, condition) -> bool` - evaluates trigger conditions
- `increment_times_met(state, npc_id)` - tracks NPC encounter count
- `mark_trigger_fired(state, npc_id, trigger_index)` - marks non-repeatable triggers as fired

**Flow**:
1. For each NPC in area, check each trigger's condition
2. If condition met and trigger is repeatable or not yet fired, return it
3. Caller (fragments.rs) builds continuation prompt and calls LLM

**Integration**:
- Consumers: `server/fragments.rs` (trigger narration)

---

## `model/` Module

**Responsibility**: Data structures for game state, world data, characters

### `model/mod.rs`

**Design**: Re-exports: `character`, `map`, `scenario`, `state`, `trigger`, `world`

---

### `model/character.rs`

**Responsibility**: Character definitions (player and NPCs)

**Design**:
- `CharacterSheet`: name, description, personality, scenario, example_dialogue, profile_image, headshot_image
- `PlayerCard`: flattens CharacterSheet + inventory
- `NpcCard`: id + flattened CharacterSheet + inventory + triggers
- `preferred_image()` method - returns headshot or profile

**Integration**:
- Used by: `narrative/prompt.rs`, `server/fragments.rs`

---

### `model/map.rs`

**Responsibility**: World map structure

**Design**:
- `MapDef`: contains `Overworld`
- `Overworld`: id, name, regions
- `Region`: id, name, rooms
- `Room`: id, name, description, exits (HashMap<Direction, String>), items, npcs, image_path, navigation_description
- `Direction` enum: North, South, East, West, Up, Down

**Integration**:
- Used by: `engine/logic.rs`, `narrative/prompt.rs`, `server/fragments.rs`

---

### `model/scenario.rs`

**Responsibility**: Starting scenario definitions

**Design**:
- `StartingScenario`: id, name, description, starting_room_id, text (optional narration)

**Integration**:
- Used by: `model/world.rs`, `main.rs`

---

### `model/state.rs`

**Responsibility**: Runtime game state

**Design**:
- `LogType`: Narration, Dialogue, System, Input
- `LogEntry`: sender, text, log_type, timestamp
- `GenerationState`: input, cursor_position, scroll_offset, is_generating, error_message
- `GeneratingGuard` - RAII guard that sets is_generating=true on drop
- `GameState`: world, map, player, npcs (HashMap), current_room_id, narration_history, npcs_in_area, generation_state, dynamic_rooms, character_state

**Flow**:
- `add_log()` adds entries with FIFO removal at MAX_LOG_ENTRIES (1000)
- `GeneratingGuard` ensures is_generating is reset when thread completes

**Integration**:
- Central to: `server/fragments.rs`, `main.rs`

---

### `model/trigger.rs`

**Responsibility**: Trigger system for NPC event-driven narration

**Design**:
- `ComparisonOperator`: Eq, Lt, Gte
- `TriggerCondition`: TimesMet(ComparisonOperator, u32)
- `TriggerAction`: narration_prompt
- `Trigger`: condition, action, repeat
- `NpcEncounterState`: times_met, trigger_fired (HashMap)
- `CharacterState`: npcs (HashMap<String, NpcEncounterState>)

**Integration**:
- Used by: `engine/trigger_eval.rs`, `model/character.rs`

---

### `model/world.rs`

**Responsibility**: World and manifest definitions

**Design**:
- `WorldCard`: name, description, global_rules, default_room_image
- `WorldManifest`: id, name, description, global_rules, starting_room_id, map_file, player_file, scenarios, default_scenario_id, default_room_image
- `default_scenario()` - returns first scenario
- `From<WorldManifest> for WorldCard` conversion

**Integration**:
- Used by: `main.rs`, `narrative/prompt.rs`

---

## `narrative/` Module

**Responsibility**: LLM integration - prompt building, API clients, quantifier

### `narrative/mod.rs`

**Design**: Re-exports: `continuation`, `llm`, `openrouter_client`, `prompt`, `quantifier`

---

### `narrative/llm.rs`

**Responsibility**: LLM backend trait and implementations

**Design**:
- `LlmBackend` trait: `generate_dialogue()`, `narrate_action()`, `narrate_arrival()`, `name()`
- `LlmBackendType`: OpenRouter, DeepSeek, Mock
- `get_llm_backend()` - factory based on `LLM_BACKEND` env var
- Implementations:
  - `OpenRouterBackend` - calls OpenRouter API
  - `DeepSeekBackend` - placeholder
  - `MockBackend` - test implementation with keyword detection

**Flow**:
- `generate_dialogue()` - NPC response to player
- `narrate_action()` - consequence of player action
- `narrate_arrival()` - initial room description

**Integration**:
- Consumers: `server/fragments.rs`, `main.rs`

---

### `narrative/openrouter_client.rs`

**Responsibility**: HTTP client for OpenRouter API

**Design**:
- `get_llm_model()` - reads `LLM_MODEL` env var (default: openai/gpt-4o-mini)
- `get_quantifier_model()` - reads `QUANTIFIER_MODEL` env var
- `call_openrouter(api_key, system_prompt, user_text)` - convenience wrapper
- `call_openrouter_with_model(api_key, system_prompt, user_text, model)` - core function
- Robust response parsing: tries content, reasoning, reasoning_content fields

**Flow**:
- Builds JSON payload → POST to openrouter.ai → parse response → extract content

**Integration**:
- Used by: `llm.rs`, `quantifier.rs`

---

### `narrative/prompt.rs`

**Responsibility**: Prompt construction for LLM calls

**Design**:
- `PromptLayer` enum: System(0), GameState(1), NpcCards(2), Player(3), WorldInfo(4), History(5), User(6), Phi(7)
- Token budget constants: MAX_CONTEXT_TOKENS (8192), MAX_HISTORY_TOKENS (4096), MAX_SYSTEM_TOKENS (1024), MAX_RESPONSE_TOKENS (512)
- `PromptContext`: world, room, all_npcs, npcs_in_area, player, user_message, history
- `PromptBuilder`: constructs layered prompts
  - `build()` - single combined prompt
  - `build_split()` - system + user prompts (for OpenAI format)
  - `build_system_only()`, `build_user_only()`
- `sanitize_for_prompt()` - filters `{{...}}` injection patterns
- `estimate_tokens()` - rough estimation (chars/4)
- `truncate_to_budget()` - keeps most recent text

**Flow**:
- Layers rendered in order: System → GameState → NpcCards → Player → WorldInfo → History → User → Phi

**Integration**:
- Consumers: `llm.rs`, `server/fragments.rs`, `continuation.rs`

---

### `narrative/continuation.rs`

**Responsibility**: Prompt building for trigger-driven continuation narration

**Design**:
- `build_continuation_prompt(context, first_narration, trigger_text) -> (system, user)`
- `truncate_first_narration()` - truncates with ellipsis
- `build_room_context()` - compact room info

**Flow**:
- Used when trigger fires: builds prompt incorporating previous narration + trigger event

**Integration**:
- Consumers: `server/fragments.rs`

---

### `narrative/quantifier.rs`

**Responsibility**: LLM-powered NPC presence detection and movement intent detection

**Design**:
- `QuantifierConfidence`: High, Medium, Low
- `QuantifierParseResult`: npc_ids, confidence
- `MovementType`: Entering, In, Leaving
- `MovementParseResult`: movement_type, destination, confidence
- `QuantifierResult`: npcs (QuantifierParseResult), movement (MovementParseResult)
- `QuantifierPromptBuilder`: builds system + user prompts
- `parse_quantifier_response()` - JSON or text fallback
- `parse_quantifier_response_with_movement()` - includes movement parsing
- `QuantifierBackend::quantify_room()` - calls LLM and parses response

**Flow**:
1. Build prompt with all known NPCs, rooms, history
2. Call quantifier model via OpenRouter
3. Parse JSON response (falls back to text matching)
4. Return NPC IDs and movement intent

**Integration**:
- Consumers: `server/fragments.rs`

---

## `server/` Module

**Responsibility**: HTTP server with HTMX partial updates

### `server/mod.rs`

**Responsibility**: Axum router setup and server startup

**Design**:
- `ServerConfig`: port
- `AppState`: wraps Arc<Mutex<GameState>>
- `create_app_for_testing()` - test router with all routes
- `run_server()` / `run_server_with_config()` - async server startup
- `bind_with_retry()` - handles port conflicts
- Routes:
  - `GET /` - index.html
  - `GET /fragment/*` - HTMX partials
  - `POST /action` - command submission
  - `GET /hints` - available actions
  - `GET /status/*` - generation status
  - `/assets`, `/data` - static file serving

**Integration**:
- Dependencies: `fragments.rs`, `templates.rs`

---

### `server/fragments.rs`

**Responsibility**: Request handlers and game logic coordination

**Design**:
- HTMX fragment handlers: `header_fragment`, `story_log_fragment`, `visual_sidebar_fragment`, `action_area_fragment`, `character_headshots_fragment`
- `action_handler()` - main command processor
- `determine_npcs_in_room()` - uses quantifier or static NPCs
- `handle_movement()` - processes movement from quantifier
- `evaluate_and_narrate_triggers()` - evaluates and executes triggers
- `process_sync_action()` - Look, Inventory, Quit
- `process_action()` - async action processing in spawned thread

**Flow**:
1. `action_handler` receives command → parse → determine sync/async
2. Sync actions: process immediately, return with HX-Trigger
3. Async actions: spawn thread → call LLM → quantifier → triggers → update state
4. Return status (ready/thinking)

**Integration**:
- Coordinates: engine (parser, logic, trigger_eval), model (state), narrative (llm, prompt, continuation, quantifier)

---

### `server/templates.rs`

**Responsibility**: Askama HTML templates for HTMX fragments

**Design**:
- `SafeHtml` - wrapper for trusted HTML (used in LogEntryView)
- `markdown_to_html()` - converts markdown to HTML with XSS protection
- Templates:
  - `HeaderTemplate` - room name display
  - `StoryLogTemplate` - narration history
  - `VisualSidebarTemplate` - room image + NPC portraits
  - `CharacterHeadshotsTemplate` - all NPC headshots
  - `ActionAreaTemplate` - command input + hints + status
- `LogEntryView` - converts LogEntry to display format

**Integration**:
- Consumers: `fragments.rs` (render_* functions)

---

## Data Flow Summary

```
User Input (HTTP POST /action)
    │
    ▼
server/fragments.rs::action_handler
    │
    ├─► engine/parser::parse_command
    │       │
    │       ▼
    │       Action enum
    │
    ├─► process_sync_action (Look/Inventory/Quit)
    │       │
    │       ▼
    │       GameState::add_log
    │
    └─► process_action (spawned thread)
            │
            ├─► narrative/llm::narrate_action
            │       │
            │       ├─► narrative/prompt::PromptBuilder
            │       │       │
            │       │       ▼
            │       │       system + user prompts
            │       │
            │       └─► narrative/openrouter_client::call_openrouter
            │               │
            │               ▼
            │               narration text
            │
            ├─► narrative/quantifier::determine_npcs_in_room
            │       │
            │       ├─► QuantifierBackend::quantify_room
            │       │       │
            │       │       ▼
            │       │       NPC IDs + movement intent
            │       │
            │       └─► static fallback if no API key
            │
            ├─► handle_movement (if detected)
            │       │
            │       └─► engine/logic::attempt_semantic_walk
            │
            ├─► engine/trigger_eval::evaluate_triggers
            │       │
            │       └─► narrative/continuation::build_continuation_prompt
            │               │
            │               ▼
            │               continuation narration
            │
            └─► GameState::add_log (narration + NPCs in area)
                    │
                    ▼
            HTMX fragment response
```

---

## Key Patterns

1. **Error Handling**: All fallible operations return `Result<T, EngineError>`
2. **State Management**: `Arc<Mutex<GameState>>` shared across threads
3. **LLM Abstraction**: Trait-based backend (Mock, OpenRouter, DeepSeek)
4. **Prompt Layering**: 8-layer prompt system inspired by SillyTavern
5. **HTMX Partial Updates**: Each route returns HTML fragment, not full page
6. **Token Budgeting**: Context truncation to stay within LLM limits
7. **Trigger System**: Condition-action pairs with repeat semantics

---

## External Dependencies

- **Web**: `axum`, `tower-http`, `tokio`
- **Templates**: `askama`, `pulldown_cmark`
- **LLM**: `reqwest` (blocking)
- **Serialization**: `serde`, `serde_json`
- **Error**: `thiserror`
- **CLI**: `clap`
- **Logging**: `log`, `env_logger`
- **Time**: `chrono`
- **Regex**: `regex`, `once_cell`