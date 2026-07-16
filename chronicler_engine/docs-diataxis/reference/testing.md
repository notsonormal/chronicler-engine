---
diataxis: reference
title: Testing
---

> **Diátaxis mode:** Reference. This document is the testing-policy overview that cross-references the canonical sources for each topic: the builder API lives in `./test_support.md`, the unit-tier patterns in `./unit_test_standards.md`, the integration-tier patterns in `./integration_test_standards.md`, the running-test commands in `chronicler_engine/AGENTS.md`, and the coverage-exclusion policy in `./guardrails.md`. The policy items that do not have a canonical home elsewhere — critical test categories beyond XSS, the real-LLM gate mechanism, the Playwright UI test setup, and the smart-waiting stance — live here.

## Overview

The three-doc split for tests `./unit_test_standards.md` (unit-tier patterns), `./integration_test_standards.md` (integration-tier patterns), and this doc (cross-cutting policy that lives outside the three above). Running tests, coverage policy, and clippy/arch-lint enforcement live in `chronicler_engine/AGENTS.md` and `./guardrails.md` respectively.

## Critical Test Categories

These four categories of test must never be deleted without replacement. XSS regression checks are the canonical load-bearing case and are documented in `./unit_test_standards.md` Pattern 7 + Cross-cutting B. The three below are not catalogued elsewhere:

| Category | Why |
|----------|-----|
| Empty / whitespace-only command handling | SillyTavern "Continue" continuation behaviour |
| Scroll behaviour of the story log | Functional regression guard for the history view |
| Horizontal-overflow sanity on rendered pages | Layout regression guard |

When removing any critical-category test, replace with an assertion at least as strong before deletion.

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

For unit tests of concurrency invariants, the `wait_for_condition` helper is file-local at `src/application/is_generating_invariant_tests.rs:215` — it is local by design and stays scoped to that file (see `./unit_test_standards.md` Pattern 8).

## Document References

- [`./test_support.md`](./test_support.md) — builder API + selection rule + fixtures + recording-forensics + integration helpers.
- [`./unit_test_standards.md`](./unit_test_standards.md) — canonical nine-pattern form for `*_tests.rs` unit tests, with four cross-cutting patterns (XSS regression is Cross-cutting B).
- [`./integration_test_standards.md`](./integration_test_standards.md) — canonical seven-pattern form for tests under `tests/`, with eight cross-cutting patterns.
- [`./guardrails.md`](./guardrails.md) — coverage-exclusion policy and the test-module-header convention guardrail.
- [`chronicler_engine/AGENTS.md`](../../AGENTS.md) — essential commands: `python build.py`, `cargo nextest run`, `--llm-only`, `--coverage`, iteration strategy.
- [`tests/AGENTS.md`](../../tests/AGENTS.md) — live structure index for the integration test tree and the TEST MIRROR CONVENTION.
- [`scripts/check_test_structure.py`](../../scripts/check_test_structure.py) — enforces `*_tests.rs` sibling-file layout (no inline `#[cfg(test)]` modules).
- [ADR-028: Test Module Header Convention](../../docs/adr/adr-028-test-module-header-convention.md) — the single-line `//! <summary>` convention on every `*_tests.rs` file.