# Plan: Granular Status Phases for LLM Pipeline

## Overview

Replace the single opaque "Thinking..." status with phase-aware updates that reflect the Chronicler Engine's three-stage LLM processing pipeline. The user will see "Generating narration...", "Quantifying scene...", or "Evaluating events..." instead of a static "Thinking..." throughout the multi-step process. The final scene application step (movement, logs, NPC events) is nearly instantaneous and does not need a distinct phase.

## Architecture Decisions

- **Separate `GenerationPhase` from `GenerationStatus`**: Keep `GenerationStatus` unchanged (`Idle`/`Generating`/`Error`) so `is_generating()`, error handling, and all existing match sites compile without modification. Phase is a secondary field on `GenerationState`.
- **Phase enum with display methods**: `GenerationPhase` owns its user-facing text and endpoint serialization, preventing display strings from leaking into business logic.
- **Pipeline-driven phase transitions**: Each major pipeline boundary in `game_service.rs` and `action_processing.rs` sets the phase before its work begins.
- **Endpoint returns kebab-case phase names**: `/status/generating` returns `"narrating"`, `"quantifying"`, `"evaluating-triggers"`, or `"idle"`. The frontend maps these to human text. This keeps the HTTP contract clean and testable.
- **Unified `.thinking` CSS class**: All generating phases use the existing `.thinking` amber style; no per-phase CSS needed.

## Risks and Mitigations

| Risk | Impact | Mitigation |
|------|--------|------------|
| Frontend breaks because `onStatusPoll` doesn't recognize new endpoint responses | High | Task 3 updates frontend atomically with the endpoint change; keep fallback handling |
| `wait_for_status_not_thinking` returns early during brief `ApplyingScene` phase | Medium | Update test utility to wait for "Ready" or "Error" explicitly |
| `wait_for_llm_idle` endpoint check breaks | Low | Endpoint still returns `"idle"` when complete; no change needed |
| `ApplyingScene` phase is too brief to be visible | Low | Acceptable — poll interval is 5s; brief phases may be skipped by the UI |

## Task List

### Task 1: Add `GenerationPhase` type and wire into `GenerationState`

**Description:** Add the `GenerationPhase` enum and integrate it into `GenerationState` as a new `phase` field. Ensure all existing `Default::default()` sites continue to compile (phase defaults to `Narrating`).

**Acceptance criteria:**
- [ ] `GenerationPhase` enum exists with three variants: `Narrating`, `Quantifying`, `EvaluatingTriggers`
- [ ] `GenerationPhase::display_text()` returns the correct human-friendly string for each variant
- [ ] `GenerationPhase::as_endpoint_str()` returns the correct kebab-case string for each variant
- [ ] `GenerationState` has a `phase: GenerationPhase` field with `#[default]`
- [ ] All existing tests in `model/state.rs` still pass

**Verification:**
- [ ] `cargo test -p chronicler_engine model::state::tests` passes
- [ ] `cargo build` succeeds with no new warnings

**Dependencies:** None

**Files touched:**
- `src/model/state.rs`

**Estimated scope:** Small (1 file, type additions only)

---

### Task 2: Update game pipeline to set phases at each boundary

**Description:** Add `set_phase(state, phase)` helper in `game_service.rs` and call it at each pipeline step: before `narrate_action`, before `determine_npcs_in_room`, before/inside `execute_freeaction_impl` for triggers and scene application. Also set the initial phase to `Narrating` in `fragments.rs` `action_handler`.

**Acceptance criteria:**
- [ ] `set_phase` utility exists and sets both `status = Generating` and the requested phase
- [ ] `execute_action` (FreeAction branch) sets `Narrating` before LLM narration
- [ ] `execute_action` sets `Quantifying` before scene quantification
- [ ] `execute_freeaction_impl` sets `EvaluatingTriggers` before `evaluate_and_narrate_triggers`
- [ ] Scene application (movement, logs, events) runs without a phase change — it is fast enough that `reset_generating` handles the transition back to `Idle`
- [ ] `action_handler` sets `phase = Narrating` alongside `status = Generating`
- [ ] `reset_generating` resets phase to `Narrating` (default)

**Verification:**
- [ ] `cargo test -p chronicler_engine game_service_tests` passes
- [ ] `cargo test -p chronicler_engine execute_freeaction_impl_tests` passes

**Dependencies:** Task 1

**Files touched:**
- `src/engine/game_service.rs`
- `src/engine/action_processing.rs`
- `src/server/fragments.rs`

**Estimated scope:** Medium (3 files, logic changes across pipeline)

---

### Task 3: Update server endpoint, template, and frontend to render phases

**Description:** Update `generating_status_handler` to return the phase name when generating. Update `ActionAreaTemplate` to render phase-specific text. Update frontend `onStatusPoll` and `MutationObserver` to handle the new endpoint responses. Keep the optimistic "Thinking..." on form submit.

**Acceptance criteria:**
- [ ] `/status/generating` returns `"narrating"` / `"quantifying"` / `"evaluating-triggers"` when generating
- [ ] `/status/generating` still returns `"idle"` when idle and error HTML when error
- [ ] `ActionAreaTemplate` renders phase-specific text instead of "Thinking..." for all generating states
- [ ] JavaScript `onStatusPoll` maps phase names to user-friendly display text (no `applying-scene` key needed)
- [ ] JavaScript `MutationObserver` correctly disables button for all generating phases and re-enables for Ready/Error
- [ ] `updateToThinking()` still shows "Thinking..." optimistically on form submit

**Verification:**
- [ ] `cargo test -p chronicler_engine component_tests` passes
- [ ] Manual: load UI, submit a command, observe status text changes (or verify via existing mock flow tests)

**Dependencies:** Task 1, Task 2

**Files touched:**
- `src/server/fragments.rs`
- `src/server/templates.rs`
- `assets/index.html`

**Estimated scope:** Medium (3 files, endpoint + template + frontend must land together)

---

## Checkpoint: Core Implementation

Before proceeding to test updates, verify:
- [ ] `cargo build` succeeds with zero warnings
- [ ] `cargo test -p chronicler_engine` passes (unit tests only, excluding Playwright tests)
- [ ] UI mock flow works: status cycles through phases and returns to Ready

---

### Task 4: Fix existing test utilities broken by new status text

**Description:** Update `wait_for_status_not_thinking` to wait for "Ready" or "Error" instead of `!contains("Thinking")`. Update `test_llm_narration_appears_via_polling` to accept any generating phase instead of requiring "Thinking".

**Acceptance criteria:**
- [ ] `wait_for_status_not_thinking` waits until status contains "Ready" or "Error"
- [ ] `test_llm_narration_appears_via_polling` asserts status is NOT "Ready" during generation (instead of asserting "Thinking")
- [ ] `test_llm_handles_arrival_narration` still passes
- [ ] `test_look_command_shows_thinking` still passes (Look is sync, returns Ready)

**Verification:**
- [ ] `cargo test -p chronicler_engine flow_mock_tests` passes
- [ ] `cargo test -p chronicler_engine flow_llm_tests` passes (if OPENROUTER_API_KEY is set, or skips gracefully)

**Dependencies:** Task 3

**Files touched:**
- `tests/test_utils.rs`
- `tests/flow_llm_tests.rs`

**Estimated scope:** Small (2 files, assertion updates)

---

### Task 5: Add unit and integration tests for phase transitions

**Description:** Add tests verifying that the phase field transitions correctly through the pipeline and that the endpoint returns the expected phase names.

**Acceptance criteria:**
- [ ] `test_generating_status_handler_narrating` — sets state to `Generating` with `Narrating` phase, asserts endpoint returns `"narrating"`
- [ ] `test_generating_status_handler_quantifying` — same for `Quantifying` / `"quantifying"`
- [ ] `test_freeaction_phase_starts_narrating` — after `execute_action` with FreeAction, phase is `Narrating`
- [ ] `test_freeaction_phase_transitions_mock` — with mock backend, phase eventually returns to `Idle`
- [ ] `test_action_area_template_narrating` — `ActionAreaTemplate::new` with `Narrating` phase renders "Generating narration..."

**Verification:**
- [ ] `cargo test -p chronicler_engine component_tests` passes
- [ ] `cargo test -p chronicler_engine game_service_tests` passes
- [ ] `cargo test -p chronicler_engine template_tests` passes

**Dependencies:** Task 3

**Files touched:**
- `tests/component_tests.rs`
- `tests/game_service_tests.rs`
- `src/server/templates.rs` (test block)

**Estimated scope:** Small (3 files, new test cases)

---

## Checkpoint: Complete

- [ ] All tests pass: `cargo test -p chronicler_engine`
- [ ] Build is clean: `cargo clippy` with no new warnings
- [ ] UI verification: mock flow test shows status text changing
- [ ] Plan file updated if any deviations occurred during implementation

## Open Questions

1. **Should the optimistic update show the first phase?** The `action_handler` immediate response currently returns "Thinking...". Since the first phase is always `Narrating`, we could return "Generating narration..." instead. This is a minor UX improvement.
2. ~~Should `ApplyingScene` be a distinct phase?~~ **Resolved:** No — scene application is instantaneous and folded into `EvaluatingTriggers` / reset.
3. ~~Distinct CSS per phase?~~ **Resolved:** Keep unified `.thinking` class; no per-phase CSS needed.
