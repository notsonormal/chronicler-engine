# Testing Strategy (Consolidated)

> **Note**: This document reflects the consolidated test structure (4 files). See `docs/reference/testing.md` for historical context.

## Overview

The Chronicler Engine uses a dual-layer testing strategy:

1. **Unit tests** (`#[cfg(test)]` in source files) - Core logic, zero overhead
2. **Integration tests** (`tests/` directory) - End-to-end with HTTP server

## Test Organization

After consolidation, tests are organized by execution model:

| File | Purpose | Execution Model | Runtime |
|------|---------|---------------|---------|
| `flow_mock_tests.rs` | Game loop, polling, real-time updates | Browser + Mock LLM | Fast |
| `flow_llm_tests.rs` | LLM narrative generation | Browser + Real LLM | Slow |
| `component_tests.rs` | Templates, endpoints, validation | In-process | Very Fast |
| `e2e_tests.rs` | UI structure, layouts, interactions | Browser | Medium |

## Test Files Explained

### In-Process Tests (component_tests.rs)

Fast tests that don't spawn a browser:

- **Template tests**: Askama template rendering, XSS escaping
- **Fragment tests**: HTTP endpoint responses
- **Validation**: Empty command rejection

Runtime: ~5 seconds

### Browser Tests (e2e_tests.rs)

Full browser automation via Playwright:

- **UI structure**: Header, story log, action area exist
- **Layout**: Overflow, positioning, scrollability
- **Interactions**: Form submission, element updates

Runtime: ~60 seconds

### Flow Tests (separate per intent)

- **flow_mock_tests.rs**: Fast CI - mocked LLM responses
- **flow_llm_tests.rs**: Full integration - real API calls

## Running Tests

```bash
# Default: fast suite (~3 min, LLM tests excluded)
cargo test
python build.py

# Fast suite only (in-process)
cargo test --test component_tests

# Browser tests only
cargo test --test e2e_tests
cargo test --test flow_mock_tests

# Include slow LLM tests in full suite
cargo test -- --ignored
cargo nextest run --run-ignored all
python build.py --include-llm

# Run ONLY the LLM tests (focused validation)
cargo test --test flow_llm_tests -- --ignored
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

| Suite | Before | After |
|-------|--------|-------|
| All tests | ~15 min | ~3 min |
| component_tests | N/A | ~5 sec |
| e2e_tests | ~6 min | ~1 min |
| flow_mock_tests | ~3 min | ~1 min |

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