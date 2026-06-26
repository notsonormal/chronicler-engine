# Plan: Abstraction Anti-Pattern Fixes (Tiered)

**Date:** 2026-06-26
**Status:** Planned
**Goal:** Remediate the 47 findings from `reports/abstraction-antipatterns-summary.md` in risk-ordered phases, with explicit validation contracts per phase.

**Source investigation:**

- `reports/abstraction-antipatterns-summary.md` (master, 47 findings)
- `reports/zone-{a,b,c,d}-*.md` (per-zone detail)

**Call-site verification done:** grep-confirmed call sites for all Tier 1 + most Tier 2 findings before drafting. Reviewer errors noted where applicable. See "Verification log" appendix.

---

## Overview

47 findings across 6 risk tiers. Single plan, 7 phases ordered by risk + dependency:

| Phase | Tier cluster | Risk | Findings | Behavior change? |
|-------|--------------|------|----------|------------------|
| 1 | Pre-work | trivial | 1 (stale file) | None |
| 2 | Tier 1 surgical | low | ~8 | None (deletes + inlines) |
| 3 | Tier 2 collapses | low-med | ~7 | Type shape, not behavior |
| 4 | Tier 5 module reorg | low-med | 4 | Module paths only |
| 5 | Tier 3 error model | high | ~5 | Error flow changes |
| 6 | Tier 4 pipeline | high | ~3 | Pipeline restructure |
| 7 | Tier 6 domain | high | ~3 | Schema + domain types |

**Gating:** each phase must pass `python build.py` before next phase begins. Architectural phases (5-7) additionally require `docs/system/*.md` updates before code per `chronicler_engine/AGENTS.md` PLANNING REQUIREMENTS.

---

## Architecture Decisions

1. **Single plan, multi-phase.** User explicitly requested single plan. Phases are independent enough to gate separately but share the master investigation as source.

2. **Risk-ordered.** Mechanical deletes first (safe, fast value), then type collapses (still mechanical), then module reorg (mechanical but multi-file), then architectural changes (require spec). Each phase lands separately.

3. **Per-phase validation contract.** Each phase has explicit test criteria. `python build.py` is necessary but not sufficient for architectural phases — must also identify which tests prove the change is correct.

4. **Architectural phases require spec update first.** Per AGENTS.md: error model (Phase 5), pipeline (Phase 6), domain (Phase 7) must update `docs/system/*.md` before code. This plan notes which docs; spec writing is part of phase scope.

5. **Reviewer errors corrected.** Two findings (A9 `push_section`, D9 `add_status_swap_headers`) claimed single-caller by reviewer but grep showed 2+ callers. Dropped from inline list. See Verification Log.

6. **Stale file cleanup first.** `src/model/agent.rs_temp` discovered during grep — not part of original findings but blocks Phase 2 (creates grep noise). Pre-work phase.

---

## Phase 1: Pre-Work

### Task 1.1: Delete stale `agent.rs_temp`

- **Source:** grep discovered `src/model/agent.rs_temp` — duplicate of `agent.rs`, not in module tree.
- **Action:** `rm src/model/agent.rs_temp`
- **Files:** `src/model/agent.rs_temp` (delete)
- **Validation:**
  - [ ] `cargo check` clean
  - [ ] `python build.py` passes
  - [ ] No module references the file (grep `agent.rs_temp` empty)

---

## Phase 2: Tier 1 Surgical Deletes (Mechanical, Low Risk)

All deletes + inlines. No behavior change. No type shape change.

### Task 2.1: Delete `ActionOutcome::Error` variant (B2)

- **Call sites verified:** `src/application/action_pipeline/retry.rs:95, 157` — both are match arms (`ActionOutcome::Error { .. } => {}`), not constructions. Variant never built.
- **Action:**
  1. Delete variant + `#[allow(dead_code)]` attribute from enum in `src/application/action_pipeline/pipeline.rs:28-33`
  2. Delete the 2 match arms in `retry.rs`
- **Files:** `src/application/action_pipeline/pipeline.rs`, `src/application/action_pipeline/retry.rs`
- **Validation:**
  - [ ] `cargo check` clean (no remaining references to `ActionOutcome::Error`)
  - [ ] `cargo nextest run action_pipeline` passes
  - [ ] `python build.py` passes

### Task 2.2: Delete `PromptLayer::Phi` variant (C7)

- **Call sites verified:** only `src/narrative/prompt/types_tests.rs:12` — discriminant test.
- **Action:**
  1. Delete variant from `PromptLayer` enum in `src/narrative/prompt/types.rs:19`
  2. Delete the discriminant test
- **Files:** `src/narrative/prompt/types.rs`, `src/narrative/prompt/types_tests.rs`
- **Validation:**
  - [ ] `cargo check` clean
  - [ ] `python build.py` passes

### Task 2.3: Delete `narrate_continuation` trait method (C1)

- **Call sites verified:** zero production callers. Impls in `deepseek.rs:53`, `mock.rs:117`, `openrouter.rs:57`, `ollama.rs:71`. Tests in `mock_tests.rs`, `deepseek_tests.rs`, `orchestration_tests.rs:119,148` (mock impl).
- **Action:**
  1. Delete method from `LlmBackend` trait in `src/narrative/llm/backend.rs:86`
  2. Delete impls from `deepseek.rs`, `mock.rs`, `openrouter.rs`, `ollama.rs`
  3. Delete tests: `test_mock_narrate_continuation*` (4 tests), `test_deepseek_narrate_continuation`, mock impl in `orchestration_tests.rs`
- **Files:** 5 source + 3 test files
- **Validation:**
  - [ ] `cargo check` clean
  - [ ] `python build.py` passes
  - [ ] No production code regresses (all remaining backend tests still pass)

### Task 2.4: Delete `_player_name` param (B1)

- **Call sites verified:** `application_service.rs:142,147,191` → `game_service.rs:96` → `actions.rs:15`. Also `#[instrument(..., fields(player_name, ...))]` at `actions.rs:10`.
- **Action:**
  1. Drop param from `execute_action_impl` signature in `actions.rs:11-15`
  2. Drop `player_name` from `#[instrument]` fields
  3. Drop param from `GameService::execute_action` in `game_service.rs:96`
  4. Drop construction + passing at `application_service.rs:142-191`
  5. Update 4 tests in `actions_tests.rs` that pass `player_name`
- **Files:** `actions.rs`, `actions_tests.rs`, `application_service.rs`, `game_service.rs`
- **Validation:**
  - [ ] `cargo check` clean
  - [ ] `cargo nextest run execute_action` passes
  - [ ] `python build.py` passes

### Task 2.5: Delete `_all_rooms` param + `extract_movement_from_text` (C3)

- **Call sites verified:** `_all_rooms` passed at `orchestration.rs:63`. `extract_movement_from_text` only called from `parser_tests.rs:375,387,399` (tests).
- **Action:**
  1. Drop `_all_rooms: &[RoomInfo]` param from `parse_quantifier_response_with_movement` in `parser.rs:65`
  2. Delete `extract_movement_from_text` function entirely
  3. Update `orchestration.rs:63` call site (drop the `&rooms` argument)
  4. Update tests in `parser_tests.rs` to drop the third arg
  5. Delete `extract_movement_from_text` tests at `parser_tests.rs:370-405`
- **Files:** `parser.rs`, `parser_tests.rs`, `orchestration.rs`, `mod.rs` (export removal)
- **Validation:**
  - [ ] `cargo check` clean
  - [ ] `cargo nextest run quantifier` passes
  - [ ] `python build.py` passes

### Task 2.6: Delete `_exits` param (D6)

- **Call sites verified:** `view_models.rs:157` signature. Callers: `renderers.rs:86` (passes `&[]`), `templates_tests.rs:454,466,478,490,502` (pass actual).
- **Action:**
  1. Drop `_exits: &[String]` from `ActionAreaViewModel::new`
  2. Update 5 test call sites in `templates_tests.rs`
  - **Files:** `view_models.rs`, `templates_tests.rs`, `renderers.rs`
- **Validation:**
  - [ ] `cargo check` clean
  - [ ] `cargo nextest run templates` passes
  - [ ] `python build.py` passes

### Task 2.7: Inline `sanitize_for_prompt` (C10)

- **Call sites verified:** 1 caller (`assembler.rs:379`). Has own test file (`sanitize_tests.rs`). Not pure single-caller due to tests.
- **Action:**
  1. Inline the regex/body into `assembler.rs:379`
  2. Delete `sanitize_for_prompt` from `sanitize.rs:8`
  3. Re-export removal from `mod.rs:12`
  4. Move tests into `assembler_tests.rs` or delete if covered
- **Files:** `sanitize.rs`, `sanitize_tests.rs`, `assembler.rs`, `mod.rs`, `assembler_tests.rs`
- **Validation:**
  - [ ] `cargo check` clean
  - [ ] `cargo nextest run prompt` passes
  - [ ] `python build.py` passes

### Task 2.8: Remove or wire `NarratorAgent` (C9)

- **Call sites verified:** `registry.rs:57` constructs from string `"narrator"`. `execute` returns `NoOp`. If deleted, registry arm breaks.
- **Decision: Delete (no plan to wire it).**
- **Action:**
  1. Delete `NarratorAgent` struct + impls from `registry.rs:90-100`
  2. Delete `"narrator" =>` arm in `registry.rs:57`
  3. Delete tests in `registry_tests.rs:174-185`
- **Files:** `registry.rs`, `registry_tests.rs`
- **Validation:**
  - [ ] `cargo check` clean
  - [ ] `cargo nextest run agents` passes
  - [ ] `python build.py` passes

### Task 2.9: Add `Operation::CountSwipes` variant (D5)

- **Call sites verified:** `swipes.rs:~164` uses `Operation::LoadSwipesForMessages` for `count_swipes_for_message` — piggybacking unrelated variant.
- **Action:**
  1. Add `Operation::CountSwipes` to enum in `storage/backend/core.rs`
  2. Update `count_swipes_for_message` to use the new variant
  3. Handle new variant in `Backend::Test` match arm (if applicable)
- **Files:** `storage/backend/core.rs`, `storage/backend/swipes.rs`
- **Validation:**
  - [ ] `cargo check` clean
  - [ ] `cargo nextest run storage` passes
  - [ ] `python build.py` passes

### DROPPED from original Tier 1 list (reviewer errors)

- **A9 `push_section` inline** — grep found 2 separate definitions (`assembler.rs:139` + `prompt_preset.rs:90`), each with 3 callers. NOT single-caller. Keep as-is.
- **D9 `add_status_swap_headers` inline** — grep found 3 callers (`actions.rs:94, 107, 122`). NOT single-caller. Legitimate helper. Keep as-is.

### Task 2.10: Phase 2 final validation

- **Action:** Run `python build.py` after Tasks 2.1-2.9 land.
- **Validation:**
  - [ ] Full build passes
  - [ ] Git diff shows only deletions + minor signature edits, no logic changes
  - [ ] Test count delta: ensure deleted tests = deleted code only, no意外 test drops

---

## Phase 3: Tier 2 Type Collapses (Low-Med Risk)

Convert premature generalizations to concrete types. Behavior unchanged; type shapes change. Multi-file ripple per finding.

### Task 3.1: Convert `StatePatch` enum → struct (A1)

- **Call sites verified:** `game_service.rs:150,159,160` (destructure `StatePatch::Scene`). Definition at `model/agent.rs:96`.
- **Action:**
  1. Delete `pub enum StatePatch { Scene { ... } }`
  2. Add `pub struct ScenePatch { npc_ids: Vec<String>, movement_destination: Option<String>, confidence: Confidence }`
  3. Replace `StatePatch::merge` with `ScenePatch::merge(self, other: ScenePatch) -> ScenePatch`
  4. Update `AgentResult::StatePatch(StatePatch)` → `AgentResult::ScenePatch(ScenePatch)` variant
  5. Update `game_service.rs:160` destructure
- **Files:** `model/agent.rs`, `application/game_service.rs`, `narrative/agents/mod.rs`, `action_pipeline/pipeline.rs` (if used)
- **Validation:**
  - [ ] `cargo check` clean
  - [ ] `cargo nextest run pipeline` + `agents` passes
  - [ ] Review: `merge` impl simpler than before (no match arm needed)

### Task 3.2: Convert `TriggerRequirement` enum → struct (A2)

- **Call sites verified:** ~20 references in tests + fixtures. Heavily used.
- **Action:**
  1. Replace `pub enum TriggerRequirement { TimesMet(ComparisonOperator, u32) }` with `pub struct TimesMetRequirement { op: ComparisonOperator, count: u32 }`
  2. Add accessor methods
  3. Update all `TriggerRequirement::TimesMet(op, n)` construction sites → `TimesMetRequirement { op, count: n }`
  4. Rename field type in `Trigger` struct
- **Files:** `model/trigger.rs`, plus ~6 test files + `test_support/fixtures.rs`
- **Acceptance criteria:**
  - [ ] `cargo check` clean
  - [ ] All trigger tests pass unchanged in semantics
  - [ ] `python build.py` passes

### Task 3.3: Unify `Confidence` and `QuantifierConfidence` (A3)

- **Call sites verified:** Bidirectional `From` impls already exist. Tests use `QuantifierConfidence::High` directly.
- **Action:**
  1. Delete `QuantifierConfidence` enum from `model/quantifier.rs:7`
  2. Use `Confidence` (from `model/agent.rs:88`) in `quantifier.rs`
  3. Delete `From<Confidence> for QuantifierConfidence` + reverse
  4. Update all `QuantifierConfidence::X` references → `Confidence::X`
- **Files:** `model/quantifier.rs`, `model/agent.rs` (if it references QuantifierConfidence), `engine/action_processing.rs`, multiple test files
- **Validation:**
  - [ ] `cargo check` clean
  - [ ] `python build.py` passes
  - [ ] No `QuantifierConfidence` symbol remains in repo

### Task 3.4: Remove `preprocess_user_text` from trait, inline in Ollama (C8)

- **Call sites verified:** Trait default in `backend.rs:71`. Override in `ollama.rs:62`. Tests in `ollama_tests.rs:123,145,166`. Called from `ollama.rs:42` (inside `OllamaBackend::call`).
- **Action:**
  1. Delete `preprocess_user_text` from `LlmBackend` trait
  2. Move body into a private `fn preprocess_user_text` in `ollama.rs`
  3. Update call in `ollama.rs:42` to use private fn
  4. Keep tests as-is (they call the private fn via `pub(crate)` if needed)
- **Files:** `backend.rs`, `ollama.rs`, possibly `ollama_tests.rs`
- **Validation:**
  - [ ] `cargo check` clean
  - [ ] Ollama tests pass
  - [ ] `python build.py` passes

### Task 3.5: Collapse `PromptAssembler` trait → concrete type (C4)

- **Call sites verified:** Trait `assembler.rs:23`. One impl `LayeredPromptAssembler`. Used as `Arc<dyn PromptAssembler>` field in `game_service.rs:20`, `assembler(&self) -> &dyn PromptAssembler` accessor at `game_service.rs:117`, mock impl in `pipeline_tests.rs:35`, import in `init_game.rs:15`.
- **Action:**
  1. Delete `pub trait PromptAssembler`
  2. Replace `Arc<dyn PromptAssembler>` with `Arc<LayeredPromptAssembler>` in `game_service.rs:20`
  3. Change `assembler()` return type to `&LayeredPromptAssembler`
  4. Drop `impl PromptAssembler for LayeredPromptAssembler` block — methods become inherent
  5. Update mock in `pipeline_tests.rs:35` to use concrete type
  6. Drop `PromptAssembler` from imports in `init_game.rs:15`, `application/game_service.rs:15`
- **Files:** `assembler.rs`, `assembler_tests.rs`, `game_service.rs`, `init_game.rs`, `pipeline_tests.rs`
- **Validation:**
  - [ ] `cargo check` clean
  - [ ] `cargo nextest run prompt` passes
  - [ ] No `PromptAssembler` trait symbol remains (only `LayeredPromptAssembler`)
  - [ ] `python build.py` passes

### Task 3.6: Replace `TemplateVars` with direct fn arg (A6)

- **Call sites verified:** struct `template.rs:5` with 1 field `user: String`. Function `render_template(text, vars: &TemplateVars)`.
- **Action:**
  1. Change signature to `render_template(text: &str, user: &str) -> String`
  2. Delete `TemplateVars` struct
  3. Update callers
- **Files:** `model/template.rs`, callers (grep `TemplateVars` for full list)
- **Validation:**
  - [ ] `cargo check` clean
  - [ ] `python build.py` passes

### Task 3.7: Convert `QueryHandlers` struct → free functions (B11)

- **Call sites verified:** Stateless struct. Used in `application_service.rs:105,113` (field) + `query_handlers_tests.rs` (10+ tests call `QueryHandlers::new()`).
- **Action:**
  1. Delete `pub struct QueryHandlers;` + `impl QueryHandlers { ... }`
  2. Move methods to free functions `pub fn get_generating_status(ctx: ...) -> ...` etc
  3. Drop field from `DefaultApplicationService` in `application_service.rs:105,113`
  4. Update all test call sites to call free functions directly
- **Files:** `query_handlers.rs`, `application_service.rs`, `query_handlers_tests.rs`
- **Validation:**
  - [ ] `cargo check` clean
  - [ ] All query_handlers tests pass with new call style
  - [ ] `python build.py` passes

### Task 3.8: Phase 3 final validation

- **Action:** Run `python build.py` after Tasks 3.1-3.7 land.
- **Validation:**
  - [ ] Full build passes
  - [ ] No premature generalization symbols remain: `StatePatch::Scene`, `TriggerRequirement::TimesMet`, `QuantifierConfidence`, `PromptAssembler` trait, `TemplateVars`, `QueryHandlers` struct

---

## Phase 4: Tier 5 Module Reorganization (Low-Med Risk, Mechanical)

Pure module moves. No logic changes. Many import path updates.

### Task 4.1: Split `model/state.rs` grab-bag (A7)

- **Action:** Split into `model/generation_status.rs`, `model/movement.rs`, `model/scene.rs`, `model/narrative_state.rs`. Keep `state.rs` as `GameState` only.
- **Files:** `model/state.rs` + new files + `model/mod.rs`
- **Validation:**
  - [ ] `cargo check` clean
  - [ ] `python build.py` passes
  - [ ] Each new file has only cohesive types

### Task 4.2: Split `server/fragments/misc.rs` (D1)

- **Action:** Split `misc.rs` into `text_check.rs`, `swipe.rs`, `game_control.rs` (retry, retrigger, reset). Move `ActionForm` reuse issue (D7) by defining `CheckTextForm` in `text_check.rs`.
- **Files:** `fragments/misc.rs` (delete), new fragment files, `fragments/mod.rs`, `fragments/actions.rs` (D7 fix)
- **Validation:**
  - [ ] `cargo check` clean
  - [ ] `python build.py` passes

### Task 4.3: Split `server/fragments/renderers.rs` (D8)

- **Action:** Split into `fragments/response.rs` (HTTP helpers: `ok`, `bad_request`, `app_err_to_response`, `html_escape`) + `fragments/fragment_renderers.rs` (UI renderers: `render_header`, `render_story_log`).
- **Files:** `fragments/renderers.rs` (delete), new files, `fragments/mod.rs`
- **Validation:**
  - [ ] `cargo check` clean
  - [ ] `python build.py` passes

### Task 4.4: Move `NpcEncounterLog` CRUD out of `trigger_eval.rs` (B12)

- **Action:** Move `increment_times_met`, `mark_trigger_fired`, `set_currently_meeting`, `get_times_met`, `is_trigger_fired` to `model/trigger.rs` (or new `npc_encounter_log.rs`). Keep `evaluate_triggers` + `check_condition` in `trigger_eval.rs`.
- **Files:** `engine/trigger_eval.rs`, `model/trigger.rs` (or new module)
- **Validation:**
  - [ ] `cargo check` clean
  - [ ] `python build.py` passes

### Task 4.5: Phase 4 final validation

- **Action:** Run `python build.py`.
- **Validation:**
  - [ ] Full build passes
  - [ ] No file named `misc.rs` in `src/server/` (matches prevention rule)
  - [ ] `model/state.rs` contains only `GameState`

---

## Phase 5: Tier 3 Error Model Consolidation (High Risk, Needs Spec)

**Pre-flight:** Update `docs/system/error_model.md` before code. Per AGENTS.md.

Error model is currently split across:

- `EngineError` (create error variants)
- `ActionOutcome` (Cancelled / Completed) — with `Error` variant deleted in Phase 2
- `GenerationStatus::Error(String)` (runtime error state) (B2, B9, D4, D5, D12)
- `Operation` enum threaded through storage calls for `Backend::Test` injection (D4, D12)

### Task 5.1: Spec update — error model

- **Action:** Write `docs/system/error_model.md` documenting:
  - Single error type at application boundary (`EngineError` or new `PipelineError`)
  - `GenerationStatus` for status only, NOT for errors
  - Storage `Operation` enum removed; test injection via decorator
- **Files:** `docs/system/error_model.md` (new)
- **Validation:**
  - [ ] Doc reviewed + approved
  - [ ] Cross-linked from `docs/architecture/guardrails.md`

### Task 5.2: Remove `error_return` helper (B9)

- **Pre-flight:** Spec from 5.1 approved.
- **Action:**
  1. Make `phase_narrate` etc return `Result<_, EngineError>` for real errors
  2. Replace `error_return(state, msg)` callers with `Err(EngineError::...)`
  3. Top-level caller decides how to render error to `state.status`
- **Files:** `phases.rs`, `pipeline.rs`
- **Validation:**
  - [ ] `cargo nextest run pipeline` passes
  - [ ] Error path tests assert `Err(...)` rather than `Ok(...)` with state mutation
  - [ ] `python build.py` passes

### Task 5.3: Remove `GenerationStatus::Error` as error channel

- **Action:** Delete `GenerationStatus::Error(String)` variant or repurpose strictly for UI status. Errors flow through `Result`.
- **Files:** `model/state.rs` (GenerationStatus enum), `application/action_pipeline/*`, `server/fragments/*` (callers)
- **Validation:**
  - [ ] `cargo check` clean
  - [ ] Error integration tests pass
  - [ ] `python build.py` passes

### Task 5.4: Remove `Operation` enum from storage prod path (D4, D12)

- **Action:**
  1. Move test interception into decorator/wrapper trait
  2. Storage methods stop taking `Operation` parameter
  3. `Backend::Test` arms deleted from all per-resource files
  4. `with_backend_mut` signature simplified
- **Files:** `storage/backend/core.rs`, `storage/backend/*.rs` (all per-resource files)
- **Validation:**
  - [ ] All storage tests pass
  - [ ] No `Backend::Test { .. } => unimplemented!()` arms remain
  - [ ] `python build.py` passes

### Task 5.5: Fix `with_backend_mut` dummy `_game_id` (D3)

- **Action:** Split into `with_backend_mut_global` (no game_id) + `with_backend_mut_game_scoped(game_id)` per original recommendation.
- **Files:** `storage/backend/core.rs` + all per-resource files
- **Validation:**
  - [ ] No `_game_id` prefix args remain
  - [ ] `python build.py` passes

### Task 5.6: Deduplicate `save_message` across backends (C11)

- **Action:** Add shared `save_message` impl via trait extension or `StorageHandle` wrapper. Delete copy-pasted impls in deepseek/ollama/openrouter/mock.
- **Files:** `narrative/llm/{deepseek,ollama,openrouter,mock}.rs`, `narrative/llm/backend.rs`
- **Validation:**
  - [ ] `python build.py` passes
  - [ ] Single `save_message` definition

---

## Phase 6: Tier 4 Pipeline Restructure (High Risk, Needs Spec)

**Pre-flight:** Update `docs/system/action_pipeline.md` before code.

### Task 6.1: Spec update — pipeline restructure

- **Action:** Write spec for state-machine or declarative pipeline shape.
  - `PipelineStep` enum or equivalent
  - Replay-from-snapshot contract (so retry can feed existing state)
  - `PipelineInputs<'a>` struct grouping immutable inputs (B5, B6)
- **Files:** `docs/system/action_pipeline.md` (likely exists — update)
- **Validation:**
  - [ ] Spec reviewed + approved

### Task 6.2: Group pipeline args into `PipelineInputs<'a>` (B5, B6)

- **Action:** Define `PipelineInputs<'a> { world, map, player, all_npcs }`. Update `phase_narrate`, `build_trigger_request`, and other phase signatures.
- **Files:** `action_pipeline/phases.rs`, `pipeline.rs`
- **Validation:**
  - [ ] No `#[allow(clippy::too_many_arguments)]` in `phases.rs`
  - [ ] `python build.py` passes

### Task 6.3: Reconcile `retry.rs` + `ArrivalTaskContext` into pipeline (B7, B8)

- **Action:** Parameterize `run_from_input` (or new `run_from_state`) so retry feeds existing state + trigger context. Refactor `ArrivalTaskContext` to be a `FreeAction("")` or `Action::Arrive` flowing through the pipeline.
- **Files:** `action_pipeline/pipeline.rs`, `action_pipeline/retry.rs`, `bootstrap/init_game.rs`
- **Validation:**
  - [ ] `retry_event_continuation` deletes its re-impl, calls pipeline
  - [ ] `ArrivalTaskContext` struct deleted; arrival flows through pipeline
  - [ ] All arrival + retry integration tests pass
  - [ ] `python build.py` passes
  - [ ] UI smoke test: verify arrival narration still works in browser via screenshot per AGENTS.md

### Task 6.4: Flatten `DefaultApplicationService` identity wrappers (B4)

- **Action:** Either inline `GameLifecycleService` into `DefaultApplicationService`, or expose the inner service. Delete 14 identity-wrapper methods.
- **Files:** `application/application_service.rs`, `application/game_lifecycle.rs`
- **Validation:**
  - [ ] No passthrough methods remain
  - [ ] `python build.py` passes

### Task 6.5: Deduplicate `spawn_blocking` boilerplate (B10)

- **Action:** Extract `spawn_pipeline_task<F>(ctx, service, f)` helper. `retry`, `retrigger`, `process_action` use it.
- **Files:** `application/message_editing.rs`, `application/application_service.rs`
- **Validation:**
  - [ ] `python build.py` passes

### Task 6.6: Address `MockBackend` flag-bag (C6)

- **Action:** Replace `AtomicBool` flag bag with per-test closures or small struct-per-agent stubs.
- **Files:** `narrative/llm/mock.rs`, all tests using MockBackend
- **Validation:**
  - [ ] MockBackend tests still test same scenarios
  - [ ] `python build.py` passes

---

## Phase 7: Tier 6 Domain Model Fixes (Highest Risk, Needs Schema Work)

**Pre-flight:** Significant domain + storage spec updates.

### Task 7.1: Spec update — Message/Swipe domain model

- **Action:** Document new domain model: swipes first-class, active values derived via accessor.
- **Files:** `docs/system/message_model.md` (new or update)
- **Validation:**
  - [ ] Spec reviewed + approved

### Task 7.2: Remove mirrored fields from `Message` (A4, A10)

- **Action:**
  1. Make `Message::text`, `Message::location_header`, `Message::event_header`, `Message::snapshot_id` derived via accessors on `Message` reading `swipes[active_swipe_index]`
  2. OR normalize DB: store swipes as first-class rows; `Message` constructed with swipes
  3. Delete `Message::from_db` factory producing invalid object; replace with proper repository hydration
- **Files:** `model/message.rs`, `storage/backend/messages.rs`, `storage/mappers/message.rs`
- **Validation:**
  - [ ] All message integration tests pass
  - [ ] No invalid `Message` can be constructed
  - [ ] `python build.py` passes

### Task 7.3: Snapshot reconstruction (A12)

- **Action:** Treat snapshot load as full reconstruction via `GameState::from_snapshot` instead of partial `apply_to` mutation.
- **Files:** `model/state_snapshot.rs`, callers
- **Validation:**
  - [ ] Snapshot round-trip tests cover all `GameState` fields
  - [ ] `python build.py` passes

### Task 7.4: `MessageHistory` encapsulation (A5)

- **Action:** Remove `replace`, `retain`, `iter_mut`, `as_slice` bypass methods. Expose read-only view + `append` only. Update callers.
- **Files:** `model/message_history.rs`, callers
- **Validation:**
  - [ ] `python build.py` passes

### Task 7.5: `MessageEntry` DTO co-location (A11)

- **Action:** Add `From<&Message> for MessageEntry` impl in `model/state.rs`. Move `to_message_entries` next to it.
- **Files:** `model/state.rs`, `model/message_history.rs`
- **Validation:**
  - [ ] `python build.py` passes

### Task 7.6: Move `PromptPreset::assemble_prompt_text` to assembler service (A8)

- **Action:** Move assembly logic to `narrative/prompt/assembler.rs`. `PromptPreset` exposes only config accessors.
- **Files:** `model/prompt_preset.rs`, `narrative/prompt/assembler.rs`
- **Validation:**
  - [ ] `python build.py` passes

---

## Remaining findings (lowest severity, defer to next sprint)

These findings are low-priority and can be deferred without blocking the main remediation:

| ID | Site | Action |
|----|------|--------|
| C6 | `MockBackend` flag-bag | Already covered in Task 6.6 |
| D2 | `helpers.rs::empty_to_none` one-function module | Inline or split across callers, but `helpers.rs` filename issue + trivial. Address if Phase 4 touched. |
| D7 | `ActionForm` reused for text-check | Address as part of Task 4.2 (split `misc.rs`) — define `CheckTextForm` |
| D10 | `empty_to_none` vs `opt_string` duplication | Address when D2 is addressed |
| D11 | Inconsistent `from_row` on Db* models | Add `from_row` to `DbGame`, `DbGameStateSnapshot` |
| C5 | OpenRouter headers in generic request | Address in Phase 7 or separate storage/LLM refactor |
| C2 | Global `sanitize_llm_output` | Address when Phase 6 backend refactor happens |
| B3 | `run_from_input` monolith | Address as part of Phase 6 spec work (state machine) |
| B5/B6 (param carry) | Already in Task 6.2 |
| B9 (error_return) | Already in Task 5.2 |
| B12 | Already in Task 4.4 |
| A12 | Already in Task 7.3 |

Total findings count check: 47 mapped to phases + ~12 deferred / overlap-coverage = aligned.

---

## Dependencies

**Within phases:**

- Each phase gates the next via `python build.py`.
- Within a phase, tasks are mostly independent except where noted:
  - Phase 5 Task 5.1 (spec) blocks all other Phase 5 tasks
  - Phase 6 Task 6.1 (spec) blocks all other Phase 6 tasks
  - Phase 6 Task 6.3 depends on Task 6.2 (inputs context)
  - Phase 7 Task 7.1 (spec) blocks all other Phase 7 tasks
  - Phase 7 Task 7.2 + 7.3 + 7.4 + 7.5 + 7.6 are mostly sequential (Message domain)

**Cross-phase:**

- Phase 2 (deletes) unblocks Phase 3 (collapses)
- Phase 3 (collapses) makes Phase 5 (error model) easier — `Operation` enum logic cleaner after Phase 3 collapses other abstractions
- Phase 7 depends on Phase 6 (domain fixes need pipeline + error model stable)

---

## Risks

| Risk | Mitigation |
|------|-----------|
| Phase 2 deletes break tests that exercise deleted code | Test deletions explicit per task; verify no test regression via `cargo nextest run` |
| Phase 3 collapses have cascading type errors across many files | Land one task at a time; `cargo check` after each; fix errors before moving on |
| Phase 4 module reorg breaks import paths | Use IDE-assisted rename/move; full `cargo check` per commit |
| Phase 5 error model touches every storage + pipeline method | Split into sub-tasks per storage file; full test suite per sub-task |
| Phase 6 pipeline restructure has high regression risk | UI smoke tests mandatory per AGENTS.md; arrival + retry flows manually verified with screenshots |
| Phase 7 domain changes require DB schema changes | Coordinate with `storage/` layer; migration script; snapshot round-trip tests |
| Reviewer errors (A9, D9) could indicate other reviewer errors | Verification Log appendix tracks confirmed cases. Re-verify any finding whose fix seems wrong on implementation. |
| Cascading test updates bloat PRs | Land Phase 2 + 3 + 4 as separate commits within one or two PRs; reviewable per phase |
| Spec-writing phases underestimated | Time-box for spec (half-day per architectural phase) |

---

## Success Criteria

1. All 47 findings from `reports/abstraction-antipatterns-summary.md` are either:
   - Remediated in Phases 1-7 (with verification)
   - Dropped due to reviewer error (A9, D9 — verified via grep, see Verification Log)
   - Deferred explicitly to future work (lowest severity items in "Remaining findings" table)
2. `python build.py` passes after each phase completes
3. No premature generalization symbols remain in repo:
   - `StatePatch::Scene`, `TriggerRequirement::TimesMet`, `PromptAssembler` trait, `narrate_continuation`, `PromptLayer::Phi`, `NarratorAgent`, `QuantifierConfidence`, `preprocess_user_text`, `GenerationStatus::Error`, `Operation` (storage)
4. No `misc.rs` or `helpers.rs` (server fragment) after Phase 4
5. No `#[allow(clippy::too_many_arguments)]` in `phases.rs` after Phase 6
6. `Message::from_db` no longer produces invalid objects (Phase 7)
7. Architectural phases (5-7) have updated `docs/system/*.md` before code

---

## Out of Scope

- **Static prevention rules** — covered by separate plan `abstraction-antipattern-healthcheck-plan.md` (advisory clippy check).
- **Antipattern-checker agent skill** — covered by separate plan `antipattern-checker-skill-plan.md` (LLM-based semantic review).
- **Performance optimizations** — findings target maintainability, not perf. If perf becomes concern, separate plan.
- **Other findings outside the 47** — investigation was scoped to abstraction patterns only. Other categories (dead code unrelated to abstraction, naming, comments) ignored.

---

## Verification Log

Findings re-verified via grep before plan was drafted. Reviewer errors caught:

| Finding ID | Reviewer claim | Grep result | Disposition |
|-----------|----------------|-------------|-------------|
| A9 `push_section` | 1 caller | 2 separate definitions, 3 callers each | DROPPED from inline list |
| D9 `add_status_swap_headers` | 1 caller | 3 callers (`actions.rs:94,107,122`) | DROPPED from inline list |
| B2 `ActionOutcome::Error` | dead variant | Confirmed: 2 match arms, 0 constructions | DELETE in Task 2.1 |
| C1 `narrate_continuation` | zero prod callers | Confirmed: only test caller impls | DELETE in Task 2.3 |
| C4 `PromptAssembler` | 1 impl | Confirmed: `LayeredPromptAssembler` only, but heavily used as `Arc<dyn>` | Phase 3 Task 3.5 (larger change) |
| B11 `QueryHandlers` | stateless | Confirmed: zero-field struct, 10+ test call sites | Task 3.7 |

**Other notes:**

- `src/model/agent.rs_temp` discovered during grep — stale duplicate of `agent.rs`. Pre-work Task 1.1 deletes it.
- Phase 2-4 ready to implement immediately. Phases 5-7 need spec before code per AGENTS.md.

---

## Recommended Execution Order

1. **Phase 1** — immediate, trivial. Single file delete.
2. **Phase 2** — single PR, low risk. All surgical deletes + inlines. Run after agent.rs_temp cleanup.
3. **Phase 3** — single PR per task or pair of tasks. Type collapses. Each task self-contained.
4. **Phase 4** — single PR, mechanical. Module moves.
5. **Phase 5** — write spec first, then multi-PR phase. Error model.
6. **Phase 6** — write spec first. Pipeline restructure.
7. **Phase 7** — write spec first. Domain + schema.

Phases 1-4 can land within one sprint. Phases 5-7 each warrant their own sprint + dedicated attention.

Review plan and approve before implementing any phase? Recommend starting with Phase 1 + 2 (lowest risk, immediate value).
