# Specification: Core Architecture (Modular)

## Objective
Establish a domain-driven modular architecture for the Chronicler Engine. This structure separates core data models from game mechanics, narrative processing, and user interface logic.

## Module Domains

### 1. The Model Tier (`crate::model::*`)
Contains pure data structures, serialization schemas, and the "Single Source of Truth" for game state. This tier has zero knowledge of the UI or LLM logic.
- **`world`**: Setting lore and global rules.
- **`map`**: Room/Region hierarchy and cardinal direction definitions.
- **`character`**: NPC attributes and Player inventory.
- **`state`**: The `GameState` aggregation and narration history logs.

### 2. The Engine Tier (`crate::engine::*`)
Contains the mechanics that drive the simulation. It translates user intent and state into outcomes.
- **`parser`**: Natural language command decomposition.
- **`action`**: The `Action` enum defining all supported system intents.
- **`logic`**: (Currently `engine.rs`) Rules for movement, fuzzy-matching, and room resolution.

### 3. The Narrative Tier (`crate::narrative::*`)
The interface between the synchronous engine and stochastic LLM generation.
- **`llm`**: Traits (`LlmBackend`) and implementations (OpenRouter) for Game Master narration.

### 4. The Presentation Tier (`crate::ui::*`)
The rendering logic for the TUI.
- **`dashboard`**: Handles layout chunks and component-based widget drawing.
- **`visuals`**: Handles image loading, caching, and smart-pixel cropping protocols.

## File Mapping (Migration Blueprint)

| Current File | New Domain | Note |
| :--- | :--- | :--- |
| `src/world.rs` | `src/model/world.rs` | |
| `src/map.rs` | `src/model/map.rs` | |
| `src/character.rs` | `src/model/character.rs` | |
| `src/state.rs` | `src/model/state.rs` | |
| `src/parser.rs` | `src/engine/parser.rs` | |
| `src/action.rs` | `src/engine/action.rs` | |
| `src/engine.rs` | `src/engine/logic.rs` | Renamed for clarity. |
| `src/llm.rs` | `src/narrative/llm.rs` | |
| `src/ui.rs` | `src/ui/dashboard.rs` | Renamed for modularity. |

## Error Strategy
A unified error type (`crate::error::EngineError`) is shared across all tiers to provide consistent error propagation from data loading through LLM failures to the final UI report.
