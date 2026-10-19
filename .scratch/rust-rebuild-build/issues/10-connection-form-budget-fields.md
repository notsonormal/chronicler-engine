# Decide if the Connection form sets the context window and the answer budget

Type: grilling (HITL)
Status: open
Blocked by: —

## Question

A Connection has `max_context_tokens` and `max_tokens`, and the Narrator reads the window per call. But no client can set either value. `ConnectionForm` and its template carry only the name, provider, model, API key, base URL and the single-user-message toggle. `add_connection_handler` writes `max_tokens: None` and `max_context_tokens: None`, and the edit handler never touches them. The window therefore comes only from the provider default of `resolve_max_context_tokens` (Mock 4096, Ollama 8192, OpenRouter and DeepSeek 32768).

- **Option A: add both fields** to the Connection form, with spec scenarios in `docs/specs/settings.md` and tier-1 tests. Empty means the provider default.
- **Option B: add only the context window.** The answer budget stays `None`.
- **Option C: leave it.** The provider defaults are enough, and scenario 20.25 (a Connection edited to a provider with a larger default window) stays the only way to change the window.

Open point for A or B: does a value larger than the model's real window need a check, or is it the player's choice?

## Context

- Source: P4 ([P4: the Agent edge on values](05-p4-agent-edge-on-values.md)): the lead wrote scenario 20.25 against a context-window field the form does not have, and rewrote it.
- Evidence: `src/adapters/driving/http/settings/templates/settings.rs` (`ConnectionForm`); `tests/http/settings.rs::test_connection_context_window_applies_from_the_next_narration`.
