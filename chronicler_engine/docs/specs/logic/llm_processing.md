# Specification: LLM Processing & Integration

## Objective
The engine utilizes Large Language Models (LLMs) via the OpenRouter API to handle Game Master narration and NPC dialogue. 

## Technical Architecture

### 1. The Worker Thread Pattern
- **Threading**: The engine uses `std::thread::spawn` to run LLM requests on background threads. This prevents the TUI from freezing during network I/O.
- **Communication**: Results are streamed back to the main UI loop via `std::sync::mpsc` channels.

### 2. Model Configuration
The engine supports flexible model selection via the `LLM_MODEL` environment variable.
- **Variable**: `LLM_MODEL`
- **Fallback**: Defaults to `z-ai/glm-4.5-air:free` if not specified.
- **Authentication**: Requires `OPENROUTER_API_KEY` stored in `.env`.

### 3. Prompt Construction
The engine anchors roleplay output by constructing a context window comprising:
- **World Context**: Global rules and setting lore.
- **Local Context**: Current room name, description, and list of present NPCs.
- **Participant Context**: Player and target NPC personalities/scenarios.

## Implementation Standards
- Use the `LlmBackend` trait for all implementation.
- Maintain a `MockBackend` for test environments to ensure zero-network unit testing.
