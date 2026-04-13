# ADR-002: Runtime LLM Backend Selection via Environment Variable

## Date
2026-04-13

## Status
Accepted

## Context
The engine needs to support both real LLM calls (for production) and mock responses (for testing) without code changes.

## Decision
The LLM backend is selected at runtime via the `LLM_BACKEND` environment variable.

## Implementation

### Configuration
```bash
# Use mock backend (fast, no network)
LLM_BACKEND=mock cargo test

# Use real OpenRouter (default)
LLM_BACKEND=openrouter cargo run

# Use DeepSeek (future)
LLM_BACKEND=deepseek cargo run
```

### Code
```rust
// src/narrative/llm.rs
pub enum LlmBackendType {
    OpenRouter,
    Mock,
    DeepSeek,
}

pub fn get_llm_backend() -> Box<dyn LlmBackend> {
    match std::env::var("LLM_BACKEND").as_deref() {
        Ok("mock") => Box::new(MockBackend),
        Ok("deepseek") => Box::new(DeepSeekBackend),
        _ => Box::new(OpenRouterBackend::new()),
    }
}
```

## Reasons

1. **Test Speed**: Mock backend returns immediately (no API calls)
2. **No API Costs**: Tests don't consume OpenRouter credits
3. **Separation**: Test environment doesn't need network access
4. **Flexibility**: Easy to add new backends (DeepSeek, Anthropic, etc.)

## Consequences

### Positive
- Fast test execution (~15s vs ~100s with real LLM)
- Zero API costs for testing
- Clear separation between test and production behavior

### Negative
- Tests don't verify actual LLM behavior
- Need separate integration tests for LLM verification

## Test Files
- `flow_mock_tests.rs` (port 3006) - Uses mock backend, 8 tests, ~15s
- `flow_llm_tests.rs` (port 3007) - Uses real backend, 4 tests, ~100s

## Related
- See `docs/reference/testing.md` for testing strategy
- MockBackend implemented in `src/narrative/llm.rs`