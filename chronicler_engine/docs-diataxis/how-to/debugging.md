---
diataxis: how-to
title: Debugging the Engine
---

> **Diátaxis mode:** How-to. This document gives step-by-step procedures for diagnosing test failures and runtime issues in the Chronicler Engine: inspecting the LLM call forensics table, reading tracing output, and locating the common-failure paths. The problem it solves for the reader is *goal*: a test failed or an in-flight generation misbehaved, what now. For the LLM transport contract, see `./llm_processing.md`; for the error catalog, see `./error_catalog.md`; for the testing policy, see `./testing.md`.

## Quick Start

When a test fails or an in-flight generation misbehaves, work these checks in order:

1. **Query the `llm_messages` table.** The most recent LLM call payload and raw response are persisted there (see `Inspect LLM Call Forensics` below). Start with this — it is the single highest-signal artefact for LLM-driven behaviour.
2. **Run with `RUST_LOG=info`** to see structured tracing spans and events across the action pipeline.
3. **Re-run the failing test with `--nocapture`** (`RUST_LOG=trace cargo test --test <name> -- --nocapture`) to surface `print!` output and full spans for the test binary in question.
4. **Reach for `RUST_LOG=trace` only when `=info` is not enough.** The output volume grows fast; the higher level is the sledgehammer, not the first move.

## Inspect LLM Call Forensics

Every outbound LLM call flows through `LlmCallRecorder::complete()` (`src/application/llm_recorder.rs`). On success the recorder saves a row to the `llm_messages` SQLite table via the `LlmMessageRepository` port. This table is the authoritative forensics source for LLM-driven behaviour.

### Query the table

The engine's default SQLite file lives at `data/chronicler.db` for runtime runs. Three queries cover most diagnostic needs:

```bash
# Find the most recent LLM calls
sqlite3 data/chronicler.db "SELECT id, agent_name, model_name, created_at FROM llm_messages ORDER BY id DESC LIMIT 10;"

# Pull the full payload of a specific call (substitute the id)
sqlite3 data/chronicler.db "SELECT system_prompt, user_prompt, raw_response_json, parsed_response FROM llm_messages WHERE id = 123;"

# Find failed calls
sqlite3 data/chronicler.db "SELECT id, agent_name, error_message, created_at FROM llm_messages WHERE error_message IS NOT NULL ORDER BY id DESC LIMIT 20;"
```

The table holds only the most recent 50 calls globally — every insert prunes older rows, and there is no `game_id` column, so the cap is across all games. If a failure happened more than 50 calls ago, the row is gone.

### Read the columns

Each `llm_messages` row carries:

- `agent_name` — which agent made the call (`narrator`, `quantifier`, etc.).
- `backend_name` and `model_name` — the provider and model actually used.
- `system_prompt` and `user_prompt` — the exact prompts sent on the wire.
- `raw_request_json` and `raw_response_json` — wire-level payloads for replay or diff.
- `parsed_response` — the text returned to the pipeline after sanitisation.
- `error_message` — `NULL` on success; the error text when the call failed.
- `created_at` — ISO 8601 timestamp.

These columns are what the diagnostic queries (above) project; each maps directly to a `CREATE TABLE` column in `src/adapters/driven/storage/db.rs`.

### API Keys Are Not Persisted

API keys live in `AppSettings.api_key` (`src/domain/model/settings.rs`) with an env-var fallback (`OPENROUTER_API_KEY`). They never reach the `llm_messages` table — the column list above has no key column. Prompts and raw responses are stored verbatim by design; they are the diagnostic payload.

## Capture Forensics in Test Runs

Test runs do not write to SQLite. Instead, the test-support module wires in `RecordingForensics` (`src/test_support/recording_forensics.rs`), a spy implementation of `LlmMessageRepository` that captures the same data in memory.

When a test fails:

1. **Check whether the test asserts on `RecordingForensics`** — look for `recording_forensics` or `RecordingForensics::new()` in the test body. If the test captures forensics, the assertion that failed tells you which field is wrong.
2. **Inspect `last_saved_message()`** to see the exact prompts and parsed response the orchestrator persisted.
3. **Count `save_call_count()`** to confirm the recorder fired (or did not fire) the expected number of times.

The recorder captures every attempt, including those that returned a configured error — useful for verifying failure paths.

## Read Tracing Output

The engine uses the `tracing` crate (`src/bootstrap/logging.rs` initialises the subscriber at startup). Spans and events fire when `RUST_LOG` is set; without it, nothing prints.

### Enable trace output

```bash
# Info-level across the whole engine
RUST_LOG=info cargo nextest run <test_name>

# Debug-level for one module
RUST_LOG=chronicler_engine::engine=debug cargo nextest run <test_name>

# Full trace output, with print! output visible
RUST_LOG=trace cargo test --test <name> -- --nocapture
```

Test binaries do not call `init_logging()` themselves — the subscriber is only initialised through `main.rs`. Test fixtures (`tests/test_utils/server.rs`) set the `RUST_LOG` env var on the spawned process so the engine subprocess prints during the test.

### Key span markers

These markers are the load-bearing signals for diagnosing action-pipeline and LLM behaviour. The function names are the seam identifiers — grep for them in the source to find the contract:

- **Action pipeline** — `handle_movement` (room transitions), `apply_npc_events` (NPC enter/leave), `execute_freeaction_impl` (full action lifecycle), `execute_action_impl` (top-level entry point), `retry_last_response_impl`, `retrigger_event_impl`.
- **Trigger evaluation** — `evaluate_triggers`, `check_condition`.
- **Quantifier** — `determine_npcs_in_room` (NPC detection confidence surfaces in the log as `[Quantifier] Detected NPCs: ... (confidence: ...)`).
- **LLM client** — `call_chat_completions` (HTTP request/response lifecycle; `[LLM][req:N] Using model: ...` precedes the call).
- **Offload** — `spawn_blocking` lifecycle (`spawn_blocking: task started`, `spawn_blocking: shutting down before execute_action`, `spawn_blocking: execute_action completed`).

The `tracing` markers across the engine are listed in `src/`; this list is the high-signal subset, not exhaustive.

## Diagnose Common Failures

### Tracing output is empty

**Cause.** `RUST_LOG` is not set, or the test binary does not initialise the subscriber (only `main.rs` does, by design).

**Fix.** Set `RUST_LOG=info` (or `=trace`). For integration tests that spawn the engine as a subprocess, ensure the fixture forwards the env var to the child.

### Test panicked before the LLM call was persisted

**Cause.** The provider returned an error before `LlmCallRecorder::complete()` reached the save step. The recorder persists only after a successful provider response — provider-level failures propagate up before any row is written.

**Fix.** Check `RUST_LOG=trace` output for the in-flight prompt and the failing HTTP call; the prompt appears in the tracing spans even when no row is saved. The error variant and its `First Check` diagnosis are in `./error_catalog.md`.

### SQLite query returns no rows

**Cause.** The 50-row cap prunes older rows on every insert, and the cap is global across games. The failure happened more than 50 calls ago.

**Fix.** Reproduce the failure with `RUST_LOG=trace` and capture the spans, or shorten the gap between the failure and the inspection.

### `RecordingForensics` shows zero save calls

**Cause.** The test wired a different `LlmMessageRepository` (for example, `NoopForensics` or a fresh in-memory mock) instead of `RecordingForensics`.

**Fix.** Confirm the test builder passes `RecordingForensics::new()` into the service wiring. The helper API lives in `./test_support.md`.

## Document References

- [ADR-012: LLM Call Logging and Forensics](../../docs/adr/adr-012-llm-message-logging.md) — the `llm_messages` table + retention cap + dashboard tab.
- [`./llm_processing.md`](./llm_processing.md) — LLM transport and orchestration contract; `LlmCallRecorder` forensics pipeline; per-backend sanitisation.
- [`./error_catalog.md`](./error_catalog.md) — `EngineError` variant catalog with first-check diagnosis per variant.
- [`./testing.md`](./testing.md) — testing policy; the `RecordingForensics` fixture API entry-points; smart-waiting helpers.