# Decide if the Connection form sets the context window and the answer budget

Type: grilling (HITL)
Status: resolved
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

## Answer

Owner, 2026-10-19: **option B**. The Connection form gets one field, the context window (`max_context_tokens`). The answer budget (`max_tokens`) stays `None`: `fit_messages_to_context` uses `MAX_RESPONSE_TOKENS` (2048) and fits it to the window.

- **Check (2a):** an empty field means the provider default of `resolve_max_context_tokens`. Any other value must be a positive integer. No lower limit and no upper limit: a large value is the player's choice, and a value too small for the prompt fails at run time with `ContextOverflow`.
- **Edit rule (keep):** an edit that changes the provider keeps an explicit window value. Only an empty field follows the default of the current provider.
- **Spec:** scenario 20.25 becomes "the client sets the window field, and the next narration uses it". A new scenario covers "an empty field uses the provider default". Both get tier-1 tests.

Built the same day:

- `ConnectionForm` gets `conn_max_context_tokens` (`#[serde(default)]`). `resolved_max_context_tokens` applies the check. The add path, the edit path and the form-test path each refuse a bad value. `ConnectionFormTemplate` shows the stored value as a number input.
- Spec scenarios 20.25 (rewritten: the field sets the window), 20.26 (an empty field uses the provider default), 20.27 (a provider change keeps an explicit value) and 20.28 (a bad value is refused, and nothing is saved). Each has a tier-1 test in `tests/http/settings.rs`. Each test fails when its behaviour is removed (checked). One template unit test was added.
- `narration_system.md` has one sentence about the field. `python build.py --no-browser` is green: 1335 passed, 2 skipped. `python build.py browser` is green: 29 passed.
- Not committed: the standing approval names the slices, not the follow-up tickets.
