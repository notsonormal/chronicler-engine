# Plan: Server Fragment Unit Tests

## Goal
Add unit tests for 6 server fragment files (530 uncovered lines) by calling handlers directly, following the `prompt_presets_fragment/handlers_tests.rs` pattern.

## Strategy
- Each test file gets a `make_test_app_state()` helper (same pattern as `renderers_tests.rs`)
- Handlers are called directly: `handler(State(state), Form(form)).await` — no HTTP/router
- Test all branches: happy path, error paths, edge cases
- Reuse `test_support::TestWorld`, `TestPlayer`, `TestMap` fixtures

## Key Pattern
```rust
fn make_test_app_state() -> crate::server::AppState {
    let storage = Arc::new(Storage::new_in_memory());
    let settings = Arc::new(RwLock::new(AppSettings::default()));
    let game_service = Arc::new(GameService::with_storage(
        Some(Arc::clone(&storage)), None, Arc::clone(&settings),
    ));
    crate::server::AppState {
        storage, preset_storage: Arc::new(Storage::new_in_memory()),
        world: Arc::new(TestWorld::minimal()), map: Arc::new(TestMap::single_room("start")),
        player: Arc::new(TestPlayer::standard()), npcs: Arc::new(HashMap::new()),
        game_service: Arc::clone(&game_service),
        application_service: Arc::new(DefaultApplicationService::new(game_service)),
        settings, cancel_token: Arc::new(RwLock::new(CancellationToken::new())),
        is_generating: Arc::new(AtomicBool::new(false)),
    }
}

#[tokio::test]
async fn test_action_handler_ok() {
    let state = make_test_app_state();
    let form = ActionForm { command: "look".into() };
    let response = action_handler(State(state), Form(form)).await;
    assert_eq!(response.status(), StatusCode::OK);
}
```

## Implementation Summary

**Created:**
- `src/server/fragments/games_tests.rs` (128 lines, 9 tests)
- `src/server/fragments/endpoints_tests.rs` (135 lines, 13 tests)
- `src/server/fragments/misc_tests.rs` (134 lines, 8 tests)

**Expanded:**
- `src/server/fragments/actions_tests.rs` (178 lines, 15 tests total) - Added 9 handler tests
- `src/server/fragments/history_tests.rs` (92 lines, 5 tests total) - Added 1 handler test

**Modified:**
- `src/server/fragments/mod.rs` - Registered test modules

## Results
- **68 tests** for server fragments (was ~20, now 68)
- **~530 lines** of new test code across 5 files
- All 6 target files covered per plan

## Verification
✅ All 833 tests pass (68 fragment tests)
✅ Clippy: zero warnings with `-D warnings`
✅ Import ordering guardrails pass
✅ Full `python build.py` pipeline green
✅ Test structure guardrail passes

