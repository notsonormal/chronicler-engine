# LLM Infrastructure Improvements

**Status:** Planned  
**Created:** 2026-05-03  
**Source:** Analysis of Marinara-Engine LLM infrastructure (`docs/reference/marinara_engine.md`)  
**Priority:** Medium — not blocking current work  

---

## Goal

Port or adapt proven patterns from Marinara-Engine's TypeScript LLM layer into chronicler_engine's Rust narrative stack. Improve robustness when using reasoning models (Gemma 4, DeepSeek-R1, QwQ, o-series) and add provider-specific parameter adaptation.

---

## Background

During Gemma 4 26B-A4B integration (May 2026), we discovered that chronicler_engine's LLM client has several gaps compared to Marinara-Engine:

1. **~~Fixed `max_tokens` (1024)~~** ✅ *Fixed 2026-05-03* — `Connection.max_tokens` + `fit_messages_to_context()` provide dynamic budget management
2. **~~No context fitting~~** ✅ *Fixed 2026-05-03* — `fit_messages_to_context()` trims history and caps `max_tokens` to fit the connection's context window
3. **Limited reasoning extraction** — Only handles `content`, `reasoning`, and `reasoning_content` string fields
4. **No model-specific parameter adaptation** — All models get identical payload structure

**Gemma 4 26B loop — SOLVED 2026-05-03**: The abliterated 26B model (`mradermacher/gemma-4-26b-a4b-it-abliterated:iq2xs`) was burning all completion tokens in an infinite `<|channel>thought` reasoning loop, returning empty `content`. The fix was **not** switching models — `gemma4:e4b` is only 8B parameters. Instead, `apply_gemma4_thinking_suffix()` was added to `llm_client.rs`. It detects Gemma 4 models by name and appends the chat-template closure marker (`<turn|>\n<|turn>model\n<|channel>thought\n<channel|>`) to the user message. This tells the model the thinking slot is already filled, bypassing the loop entirely. Validated via direct API testing: 2048 tokens all-reasoning → 211 tokens of actual narrative content.

This plan captures the remaining deeper infrastructure improvements.

---

## Improvements

### Phase 1: Reasoning Format Compatibility (Low Effort)

**Problem:** OpenRouter and newer providers are moving to structured reasoning formats that chronicler_engine doesn't parse.

**Changes:**
- [ ] Update `extract_content_from_response()` in `llm_client.rs` to handle `reasoning_details` arrays
  ```json
  {
    "message": {
      "content": "...",
      "reasoning_details": [
        {"type": "reasoning.text", "text": "..."},
        {"type": "reasoning.summary", "summary": "..."}
      ]
    }
  }
  ```
- [ ] Add fallback: if `content` is an array of typed blocks (Anthropic-style via OpenRouter), extract `text` blocks

**Files:**
- `chronicler_engine/src/narrative/llm_client.rs`
- `chronicler_engine/src/narrative/llm_client.rs` (tests)

**Acceptance Criteria:**
- Unit tests pass for `reasoning_details` array extraction
- Unit tests pass for content block array extraction
- Existing behavior unchanged for string-field responses

---

### Phase 2: Model-Specific Parameter Adaptation (Medium Effort)

**Problem:** Some models require special payload structure. chronicler_engine sends `"system"` role and omits temperature universally, which breaks on newer model families.

**Changes:**
- [ ] Add `model_family` or `capabilities` detection to `Connection` or `LlmBackend`
- [ ] Adapt role: `o1`/`o3`/`o4`/`gpt-5` use `"developer"` instead of `"system"`
- [ ] Adapt temperature: `o1`/`o3`/`o4` and Claude Opus 4.7+ don't support `temperature`/`top_p`
- [ ] Adapt reasoning config: GLM uses boolean `thinking`; OpenAI uses `reasoning_effort`
- [ ] Store these as connection metadata (not hardcoded per model ID)

**Files:**
- `chronicler_engine/src/model/settings.rs` — add capability flags to `Connection`
- `chronicler_engine/src/narrative/llm_client.rs` — adapt payload construction
- `chronicler_engine/src/narrative/llm.rs` — wire through backends
- `chronicler_engine/data/settings.json` — update connection schemas

**Acceptance Criteria:**
- GPT-5 connections use `"developer"` role
- o1/o3 connections omit `temperature`
- Existing OpenRouter/Ollama connections unchanged

---

### Phase 3: Dynamic Context Budget Management ✅ *Completed 2026-05-03*

**Problem:** `DEFAULT_MAX_TOKENS = 1024` is too small for reasoning models with long system prompts. Hardcoding 2048 wastes tokens on simple prompts. There's no calculation of "how much input vs output budget do we actually have?"

**Changes:**
- [x] Add `max_context_tokens` field to `Connection` (defaults: 8192 Ollama, 32768 OpenRouter/DeepSeek, 4096 Mock)
- [x] Implement token estimation for system prompt + message history
- [x] Calculate `max_tokens` dynamically via `fit_messages_to_context()`:
  ```
  usable_window = max_context_tokens - safety_margin
  input_budget = estimated_input_tokens
  max_tokens = min(requested_max_tokens, usable_window - input_budget - safety_margin)
  ```
- [x] Trim message history if input exceeds budget (keep newest, drop oldest)
- [ ] Reserve extra budget for reasoning models (e.g., 50% overhead for thinking) — *deferred to future work*

**Files:**
- `chronicler_engine/src/narrative/prompt.rs` — `fit_messages_to_context()` + token estimation
- `chronicler_engine/src/model/settings.rs` — `max_context_tokens` on `Connection`
- `chronicler_engine/src/narrative/llm.rs` — backends wire connection context window into `PromptBuilder`

**Acceptance Criteria:**
- [x] A 5000-char system prompt + 1024 requested max_tokens doesn't exceed model context window
- [ ] Reasoning models get 1.5x-2x token overhead automatically — *deferred*
- [x] Oldest history entries are dropped when budget is exceeded
- [x] Unit tests for edge cases (very long prompts, empty history, etc.)

---

### Phase 4: Reasoning Control Per Connection (Low-Medium Effort)

**Problem:** Some reasoning models allow disabling or controlling the thinking channel. chronicler_engine has no way to leverage this.

**Changes:**
- [ ] Add `enable_thinking: Option<bool>` to `Connection`
- [ ] Add `reasoning_effort: Option<String>` to `Connection` ("low" / "medium" / "high")
- [ ] Pass through to provider payloads:
  - OpenAI: `reasoning_effort` field
  - OpenRouter: `include_thinking` or similar
  - Ollama: may require template overrides (research needed)
- [ ] When `enable_thinking = false`, add loop-breaker text to system prompt:
  ```
  Begin the narrative response immediately. Do not analyze the prompt structure.
  ```

**Files:**
- `chronicler_engine/src/model/settings.rs`
- `chronicler_engine/src/narrative/llm_client.rs`
- `chronicler_engine/src/narrative/prompt.rs`

**Acceptance Criteria:**
- Connection with `enable_thinking: false` produces faster, direct responses
- Connection with `reasoning_effort: "high"` produces more thorough responses
- UI/settings panel exposes these options

---

## Out of Scope

These Marinara-Engine features are **not applicable** to chronicler_engine's architecture:

- **Client-side streaming think-tag filter** — chronicler_engine uses blocking HTTP, not SSE streams. Ollama handles inline thinking tags at the API level.
- **Typewriter UI effects** — HTMX fragment swaps don't support streaming token-by-token display.
- **Multimodal content arrays** — chronicler_engine doesn't support image inputs (yet).

---

## Dependencies

- Requires Rust 1.85+ (already satisfied)
- May need `tiktoken-rs` or similar for token estimation (Phase 3)
- No breaking changes to existing connections without `max_tokens` set

---

## Related Documents

- `docs/reference/marinara_engine.md` — Full analysis of Marinara-Engine patterns
- `chronicler_engine/src/narrative/llm_client.rs` — Current extraction logic
- `chronicler_engine/src/model/settings.rs` — Connection schema
