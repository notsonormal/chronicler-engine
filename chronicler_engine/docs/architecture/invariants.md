# Runtime Invariants

Machine-checkable statements about engine runtime behavior. Violations indicate bugs.

## State Mutations

### INV-001: Generation Status Lifecycle
`generation_state.status` must return to `Idle` after every action.
- **Test:** `tests/invariant_contract_tests.rs::test_inv001_generation_status_resets_on_panic`

### INV-002: State Mutation Order
`execute_freeaction_impl` applies mutations in order: handle_movement → resolve NPCs → add_log → evaluate_triggers → apply_npc_events. Violations compile but break silently.
- **Test:** `tests/invariant_contract_tests.rs::test_inv002_state_mutation_order`
- **Test:** `tests/invariant_contract_tests.rs::test_inv002_mutation_order_property` (proptest)
- **Checklist:** `docs/architecture/mutation-order.md`

## Concurrency

### INV-003: No Raw OS Thread Spawning
No `std::thread::spawn` in `src/`. All concurrent work uses `tokio::task::spawn_blocking`.
- **Test:** `tests/guardrails/structure.rs::guardrails_no_std_thread`

### INV-004: LLM Calls Are Cancellable
Blocking LLM work checks `CancellationToken` before/after backend calls and at `ActionPipeline` stage boundaries.
- **Test:** `tests/invariant_contract_tests.rs::test_inv004_cancellable_at_boundaries`

### INV-004b: No Concurrent Async Actions
Only one `FreeAction` generation in flight at a time. Server rejects overlaps.
- **Test:** `tests/invariant_contract_tests.rs::test_inv004b_no_concurrent_async_actions`

### INV-005: Lock Poison Recovery
All `Mutex`/`RwLock` sites recover from poison via `into_inner()`.
- **Test:** `tests/poison_recovery.rs::test_settings_recover_from_poisoned_rwlock`

## HTTP Layer

### INV-006: All Actions Are Async
All player input is parsed as `FreeAction` and offloaded to `spawn_blocking`.
- **Enforced by:** Architecture review (no dynamic test)

### INV-007: Actions Return Immediately
Handlers return `"Thinking..."` before the LLM call begins.
- **Enforced by:** Architecture review (no dynamic test)
