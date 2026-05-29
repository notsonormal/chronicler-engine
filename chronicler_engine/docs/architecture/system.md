# Specification: Core Architecture (Modular)

## Objective
Establish a domain-driven modular architecture for the Chronicler Engine. This structure separates core data models, game mechanics, application orchestration, narrative processing, and user interface logic.

## Module Domains

### 1. The Model Tier (`crate::model::*`)
Contains pure data structures, serialization schemas, and the "Single Source of Truth" for game state. This tier has zero knowledge of the UI or LLM logic.
- **`world`**: Setting lore, global rules, and starting scenarios.
- **`map`**: Room/Region hierarchy and cardinal direction definitions.
- **`character`**: NPC attributes (name, description, personality, scenario, **profile_image**, **headshot_image**) and Player inventory. `inventory` lives on `PlayerCard`/`NpcCard`, not on the shared `CharacterSheet`.
- **`state`**: The `GameState` aggregation, narration history logs, and TUI state. `NarrativeState` holds a `MessageHistory` which encapsulates `Vec<Message>` where each `Message` is an independent narrative unit (input, narration, dialogue, or system). Each `Message` carries a `swipes: Vec<Swipe>` set — alternate generations preserved during retry, with `active_swipe_index` selecting the currently displayed version. `LogEntry` remains the atomic rendering unit for templates and prompts, now carrying `swipe_count` and `active_swipe_index` for swipe control rendering. `StoredTriggerContext` enables replaying trigger continuations on retry or retrigger. `LogEntry` carries optional `location_header` and `event_header` metadata for visual rendering; `NarrativeState` tracks `pending_location` and `pending_event` for consumption by the next `add_log` call. `GameState::current_room()` resolves the player's active room from the map or dynamic rooms.
- **`scenario`**: Starting scenario definitions for narrative introductions.
- **`trigger`**: Trigger definitions, conditions, and NPC encounter tracking (`Trigger`, `TriggerCondition`, `TriggerEffect`, `NpcEncounterState`, `NpcEncounterLog`).
- **`settings`**: `AppSettings`, `Connection`, and agent configuration data models.
- **`agent`**: `AgentConfig`, `AgentResult`, `AgentContext`, `StatePatch`, `ExecutionPhase`, `BackendSelector`, `Confidence`. `AgentContext` carries the current room for agents that need spatial awareness.
- **`quantifier`**: `QuantifierResult`, `QuantifierParseResult`, `MovementParseResult`, `QuantifierConfidence`, `NpcEvent`, `NpcEventType`, `NpcEventList`. Mechanical result types produced by the narrative quantifier but consumed by the engine tier. Living in `model` prevents engine→narrative coupling.
- **`llm_backend`**: `LlmBackendType` enum for backend selection.
- **`llm_message`**: `LlmMessage` struct for LLM call forensics — agent name, backend, model, prompts, raw request/response JSON, parsed response, error, timestamp.
- **`state_snapshot`**: `GameStateSnapshot` for SQLite persistence. Snapshots are standalone state blobs with an auto-increment `id`. Each message stores `snapshot_id` referencing the snapshot saved after it was created.

### 2. The Engine Tier (`crate::engine::*`)
Contains the mechanics that drive the simulation. It translates user intent and state into outcomes.
- **`parser`**: Command wrapping — all input becomes `Action::FreeAction(String)`.
- **`action`**: The `Action` enum defining all supported system intents.
- **`logic`**: Rules for movement, fuzzy-matching, and room resolution.
- **`trigger_eval`**: Pure function evaluation of NPC triggers based on character state and room location (`evaluate_triggers(state) -> Vec<(NpcCard, Trigger, usize)>`). Triggers with `room_id` only fire in that room.
- **`action_processing`**: Extracted pure functions for server handlers (`handle_movement`, `apply_npc_events`, `commit_trigger_narration`, `execute_freeaction_impl`). `execute_freeaction_impl` evaluates triggers before applying NPC events and returns `TriggerMatch` data for the application tier to build continuation prompts. Enables unit testing of server-side logic.
- **`state_diagnostics`**: Runtime invariant checks (`INV-ROOM`, `INV-NPC`, `INV-CHAR`, `INV-LOG`), feature-flagged via `diagnostics` feature.

### 2.5. The Application Tier (`crate::application::*`)
Orchestration layer that coordinates game flow, persistence, and LLM generation. Sits between the HTTP server and the pure simulation engine.
- **`context`**: Shared infrastructure for game service operations.
  - `GameServiceContext`: Storage (unified SQLite/in-memory/test backend), preset storage, world/map/player/npc references, cancellation token, settings.
  - `context.rs`: Shared persistence helpers (`load_state`, `save_state`, `save_message_and_snapshot`, `map_llm_error`). Cross-storage coordination helpers (`load_messages`, `update_message_text`, `migrate_swipes`).
  - `save_message_and_snapshot()`: Saves a snapshot and immediately persists the newest unpersisted message with the snapshot ID. Messages are persisted as they are created; there is no batching or `committed` flag.
- **`game_lifecycle.rs`**: Game lifecycle operations - `create_game`, `switch_game`, `delete_game`, `list_games`, `current_game_id`, `reset`.
- **`message_editing.rs`**: Message editing operations - `switch_swipe`, `edit_history`, `delete_last`, `retry`, `retrigger`.
- **`query_handlers.rs`**: Read-only query operations - `get_generating_status`, `get_current_game_name`, `list_latest_llm_messages`, `get_story_log_entries`, `get_input_status`, `get_current_room_view`, `get_npc_headshots`, `get_debug_state`.
- **`action_pipeline`**: Action-processing workflows and the `ActionPipeline` orchestration struct.
  - `pipeline.rs`: `ActionPipelineBackend` trait (narrow seam: `assembler()`, `complete`, `run_post_generation_agents`) and `ActionPipeline<'a, B>` generic over the trait. Encapsulates the full FreeAction pipeline (narrate → quantify → triggers → event continuation) with explicit phase methods. Used by both normal action handling (`run_from_input`) and retry logic (`run_trigger_continuation`). Checks `CancellationToken::is_cancelled()` at stage boundaries and aborts gracefully via `handle_cancellation()` to avoid wasted LLM calls on stale requests.
  - `actions.rs`: Thin dispatch layer — `execute_action_impl` creates `ActionPipeline` and delegates to `run_from_input`.
  - `retry.rs`: Retry-specific setup (anchor finding, message deletion, snapshot loading) delegates continuation regeneration to `ActionPipeline::run_trigger_continuation()` and main narration retry to `ActionPipeline::run_from_input()`.
- **`game_service`**: Service boundary — `DefaultGameService` struct with inherent methods (`execute_action`, `retry_last_response`, `retrigger_event`). Implements `ActionPipelineBackend` trait to wire internal backends to the pipeline seam.
- **`application_service`**: Thin orchestrator struct (`DefaultApplicationService`) delegating to submodules. Contains `process_action` entry point and `GenerationGuard` RAII helper.

### 3. The Narrative Tier (`crate::narrative::*`)
The interface between the synchronous engine and stochastic LLM generation.
- **`llm`**: Directory module with traits (`LlmBackend`) and per-provider implementations (OpenRouter, DeepSeek stub, Ollama, Mock) for Game Master narration. The `LlmBackend` trait exposes transport primitives: `model()`, `name()`, `save_message()`, `wrap_and_save()`, `narrate_continuation()`, `complete()`. Backend-specific preprocessing (`preprocess_user_text`) and postprocessing (`postprocess_response_text`) hooks allow model-specific hacks (e.g., Gemma 4 thinking suffix, response sanitization) to live in the provider modules instead of the generic HTTP client.
  - **`get_llm_backend_for(connection, storage, settings)`**: Create a backend for a specific `Connection` profile. Settings are passed in — no file I/O inside the backend.
  - **`DefaultGameService::with_storage(storage, settings)`**: Production constructor that receives pre-loaded settings.
  - **`DefaultGameService::with_backends(llm, registry)`**: Constructor for dependency-injecting mock backends and agent registry in tests. No globals, no file I/O, fully isolated.
- **`prompt`**: Directory module for layered prompt construction with token budget management. Uses XML-sectioned instructions + XML-wrapped data for reasoning-model compatibility. The `PromptAssembler` trait decouples prompt assembly from LLM transport. Includes `fit_messages_to_context()` for dynamic context-window fitting.
- **`agents`**: Directory module for the agent trait, registry, and agent implementations.
  - **`Agent` trait**: Core abstraction for pre-generation and post-generation agents
  - **`AgentRegistry`**: Loads agents from config and iterates by execution phase
  - **`QuantifierAgent`**: Post-generation agent for scene quantification and dynamic room presence detection
  - **`NarratorAgent`**: Stub pre-generation agent (reserved for future use)
- **`quantifier`** (under `agents/`): Quantifier implementation module.
  - **`QuantifierAgent`**: Post-generation agent that uses `LlmBackend::complete()` for scene quantification. Receives the current room via `AgentContext.current_room`.
  - **`NpcEventList`**: NPC movement events from quantification (Entered, Left). Now lives in `model::quantifier`.
- **`text_check`**: Directory module for spell and grammar checking of player input.
  - **`HarperBackend`**: Wraps harper-core with curated + user dictionaries
  - **`check_player_input()`**: Facade that returns `Option<CheckResult>` based on `TextCheckMode`
  - **`CheckResult`/`CheckIssue`**: Structured lint results with byte spans and suggestions
- **`llm_client`**: HTTP client helpers for OpenRouter and Ollama.

#### NPC Event Layer

Quantifier results include NPC movement events:

| Event | Trigger |
|-------|---------|
| `Entered` | NPC transitions from NOT in area → in area |
| `Left` | NPC transitions from in area → NOT in area |

When `Entered` fires: `currently_meeting = true`  
When `Left` fires: `currently_meeting = false`  

**times_met semantics**: Only increments on `Entered` (first encounter or NPC rejoins after leaving). Not on continuous presence across turns.

### 4. The Server Tier (`crate::server::*`)
The HTTP layer for the HTMX web dashboard with polling-based real-time updates.

**Layer Boundary:** The server tier must never access `GameState` directly. All reads go through the `ApplicationService` trait (`get_story_log_entries`, `get_input_status`, `get_current_room_view`, `get_npc_headshots`, `get_debug_state_view`, etc.). All writes go through `ApplicationService` command methods (`process_action`, `retry`, `reset`, etc.). This keeps the HTTP layer decoupled from domain state structure.

- **`mod`**: Axum router, request handlers, `AppState`, `run_server_with_config`. Test constructors (`create_app_for_testing`, `create_app_for_testing_with_settings`) live in `test_support/server_helpers.rs`.
- **`fragments`**: HTML fragment generators for HTMX partial updates. Split into submodules:
  - **`actions`**: Action form handlers and renderers
  - **`endpoints`**: HTMX fragment endpoints (`/fragment/story-log`, `/fragment/visual-sidebar`, etc.)
  - **`games`**: Game management fragment endpoints
  - **`generation_guard`**: Generation lock/status fragment endpoints
  - **`history`**: History editing, deletion, and retry endpoints
  - **`misc`**: Utility endpoints (status, hints, text check)
  - **`renderers`**: HTML rendering helpers, markdown→HTML via `pulldown-cmark`
- **`settings_fragment`**: Settings panel fragment handlers and template rendering.
- **`prompt_presets_fragment`**: Prompt Presets panel with two independent collections (System, Quantifier). Supports CRUD operations, active selection, and protected default presets.
- **`view_models`**: View model structs that decouple templates from domain types.
  - `view_models.rs`: `LogEntryView`, `LlmMessageView`, `PreviewIssueView`, `ActionAreaViewModel`, `VisualSidebarViewModel`, `NpcPortraitView`, and `SafeHtml` / `markdown_to_html`.
  - Domain-to-view mapping lives here; `templates.rs` focuses purely on HTML.
- **`templates`**: Askama template definitions with type-safe rendering.
  - Templates declare required data shapes at compile time.
  - Missing fields = compiler error (not runtime failure).
- **`debug`**: Dev diagnostic endpoint (`/debug/state`).

### 5. The Settings Tier (`crate::settings` + `crate::model::settings`)
Persistent JSON-based settings system for LLM configuration with reusable connection profiles.

| Component | Purpose |
|-----------|---------|
| `data/settings.json` | Persistent settings file |
| `AppSettings` struct | Configuration data model (connections, agents, prompt presets, text check settings) |
| `Connection` struct | Named provider+model profile |
| `AppState.settings` | Runtime access via `Arc<RwLock<AppSettings>>` |

#### Settings Flow

Settings are loaded **once** at startup and passed down through the construction chain:

```mermaid
flowchart TD
    A["bootstrap/run.rs"] --> B["load_settings() — ONCE"]
    B --> C["Arc<RwLock<AppSettings>>"]
    C --> D["AppState.settings"]
    D --> E["DefaultGameService::with_storage(storage, settings)"]
    E --> F["get_llm_backend_for(connection, storage, settings)"]
    E --> G["AgentRegistry::from_configs_with_storage(configs, storage, &settings)"]
    G --> H["QuantifierAgent::from_config_with_storage(config, storage, &settings)"]
```

- Backends store `Option<Arc<RwLock<AppSettings>>>` for settings access.
- Connection changes still require a server restart (Approach A).
- Only `max_context_tokens` is read dynamically at runtime.

#### Configuration Options

| Setting | Type | Default |
|---------|------|---------|
| `connections` | `Vec<Connection>` | Three default connections (OpenRouter GPT-4o Mini, OpenRouter Euryale, Ollama Gemma) |
| `narration_connection_id` | string | `"openrouter-gpt-4o-mini"` |
| `quantifier_connection_id` | string | `"openrouter-gpt-4o-mini"` |
| `text_check` | `TextCheckSettings` | Spell/grammar check config |
| `agents` | `Vec<AgentConfig>` | Agent registry config |
| `active_system_prompt_preset_id` | string | Active system prompt preset |
| `active_quantifier_prompt_preset_id` | string | Active quantifier preset |
| `active_system_prompt` | `Option<String>` | Runtime override (serde-skipped) |
| `active_quantifier_prompt` | `Option<String>` | Runtime override (serde-skipped) |

#### Connection Context Windows

Each connection can specify a `max_context_tokens` value. When unset, defaults are resolved by provider:

| Provider | Default `max_context_tokens` |
|----------|------------------------------|
| `ollama` | 8192 |
| `openrouter` / `deepseek` | 32768 |
| `mock` | 4096 |

Each `Connection` contains: `id`, `name`, `provider`, `model`, `api_key` (optional), `base_url` (optional), `single_user_message` (optional, default `false`), `max_tokens` (optional), `max_context_tokens` (optional).

#### Environment Fallback

- `OPENROUTER_API_KEY` env var used as fallback when connection `api_key` is None
- `LLM_BACKEND` env var is **not** consulted (settings file is sole source of truth)

### 6. The Error Tier (`crate::error`)
Unified error type shared across all tiers.
- **`EngineError`**: Top-level error enum (`Llm`, `Narrative`, `Internal`, `Io`, `Serde`, `Parse`, `Serialize`, `Navigation`, `RoomNotFound`, `NpcNotFound`, `WorldNotFound`, `Config`, `Template`, `DataLoad`, `ContextOverflow`)
- **`LlmFailure`**: LLM-specific errors (`EmptyResponse`, `Http`, `Network`, `ParseError`, `Timeout`)
- **`NarrativeFailure`**: Prompt build and generation failures
- **`InternalError`**: Invariant violations

### 7. The Storage Tier (`crate::storage`)
SQLite-based persistence for game state and LLM call forensics.
- **`db`**: Database connection and schema management. Schema includes:
  - `games` — top-level game session record (`id`, `name`, `world_name`, `created_at`, `updated_at`)
  - `game_state_snapshots` — serialized game state metadata, scoped to `game_id`
  - `messages` — narrative history, scoped to `game_id` (`id`, `game_id`, `sender`, `log_type`, `timestamp`, `active_swipe_index`, `is_deleted`)
  - `message_swipes` — per-message swipe versions (`id`, `message_id`, `swipe_index`, `text`, `snapshot_id`, `location_header`, `event_header`), cascades on message delete
  - `llm_messages` — LLM API call logging (not game-scoped)
- **`models`**: Database row structs (`DbGame`, `DbGameStateSnapshot`, `DbMessage`, `DbSwipe`, `DbLlmMessage`) — one per table, using raw SQLite types.
- **`mappers`**: Conversion logic between DB models and domain models (`TryFrom`/`From` impls and free functions for context-dependent mapping).
- **`backend`**: Unified `Storage` struct with `Backend` enum (`Sqlite`, `InMemory`, `Test`). All table-scoped methods live on `Storage` — no traits, no per-table repository structs. `Backend::Test` supports dynamic failure injection via `Operation` enum + `TestOverride`. Cross-table coordination (load full messages, update swipe text, save message + snapshot atomically) lives in `GameServiceContext` helpers.
- **`GameStateSnapshot`**: Serializable subset of `GameState` for persistence (messages excluded; hydrated separately). Lives in `crate::model::state_snapshot`.

### 8. The Bootstrap Tier (`crate::bootstrap`)
World loading, validation, and server initialization.
- **`load`**: World data loading from `data/worlds/`
- **`validate`**: World data validation (rooms, NPCs, triggers)
- **`scenario`**: Starting scenario selection
- **`logging`**: Structured logging setup
- **`run`**: Server initialization and startup
- **`state.rs`**: Fresh game state initialization (`build_fresh_initial_state`)

### 9. The CLI Tier (`crate::cli`)
Command-line argument parsing via `clap`.
- **`Cli`**: CLI args struct (`--world`, `--port`, etc.)

### 10. The Test Support Tier (`crate::test_support`)
Shared test fixtures and utilities.
- **`fixtures`**: `TestGameState`, `TestNpc`, `TestMap`, etc.
- **`context`**: Test context helpers
- **`test_app_builder`**: Fluent test app builder API
- **`server_helpers`**: `create_app_for_testing`, `create_app_for_testing_with_settings`
- **`test_app_builder`**: Fluent test app builder API

> **Note:** `assets/` contains static web assets (`index.html`) served by the server. It is not a Rust module tier.

## File Navigation

For file-to-module mapping, use `cargo doc` or navigate `src/` directly. Files follow the pattern `src/<tier>/<module>.rs` with sibling `*_tests.rs` test files.

## Sub-system References

The following concerns are documented in dedicated `docs/system/` files. Those are the authoritative source — this file covers module structure only.

| Topic | Document |
|-------|----------|
| Dashboard layout, tabs, polling, edit/retry UI | [`system/dashboard.md`](../system/dashboard.md) |
| UI design tokens, CSS components, animations | [`system/ui_design.md`](../system/ui_design.md) |
| Full game loop phases, retry flow, status phases | [`system/game_flow.md`](../system/game_flow.md) |
| History management, Message model | [`system/game_flow.md`](../system/game_flow.md) |
| Server endpoint reference | [`system/game_flow.md`](../system/game_flow.md) |
| Auto-trigger system and mutation order invariant | [`system/triggers.md`](../system/triggers.md) |
| Navigation and movement resolution | [`system/navigation.md`](../system/navigation.md) |
| Narration engine and Game Master logic | [`system/narration_engine.md`](../system/narration_engine.md) |
| LLM call logging and forensics | [`system/llm_processing.md`](../system/llm_processing.md) |

## Error Strategy

A unified error type (`crate::error::EngineError`) is shared across all tiers for consistent error propagation — from data loading through LLM failures to HTTP responses. See `src/error.rs` for the full variant list.
