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
pub trait LlmBackend {
    fn generate_dialogue(&self, world: &WorldCard, room: &Room, npc: &NpcCard, user_message: &Option<String>) -> String;
    fn narrate_action(&self, world: &WorldCard, room: &Room, nearby_npcs: &[&NpcCard], player: &PlayerCard, player_input: &str) -> String;
}
```rust

The engine will provide multiple implementations of this trait:
- `OpenRouterBackend`: Used by the live executable. Contacts the HTTP API using `reqwest` and parses the JSON response.
- `MockBackend`: Used in test scenarios. Returns static mock responses immediately.
- `DeepSeekBackend`: Placeholder for future implementation.

### Backend Selection
The LLM backend can be selected in two ways:

1. **Config File** (`tests/test_config.json`): Priority method
   ```json
   {
     "port_range": {"min": 3010, "max": 3030},
     "default_backend": "mock",
     "test_specific": {
       "flow_llm_tests": {"backend": "real"}
     }
   }
   ```
2. **Environment Variable**: Fallback for backward compatibility
   - `LLM_BACKEND=mock` - Uses MockBackend for fast, no-network tests
   - `LLM_BACKEND=real` (or unset) - Uses OpenRouterBackend for real LLM responses

The `get_llm_backend()` function in `src/narrative/llm.rs` reads the environment variable for backward compatibility.

## UI Tests

UI tests use **playwright-rs** (Rust bindings for Microsoft Playwright) for browser automation testing.

### Running UI Tests

```bash
# Install Playwright browsers first (requires Node.js)
npx playwright install chromium

# Run all tests
cargo test

# Run specific test suites
cargo test --test flow_mock_tests     # Fast tests with mock LLM
cargo test --test flow_llm_tests      # Tests requiring real LLM
cargo test --test behavior_tests       # UI behavior tests
cargo test --test layout_tests         # CSS/layout tests

# Run with single thread (recommended for tests sharing ports)
cargo test -- --test-threads=1
```

### Test Requirements
- Node.js 18+ (for Playwright browser installation)
- Chromium browser installed via `npx playwright install chromium`

### Test Files

| Test File | Purpose | LLM Backend | Port Allocation |
|----------|---------|-------------|-----------------|
| `flow_mock_tests.rs` | Core game loop, polling, real-time updates | Mock (config) | Dynamic (3010-3030) |
| `flow_llm_tests.rs` | LLM narrative generation | Real (config) | Dynamic |
| `behavior_tests.rs` | Form submission, button re-enable, UI behavior | Mock (config) | Dynamic |
| `layout_tests.rs` | CSS, element sizing, scrolling | Mock (config) | Dynamic |
| `spec_tests.rs` | Page structure, HTMX, element presence | Mock (config) | Dynamic |
| `ui_tests.rs` | HTMX, WebSocket, connection status | Mock (config) | Dynamic |

**Dynamic Port Allocation**: Tests now use `tests/test_config.json` to allocate ports dynamically from range 3010-3030, eliminating port conflicts between test files.

### Test Coverage

**flow_mock_tests.rs** (8 tests):
- Initial load (header, story-log, status)
- Connection status indicator
- Command submission (look, free action)
- Polling mechanism (updates without reload)
- Message ordering (new messages at bottom)

**flow_llm_tests.rs** (4 tests):
- LLM narration appears via polling
- LLM error handling
- LLM arrival narration
- LLM free action narration

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
- **API client code** (`OpenRouterBackend`) - requires API key
- **`add_log` overflow** - requires >1000 log entries

See `docs/adr/` for detailed rationale behind these patterns.