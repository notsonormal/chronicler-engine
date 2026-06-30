# Specification: LLM Processing & Integration

> **Related Decisions**: [ADR-004](../adr/adr-004-xml-prompt-format.md), [ADR-007](../adr/adr-007-settings-system.md), [ADR-010](../adr/adr-010-concurrency-generation-gate.md)

## Objective

The engine utilizes Large Language Models (LLMs) via the OpenRouter API, DeepSeek, or local Ollama to handle Game Master narration and NPC dialogue.

## Technical Architecture

### 1. The Blocking Task Pattern

- **Concurrency**: The engine keeps the `GameService` trait fully synchronous. HTTP handlers in `src/adapters/driving/http/fragments/actions.rs` offload LLM work to the async runtime via `tokio::task::spawn_blocking`. This prevents the Axum event loop from stalling during network I/O while avoiding the `#[async_trait]` + `dyn Trait` incompatibility in Rust 2024 edition.
- **Cancellation**: Each spawned task checks a `CancellationToken` before and after execution to handle graceful shutdown. Long-running pipelines (`ActionPipeline::run_from_input`) also check the token at internal stage boundaries (after main narration, before trigger continuation, after trigger continuation) to abort early and avoid wasting LLM calls on stale requests. When cancelled mid-pipeline, `ActionPipeline::handle_cancellation()` resets `GenerationStatus::Idle`, clears the phase, and persists the state.

### 2. Model Configuration

The engine supports flexible model selection via connection profiles stored in the SQLite `settings` table (seeded from `data/settings.json` at startup; see ADR-024).

- **Connections**: Named profiles combining `provider` + `model` + `api_key` + `base_url`
- **Narration Connection**: The connection used for Game Master narration and NPC dialogue
- **Quantifier Connection**: The connection used for scene quantification (can differ from narration)
- **Authentication**: Per-connection `api_key`, with provider-specific env var fallback (`OPENROUTER_API_KEY`)
- **Settings Lifecycle**: Settings are loaded once at startup (`bootstrap/run.rs`) and passed down as `Arc<RwLock<AppSettings>>`. No business logic reloads settings from disk.

### 3. Backend Selection

Backend is selected per-connection via `Connection.provider`:

- `openrouter` → Uses OpenRouter API with the connection's model and API key
- `deepseek` → **Stub — not yet implemented.** Returns error on all calls.
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

The engine uses an 8-layer prompt system inspired by SillyTavern's Prompt Manager, with XML-sectioned instructions + XML-wrapped data for reasoning-model compatibility. The prompt is split into a system half (Layer 0) and a user half (Layers 1–7). For the complete layer table, per-layer examples, system/user separation rationale, and token budget constants, see [`prompt_system.md`](prompt_system.md) — that document is the authoritative source for prompt composition.

### 6. Token Budget Management

Token budget constants (`MAX_CONTEXT_TOKENS`, `MAX_RESPONSE_TOKENS`, `MAX_HISTORY_TOKENS`, `SAFETY_MARGIN_TOKENS`, `MIN_INPUT_BUDGET_TOKENS`) and the `fit_messages_to_context()` trimming strategy are defined in [`prompt_system.md`](prompt_system.md). Connectors override `max_context_tokens` per connection via the settings system (see Section 2 above).

### 7. Prompt Injection Sanitization

User input is sanitized to prevent prompt injection:

- `{{variable}}` patterns are escaped
- Known injection patterns are stripped
- Legitimate text passes through unchanged

### 8. Gemma 4 Thinking-Channel Suffix

Gemma 4 models on Ollama use a `{{ .Prompt }}` passthrough template — Ollama does not apply a native chat template. The 26B variant (especially abliterated quants) can enter an infinite reasoning loop, burning all `max_tokens` in `<|channel>thought` and returning empty `content`.

**Fix**: `OllamaBackend::preprocess_user_text()` detects models with `"gemma-4"` or `"gemma4"` in their name and appends the closure marker to the user message **only for Ollama backends**:

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
- **Safety net**: `narrative::llm::sanitize::sanitize_llm_output()` (at `src/adapters/driven/llm/sanitize.rs`) strips any leaked `<channel|>`, `<thought>`, or `<|channel>thought` artifacts from all responses regardless of model. Sanitization runs inside `LlmCallRecorder::complete()` as part of postprocessing.
- **Ref**: [SillyTavern Reddit discussion](https://old.reddit.com/r/SillyTavernAI/comments/1sbjwke/)

### 9. LLM Call Logging & Forensics

Every LLM call is logged to a SQLite `llm_messages` table with a strict 50-row global cap. This enables rapid diagnosis when the engine misbehaves — the full request/response JSON is preserved alongside metadata.

#### Architecture

- **`call_chat_completions()`** in `llm_client.rs` is the single chokepoint. It returns `ChatCompletionResult { text, system_prompt, user_prompt, raw_request_json, raw_response_json }`.
- **`LlmProvider` trait** methods take `agent_name: &str` and return `LlmCallResult`, which wraps the `ChatCompletionResult` with `backend_name` and `model_name`. Trait is transport-only.
- **Quantifier path** logs via `LlmCallRecorder::complete()` which delegates to `LlmProvider::complete()` (transport) then `LlmMessageRepository::save_llm_message()` (forensics). No `wrap_and_save` default impl — deleted in Phase 2.1.
- **LLM logging implementation**: `LlmMessageRepository` port at `src/application/ports/llm_message_repository.rs`, implemented by `Storage` struct.
  - SQLite backend: INSERT + auto-prune to keep last 50 rows
  - InMemory backend: Ring buffer for tests
- **Storage is optional**: Backends accept `Option<Arc<Storage>>`. When `None`, logging is silently skipped (useful for tests that don't care about forensics).

#### Agent Names

Four agent names are used consistently across the codebase:

| Agent | Role |
|-------|------|
| `narrator` | Game Master narration |
| `quantifier` | Scene quantification |
| `trigger` | Trigger event continuation |
| `dialogue` | NPC dialogue generation |

#### Dashboard Integration

The LLM Messages tab (`/fragment/llm-messages`) renders the last 50 calls as an expandable list, polled every 4 seconds via HTMX.

### 10. Runtime Tracing & Test Forensics

The engine uses [`tracing`](https://tracing.rs) for structured runtime diagnostics. Critical execution paths emit spans and events automatically when `RUST_LOG` is set.

#### Instrumented Functions

**Action Processing** (`src/domain/engine/action_processing.rs`):

- `handle_movement` — tracks room transitions
- `apply_npc_events` — tracks NPC enter/leave events
- `execute_freeaction_impl` — tracks complete action lifecycle

**Trigger Evaluation** (`src/domain/engine/trigger_eval.rs`):

- `evaluate_triggers` — tracks trigger firing decisions
- `check_condition` — tracks condition evaluation

**Quantifier** (`src/application/agents/quantifier/orchestration.rs`):

- `determine_npcs_in_room` — tracks NPC detection confidence

**LLM Client** (`src/adapters/driven/llm/transport/mod.rs`):

- `call_chat_completions` — tracks HTTP request/response lifecycle
- `handle_response` — tracks response parsing and error handling
- `parse_chat_response` — tracks JSON parsing and content extraction
**Game Service** (`src/application/action_pipeline/actions.rs`, `retry.rs`, `pipeline.rs`, `phases.rs`):
- `ActionPipeline::run_from_input()` — main pipeline entry from player input, delegates to `self.phase_*()` methods defined in split `impl` block across `pipeline.rs` and `phases.rs`
- `ActionPipeline::phase_trigger_continuation()` — wrapper around `phase_trigger_continuation_raw()` for retry/retrigger
- `ActionPipeline::phase_narrate()`, `ActionPipeline::phase_post_generation()`, `ActionPipeline::phase_engine_commit()`, `ActionPipeline::phase_trigger_continuation_raw()`, `ActionPipeline::reconcile_post_trigger_npcs()`, `ActionPipeline::build_trigger_request()` — granular phase implementations (split `impl` block in `phases.rs`)
- Pipeline checks `CancellationToken::is_cancelled()` at stage boundaries and emits `tracing::debug!` events on phase transitions (not info-level, to reduce noise)

#### Forensics Collector

The `ForensicsCollector` (`src/test_support/forensics.rs`) is a tracing subscriber that:

- Buffers spans and events during test execution
- Automatically writes JSON on test failure to `tmp/diagnostics/`
- Redacts sensitive fields (`api_key`, `prompt`, `raw_response`, `authorization`)
- Truncates long strings (>10KB) to prevent oversized files

#### Usage

```bash
# See info-level traces
RUST_LOG=info cargo test

# Debug specific module
RUST_LOG=chronicler_engine::engine=debug cargo test

# Full trace output
RUST_LOG=trace cargo test
```

#### Module Location

- **Crate path**: `crate::test_support::forensics` — `ForensicsCollector`, `ForensicsLayer`
- **Bootstrap**: `bootstrap/run.rs` initializes `tracing_subscriber::fmt()` with `EnvFilter`

### Module Location

- **Crate path**: `crate::adapters::driven::llm::providers` — directory module (`mod.rs`, `backend.rs`, `openrouter.rs`, `deepseek.rs`, `ollama.rs`, `mock.rs`, `sanitize.rs`)
- **Crate path**: `crate::application::narrative_prompt` — directory module (`mod.rs`, `assembler.rs`, `budget.rs`, `context.rs`, `sanitize.rs`, `types.rs`, plus sibling `*_tests.rs` files)
- **Crate path**: `crate::adapters::driven::llm::transport` — directory module (`mod.rs`, `request.rs`, `response.rs`, `client.rs`, plus sibling `request_tests.rs`, `response_tests.rs`)
- **Crate path**: `crate::adapters::driven::storage::backend::llm_messages` — LLM logging implementation (`Storage::save_llm_message()`, `Storage::list_latest_llm_messages()`)
- **Crate path**: `crate::adapters::driven::storage::db` — SQLite schema (`llm_messages` table)
- **Crate path**: `crate::adapters::driven::llm::forensics::message` — `LlmMessage` data model

## Implementation Standards

- Use the `LlmProvider` trait for transport implementations (OpenRouter, DeepSeek, Ollama, Mock)
- Maintain a `MockBackend` for test environments
- Use `LlmCallRecorder` for orchestration (forensics + postprocessing)
- Use `LayeredPromptAssembler` for all prompt construction; `LlmProvider::complete()` for transport; `LlmCallRecorder::complete()` for orchestration
- Configure `max_context_tokens` per connection to match the model's actual context window
