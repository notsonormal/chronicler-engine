---
diataxis: reference
title: Narration System
---

> **Diátaxis mode:** Reference. This document describes the narrator as it is: per-connection model configuration, the four backend adapters, the Game Master's role and narrative modes, response sanitization and the Gemma 4 thinking-channel workaround, the LLM call forensics pipeline, and runtime diagnostics. The problem it solves for the reader is *look-up*: which layer owns model selection, prompt invocation, response handling, and forensics; where the narrative result returns to the Action Pipeline. Per-layer content is defined by the preset source in `data/prompt_presets/`.

## §1 Model Configuration

LLM connections are stored as a list on the `settings` singleton row. Each connection carries an identity (id + name, surfaced in the dashboard), a `LlmBackendType` discriminator (`openrouter` / `deepseek` / `ollama` / `mock`), a model identifier, an `api_key` with provider-specific env-var fallback (OpenRouter and DeepSeek fall back to `OPENROUTER_API_KEY` when no stored key is set), a `base_url` endpoint, a single-user-message toggle, and per-connection token caps (`max_tokens` for the response, `max_context_tokens` for the context window). The full set of fields and their per-provider defaults lives in `src/domain/model/settings.rs`; this reference does not restate them.

Two named connection ids are read from settings: `narration_connection_id` (main narrative call) and `quantifier_connection_id` (post-narration scene analysis). They may resolve to the same connection record or to different ones; the wiring builds a dedicated `LlmCallRecorder` for each.

Settings are loaded once at startup and held as `Arc<RwLock<AppSettings>>`. No business logic reloads settings from disk.

## §2 Backend Selection

Four adapters implement the `LlmProvider` port; the dispatcher selects by `provider`:

- **`openrouter`** — `OpenRouterBackend`. Calls the OpenRouter chat-completions endpoint with the connection's `api_key` (or the env-var fallback) and the connection's `model`.
- **`deepseek`** — `DeepSeekBackend`. Wired into the `LlmProvider` port and selected by `provider = "deepseek"`, but every call returns `EngineError::Config` because the transport layer is not implemented. The adapter exists so configuration can name DeepSeek without the engine crashing at load time.
- **`ollama`** — `OllamaBackend`. Calls a local Ollama instance with the connection's `base_url` and `model`. Ollama does not apply a native chat template, so the adapter holds the responsibility for any user-message preprocessing (see "Response Sanitization & Gemma 4 Thinking-Channel Suffix" below).
- **`mock`** — `MockBackend`. In-process deterministic backend used by tests; no API key or endpoint required.

## §3 Game Master Role

The narrator is an LLM acting as a Game Master for a text adventure. All non-empty player input that does not match a recognized system command is treated as a **Free Action** and sent to the LLM for narration. The narrator produces a single paragraph describing the outcome; the engine then runs the post-generation quantifier to detect NPCs and movement, evaluates NPC triggers, and may generate a continuation narration.

The narrator is narrative-only. State mutation is the engine's job, run through the action pipeline after the LLM has spoken.

### Game Master Context

The Game Master's prompt is built from the current game state at action receipt:

- **World lore** — `WorldCard.global_rules` injected into the system prompt; `<WorldLore>` user data layer carries only `world.name` and `world.description`.
- **Room context** — `Room.name` and `Room.description` for the player's current room.
- **Present NPCs** — `NpcCard`s located in the current room, with `personality`, `scenario`, and `description`.
- **Player identity** — `PersonaCard.name` and `PersonaCard.description`.
- **Conversation history** — full narrative history, up to the FIFO cap.

History is sent in full and trimmed oldest-first if it exceeds the cap. Token budget enforcement is deterministic and happens during assembly.

## §4 Narrative Modes

The Game Master responds to three primary events:

1. **Free Actions** — non-command text input. The default mode for any non-empty input.
2. **Dialogue** — NPC speech embedded within a free-action narration. There is no separate "speak" command; NPC lines appear inside the narrator's paragraph when context calls for them.
3. **Arrivals** — the player entering a new room via quantifier-detected movement. The room dashboard is rendered before narration to provide system context.

## §5 Per-Action Flow

A FreeAction runs through the engine's phase pipeline: state transition → narration → quantifier → trigger evaluation. The α-check at each phase boundary is what makes in-flight shutdown signal cancellation race-safe.

## §6 Continuation Narration

After main narration, the engine evaluates NPC triggers and may generate a continuation narration. The `StoredTriggerContext` carries the previous-turn snapshot so the diff has a stable input.

## §7 Response Sanitization & Gemma 4 Thinking-Channel Suffix

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

## §8 LLM Call Logging & Forensics

Every LLM call flows through `LlmCallRecorder::complete()`. The recorder:

1. Calls the backend's transport (pure API call).
2. Runs `sanitize_llm_output` on the response text.
3. Builds an `LlmMessage` record with the prompts, raw request JSON, raw response JSON, backend name, model name, agent name, sanitized parsed response, optional error message, and timestamp.
4. Saves the record through the `LlmMessageRepository` port.
5. Returns the sanitized result to the pipeline.

Storage holds the table `llm_messages` with a 50-row global cap: each insert prunes older rows so only the most recent 50 calls across all games are retained. The table has no `game_id` column, so the cap is global, not per-game.

The dashboard exposes the table via the LLM Messages tab at `/fragment/llm-messages`. The view shows the latest 50 calls with their prompts, raw request/response, and parsed response.

## §9 Runtime Tracing

The engine uses `log` + `env_logger` for structured runtime diagnostics. Spans and events fire automatically when `RUST_LOG` is set. Key markers cover per-LLM-call model + transport (`[LLM][req:N] Using model: ...`) and quantifier confidence (`[Quantifier] Detected NPCs: ... (confidence: ...)`). The project explicitly does not use `tracing` (see `arch-lint.toml`).

## Document References

- [ADR-004: XML-Structured LLM Prompts](../../../docs/adr/adr-004-xml-prompt-format.md) — XML-sectioned instructions + XML-wrapped data; sanitization rationale.
- [ADR-005: SillyTavern-Style Layered Prompt System](../../../docs/adr/adr-005-layered-prompts.md) — layered prompt architecture and per-layer placement.
- [ADR-006: Quantifier-Driven Game Systems](../../../docs/adr/adr-006-quantifier-systems.md) — dual-LLM architecture; quantifier detects NPCs and movement after narration.
- [ADR-007: Settings System Architecture](../../../docs/adr/adr-007-settings-system.md) — `AppSettings` + connection profiles + `OPENROUTER_API_KEY` env-var fallback.
- [ADR-012: LLM Call Logging and Forensics](../../../docs/adr/adr-012-llm-message-logging.md) — `llm_messages` table + retention + dashboard tab.
- [ADR-022: PromptAssembler Trait Decoupling](../../../docs/adr/adr-022-prompt-assembler.md) — assembly decoupled from transport; preset-driven system prompt.
- [`./prompt_system.md`](./prompt_system.md) — layered prompt architecture + token budget constants + system/user separation + single-user-message mode + prompt-injection sanitization.
- [`./agent_system.md`](./agent_system.md) — post-generation agent that detects NPCs and movement; runs the quantifier's separate prompt.
- [`../game_flow.md#phase-flow`](../game_flow.md#phase-flow) — full pipeline phase sequence + retry flow + cancellation; home of the α-check + `GenerationGuard::Drop`.
- [`../game_flow.md#trigger-evaluation`](../game_flow.md#trigger-evaluation) — trigger evaluation rules that produce continuation narration.
