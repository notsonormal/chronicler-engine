# Testing Policy

## Unit Tests

`*_tests.rs` sibling files, not inline `#[cfg(test)]`. Enforced by [`scripts/check_test_structure.py`](../../scripts/check_test_structure.py).

## Integration Tests

Cross-module and end-to-end tests live in `tests/`, organised by fixture weight per binary and within each binary mirroring `src/` paths. See [`tests/AGENTS.md`](../../tests/AGENTS.md) for the live structure index and `TEST MIRROR CONVENTION` for layout rules.

## LLM Abstraction

No component outside `main.rs` hardcodes an external LLM API. Use `MockBackend` (implements `LlmProvider`) and the `make_test_recorder` / `make_test_recorder_with_storage` helpers at [`src/test_support/noop_forensics.rs`](../../src/test_support/noop_forensics.rs) to wrap `MockBackend` in `LlmCallRecorder` for `with_backends` / `with_mock_quantifier`.

## Backend Selection

```rust
let service = GameService::with_mock_quantifier(
    make_test_recorder(Arc::new(MockBackend::new())),
    Arc::new(MockBackend::new()),
);

let app = make_test_app_with_storage_and_service(storage, Arc::new(service))?;
app.process_action("look around".to_string());
```

Factory selection: `make_test_app(state)` (default in-memory + mock backend), `make_test_app_with_sqlite(state)`, `make_test_app_with_mock_backend(state, factory)`, `make_test_app_with_backends(state, narrator)`, `make_test_app_with_separate_backends(state, n, q)`, `make_test_app_with_game_service(state, |storage| { ... })`, `make_test_app_with_storage_and_service(storage, gs)`. All `Result`-returning factories must be `?`'d (lib clippy denies unwrap/expect/panic). For tests that need to invoke pipeline phases through a `GameService`, build the `DefaultApplicationService` via `make_test_app_with_storage_and_service` and call `app.process_action(input)` (the `DefaultApplicationService::process_action` entry point).

`ActionPipeline` is non-generic; `run_post_generation_agents` is an inline phase method. See `tests/integration/application/action_pipeline/pipeline.rs` for working examples.

**Do not** call `execute_action_impl()` directly — use the public `execute_action()` wrapper.

## Critical Tests

These categories of test must never be removed without replacement:

| Category | Why |
|----------|-----|
| HTML escaping for any fragment interpolating user input | XSS |
| Empty / whitespace-only command handling | SillyTavern "Continue" continuation |
| Scroll behaviour of the story log | Functional regression |
| Horizontal-overflow sanity on rendered pages | Layout regression |

## Running Tests

```bash
python build.py                     # Default: fast suite
python build.py --llm-only          # Include real LLM tests
cargo nextest run --test <binary>   # Iterate on one suite
```

## UI Tests

Playwright via `playwright-rs`. Requires Node 18+ and `npx playwright install chromium`.

```bash
HEADED=1 cargo nextest run --test browser <test_name>
```

Diagnostics on failure: `chronicler_engine/tmp/screenshots/` (PNG) and `tmp/test_diagnostics/` (DOM dumps).

## Smart Waiting

Polling, not `sleep`. Helpers at `tests/test_utils/wait.rs`: `wait_for_llm_idle`, `wait_for_status_ready`, `wait_for_element_children`.

## Coverage

`cargo +stable install cargo-llvm-cov --locked`, then `cargo llvm-cov test --json --output-path coverage.json`.