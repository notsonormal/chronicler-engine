# Server Module Refactor

## Goal
Extract business logic from `chronicler_engine/src/server/mod.rs` (368 lines) into focused modules, leaving only module declarations and re-exports.

## Status: ✅ COMPLETE (Cleanup Required)

All acceptance criteria met. Refactor successfully completed on 2026-05-31.

**NOTE**: Clippy warnings must be fixed before merge.

## Cleanup Required (Before Merge)

### Clippy Warnings to Fix

1. **`src/server/mod.rs`** - Add `#[cfg(test)]` to `mod mod_tests;` line 21
2. **`src/bootstrap/run_tests.rs`** - Fix unused imports (lines 1, 2, 200, 284, 285)
3. **`src/storage/backend/core_tests.rs`** - Remove unused `Swipe` import (line 153)
4. **`src/storage/backend/messages_tests.rs`** - Fix duplicate `#[test]` attribute (line 272), prefix unused `_id` (line 69)
5. **`src/storage/backend/snapshots_tests.rs`** - Remove unused `NarrativeState` import (line 9), prefix unused `_id1` (line 160)
6. **`src/server/mod.rs`** - Rename `server.rs` module to `server_impl.rs` to avoid `clippy::module_inception` warning

### Clippy Command
```bash
cd chronicler_engine && cargo clippy --all-targets --all-features -- -D warnings
```

### Documentation Sync Required

Before merging, update:
- [ ] `docs/architecture/system.md` - Server tier section (already accurate)
- [ ] `CHANGELOG.md` - Record the refactor
- [ ] Archive this plan to `docs/plans/archived/server-module-refactor-2026-05-31.md`

## Test Coverage Analysis

### Existing Tests (Pre-Refactor)
The following test coverage existed in `mod.rs` and continues to work after extraction:

#### `ServerConfig` (app_state.rs) - HIGH COVERAGE ✅
- `test_server_config_default` - Default port 3000
- `test_server_config_custom_port` - Custom port 8080
- `test_server_config_default_is_consistent` - Consistency check
- `test_server_config_clone` - Clone behavior
- `test_server_config_debug` - Debug formatting
- `test_server_config_min_port` - Port 1
- `test_server_config_max_port` - Port 65535

#### `AppState` (app_state.rs) - MEDIUM COVERAGE ⚠️
- `test_app_state_struct_fields` - Struct field verification
- `test_game_service_trait_bounds` - Send + Sync bounds
- Used extensively in fragment tests via `make_test_app_state()` helper
- Missing: Direct tests for `as_game_service_context()`, `current_cancel_token()`, `replace_cancel_token()`, `settings()` methods

#### `build_router()` (router.rs) - HIGH COVERAGE ✅
- Used in `test_app_builder.rs` for test harness construction
- All fragment handler tests exercise router indirectly (100+ tests)
- Routes verified through integration testing

#### `run_server_with_config()` (server.rs) - LOW COVERAGE ⚠️
- No direct unit tests
- Only called from `bootstrap/run.rs` main entry point
- **Recommendation**: Add integration test for server lifecycle and shutdown

#### Port Utilities (port_utils.rs) - NO COVERAGE ❌
- `bind_with_retry()` - No tests
- `find_process_on_port()` - No tests (Windows-specific)
- `kill_process()` - No tests (Windows-specific)
- **Recommendation**: Low priority - OS-specific, hard to test portably

#### `index_handler()` (handlers.rs) - NO COVERAGE ❌
- No existing tests
- **Recommendation**: Add test for content type and file resolution

### Test Usage Map

| File | Test Users | Coverage Level |
|------|-----------|----------------|
| `app_state.rs` | `mod_tests.rs`, `fragments_tests.rs`, all fragment tests | High |
| `router.rs` | `test_app_builder.rs`, all integration tests | High |
| `server.rs` | `bootstrap/run.rs` only | Low |
| `handlers.rs` | None | None |
| `port_utils.rs` | None | None |

### Test Files Using Extracted Code

```
src/server/mod_tests.rs                - ServerConfig tests
src/server/fragments_tests.rs          - Uses AppState via make_test_app_state()
src/server/prompt_presets_fragment/handlers_tests.rs - Uses AppState
src/test_support/test_app_builder.rs   - Uses build_router()
tests/poison_recovery.rs               - Uses AppState directly
```

### Test Gaps and Recommendations

#### High Priority
1. **`index_handler()` test** (`handlers_tests.rs`)
   ```rust
   #[tokio::test]
   fn test_index_handler_returns_html_with_correct_content_type()
   ```

2. **`AppState` method tests** (`app_state_tests.rs`)
   ```rust
   fn test_as_game_service_context_clones_correctly()
   fn test_current_cancel_token_handles_poisoned_lock()
   fn test_replace_cancel_token_creates_new_token()
   fn test_settings_handles_poisoned_lock()
   ```

#### Medium Priority
3. **Server lifecycle test** (`server_tests.rs`)
   ```rust
   #[tokio::test]
   async fn test_run_server_with_config_startup_and_shutdown()
   ```

#### Low Priority (Windows-specific, hard to test portably)
4. Port utilities - Consider skipping or using mocking framework

## Implementation Summary

### Files Created
1. `port_utils.rs` - 1993 bytes (port management)
2. `handlers.rs` - 177 bytes (static file handler)
3. `app_state.rs` - 3361 bytes (state structs)
4. `server.rs` - 2624 bytes (server lifecycle)
5. `router.rs` - 5444 bytes (route definitions)
6. `mod.rs` - 641 bytes, 29 lines (declarations only)

### Verification Results
- ✅ `cargo check` - clean
- ✅ Architecture tests - clean (storage import violation fixed)
- ✅ Server tests - 130 passed
- ✅ Bootstrap tests - 39 passed
- ✅ Public API preserved via re-exports

### Architecture Compliance
- Used `crate::storage::Storage` full paths (not imports) to comply with arch-lint rule: "Server layer must not import from storage directly"
- Server layer depends on Application layer, not Storage layer
- All re-exports in `mod.rs` maintain stable public API

## Original Plan Details

### Problem
Original `server/mod.rs` had 368 lines mixing module declarations with business logic.

### Solution
Extracted into 6 focused modules with single responsibilities.

### New Structure
```
chronicler_engine/src/server/
├── mod.rs              # 29 lines - Declarations + re-exports only
├── router.rs           # build_router(), create_app_with_state()
├── app_state.rs        # ServerConfig, ServerResources, AppState
├── server.rs           # run_server_with_config()
├── handlers.rs         # index_handler()
└── port_utils.rs       # bind_with_retry(), port utilities
```

### Critical Files Modified
1. `chronicler_engine/src/server/mod.rs` - stripped to declarations
2. `chronicler_engine/src/server/router.rs` - NEW
3. `chronicler_engine/src/server/app_state.rs` - NEW
4. `chronicler_engine/src/server/server.rs` - NEW
5. `chronicler_engine/src/server/handlers.rs` - NEW
6. `chronicler_engine/src/server/port_utils.rs` - NEW

### Migration Strategy (Completed)
1. ✅ Extracted in dependency order (bottom-up)
2. ✅ Preserved exact function signatures
3. ✅ Updated imports using full paths for Storage to comply with architecture
4. ✅ Visibility: `build_router()` is `pub(crate)`, `AppState` is `pub`

## Trade-offs

### Why This Granularity?
- `router.rs` (140 lines) - Route definitions are cohesive
- `app_state.rs` (80 lines) - State structs are cohesive
- `server.rs` (60 lines) - Lifecycle logic is cohesive
- Each file answers: "How does X work?"

### Why NOT Coarser Splits?
Could combine `handlers.rs` + `port_utils.rs` → `utils.rs`, but:
- Handlers are HTTP-specific (different concern)
- Port utils are OS-specific (Windows only)
- Separation makes future testing easier

### Why Keep Re-exports?
- `server/` is a **public API boundary**
- Callers use `crate::server::AppState`, not `crate::server::app_state::AppState`
- Hides internal file structure from rest of crate

## Next Steps (Optional)

If test coverage improvement is desired:
1. Add `handlers_tests.rs` with `index_handler()` test
2. Add `app_state_tests.rs` for AppState methods
3. Consider integration test for server lifecycle
4. Port utils tests can be skipped (OS-specific, low value)
