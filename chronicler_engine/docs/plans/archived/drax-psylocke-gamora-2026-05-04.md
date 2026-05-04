# Plan: Dependency-Inject LLM/Quantifier Backends into DefaultGameService

## Overview

Eliminate global state as the source of test flakiness. `DefaultGameService` currently reaches for `get_llm_backend()` and `get_quantifier_backend()` inside spawned threads. These globals read atomics and disk (`load_settings()`), creating race conditions when 17 tests run in parallel and each spawns OS threads.

The fix is simple: give `DefaultGameService` ownership of its backends via `Arc<dyn ...>`, thread them through `execute_action` / `retry_last_response` / `execute_freeaction_impl` / `evaluate_and_narrate_triggers`, and delete all global test-override atomics.

## Architecture Decisions

- **Backends live in the service, not in global functions.** `DefaultGameService` owns `Arc<dyn LlmBackend>` and `Arc<dyn QuantifierBackendTrait>`. Production loads them from settings once at startup. Tests inject mocks directly.
- **Trait `GameService` stays unchanged.** Only `DefaultGameService` internals and its callers change.
- **Spawned threads receive cloned `Arc`s.** No globals, no atomics, no file I/O in the hot path.
- **`main.rs` keeps `get_llm_backend()` for its one-off arrival thread.** It is the binary entry point; global state there is harmless and out of scope for test flakiness.
- **`FreeActionContext` carries the LLM backend.** `execute_freeaction_impl` already passes everything via `FreeActionContext`; adding the backend there is the cleanest threading path.

## Dependency Graph

```
narrative/llm.rs          narrative/quantifier.rs
    │                            │
    ▼                            ▼
server/mod.rs ──────► DefaultGameService
    │                        │
    │                        ├── execute_action ──► thread::spawn (clone Arc)
    │                        │                      │
    │                        │                      ▼
    │                        │              execute_freeaction_impl
    │                        │                      │
    │                        │                      ▼
    │                        │          evaluate_and_narrate_triggers
    │                        │
    │                        └── retry_last_response ──► thread::spawn (clone Arc)
    │
    ▼
create_app_for_testing ──► DefaultGameService::new()

main.rs (untouched)
    │
    └── get_llm_backend() for arrival thread (kept)
```

## Task List

### Task 1: Add backend fields to `DefaultGameService`

**Description:** Convert `DefaultGameService` from a unit struct to a struct holding `Arc<dyn LlmBackend>` and `Arc<dyn QuantifierBackendTrait>`. Add `with_backends()` constructor for tests. `new()` continues to load from settings.

**Files touched:**
- `src/engine/game_service.rs`

**Acceptance criteria:**
- [ ] `DefaultGameService` has `llm_backend: Arc<dyn LlmBackend>` and `quantifier_backend: Arc<dyn QuantifierBackendTrait>`
- [ ] `DefaultGameService::new()` loads backends via `Arc::from(get_llm_backend())` / `Arc::from(get_quantifier_backend())`
- [ ] `DefaultGameService::with_backends(llm, quantifier)` constructor exists
- [ ] `GameService` trait unchanged
- [ ] `server/mod.rs` tests that assert `Send + Sync` on `DefaultGameService` still pass

**Verification:**
- [ ] `cargo check` passes

---

### Task 2: Thread backends through `execute_action` and `retry_last_response`

**Description:** Replace `get_llm_backend()` / `get_quantifier_backend()` calls inside `DefaultGameService` with cloned `Arc`s from `self`. Pass them into spawned threads.

**Files touched:**
- `src/engine/game_service.rs`

**Acceptance criteria:**
- [ ] `execute_action` clones `Arc::clone(&self.llm_backend)` and `Arc::clone(&self.quantifier_backend)` before `thread::spawn`
- [ ] `retry_last_response` clones `Arc::clone(&self.llm_backend)` before its spawned thread
- [ ] No remaining `get_llm_backend()` or `get_quantifier_backend()` calls inside `game_service.rs`

**Verification:**
- [ ] `cargo check` passes
- [ ] `cargo test --test game_service_tests -- --test-threads=1` passes (baseline before removing globals)

---

### Task 3: Thread backend through `execute_freeaction_impl` to `evaluate_and_narrate_triggers`

**Description:** Add `llm_backend: &'a dyn LlmBackend` to `FreeActionContext`. Update `execute_freeaction_impl` to pass it to `evaluate_and_narrate_triggers`. Update `evaluate_and_narrate_triggers` signature to accept a backend reference instead of calling `get_llm_backend()`.

**Files touched:**
- `src/engine/action_processing.rs`

**Acceptance criteria:**
- [ ] `FreeActionContext` has `pub llm_backend: &'a dyn LlmBackend`
- [ ] `evaluate_and_narrate_triggers` takes `llm_backend: &dyn LlmBackend` as parameter
- [ ] No remaining `get_llm_backend()` call inside `action_processing.rs`
- [ ] `action_processing.rs` test `test_evaluate_and_narrate_triggers_adds_event_header` passes after update

**Verification:**
- [ ] `cargo test --test action_processing` passes

---

### Task 4: Update server initialization and test factory

**Description:** `AppState::new` and `create_app_for_testing` both call `DefaultGameService::new()`. No code changes needed there, but verify they still compile after `DefaultGameService` struct changes. Update `server/mod.rs` unit test `test_app_state_struct_fields` if it constructs `DefaultGameService` manually.

**Files touched:**
- `src/server/mod.rs`

**Acceptance criteria:**
- [ ] `AppState::new` compiles unchanged
- [ ] `create_app_for_testing` compiles unchanged
- [ ] `test_app_state_struct_fields` compiles and passes

**Verification:**
- [ ] `cargo test --test component_tests` passes

---

### Task 5: Convert all `game_service_tests` to dependency injection

**Description:** Every test that uses `with_test_backend()` / `with_test_quantifier_backend()` should instead create a `DefaultGameService::with_backends(Arc::new(MockBackend), Arc::new(MockQuantifierBackend::default()))`.

**Files touched:**
- `tests/game_service_tests.rs`

**Acceptance criteria:**
- [ ] No remaining `with_test_backend` or `with_test_quantifier_backend` calls in `game_service_tests.rs`
- [ ] All 17 tests pass under parallel execution (`cargo test --test game_service_tests`)
- [ ] Timeouts can be reduced to 200ms because there is no disk I/O and no atomic races

**Verification:**
- [ ] `cargo test --test game_service_tests` passes (parallel)
- [ ] `cargo test --test game_service_tests -- --test-threads=1` passes (serial)

---

### Task 6: Convert `action_processing.rs` test to dependency injection

**Description:** Update the one test (`test_evaluate_and_narrate_triggers_adds_event_header`) that uses `with_test_backend()` to pass a mock backend directly into `evaluate_and_narrate_triggers`.

**Files touched:**
- `src/engine/action_processing.rs` (test module)

**Acceptance criteria:**
- [ ] No remaining `with_test_backend` or `with_test_quantifier_backend` calls in `action_processing.rs` tests
- [ ] Test passes

**Verification:**
- [ ] `cargo test --test action_processing` passes

---

### Task 7: Delete global test-override infrastructure

**Description:** Remove the atomic overrides, guard types, and setter functions from `llm.rs` and `quantifier.rs`. These are no longer needed once tests inject backends directly.

**Files touched:**
- `src/narrative/llm.rs`
- `src/narrative/quantifier.rs`

**Acceptance criteria:**
- [ ] `TEST_BACKEND_OVERRIDE` deleted
- [ ] `TEST_MOCK_SHOULD_FAIL` deleted
- [ ] `set_test_backend`, `clear_test_backend`, `with_test_backend`, `TestBackendGuard` deleted
- [ ] `set_mock_should_fail`, `with_mock_failure`, `MockFailureGuard` deleted
- [ ] `TEST_QUANTIFIER_OVERRIDE` deleted
- [ ] `set_test_quantifier_backend`, `clear_test_quantifier_backend`, `with_test_quantifier_backend`, `TestQuantifierGuard` deleted
- [ ] `get_llm_backend()` and `get_quantifier_backend()` are **preserved** for production use (server init, main.rs)

**Verification:**
- [ ] `cargo check` passes
- [ ] `cargo test --test guardrails` passes (no dead code warnings)

---

### Checkpoint: After Tasks 1–7

- [ ] Full build passes: `python build.py --coverage`
- [ ] Coverage ≥ 90%
- [ ] Zero flaky tests under nextest
- [ ] `cargo test --test game_service_tests` passes in parallel with 200ms timeouts

## Risks and Mitigations

| Risk | Impact | Mitigation |
|------|--------|------------|
| `Arc::from(Box<dyn Trait>)` doesn't compile for one of the traits | High | Verified: both `LlmBackend` and `QuantifierBackendTrait` are `Send + Sync`, `Arc::from(Box<T>)` works for `T: ?Sized` |
| `FreeActionContext` lifetime conflict with new field | Medium | `&'a dyn LlmBackend` is straightforward; no lifetime conflicts expected |
| Server/mod.rs tests construct `DefaultGameService` directly | Low | `DefaultGameService::new()` signature unchanged; only internal struct changes |
| `main.rs` arrival narration still uses `get_llm_backend()` | Low | Out of scope; main.rs is a single binary entry point, not a test |

## Open Questions

- Should we also pass `max_context` / `max_tokens` into `evaluate_and_narrate_triggers` to eliminate its `load_settings()` call? (Currently not causing flakiness, but for consistency it could be a follow-up.)
- Should `get_llm_backend()` and `get_quantifier_backend()` be moved behind a factory trait for even cleaner DI? (Overkill for current scope; `Arc::from(get_*_backend())` is sufficient.)