# Specification: LLM Processing & Integration

## Objective
The engine utilizes Large Language Models (LLMs) via the OpenRouter API, DeepSeek, or local Ollama to handle Game Master narration and NPC dialogue.

## Technical Architecture

### 1. The Blocking Task Pattern
- **Concurrency**: The engine keeps the `GameService` trait fully synchronous. HTTP handlers in `src/server/fragments.rs` offload LLM work to the async runtime via `tokio::task::spawn_blocking`. This prevents the Axum event loop from stalling during network I/O while avoiding the `#[async_trait]` + `dyn Trait` incompatibility in Rust 2024 edition.
- **Cancellation**: Each spawned task checks a `CancellationToken` before and after execution to handle graceful shutdown.

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

This separation reduces the chance of reasoning models (e.g., Gemma 4) entering meta-analysis mode. However, the Gemma 4 26B model (particularly abliterated quants) can still get stuck in an infinite `<|channel>thought` loop even with plain-text instructions. An additional prompt-level fix is applied for Gemma 4 models (see §8).

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

### 8. Gemma 4 Thinking-Channel Suffix

Gemma 4 models on Ollama use a `{{ .Prompt }}` passthrough template — Ollama does not apply a native chat template. The 26B variant (especially abliterated quants) can enter an infinite reasoning loop, burning all `max_tokens` in `<|channel>thought` and returning empty `content`.

**Fix**: `llm_client.rs::apply_gemma4_thinking_suffix()` detects models with `"gemma-4"` or `"gemma4"` in their name and appends the closure marker to the user message **only for Ollama backends**:

```
<|turn>model
<|channel>thought
<channel|>
```

This matches SillyTavern's `last_output_sequence` preset for Gemma 4. It pre-fills an empty thought block so the model skips reasoning and generates narrative content immediately.

- **Scope**: Ollama backends only; Gemma 4 models detected by name
- **OpenRouter / chat-template backends**: Suffix is NOT applied — the backend's native chat template handles turn structure
- **Non-Gemma models**: Completely unaffected
- **Validation**: Reduced completion tokens from 2048 (all reasoning) to ~211 (actual content) on `mradermacher/gemma-4-26b-a4b-it-abliterated:iq2xs`
- **Safety net**: `sanitize_llm_output()` strips any leaked `<channel|>`, `<thought>`, or `<|channel>thought` artifacts from all responses regardless of model
- **Ref**: [SillyTavern Reddit discussion](https://old.reddit.com/r/SillyTavernAI/comments/1sbjwke/)

### Module Location
- **Crate path**: `crate::narrative::llm` — directory module (`mod.rs`, `backend.rs`, `openrouter.rs`, `deepseek.rs`, `ollama.rs`, `mock.rs`)
- **Crate path**: `crate::narrative::prompt` — directory module (`mod.rs`, `builder.rs`, `budget.rs`, `context.rs`, `sanitize.rs`, `templates.rs`, `types.rs`, `tests.rs`)
- **Crate path**: `crate::narrative::llm_client` — HTTP client helpers (`src/narrative/llm_client.rs`)

## Implementation Standards
- Use the `LlmBackend` trait for all implementations
- Maintain a `MockBackend` for test environments
- Use `PromptBuilder` for all LLM calls
- Configure `max_context_tokens` per connection to match the model's actual context window
