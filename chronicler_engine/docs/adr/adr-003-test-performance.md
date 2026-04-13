# ADR-003: Test Performance Optimization Strategy

## Date
2026-04-13

## Status
Accepted

## Context
Initial test suite analysis showed excessive sleep delays causing slow test execution (~5 minutes for full suite).

## Problem Analysis

### Observed Times (Before)
| Test Type | Duration |
|-----------|----------|
| `flow_mock_tests.rs` (8 tests) | ~75s |
| `flow_llm_tests.rs` (4 tests) | ~100s |

### Root Causes
1. **Fixed LLM waits**: 15 second sleeps regardless of actual LLM response time
2. **Fixed polling waits**: 5 second sleeps for 5 second polling interval
3. **Slow server startup**: 100ms sleep * 50 retries = 5s per test
4. **No condition-based waiting**: Tests waited for max time instead of checking for completion

## Implemented Optimizations

### 1. Smart LLM Waiting (ADR-002)
- Use `wait_for_llm_idle()` to poll `/status/generating` endpoint
- Average wait: 0.5-2s (mock) vs fixed 15s
- **Savings**: ~13s per LLM test

### 2. Server Startup Faster
- Reduced retry interval: 100ms → 50ms
- Reduced max retries: 50 → 30
- **Savings**: ~4s total across all tests

### 3. Condition-Based UI Polling
Instead of fixed sleep after LLM completes:
```rust
// OLD: Fixed 3 second wait
sleep(Duration::from_millis(3000)).await;
let result = page.evaluate(...);

// NEW: Poll until condition met
let mut result = initial.clone();
for _ in 0..10 {
    result = page.evaluate(...).await;
    if result != initial { break; }
    sleep(Duration::from_millis(500)).await;
}
```
- **Savings**: 1-4s per test (stops as soon as condition met)

### 4. Reduced Initial Load Wait
- Mock backend is instant, so reduced from 3s to 2s
- **Savings**: ~1s per test

## Results (After)

| Test Type | Before | After | Improvement |
|-----------|--------|-------|-------------|
| `flow_mock_tests.rs` (8 tests) | ~75s | ~67s | ~11% |
| `flow_llm_tests.rs` (4 tests) | ~100s | ~115s* | +15% |

*LLM tests now wait properly for LLM instead of fixed 15s (actual LLM time ~20-30s)

## Key Learnings

1. **Poll for conditions, don't sleep**: Always check for expected state rather than waiting fixed durations
2. **Smart waiting > fixed waits**: `wait_for_llm_idle()` is faster than 15s fixed because it detects completion immediately
3. **Balance speed and reliability**: Too aggressive polling can cause flaky tests; 500ms interval works well

## Related ADRs
- [ADR-001](adr-001-polling-for-realtime-updates.md) - Polling for real-time updates
- [ADR-002](adr-002-llm-backend-selection.md) - LLM backend selection