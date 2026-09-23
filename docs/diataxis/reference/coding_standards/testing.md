---
diataxis: reference
title: Testing
---

## Real-LLM Tests

`tests/llm/` is the only binary that exercises real LLM providers. It gates itself on `has_llm_api_key()`. The runtime check returns early when `OPENROUTER_API_KEY` is unset, so no provider call happens by default. The suite runs only under `python build.py --llm-only`. The gating mechanism is in `tests/llm/flow_llm_tests.rs::with_real_llm`.

## UI Tests

UI tests run via Playwright (`playwright-rs`). The browser binary is `tests/browser/`, which holds two tiers. Setup requires Node 18+ and `npx playwright install chromium`.

Most new browser tests belong to the **stub tier** (`tests/browser/stub/`), which needs no engine process. It drives a stub server that serves the real dashboard shell and canned fragments, so every test reads the shipped client JavaScript without spawning the engine. Each stub test launches its own Chromium via `with_stub_page` (per-test isolation is deliberate — no shared browser runtime); `SharedBrowser` shares one launch across the subtests inside `stub/invariants.rs` only.

The **full-stack tier** (`tests/browser/<surface>.rs`) boots the real engine. Its entry point is the page-fixture helper `with_test_page` at `tests/test_utils/browser.rs`, which spawns the engine on a file-locked test port and returns a typed page wrapper. Full-stack interactions go through the htmx-settle helpers, which the build enforces.

`tests/STRATEGY.md` holds the placement rule that picks a tier for a new test.

```bash
HEADED=1 python build.py test-pattern <test_name>
```

Diagnostics on failure land in `tmp/screenshots/` (PNG) and `tmp/test_diagnostics/` (DOM dumps).

## Smart Waiting

Tests poll for conditions rather than `sleep`. The helpers live in `tests/test_utils/wait.rs`: `wait_for_llm_idle`, `wait_for_status_ready`, and `wait_for_element_children`. Each helper retries until the condition is met or a per-helper timeout fires; the helpers are the contract for browser and HTTP-test synchronization.

## Document References

- [`./unit_test_standards.md`](./unit_test_standards.md) — canonical nine-pattern form for `*_tests.rs` unit tests, with four cross-cutting patterns (XSS regression is Cross-cutting B).
- [`./integration_test_standards.md`](./integration_test_standards.md) — canonical seven-pattern form for tests under `tests/`, with eight cross-cutting patterns.
- [`./guardrails.md`](./guardrails.md) — coverage-exclusion policy and the test-module-header convention guardrail.
- `tests/AGENTS.md` — live structure index for the integration test tree and the TEST MIRROR CONVENTION.
- [`scripts/check_test_structure.py`](../../../scripts/check_test_structure.py) — enforces `*_tests.rs` sibling-file layout (no inline `#[cfg(test)]` modules).
