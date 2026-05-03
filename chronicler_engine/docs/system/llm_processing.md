# Specification: LLM Processing & Integration

## Objective
The engine utilizes Large Language Models (LLMs) via the OpenRouter API, DeepSeek, or local Ollama to handle Game Master narration and NPC dialogue.

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

### 5. Prompt Construction (Layered Prompts)
The engine uses a layered prompt system inspired by SillyTavern's Prompt Manager, refined with a **plain-text instructions + XML-wrapped data** pattern for reasoning-model compatibility:

| Layer | Name | Content | Role |
|-------|------|---------|------|
| 0 | System | Plain-text game rules, role instructions, narrative style | System |
| 1 | Game State | `<GameState>` — Current room, inventory, present NPCs | User (data) |
| 2 | NPC Cards | `<KnownNpcs>` roster (all NPCs, condensed) + `<NpcsInRoom>` full cards (present NPCs only) | User (data) |
| 3 | Player | `<PlayerCharacter>` — Player persona and description | User (data) |
| 4 | World Info | `<WorldLore>` — World lore triggered by keywords | User (data) |
| 5 | History | `<ConversationHistory>` — Full narration_history | User (data) |
| 6 | User Input | `<PlayerInput>` — Current player message | User (data) |
| 7 | PHI | Plain-text post-history behavioral guidance | User (instruction) |

**`build_split()` separation**:
- **System half**: Plain-text instructions only (Layer 0)
- **User half**: XML-wrapped data (Layers 1–6) + plain-text PHI (Layer 7)

This separation prevents reasoning models (e.g., Gemma 4) from entering meta-analysis mode.

### 6. Token Budget Management
- **MAX_CONTEXT_TOKENS**: 32768 (fallback default; configurable per connection via `max_context_tokens`)
- **MAX_RESPONSE_TOKENS**: 2048 (fallback default)
- **MAX_HISTORY_TOKENS**: 16000
- **SAFETY_MARGIN_TOKENS**: 256 (reserved for token estimation error)
- **MIN_INPUT_BUDGET_TOKENS**: 512 (minimum space reserved for input)
- **Strategy**: Context-aware fitting via `fit_messages_to_context()` — dynamically caps `max_tokens`, trims oldest history entries first to fit within the connection's configured context window
- **No summarization** — maintains accuracy over compression
- **Estimation**: Character-based token estimation (simple and fast)

### 7. Prompt Injection Sanitization
User input is sanitized to prevent prompt injection:
- `{{variable}}` patterns are escaped
- Known injection patterns are stripped
- Legitimate text passes through unchanged

### Module Location
- **Crate path**: `crate::narrative::llm` (LLM backends)
- **Crate path**: `crate::narrative::prompt` (PromptBuilder, context fitting)
- **Crate path**: `crate::narrative::llm_client` (HTTP client helpers)

## Implementation Standards
- Use the `LlmBackend` trait for all implementations
- Maintain a `MockBackend` for test environments
- Use `PromptBuilder` for all LLM calls
- Configure `max_context_tokens` per connection to match the model's actual context window
