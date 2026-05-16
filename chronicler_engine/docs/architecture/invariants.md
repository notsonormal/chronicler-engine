# Chronicler Engine Runtime Invariants

These invariants are machine-checkable statements about the engine's runtime behavior. Violations indicate bugs.

## State Mutations

### INV-001: Generation Status Lifecycle
`generation_state.status` must return to `Idle` after every action (sync or async). No action may leave the engine permanently stuck in `Generating`.

- **Enforced by:** `GeneratingGuard::drop` resets to `Idle` on scope exit. Poisoned mutexes are recovered via `Mutex::clear_poison()`.
- **Spawn sites:** `fragments.rs` handlers check `CancellationToken` and reset status if cancelled.

### INV-002: State Mutation Order in FreeAction
`execute_freeaction_impl` must apply mutations in this order:
1. Parse quantifier result for world-state changes (movement, item transfers).
2. Update `npcs_in_area` based on quantifier output.
3. Append narration to `narrative` history via `add_log()`.
4. Evaluate triggers against the mutated state (inside lock).
5. Apply NPC events (`times_met` increments, `currently_meeting` updates).
6. Release lock → trigger LLM call (frontend can poll main narration).
7. Re-acquire lock → commit trigger narration logs and mark trigger fired.

Reordering steps 1 and 3 would cause triggers to fire against stale state.
Moving the trigger LLM call inside the lock would block frontend polling, causing both narrations to appear simultaneously.

## Concurrency

### INV-003: No Raw OS Thread Spawning in Production Code
No `std::thread::spawn` or `std::thread::sleep` may appear in `src/`. All concurrent work must use Tokio's structured concurrency (`tokio::task::spawn_blocking`).

- **Exception:** `tests/` may use `std::thread` for mutex-poisoning tests.
- **Guardrail:** `tests/guardrails.rs` enforces this via `guardrails_no_std_thread`.

### INV-004: LLM Backend Calls Must Be Cancellable
All blocking LLM work runs inside `tokio::task::spawn_blocking`. Spawn closures check `CancellationToken::is_cancelled()` before and after the backend call. If cancelled, `generation_state.status` is reset to `Idle`.

- **Shutdown path:** `Ctrl+C` triggers `axum::serve` graceful shutdown, which cancels the token and lets in-flight tasks finish cleanly.

### INV-004b: No Concurrent Async Actions
Only one async (`FreeAction`) generation may be in flight at a time. The server rejects subsequent async action requests with `"Still thinking..."` while a generation is active. This prevents snapshot race conditions from overlapping read-modify-write cycles.

- **Enforced by:** `AppState::is_generating` (`AtomicBool`) checked with `compare_exchange` in `process_action`. The flag is cleared by `GenerationGuard::drop` when the `spawn_blocking` task exits, even on panic.
- **Client-side:** HTMX `hx-sync="this:drop"` on the command form drops duplicate submissions before they reach the server.

### INV-005: Mutex Poison Recovery
If a `std::sync::Mutex<GameState>` is poisoned, the engine must recover rather than panic. `GeneratingGuard` recovers poisoned locks by calling `Mutex::clear_poison()` and resetting status.

## HTTP Layer

### INV-006: All Actions Are Async
All player input is parsed as `FreeAction` or `Talk` and offloaded to `tokio::task::spawn_blocking` for LLM generation. There are no synchronous action paths.

### INV-007: Actions Return Immediately
All action handlers set `status = Generating`, save a snapshot, and spawn a blocking task. The HTTP response returns `"Thinking..."` before the LLM call begins.
