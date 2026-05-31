# Debugging Guide

## Overview

This guide explains how to debug test failures and runtime issues in the Chronicler Engine using the observability infrastructure.

## Quick Start

When a test fails:

1. **Check for forensics JSON** in `chronicler_engine/tmp/diagnostics/`
2. **Run with `RUST_LOG=info`** to see structured traces
3. **Review the forensics file** for execution state

## Forensics Capture

### Automatic Capture on Test Failure

When a test fails, the `ForensicsCollector` automatically writes a JSON file to:

```
chronicler_engine/tmp/diagnostics/forensics_<test_name>_<timestamp>.json
```

The forensics file contains:

- **test_name**: Name of the failing test
- **timestamp**: When the failure occurred
- **spans**: Hierarchical trace of function calls with fields
- **events**: Log events with levels (info, warn, error)
- **duration_ms**: Test execution time

### Sensitive Data Redaction

The forensics collector automatically redacts:

- `api_key` → `[REDACTED]`
- `prompt` → `[REDACTED]`
- `raw_response` → `[REDACTED]`
- `authorization` → `[REDACTED]`

Long strings (>10KB) are truncated to prevent oversized files.

### Example Forensics File

```json
{
  "test_name": "test_quantifier_high_confidence",
  "timestamp": "2026-05-31T12:34:56Z",
  "spans": [
    {
      "name": "determine_npcs_in_room",
      "id": 1,
      "fields": {
        "confidence": "High"
      }
    }
  ],
  "events": [
    {
      "message": "Using dynamic NPCs",
      "level": "info",
      "fields": {
        "npc_ids": ["npc_1", "npc_2"]
      }
    }
  ],
  "duration_ms": 245
}
```

### Forensics JSON Schema

For tooling integration, the forensics JSON contains:

- `test_name` (string): Test identifier
- `timestamp` (string): ISO 8601 timestamp
- `spans` (array): Function call traces with `{name, id, parent_id, fields}`
- `events` (array): Log events with `{message, level, span_id, fields}`
- `duration_ms` (number): Execution time in milliseconds

Sensitive fields (`api_key`, `prompt`, `raw_response`) are auto-redacted to `[REDACTED]`. Values >10KB are truncated.

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

### Step 2: Review Forensics JSON

```bash
# Find the latest forensics file
ls -lt chronicler_engine/tmp/diagnostics/*.json | head -1

# Review the file
cat chronicler_engine/tmp/diagnostics/forensics_*.json | jq .
```

### Step 3: Trace the Execution

Look for:

1. **Span hierarchy** - What functions were called?
2. **Field values** - What were the inputs?
3. **Events** - What decisions were made?
4. **Error events** - Where did it fail?

### Step 4: Reproduce with Enhanced Logging

```bash
# Run the specific test with full tracing
RUST_LOG=trace cargo test test_name -- --nocapture
```

### Step 5: Fix and Verify

After fixing the code:

```bash
# Ensure tests pass
cargo test

# Clean up old forensics files
rm -rf chronicler_engine/tmp/diagnostics/*.json
```

## Advanced Techniques

### Custom Forensics Collection

For manual forensics capture in tests:

```rust
use chronicler_engine::test_support::ForensicsCollector;

#[test]
fn test_with_forensics() {
    let collector = ForensicsCollector::new();
    collector.set_test_name("my_test");
    
    // ... test code ...
    
    // On failure, manually capture
    collector.capture_on_failure().unwrap();
}
```

### Replay Infrastructure

The replay infrastructure (TODO: implement) allows you to:

1. Load a forensics snapshot
2. Rebuild the exact game state
3. Re-execute the action that failed
4. Compare results

## Common Issues

### No Forensics File Generated

**Cause:** Test panicked before the collector could flush.

**Solution:** Ensure the test framework is properly integrated with the forensics layer.

### Forensics File is Empty

**Cause:** No tracing spans or events were recorded.

**Solution:** Verify `RUST_LOG` is set and the subscriber is initialized in bootstrap.

### API Keys in Forensics

**Cause:** Custom field names not in the redaction list.

**Solution:** Add the field name to `SENSITIVE_FIELDS` in `forensics.rs`.

## Related Documentation

- [Testing Guide](../reference/testing.md)
- [System Architecture](../architecture/system.md)
- [Error Catalog](error_catalog.md)
