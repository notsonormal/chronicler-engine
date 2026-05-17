# Testing Strategy

 | **Reference**: [reference/testing.md](../reference/testing.md)

> **Note**: See `docs/reference/testing.md` for command reference and coverage thresholds.

## Overview

The Chronicler Engine uses a dual-layer testing strategy:

1. **Unit tests** (`#[cfg(test)]` in source files) - Core logic, zero overhead
2. **Integration tests** (`tests/` directory) - End-to-end with HTTP server

## Test Organization

Tests are organized by execution model:

| File / Directory | Purpose | Execution Model | Runtime |
|----------------|---------|---------------|---------|
| `architecture.rs` | Architecture guardrails (clippy, import order, doc anchors) | In-process | Very Fast |
| `components/` | Templates, endpoints, settings, validation, fragments | In-process | Very Fast |
| `browser/` | UI structure, layouts, interactions, editing | Browser | Medium |
| `flow_mock/` | Game loop, retry, polling, state consistency | In-process + Mock LLM | Fast |
| `flow_llm_tests.rs` | LLM narrative generation | Browser + Real LLM | Slow |
| `game_service/` | Game service logic, action handling, retry | In-process | Very Fast |
| `guardrails/` | Custom guardrails (what-comments, long comment runs, single-letter vars) | In-process | Very Fast |
| `logic_tests.rs` | Movement, room resolution, fuzzy matching | In-process | Very Fast |
| `snapshot_storage_tests.rs` | SQLite snapshot persistence, checkpoints | In-process | Very Fast |
| `state_snapshot_tests.rs` | Snapshot serialization/deserialization | In-process | Very Fast |
| `test_data.rs` | Shared test fixtures (world, map, game state builders) | In-process | Very Fast |
| `trigger_tests.rs` | Trigger evaluation and firing | Browser + Mock LLM | Fast |
| `text_check_tests.rs` | Spell/grammar checking | In-process | Very Fast |
| `diagnostic/` | Backend diagnostics, scenario validation | In-process | Very Fast |

## Test Files Explained

### In-Process Tests (`components/`, `game_service/`, `guardrails/`)

Fast tests that don't spawn a browser:

- **Template tests**: Askama template rendering, XSS escaping
- **Fragment tests**: HTTP endpoint responses
- **Validation**: Empty command rejection
- **Game service**: Action handling, retry logic, trigger evaluation
- **Snapshot storage**: SQLite persistence, checkpoint CRUD
- **Guardrails**: Code style enforcement

Runtime: ~5 seconds

### Browser Tests (`browser/`)

Full browser automation via Playwright:

- **UI structure**: Header, story log, action area exist
- **Layout**: Overflow, positioning, scrollability
- **Interactions**: Form submission, element updates
- **Editing**: Inline edit, save/cancel flow

Runtime: ~60 seconds

### Flow Tests (`flow_mock/`)

- **sequence.rs**: Sequential service-level flow tests with mock backends
- **retry_main.rs**: Main narration retry via snapshot rollback
- **retry_event.rs**: Event continuation retry preserving quantifier results
- **flow_llm_tests.rs**: Full integration - real API calls

## Running Tests

```bash
# Default: fast suite (~3 min, LLM tests excluded)
cargo nextest run
python build.py

# Fast suite only (in-process)
cargo nextest run --test components
cargo nextest run --test game_service

# Browser tests only
cargo nextest run --test browser
cargo nextest run --test flow_mock

# Include slow LLM tests in full suite
cargo nextest run --run-ignored only
cargo nextest run --run-ignored all
python build.py --include-llm

# Run ONLY the LLM tests (focused validation)
cargo nextest run --test flow_llm_tests --run-ignored only
cargo nextest run --run-ignored all --test flow_llm_tests
python build.py --llm-only
```

## Test Requirements

- Node.js 18+ (for Playwright)
- Chromium: `npx playwright install chromium`
- `OPENROUTER_API_KEY` (for flow_llm_tests only)

## What We Keep

Critical tests that must not be removed:

| Test | Why |
|------|-----|
| `test_header_template_escapes_html` | XSS security - only test |
| `test_action_handler_empty_command` | Validation - rejects blank input |
| `test_story_log_scrollable` | Functional - can't scroll history |
| `test_no_horizontal_overflow` | Regression - breaks page layout |

## What We Removed

| Test | Why |
|------|-----|
| `test_htmx_loaded` | Tests CDN, not our code |
| `test_ws_extension_loaded` | Placeholder - just logs |
| `test_llm_error_shows_in_story_log` | Not implemented |
| Duplicate existence checks | Single test verifies |

## Runtime Expectations

| Suite | Runtime |
|-------|---------|
| Full suite (`python build.py`) | ~70 sec (LLM tests excluded) |
| components | ~5 sec |
| browser | ~60 sec |
| flow_mock | ~30 sec |
| game_service | ~5 sec |
| guardrails | ~2 sec |
| logic_tests | ~2 sec |
| snapshot_storage_tests | ~2 sec |
| state_snapshot_tests | ~2 sec |
| trigger_tests | ~30 sec |
| text_check_tests | ~2 sec |
| flow_llm_tests | ~30–120 sec |

## Smart Waiting Patterns

Tests use polling, not fixed sleep:

```rust
// BAD: Fixed delay
sleep(Duration::from_millis(15000)).await;

// GOOD: Wait for condition
wait_for_llm_idle(port, Duration::from_secs(30)).await;
wait_for_status_ready(&page).await;
```

## Test Config

Dynamic port allocation avoids conflicts:

```json
// tests/test_config.json
{
  "port_range": {"min": 3010, "max": 3030},
  "default_backend": "mock"
}
```