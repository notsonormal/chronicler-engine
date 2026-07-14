# Error Catalog

Structured reference for every `EngineError` variant. Use this instead of grepping for strings.

---

## `EngineError::Llm(LlmFailure)`

LLM subsystem errors. The five variants map to distinct failure modes: `EmptyResponse` = backend returned no content; `Http` = provider returned non-2xx status; `ParseError` = response not in expected JSON shape; `Timeout` = request exceeded 180s budget; `Network` = transport-layer failure (DNS, TCP, TLS, gzip truncation).

### `LlmFailure::EmptyResponse`
- **First Check:** Backend logs for `[LLM][req:N] Extracted content via:` — if this line is absent, the model returned an empty content/reasoning field.
- **Common Causes:** Model returned empty content field; prompt too long and truncated; Ollama model unloaded mid-request.
- **Related Invariants:** See `docs/architecture/guardrails.md` §5 Runtime Invariants

### `LlmFailure::Http { status, body }`
- **First Check:** The `status` code. `401` = API key issue; `429` = rate limited; `5xx` = provider outage.
- **Common Causes:** Invalid API key; rate limiting; model routing failure; provider maintenance.
- **Related Invariants:** Connection settings must have a valid API key for non-Ollama backends.

### `LlmFailure::Network { url, detail }`
- **First Check:** Can you reach the URL from the host? `curl -I <url>`
- **Common Causes:** Ollama not running; network partition; DNS failure; overall request timeout (180s); truncated gzip stream; server closed connection.
- **Related Invariants:** Ollama must be reachable at the configured `base_url`.

### `LlmFailure::ParseError { raw_response, expected_format }`
- **First Check:** The `raw_response` field in logs. Is it valid JSON? Does it have the expected shape?
- **Common Causes:** Model returned non-JSON (e.g. raw text); JSON missing `choices[0].message.content`; streaming response when `stream: false` was requested.
- **Related Invariants:** All LLM responses must be valid JSON with a `choices` array containing a `message` object.

### `LlmFailure::Timeout`
- **First Check:** `RUST_LOG=debug` logs for `[LLM][req:N] Request failed after ...`
- **Common Causes:** 180-second overall timeout exceeded; model too slow for prompt size; network congestion.
- **Related Invariants:** None — this is an operational/environmental failure.

---

## `EngineError::Narrative(NarrativeFailure)`

Narrative generation errors. These originate in the prompt builder and backend narration methods.

### `NarrativeFailure::PromptBuild { stage, reason }` (test-fixture only)
- **First Check:** `docs/reference/quantifier_prompt.md` §Token Budget for budget calculation; `docs/system/prompt_system.md` for context-window constants.
- **Common Causes:** Prompt exceeds `max_context_tokens`; history too long; token budget miscalculation.
- **Related Invariants:** In production, context overflow raises `EngineError::ContextOverflow` directly. The `PromptBuild` variant is only constructed in `src/error_tests.rs` (test fixture); no production path uses it.

### `NarrativeFailure::Generation { stage, reason }`To b
- **First Check:** Backend-specific logs. Mock backend uses `stage: "mock"` (narration path) or `stage: "mock_trigger"` (trigger-continuation path).
- **Common Causes:** LLM call failed after prompt built successfully; backend misconfiguration (e.g. DeepSeek not implemented).
- **Related Invariants:** See `docs/architecture/guardrails.md` §5 Runtime Invariants

---

## `EngineError::Internal(InternalError)`

Logic invariant violated. These should never happen in normal operation.

- **First Check:** The `invariant` field names the violated rule.
- **Common Causes:** State corruption — invariants about message-history ordering, room-map consistency, NPC-set consistency, or log ordering are violated. The stable invariant strings are matched by `tests/infrastructure/invariant_contract.rs`. Recovery: reload state from snapshot; the heal path runs on the next action.
- **Related Invariants:** `docs/architecture/guardrails.md` §5 Runtime Invariants (INV-002)

---

## `EngineError::Io(String)`

Filesystem or network I/O error.

- **First Check:** File path in the error message; permissions; disk space.
- **Common Causes:** Missing world directory; read-only data directory; failed to create HTTP client (rare, usually wrapped in `LlmFailure::Network`).
- **Related Invariants:** `data/worlds/` must exist and be readable.

---

## `EngineError::Serde(serde_json::Error)`

JSON deserialization failure.

- **First Check:** The file path in the surrounding `DataLoad` or `Parse` error.
- **Common Causes:** Schema mismatch in data files; manual JSON editing introduced syntax errors.
- **Related Invariants:** Run `python build.py --validate-data` to catch schema mismatches early.

---

## `EngineError::Parse(String)`

Generic parse failure (not JSON).

- **First Check:** The input string that failed parsing.
- **Common Causes:** Malformed command in `parser.rs`; invalid settings TOML/JSON.
- **Related Invariants:** None.

---

## `EngineError::Serialize(String)`

JSON serialization failure.

- **First Check:** The object being serialized.
- **Common Causes:** Circular references; non-serializable types in state.
- **Related Invariants:** None.

---

## `EngineError::Navigation(String)`

Player movement failure.

- **First Check:** `current_room_id` and available exits.
- **Common Causes:** Attempting to walk in a direction with no exit; room ID typo.
- **Related Invariants:** `docs/system/navigation.md`

---

## `EngineError::RoomNotFound(String)`

Room lookup failure.

- **First Check:** `state.movement.current_room_id` — if it starts with `dynamic_`, the quantifier returned an unrecognized destination.
- **Common Causes:** `room_id` mismatch between map and trigger; quantifier movement detection returned an unknown room.
- **Related Invariants:** `docs/system/dynamic_rooms.md`

---

## `EngineError::GameNotFound(u64)`

Game row lookup failure on a required read.

- **First Check:** The `game_id` in the error payload; verify it exists in the `games` table.

## `EngineError::PersonaNotFound(String)`

Persona row lookup failure on a required read.

- **First Check:** The persona key in the error payload; verify it exists in the `personas` table.

---

## `EngineError::WorldNotFound(String)`

World row lookup failure on a required read.

- **First Check:** The world key in the error payload; verify it exists in the `worlds` table.

---

## `EngineError::MessageNotFound(u64)`

Message row lookup failure on a required read.

- **First Check:** The `message_id` in the error payload; verify it exists in the `messages` table for the current game.

---

## `EngineError::Config(String)`

Settings or backend configuration error.

- **First Check:** `src/settings.rs` and the backend that raised the error.
- **Common Causes:** Missing settings file; backend not implemented (e.g. DeepSeek); lock poisoned in server state.
- **Related Invariants:** `docs/system/startup.md`

---

## `EngineError::Template(String)`

Askama template render failure.

- **First Check:** Template file syntax; variable names in template context.
- **Common Causes:** Template variable renamed in Rust but not in HTML; HTML syntax error in template.
- **Related Invariants:** `src/adapters/driving/http/templates.rs`

---

## `EngineError::DataLoad { path, source }`

Data file loading failure with nested cause.

- **First Check:** The `path` field; verify the file exists and is valid JSON.
- **Common Causes:** File missing; JSON syntax error; schema mismatch.
- **Related Invariants:** Run `python build.py --validate-data`

---

## `EngineError::ContextOverflow { requested, max }`

Prompt exceeds token budget.

- **First Check:** `docs/reference/quantifier_prompt.md` §Token Budget for budget calculation.
- **Common Causes:** History too long; system prompt too large; combined context exceeds `max_context_tokens`.
- **Related Invariants:** Prompt builder must never exceed the configured context window.

## Document References

- [system/storage.md](../system/storage.md) — `get_*`/`require_*` read contract + `EngineError` not-found variants
