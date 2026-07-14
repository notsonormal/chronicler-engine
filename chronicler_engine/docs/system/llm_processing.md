# Specification: LLM Processing & Integration

## Objective

The engine utilizes Large Language Models (LLMs) via the OpenRouter API, DeepSeek, or local Ollama to handle Game Master narration and NPC dialogue.

## Technical Architecture

### 1. The Blocking Task Pattern

- **Concurrency**: `GameService` is fully synchronous; HTTP handlers in `src/adapters/driving/http/fragments/actions.rs` offload LLM work to the async runtime via `tokio::task::spawn_blocking`.
- **Cancellation**: Each spawned task checks a `CancellationToken` before and after execution to handle graceful shutdown. Long-running pipelines (`ActionPipeline::run_from_input`) also perform an in-phase α-check (`app.current_game_id()` against the started id) at internal stage boundaries (after main narration, before trigger continuation, after trigger continuation) to abort stale generations whose `current_game_id()` changed mid-pipeline (e.g. via `switch_game` or `delete_game`). When cancelled mid-pipeline, `ActionPipeline::handle_cancellation()` resets `GenerationStatus::Idle`, clears the phase, and persists the state.

### 2. Model Configuration

The engine supports flexible model selection via connection profiles stored in the SQLite `settings` table (seeded from `data/settings.json` at startup).

- **Connections**: Named profiles combining `provider` + `model` + `api_key` + `base_url`
- **Narration Connection**: The connection used for Game Master narration and NPC dialogue
- **Quantifier Connection**: The connection used for scene quantification (can differ from narration)
- **Authentication**: Per-connection `api_key`, with provider-specific env var fallback (`OPENROUTER_API_KEY`)
- **Settings Lifecycle**: Settings are loaded once at startup (`bootstrap/run.rs`) and passed down as `Arc<RwLock<AppSettings>>`. No business logic reloads settings from disk.

### 3. Backend Selection

Backend is selected per-connection via `LlmProviderConfig.provider`:

- `openrouter` → Uses OpenRouter API with the connection's model and API key
- `deepseek` → Stub. `DeepSeekBackend` is wired into the `LlmProvider` port and selected by `provider = "deepseek"`; every call returns `EngineError::Config` because the transport layer is not implemented.
- `ollama` → Uses local Ollama instance with the connection's base URL and model
- `mock` → Uses MockBackend for testing (no API key needed)

### 4. Single User Message Mode

Some models (particularly certain local/quantized models) ignore or poorly handle the `system` role. The `LlmProviderConfig` struct provides a `single_user_message` toggle for this case:

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

The engine uses a layered prompt system (with post-history splice) inspired by SillyTavern's Prompt Manager, with XML-sectioned instructions + XML-wrapped data for reasoning-model compatibility. The prompt is split into a system half (Layer 0) and a user half (Layers 1–6, with post-history splice between Layer 5 and Layer 6).

### 6. Token Budget Management

Connectors override `max_context_tokens` per connection via the settings system (see Section 2 above).

### 7. Prompt Injection Sanitization

User input is sanitized to prevent prompt injection:

- `{{variable}}` patterns are replaced with `[FILTERED]`
- Known injection patterns are stripped
- Legitimate text passes through unchanged

### 8. Gemma 4 Thinking-Channel Suffix

Gemma 4 models on Ollama use a `{{ .Prompt }}` passthrough template — Ollama does not apply a native chat template. The 26B variant (especially abliterated quants) can enter an infinite reasoning loop, burning all `max_tokens` in `<|channel>thought` and returning empty `content`.

For Ollama backends whose model name contains `gemma-4` or `gemma4`, `OllamaBackend::preprocess_user_text()` appends the following closure marker to the user message:

```
<|turn>model
<|channel>thought
<channel|>
```

This matches SillyTavern's `last_output_sequence` preset for Gemma 4. It pre-fills an empty thought block so the model skips reasoning and generates narrative content immediately.

- **Scope**: Ollama backends only; Gemma 4 models detected by name.
- **OpenRouter / chat-template backends**: Suffix is not applied — the backend's native chat template handles turn structure.
- **Non-Gemma models**: Unaffected.
- **Safety net**: `application::llm_sanitizer::sanitize_llm_output()` (at `src/application/llm_sanitizer.rs`) strips any leaked `<channel|>`, `<thought>`, or `<|channel>thought` artifacts from all responses regardless of model. Sanitization runs inside `LlmCallRecorder::complete()` as part of postprocessing.

### 9. LLM Call Logging & Forensics

- **WHAT**: Every LLM call logs prompt + response pairs, metadata, and timestamps to a SQLite `llm_messages` table.
- **RETENTION**: 50 rows globally — oldest rows auto-pruned, most recent kept. (The `llm_messages` table has no `game_id` column; the cap is global, not per-game.)
- **VIEW**: **LLM Messages** tab in the dashboard (`/fragment/llm-messages`).

### 10. Runtime Tracing

The engine uses [`tracing`](https://tracing.rs) for structured runtime diagnostics. Critical execution paths emit spans and events automatically when `RUST_LOG` is set. The instrumented function list and tracing commands are documented in the diagnostics reference.

## Conventions

- Use the `LlmProvider` trait for transport implementations (OpenRouter, DeepSeek, Ollama, Mock)
- Maintain a `MockBackend` for test environments
- Use `LlmCallRecorder` for orchestration (forensics + postprocessing)
- Use `PromptAssembler` for all prompt construction; `LlmProvider::complete()` for transport; `LlmCallRecorder::complete()` for orchestration
- Configure `max_context_tokens` per connection to match the model's actual context window

## Document References

- [ADR-004: XML-Structured LLM Prompts](../adr/adr-004-xml-prompt-format.md) — XML-sectioned instructions + XML-wrapped data
- [ADR-007: Settings System Architecture](../adr/adr-007-settings-system.md) — `AppSettings` + connection profiles + per-connection overrides
- [ADR-010: Concurrency and Generation Gate Model](../adr/adr-010-concurrency-generation-gate.md) — `spawn_blocking` + `CancellationToken` + `is_generating` invariant
- [ADR-012: LLM Call Logging and Forensics](../adr/adr-012-llm-message-logging.md) — `llm_messages` table + retention + dashboard tab
- [system/prompt_system.md](./prompt_system.md) — layered prompt composition + token budget constants + trimming strategy
- [system/action_pipeline.md](./action_pipeline.md) — α-check stage boundaries and cancellation flow
- [diagnostics/DEBUGGING.md](../diagnostics/DEBUGGING.md) — instrumented function list + tracing commands
- [architecture/rust_technical.md](../architecture/rust_technical.md) — `spawn_blocking` offload rationale (sync services, no async traits)
