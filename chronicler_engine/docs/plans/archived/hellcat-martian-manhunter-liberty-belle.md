# Chronicler Engine: Leaky Presentation Layer Cleanup

> **STATUS: COMPLETED** — 2026-05-24
> All 11 tasks finished. 888 tests pass. Clippy clean. Arch-lint clean. Guardrails clean.
> Build: `python build.py` passes.

## Overview
Move business logic out of Axum handlers and into a dedicated `ApplicationService`, complete the view model layer, and add architectural guardrails to prevent regression. The codebase already compiles and has partial ViewModels and existing application helpers; this plan builds on those foundations.

## Architecture Decisions
- **ApplicationService (1b)**: Create a new trait/struct for state orchestration rather than expanding the existing `GameService`. This keeps `GameService` focused on LLM backends and creates a clean seam for testing.
- **Dedicated `view_models.rs`**: Extract all view models to a single file. Template structs stay in `templates.rs` and focus purely on HTML.
- **Guardrail after refactor**: Add `arch-lint` rule banning `server` -> `storage` only after Phase 1 is complete, or the build breaks immediately.
- **Remove dead UI fields**: `cursor_position` and `scroll_offset` are unused outside unit tests; delete them rather than maintain dead code.

## Task List

### Phase 1: Application Service Firewall

- [x] **Task 1: Define ApplicationService trait and wire into AppState**
  - **Description:** Create `src/application/application_service.rs` with the `ApplicationService` trait and `DefaultApplicationService` struct. Add the service to `AppState` and update all test constructors.
  - **Acceptance criteria:**
    - [x] `ApplicationService` trait exists with method signatures for all orchestration operations.
    - [x] `DefaultApplicationService` has a constructor accepting `Arc<dyn GameService>`.
    - [x] `AppState` contains `application_service: Arc<dyn ApplicationService>`.
    - [x] All test constructors compile and pass.
  - **Verification:**
    - [x] `cargo check` passes.
    - [x] `cargo nextest run --test components` passes.
  - **Dependencies:** None
  - **Files likely touched:**
    - `src/application/application_service.rs` (new)
    - `src/application/mod.rs`
    - `src/server/mod.rs`
  - **Estimated scope:** Medium

- [x] **Task 2: Migrate action submission to ApplicationService**
  - **Description:** Implement `process_action` in `DefaultApplicationService` and refactor `server/fragments/actions.rs` handlers to use it.
  - **Acceptance criteria:**
    - [x] `process_action` loads state, adds log, sets generating flags, saves snapshot, and spawns the game service call.
    - [x] `action_handler`, `action_confirm_handler`, and `action_check_handler` only parse requests and delegate to `ApplicationService`.
  - **Verification:**
    - [x] `cargo nextest run --test components action` passes.
    - [x] `cargo nextest run --test browser` passes.
  - **Dependencies:** Task 1
  - **Files likely touched:**
    - `src/application/application_service.rs`
    - `src/server/fragments/actions.rs`
  - **Estimated scope:** Medium

- [x] **Task 3: Migrate retry/retrigger to ApplicationService**
  - **Description:** Implement `retry` and `retrigger` in `DefaultApplicationService` and refactor `server/fragments/misc.rs` handlers.
  - **Acceptance criteria:**
    - [x] `retry` and `retrigger` load state, validate preconditions, set generating flags, save snapshots, and spawn game service calls.
    - [x] `retry_handler` and `retrigger_handler` are reduced to request parsing and HTTP response mapping.
  - **Verification:**
    - [x] `cargo nextest run --test components retry` passes.
    - [x] `cargo nextest run --test components retrigger` passes.
  - **Dependencies:** Task 1
  - **Files likely touched:**
    - `src/application/application_service.rs`
    - `src/server/fragments/misc.rs`
  - **Estimated scope:** Medium

- [x] **Task 4: Migrate swipe switching to ApplicationService**
  - **Description:** Implement `switch_swipe` in `DefaultApplicationService` and refactor `server/fragments/misc.rs` handler.
  - **Acceptance criteria:**
    - [x] `switch_swipe` loads messages, validates target, updates active swipe, loads snapshot, restores state, overrides messages, and saves new snapshot.
    - [x] `switch_swipe_handler` delegates all logic to `ApplicationService`.
  - **Verification:**
    - [x] `cargo nextest run --test components swipe` passes.
  - **Dependencies:** Task 1
  - **Files likely touched:**
    - `src/application/application_service.rs`
    - `src/server/fragments/misc.rs`
  - **Estimated scope:** Small

- [x] **Task 5: Migrate history editing to ApplicationService**
  - **Description:** Implement `edit_history` and `delete_last` in `DefaultApplicationService` and refactor `server/fragments/history.rs` handlers.
  - **Acceptance criteria:**
    - [x] `edit_history` edits the message, saves snapshot, and updates DB message.
    - [x] `delete_last` deletes last message, saves snapshot, and removes DB message.
    - [x] Handlers in `history.rs` delegate all logic.
  - **Verification:**
    - [x] `cargo nextest run --test components history` passes.
  - **Dependencies:** Task 1
  - **Files likely touched:**
    - `src/application/application_service.rs`
    - `src/server/fragments/history.rs`
  - **Estimated scope:** Small

- [x] **Task 6: Migrate game lifecycle to ApplicationService**
  - **Description:** Implement `reset`, `create_game`, `switch_game`, `delete_game`, and `list_games` in `DefaultApplicationService`. Refactor `server/fragments/games.rs` and `server/fragments/misc.rs` handlers.
  - **Acceptance criteria:**
    - [x] `reset` deletes current game, creates new game, builds fresh state, persists snapshot and message, resets flags.
    - [x] `create_game` creates game, switches IDs, builds fresh state, persists, handles rollback on error.
    - [x] `switch_game` validates game exists and belongs to current world, switches IDs.
    - [x] `delete_game` validates game is not active, deletes.
    - [x] `list_games` returns games for rendering.
    - [x] All game handlers delegate logic.
  - **Verification:**
    - [x] `cargo nextest run --test components game` passes.
    - [x] `cargo nextest run --test browser` passes.
  - **Dependencies:** Task 1
  - **Files likely touched:**
    - `src/application/application_service.rs`
    - `src/server/fragments/games.rs`
    - `src/server/fragments/misc.rs`
  - **Estimated scope:** Medium

- [x] **Task 7: Migrate status endpoints to ApplicationService**
  - **Description:** Implement `get_generating_status` and `reset_generating_status` in `DefaultApplicationService`. Refactor `server/fragments/endpoints.rs` handlers.
  - **Acceptance criteria:**
    - [x] `get_generating_status` returns status/phase or defaults.
    - [x] `reset_generating_status` sets status to Idle and saves snapshot.
    - [x] Handlers in `endpoints.rs` delegate logic.
  - **Verification:**
    - [x] `cargo nextest run --test components status` passes.
  - **Dependencies:** Task 1
  - **Files likely touched:**
    - `src/application/application_service.rs`
    - `src/server/fragments/endpoints.rs`
  - **Estimated scope:** Small

### Checkpoint: After Tasks 1-7
- [x] `cargo check` passes
- [x] `cargo nextest run --tests` passes
- [x] No Axum handler contains `snapshot_storage.save()`, `message_storage.load_messages()`, or `GameStateSnapshot::from_game_state()`
- [x] All handlers are reduced to request parsing + `ApplicationService` delegation + HTTP response mapping

### Phase 2: Complete the View Model Layer

- [x] **Task 8: Extract existing view models to `server/view_models.rs`**
  - **Description:** Move `LogEntryView`, `LlmMessageView`, and `PreviewIssueView` from `templates.rs` to a new `view_models.rs`. Update imports.
  - **Acceptance criteria:**
    - [x] `src/server/view_models.rs` exists and contains all existing view models.
    - [x] `templates.rs` and `renderers.rs` compile with updated imports.
  - **Verification:**
    - [x] `cargo check` passes.
    - [x] `cargo nextest run --test templates_tests` passes.
  - **Dependencies:** None (can run in parallel with Phase 1)
  - **Files likely touched:**
    - `src/server/view_models.rs` (new)
    - `src/server/templates.rs`
    - `src/server/fragments/renderers.rs`
  - **Estimated scope:** Small

- [x] **Task 9: Create ActionAreaViewModel and VisualSidebarViewModel**
  - **Description:** Add view models decoupled from `GenerationStatus`/`GenerationPhase` and raw tuples. Update templates and renderers.
  - **Acceptance criteria:**
    - [x] `ActionAreaViewModel` replaces direct `GenerationStatus`/`GenerationPhase` usage in `ActionAreaTemplate`.
    - [x] `VisualSidebarViewModel` replaces raw `(String, String)` tuples in `VisualSidebarTemplate`.
    - [x] `templates.rs` no longer imports from `model::state` except through `From` impls in `view_models.rs`.
  - **Verification:**
    - [x] `cargo check` passes.
    - [x] `cargo nextest run --test templates_tests` passes.
    - [x] `cargo nextest run --test components` passes.
  - **Dependencies:** Task 8
  - **Files likely touched:**
    - `src/server/view_models.rs`
    - `src/server/templates.rs`
    - `src/server/fragments/renderers.rs`
  - **Estimated scope:** Small

### Checkpoint: After Tasks 8-9
- [x] `cargo check` passes
- [x] `cargo nextest run --tests` passes
- [x] Templates only accept view model structs

### Phase 3: Architectural Guardrails

- [x] **Task 10: Add arch-lint rule banning server -> storage**
  - **Description:** Add `deny-scope-dep` to `arch-lint.toml` preventing `server` from importing `storage`. This forces all storage access through `application`.
  - **Acceptance criteria:**
    - [x] `arch-lint.toml` contains rule banning `server` -> `storage`.
    - [x] `cargo nextest run --test architecture` passes.
  - **Verification:**
    - [x] `cargo nextest run --test architecture` passes.
    - [x] `cargo check` passes.
  - **Dependencies:** Tasks 1-7 (must be complete or build breaks)
  - **Files likely touched:**
    - `arch-lint.toml`
  - **Estimated scope:** XS

### Checkpoint: After Task 10
- [x] `cargo nextest run --test architecture` passes
- [x] `cargo nextest run --tests` passes

### Phase 4: UI State Extraction (Optional)

- [x] **Task 11: Remove dead UI fields from InputBuffer**
  - **Description:** Delete `cursor_position` and `scroll_offset` from `InputBuffer`. Remove `push_char`, `pop_char`, `clear_input` methods. Update snapshot mappers and tests.
  - **Acceptance criteria:**
    - [x] `InputBuffer` no longer contains `cursor_position` or `scroll_offset`.
    - [x] `state_tests.rs` updated or removed.
    - [x] Snapshot serialization unchanged for used fields.
  - **Verification:**
    - [x] `cargo check` passes.
    - [x] `cargo nextest run --tests` passes.
  - **Dependencies:** None
  - **Files likely touched:**
    - `src/model/state.rs`
    - `src/model/state_snapshot.rs`
    - `src/model/state_tests.rs`
    - `src/storage/mappers/` (if snapshot mapping changes)
  - **Estimated scope:** Small

### Checkpoint: After Task 11
- [x] `cargo nextest run --tests` passes
- [x] No behavioral regressions

## Risks and Mitigations

| Risk | Impact | Mitigation |
|------|--------|------------|
| AppState change breaks many test constructors | Medium | Task 1 is isolated; only touches constructors. Compile errors guide fixes. |
| ApplicationService method signatures are wrong on first pass | Low | Start with the most heavily leaked handlers (actions, retry, reset); adjust trait as needed before migrating remaining handlers. |
| arch-lint rule breaks build before refactor is done | Medium | Task 10 is explicitly ordered after all handler migrations. Do not add the rule early. |
| Removing cursor_position/scroll_offset breaks frontend JS | Low | These fields are not serialized to the frontend in any API response or template. Verify with `grep`. |

## Open Questions

- Should `ApplicationService` methods return rendered HTML strings, or raw data that handlers render? (Recommendation: raw data / `Result<..., EngineError>` to keep presentation out of the service.)
- Should `check_text_handler` remain in `actions.rs` or move elsewhere? It does not touch game state. (Recommendation: leave it; it's not part of the leak.)
