# Plan: Phase 4 — Replace std::thread::spawn with Tokio

**Goal:** Remove all `std::thread::spawn` from production code in `chronicler_engine`, replacing with `tokio::task::spawn_blocking` (or `runtime.spawn_blocking`). Keep backend traits (`LlmBackend`, `QuantifierBackendTrait`, `GameService`) synchronous. Move spawning responsibility to the callers (HTTP handlers and bootstrap).

**Approach:** Make `GameService` methods fully synchronous (no internal thread spawning). Callers that need non-blocking behavior spawn the synchronous work via `tokio::task::spawn_blocking`. This avoids `#[async_trait]`, `dyn Trait` async incompatibility, and minimizes test changes.

---

## Files to Change

### 1. `src/engine/game_service.rs`

**Remove internal thread spawning.** Both `execute_action` (FreeAction branch) and `retry_last_response` currently spawn `std::thread` to do LLM + quantifier work. Extract this work to run inline (synchronously).

- **Line 4:** Remove `use std::thread;`
- **Lines 127-243 (`execute_action` FreeAction branch):** Remove `thread::spawn(move || { ... })` wrapper. The body already `drop(state_guard)` before the heavy work, so the mutex is released. The code inside the closure becomes inline.
- **Lines 248-346 (`retry_last_response`):** Remove `thread::spawn(move || { ... })` wrapper. The body already extracts data and drops locks before the thread starts.
- **Line 293:** Remove `std::thread::sleep(Duration::from_millis(50))` (was a hack to let "inner threads" start first — no longer needed).

**Test impact:** `tests/game_service_tests.rs` calls these methods directly. Mock backends return instantly, so synchronous execution completes before the test polls state. All existing tests pass without modification.

### 2. `src/server/fragments.rs`

**Replace `std::thread::spawn` with `tokio::task::spawn_blocking` in HTTP handlers.** The handlers already spawn threads to call `GameService`; we just change the spawn mechanism.

- **Lines 328-341 (`action_handler`, async action path):**
  ```rust
  // BEFORE
  std::thread::spawn(move || {
      std::thread::sleep(std::time::Duration::from_millis(50));
      game_service.execute_action(state_clone, cmd, pname);
  });
  // AFTER
  tokio::task::spawn_blocking(move || {
      game_service.execute_action(state_clone, cmd, pname);
  });
  ```
- **Lines 616-622 (`retry_handler`):**
  ```rust
  // BEFORE
  std::thread::spawn(move || {
      std::thread::sleep(std::time::Duration::from_millis(50));
      game_service.retry_last_response(state_clone);
  });
  // AFTER
  tokio::task::spawn_blocking(move || {
      game_service.retry_last_response(state_clone);
  });
  ```

Handlers are `async fn` running under the Tokio runtime, so `tokio::task::spawn_blocking` is available and returns immediately (non-blocking for the HTTP response).

### 3. `src/bootstrap.rs`

**Replace arrival narration `thread::spawn` with `runtime.spawn_blocking()`.** Bootstrap creates the Tokio runtime before blocking on the server. We must use `runtime.spawn_blocking()` (not `tokio::task::spawn_blocking`) because the spawn happens *before* `block_on` enters the runtime context.

- **Line 8:** Remove `thread` from `use std::{...}`
- **Lines 305-346:** Restructure:
  ```rust
  let runtime = tokio::runtime::Runtime::new()?;

  if !has_scenario {
      let state_for_task = state.clone();
      // Move captured values into the closure (world, map, player, room_id, history, all_npcs, nearby_npcs)
      let _handle = runtime.spawn_blocking(move || {
          let _guard = GeneratingGuard::new(state_for_task.clone());
          // ... existing arrival narration body ...
      });
  }

  runtime.block_on(crate::server::run_server_with_config(state, config))?;
  ```

The `_handle` is dropped; `block_on` keeps the runtime alive until the server future completes, so the spawned blocking task can finish.

### 4. `src/model/state.rs` (test only — NO CHANGE)

Line 495 `std::thread::spawn` is inside `#[test] fn test_generating_guard_poisoned_lock_recovers`. It intentionally tests mutex poisoning across OS threads. Keep as-is.

---

## Execution Order

1. **`game_service.rs`** — Remove internal spawns first. Run `python build.py`.
2. **`fragments.rs`** — Update handlers to `spawn_blocking`. Run `python build.py`.
3. **`bootstrap.rs`** — Update arrival narration spawn. Run `python build.py`.

Each step is independent enough to build and test individually. The correct dependency order is: game_service → fragments → bootstrap.

---

## Testing Strategy

- After each file: `cd chronicler_engine && python build.py` (fmt → clippy → arch tests → guardrails → build → nextest)
- Verify: 522 tests passing, 3 skipped, coverage ≥ 85.7%
- No test modifications required in `tests/game_service_tests.rs` because mock backends are synchronous and fast.
- `fragments.rs` unit tests (`test_html_escape_*`, `test_action_form_deserialization`) don't touch the handlers.
- `bootstrap.rs` unit tests don't touch the `run()` function.

---

## Risks & Mitigations

| Risk | Mitigation |
|------|------------|
| `tokio::task::spawn_blocking` panics if no runtime | Handlers are `async fn` served by Axum/Tokio — runtime guaranteed. Bootstrap uses `runtime.spawn_blocking()` explicitly. |
| FreeAction test race conditions | Mock backend returns instantly; synchronous `execute_action` completes before test assertions. `wait_for_generation_complete` will see Idle on first poll. |
| `GeneratingGuard` in bootstrap not dropped if server exits early | Same behavior as before — guard drops when blocking task ends. `_handle` doesn't need explicit awaiting. |
| Clippy warnings from unused imports | Remove `use std::thread;` in `game_service.rs` and `bootstrap.rs`. |

---

## Post-Phase 4 State

- Zero `std::thread::spawn` in production Rust code (only `model/state.rs` test remains).
- Zero `std::thread::sleep` in production Rust code.
- Backend traits remain synchronous.
- All 522 tests passing.
