# Debugging Guide

## Overview

This guide explains how to debug test failures and runtime issues in the Chronicler Engine using the observability infrastructure.

## Quick Start

When a test fails:

1. **Check the `llm_messages` table** (SQLite LLM call log — see ADR-012) for the most recent call payload + raw response
2. **Run with `RUST_LOG=info`** to see structured traces
3. **Re-run the failing test** with `RUST_LOG=trace cargo test -- --nocapture` for full spans/events

## LLM Call Forensics (SQLite)

Per ADR-012, every `LlmCallRecorder::complete()` call persists a row to the `llm_messages` SQLite table via the `LlmMessageRepository` port. This is the authoritative forensics source for LLM-driven behavior.

Each row contains:

- **id**: sequential row id
- **agent_name**: which agent (narrator, quantifier, etc.) made the call
- **backend_name** / **model_name**: provider + model used
- **system_prompt** / **user_prompt**: exact prompts sent
- **raw_request_json** / **raw_response_json**: wire-level payloads
- **parsed_response**: sanitized text after `strip_thought_tags` etc.
- **error_message**: `NULL` on success, error text on failure
- **created_at**: ISO 8601 timestamp

### Querying LLM Forensics

```bash
# Find the most recent LLM calls
sqlite3 data/chronicler.db "SELECT id, agent_name, model_name, created_at FROM llm_messages ORDER BY id DESC LIMIT 10;"

# Pull the full payload of a specific call
sqlite3 data/chronicler.db "SELECT system_prompt, user_prompt, raw_response_json, parsed_response FROM llm_messages WHERE id = 123;"

# Find failed calls
sqlite3 data/chronicler.db "SELECT id, agent_name, error_message, created_at FROM llm_messages WHERE error_message IS NOT NULL ORDER BY id DESC LIMIT 20;"
```

For test runs, `RecordingForensics` (a spy impl of `LlmMessageRepository` at `src/test_support/recording_forensics.rs`) captures the same data in-memory — use it to assert on what the orchestrator persisted.

### Sensitive Data

API keys live in environment variables (see `docs/env.md`) and are **never** stored in `llm_messages`. Prompts and raw responses are stored verbatim by design — they are the diagnostic payload.

## Using Tracing

### Enable Trace Output

```bash
# See all info-level traces
RUST_LOG=info cargo test

# See debug traces for specific modules
RUST_LOG=chronicler_engine::engine=debug cargo test

# See full trace output
RUST_LOG=trace cargo test
```

### Instrumented Functions

The following critical paths are instrumented:

**Action Processing:**
- `handle_movement` - tracks room transitions
- `apply_npc_events` - tracks NPC enter/leave events
- `execute_freeaction_impl` - tracks complete action lifecycle

**Trigger Evaluation:**
- `evaluate_triggers` - tracks trigger firing decisions
- `check_condition` - tracks condition evaluation

**Quantifier:**
- `determine_npcs_in_room` - tracks NPC detection confidence

**LLM Client:**
- `call_chat_completions` - tracks HTTP request/response lifecycle

**Game Service:**
- `execute_action` - top-level action entry point
- `retry_last_response` - retry logic
- `retrigger_event` - event retriggering

## Diagnosis Workflow

### Step 1: Identify the Failure

```bash
cargo test --test your_test 2>&1 | grep "FAILED"
```

### Step 2: Check LLM Forensics

For LLM-driven tests, query the `llm_messages` table (above) to see the exact prompt + response involved in the failure.

### Step 3: Trace the Execution

Re-run with full tracing:

```bash
RUST_LOG=trace cargo test test_name -- --nocapture
```

In the output look for:

1. **Span hierarchy** - What functions were called?
2. **Field values** - What were the inputs?
3. **Events** - What decisions were made?
4. **Error events** - Where did it fail?

### Step 4: Fix and Verify

After fixing the code:

```bash
# Ensure tests pass
cargo test
```

## Common Issues

### No tracing output visible

**Cause:** `RUST_LOG` not set, or test binary doesn't initialize the subscriber.

**Solution:** Set `RUST_LOG=info` (or `=trace` for full detail). Test binaries initialize the subscriber via `bootstrap/run.rs`.

### Test panics before LLM call is persisted

**Cause:** Provider-level error returned before `save_llm_message` fires — by design, the recorder saves only on provider success.

**Solution:** Check `RUST_LOG=trace` output instead of the `llm_messages` table; the in-flight prompt will appear in the tracing spans.

## Related Documentation

- [Testing Guide](../reference/testing.md)
- [System Architecture](../architecture/system.md)
- [Error Catalog](error_catalog.md)
- [ADR-012: LLM Call Logging and Forensics](../adr/adr-012-llm-message-logging.md)
