# Title: Delete `LlmMessageRepository` Port, Closure-Substitute Recorder Save Seam, Update ADR-027

## Summary

Remove the `LlmMessageRepository` port trait and replace its single production consumer (`LlmCallRecorder`) with a closure-based save seam (`SaveLlmMessageFn = Arc<dyn Fn(&LlmMessage) -> Result<(), EngineError> + Send + Sync>`). Restore architectural symmetry with the rejected `StateRepository` decision: both persistence concerns now use concrete-struct + substitution-by-mechanism-other-than-trait (Backend enum for state, closure for recorder). Update ADR-027 to reflect the revision.

## Key Changes

- **Delete trait** `LlmMessageRepository` and its `_tests` file.
- **Keep DTO** `LlmMessage` — move from `application/ports/llm_message_repository.rs` to new `application/llm_message.rs` (alongside `LlmCallRecorder`). Add `SaveLlmMessageFn` type alias in same module.
- **Recorder signature** `LlmCallRecorder::new(provider: Arc<dyn LlmProvider>, save_fn: SaveLlmMessageFn)`.
- **Storage methods** stay concrete (`save_llm_message`, `list_latest_llm_messages`) — convert trait impl in `adapters/driven/storage/backend/llm_messages.rs` to inherent impl, keep filename.
- **Test support delete** `noop_forensics.rs`, `recording_forensics.rs`, `recording_forensics_tests.rs`. Replace with closure factories in `test_support/mod.rs` (`make_noop_save_fn`, `make_recording_save_fn` builder returning `(SaveLlmMessageFn, RecordingSaveSpy)`).
- **Bootstrap** `bootstrap/llm_factory.rs`: wrap `Arc<Storage>::save_llm_message` in closure.
- **Test migration** ~22 files. `SpyForensics::save_count()` → `Arc<AtomicUsize>::load()`. `RecordingForensics::with_next_save_error()` → `RecordingSaveSpy` builder method that toggles internal `Arc<Mutex<Option<EngineError>>>`.
- **ADR-027** revision: remove `LlmMessageRepository` from Accepted Port Traits, rewrite Phantom Port Heuristic as two-clause rule (dependency inversion + test-seam/ceremony justification), add 2026-07-06 History entry.
- **Doc touchups** `docs/system/llm_processing.md` — remove any references to `LlmMessageRepository` port (the doc references the recorder contract, not the port, but verify).

## Implementation

### Phase 1: Port cleanup + recorder signature change (3 SP)

- [ ] #### Task 1.1: Create `application/llm_message.rs` with `LlmMessage` struct + `SaveLlmMessageFn` alias (1 SP)
  - [ ] ##### SubTask 1.1.1: Move `LlmMessage` struct verbatim from `application/ports/llm_message_repository.rs` to new `application/llm_message.rs`. Preserve `Debug, Clone` derives, all fields, all `#[doc]` attributes.
  - [ ] ##### SubTask 1.1.2: Add `pub type SaveLlmMessageFn = Arc<dyn Fn(&LlmMessage) -> Result<(), EngineError> + Send + Sync>;` to the same module.
  - [ ] ##### SubTask 1.1.3: Register module in `src/application/mod.rs`. Update all current importers of `application::ports::llm_message_repository::LlmMessage` (production + adapter + test) to import from `application::llm_message::LlmMessage`.
- [ ] #### Task 1.2: Update `LlmCallRecorder` to accept `SaveLlmMessageFn` (1 SP)
  - [ ] ##### SubTask 1.2.1: Change field `forensics: Arc<dyn LlmMessageRepository>` → `save_fn: SaveLlmMessageFn`. Change constructor parameter type accordingly.
  - [ ] ##### SubTask 1.2.2: Change `self.forensics.save_llm_message(&message)?` call inside `complete()` to `(*self.save_fn)(&message)?`.
  - [ ] ##### SubTask 1.2.3: Update `src/application/llm_recorder.rs` imports — drop `LlmMessageRepository`, add `SaveLlmMessageFn` from `crate::application::llm_message`.
  - [ ] ##### SubTask 1.2.4: Add unit test for error propagation: closure that returns `Err(EngineError::...)` → `LlmCallRecorder::complete` returns same error. Replaces coverage lost from `RecordingForensics::with_next_save_error` test path.
- [ ] #### Task 1.3: Delete trait files; convert Storage impl to inherent (1 SP)
  - [ ] ##### SubTask 1.3.1: Convert `adapters/driven/storage/backend/llm_messages.rs`: change `impl LlmMessageRepository for Storage { ... }` → `impl Storage { pub fn save_llm_message(...) -> ...; pub fn list_latest_llm_messages(...) -> ...; }`. Keep file path.
  - [ ] ##### SubTask 1.3.2: Delete `src/application/ports/llm_message_repository.rs` (now only contains trait + doc anchor; trait gone, doc anchor moves to `llm_message.rs`).
  - [ ] ##### SubTask 1.3.3: Delete `src/application/ports/llm_message_repository_tests.rs`.
  - [ ] ##### SubTask 1.3.4: Update `src/application/ports/mod.rs` — remove `pub mod llm_message_repository;` and `#[cfg(test)] mod llm_message_repository_tests;`.

### Phase 2: Test support migration (5 SP)

- [ ] #### Task 2.1: Build closure factories in `test_support/mod.rs` (1 SP)
  - [ ] ##### SubTask 2.1.1: Add `pub fn make_noop_save_fn() -> SaveLlmMessageFn` — closure returning `Ok(())`.
  - [ ] ##### SubTask 2.1.2: Add `RecordingSaveSpy` struct holding `Arc<Mutex<RecordingState>>` with fields `{ save_count: usize, last_message: Option<LlmMessage>, next_save_error: Option<EngineError> }`. Add methods `save_call_count()`, `last_saved_message()`, `with_next_save_error(err)`, `new()`.
  - [ ] ##### SubTask 2.1.3: Add `pub fn make_recording_save_fn() -> (SaveLlmMessageFn, Arc<RecordingSaveSpy>)` — returns closure that increments count, captures message, injects configured error.
  - [ ] ##### SubTask 2.1.4: Delete `src/test_support/noop_forensics.rs`, `src/test_support/recording_forensics.rs`, `src/test_support/recording_forensics_tests.rs`.
  - [ ] ##### SubTask 2.1.5: Update `src/test_support/mod.rs` — remove module decls and `pub use noop_forensics::*; pub use recording_forensics::*;`. Add new `pub use` for `RecordingSaveSpy`, `make_noop_save_fn`, `make_recording_save_fn`.
  - [ ] ##### SubTask 2.1.6: Update `make_test_recorder`, `make_test_recorder_with_storage`, `make_spy_recorder` helpers (currently in `noop_forensics.rs`) — port to `test_support/mod.rs` using new closure factories.
- [ ] #### Task 2.2: Migrate unit test sites (3 SP)
  - [ ] ##### SubTask 2.2.1: Migrate `src/application/**/*tests.rs` (7 files: `llm_recorder_tests`, `action_pipeline/{actions,pipeline,retry}_tests`, `agents/quantifier/{agent,orchestration}_tests`, `agents/registry_tests`) — mechanical replacement: `NoopForensics` → `make_noop_save_fn`, `SpyForensics::save_count()` → `Arc<AtomicUsize>::load()`, `RecordingForensics::with_*` → `RecordingSaveSpy::with_*` + use returned closure. (1 SP)
  - [ ] ##### SubTask 2.2.2: Migrate `src/adapters/**/*tests.rs` (3 files: `llm/providers/mock_tests`, `storage/backend/llm_messages_tests`, `storage/mappers/llm_message_tests`) + `src/bootstrap/wiring.rs` — same pattern. (1 SP)
  - [ ] ##### SubTask 2.2.3: Migrate production consumers of the trait in `src/application/agents/quantifier/agent.rs` + `src/application/agents/registry.rs` — these pass `Arc<dyn LlmMessageRepository>` (or `Arc<Storage>`) into agent constructors. Update to construct `SaveLlmMessageFn` closure. (1 SP)
- [ ] #### Task 2.3: Migrate integration test sites (1 SP)
  - [ ] ##### SubTask 2.3.1: Migrate `tests/integration/**` (8 files: `application/{action_pipeline/{pipeline,retry}, game_service, lifecycle, wiring}`, `flow/{arrival_persistence, retry_event, retry_main, sequence}`) + `tests/integration/mod.rs` + `tests/test_utils/mod.rs` + `tests/infrastructure/invariant_contract.rs` — same mechanical replacement as unit tests.

### Phase 3: ADR + docs + validation (1 SP)

- [ ] #### Task 3.1: Revise ADR-027 (1 SP)
  - [ ] ##### SubTask 3.1.1: Remove `LlmMessageRepository` row from "Accepted Port Traits" table.
  - [ ] ##### SubTask 3.1.2: Rewrite "Phantom Port Heuristic" section. New rule: a port is justified when **(consumer in core AND producer is adapter)** *and* **test seam requires capabilities not satisfied by an existing mechanism** (Backend enum, direct methods) *and* **trait ceremony is proportionate** (~few methods, focused concern). Otherwise the concrete struct + closure or concrete struct + Backend enum covers the seam with less ceremony.
  - [ ] ##### SubTask 3.1.3: Add History entry dated 2026-07-06: "Removed `LlmMessageRepository` port. `LlmCallRecorder` now uses `SaveLlmMessageFn = Arc<dyn Fn(&LlmMessage) -> Result<(), EngineError> + Send + Sync>` closure. Test support types (NoopForensics, SpyForensics, RecordingForensics) replaced with closure factories. Restores symmetry with rejected `StateRepository` decision. Phantom-port heuristic rewritten as three-clause rule."
  - [ ] ##### SubTask 3.1.4: Update "Consequences → Positive" — remove or rephrase "LLM, TextChecker, Storage-direct-access exemptions" to "LLM, TextChecker, Storage-direct, Recorder-save-closure".
  - [ ] ##### SubTask 3.1.5: Update "Consequences → Negative" — remove the bullet if it referenced the dual-port shape.
- [ ] #### Task 3.2: Doc touchups (0 SP, bundled with 3.1)
  - [ ] ##### SubTask 3.2.1: Verify `docs/system/llm_processing.md` references the recorder contract, not the deleted port. Update if it mentions `LlmMessageRepository` as a port trait.
- [ ] #### Task 3.3: Run validation (0 SP, blocking completion)
  - [ ] ##### SubTask 3.3.1: `cd chronicler_engine && python build.py` — must pass (fmt + clippy + tests + coverage). Verify no arch-lint regression in `tests/infrastructure/guardrails/layers.rs::check_application_storage_direct`.

## Test Plan

- `python build.py` from `chronicler_engine/` is the primary acceptance gate.
- `tests/infrastructure/guardrails/layers.rs` already enforces `application/` not importing `Storage` except in 5 grandfathered files — must remain green. `query_handlers.rs` is already compliant via `ctx.storage` field access (no `use Storage` statement), so removing the trait doesn't add new violations.
- New unit test in `llm_recorder_tests.rs`: closure returning `Err(EngineError::Storage(...))` propagates from `LlmCallRecorder::complete` — replaces coverage previously provided by `RecordingForensics::with_next_save_error`.
- Coverage parity check: every existing test path that asserted on `SpyForensics::save_count` must have a counterpart asserting on `Arc<AtomicUsize>::load`; every `RecordingForensics::with_list_response` test path must move to direct Storage testing (already used Storage direct for reads in `query_handlers`).
- Manual sanity: integration test `tests/integration/application/wiring.rs` already asserts on `storage.list_latest_llm_messages(...)` end-state after recorder calls — verifies the closure wiring reaches real storage.

## Assumptions

- `LlmMessage` struct stays in `application/` (not moved to `domain/model/`) — minimizes import churn, same architectural layer, just different module name.
- `SaveLlmMessageFn` type alias exported from `application::llm_message` — callers don't restate the closure signature.
- `Arc<dyn Fn + Send + Sync>` matches existing `Arc<dyn LlmMessageRepository>` dispatch cost — no perf regression.
- `LlmCallRecorder::complete` and `LlmCallRecorder::provider` signatures unchanged. Only constructor signature changes.
- ADR-027 status stays `Accepted` — revision captured in History block per project convention. No new ADR created (no new decision; revising an old one).
- `LlmProvider` port unchanged — it uses `LlmMessage` *type* via `to_message()`, no trait dependency.
- `query_handlers::list_latest_llm_messages` already uses `ctx.storage.*` direct access — no change needed, already arch-lint-compliant.
- Closure types cost no more than trait dispatch at runtime — both are `Arc<dyn ...>` with vtable lookup.
- Delete `recording_forensics_tests.rs` (it tests the deleted struct); new closure tests live in `llm_recorder_tests.rs` and the closure factory module itself.
]<]minimax[>[
