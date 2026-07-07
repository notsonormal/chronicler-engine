# Tier 1 + Tier 2 Fix Plan (from code-quality-review-simpler-hexagon.md + holistic-hexagon-review-2026-07-06.md)

## Summary

Fix 8 findings from two reviews. Tier 1 (3 critical): ADR-030 GenerationGuard amendment + test sharpening, delete `application/error.rs` entirely, OpContext axum `FromRequestParts` extractor. Tier 2 (5 refinements): WorldSnapshot canonicalization (GameState constructors take WorldSnapshot directly), arrival_service::run dedup, process_action split, retry.rs test-literal dedup, property test fail-fast.

User decisions (locked):
- Q1: Delete `application/error.rs` entirely.
- Q2: Holistic approach for WorldSnapshot — keep as container, make it canonical type.
- Q3: Full `FromRequestParts` extractor for `OpContext`.

EM-review amendments (locked):
- T2.1 SubTask 2.1.4: rewrite all 6 existing tests against `into_response()` (was: keep 4 as-is). Move file to adapter dir.
- T2.3 SubTask 2.3.2: 18+ callsites (9 test files), 3 SP (was: 5+ sites, 2 SP). SubTask 2.3.3 deleted (phantom methods + layering violation).
- T2.4 SubTask 2.4.2: 3 explicit outcomes enumerated for rollback clarity.
- T3.3: inject `(cached=false, persisted=Generating)` during flight + assert fail-fast (was: poll for forbidden state).
- T1.2 dropped: no ADR-029 amendment. ADR-029 remains historical record.

## Key Changes

- **T1.1** — Amend ADR-030 §"Single-Writer Rule" + §"Read-Only Elsewhere" + History entry: acknowledge `GenerationGuard::Drop` as 2nd AtomicBool writer (RAII panic-safety, writes `false` only).
- **T2.1** — Delete `application/error.rs` + `error_tests.rs`. Inline `impl IntoResponse for ApplicationError` in adapter. HTML escape + error_div + status mapping all live in adapter. Application layer keeps `Display` + `is_user_displayable()` only. Consolidate dual `is_user_displayable` predicate (Review1.H).
- **T2.2** — `impl FromRequestParts<AppState> for OpContext` in `op_context_loader.rs`. 10+ handlers migrate from `State<AppState> + load_op_context_for_active_game(&state)` to `ctx: OpContext` extractor. Deletes ~170 lines of mechanical idiom + `ctx_or_error` helper. Fixes fidelity regression: `EngineError::Config` preserved (was flattened to `Render`).
- **T2.3** — `GameState::new` + `GameState::from_snapshot` signatures take `WorldSnapshot` directly. 18+ callsites (9 test files) collapse `Arc::clone(&ctx.world_snapshot.world) × 4` → `ctx.world_snapshot.clone()`. No extra methods on WorldSnapshot — wholesale passing IS the abstraction.
- **T2.4** — `process_action` split into `heal_stale_generating` + `claim_generation_slot` (3 explicit outcomes) + `release_generation_slot`. Body becomes linear narrative ≤30 lines (was ~60+).
- **T3.1** — `arrival_service::run` lines 60-110 replaced with `context::load_expecting_valid_state(&self.ctx)?`. Scenario-logs injection on fresh-state branch preserved (worker decides mechanism).
- **T3.2** — New `make_test_ctx_with_default_preset` in `tests/helpers/fixtures.rs`. 4 OpContext literals in retry.rs replaced.
- **T3.3** — Rewrite `wait_until_idle` in `is_generating_invariant_tests.rs` to fail-fast: inject `(cached=false, persisted=Generating)` during flight, assert helper returns false immediately.

## Implementation

### Phase 1: Documentation Lock

- [ ] #### Task 1.1: Amend ADR-030 to acknowledge GenerationGuard::Drop (1 SP)
  - [ ] ##### SubTask 1.1.1: Amend §"Single-Writer Rule" — ApplicationService is single writer for "true" transitions (CAS); GenerationGuard::Drop is 2nd writer for "false" RAII fallback. Two writers never disagree on value (both write false when something completes/fails; only ApplicationService writes true). (0.5 SP)
  - [ ] ##### SubTask 1.1.2: Amend §"Read-Only Elsewhere" — replace "All code paths outside ApplicationService treat the AtomicBool as read-only" with explicit allowance for GenerationGuard::Drop. Tightening: any *3rd* writer is a bug. (0.25 SP)
  - [ ] ##### SubTask 1.1.3: Add History entry: "**2026-07-06**: Acknowledged `GenerationGuard::Drop` as second writer for AtomicBool (RAII panic-safety); tightened verification strategy to fail-fast on divergence." (0.25 SP)
  - [ ] ##### Validate: `python scripts/validate_adrs.py` passes; ADR-030 conflict check vs ADR-010 + ADR-027 + ADR-018 still clean.

### Phase 2: Refactoring (sequential)

- [ ] #### Task 2.1: Delete `application/error.rs` (5 SP)
  - [ ] ##### SubTask 2.1.1: Inline `impl IntoResponse for ApplicationError` in `src/adapters/driving/http/error.rs`. Body construction (html_escape + error_div from application/error.rs:23-37) moves with it. Status mapping (StatusCode from match on variant) moves with it. (2 SP)
  - [ ] ##### SubTask 2.1.2: Adapter consults `err.is_user_displayable()` (existing predicate at application_service.rs:39) to decide body content: displayable → `<div>...{err.to_string()}...</div>`; not → generic `<div>Internal Server Error</div>`. Consolidates dual predicate (Review1.H). (1 SP)
  - [ ] ##### SubTask 2.1.3: Delete `src/application/error.rs` + `src/application/error_tests.rs`. Update `src/application/mod.rs` (drop `pub mod error;` and `#[cfg(test)] mod error_tests;`). (0.5 SP)
  - [ ] ##### SubTask 2.1.4: Move tests to `src/adapters/driving/http/error_tests.rs`. Rewrite all 6 existing tests against `into_response()` output: extract `StatusCode` via `Response::status()` + body via `axum::body::to_bytes(...).await`. Drop `HttpError` trait imports. Drop EngineError direct HttpError impl test (EngineError no longer implements HttpError). Net: 5 tests (validation → 400, concurrent → 503, shutting_down → 503, engine → 500, html_escape consolidated as body-shape assertion). Add 1 regression test: non-displayable variant returns generic "Internal Server Error" body. (1.5 SP)
  - [ ] ##### Validate: `python build.py` passes. `grep -rn "HttpError\|HttpStatusCode\|ErrorResponse" src/` returns 0 hits outside ADR-029 (historical doc).

- [ ] #### Task 2.2: OpContext axum `FromRequestParts` extractor (5 SP)
  - [ ] ##### SubTask 2.2.1: Implement `impl FromRequestParts<AppState> for OpContext` in `src/adapters/driving/http/op_context_loader.rs`. Body calls existing `load_op_context_for_active_game`. Maps `EngineError::Config` → `ApplicationError::Engine(EngineError::Config(...))` preserving fidelity (current Pattern 2 flattens to `EngineError::Render` — fix this regression). (2 SP)
  - [ ] ##### SubTask 2.2.2: Migrate 10+ handlers. Replace `State<AppState> + ctx_or_error(&state) / load_op_context_for_active_game(&state).map_err(...)` idioms with `ctx: OpContext` extractor parameter. Files: `debug.rs`, `fragments/endpoints.rs`, `fragments/history.rs`, `fragments/misc/game_control.rs`, `fragments/misc/swipe.rs`, `fragments/renderers/fragment_renderers.rs`, `fragments/renderers/response.rs`, `games_fragment/handlers.rs`, `worlds_fragment/handlers.rs`. (2 SP)
  - [ ] ##### SubTask 2.2.3: Delete `pub fn ctx_or_error` in `fragments/renderers/response.rs` (no remaining callers). Keep `load_op_context_for_active_game` + `load_op_context` as building blocks for the extractor (or inline if extractor becomes sole caller). (1 SP)
  - [ ] ##### Validate: `python build.py` passes. `grep -rn "ctx_or_error" src/adapters/driving/http/` returns 0 hits. `grep -rn "load_op_context_for_active_game" src/adapters/driving/http/` returns ≤2 hits.

- [ ] #### Task 2.3: WorldSnapshot canonicalization (4 SP)
  - [ ] ##### SubTask 2.3.1: Change `GameState::new` + `GameState::from_snapshot` signatures in `src/domain/model/state/game_state.rs` to take `WorldSnapshot` (cloned, not 4 individual `Arc<>` + `HashMap`). GameState stores the 4 fields internally as today (no internal change to GameState struct). (1 SP)
  - [ ] ##### SubTask 2.3.2: Migrate 18+ callsites (including 9 test files): `context.rs` (3 sites: load_or_fresh, load_expecting_valid_state, build_initial_state variants), `arrival_service.rs::run`, `is_generating_invariant_tests.rs::persisted_flag`, `init_game.rs` (2 sites), `run_tests.rs`, `action_pipeline/retry.rs`, `action_pipeline/pipeline_tests.rs` (3 sites), `domain/engine/logic_tests.rs`, `domain/engine/trigger_eval_tests.rs`, `domain/engine/action_processing_tests.rs`, `test_support/fixtures.rs` (3 sites), `test_support/test_app_builder.rs`, `application/query_handlers_tests.rs`, `application/agents/quantifier/agent_tests.rs`. Each `Arc::clone(&ctx.world_snapshot.world) × 4` becomes `ctx.world_snapshot.clone()`. (3 SP)
  - [ ] ##### Validate: `python build.py` passes. `grep -rn "Arc::clone(&ctx.world_snapshot.world)" src/ tests/` returns 0 hits. `grep -rn "Arc::clone(&ctx.world_snapshot" src/ tests/` returns 0 hits.

- [ ] #### Task 2.4: `process_action` split (3 SP)
  - [ ] ##### SubTask 2.4.1: Extract `Self::heal_stale_generating(ctx: &OpContext, state: &mut GameState)` — encapsulates lines ~104-112 (stale-Generating check + reset + warn log). (0.5 SP)
  - [ ] ##### SubTask 2.4.2: Extract `Self::claim_generation_slot(ctx: &OpContext, state: &mut GameState, player_name: &str, input: &str) -> Result<ProcessActionResult, EngineError>` — does `add_message` + CAS + status mutation + `save_message_and_snapshot`. Three explicit outcomes: (a) CAS won + save succeeded → `Ok(Started)`, AtomicBool=true, persisted=Generating; (b) CAS lost (someone else holds it) → `Ok(ConcurrentGeneration)`, AtomicBool unchanged, persisted unchanged, NO rollback; (c) CAS won + save failed → `Err(EngineError)`, AtomicBool=true (already set by CAS), persisted=Idle. Caller (process_action body) MUST call `release_generation_slot` on Err before propagating. Helper does NOT call release itself. (1.5 SP)
  - [ ] ##### SubTask 2.4.3: Extract `Self::release_generation_slot(ctx: &OpContext)` — `ctx.is_generating.store(false, Ordering::SeqCst)`. Replaces inline rollback at lines ~152-156. (0.5 SP)
  - [ ] ##### SubTask 2.4.4: Rewrite body of `process_action` to compose helpers linearly: load_state → heal_stale → claim_generation_slot (match on outcome: Started → continue; ConcurrentGeneration → return; Err → release + propagate) → cancel check → spawn pipeline. (0.5 SP)
  - [ ] ##### Validate: `python build.py` passes. process_action body ≤30 lines.

### Phase 3: Cleanup (sequential)

- [ ] #### Task 3.1: `arrival_service::run` dedup (1 SP)
  - [ ] ##### SubTask 3.1.1: Replace lines 60-110 of `arrival_service.rs::run` with `let mut state = context::load_expecting_valid_state(&self.ctx)?;`. Preserve scenario-logs injection on fresh-state branch (worker decides: `was_fresh` flag return from `load_expecting_valid_state`, or separate `load_or_fresh_raw` helper, or check `state.narrative.history.is_empty()` if reliable). (1 SP)
  - [ ] ##### Validate: `python build.py` passes. Behavior unchanged: arrival_service tests pass.

- [ ] #### Task 3.2: Test literal dedup in retry.rs (2 SP)
  - [ ] ##### SubTask 3.2.1: Add `make_test_ctx_with_default_preset(storage: Arc<Storage>, state: GameState) -> OpContext` in `tests/helpers/fixtures.rs`. Calls `make_test_ctx(storage, state)` then seeds "system_default" `PromptPreset` (id="system_default", name="Default System", role="You are a narrator.") into `preset_storage.save_preset(...)`. (1 SP)
  - [ ] ##### SubTask 3.2.2: Replace 4 OpContext literals at `tests/integration/application/action_pipeline/retry.rs:172, 264, 353, 443` with `make_test_ctx_with_default_preset(storage, state)`. Keep pre-existing message/seeding setup before each literal. (1 SP)
  - [ ] ##### Validate: `python build.py` passes. retry.rs shrinks by ~45 lines.

- [ ] #### Task 3.3: Property test fail-fast (1 SP)
  - [ ] ##### SubTask 3.3.1: Rewrite `wait_until_idle` in `src/application/is_generating_invariant_tests.rs` to fail-fast on divergence. Each 50ms poll iteration: check `invariant_holds(ctx)`. Allowed transient per ADR ordering: `(cached=true, persisted=Idle)` during store-then-persist convergence window. Forbidden at any poll: `(cached=false, persisted=Generating)`. Fail immediately on forbidden state. Returns true when both reach `(false, false)`. Add new test: inject `(cached=false, persisted=Generating)` during flight (manually `ctx.is_generating.store(false)` while generation in progress) + assert `wait_until_idle` returns false promptly (within 1-2 poll cycles, not the full timeout). (1 SP)
  - [ ] ##### Validate: `python build.py` passes. 3 existing tests + 1 new test pass. If new test proves flaky due to race timing, mark `#[ignore]` with comment explaining the race.

## Sub-plan Creation Order

Phase 1 (T1.1) → Phase 2 (T2.1 → T2.2 → T2.3 → T2.4) → Phase 3 (T3.1 → T3.2 → T3.3). Each task ends with `python build.py`. Per AGENTS.md "Plan Adherence" rule: any deviation requires stop-and-report.

## Story Point Total

**~21 SP across 8 tasks.** No task ≥8 SP. Tasks =5 SP (T2.1, T2.2) → worker subagent + primary verify + `build.py` per AGENTS.md.

## Test Plan

- **T1.1**: doc-only, `validate_adrs.py` passes.
- **T2.1**: 5 rewritten tests + 1 new regression test in adapter error_tests.rs asserting status codes + body shape via `into_response()`.
- **T2.2**: all 10+ handler tests pass with new extractor signature. Fidelity: `ApplicationError::Engine(EngineError::Config(...))` instead of flattened `Render(...)`.
- **T2.3**: 18+ callsites updated; GameState constructor tests pass unchanged; behavior preserved.
- **T2.4**: process_action tests pass unchanged; 3 helpers enable targeted unit tests (optional).
- **T3.1**: existing arrival_service integration tests pass.
- **T3.2**: 4 retry.rs integration tests pass.
- **T3.3**: 3 existing tests pass; 1 new test catches `(cached=false, persisted=Generating)` divergence fail-fast.

## Assumptions

- WorldSnapshot holistic approach: keep struct, make it canonical. GameState constructors take it. No extra methods (wholesale passing IS the abstraction). 80+ `.world.` / `.map.` access sites NOT bulk-reverted — they remain acceptable since struct is now real abstraction. Case-by-case method extraction is Phase-2-followup.
- ADR-030 file as read shows no GenerationGuard mention — plan amends explicitly. If user's separate update exists, implementer reconciles during T1.1.
- T2.4 helper does NOT call `release_generation_slot` itself on save failure (caller's job per linear-narrative pattern). 3 outcomes explicitly enumerated.
- T3.1 fresh-state scenario-logs injection mechanism is worker-discretion (was_fresh flag, separate helper, or history-empty check). If `load_expecting_valid_state` doesn't expose fresh-state and worker cannot cleanly extract, stop-and-report.
- T3.3 `wait_until_idle` allows ADR-documented `(cached=true, persisted=Idle)` transient window. Forbidden state is `(cached=false, persisted=Generating)`. New test injects that state during flight + asserts fail-fast.
- ADR-029 NOT amended. Deletion of HttpError trait self-justifying (zero callers + zero adapters). ADR-029 remains historical record. Pattern of prescriptive ADRs needing constant amendment is a separate audit concern, out of scope.
- Per AGENTS.md: 5 SP tasks → worker subagent + primary verify + `build.py`. 3 SP task (T2.4) → worker subagent + primary verify. 1-2 SP tasks can be primary-direct or delegate subagent.
- Per AGENTS.md "Plan Adherence" rule: stop-and-report on any deviation. No silent scope changes.
