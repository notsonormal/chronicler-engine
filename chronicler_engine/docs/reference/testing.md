# Specification: Testing Strategy and Architecture

## Objective
Establish a formal policy and architectural design pattern for ensuring the Chronicler Engine remains heavily tested locally without incurring financial costs or massive latency from interacting with external LLM APIs (like OpenRouter).

## Policy Rules
1. **Isolated Unit Tests**: All modules (`parser`, `map`, `state`) must continue to maintain fully isolated, embedded unit tests `#[test]` that evaluate standard library behaviors with zero networking overhead.
2. **Integration Capabilities**: As the engine develops, an overarching `tests/` directory will be required. These integration tests will evaluate the state graph moving from end-to-end.
3. **LLM Abstraction (The Trait Pattern)**: No component outside of the executable `main.rs` loop should ever be hardcoded to contact OpenRouter.

## The `LlmBackend` Interface
To satisfy the LLM Abstraction policy, `llm.rs` must implement an interface:
```rust
pub trait LlmBackend: Send + Sync {
    fn generate_dialogue(&self, context: &PromptContext, npc: &NpcCard) -> Result<String, EngineError>;
    fn narrate_action(&self, context: &PromptContext) -> Result<String, EngineError>;
    fn narrate_arrival(&self, context: &PromptContext) -> Result<String, EngineError>;
    fn narrate_continuation(&self, system_prompt: &str, user_prompt: &str, trigger_prompt: &str) -> Result<String, EngineError>;
    fn narrate_action_from_prompt(&self, system_prompt: &str, user_prompt: &str) -> Result<String, EngineError>;
    fn name(&self) -> &str;
}
```

The engine will provide multiple implementations of this trait:
- `OpenRouterBackend`: Used by the live executable. Contacts the HTTP API using `reqwest` and parses the JSON response.
- `MockBackend`: Used in test scenarios. Returns static mock responses immediately.
- `DeepSeekBackend`: Placeholder for future implementation.

### Backend Selection
The LLM backend can be selected in three ways:

1. **Test Override** (Recommended for unit tests): Atomic override with RAII guard
   ```rust
   use chronicler_engine::narrative::llm::{with_test_backend, LlmBackendType};

   #[test]
   fn test_with_mock_llm() {
       let _guard = with_test_backend(LlmBackendType::Mock);
       // Test code here uses MockBackend regardless of settings file
   } // override automatically cleared on drop
   ```
   - Thread-safe (atomic operations)
   - No file I/O required
   - Auto-cleanup via RAII guard prevents cross-test pollution
   - Works in both unit tests and integration tests

2. **Config File** (`tests/test_config.json`): For integration tests
   ```json
   {
     "port_range": {"min": 3010, "max": 3030},
     "default_backend": "mock",
     "test_specific": {
       "flow_llm_tests": {"backend": "real"}
     }
   }
   ```
3. **Mock Settings File**: Integration tests write a temporary settings file with Mock connections and set `CHRONICLER_SETTINGS_PATH` to point to it. This is the preferred approach for subprocess-based tests.

The `get_llm_backend()` function checks the test override first, then falls back to the narration connection from `settings.json`.

## UI Tests

UI tests use **playwright-rs** (Rust bindings for Microsoft Playwright) for browser automation testing.

### Running UI Tests

```bash
# Install Playwright browsers first (requires Node.js)
npx playwright install chromium

# Run all tests
cargo test

# Run specific test suites
cargo test --test component_tests   # In-process tests (fast)
cargo test --test e2e_tests         # Browser tests
cargo test --test flow_mock_tests   # Fast tests with mock LLM
cargo test --test flow_llm_tests     # Tests requiring real LLM

# Run with single thread (recommended for tests sharing ports)
cargo test -- --test-threads=1
```

### Test Requirements
- Node.js 18+ (for Playwright browser installation)
- Chromium browser installed via `npx playwright install chromium`

### Test Files (Consolidated)

| Test File | Purpose | Execution Model | Runtime |
|----------|---------|---------------|---------|
| `flow_mock_tests.rs` | Core game loop, polling | Browser + Mock LLM | Fast |
| `flow_llm_tests.rs` | LLM narrative | Browser + Real LLM | Slow |
| `component_tests.rs` | Templates, endpoints, settings | In-process | Very Fast |
| `e2e_tests.rs` | UI structure, layouts | Browser | Medium |
| `trigger_tests.rs` | Trigger evaluation and firing | Browser + Mock LLM | Fast |
| `game_service_tests.rs` | Game service logic | In-process | Very Fast |
| `architecture.rs` | Architecture guardrails | In-process | Very Fast |

### Test Coverage

**component_tests.rs** (52 tests):
- XSS security (template escaping)
- Template rendering
- HTTP endpoint responses (game fragments)
- Status endpoint phase responses (`narrating`, `quantifying`)
- Validation (empty command rejection)
- Settings UI integration (16+ tests)

**e2e_tests.rs** (21 tests):
- Page loads, UI structure
- Action area elements
- Story log functionality
- Layout positioning
- Visual sidebar
- Edit mode and retry functionality

**flow_mock_tests.rs** (4 tests):
- Initial load (header, story-log, status)
- Connection status indicator
- Command submission
- Polling mechanism

**flow_llm_tests.rs** (3 tests):
- LLM narration via polling
- LLM arrival narration
- LLM free action narration

**trigger_tests.rs** (6 tests):
- Trigger evaluation and firing
- Non-repeatable trigger behavior
- Multiple trigger handling

### Known Limitations

1. **Headless Browser**: HTMX polling is reliable in headless mode; 5-second delay is acceptable for tests
2. **LLM Response Time**: Real LLM tests may be slow; use mock tests for fast iteration

**Port Conflicts Solved**: Dynamic port allocation (3010-3030) eliminates the need for sequential test execution (`--test-threads=1`).

### Smart Waiting Patterns

Instead of using fixed sleep durations, tests should use smart waiting:

1. **For LLM completion**: Use `wait_for_llm_idle(port, timeout)` which polls `/status/generating` until the LLM finishes
2. **For UI elements**: Poll the DOM for expected elements/content instead of fixed delays
3. **Avoid bare sleeps**: Never use `sleep(Duration::from_millis(X))` without a documented reason

Example:
```rust
// BAD: Fixed 15 second wait
sleep(Duration::from_millis(15000)).await;

// GOOD: Wait for LLM to complete
let llm_result = wait_for_llm_idle(TEST_PORT, Duration::from_secs(30)).await;
if llm_result.is_err() {
    println!("Warning: LLM did not become idle within timeout");
}
sleep(Duration::from_millis(1000)).await; // Brief wait for poll to catch update
```

### Settings Integration Tests

Settings UI tests use in-process HTTP testing via `tower::ServiceExt::oneshot`. No browser needed.

#### Test Infrastructure

The `create_app_for_testing()` helper in `src/server/mod.rs` creates a test router with:
- All game fragment routes (`/fragment/header`, `/fragment/story-log`, etc.)
- Settings routes (`/fragment/settings`, `/settings`)
- In-memory `AppState` with default settings

#### Patterns

```rust
#[tokio::test]
async fn test_settings_panel_returns_html() {
    let state = create_test_state();
    let app = create_app_for_testing(state);

    let req = Request::builder()
        .uri("/fragment/settings")
        .body(Body::empty())
        .unwrap();
    let response = app.oneshot(req).await.unwrap();

    assert!(response.status().is_success());
    let body = axum::body::to_bytes(response.into_body(), 1024).await.unwrap();
    let body_str = String::from_utf8_lossy(&body);
    assert!(body_str.contains("LLM Settings"));
}

#[tokio::test]
async fn test_save_settings_updates_memory() {
    let state = create_test_state();
    let app = create_app_for_testing(state);

    let req = Request::builder()
        .uri("/settings")
        .method(http::Method::POST)
        .header(http::header::CONTENT_TYPE, "application/x-www-form-urlencoded")
        .body(Body::from("narration_connection_id=openrouter-gpt-4o-mini&quantifier_connection_id=openrouter-gpt-4o-mini"))
        .unwrap();
    let response = app.oneshot(req).await.unwrap();

    assert!(response.status().is_success());
}
```

#### Routes Tested
- GET `/fragment/settings` - Settings panel HTML fragment
- POST `/settings` - Save active connection selections (narration_connection_id, quantifier_connection_id)
- POST `/connections/add` - Add a new connection profile

## Code Coverage

Coverage is verified using `cargo-llvm-cov`. This tool instruments the code and reports which lines are executed during tests.

### Running Coverage

```bash
# Install cargo-llvm-cov (once)
cargo +stable install cargo-llvm-cov --locked

# Run coverage (after tests pass)
cargo llvm-cov test --json --output-path coverage.json
```

### Coverage Thresholds

| Module Type | Minimum Coverage |
|------------|------------------|
| Core logic (`engine/logic.rs`) | 90% |
| Parser (`engine/parser.rs`) | 85% |
| Data models (`model/*.rs`) | 70% |
| LLM prompts (`narrative/llm.rs`) | 50% |
| Server HTTP (`server/*.rs`) | N/A (integration only) |

### Known Gaps (Acceptable)

These have low coverage but are acceptable:
- **Runtime env detection** (`LlmBackendType::from_env()`) - cannot be unit tested
- **API client code** (`OpenRouterBackend`) - LLM requests are slow
- **`add_log` overflow** - requires >1000 log entries

See `docs/adr/` for detailed rationale behind these patterns.