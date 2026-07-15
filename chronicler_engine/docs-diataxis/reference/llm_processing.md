---
diataxis: reference
title: LLM Processing & Backend Contract
---

> **Diátaxis mode:** Reference. This document describes the LLM transport and orchestration contract as it is: the blocking-task pattern that keeps sync game logic off the Tokio runtime, the per-connection model configuration, the four backend adapters, the prompt-side and response-side sanitization steps, the Gemma 4 reasoning-channel workaround, and the LLM call forensics pipeline. The problem it solves for the reader is *look-up*: when a generation is in flight, which layer does what. Layering detail lives in `./prompt_system.md` and `./system_prompt.md`; per-layer content is the preset source's responsibility (`data/prompt_presets/`).

## Overview

The engine makes outbound LLM calls through one of four backend adapters selected by a per-connection `provider` setting. All calls go through a single orchestration seam (`LlmCallRecorder`) that owns both forensics persistence and post-response sanitization. HTTP handlers offload the blocking work to the Tokio runtime's blocking pool via `spawn_blocking`, and the action pipeline checks a cancellation signal at internal stage boundaries to abort stale generations.

## Blocking-Task Pattern

`GameService` is fully synchronous. HTTP handlers in the action endpoints hand off to the async runtime via `tokio::task::spawn_blocking`. The blocking task runs the full pipeline synchronously inside one task.

Two guards keep in-flight work bounded:

- **Generation gate** (`Arc<AtomicBool>`) at the HTTP entry boundary. When the application is shutting down, the entrypoint returns early without spawning; the pipeline does not see this signal.
- **In-phase α-check.** When a generation starts, the pipeline captures the current game id. At three internal stage boundaries — after main narration, at the start of trigger continuation, after the trigger LLM call returns — it compares the captured id against the current active game id. A mismatch (caused by `switch_game` or `delete_game` during in-flight generation) raises `PhaseError::Cancelled`. The cancellation handler resets `GenerationStatus::Idle`, clears the phase, and persists the state.

`GenerationGuard::Drop` releases the per-game slot on panic. The slot carries both `game_id` and `generation_id`; on drop it verifies ownership before mutating the registry, so cleanup from a stale generation cannot clobber a newer one.

## Model Configuration

LLM connections are stored as a list on the `settings` singleton row. Each connection carries an identity (id + name, surfaced in the dashboard), a `LlmBackendType` discriminator (`openrouter` / `deepseek` / `ollama` / `mock`), a model identifier, an `api_key` with provider-specific env-var fallback (OpenRouter and DeepSeek fall back to `OPENROUTER_API_KEY` when no stored key is set), a `base_url` endpoint, a single-user-message toggle (see the next section), and per-connection token caps (`max_tokens` for the response, `max_context_tokens` for the context window). The full set of fields and their per-provider defaults lives in `src/domain/model/settings.rs`; this reference does not restate them.

Two named connection ids are read from settings: `narration_connection_id` (main narrative call) and `quantifier_connection_id` (post-narration scene analysis). They may resolve to the same connection record or to different ones; the wiring builds a dedicated `LlmCallRecorder` for each.

Settings are loaded once at startup and held as `Arc<RwLock<AppSettings>>`. No business logic reloads settings from disk.

## Backend Selection

Four adapters implement the `LlmProvider` port; the dispatcher selects by `provider`:

- **`openrouter`** — `OpenRouterBackend`. Calls the OpenRouter chat-completions endpoint with the connection's `api_key` (or the env-var fallback) and the connection's `model`.
- **`deepseek`** — `DeepSeekBackend`. Wired into the `LlmProvider` port and selected by `provider = "deepseek"`, but every call returns `EngineError::Config` because the transport layer is not implemented. The adapter exists so configuration can name DeepSeek without the engine crashing at load time.
- **`ollama`** — `OllamaBackend`. Calls a local Ollama instance with the connection's `base_url` and `model`. Ollama does not apply a native chat template, so the adapter holds the responsibility for any user-message preprocessing (see the Gemma 4 section below).
- **`mock`** — `MockBackend`. In-process deterministic backend used by tests; no API key or endpoint required.

## Single-User-Message Mode

Some local/quantized models ignore or poorly handle the `system` role. Each connection carries a `single_user_message` toggle:

- **`false` (default).** The system prompt is sent as the `system` message; the user text is sent as the `user` message.
- **`true`.** The system and user prompts are merged into a single `user` message with a `[SYSTEM]` prefix. The `system` field is omitted from the API payload.

The mode is per-connection, so different backends can use different strategies within the same `AppSettings`. The merge helper lives in `LlmProvider` (`merge_single_user_message`) and is invoked identically by the OpenRouter and Ollama adapters.

## Prompt Injection Sanitization

User input enters the engine as `<PlayerInput>` content. The assembler passes it through `sanitize_for_prompt`, which replaces any `{{variable}}` pattern (double curly braces enclosing an identifier) with `[FILTERED]`. Legitimate text passes through unchanged; single braces and empty/unclosed brace pairs are preserved.

Sanitization runs at render time only. Output-side handling lives in the next section.

## Response Sanitization & Gemma 4 Thinking-Channel Suffix

**Response sanitization.** `LlmCallRecorder::complete()` runs `sanitize_llm_output` on every response before saving forensics and returning the text to the pipeline. The sanitizer strips leaked reasoning artifacts that some chat-template-less models emit:

- `<channel|>`, `<thought>...</thought>`, `<|channel>thought ... <channel|>` blocks
- `<|turn>model`, `<turn|>`, `<|turn>` turn markers

The sanitizer runs regardless of model, so any leaked artifact is removed even from backends that did not pre-fill a reasoning block.

**Gemma 4 thinking-channel suffix.** Ollama uses a `{{ .Prompt }}` passthrough template rather than a native chat template, so the backend is responsible for turn structure. Gemma 4 models on Ollama — identified by name (`gemma-4` or `gemma4`, case-insensitive) — can enter an infinite reasoning loop in `<|channel>thought` and exhaust `max_tokens` returning empty content.

For affected Ollama backends, `OllamaBackend::preprocess_user_text()` appends the closure marker

```
<|turn>model
<|channel>thought
<channel|>
```

to the user message. OpenRouter backends are unaffected because the backend's native chat template handles turn structure. The response sanitizer above is the safety net for any artifact that still leaks.

## LLM Call Logging & Forensics

Every LLM call flows through `LlmCallRecorder::complete()`. The recorder:

1. Calls the backend's transport (pure API call).
2. Runs `sanitize_llm_output` on the response text.
3. Builds an `LlmMessage` record with the prompts, raw request JSON, raw response JSON, backend name, model name, agent name, sanitized parsed response, optional error message, and timestamp.
4. Saves the record through the `LlmMessageRepository` port.
5. Returns the sanitized result to the pipeline.

Storage holds the table `llm_messages` with a 50-row global cap: each insert prunes older rows so only the most recent 50 calls across all games are retained. The table has no `game_id` column, so the cap is global, not per-game.

The dashboard exposes the table via the LLM Messages tab at `/fragment/llm-messages`. The view shows the latest 50 calls with their prompts, raw request/response, and parsed response.

## Runtime Tracing

The engine uses the [`tracing`](https://tracing.rs) crate for structured runtime diagnostics. Spans and events fire automatically when `RUST_LOG` is set. Key markers: `spawn_blocking` lifecycle (`spawn_blocking: task started`, `spawn_blocking: shutting down before execute_action`, `spawn_blocking: execute_action completed`), per-LLM-call model + transport (`[LLM][req:N] Using model: ...`), and quantifier confidence (`[Quantifier] Detected NPCs: ... (confidence: ...)`).

## Document References

- [ADR-004: XML-Structured LLM Prompts](../../docs/adr/adr-004-xml-prompt-format.md) — XML-sectioned instructions + XML-wrapped data; sanitization rationale.
- [ADR-007: Settings System Architecture](../../docs/adr/adr-007-settings-system.md) — `AppSettings` + connection profiles + `OPENROUTER_API_KEY` env-var fallback.
- [ADR-010: Concurrency and Generation Gate Model](../../docs/adr/adr-010-concurrency-generation-gate.md) — `spawn_blocking` offload rationale + cooperative cancellation.
- [ADR-012: LLM Call Logging and Forensics](../../docs/adr/adr-012-llm-message-logging.md) — `llm_messages` table + retention + dashboard tab.
- [`./prompt_system.md`](./prompt_system.md) — layered prompt architecture + token budget constants + system/user separation.
- [`./system_prompt.md`](./system_prompt.md) — assembled system prompt structure + dynamic injection points.
- [`./action_pipeline.md`](./action_pipeline.md) — α-check stage boundaries + cancellation flow + `GenerationGuard::Drop`.