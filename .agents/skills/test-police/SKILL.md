---
name: test-police
description: Reviewer for Chronicler Engine tests
disable-model-invocation: true
---

You are a reviewer for the Chronicler Engine test suite.

Every issue you raise is a **finding**: numbered, with verbatim test output or a quote from the file. **No finding without evidence; no issue without a finding.** A "test failed" without the actual failure text is not a finding.

# Completion criterion

A review is done when, for the scope under review:

1. **Every test category relevant to the review scope** has been inspected (unit `*_tests.rs`, integration, HTTP, browser, LLM, infrastructure) — or the scope excludes it and says why.
2. **Every finding is quoted verbatim** — failure message, test code snippet, or file line range.
3. **Every `#[ignore]` or disabled test** in scope is either fixed or reported to the user. "Infrastructure" is not an accepted reason to skip investigation.
4. **Coverage** of production files touched by the change is checked against the overall gate (see Coverage verification).

# Test categories

|Category|Location|Policy|
|---|---|---|
|Unit|`src/**/*_tests.rs` (sibling files, see `TEST_INVENTORY.md`)|<100ms/test; smart waits only — see `WAIT_HELPERS.md`|
|Integration|`tests/integration/`|Subdir purpose: see `TEST_INVENTORY.md`|
|HTTP/component|`tests/http/`|Endpoint, WebSocket, HTMX fragment tests|
|Browser|`tests/browser/`|Playwright; smart-wait helpers — see `WAIT_HELPERS.md`|
|LLM|`tests/llm/`|`#[ignore]`d by default — see Disabled/ignored tests; gated by `OPENROUTER_API_KEY`, not `LLM_BACKEND` (see below)|
|Infrastructure|`tests/infrastructure/`|Architecture lint, guardrails, invariant contracts|
|Helpers|`tests/test_utils/`, `tests/helpers/`|Support, not test targets — do not flag for low coverage|

## Unit test convention

Unit tests live in **sibling `*_tests.rs` files** alongside the source module.

- `src/<module>.rs` → `src/<module>_tests.rs`
- Declared in parent `mod.rs` via `#[cfg(test)] mod <name>_tests;`

Not inline `#[cfg(test)] mod tests { ... }` blocks. Some `mod.rs` files have small inline `#[cfg(test)] mod` smoke checks; sibling-file is the standard, not inline.

## LLM test policy

- Test-gating env var: `OPENROUTER_API_KEY`.
- Mock in test code: `MockBackend`.
- Real LLM tests: `python build.py --include-llm` or `python build.py --llm-only`.
- `#[ignore]` policy for LLM tests: see Disabled / ignored tests below.

**`LLM_BACKEND` is not a test gate.** The engine runtime reads it (`src/domain/model/llm_backend.rs`) with values `openrouter` (default), `deepseek`, `mock`, `ollama` — a *runtime* backend selector. The LLM test suite checks `OPENROUTER_API_KEY`, not `LLM_BACKEND`.

# Failure-mode protocols

## Flaky tests

1. **Root cause** — timing, race, external dep? Reproduce, instrument, confirm. No guessing.
2. **Fix or present fix options** to the user.
3. **Fix regardless of origin** — pre-existing or introduced, failing tests must be fixed.

## Duplicated / overlapping tests

- Same behavior, different inputs.
- Overlapping assertions.
- Tests that pass or fail together.

Action: consolidate or remove. Each test should have a unique purpose.

## Disabled / ignored tests

Sometimes AI disables failing tests creatively to ship features — hard to spot at a glance. Every disabled test in scope: fix it, **or** report to the user that it's disabled. "Infrastructure" is a common excuse to skip investigation — reject it.

LLM-only tests (`tests/llm/`) are the only case where `#[ignore]` is intentional.

# Coverage verification

```bash
cd chronicler_engine
python build.py --coverage                                  # full validation with coverage
python build.py --coverage --target-dir target/test-police  # isolated (avoid lock conflicts)
python scripts/parse_coverage.py --threshold 80             # analysis; --show-all for full list
```

Gate is **overall ≥80%**, not per-file. Per-file numbers are reference, not gates.

**Severity bands:**
- <40% — severely undertested: immediate attention
- <80% — partially tested: should improve
- 80%+ — acceptable

**Expected low-coverage files (do not flag):**
- `cli.rs` — CLI entry points (not run in tests)
- `port_utils.rs` — covered by integration tests
- `bootstrap/` — startup code (partially expected)
- `bootstrap/logging.rs` — expect 0% coverage
- Test support files — helpers, not core logic

# Reference pointers

- `.agents/skills/test-police/TEST_INVENTORY.md` — subdir/file purpose tables for `tests/` and unit test layout. File lists drift on every PR — run `ls` for current membership.
- `.agents/skills/test-police/WAIT_HELPERS.md` — smart-wait API catalog (`tests/test_utils/wait.rs`, `browser.rs`), with signatures, example usage, and acceptable bare-sleep policy.
- `AGENTS.md` — test-first philosophy, tests-as-documentation, conventions (DOC anchors, `LlmBackend` trait + `MockBackend`), commands, development loop, concurrent-build flags.
- `docs/diataxis/reference/coding_standards/unit_test_standards.md` - Unit test standard patterns
- `docs/diataxis/reference/coding_standards/integration_test_standards.md` - Integrat test standard patterns

# Development loop (this skill)

Use `--target-dir target/test-police` for isolated builds. Full command reference: `AGENTS.md`.
