# Specification: LLM Processing & Integration

## Objective
The engine utilizes Large Language Models (LLMs) via the OpenRouter API or DeepSeek to handle Game Master narration and NPC dialogue.

## Technical Architecture

### 1. The Worker Thread Pattern
- **Threading**: The engine uses `std::thread::spawn` to run LLM requests on background threads. This prevents the TUI from freezing during network I/O.
- **Communication**: Results are streamed back to the main UI loop via `std::sync::mpsc` channels.

### 2. Model Configuration
The engine supports flexible model selection via environment variables.
- **Variable**: `LLM_MODEL` (OpenRouter) or `DEEPSEEK_MODEL` (DeepSeek)
- **Backend Selection**: `LLM_BACKEND` env var: `openrouter` (default), `deepseek`, or `mock`
- **Authentication**: Requires API key in `.env` (`OPENROUTER_API_KEY` or `DEEPSEEK_API_KEY`)

### 3. Prompt Construction (SillyTavern-Style Layered Prompts)
The engine uses a layered prompt system inspired by SillyTavern's Prompt Manager. The prompt is built from 8 layers:

| Layer | Name | Content |
|-------|------|---------|
| 0 | System | Game rules, role instructions, narrative style |
| 1 | Game State | Current room, inventory, present NPCs |
| 2 | NPC Cards | In-room NPC character sheets (name, description, personality, scenario) |
| 3 | Player | Player persona and description |
| 4 | World Info | World lore triggered by keywords |
| 5 | History | Full narration_history (up to 1000 entries) |
| 6 | User Input | Current player message |
| 7 | PHI | Post-History Instructions (behavioral guidance) |

### 4. Token Budget Management
- **MAX_CONTEXT_TOKENS**: 8192 (configurable)
- **MAX_RESPONSE_TOKENS**: 1024
- **MAX_HISTORY_TOKENS**: Available after other layers
- Hard truncation with `truncate_to_budget()` - no summarization

### 5. Prompt Injection Sanitization
User input is sanitized to prevent prompt injection:
- `{{variable}}` patterns are escaped
- Known injection patterns are stripped
- Legitimate text passes through unchanged

### Module Location
- **Crate path**: `crate::narrative::llm` (LLM backends)
- **Crate path**: `crate::narrative::prompt` (PromptBuilder)

## Implementation Standards
- Use the `LlmBackend` trait for all implementations
- Maintain a `MockBackend` for test environments
- Use `PromptBuilder` for all LLM calls
