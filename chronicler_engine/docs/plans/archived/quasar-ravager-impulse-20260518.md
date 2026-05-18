# Implementation Plan: Extract `ActionPipeline` to Unify Action and Retry Flows

## Overview

Extract an `ActionPipeline` module from `actions.rs` and `retry.rs` to unify the normal play and retry flows. The pipeline explicitly models the documented game flow phases (see `docs/system/game_flow.md`).

**Current state:**
- `actions.rs` (417 lines): `execute_freeaction_pipeline` (~190 lines) mixed with 6 helper functions
- `retry.rs` (204 lines): `retry_event_continuation` duplicates trigger continuation logic (LLM call + commit + reconcile)
- Cross-import: `retry.rs` imports `execute_freeaction_pipeline`, `finish_action`, `reconcile_post_trigger_npcs` from `actions.rs`

**Target state:**
- `action_pipeline.rs` (~300-350 lines): `ActionPipeline` struct with explicit phase methods
- `actions.rs` (~80 lines): Thin dispatch layer (Talk vs FreeAction)
- `retry.rs` (~80 lines): Retry-specific setup + pipeline delegation
- Both normal play and event retry use the same `ActionPipeline::run_trigger_continuation`

---

## Architecture Decisions

1. **Pipeline takes `&DefaultGameService` (concrete)** — unchanged from current code. It needs `llm_backend` and `agent_registry`. A trait abstraction would be premature.
2. **`finish_action` moves to `helpers.rs`** — shared by pipeline finalize and retry cancellation. It is state-persistence glue, not pipeline logic.
3. **`ActionOutcome` enum** — captures the three terminal states (Completed, Error, Cancelled) so callers don't match on raw `GameState` mutations.
4. **Phase methods are private** — the public API is `run_from_input` and `run_trigger_continuation`. Internal phase decomposition is for readability/testability, not external use.
5. **Error handling preserved exactly** — early errors reload from storage; late errors use in-memory state; cancellation checkpoints at same boundaries.

---

## Task List

### Phase 1: Foundation

#### Task 1: Remove the Talk action

**Description:**
The Talk action is legacy code that should have been removed in a previous refactor (like the Look action was). Remove it entirely from the engine.

Changes needed:
1. `src/engine/action.rs` — remove `Talk(String, Option<String>)` variant from `Action` enum
2. `src/engine/parser.rs` — remove the `"t" | "talk"` parsing branch and the quote-parsing logic (`base_input`, `message`) that only existed to support dialogue messages. All input becomes `Action::FreeAction`
3. `src/engine/parser_tests.rs` — remove all Talk-specific tests and assertions; keep free-action tests
4. `src/application/game_service/actions.rs` — remove the `Action::Talk` handler from `execute_action_impl` (this will be done in Task 3, but note it here)

**Acceptance criteria:**
- [ ] `Action` enum has only `FreeAction(String)` variant
- [ ] `parse_command` returns `Action::FreeAction` for all non-empty input
- [ ] Parser tests no longer reference `Action::Talk`
- [ ] All parser tests pass

**Verification:**
- [ ] `cargo test --lib parser_tests` passes
- [ ] `cargo check` passes

**Dependencies:** None

**Files likely touched:**
- `src/engine/action.rs`
- `src/engine/parser.rs`
- `src/engine/parser_tests.rs`

**Estimated scope:** Small (3 files, straightforward deletion)

---

#### Task 2: Move `finish_action` to `helpers.rs`

**Description:**
`finish_action` is used by three consumers: the Talk handler in `actions.rs`, the pipeline finalize phase, and retry cancellation handling. Move it to `helpers.rs` alongside `save_state` and `save_committed_state` where it belongs.

**Acceptance criteria:**
- [ ] `finish_action` is defined in `helpers.rs` with `pub(crate)` visibility
- [ ] `actions.rs` no longer defines `finish_action`; it imports from `helpers.rs`
- [ ] `retry.rs` imports `finish_action` from `helpers.rs` instead of `actions.rs`
- [ ] All existing call sites compile without change to call semantics

**Verification:**
- [ ] `cargo check` passes
- [ ] `cargo test --lib retry_tests` passes

**Dependencies:** None

**Files likely touched:**
- `src/application/game_service/helpers.rs`
- `src/application/game_service/actions.rs`
- `src/application/game_service/retry.rs`

**Estimated scope:** Small (3 files, 1 function move)

---

#### Task 3: Create `action_pipeline.rs` with `ActionPipeline` struct and moved helpers

**Description:**
Create the new `action_pipeline.rs` module. Move the following functions from `actions.rs` into it, reorganizing them as methods on `ActionPipeline`:

- `save_pipeline_error` → `ActionPipeline::save_error`
- `handle_pipeline_cancellation` → `ActionPipeline::handle_cancellation`
- `default_quantifier_result` → keep as free function (or associated fn)
- `run_post_generation_agents` → `ActionPipeline::phase_post_generation`
- `build_trigger_request` → `ActionPipeline::phase_trigger_build_request`
- `reconcile_post_trigger_npcs` → `ActionPipeline::phase_post_trigger_reconcile`
- `execute_freeaction_pipeline` → `ActionPipeline::run_from_input`

Implement `run_trigger_continuation` by extracting the trigger continuation logic that currently exists in both `execute_freeaction_pipeline` and `retry_event_continuation`.

Define the public types:
```rust
pub struct ActionPipeline<'a> {
    service: &'a DefaultGameService,
    ctx: &'a GameServiceContext,
}

pub enum ActionOutcome {
    Completed(GameState),
    Error { state: GameState, message: String },
    Cancelled(GameState),
}
```

The private phase methods follow the game flow document:
1. `phase_pre_main_snapshot` — save committed snapshot, set `Narrating`
2. `phase_narrate` — build prompt, call `backend.narrate_action()`, validate non-empty
3. `phase_post_generation` — run agents via `agent_registry`, build `QuantifierResult`
4. `phase_engine_commit` — call `execute_freeaction_impl`, handle movement + triggers
5. `phase_trigger_build_request` — build `TriggerContinuationRequest` from engine `TurnResult`
6. `phase_trigger_continuation` — save pre-event snapshot, call `backend.complete()`, call `commit_trigger_narration()`
7. `phase_post_trigger_reconcile` — re-quantify, compute NPC events, apply via `apply_npc_events()`
8. `phase_finalize` — set `Idle`, call `save_state()` (via `finish_action` from helpers)

**Critical preservation requirements:**
- 3 cancellation checkpoints at exact same phase boundaries (post-narrate, pre-event-save, post-LLM-complete)
- `save_committed_state` calls at pre-main and pre-event
- Early errors: `load_state(ctx)` + `GenerationStatus::Error` + `save_state`
- Late errors (post-engine-commit): use current in-memory state + `GenerationStatus::Error` + `save_state`
- Cancellation: `load_state(ctx)` + `GenerationStatus::Idle` + `GenerationPhase::default()` + `save_state`

**Acceptance criteria:**
- [ ] New file `action_pipeline.rs` compiles with all phase methods
- [ ] `ActionPipeline::new`, `run_from_input`, `run_trigger_continuation` are public
- [ ] All moved functions from `actions.rs` exist in the new module
- [ ] No behavior changes — error handling, cancellation, snapshot timing identical

**Verification:**
- [ ] `cargo check` passes
- [ ] `cargo clippy --all-targets -- -D warnings` passes

**Dependencies:** Task 2 (finish_action must be in helpers.rs)

**Files likely touched:**
- `src/application/game_service/action_pipeline.rs` (new)
- `src/application/game_service/actions.rs` (functions removed)

**Estimated scope:** Medium (new file, ~300 lines, complex logic migration)

---

#### Task 4: Update `actions.rs` to delegate to `ActionPipeline`

**Description:**
Replace the monolithic `execute_freeaction_pipeline` function in `actions.rs` with a thin ~5-line wrapper that constructs `ActionPipeline` and calls `run_from_input`. Remove all functions that were moved to `action_pipeline.rs`. Keep `execute_action_impl` (Talk vs FreeAction dispatch) and any remaining imports.

The FreeAction branch should look like:
```rust
Action::FreeAction(text) => {
    let _lock = match ctx.action_lock.lock() { ... };
    let mut state = load_state(&ctx);
    state.narrative.last_trigger = None;
    let pipeline = ActionPipeline::new(service, &ctx);
    match pipeline.run_from_input(state, text) {
        ActionOutcome::Completed(_) => {}
        ActionOutcome::Error { message, .. } => log::error!("Action failed: {message}"),
        ActionOutcome::Cancelled(_) => {}
    }
}
```

**Acceptance criteria:**
- [ ] `actions.rs` no longer contains moved functions (`save_pipeline_error`, `handle_pipeline_cancellation`, `default_quantifier_result`, `run_post_generation_agents`, `build_trigger_request`, `reconcile_post_trigger_npcs`, `execute_freeaction_pipeline`)
- [ ] `execute_action_impl` delegates FreeAction to `ActionPipeline`
- [ ] Talk handler removed (covered by Task 1)
- [ ] File size reduced from ~417 lines to ~50 lines

**Verification:**
- [ ] `cargo check` passes
- [ ] `cargo test --lib retry_tests` passes
- [ ] `cargo test --test flow_mock` passes (if applicable)

**Dependencies:** Task 3

**Files likely touched:**
- `src/application/game_service/actions.rs`

**Estimated scope:** Small (1 file, mostly deletions)

---

#### Task 5: Update `retry.rs` to delegate to `ActionPipeline`

**Description:**
Replace duplicated trigger continuation logic in `retry.rs` with `ActionPipeline` delegation.

Changes:
- `retry_event_continuation`: construct `ActionPipeline`, call `run_trigger_continuation`, match `ActionOutcome`. Remove direct calls to `backend.complete()`, `commit_trigger_narration()`, `reconcile_post_trigger_npcs()`.
- `retry_main_narration`: construct `ActionPipeline`, call `run_from_input`.
- `retry_last_response_impl`: unchanged (retry-specific setup: anchor finding, message deletion, snapshot loading).
- Remove imports of `execute_freeaction_pipeline`, `finish_action`, `reconcile_post_trigger_npcs` from `actions.rs`. Import `finish_action` from `helpers.rs` and `ActionPipeline` from `action_pipeline`.

**Acceptance criteria:**
- [ ] `retry_event_continuation` no longer duplicates trigger continuation logic
- [ ] `retry_event_continuation` is ~15-20 lines (down from ~80)
- [ ] `retry_main_narration` is ~5 lines
- [ ] `retry_last_response_impl` unchanged
- [ ] `retry.rs` no longer imports from `actions.rs`

**Verification:**
- [ ] `cargo check` passes
- [ ] All retry tests pass: `cargo test --lib retry_tests`

**Dependencies:** Task 3, Task 4

**Files likely touched:**
- `src/application/game_service/retry.rs`

**Estimated scope:** Small (1 file, simplification)

---

### Phase 2: Integration

#### Task 6: Update `mod.rs` and test imports

**Description:**
- Add `mod action_pipeline;` to `mod.rs`
- Export `ActionPipeline` and `ActionOutcome` if needed by external callers
- Fix any import issues in `retry_tests.rs`
- The test file imports from `actions.rs` (`execute_freeaction_pipeline`, `finish_action`, `reconcile_post_trigger_npcs`). These should now come from `helpers.rs` (for `finish_action`) or be removed (the other two are no longer needed by tests since tests call retry functions, not pipeline internals directly).

**Acceptance criteria:**
- [ ] `mod.rs` declares `mod action_pipeline`
- [ ] `retry_tests.rs` compiles with corrected imports
- [ ] No unused imports in test module

**Verification:**
- [ ] `cargo check --all-targets` passes
- [ ] `cargo clippy --all-targets -- -D warnings` passes

**Dependencies:** Task 4, Task 5

**Files likely touched:**
- `src/application/game_service/mod.rs`
- `src/application/game_service/retry_tests.rs`

**Estimated scope:** Small (2 files, import fixes)

---

### Checkpoint: After Tasks 1-6
- [ ] All library tests pass: `cargo test --lib`
- [ ] All integration tests pass: `cargo test --tests`
- [ ] No clippy warnings: `cargo clippy --all-targets -- -D warnings`
- [ ] The action flow is readable in one file (`action_pipeline.rs`)
- [ ] Retry event continuation no longer duplicates trigger continuation logic
- [ ] Talk action removed from engine and parser

---

### Phase 3: Validation

#### Task 7: Run full validation suite

**Description:**
Run the complete validation pipeline as defined in `AGENTS.md`.

**Acceptance criteria:**
- [ ] `python build.py` passes (fmt + clippy + guardrails + tests)
- [ ] All test suites pass:
  - Unit tests: `cargo test --lib`
  - Integration tests: `cargo test --tests`
  - Game service tests: `cargo test --lib game_service`
  - Retry tests: `cargo test --lib retry`
  - Flow mock tests: `cargo test --test flow_mock`

**Verification:**
- [ ] `python build.py` exits with code 0
- [ ] Screenshot or quote of final build output

**Dependencies:** Tasks 1-6

**Files likely touched:** None (validation only)

**Estimated scope:** Small (validation)

---

## Risks and Mitigations

| Risk | Impact | Mitigation |
|------|--------|------------|
| Cancellation checkpoints drift | High | Preserve exact `cancel_token.is_cancelled()` checks at same phase boundaries; verify with cancellation tests |
| Snapshot timing changes | High | Preserve `save_committed_state` calls at same points (pre-main, pre-event); verify with retry tests |
| Error state persistence changes | High | Preserve dual pattern: early errors load from storage, late errors use in-memory state; verify with error-path tests |
| Phase enum values change | Medium | No changes to `GenerationPhase` assignments; verify status in tests |
| Test import breakage | Low | Fix imports in `retry_tests.rs`; tests exercise stable retry interface, not pipeline internals |

## Open Questions

- None at this time. The plan preserves exact behavior; all types and error patterns are maintained.

## Success Criteria (from original plan)

1. `cargo clippy --all-targets -- -D warnings` passes
2. `cargo nextest run --tests` passes (all game_service, flow_mock, retry, component tests)
3. `python build.py` passes (fmt + clippy + guardrails + tests)
4. The action flow is readable in one file (`action_pipeline.rs`)
5. Retry event continuation no longer duplicates trigger continuation logic
