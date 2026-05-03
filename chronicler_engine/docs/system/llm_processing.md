# Specification: LLM Processing & Integration

## Objective
The engine utilizes Large Language Models (LLMs) via the OpenRouter API or DeepSeek to handle Game Master narration and NPC dialogue.

## Technical Architecture

### 1. The Worker Thread Pattern
- **Threading**: The engine uses `std::thread::spawn` to run LLM requests on background threads. This prevents the TUI from freezing during network I/O.
- **Communication**: Results are streamed back to the main UI loop via `std::sync::mpsc` channels.

### 2. Model Configuration
The engine supports flexible model selection via connection profiles in `data/settings.json`.
- **Connections**: Named profiles combining `provider` + `model` + `api_key` + `base_url`
- **Narration Connection**: The connection used for Game Master narration and NPC dialogue
- **Quantifier Connection**: The connection used for scene quantification (can differ from narration)
- **Authentication**: Per-connection `api_key`, with provider-specific env var fallback (`OPENROUTER_API_KEY`)

### 3. Backend Selection
Backend is selected per-connection via `Connection.provider`:
- `openrouter` → Uses OpenRouter API with the connection's model and API key
- `deepseek` → Uses DeepSeek API with the connection's model and API key
- `ollama` → Uses local Ollama instance with the connection's base URL and model
- `mock` → Uses MockBackend for testing (no API key needed)

### 4. Single User Message Mode
Some models (particularly certain local/quantized models) ignore or poorly handle the `system` role. The `Connection` struct provides a `single_user_message` toggle for this case:
- **When `false` (default)**: System prompt is sent as the `system` message, user text as the `user` message (standard behavior)
- **When `true`**: System and user prompts are merged into a single `user` message with a `[SYSTEM]` prefix:
  ```
  [SYSTEM]
  <system prompt content>

  <user prompt content>
  ```
- The system message is omitted from the API payload when merging
- This is a per-connection setting, so different backends can use different strategies

### 5. Prompt Construction (SillyTavern-Style Layered Prompts)
The engine uses a layered prompt system inspired by SillyTavern's Prompt Manager. The prompt is built from 8 layers:

| Layer | Name | Content |
|-------|------|---------|
| 0 | System | Game rules, role instructions, narrative style |
| 1 | Game State | Current room, inventory, present NPCs |
| 2 | NPC Cards | `<KnownNpcs>` roster (all NPCs, condensed) + `<NpcsInRoom>` full cards (present NPCs only) |
| 3 | Player | Player persona and description |
| 4 | World Info | World lore triggered by keywords |
| 5 | History | Full narration_history (up to 1000 entries) |
| 6 | User Input | Current player message |
| 7 | PHI | Post-History Instructions (behavioral guidance) |

### 6. Token Budget Management
- **MAX_CONTEXT_TOKENS**: 32000
- **MAX_RESPONSE_TOKENS**: 1024
- **MAX_HISTORY_TOKENS**: 16000
- Hard truncation with `truncate_to_budget()` - no summarization

### 7. Prompt Injection Sanitization
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
