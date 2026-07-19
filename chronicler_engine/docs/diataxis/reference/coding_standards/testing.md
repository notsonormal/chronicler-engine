---
diataxis: reference
title: Testing
---

## Real-LLM Tests

`tests/llm/` is the only binary that exercises real LLM providers. It gates itself on `has_llm_api_key()` — the runtime check returns early when `OPENROUTER_API_KEY` is unset in the environment, so no provider call happens by default. The suite runs only under `python build.py --llm-only`. The gatestand mechanism is in `tests/llm/flow_llm_tests.rs::with_real_llm`.

## UI Tests

UI tests run via Playwright (`playwright-rs`). The browser binary is `tests/browser/`. Setup requires Node 18+ and `npx playwright install chromium`. The canonical entry-point for new browser tests is the page-fixture helper at `tests/test_utils/browser.rs`, which spawns the real engine on a file-locked test port and returns a typed page wrapper.

```bash
HEADED=1 cargo nextest run --test browser <test_name>
```

Diagnostics on failure land in `chronicler_engine/tmp/screenshots/` (PNG) and `tmp/test_diagnostics/` (DOM dumps).

## Smart Waiting

Tests poll for conditions rather than `sleep`. The helpers live in `tests/test_utils/wait.rs`: `wait_for_llm_idle`, `wait_for_status_ready`, and `wait_for_element_children`. Each helper retries until the condition is met or a per-helper timeout fires; the helpers are the contract for browser and HTTP-test synchronization.

For unit tests of concurrency invariants, the `wait_for_condition` helper is file-local at `src/application/is_generating_invariant_tests.rs:215` — it is local by design and stays scoped to that file.

## LLM-call test helpers

`RecordingForensics` (`src/test_support/recording_forensics.rs`) is a spy implementation of `LlmMessageRepository` that captures every LLM call in memory for test assertion. It is a test-writing fixture, not a runtime-debugging tool — production runs write to the SQLite `llm_messages` table via `LlmCallRecorder`, and a `/fragment/llm-messages` UI tab exists for runtime inspection.

Two reader methods expose the captured calls:

| Method | Returns | Use |
|--------|---------|-----|
| `last_saved_message()` | `Option<&LlmMessage>` | The exact prompts and parsed response the orchestrator persisted for the most recent call |
| `save_call_count()` | `usize` | Number of recorder fires — confirms the recorder did (or did not) fire the expected number of times |

The recorder captures every attempt, including those that returned a configured error, which makes the two readers the right hook for verifying failure-path assertions.

## Document References

- [`./unit_test_standards.md`](./unit_test_standards.md) — canonical nine-pattern form for `*_tests.rs` unit tests, with four cross-cutting patterns (XSS regression is Cross-cutting B).
- [`./integration_test_standards.md`](./integration_test_standards.md) — canonical seven-pattern form for tests under `tests/`, with eight cross-cutting patterns.
- [`./guardrails.md`](./guardrails.md) — coverage-exclusion policy and the test-module-header convention guardrail.
- [`tests/AGENTS.md`](../../../tests/AGENTS.md) — live structure index for the integration test tree and the TEST MIRROR CONVENTION.
- [`scripts/check_test_structure.py`](../../../scripts/check_test_structure.py) — enforces `*_tests.rs` sibling-file layout (no inline `#[cfg(test)]` modules).
- [ADR-028: Test Module Header Convention](../../../docs/adr/adr-028-test-module-header-convention.md) — the single-line `//! <summary>` convention on every `*_tests.rs` file.