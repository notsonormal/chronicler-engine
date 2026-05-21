# ADR-012: LLM Call Logging and Forensics

**Date:** 2026-05-09

---

## Context

When the engine misbehaves — a test fails, narration quality degrades, or a quantifier produces unexpected NPC events — diagnosis currently requires:

1. Reproducing the failure (often involving LLM nondeterminism)
2. Reading source code to infer what prompts were sent
3. Adding temporary log lines and rerunning

This inferential loop is slow and unreliable. We need structured forensics that capture the complete decision path for every LLM call.

---

## Decision

**Log every LLM call to a SQLite table with a strict global cap, and expose it via a dedicated dashboard tab.**

### Key architectural choices

1. **Unified logging at the HTTP client level** — The LLM client layer is the single chokepoint for all LLM traffic (narrator, quantifier, triggers). Logging here captures everything consistently without duplicating logic across backend implementations or agent paths.

2. **Rich result types** — The backend interface returns not just generated text, but also the full system prompt, user prompt, raw request JSON, and raw response JSON. Callers extract the text; the metadata is available for logging.

3. **Agent name tagging** — Every LLM call is tagged with an agent identifier (narrator, quantifier, trigger, dialogue). This enables filtering and attribution in the forensics UI.

4. **SQLite auto-pruning** — Insert is wrapped in a transaction with automatic deletion of oldest rows. No background jobs, no runtime configuration. The row cap is hardcoded and global.

5. **Flat global log** — No foreign keys to game state or snapshots. The log survives game resets and supports multi-game scenarios.

6. **Storage trait abstraction** — A storage trait with save and list operations enables production (SQLite), test (in-memory), and no-op implementations.

7. **Dashboard integration** — An HTMX-polling fragment renders the latest calls as a compact expandable list, oldest-first to match narrative chronology.

---

## Consequences

### Positive

- **Faster diagnosis**: Full request/response JSON is preserved for every call. No more guessing what prompt was sent.
- **No external dependencies**: Uses existing SQLite connection. No new services or infrastructure.
- **Bounded storage**: Hard row cap prevents unbounded growth. Total storage is negligible.
- **Test-friendly**: In-memory storage enables test assertions on logged calls without file I/O.
- **Non-breaking**: Existing code paths continue to work when logging is disabled.

### Negative

- **Row cap is hardcoded**: Not configurable per-deployment. If more history is needed, the cap must be changed in code.
- **No structured query**: The flat schema supports list-by-time but not filtering by agent or model. Complex forensics may require ad-hoc SQL.
- **Privacy consideration**: Raw prompts include player input and world lore. The table is local-only (SQLite), but backups or log shipping would need redaction.

---

## Alternatives Considered

### Structured tracing (`tracing` crate)

Rejected for this phase. Tracing spans/events are excellent for request-level tracing but do not persist raw request/response JSON in a queryable form. We may adopt tracing later as a complementary layer.

### Narrative state log with foreign key

Rejected. A foreign key would tie LLM logs to the narrative state, but this breaks on game reset and complicates the schema. The flat log is simpler and more resilient.

### File-based logging (JSON lines)

Rejected. SQLite provides atomicity, queryability, and integrates with the existing database pool. File-based logs would require rotation, parsing, and separate query tooling.

---

## History

- **2026-05-09**: Initial implementation — unified logging, rich result types, auto-pruning, dashboard tab
