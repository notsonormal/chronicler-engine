# Ticket #12: Block `Storage` namespace in HTTP layer

## Summary

Remove the remaining direct `Storage` references from non-test files under `src/adapters/driving/http/` by introducing two application-layer seams (`SettingsService`, `PromptPresetService`), migrating the HTTP handlers to them, deleting the now-dead `utils::settings::save_settings` free function, migrating test consumers off `AppState.storage`, and adding a guardrail walker that flags future regressions. This closes the gap where `arch-lint.toml`'s import-based `server → storage` deny misses fully-qualified inline paths like `crate::adapters::driven::storage::Storage`.

## Key Changes

- Two new thin application-layer services in `src/application/` (`SettingsService`, `PromptPresetService`) holding `Arc<Storage>` — identical delegation signatures so handler churn is minimal.
- `AppState` drops `storage` and `preset_storage` fields; `WiredApp` keeps them (bootstrap still needs them) and additionally carries the two new services.
- `settings.rs` and `prompt_presets.rs` handlers migrate to the services.
- `utils/settings.rs::save_settings` deleted (only the two handler files used it).
- Test helpers and affected integration tests switch to builder methods that return `(AppState, Arc<Storage>)`; `PipelineHelpers::save_test_state` / `add_input_and_save` gain a `&Storage` param.
- Guardrail walker `check_http_storage_leak` added to `tests/infrastructure/guardrails/layers.rs` and registered in `mod.rs`.

## Implementation

### Phase 1: Create application-layer seams

- [ ] #### Task 1.1: Create `SettingsService` (1 SP)
  - New `src/application/settings_service.rs` with `#[derive(Clone)] pub struct SettingsService { storage: Arc<Storage> }` and `pub fn save_settings(&self, settings: &AppSettings) -> Result<(), EngineError>` delegating to `self.storage.save_settings(settings)`.
  - Add `pub mod settings_service;` and `pub use settings_service::SettingsService;` to `src/application/mod.rs`.
  - Add `src/application/settings_service_tests.rs` with a round-trip save/reload test through in-memory storage; register as `#[cfg(test)] mod settings_service_tests;` in `application/mod.rs`.

- [ ] #### Task 1.2: Create `PromptPresetService` (1 SP)
  - New `src/application/prompt_preset_service.rs` with `#[derive(Clone)] pub struct PromptPresetService { storage: Arc<Storage> }` and methods `get_preset`, `list_presets`, `save_preset`, `delete_preset` matching `Storage`'s signatures exactly.
  - Add `pub mod prompt_preset_service;` and `pub use prompt_preset_service::PromptPresetService;` to `src/application/mod.rs`.
  - Add `src/application/prompt_preset_service_tests.rs` with round-trip CRUD tests; register as `#[cfg(test)] mod prompt_preset_service_tests;`.

### Phase 2: Wire services into composition root and AppState

- [ ] #### Task 2.1: Extend `WiredApp` (1 SP)
  - In `src/bootstrap/wiring.rs`, in `build_wired_app`, construct `SettingsService::new(Arc::clone(&storage))` and `PromptPresetService::new(Arc::clone(&preset_storage))`.
  - Add `settings_service: SettingsService` and `prompt_preset_service: PromptPresetService` fields to `WiredApp`; populate them in the struct literal.
  - Keep `storage: Arc<Storage>` and `preset_storage: Arc<Storage>` on `WiredApp` (still needed for boot-heal, `AgentRegistry`, and the test `rebind_for_test` path).

- [ ] #### Task 2.2: Trim `AppState` (1 SP)
  - In `src/adapters/driving/http/app_state.rs`, remove the `storage` and `preset_storage` fields and their inline `crate::adapters::driven::storage::Storage` references.
  - Add `settings_service: SettingsService` and `prompt_preset_service: PromptPresetService`.
  - Update `AppState::from_wired` to copy `wired.settings_service` and `wired.prompt_preset_service`.

### Phase 3: Migrate HTTP handlers

- [ ] #### Task 3.1: Migrate `settings.rs` handlers (1 SP)
  - Remove `use crate::utils::settings::save_settings;`.
  - Replace all 7 `save_settings(&settings, &app_state.storage)` call sites with `app_state.settings_service.save_settings(&settings)`.

- [ ] #### Task 3.2: Migrate `prompt_presets.rs` handlers (1 SP)
  - Remove `use crate::utils::settings::save_settings;`.
  - Rename every `app_state.preset_storage.X(...)` call to `app_state.prompt_preset_service.X(...)` (including all `require_preset!` macro call sites).
  - In `activate_preset_handler`, replace `save_settings(&settings, &app_state.storage)` with `app_state.settings_service.save_settings(&settings)`.

### Phase 4: Delete dead free function

- [ ] #### Task 4.1: Remove `utils::settings::save_settings` (1 SP)
  - Delete the `save_settings` free function from `src/utils/settings.rs`.
  - Keep `load_settings` (used by `bootstrap/run.rs`) and `get_settings_path`.

### Phase 5: Migrate test consumers off `AppState.storage`

- [ ] #### Task 5.1: Update `PipelineHelpers` trait (1 SP)
  - In `tests/helpers/application_ext.rs`, add a `&Storage` parameter to `save_test_state` and `add_input_and_save`; `add_input_and_save` forwards the parameter to `save_test_state`.
  - Leave `wait_for_generation_complete`, `latest_state`, and `latest_snapshot` unchanged (they use `message_service` only).

- [ ] #### Task 5.2: Update `SqliteTestAppBuilder` (1 SP)
  - In `tests/helpers/sqlite_test_app_builder.rs`, replace internal `app_state.storage.X()` calls in the `is_generating` branches with a local pre-cloned `Arc<Storage>` handle.
  - **Invariant:** clone `Arc::clone(&storage)` into a local **before** calling `build_test_wired_app*`; use that local in the `is_generating` branch, not a re-borrow of the consumed `storage`. Both `Arc`s point at the same `Storage`, which is the intended shared state.
  - Add `build_with_state_and_storage() -> Result<(AppState, Arc<Storage>)>` and `build_service_and_storage() -> Result<(AppState, Arc<Storage>)>`.

- [ ] #### Task 5.3: Update `TestAppBuilder` (1 SP)
  - In `src/test_support/test_app_builder.rs`, add `build_service_with_storage() -> (AppState, Arc<Storage>)`.

- [ ] #### Task 5.4: Update affected tests (1 SP)
  - `tests/integration/flow/{arrival_persistence,retry_main,retry_event,sequence}.rs`: switch relevant test functions to the new `build_with_state_and_storage()` method; pass `&storage` to `add_input_and_save` and `save_test_state`; replace `app.storage.X()` with `storage.X()`.
  - `tests/infrastructure/invariant_contract.rs` and `tests/integration/application/action_pipeline/pipeline.rs`: switch only the test functions that touch `app.storage` to `build_with_state_and_storage()`; leave other `build_with_state()` callers untouched.
  - `tests/integration/application/lifecycle.rs`: switch relevant `TestAppBuilder` calls to `build_service_with_storage()`; replace `app_service.storage.current_game_id()` with the local `storage.current_game_id()`.
  - `src/adapters/driving/http/app_state_tests.rs`: update the two manual `AppState { … }` literals to use `settings_service: wired.settings_service.clone()` and `prompt_preset_service: wired.prompt_preset_service.clone()` instead of the removed `storage`/`preset_storage` fields.

### Phase 6: Add the guardrail walker

- [ ] #### Task 6.1: Implement `check_http_storage_leak` (1 SP)
  - In `tests/infrastructure/guardrails/layers.rs`, add `pub fn check_http_storage_leak(file_path: &str, content: &str) -> Vec<Violation>` that:
    - returns early unless the normalized path starts with `adapters/driving/http/`;
    - returns early for `*_tests.rs`;
    - skips comment lines;
    - flags any line containing `crate::adapters::driven::storage::Storage` or `adapters::driven::storage::Storage`.
  - Add unit tests in the `tests` module: catches a violation in `adapters/driving/http/app_state.rs`, allows `*_tests.rs`, allows a non-http file referencing `Storage`, skips comments.

- [ ] #### Task 6.2: Register the walker (1 SP)
  - In `tests/infrastructure/guardrails/mod.rs`, add:
    ```rust
    #[test]
    fn guardrails_http_storage_leak() {
        check_src_files("HTTP storage leak", check_http_storage_leak);
    }
    ```

### Phase 7: Verify and resolve

- [ ] #### Task 7.1: Verification (1 SP)
  - `cargo check --all-targets --all-features` green.
  - `cargo nextest run --test guardrails` green (new walker passes; existing guardrails stay green).
  - `cargo nextest run --test architecture` green.
  - `python chronicler_engine/build.py` green.
  - Grep: no `Storage` references remain in non-test files under `src/adapters/driving/http/`.

- [ ] #### Task 7.2: Resolve ticket #12 (1 SP)
  - Post resolution comment on `.scratch/pipeline-review-hygiene/issues/12-block-storage-in-http-layer.md` summarizing the services added, handler migration, test migration, walker added, and dead function deleted.
  - Set `Status: closed`, clear assignee.
  - Append a one-line gist to `.scratch/pipeline-review-hygiene/map.md` Decisions-so-far.

## Test Plan

- New unit tests: `settings_service_tests.rs` (settings save round-trip), `prompt_preset_service_tests.rs` (preset CRUD round-trip).
- Existing guardrail tests: new walker exercises `check_http_storage_leak` via in-module unit tests; full `guardrails` integration test verifies the walker catches `app_state.rs` after migration.
- Full suite: `build.py` (cargo check + nextest + integration/LLM smoke tests) must remain green.

## Per Task/Sub Task Validation Steps

| Task | Validation |
|---|---|
| 1.1 / 1.2 | `cargo check` green; new `*_tests.rs` compile and pass. |
| 2.1 / 2.2 | `cargo check` green; grep confirms no `Storage` text in `app_state.rs`. |
| 3.1 / 3.2 | `cargo check` green; handlers compile; `utils::settings::save_settings` no longer imported. |
| 4.1 | `cargo check` green (stragglers would fail here). |
| 5.1–5.4 | `cargo check --all-targets` green; `cargo nextest run --test guardrails` green; grep confirms no `app.storage` / `app_state.storage` / `app_service.storage` field access remains. |
| 6.1 / 6.2 | `cargo nextest run --test guardrails` green; deliberately re-adding `Storage` to `app_state.rs` makes the new test fail. |
| 7.1 | `python chronicler_engine/build.py` green. |
| 7.2 | Tracker updated; map Decisions-so-far appended. |

## Assumptions

- Ticket #11's `MessageService` migration is complete and green in the working tree (per your direction); #12's verification step (`cargo check --all-targets`) will catch any latent #11 compile breakage.
- No HTTP handler outside `settings.rs` and `prompt_presets.rs` references `Storage` directly (verified by grep; the new walker will enforce this).
- `WiredApp` must retain raw `storage`/`preset_storage` fields for bootstrap and test-rebind paths; only `AppState` is constrained.
- The mechanical `&Storage` parameter on `PipelineHelpers` and the new builder `*_and_storage` methods are the minimal-churn way to keep ~75 unaffected `build_with_state`/`build_service` callers untouched.
- Unit tests for the two new pass-through services follow the #10 catalogue precedent even though they primarily exercise `Storage` round-trips; they lock the seams' public shapes.
