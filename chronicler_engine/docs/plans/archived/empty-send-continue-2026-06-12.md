# Plan: Empty Send Triggers Narrative Continuation

**Status:** ✅ COMPLETED  
**Date:** 2026-06-12  
**Author:** Agent (user-approved)

## Problem

When the user presses Send with an empty text box, the engine should generate a new narrator message that continues the story without additional player input — the same behavior as SillyTavern's "Continue" button. Currently, empty input is rejected at three layers (HTML5 `required minlength="1"`, three backend guards returning "Enter a command").

## Solution

Implement narrative continuation for empty input by:
1. Adding `run_from_continue()` method to `ActionPipeline` that passes `[Continue]` sentinel
2. Adding `execute_continue_impl()` function in action pipeline module
3. Adding `continue_narration()` method to `DefaultApplicationService`
4. Routing empty input to continuation instead of rejecting
5. Removing HTML5 validation blocking empty submit
6. Updating tests and documentation

## Files Changed

### Core Implementation
- `src/application/action_pipeline/pipeline.rs` — Added `run_from_continue()` method
- `src/application/action_pipeline/actions.rs` — Added `execute_continue_impl()` function
- `src/application/action_pipeline/mod.rs` — Exported `execute_continue_impl`
- `src/application/application_service.rs` — Added `continue_narration()` method
- `src/server/fragments/actions.rs` — Replaced empty guards with continuation routing
- `assets/index.html` — Removed `required minlength="1"` from input

### Tests
- `src/server/fragments/actions_tests.rs` — Updated tests to expect OK, added response text verification
- `tests/infrastructure/guardrails/structure.rs` — Fixed pre-existing dead code warning

### Documentation
- `docs/CHANGELOG.md` — Added 2026-06-12 entry
- `docs/system/game_flow.md` — Documented empty input behavior
- `docs/system/dashboard.md` — Documented "Continuing..." status and empty input behavior

## Verification

1. ✅ All 885 tests pass
2. ✅ No clippy warnings
3. ✅ Server starts successfully on port 3000
4. ✅ Empty input now triggers narrative continuation
5. ✅ Response text contains "Continuing..." for empty input
6. ✅ No AI slop or hacks introduced

## Coverage Impact

- Core flow changes maintain existing coverage levels
- `actions.rs`: 45.5% (110/242 lines)
- `application_service.rs`: 58.2% (343/589 lines)
- `pipeline.rs`: 62.2% (768/1235 lines)
- Overall: 60.1% (within historical norms for this codebase)

## Design Notes

- Uses `[Continue]` sentinel that flows through existing prompt layers
- Does NOT add Input message to history (clean UX)
- Parallel structure to `process_action()` for maintainability
- No architectural changes — follows established patterns
