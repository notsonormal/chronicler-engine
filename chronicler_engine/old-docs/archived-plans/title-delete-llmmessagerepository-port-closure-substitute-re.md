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
- [ ] #### Task 1.3: Delete trait files; drop forwarding trait impl on Storage (1 SP)
  - [ ] ##### SubTask 1.3.1: Edit `adapters/driven/storage/backend/llm_messages.rs`: **delete the `impl LlmMessageRepository for Storage { ... }` forwarding block only.** The inherent `impl Storage { pub fn save_llm_message; pub fn list_latest_llm_messages }` block already exists above it — leave it untouched. Also drop the `use crate::application::ports::llm_message_repository::LlmMessageRepository;` import; keep the `LlmMessage` import (now redirected to `application::llm_message::LlmMessage`).
  - [ ] ##### SubTask 1.3.2: Delete `src/application/ports/llm_message_repository.rs` (now only contains trait + doc anchor; trait gone, doc anchor moves to `llm_message.rs`).
  - [ ] ##### SubTask 1.3.3: Delete `src/application/ports/llm_message_repository_tests.rs`.
  - [ ] ##### SubTask 1.3.4: Update `src/application/ports/mod.rs` — remove `pub mod llm_message_repository;` and `#[cfg(test)] mod llm_message_repository_tests;`.

### Phase 2: Test support migration (5 SP)

- [ ] #### Task 2.1: Build closure factories in `test_support/mod.rs` (1 SP)
  - [ ] ##### SubTask 2.1.1: Add `pub fn make_noop_save_fn() -> SaveLlmMessageFn` — closure returning `Ok(())`.
  - [ ] ##### SubTask 2.1.2: ~~Add `RecordingSaveSpy` struct~~ — **DROPPED**: tests assert via real `Storage::list_latest_llm_messages` instead of a spy struct. See Phase 2A for migration details.
  - [ ] ##### SubTask 2.1.3: ~~Add `make_recording_save_fn`~~ — **DROPPED** (same reason).
  - [ ] ##### SubTask 2.1.4: Delete `src/test_support/noop_forensics.rs`, `src/test_support/recording_forensics.rs`, `src/test_support/recording_forensics_tests.rs`.
  - [ ] ##### SubTask 2.1.5: Update `src/test_support/mod.rs` — remove module decls and `pub use noop_forensics::*; pub use recording_forensics::*;`. Add `pub use` for `make_noop_save_fn`, `make_test_recorder`, `make_test_recorder_with_storage` (defined in `llm_recorder_save_seam.rs` or inline in `mod.rs`).
  - [ ] ##### SubTask 2.1.6: Keep `make_test_recorder` (uses `make_noop_save_fn`) and `make_test_recorder_with_storage` (wraps `Arc<Storage>::save_llm_message` in closure). ~~`make_spy_recorder`~~ — **DROPPED** (no spy struct).
- [ ] #### Task 2.2: Migrate unit test sites (3 SP)
  - [ ] ##### SubTask 2.2.1: Migrate `src/application/**/*tests.rs` (9 files: `llm_recorder_tests`, `action_pipeline/{actions,pipeline,retry}_tests`, `agents/quantifier/{agent,orchestration}_tests`, `agents/registry_tests`, `is_generating_invariant_tests`, `query_handlers_tests`). For tests that asserted via `SpyForensics::save_count()` / `RecordingForensics::last_saved_message()` → use `make_test_recorder_with_storage(provider, Arc::clone(&storage))` + assert on `storage.list_latest_llm_messages(N)` (returns `Vec<LlmMessage>`). For error-injection (`with_next_save_error`) → inline closure `Arc::new(|_| Err(...))` — and note `complete_propagates_closure_save_error` test already covers this case, so duplicate tests can be deleted. `NoopForensics` → `make_noop_save_fn()`. (1 SP)
  - [ ] ##### SubTask 2.2.2: Migrate `src/adapters/**/*tests.rs` (3 files: `llm/providers/mock_tests`, `storage/backend/llm_messages_tests`, `storage/mappers/llm_message_tests`). Path-only updates for `llm_messages_tests` + `llm_message_tests` (they import `LlmMessage` type, not trait). `mock_tests.rs` uses `Arc<dyn LlmMessageRepository>` as a type annotation around `Arc::new(Storage::new_in_memory())` and calls `.save_llm_message` / `.list_latest_llm_messages` directly — switch the annotation to `Arc<Storage>` (inherent methods now). (1 SP)
  - [ ] ##### SubTask 2.2.3: **Production wiring** — update `src/bootstrap/llm_factory.rs`: drop `use ...LlmMessageRepository` import (line 11); replace `let forensics: Arc<dyn LlmMessageRepository> = storage;` (line 37-38) with a `SaveLlmMessageFn` closure wrapping `Arc<Storage>::save_llm_message` (capture storage by move/clone, call `(*storage).save_llm_message(msg)`). `LlmCallRecorder::new(provider, save_fn)` takes the closure. Import `SaveLlmMessageFn` from `crate::application::llm_message`. **Test wiring** — `src/bootstrap/wiring.rs:84` calls `make_test_recorder_with_storage`; once the helper is rebuilt (2.1.6) wiring needs no other change. Verify type inference still resolves. Note: `src/application/agents/quantifier/agent.rs:54-57` is a **test** (`NoopForensics` used inline), not production — migrate it in 2.2.1. `src/application/agents/registry.rs` does NOT touch `LlmMessageRepository` (passes already-built `Arc<LlmCallRecorder>`); no change needed there. The `Option<Arc<Storage>>` on agent constructors is the unrelated G1-B `persistence_gate` carve-out, not forensics. (1 SP)
- [ ] #### Task 2.3: Migrate integration test sites (1 SP)
  - [ ] ##### SubTask 2.3.1: Migrate `tests/integration/**` (8 files: `application/{action_pipeline/{pipeline,retry}, game_service, lifecycle, wiring}`, `flow/{arrival_persistence, retry_event, retry_main, sequence}`) + `tests/integration/mod.rs` + `tests/test_utils/mod.rs` — same mechanical replacement as unit tests. `tests/helpers/sqlite_test_app_builder.rs` and `tests/infrastructure/invariant_contract.rs` only call `make_test_recorder` / `make_test_recorder_with_storage` — covered transitively by helper rebuild in 2.1.6, no direct edits expected (verify, don't assume migration).

### Phase 3: ADR + docs + validation (3 SP)

- [ ] #### Task 3.1: Revise ADR-027 (1 SP)
  - [ ] ##### SubTask 3.1.1: Remove `LlmMessageRepository` row from "Accepted Port Traits" table.
  - [ ] ##### SubTask 3.1.2: Rewrite "Phantom Port Heuristic" section. New rule: a port is justified when **(consumer in core AND producer is adapter)** *and* **test seam requires capabilities not satisfied by an existing mechanism** (Backend enum, direct methods) *and* **trait ceremony is proportionate** (~few methods, focused concern). Otherwise the concrete struct + closure or concrete struct + Backend enum covers the seam with less ceremony.
  - [ ] ##### SubTask 3.1.3: Add History entry dated 2026-07-10 (latest existing entry is 2026-07-09; new entry must be later and unique): "Removed `LlmMessageRepository` port. `LlmCallRecorder` now uses `SaveLlmMessageFn = Arc<dyn Fn(&LlmMessage) -> Result<(), EngineError> + Send + Sync>` closure. Test support types (NoopForensics, SpyForensics, RecordingForensics) replaced with closure factories. Restores symmetry with rejected `StateRepository` decision. Phantom-port heuristic rewritten as two-clause rule."
  - [ ] ##### SubTask 3.1.4: Update "Consequences → Positive" — remove or rephrase "LLM, TextChecker, Storage-direct-access exemptions" to "LLM, TextChecker, Storage-direct, Recorder-save-closure".
  - [ ] ##### SubTask 3.1.5: Update "Consequences → Negative" — remove the bullet if it referenced the dual-port shape.
- [ ] #### Task 3.2: Diataxis doc touchups (2 SP)
  - [ ] ##### SubTask 3.2.1: Verify `docs/system/llm_processing.md` references the recorder contract, not the deleted port. Update if it mentions `LlmMessageRepository` as a port trait. (Grepping the diataxis tree found no mention there, so this may be a no-op — verify and move on.)
  - [ ] ##### SubTask 3.2.2: `docs/diataxis/reference/narrative/narration_system.md` line 80 — rewrite step 4 from "Saves the record through the `LlmMessageRepository` port." to describe the closure seam: `LlmCallRecorder` invokes its `SaveLlmMessageFn` closure (wired in `bootstrap::llm_factory` to `Storage::save_llm_message`).`
  - [ ] ##### SubTask 3.2.3: `docs/diataxis/reference/coding_standards/testing.md` line 28 — replace the "LLM-call test helpers" section. New text: production runs write to the SQLite `llm_messages` table via `LlmCallRecorder` + its `SaveLlmMessageFn` closure (wired to `Storage::save_llm_message` in `bootstrap::llm_factory`); the `/fragment/llm-messages` UI tab stays as the runtime inspection surface. The `RecordingForensics` spy is gone; tests now use the `RecordingSaveSpy` returned by `make_recording_save_fn()` from `src/test_support/mod.rs`, exposing `save_call_count()` and `last_saved_message()`.
  - [ ] ##### SubTask 3.2.4: `docs/diataxis/reference/coding_standards/unit_test_standards.md` — update:
    - Lines 106-111 (Pattern 4 code block): replace `use crate::test_support::recording_forensics::RecordingForensics;` + `let forensics = Arc::new(RecordingForensics::new());` + `LlmCallRecorder::new(provider, forensics.clone())` with `use crate::test_support::make_recording_save_fn;` + `let (save_fn, spy) = make_recording_save_fn();` + `LlmCallRecorder::new(provider, save_fn)`. Update the assertion hint to `spy.save_call_count()` / `spy.last_saved_message()`.
    - Line 128 drift note: replace `SpyForensics` / `make_spy_recorder` in `src/test_support/noop_forensics.rs` with `make_recording_save_fn()` + `RecordingSaveSpy` in `src/test_support/mod.rs`. Drop the "use SpyForensics only when..." distinction — the spy builder covers both cases.
    - Line 129 drift note: delete — `src/application/ports/llm_message_repository_tests.rs` no longer exists and the trait polymorphism test concept is moot.
    - Line 141 code block comment `// from noop_forensics` → `// from test_support/mod.rs`.
    - Line 420 (Cross-cutting D — Sealed-trait polymorphism tests): `LlmMessageRepository` is no longer a trait-port example; keep the `TextChecker` example and the trait-contract pattern, but drop the `LlmMessageRepository` parenthetical.
    - Line 440 Used-in list: remove the `src/application/ports/llm_message_repository_tests.rs` entry; keep the `text_checker_tests.rs` entry.
  - [ ] ##### SubTask 3.2.5: `docs/diataxis/reference/coding_standards/integration_test_standards.md` line 60 — rewrite the wiring-test drift note. The regression it catches now takes the form: prod factory wires `SaveLlmMessageFn` to a no-op closure instead of `Storage::save_llm_message`. Keep the test's role (drive `LlmCallRecorder::complete` and assert the message landed in real `Storage`); drop `NoopForensics` reference.
  - [ ] ##### SubTask 3.2.6: `docs/diataxis/explanation/architecture.md` — line 35 remove `msgs["LlmMessageRepository"]` node from the mermaid `ports` subgraph (collapse to `LlmProvider` + `TextChecker`); remove the `app -. uses .-> msgs` and `msgs -.-> storage` edges. Line 68 change "Three port traits are accepted (`LlmProvider`, `LlmMessageRepository`, `TextChecker`)" → "Two port traits are accepted (`LlmProvider`, `TextChecker`); LLM message persistence runs through a `SaveLlmMessageFn` closure seam (wired to concrete `Storage` in bootstrap)." Rest of the paragraph (Storage-direct exemption, Backend enum dispatch) stays.
  - [ ] ##### SubTask 3.2.7: `docs/diataxis/how-to/debugging.md` line 90 — replace `RecordingForensics` reference with `RecordingSaveSpy` (returned by `make_recording_save_fn` in `src/test_support/mod.rs`). Keep the framing: test-writing fixture, not a runtime-debugging tool.
  - [ ] ##### SubTask 3.2.8: Re-run `python scripts/validate_docs.py` — diataxis linter does not currently check for stale source-path references in doc prose, so manual review of the 6 docs above is the only gate. No structural validator changes expected.
- [ ] #### Task 3.3: Run validation (0 SP, blocking completion)
  - [ ] ##### SubTask 3.3.1: `cd chronicler_engine && python build.py` — must pass (fmt + clippy + tests + coverage). Verify no arch-lint regression in `tests/infrastructure/guardrails/layers.rs::check_application_storage_direct`.

## Test Plan

- `python build.py` from `chronicler_engine/` is the primary acceptance gate.
- `tests/infrastructure/guardrails/layers.rs` already enforces `application/` not importing `Storage` except in 5 grandfathered files — must remain green. `query_handlers.rs` is already compliant via `ctx.storage` field access (no `use Storage` statement), so removing the trait doesn't add new violations.
- New unit test in `llm_recorder_tests.rs`: closure returning `Err(EngineError::Storage(...))` propagates from `LlmCallRecorder::complete` — replaces coverage previously provided by `RecordingForensics::with_next_save_error`.
- Coverage parity check: every existing test path that asserted on `SpyForensics::save_count` must have a counterpart asserting on `Arc<AtomicUsize>::load`; every `RecordingForensics::with_list_response` test path must move to direct Storage testing (already used Storage direct for reads in `query_handlers`).
- Manual sanity: unit test `src/bootstrap/llm_factory_tests.rs::mock_backend_recorder_persists_forensics_to_storage` asserts on `storage.list_latest_llm_messages(...)` end-state after `get_llm_recorder_for` + recorder call — verifies the closure wiring reaches real storage. (Migrated from `tests/integration/application/wiring.rs` to eliminate Pattern-1 drift — that test was a misclassified unit test of the factory.)

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
