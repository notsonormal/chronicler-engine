# Implementation Plan: Abstraction Anti-Pattern Fixes (Corrected)

**Status:** Phases 1–3 implemented (commit `b66455c`), Phases 4–7 pending
**Source:** Derived from `docs/plans/abstraction-fixes-plan.md`
**Created:** 2026-06-26
**Implemented:** 2026-06-27 (Phases 1–3)
**Scope:** All 7 phases, single PR/commit for Phases 1–3; remaining phases deferred
**Decisions:** 15 clarified decisions applied after investigation + improve-ai-plan review

---

## Summary

Execute the existing `docs/plans/abstraction-fixes-plan.md` as a single PR/commit with **4 scope corrections** + **11 decision locks** applied after investigation revealed the source investigation report (`docs/reviews/abstraction-antipatterns-summary.md`) had 4 misclassifications and the plan had unresolved decisions.

A6 `TemplateVars` collapse deferred per earlier user decision — 6+ callers + struct field ripple too large for this plan.

### Corrections from investigation

1. **Phase 5 Tasks 5.1, 5.2, 5.3 DROPPED** — `error_return` returning `Ok` + `GenerationStatus::Error` as error channel is DELIBERATE arch (commit `8e4acf5 "Unify error model onto GenerationStatus on state"`), documented in `system/game_flow.md` "Error Model" section + `architecture/system.md:49` + CHANGELOG line 272. Investigation did not consult these. Reversing requires explicit ADR — out of scope.

2. **Phase 6 Task 6.3 (B7 retry re-impl) DROPPED** — STALE. `architecture/system.md:52` + CHANGELOG line 90 confirm retry already delegates to `ActionPipeline::phase_trigger_continuation()` + `run_from_input()`. Not re-implementing pipeline.

3. **Phase 6 Task 6.3 (B8 `ArrivalTaskContext`) REFRAMED** — Deliberately extracted in prior refactor (CHANGELOG line 98), documented at `architecture/system.md:209`. Not an antipattern. Could pipeline-route as a future arch change but not a "fix".

4. **Phase 2 Task 2.1 (B2 `ActionOutcome::Error`) REFRAMED** — Not "side-channel antipattern." Variant retained for match exhaustiveness per `game_flow.md`. Fix is dead-arm cleanup, not error-model fix.

5. **Phase 3 Task 3.3 (A6 `TemplateVars`) DEFERRED** — 6+ callers across `action_processing.rs:172`, `bootstrap/state.rs:28`, `bootstrap/scenario.rs:22`, `assembler.rs:90/117/169`, `context.rs:124`, `quantifier/prompt.rs:21`, `types.rs:36`. Plan said "1 field, 1 consumer" — understated. Ripples to struct field `assembler.rs:169 template_vars: &TemplateVars` + `types.rs:36 pub template_vars: TemplateVars`. Separate sub-plan needed.

---

## Decision locks (resolved during review)

| # | Task | Decision |
|---|------|----------|
| 1 | 5.4 (Operation enum removal, test injection replacement) | **Option A** — Replace `HashMap<Operation, TestOverride>` with `HashMap<&'static str, TestOverride>` keyed by method name (e.g., `"load_games"`). `Backend::Test` stays as decorator. All existing test call sites rename `Operation::LoadGames` → `"load_games"`. Storage methods no longer take `Operation` param. |
| 2 | 6.4 (flatten `DefaultApplicationService` ↔ `GameLifecycleService`) | **Option B (inline)** — Move 13 methods from `GameLifecycleService` into `DefaultApplicationService`. Delete `game_lifecycle.rs` file. Update `architecture/system.md:43` (GameLifecycleService references gone). |
| 3 | 6.6 (`MockBackend` flag-bag replacement) | **Option C (minimal prune + builders)** — Audit AtomicBool flags actually set by tests. Keep only used ones. Add `MockBackend::succeeding()` / `::failing()` / `::empty_response()` builders. Keep ~100 call sites largely unchanged. Removes flag-bag smell by pruning + builders, NOT full restructure. |
| 4 | 7.2 (Message/Swipe domain model fix) | **Option A (accessor)** — Remove mirrored `text`, `location_header`, `event_header`, `snapshot_id` fields from `Message` struct. Make them accessor methods reading from `swipes[active_swipe_index]`. DB schema unchanged (already normalized per ADR-017). |

---

## Test guarantees

1. **Task 5.4**: Add unit test that override-Map with unknown method-name key causes explicit failure (not silent pass-through to base).
2. **Task 6.4**: Existing integration tests in `tests/integration/lifecycle.rs` (uses `MockBackend`) become regression test for `create_game` / `switch_game` / `delete_game`. Confirm they stay green.
3. **Task 7.2**: Add snapshot round-trip test for Message+Swipes hydration. Existing `test_message_from_db` stays as regression. Add round-trip from `GameStateSnapshot` to verify `set_snapshot_id` rewiring post-mirror-removal.

---

## Glue tasks

- **End of Phase 4**: Regenerate `chronicler_engine/AGENTS.md` AUTO-STRUCTURE block (per `<!-- AUTO-STRUCTURE START -->` marker) after module splits.
- **End of Phase 5 Task 5.4**: Remove `lib.rs:9` clippy FIXME comment ("clippy::unimplemented, // FIXME: Re-enable after Phase 1 unimplemented!() calls are removed from Backend::Test match arms").
- **End of Phase 6 Task 6.4**: Update `architecture/system.md:43, 52` — `GameLifecycleService` references removed, retry delegation note preserved.
- **End of Phase 6 Task 6.1**: Update `docs/system/action_pipeline.md` with PipelineInputs + spawn_pipeline_task helper contracts (NOT state-machine rewrite).
- **End of Phase 7 Task 7.1**: Update `docs/system/message_model.md` with accessor-pattern message domain model.

---

## Task 7.2 explicit caveat (state-diagnosis)

`Message::set_snapshot_id(&mut self, sid: Option<u64>)` at `model/message.rs:117` **currently writes only the mirrored field** (`self.snapshot_id = sid`), NOT the swipe's `snapshot_id`. After Task 7.2 removes mirrored fields, `set_snapshot_id` MUST be rewired to write `swipes[self.active_swipe_index].snapshot_id = sid` instead. Behavior-preserving but easy to miss. Round-trip hydration test guarantee (above) covers this.

---

## Phase 1: Pre-work — stale file cleanup

### Task 1.1: Delete `agent.rs_temp`

- **Action:** Delete `chronicler_engine/src/model/agent.rs_temp` (238 lines, stale duplicate of `agent.rs`, no code module references it).
- **Verification:** `cargo check` passes. Grep for `agent.rs_temp` returns only doc references in other plans (not addressed here).
- **Files:** `chronicler_engine/src/model/agent.rs_temp` (delete)

### Task 1.2: Verify pre-work

- `cargo build` passes
- `python build.py` passes (fmt + clippy + tests + coverage)

---

## Phase 2: Surgical type/code collapses (all single-enum/struct refactor)

### Task 2.1: Reframe + delete `ActionOutcome::Error` variant (B2 — reframed)

- **Reframing:** NOT "side-channel antipattern." Variant retained for match exhaustiveness per `game_flow.md` "Error Model" section: "`ActionOutcome::Error` variant is retained for exhaustiveness but never constructed in production code."
- **Action:** Delete the `Error(String)` variant from `ActionOutcome` enum at `application/action_pipeline/pipeline.rs`. Remove 2 dead match arms in `retry.rs` that handle `ActionOutcome::Error` (currently `#[allow(dead_code)]`).
- **Files:**
  - `application/action_pipeline/pipeline.rs` (delete variant + 0 match arms in this file — variant decl only)
  - `application/action_pipeline/retry.rs` (delete 2 dead match arms)
- **Verify:** `python build.py` passes. Only 2 match arms deleted, zero call-site changes needed.

### Task 2.2: Delete `PromptLayer::Phi` (C7)

- **Action:** Remove `Phi` variant from `PromptLayer` enum at `narrative/prompt/types.rs`. Only ref is `types_tests.rs:12` discriminant test.
- **Files:**
  - `narrative/prompt/types.rs` (delete variant)
  - `narrative/prompt/types_tests.rs` (delete discriminant test entry)
- **Verify:** `cargo check` passes.

### Task 2.3: Remove `narrate_continuation` from `LlmBackend` trait (C1)

- **Action:** Remove `narrate_continuation` method from `LlmBackend` trait at `narrative/llm/backend.rs`. Implementations at `deepseek.rs`, `openrouter.rs`, `ollama.rs`, `mock.rs` deleted. Test mocks (`mock_tests.rs`, `deepseek_tests.rs`, `orchestration_tests.rs`, `retry_tests.rs` helper fn) deleted. Zero prod callers.
- **Files:** ~9 files (backend trait + 4 impls + 4 test files)
- **Verify:** `python build.py` passes (specifically integration tests + retry tests).

### Task 2.4: Remove `_player_name` from `process_actions` (B1)

- **Action:** Remove `_player_name: String` parameter at `actions.rs:15`. Remove `player_name` field from `#[instrument]` decorator at `actions.rs:10`. Update callers in `actions_tests.rs`, `application_service.rs`, `game_service.rs`.
- **Note:** `retry_tests.rs:43, 355` use `player_name` variable name but refer to `state.player.sheet.name` — unrelated, NOT touched.
- **Files:** 4 files (`actions.rs` + 3 callers)
- **Verify:** `python build.py` passes.

### Task 2.5: Remove `_all_rooms` + `extract_movement_from_text` (C3)

- **Action:** Delete `_all_rooms: &[Room]` (1 call site at `parser.rs:73`) and unused `extract_movement_from_text` function at `parser.rs:219`. Update ~14 tests in `parser_tests.rs`. Update `orchestration.rs` and `mod.rs` if necessary.
- **Files:** 4 files (`parser.rs`, `parser_tests.rs`, `orchestration.rs`, `mod.rs`)
- **Verify:** All 14 test sites updated. `python build.py` passes.

### Task 2.6: Remove `_exits` from `GenerationViewModel::new` (D6)

- **Action:** Remove `_exits: &[String]` parameter at `view_models.rs:157`. Update 5 test callers in `templates_tests.rs:454-502` and 1 prod caller in `renderers.rs:86` (passes `&[]`).
- **Files:** 3 files (`view_models.rs`, `templates_tests.rs`, `renderers.rs`)
- **Verify:** `python build.py` passes.

### Task 2.7: Remove `sanitize_for_prompt` (C10)

- **Action:** Delete `sanitize_for_prompt` function (single caller at `assembler.rs:379`). Update `sanitize.rs`, `sanitize_tests.rs`, `assembler.rs`, `narrative/prompt/mod.rs`, `assembler_tests.rs`.
- **Files:** 5 files
- **Verify:** `python build.py` passes.

### Task 2.8: Remove `NarratorAgent` from registry (C9)

- **Action:** Remove `NarratorAgent` struct at `registry.rs:90`. Remove the `"narrator"` arm at `registry.rs:57` (returns `NoOp`). Update `registry_tests.rs`.
- **Files:** 2 files (`registry.rs`, `registry_tests.rs`)
- **Verify:** `python build.py` passes.

### Task 2.9: Remove `Operation::LoadSwipesForMessages` piggyback (D5)

- **Action:** Refactor `Operation::LoadSwipesForMessages` at `swipes.rs:~156` to not piggyback. Storage method takes its own Operation param (until Phase 5.4 removes Operation enum entirely) OR uses separate code path.
- **Files:** 2 files (`storage/backend/core.rs`, `storage/backend/swipes.rs`)
- **Verify:** `python build.py` passes.

### Phase 2 verification

- `python build.py` passes
- All 8 tasks independently green

---

## Phase 3: Type collapses (single-variant enums, single-impl traits, mixed struct refactors)

### Task 3.1: Collapse `StatePatch` enum → `Scene` struct (A1)

- **Action:** `StatePatch` enum at `model/agent.rs:96` has single `Scene { ... }` variant (3 destructures at lines 43, 48, 78). Replace with `pub struct Scene { ... }` or `Scene` newtype. Update 3 destructures.
- **Files:** 3-4 files (`model/agent.rs`, `game_service.rs`, `narrative/agents/mod.rs`, optional `action_pipeline/pipeline.rs`)
- **Verify:** `python build.py` passes.

### Task 3.2: Rewrite `TriggerRequirement` enum (A2)

- **Action:** Simplify `TriggerRequirement` enum at `model/trigger.rs`. Update `test_support/fixtures.rs`, `engine/trigger_eval.rs`, `trigger_eval_tests.rs`, `bootstrap/validate_tests.rs`, and others (~6 files).
- **Files:** ~6 files
- **Verify:** Trigger eval tests pass + `python build.py` passes.

### Task 3.3: ~~Collapse `Confidence` / `QuantifierConfidence`~~ DEFERRED

- **Status:** A6 `TemplateVars` deferred above (4 corrections #5). Similarly, A3's heavy usage (~80 refs across tests + prod code including `orchestration.rs`, `parser.rs`, `phases.rs` + bidirectional From impls) makes this risky for single-PR scope.
- **Decision:** DEFER A3 `Confidence`/`QuantifierConfidence` collapse to follow-up plan alongside A6. Both have similar rippling-struct-field pattern.

### Task 3.4: Collapse `PromptAssembler` trait → `LayeredPromptAssembler` struct (C4)

- **Action:** `PromptAssembler` trait at `assembler.rs:23` has single impl `LayeredPromptAssembler` at `assembler.rs:52`. Remove trait, keep struct. Callers at `assembler_tests.rs`, `game_service.rs`, `init_game.rs`, `pipeline_tests.rs`, `actions_tests.rs` updated.
- **Files:** ~6 files (some overlap with A1 callers)
- **Verify:** `python build.py` passes.

### Task 3.5: Inline `preprocess_user_text` trait default (C8)

- **Action:** `preprocess_user_text` trait default at `backend.rs` duplicated/used at `ollama.rs` + `ollama_tests.rs`. Inline implementation, remove default from trait.
- **Files:** 3 files (`backend.rs`, `ollama.rs`, `ollama_tests.rs`)
- **Verify:** LLM backend tests pass.

### Task 3.6: Remove zero-field `QueryHandlers` (B11)

- **Action:** `QueryHandlers` at `query_handlers.rs:11` is zero-field struct. Remove struct. Update ~10 test call sites in `query_handlers_tests.rs` + 2 prod references in `application_service.rs`. Update `mod.rs`.
- **Files:** 4 files
- **Verify:** `python build.py` passes.

### Phase 3 verification

- `python build.py` passes

---

## Phase 4: Module splits (with re-export shield — Issue 3, 5 decision)

### Shield principle

`state.rs`, `misc.rs`, `renderers.rs` keep `pub use self::submodule::*;` re-exports after split. Existing `crate::model::state::{GenerationStatus, MovementState, ...}` and `crate::server::fragments::misc::{...}` import paths preserved. Blast radius ~0 caller ripples initially. Follow-up Task 4.1b gradual-migrate high-churn callers over time.

### Task 4.1: Split `model/state.rs` (A7) + keep re-exports

- **Action:** Move 11 unrelated types out of `model/state.rs` into new submodule files:
  - `model/state/generation_status.rs` (`GenerationStatus`)
  - `model/state/movement.rs` (`MovementState`, `MessageType`)
  - `model/state/scene.rs` (`Scene`)
  - `model/state/narrative_state.rs` (`NarrativeState`, `NarrativeSnapshot`)
  - `model/state/npc_encounter_log.rs` (if separate — see Task 4.4 for alternative)
- Re-export via `pub use` in `state.rs` top-level (and new `state/mod.rs`).
- **Blast radius:** ~30-40 files import `crate::model::state::{...}` — UNCHANGED due to re-exports.
- **Files:** New submodule files + `state.rs` → `state/mod.rs`
- **Verify:** `cargo check` passes. `python build.py` passes. No test failures.

### Task 4.2: Split `server/fragments/misc.rs` (A7) + keep re-exports

- **Action:** Move unrelated fragment helpers out of `misc.rs` into:
  - `text_check.rs` (+ define `CheckTextForm` per D7 deferred item)
  - `swipe.rs`
  - `game_control.rs`
  - `response.rs`
- Re-export via `pub use` in `misc.rs`.
- **Files:** New submodule files + `misc.rs` → `misc/mod.rs`
- **Verify:** `python build.py` passes.

### Task 4.3: Split `server/fragments/renderers.rs` (A7) + keep re-exports

- **Action:** Move renderer types out of `renderers.rs` into:
  - `fragment_renderers.rs`
- Re-export via `pub use`.
- **Files:** New + `renderers.rs` → `renderers/mod.rs`
- **Verify:** `python build.py` passes.

### Task 4.4: Move `NpcEncounterLog` CRUD to inherent impl (Issue 6 decision)

- **Action:** Move 5 CRUD methods (`increment_times_met`, `mark_trigger_fired`, `set_currently_meeting`, `get_times_met`, `is_trigger_fired`) from `engine/trigger_eval.rs:78-114` to **inherent impl on `NpcEncounterLog`** in `model/trigger.rs` (not new file). Methods take `&mut self` / `&self` instead of free `fn(&mut NpcEncounterLog, ...)`. Callers update: `log.increment_times_met(npc_id)` instead of `increment_times_met(log, npc_id)`.
- **Rationale (Issue 6):** Methods are pure domain logic on the struct, not DB-bound. Inherent impl is simpler than new module file. Cohesive.
- **Files:** `engine/trigger_eval.rs` (delete methods) + `model/trigger.rs` (add inherent impl)
- **Verify:** `python build.py` passes (esp. trigger eval tests).

### Task 4.5: Regenerate AGENTS.md AUTO-STRUCTURE

- **Action:** Run whatever generator produces `<!-- AUTO-STRUCTURE START -->` block in `chronicler_engine/AGENTS.md`.
- **Files:** `chronicler_engine/AGENTS.md`
- **Verify:** Diff shows new module files listed.

### Phase 4 verification

- `python build.py` passes (full suite incl. fmt + clippy)
- AGENTS.md Auto-Structure block updated

---

## Phase 5: Storage backend refactor (D findings; error-model findings DROPPED per Corrections #1)

### Task 5.4: Replace `Operation` enum with stringly-keyed override HashMap (D4, D12)

- **Action:** Remove `Operation` enum from `storage/backend/core.rs:69`. Update `Backend::Test { base, overrides: Arc<Mutex<HashMap<&'static str, TestOverride>>> }` keyed by method name (e.g., `"load_games"`). Update all storage methods that take `Operation::Foo` param — remove the param. Update ~12 per-resource files' `Backend::Test` dead arms (currently `unreachable!()` / `unimplemented!()`).
- **Decision 1 (above):** Option A, NOT trait-object test double (Option C deferred).
- **Files:** ~12+ per-resource files + `core.rs` + all storage test files
- **Verify:** `python build.py` passes. All storage tests still inject failures via `.with_failure("method_name", override_)`.

### Task 5.4b: Add observability check for typo'd override keys (Issue 13)

- **Action:** Implement Drop trait (or post-test assertion) on `Backend::Test` override map that warns or fails when HashMap has unconsumed keys at end of test. Prevents silent-test-pass-through bugs from method-name typos.
- **Rationale (Issue 13):** Stringly-typed keys lose compile-time exhaustiveness. Must compensate with runtime observability.
- **Files:** `storage/backend/core.rs` + tests that use Backend::Test
- **Verify:** Typo'd override key triggers warning or assertion failure in test.

### Task 5.5: Refactor `with_backend_mut` signature to `Option<u64>` (D3)

- **Action:** Change `with_backend_mut(Operation::Foo, |backend, _game_id| ...)` → `with_backend_mut(game_id: Option<u64>, |backend| ...)` (no Operation param post-5.4). ~32 callers pass `None`. ~7 callers pass `Some(id)`.
- **Decision (Issue 14):** Option<u64> single-method, NOT two-method split (Option B deferred as too much API surface).
- **Files:** ~40 call sites across per-resource files
- **Verify:** `python build.py` passes. Call sites self-document scope.

### Task 5.6: Dedupe `save_message` (C11)

- **Action:** Find duplicated `save_message` logic across storage files. Consolidate into single implementation.
- **Files:** TBD during implementation (grep `save_message` duplicates)
- **Verify:** `python build.py` passes.

### End-of-Phase-5 Glue Task: Remove lib.rs clippy FIXME

- **Action:** Remove `lib.rs:9` comment: `// clippy::unimplemented, // FIXME: Re-enable after Phase 1 unimplemented!() calls are removed from Backend::Test match arms`. Phase 5.4 removed the dead arms.
- **Files:** `chronicler_engine/src/lib.rs:9`
- **Verify:** `cargo clippy` passes without the FIXME comment.

### Phase 5 verification

- `python build.py` passes
- Storage tests inject failures correctly with new string-keyed mechanism

---

## Phase 6: Application pipeline restructure (constrained — Issue 9)

### Phase 6.1 spec constraint (Issue 9 decision)

- **Constrained to:** (a) `PipelineInputs<'a>` struct definition contract, (b) `spawn_pipeline_task` helper contract.
- **NOT in scope:** State-machine / `PipelineStep` enum rewrite of `run_from_input`. Current method-blob is documented + works + tested. State-machine is scope creep.
- **Read-first:** Before writing any code, read `architecture/system.md:49-52` (documents current `ActionPipeline<'a, B: ActionPipelineBackend>` contract) + `system/game_flow.md` (documents phase_* methods + handle_cancellation). Spec must propose DELTA from current documented arch, not contradiction.

### Task 6.1: Write Phase 6 spec

- **Action:** Write `docs/system/action_pipeline.md` UPDATE (not new spec) covering: `PipelineInputs<'a>` definition, `spawn_pipeline_task` helper. NO state-machine proposal.
- **Files:** `docs/system/action_pipeline.md`
- **Verify:** Spec reviewed against `architecture/system.md:49-52` consistency.

### Task 6.2: Introduce `PipelineInputs<'a>` struct (B5, B6)

- **Action:** Replace `too_many_arguments` params at `phases.rs:53` + `phases.rs:310` with `PipelineInputs<'a>` struct. Update call sites.
- **Files:** `phases.rs` + callers
- **Verify:** `python build.py` passes. Clippy `too_many_arguments` warning gone for these fn signatures.

### Task 6.4: Inline `GameLifecycleService` into `DefaultApplicationService` (B4 — Decision 2)

- **Action:** Move 13 methods from `application/game_lifecycle.rs::GameLifecycleService` into `application/application_service.rs::DefaultApplicationService`. Delete `game_lifecycle.rs` file. Delete 14 passthrough methods.
- **Decision 2 (above):** Option B (inline). NOT Option A (expose field).
- **Files:** `application/game_lifecycle.rs` (delete), `application/application_service.rs` (grow by 13 methods), `application/mod.rs` (re-export updates), callers of GameLifecycleService
- **Verify:** `tests/integration/lifecycle.rs` integration tests stay green (covers create_game / switch_game / delete_game per Test Guarantee #2).

### Task 6.5: Extract `spawn_pipeline_task` helper (B5, B6 — init_game.rs:223)

- **Action:** Extract `spawn_blocking` call + cancellation setup at `init_game.rs:223` into helper. Similar extraction at `phases.rs:53, 310`.
- **Files:** `init_game.rs` (B8 `ArrivalTaskContext` used at :262 stays as-is — reframed as deliberate, not antipattern), `phases.rs`, new helper location
- **Verify:** `python build.py` passes.

### Task 6.6: Prune MockBackend flag bag + add builders (C6 — Decision 3)

- **Action:** Audit 8 AtomicBool/AtomicU64 flags in `narrative/llm/mock.rs:21`. Keep only used ones. Add `MockBackend::succeeding()` (replaces most `::default()` callers' intent), `::failing()`, `::empty_response()` builder methods.
- **Decision 3 (above):** Option C (minimal prune + builders). NOT full restructure to per-agent stubs (Option B deferred at ~100 call sites).
- **Files:** `narrative/llm/mock.rs` + minimal updates to ~100 test call sites (most use `::default()` — left alone, only failing/empty_response scenarios get builders).
- **Verify:** `python build.py` passes.

### Task 6.3 DROPPED: B7 retry re-impl, B8 ArrivalTaskContext re-route

- B7 STALE: retry already delegates per `architecture/system.md:52` + CHANGELOG line 90.
- B8 DELIBERATE: `ArrivalTaskContext` extraction documented per `architecture/system.md:209`.

### End-of-Phase-6 Glue Task: Update architecture/system.md

- **Action:** Update `architecture/system.md:43` (remove `GameLifecycleService` reference — file deleted). Preserve retry delegation note at `:52`. Preserve `ArrivalTaskContext` documented at `:209`.
- **Files:** `docs/architecture/system.md`
- **Verify:** Doc consistency — no broken refs to deleted files.

### Phase 6 verification

- `python build.py` passes
- Lifecycle integration tests green
- MockBackend callers work

---

## Phase 7: Domain model fixes (Message/Swipe, persistence)

### Task 7.1: Update `docs/system/message_model.md`

- **Read-first:** ADR-013 (Message domain model, superseded), ADR-017 (Message swipes reintroduced), `architecture/system.md:24` (snapshot + message relationship).
- **Action:** Update `docs/system/message_model.md` to reflect accessor-pattern Message struct (accessor reads from `swipes[active_swipe_index]`, no mirrored fields). Document that `from_db` returns incomplete Message (caller attaches swipes via mappers).
- **Files:** `docs/system/message_model.md`
- **Verify:** Spec consistent with ADR-017 swipes-as-separate-table model.

### Task 7.2: Collapse Message mirrored fields → accessor methods (A4, A10 — Decision 4)

- **Action:** `model/message.rs` struct has mirrored fields (`text`, `location_header`, `event_header`, `snapshot_id`) duplicating `swipes[active_swipe_index]`. Per doc comment at `:9-13` these are "runtime-mirrored fields... kept in sync." DELETE the mirrored struct fields. Existing accessor methods at `:78-90` read from `&self.swipes[self.active_swipe_index]` directly. DELETE sync code in `set_active_swipe` (`:100-103`) + `update_active_swipe_text` (`:109-111`) + `set_snapshot_id` (`:117`).
- **Decision 4 (above):** Option A (accessor). NOT Option C (separate DbMessageRow — deferred).
- **Caveat (Task 7.2 explicit caveat above):** `set_snapshot_id` MUST be rewired to write `swipes[active_swipe_index].snapshot_id` instead of mirrored field. Round-trip hydration test covers.
- **Migration status:** Codebase partially migrated — `application/context.rs:77,80,87`, `application/context_tests.rs:125`, `application/action_pipeline/retry_tests.rs:33-36`, `storage/mappers/message_tests.rs:27-32, 108-110` already use `.text()` method syntax. ~10-20 stragglers using `.text` field syntax need migration.
- **Files:** `model/message.rs` + ~10-20 caller files (stragglers)
- **Verify:** Round-trip hydration test (Test Guarantee #3) passes. `python build.py` passes.

### Task 7.3: Delete `GameStateSnapshot::apply_to` (A12)

- **Action:** Delete `apply_to` method at `model/state_snapshot.rs:66` (zero prod callers, only `state_snapshot_tests.rs:50` tests the method). Delete the test.
- **Status (Issue 11 reassessment):** `GameState::from_snapshot` at `model/state.rs:259` is the only production path (heavily used: `init_game.rs:56,120`, `retry.rs:65`, `context.rs:98,136`). `apply_to` is dead — partial mutation that skips world/map/player/npcs/messages. Task 7.3 trivial.
- **Files:** `model/state_snapshot.rs` (delete method) + `model/state_snapshot_tests.rs` (delete test)
- **Verify:** `python build.py` passes.

### Task 7.4: Tighten `MessageHistory` encapsulation (A5)

- **Action:** `MessageHistory` exposes `replace`, `retain`, `iter_mut`, `as_slice`. Remove public mutators. Callers must use `MessageHistory` methods instead of bypassing.
- **Files:** `model/message_history.rs` + callers
- **Verify:** `python build.py` passes.

### Task 7.5: Move `assemble_prompt_text` to method on `LayeredPromptAssembler` (Issue 12 decision)

- **Action:** `assemble_prompt_text` at `model/prompt_preset.rs:57` is "preset = god-assembler" — eats world rules + response length + does assembly. Move to method on `LayeredPromptAssembler` (post-Phase 3.5 collapse). Callers do `service.assembler().assemble_prompt_text(preset, world_rules, response_length)`. `PromptPreset` exposes only config accessors.
- **Callers (3 prod + 4 tests):** `application/context.rs:53`, `narrative/agents/quantifier/agent.rs:105`, `model/prompt_preset_tests.rs:63,74,91,100,110,121`
- **Files:** `narrative/prompt/assembler.rs` (add method) + `model/prompt_preset.rs` (remove method, expose config accessors) + 7 callers
- **Verify:** `python build.py` passes.

### Phase 7 verification

- `python build.py` passes
- Round-trip hydration test for Message+Swipes green
- Prompt preset tests pass with new assembler method

---

## NOT in scope

- `Operation` enum complete deletion (storage-level test-injection decorator `Backend::Test` stays — only key type swaps)
- `error_return` removal / `GenerationStatus::Error` reversal — DELIBERATE arch (Phase 5 misclassification)
- Phase 6.3 B7 retry re-impl — STALE, retry already delegates
- Phase 6.3 B8 `ArrivalTaskContext` re-route — DELIBERATE code, not antipattern
- `ActionOutcome::Error` framing as "side-channel fix" — REFramed to dead-arm cleanup only
- A6 `TemplateVars` collapse — DEFERRED (6+ callers + struct field ripple)
- A3 `Confidence`/`QuantifierConfidence` collapse — DEFERRED (similar rippling pattern, ~80 refs)
- `Operation` → trait-object `StorageBackend` (Option C from Decision 1) — bigger refactor, out of scope
- DB schema changes for Message/Swipe — DB already normalized per ADR-017
- State-machine / `PipelineStep` enum pipeline rewrite — scope creep, Phase 6.1 spec constrained out
- Anti-pattern lint prevention (covered by `abstraction-antipattern-healthcheck-plan.md`)
- C5 OpenRouter headers in generic request (separate storage/LLM refactor)
- C2 global `sanitize_llm_output` (separate Phase 6 backend refactor)
- D2/D10/D11 minor dedup (`helpers.rs::empty_to_none`, `from_row` consistency) — trivial, opportunistic

---

## What already exists (MUST reuse, NOT reimplement)

| Resource | Location | Usage |
|----------|----------|-------|
| `GameState::from_snapshot` | `model/state.rs:259` | Already full reconstruction path; Task 7.3 only deletes `apply_to` (zero prod callers) |
| `Message::text()`/`location_header()`/etc. accessors | `model/message.rs:78-90` | Partial migration done; Task 7.2 finishes it |
| `Backend::Test { base, overrides }` decorator | `storage/backend/core.rs:246+` | Already decorator pattern; Task 5.4 swaps key type only |
| `MockBackend` constructors (`::default()`, `::failing()`, `::with_delay()`) | `narrative/llm/mock.rs` | Task 6.6 extends with `::succeeding()`/`::empty_response()`, doesn't rewrite |
| `DbWorld::from_row` + `world_card_from_db` | `storage/models/world.rs`, `storage/backend/worlds.rs` | Pattern reference; Task 7.2 chose Option A (accessor) NOT Option C (DbMessageRow), so pattern unused. Worth knowing pattern exists. |
| Architecture dec docs (`architecture/system.md`, `system/game_flow.md`, `system/llm_processing.md`, `system/storage.md`) | `docs/` | Phase 6.1/7.1 spec MUST read-first before code |
| ADRs (008 SQLite Snapshot, 013 Message Domain superseded by 017 Swipes) | `docs/adr/` | Phase 7 spec references must align |
| CHANGELOG line 272 (`8e4acf5` error model unification) | `CHANGELOG.md` | Justification for Phase 5 Drops #1 |

---

## Failure modes (per-phase risk surface)

| Codepath | Failure mode | Plan handles? |
|----------|-------------|---------------|
| Task 5.4 method-name-key override map | Typo in `with_failure("load_message", ...)` silently passes through to real storage | YES — Task 5.4b Drop-trait/post-test assert observability |
| Task 5.4 storage Backend::Test | Tests pass but no longer exercise mocked failure path | Mitigated by observability check + existing per-resource tests |
| Task 6.4 inline GameLifecycleService | 13 methods move; existing integration tests must exercise through DefaultApplicationService | YES — Test Guarantee #2 (lifecycle integration tests) |
| Task 7.2 accessor pattern | `set_snapshot_id()` writes mirrored field only; post-fix must write `swipes[active].snapshot_id` | YES — Task 7.2 Explicit Caveat + Test Guarantee #3 (round-trip hydration) |
| Phase 4 state.rs split | 30-40 import paths break | Mitigated by `pub use` re-export shield (Issue 3) + follow-up Task 4.1b gradual migrate (Issue 5) |
| Phase 5 storage changes | 12 per-resource files touched | Per-file `cargo check` between mutations; existing per-resource tests gate |
| Phase 6 pipeline `PipelineInputs<'a>` | Borrow-checker fights with `'a` lifetime param threaded through phases | Spec 6.1 must address lifetime contract upfront before code |
| Phase 7.5 co-locate `MessageEntry` DTO | DTO types in different files; moves need import updates | Low risk, mechanical |

---

## Unresolved decisions (open — implementer-judgment at refactoring time, NOT blockers)

- **Phase 4 module names**: plan says `model/generation_status.rs`, `model/movement.rs`, etc. but exact type-to-file mapping not enumerated. Implementer decides during split.
- **Phase 4.2 `misc.rs` split target filenames**: `text_check.rs`, `swipe.rs`, `game_control.rs` proposed but not locked.
- **`MessageEntry` exact form post Phase 7.5**: keep as struct or refactor to borrow from `Message`? Plan says "Add `From<&Message> for MessageEntry`" — keeps DTO. But `MessageEntry` mirrors `Message+Swipe` per A11. Could reduce duplication.

---

## Tech Debt Added + Follow-up Changes

### New tech debt added by this implementation plan

1. **Phase 4 `pub use` re-export shield** (Issue 3, 5)
   - `state.rs`, `misc.rs`, `renderers.rs` keep `pub use submodule::*` re-exports after split
   - Two ways to import same type co-exist (`crate::model::state::GenerationStatus` AND `crate::model::state::generation_status::GenerationStatus`)
   - **Reversal:** Task 4.1b gradual migration of high-churn callers (`game_service.rs`, `application_service.rs`, `pipeline.rs`) to new paths, eventually remove re-exports
   - **Risk if not addressed:** split becomes name-only — callers never learn new module boundaries

2. **Task 5.4 stringly-typed override keys** (Issue 13, Decision 1)
   - `HashMap<&'static str, TestOverride>` replaces `HashMap<Operation, TestOverride>`
   - Loses compile-time exhaustiveness. Method-name typo only caught at runtime via Drop-trait observability check (Task 5.4b)
   - **Reversal:** full `StorageBackend` trait-object test double (Option C from Decision 1, deferred). ~60 storage method signatures touched. Bigger refactor.
   - **Risk if not addressed:** typo'd override silently passes through. Mitigated by observability check, not eliminated.

3. **Task 5.5 `with_backend_mut(game_id: Option<u64>)`** (Issue 14)
   - Single-method API with `Option<u64>` self-documents scope at call site but no type-level enforcement
   - Two callers could pass wrong value without compile error
   - **Reversal:** split into `with_backend_mut_global` + `with_backend_mut_game_scoped(game_id)` (Option B from Issue 14, deferred as too much API surface)
   - **Risk if not addressed:** low. Call sites self-document.

4. **Task 6.6 `MockBackend` partial fix** (Decision 4)
   - AtomicBool flag bag pruned (unused flags removed) but core pattern stays
   - Per-call `Vec<String>` consumed via `call_index` stays
   - **Reversal:** full split to per-agent stub structs (`NarratorStub`, `QuantifierStub`) — Option B from Decision 4, deferred at ~100 call sites
   - **Risk if not addressed:** flag-bag smell recurs as new test scenarios added. Mitigated by builder methods (`::succeeding()` etc.) for common cases.

5. **Task 7.2 Message keeps DB-row + domain mixing** (Decision 3)
   - Chose Option A (accessor on `Message`) over Option C (separate `DbMessageRow` type)
   - `from_db` returns incomplete Message before swipes attached — accessor pattern means callers cannot read uninitialized text, but type-system does not prevent construction
   - **Reversal:** introduce `DbMessageRow` distinct from `Message` if persistence grows another shape divergence
   - **Risk if not addressed:** low. Accessor method guarantees reads always work post-swipe-attachment.

6. **Phase 6.1 spec self-constrained — `run_from_input` monolith stays** (Issue 9, B3)
   - Did NOT propose state-machine / `PipelineStep` enum rewrite
   - `run_from_input` 92-line method (B3 finding) remains
   - **Reversal:** separate plan if maintainability complaints recur. Original investigation hinted at state-machine; this plan defers it.
   - **Risk if not addressed:** 92-line method still hard to extend. Mitigated by `PipelineInputs<'a>` struct (Task 6.2) reducing arg count + `spawn_pipeline_task` helper (Task 6.5) reducing duplication.

7. **MockBackend construction boilerplate not migrated** (Task 6.6 follow-up)
   - Builder methods added (`::succeeding()`, `::failing()`, `::empty_response()`)
   - ~100 existing call sites using `MockBackend::default()` NOT migrated to builders
   - **Reversal:** gradual rename pass during routine test edits
   - **Risk if not addressed:** cosmetic — builders exist for new code, old code works.

### Deferred from original plan (already-noted, kept deferred)

- **A6 `TemplateVars` collapse**: 6+ callers + struct field ripple (`assembler.rs:169`, `types.rs:36`). Stays mixed struct. Follow-up: separate sub-plan to migrate callers + restructure struct.
- **A3 `Confidence` / `QuantifierConfidence` collapse**: ~80 refs across tests + prod code. Bidirectional `From` impls. Deferred alongside A6 — similar rippling pattern.
- **D2 `helpers.rs::empty_to_none` one-function module**: trivial. Inline or split when Phase 4 touched (Phase 4 is now done — could pick up as cleanup).
- **D10 `empty_to_none` vs `opt_string` duplication**: address when D2 touched.
- **D11 `from_row` consistency on `Db*` models**: add `from_row` to `DbGame`, `DbGameStateSnapshot`. Low risk.
- **C5 OpenRouter headers (`X-Title`, `HTTP-Referer`) in generic request**: address in separate storage/LLM refactor.
- **C2 global `sanitize_llm_output`**: address when Phase 6 backend refactor happens (not in this plan's scope).

### Documentation tech debt (acknowledged, NOT addressed by this plan)

- **`abstraction-fixes-plan.md` NOT updated** to reflect dropped Phase 5.1/5.2/5.3, dropped Phase 6.3 B7/B8, reframed B2 framing, locked decision shapes. Future readers see stale framing.
- **`abstraction-antipatterns-summary.md` NOT updated** to annotate 4 misclassifications (B9 `error_return`, B2 framing, B7 stale, B8 deliberate). Future re-investigation could repeat mistakes — investigation demonstrated credibility gap (4+ wrong findings out of 47).
- **`docs/system/action_pipeline.md`** updated per Task 6.1 (in scope).
- **`docs/system/message_model.md`** updated per Task 7.1 (in scope).
- **`architecture/system.md`** updated per Task 6.4 (in scope, GameLifecycleService gone).
- **No retroactive ADR** for "Phase 5 error-model reversal rejected" — could add brief ADR noting investigation's misclassification + decision to preserve documented arch. Optional.

### Follow-up changes recommended (post-implementation, no deadline)

1. **Task 4.1b**: Migrate high-churn callers (`game_service.rs`, `application_service.rs`, `pipeline.rs`, `phases.rs`) from re-exported `crate::model::state::{...}` to direct `crate::model::state::generation_status::{...}`. Eventually remove re-exports. No deadline.
2. **Task 5.4c**: Audit test failures caught by Drop-trait observability check during first sprint after merge. If typo rate is low, observability is sufficient. If high, escalate to trait-object test double (Option C).
3. **Task 6.6b**: Migrate `MockBackend::default()` call sites to `::succeeding()` during routine test edits. Mechanical, opportunistic.
4. **Sub-plan A6+A3**: Break out `TemplateVars` + `Confidence`/`QuantifierConfidence` collapse as separate plan. Estimate ~10 callers + struct field ripples for A6 (`model/state.rs:36` + `assembler.rs:169`), ~80 refs for A3.
5. **ADR note (optional)**: Add to `docs/adr/` a brief note documenting that the abstraction-antipatterns investigation report dated 2026-06-26 contained 4 misclassifications due to not consulting CHANGELOG/architecture docs. Records the decision to preserve the documented error model.

### Risks if follow-ups ignored

- Phase 4 re-export shield becomes permanent (module split exists in name only) — split value erodes
- Investigation report misclassifications propagate to future audits — same mistakes repeated
- `MockBackend` flag-bag smell recurs with each new test scenario
- `run_from_input` monolith grows longer as new phases added (current 92 lines, projected 110+ within 6 months if no state-machine effort)
- MockBackend `::default()` call sites never adopt builders — pattern never fully consolidates

---

## Risks if follow-ups ignored (summary)

- Phase 4 re-export shield becomes permanent — module split exists in name only, split value erodes
- Investigation report misclassifications propagate to future audits — same 4 mistakes repeated
- `MockBackend` flag-bag smell recurs as new test scenarios added without builders
- `run_from_input` monolith grows longer as new phases added (~110+ lines within 6 months if no state-machine effort)
- `MockBackend::default()` call sites never adopt builders — pattern never consolidates

---

## Essential commands

```bash
cd chronicler_engine
python build.py    # Full validation: fmt + clippy + tests + coverage
cargo check        # Quick check between phase tasks
cargo clippy       # Lint check (lib.rs:9 FIXME removed at end of Phase 5)
```

## Plan adherence

- Single PR / single commit (per user decision)
- All 7 phases included
- 4 corrections + 11 decision locks applied
- A6 + A3 deferred
- Phase 5.1/5.2/5.3, Phase 6.3 B7/B8 dropped
- Spec update tasks (6.1, 7.1) must read existing arch docs FIRST before writing updates
