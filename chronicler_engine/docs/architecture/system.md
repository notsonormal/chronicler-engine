# Specification: Core Architecture (Modular)

## Objective
Establish a domain-driven modular architecture for the Chronicler Engine. This structure separates core data models from game mechanics, narrative processing, and user interface logic.

## Module Domains

### 1. The Model Tier (`crate::model::*`)
Contains pure data structures, serialization schemas, and the "Single Source of Truth" for game state. This tier has zero knowledge of the UI or LLM logic.
- **`world`**: Setting lore and global rules.
- **`map`**: Room/Region hierarchy and cardinal direction definitions.
- **`character`**: NPC attributes and Player inventory.
- **`state`**: The `GameState` aggregation, narration history logs, and TUI state.

### 2. The Engine Tier (`crate::engine::*`)
Contains the mechanics that drive the simulation. It translates user intent and state into outcomes.
- **`parser`**: Natural language command decomposition.
- **`action`**: The `Action` enum defining all supported system intents.
- **`logic`**: Rules for movement, fuzzy-matching, and room resolution.

### 3. The Narrative Tier (`crate::narrative::*`)
The interface between the synchronous engine and stochastic LLM generation.
- **`llm`**: Traits (`LlmBackend`) and implementations (OpenRouter) for Game Master narration.

### 4. The Server Tier (`crate::server::*`)
The HTTP and WebSocket layer for the HTMX web dashboard.
- **`mod`**: Axum router, WebSocket handler, state management.
- **`hub`**: Broadcast channel for real-time client updates.
- **`fragments`**: HTML fragment generators for HTMX partial updates.

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
| `src/engine/parser.rs` | `crate::engine::parser` | |
| `src/engine/action.rs` | `crate::engine::action` | |
| `src/engine/logic.rs` | `crate::engine::logic` | |
| `src/narrative/llm.rs` | `crate::narrative::llm` | |
| `src/server/mod.rs` | `crate::server` | HTTP + WebSocket |
| `src/server/hub.rs` | `crate::server` | Broadcast channel |
| `src/server/fragments.rs` | `crate::server` | HTML fragments |
| `assets/index.html` | Presentation | HTMX frontend |

## UI Specification

The engine presents a web-based HTMX dashboard:

- **Header**: Game title + current location (green bold)
- **Main Body**: 70% story log / 30% visual sidebar
  - Story log shows narration (cyan), dialogue (white), system (yellow), input (gray)
  - Visual sidebar shows location image + NPC portraits
- **Action Area**: Command input + status indicator (Ready/Thinking)

Real-time updates via Server-Sent Events (SSE).

## Error Strategy
A unified error type (`crate::error::EngineError`) is shared across all tiers to provide consistent error propagation from data loading through LLM failures to the final UI report.