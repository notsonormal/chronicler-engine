# Specification: Core Architecture (Modular)

## Objective
Establish a domain-driven modular architecture for the Chronicler Engine. This structure separates core data models from game mechanics, narrative processing, and user interface logic.

## Module Domains

### 1. The Model Tier (`crate::model::*`)
Contains pure data structures, serialization schemas, and the "Single Source of Truth" for game state. This tier has zero knowledge of the UI or LLM logic.
- **`world`**: Setting lore, global rules, and starting scenarios.
- **`map`**: Room/Region hierarchy and cardinal direction definitions.
- **`character`**: NPC attributes (name, description, personality, scenario, image_path, **profile_image**, **headshot_image**) and Player inventory.
- **`state`**: The `GameState` aggregation, narration history logs, and TUI state.
- **`scenario`**: Starting scenario definitions for narrative introductions.

### 2. The Engine Tier (`crate::engine::*`)
Contains the mechanics that drive the simulation. It translates user intent and state into outcomes.
- **`parser`**: Natural language command decomposition.
- **`action`**: The `Action` enum defining all supported system intents.
- **`logic`**: Rules for movement, fuzzy-matching, and room resolution.

### 3. The Narrative Tier (`crate::narrative::*`)
The interface between the synchronous engine and stochastic LLM generation.
- **`llm`**: Traits (`LlmBackend`) and implementations (OpenRouter, DeepSeek) for Game Master narration.
- **`prompt`**: PromptBuilder module for SillyTavern-style layered prompt construction with token budget management.
- **`quantifier`**: Scene quantification module for dynamic room presence detection via secondary LLM (NEW).

### 4. The Server Tier (`crate::server::*`)
The HTTP layer for the HTMX web dashboard with polling-based real-time updates.
- **`mod`**: Axum router, request handlers.
- **`fragments`**: HTML fragment generators for HTMX partial updates.
  - Uses `pulldown-cmark` for markdown→HTML conversion of LLM narrative text.
  - Uses `askama` for all 4 templates (header, story_log, visual_sidebar, action_area).
- **`templates`**: Askama template definitions with type-safe rendering.
  - Templates declare required data shapes at compile time.
  - Missing fields = compiler error (not runtime failure).

### 5. The Presentation Tier (`assets/`)
Static web assets served by the server.
- **`index.html`**: HTMX frontend with CSS styling matching terminal aesthetic.

## File Mapping

| File | Domain | Note |
| :--- | :--- | :--- |
| `src/model/world.rs` | `crate::model::world` | |
| `src/model/map.rs` | `crate::model::map` | |
| `src/model/character.rs` | `crate::model::character` | |
| `src/model/state.rs` | `crate::model::state` | |
| `src/model/scenario.rs` | `crate::model::scenario` | Starting scenarios (NEW) |
| `src/engine/parser.rs` | `crate::engine::parser` | |
| `src/engine/action.rs` | `crate::engine::action` | |
| `src/engine/logic.rs` | `crate::engine::logic` | |
| `src/narrative/llm.rs` | `crate::narrative::llm` | LLM backend implementations |
| `src/narrative/prompt.rs` | `crate::narrative::prompt` | PromptBuilder with layered prompts |
| `src/narrative/quantifier.rs` | `crate::narrative::quantifier` | Scene quantification for dynamic NPC presence (NEW) |
| `src/narrative/openrouter_client.rs` | `crate::narrative::openrouter_client` | OpenRouter HTTP client with dual-model support (NEW) |
| `src/server/mod.rs` | `crate::server` | HTTP server + HTMX endpoints |
| `src/server/fragments.rs` | `crate::server` | HTML fragments |
| `src/server/templates.rs` | `crate::server` | Askama templates (NEW) |
| `assets/index.html` | Presentation | HTMX frontend |

## UI Specification

The engine presents a web-based HTMX dashboard:

- **Header**: Game title only
- **Main Body**: Story log (80%) + visual sidebar (20%) (see `docs/system/dashboard.md`)
  - Story log shows location entries, narration, dialogue, system messages, and input
  - Location entries appear inline in story log with room name and timestamp
  - Visual sidebar displays:
    - Room location image (from `Room.image_path`)
    - NPC portraits in 2-column grid (from `CharacterSheet.headshot_image` with fallback to `image_path`)
- **Action Area**: Command input + status indicator (Ready/Thinking)

Real-time updates via HTMX polling (5s interval for story-log, 5s for status-display).

### Image Handling

Character images have two supported fields:
- **`image_path`**: Legacy field, full body image
- **`headshot_image`**: Preferred field for portraits (fallback to image_path)
- **`profile_image`**: For character profile display

Room images use `image_path` field in Room struct.

Click handlers on images trigger visual sidebar toggle.

## Error Strategy
A unified error type (`crate::error::EngineError`) is shared across all tiers to provide consistent error propagation from data loading through LLM failures to the final UI report.