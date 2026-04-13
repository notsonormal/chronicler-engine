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
```

The engine will provide multiple implementations of this trait:
- `OpenRouterBackend`: Used by the live executable. Contacts the HTTP API using `reqwest` and parses the JSON response.
- `MockBackend`: Used in test scenarios. Returns static mock responses immediately.
- `DeepSeekBackend`: Placeholder for future implementation.

### Backend Selection
The LLM backend is selected at runtime via the `LLM_BACKEND` environment variable:
- `LLM_BACKEND=mock` - Uses MockBackend for fast, no-network tests
- `LLM_BACKEND=openrouter` (default) - Uses OpenRouterBackend for real LLM responses
- `LLM_BACKEND=deepseek` - Uses DeepSeekBackend (not yet implemented)

The `get_llm_backend()` function in `src/narrative/llm.rs` reads this environment variable.

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

| Test File | Purpose | LLM Backend |
|-----------|---------|-------------|
| `flow_mock_tests.rs` | Core game loop, polling, real-time updates | Mock |
| `flow_llm_tests.rs` | LLM narrative generation | Real (OpenRouter) |
| `behavior_tests.rs` | Form submission, WebSocket, UI behavior | Real |
| `layout_tests.rs` | CSS, element sizing, scrolling | Real |
| `spec_tests.rs` | Page structure, HTMX, element presence | Real |

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

1. **Sequential Test Execution**: Some tests share ports and must run sequentially (`--test-threads=1`)
2. **Headless Browser**: WebSocket connections may be unreliable in headless mode; polling provides fallback
3. **LLM Response Time**: Real LLM tests may be slow; use mock tests for fast iteration

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

See `docs/adr/` for detailed rationale behind these patterns.