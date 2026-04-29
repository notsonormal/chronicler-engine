# Plan: Fix Test Suite Issues

**Created:** 2026-04-30
**Status:** ✅ IMPLEMENTED

## Problem Summary

| Issue | Severity | Before | After |
|-------|----------|--------|-------|
| `game_service.rs` coverage | **CRITICAL** | 1% | 61% |
| Duplicated helpers | Medium | 6 instances | 0 |
| Arbitrary sleeps | Medium | 6 instances | 0 |
| Logic navigation untested | Medium | ~63% | ~80%+ |
| **Overall coverage** | - | 77.2% | **81.1%** |

## Phases Completed

### Phase 1: Consolidate Helper Functions ✅
- Added `launch_chrome`, `send_action`, `get_status`, `count_log_entries`, `wait_for_log_entries`, `wait_for_element_exists`, `wait_for_element_not_exists` to `test_utils.rs`
- Removed duplicates from `trigger_tests.rs` and `flow_llm_tests.rs`

### Phase 2: Add GameService Unit Tests ✅
- Created `tests/game_service_tests.rs` with 6 unit tests
- Coverage for `game_service.rs` improved from 1% to 61%

### Phase 3: Parser Tests
- Parser tests were already comprehensive (short commands, case insensitivity covered)

### Phase 4: Add Logic Navigation Tests ✅
- Created `tests/logic_tests.rs` with 11 navigation tests
- Coverage for `logic.rs` improved significantly

### Phase 5: Replace Arbitrary Sleeps ✅
- Replaced 6 instances of `tokio::time::sleep(100ms)` with smart waiting functions

## Files Modified

| File | Action |
|------|--------|
| `tests/test_utils.rs` | Added 7 helper functions |
| `tests/game_service_tests.rs` | Created - 6 unit tests |
| `tests/logic_tests.rs` | Created - 11 navigation tests |
| `tests/trigger_tests.rs` | Removed duplicate helpers |
| `tests/flow_llm_tests.rs` | Removed duplicate helpers |
| `tests/e2e_tests.rs` | Replaced arbitrary sleeps with smart waits |

## Test Results

All 75 tests pass:
- component_tests: 25 ✅
- e2e_tests: 25 ✅
- trigger_tests: 6 ✅
- flow_mock_tests: 4 ✅
- flow_llm_tests: 3 ✅
- game_service_tests: 6 ✅ (new)
- logic_tests: 11 ✅ (new)