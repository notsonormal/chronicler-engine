# Specification: Testing Strategy and Architecture

## Objective

Establish a formal policy for ensuring the Chronicler Engine remains heavily tested locally without incurring costs or latency from external LLM APIs.

## Policy Rules

1. **Isolated Unit Tests**: All modules maintain fully isolated unit tests `#[test]` with zero networking overhead. Unit tests live in separate `*_tests.rs` sibling files (not inline `#[cfg(test)]` blocks) to keep source files under the 2,000-line guardrail.
2. **Integration Capabilities**: Cross-module and end-to-end tests live in the top-level `tests/` directory.
3. **LLM Abstraction (The Trait Pattern)**: No component outside of `main.rs` should be hardcoded to contact an external LLM API. Use `MockBackend` (implements `LlmProvider`) for tests. The `make_test_recorder` helper at `tests/test_utils/mod.rs` wraps `MockBackend` in `LlmCallRecorder` for use with `with_backends`/`with_mock_quantifier`.

## Test File Organization

### Unit Tests (Sibling Files)

Unit tests are co-located with source code in separate `*_tests.rs` files:

```
src/
├── engine/
│   ├── logic.rs
│   └── logic_tests.rs
├── model/
│   ├── state.rs
│   └── state_tests.rs
└── narrative/
    ├── llm_client.rs
    └── llm_client_tests.rs
```

Parent `mod.rs` declares the test module:

```rust
#[cfg(test)]
mod logic_tests;
```

### Integration Tests (`tests/` Directory)

Cross-module and browser-based tests live in the top-level `tests/` directory:

| Test File / Directory | Purpose | Execution Model |
|----------------------|---------|-----------------|
| `architecture.rs` | Architecture guardrails (arch-lint, layer enforcement) | In-process |
| `invariant_contract_tests.rs` | Runtime invariant regression tests | In-process |
| `guardrails.rs` | Custom convention tests (imports, comments, file length) | In-process |
| `integration/` | Cross-module integration tests (application service, game service, lifecycle, pipeline, llm_client, storage, model) | In-process + Mock LLM |
| `http/` | HTTP endpoint tests — action handlers, connections, fragments, status, text check | In-process |
| `browser/` | UI structure, layouts, interactions, editing | Browser (Playwright) |
| `llm/` | LLM narrative smoke tests (real LLM, `#[ignore]` by default) | Real LLM |
| `poison_recovery.rs` | Lock poison recovery for `Mutex`/`RwLock` | In-process |

## Backend Selection

### Dependency Injection (Recommended)

Pass mock backends directly to `DefaultGameService` and call the public `execute_action()` method:

```rust
let service = DefaultGameService::with_mock_quantifier(
    Arc::new(MockBackend::new(None)),
    Arc::new(MockBackend::new(None)),
);

let ctx = GameServiceContext::new(/* ... */);
service.execute_action(ctx, "look around".to_string(), "Player".to_string());
```

**Note:** Do not call `execute_action_impl()` directly in tests — use the public `DefaultGameService::execute_action()` wrapper method instead. The `execute_action_impl()` function is an internal implementation detail of the action pipeline module.

### Pipeline-Only Mocking

For tests that only need to verify pipeline behavior (narration → quantification → trigger continuation), use the `make_test_recorder` helper to construct `Arc<LlmCallRecorder>` with `MockBackend`, then pass to `GameService::with_backends` or `with_mock_quantifier`:

```rust
use chronicler_engine::application::llm_recorder::LlmCallRecorder;
use chronicler_engine::adapters::driven::llm::providers::MockBackend;
use chronicler_engine::application::context::GameServiceContext;
use std::sync::Arc;

// For slow-timing tests, delays must live in the AGENT (not the backend)
// so that run_post_generation_agents honors them.
let mock_backend = MockBackend::default().with_narrations(vec!["test response".to_string()]);
let recorder = Arc::new(LlmCallRecorder::new(
    Arc::new(mock_backend),
    Arc::new(mock_forensics), // or test impl of LlmMessageRepository
));

let service = GameService::with_backends(recorder, agent_registry);
let ctx = GameServiceContext::new(/* ... */);
service.execute_action(ctx, "look".to_string());
```

**Note:** `ActionPipeline` is non-generic and holds direct fields (`prompt_assembler`, `llm_recorder`, `agent_registry`). `run_post_generation_agents` is an inline phase method of `ActionPipeline`. See `tests/integration/pipeline/pipeline_tests.rs` for working examples.

### Config File (`tests/test_config.json`)

For integration tests that need environment-specific overrides.

### Mock Settings File

Browser tests write a temporary `settings.json` with Mock connections and pass it via `--settings-path` CLI flag when spawning the server process.

## Test Support Utilities

For `test_support::fixtures` and `TestAppBuilder` API, see [`test_support.md`](test_support.md).

## Running Tests

### Full Suite

```bash
# Default: fast suite (LLM tests excluded)
cargo nextest run
python build.py

# Include slow LLM tests
cargo nextest run --run-ignored only
python build.py --include-llm

# Run only LLM tests
cargo nextest run --test llm --run-ignored only
python build.py --llm-only
```

### Fast Iteration

Run individual test binaries directly (bypasses fmt, clippy, guardrails, and full suite):

| Command | Use When |
|---------|----------|
| `cargo nextest run --test integration` | Integration test changes |
| `cargo nextest run --test http` | HTTP endpoint changes |
| `cargo nextest run --test browser` | UI changes |
| `cargo nextest run --test llm` | LLM smoke tests |
| `cargo nextest run --test guardrails` | Guardrail changes |
| `cargo nextest run --test architecture` | Architecture changes |

## UI Tests

UI tests use **playwright-rs** for browser automation. Requires Node.js 18+ and Chromium installed via `npx playwright install chromium`.

### Running with Visible Browser

```bash
$env:HEADED = "1"; cargo nextest run --test browser test_page_loads
```

### Diagnostics on Failure

- Screenshots: `chronicler_engine/tmp/screenshots/`
- DOM dumps: `chronicler_engine/tmp/test_diagnostics/`

## Code Coverage

Coverage is verified using `cargo-llvm-cov`:

```bash
cargo +stable install cargo-llvm-cov --locked
cargo llvm-cov test --json --output-path coverage.json
```

See `docs/adr/` for detailed rationale behind testing patterns.

## What We Keep

Critical tests that must not be removed:

| Test | Why |
|------|-----|
| `test_list_games_fragment_escapes_html` | XSS security — template escaping |
| `test_action_handler_empty_command` | Empty input triggers continuation (SillyTavern "Continue" behavior) |
| `test_story_log_scrollable` | Functional — can't scroll history |
| `test_no_horizontal_overflow` | Regression — breaks page layout |

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
  "port_range": {"min": 3010, "max": 3050},
  "default_backend": "mock"
}
```
