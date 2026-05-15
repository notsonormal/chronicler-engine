# ADR-013: LLM Call Logging and Forensics

## Status
Accepted

## Context
When the engine misbehaves — a test fails, narration quality degrades, or a quantifier produces unexpected NPC events — diagnosis currently requires:
1. Reproducing the failure (often involving LLM nondeterminism)
2. Reading source code to infer what prompts were sent
3. Adding temporary `println!` or log lines and rerunning

This inferential loop is slow and unreliable. We need structured forensics that capture the complete decision path for every LLM call.

## Decision
We will log every LLM call to a SQLite table with a strict global cap, and expose it via a dedicated dashboard tab.

### Key Decisions

1. **Unified logging at the HTTP client level** — `call_chat_completions()` in `llm_client.rs` is the single chokepoint for all LLM traffic (narrator + quantifier). Adding logging here captures everything consistently without duplicating logic across 4 backend implementations and the quantifier path.

2. **`LlmBackend` trait returns rich result types** — `call_chat_completions()` returns `ChatCompletionResult { text, system_prompt, user_prompt, raw_request_json, raw_response_json }`. The `LlmBackend` trait wraps this in `LlmCallResult` with `backend_name`, `model_name`, and `agent_name`. Callers extract `.text`; the raw metadata is available for logging.

3. **Agent name tagging** — Every LLM call is tagged with an agent name (`narrator`, `quantifier`, `trigger`, `dialogue`). This enables filtering and attribution in the forensics UI.

4. **SQLite auto-pruning** — `SqliteLlmMessageStorage::save()` wraps insert + "DELETE oldest" in a transaction. No background jobs, no runtime configuration. The 50-row cap is hardcoded and global.

5. **Flat global log** — No `turn_id` foreign key. The log survives game resets and supports future multi-game scenarios.

6. **Storage trait abstraction** — `LlmMessageStorage` trait with `save()` and `list_latest()`. This enables:
   - `SqliteLlmMessageStorage` for production
   - `InMemoryLlmMessageStorage` for tests
   - `None` (logging skipped) for tests that don't care about forensics

7. **Dashboard integration** — HTMX-polling fragment (`/fragment/llm-messages`) renders the last 50 calls as a compact expandable list. Oldest-first order matches narrative chronology.

## Consequences

### Positive
- **Faster diagnosis**: Full request/response JSON is preserved for every call. No more guessing what prompt was sent.
- **No external dependencies**: Uses existing SQLite connection. No new services or infrastructure.
- **Bounded storage**: 50-row cap prevents unbounded growth. At ~1KB per row, total storage is negligible.
- **Test-friendly**: In-memory storage enables test assertions on logged calls without file I/O.
- **Non-breaking**: Existing code paths continue to work when storage is `None`.

### Negative
- **50-row cap is hardcoded**: Not configurable per-deployment. If more history is needed, the cap must be changed in code.
- **No structured query**: The flat schema supports list-by-time but not filtering by agent or model. Complex forensics may require ad-hoc SQL.
- **Privacy consideration**: Raw prompts include player input and world lore. The table is local-only (SQLite), but backups or log shipping would need redaction.

## Alternatives Considered

### Structured tracing (`tracing` crate)
Rejected for this phase. `tracing` spans/events are excellent for request-level tracing but do not persist raw request/response JSON in a queryable form. We may adopt `tracing` later (see observability plan) as a complementary layer.

### Per-turn log with foreign key
Rejected. A `turn_id` foreign key would tie LLM logs to the narrative state, but this breaks on game reset and complicates the schema. The flat log is simpler and more resilient.

### File-based logging (JSON lines)
Rejected. SQLite provides atomicity, queryability, and integrates with the existing `DbPool`. File-based logs would require rotation, parsing, and separate query tooling.

## References
- `docs/system/llm_processing.md` — Technical details of the logging integration
- `docs/system/dashboard.md` — LLM Messages tab UI specification
- `src/storage/llm_message_storage.rs` — Storage trait and implementations
- `src/model/llm_message.rs` — Data model
