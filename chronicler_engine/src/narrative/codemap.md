# chronicler_engine/src/narrative/

## Responsibility

LLM-powered narrative generation. Builds prompts, calls external LLM APIs (OpenRouter), parses quantifier responses for NPC presence and movement detection, and handles scene continuation after trigger events.

## Design

**Key modules:**
- `LlmBackend` trait (`llm.rs`) — Five methods: `generate_dialogue()`, `narrate_action()`, `narrate_arrival()`, `narrate_continuation()`, `narrate_action_from_prompt()`. Factory `get_llm_backend()` selects backend via `LLM_BACKEND` env var.
- `LlmBackendType` enum (`llm.rs`) — `OpenRouter` (default), `DeepSeek` (stub), `Mock` (testing).
- `OpenRouterBackend` (`llm.rs`) — Real implementation: builds prompts, reads `OPENROUTER_API_KEY`, calls `call_openrouter()`.
- `MockBackend` (`llm.rs`) — Returns deterministic strings. Detects movement keywords for mock JSON responses.
- `DeepSeekBackend` (`llm.rs`) — Stub returning "not yet implemented" messages.
- `PromptBuilder` (`prompt.rs`) — 8-layer prompt system (System → GameState → NpcCards → Player → WorldInfo → History → User → Phi). Token budget enforcement (8192 max context, 4096 history, 4096 system, 512 response).
- `PromptLayer` enum (`prompt.rs`) — Layer identifiers for prompt construction.
- `PhiMode` enum (`prompt.rs`) — Controls Layer 7 (Phi) behavior: `Narration` (default) for main actions, `Continuation` for trigger-based scenes.
- `sanitize_for_prompt()` (`prompt.rs`) — Filters `{{...}}` injection patterns.
- `truncate_to_budget()` (`prompt.rs`) — Tail-truncates text to fit token budget (4 chars ≈ 1 token).
- `call_openrouter()` / `call_openrouter_with_model()` (`openrouter_client.rs`) — HTTP POST to OpenRouter API with 60s timeout, JSON response parsing with fallback chain (content → reasoning → reasoning_content).
- `QuantifierBackend` (`quantifier.rs`) — Separate LLM call for NPC presence and movement detection. Returns `QuantifierResult` with confidence levels (High/Medium/Low).
- `QuantifierPromptBuilder` (`quantifier.rs`) — Builds XML-structured prompts for the quantifier model.
- `parse_quantifier_response()` (`quantifier.rs`) — Three-strategy parsing: JSON → text fallback → empty.

**Patterns:**
- Trait-based backend selection enables mock testing without API calls
- PromptBuilder uses layered composition with budget checking
- PhiMode controls continuation-specific instructions in Layer 7
- Quantifier uses JSON-first parsing with text fallback and confidence grading
- Token estimation uses `chars.count().div_ceil(4)`

## Flow

1. Player action → `PromptBuilder::from_context()` → system + user prompts
2. `LlmBackend::narrate_action()` → `call_openrouter()` → narrative text
3. For triggers: `narrate_continuation()` with `PhiMode::Continuation` → continuation narration
4. For room changes: `QuantifierBackend::quantify_room()` → `QuantifierResult` → update `state.npcs_in_area`

## Integration

- **Consumes**: `model/` types via `PromptContext` (world, room, NPCs, player, history)
- **Produces**: Narrative text strings, `QuantifierResult` (NPC IDs, movement)
- **Consumed by**: `server/` (renders narrative to client), `engine/` (trigger evaluation uses continuation)
- **External**: OpenRouter API (`https://openrouter.ai/api/v1/chat/completions`)
