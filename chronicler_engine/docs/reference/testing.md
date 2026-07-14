# Testing Policy

## Unit Tests

`*_tests.rs` sibling files, not inline `#[cfg(test)]`. Enforced by [`scripts/check_test_structure.py`](../../scripts/check_test_structure.py).

## Integration Tests

Cross-module and end-to-end tests live in `tests/`, organised by fixture weight per binary and within each binary mirroring `src/` paths. See [`tests/AGENTS.md`](../../tests/AGENTS.md) for the live structure index and `TEST MIRROR CONVENTION` for layout rules.

## LLM Abstraction

No component outside `main.rs` hardcodes an external LLM API. Use `MockBackend` (implements `LlmProvider`) and the `make_test_recorder` / `make_test_recorder_with_storage` helpers at [`src/test_support/noop_forensics.rs`](../../src/test_support/noop_forensics.rs) to wrap `MockBackend` in `LlmCallRecorder` for `with_backends` / `with_mock_quantifier`.

## App Construction

Three builders cover app construction across in-memory lib tests, in-memory integration tests, and sqlite-backed integration tests. All return `Arc<DefaultApplicationService>` via `build_service()`.

**Lib tests (`src/*_tests.rs`) — in-memory is sufficient; no sqlite needed:**

```rust
use chronicler_engine::test_support::{TestDataBuilder, TestAppBuilder};

let data = TestDataBuilder::default_test().build();
let app = TestAppBuilder::with_data(data)
    .game_service(Arc::new(service))
    .build_service()?;
app.process_action("look around".to_string());
```

**Integration tests needing sqlite-backed storage + custom `GameService` wiring:**

```rust
use chronicler_engine::test_support::TestDataBuilder;
use crate::helpers::SqliteTestAppBuilder;

let data = TestDataBuilder::default_test().build();
let app = SqliteTestAppBuilder::with_data(data)
    .game_service_fn(move |storage| {
        Arc::new(GameService::with_mock_quantifier(
            make_test_recorder_with_storage(Arc::new(MockBackend::new()), Arc::clone(storage)),
            Arc::new(MockBackend::default()),
        ))
    })
    .build_service()?;
app.process_action("look around".to_string());
```

Builder selection: `TestAppBuilder` (in-memory, `src/test_support/`) for lib tests + in-memory integration tests; `SqliteTestAppBuilder` (integration-only, `tests/helpers/`) for sqlite-backed integration tests. Three survivors remain in `src/test_support/context.rs` for narrow cases: `make_test_app`, `make_test_app_without_snapshot` (snapshot-skip semantics), `seed_test_world_into_storage` (failing-storage test). All `Result`-returning builders must be `?`'d (lib clippy denies unwrap/expect/panic). For tests that need to invoke pipeline phases through a `GameService`, use `.game_service(...)` (in-memory) or `.game_service_fn(|storage| { ... })` (sqlite) and call `app.process_action(input)` (the `DefaultApplicationService::process_action` entry point).

`ActionPipeline` is non-generic; `run_post_generation_agents` is an inline phase method. See `tests/integration/application/action_pipeline/pipeline.rs` for working examples.

**Do not** call `execute_action_impl()` directly — invoke through the `DefaultApplicationService` via `process_action(input)` so the cancellation gate + forensics + persistence flow runs end-to-end.

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

## Document References

- [test_support.md](./test_support.md) — `TestAppBuilder` + `SqliteTestAppBuilder` + builder API