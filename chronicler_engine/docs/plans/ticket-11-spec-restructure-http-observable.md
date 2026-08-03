# Ticket 11 — Spec restructure: HTTP-observable, endpoint-named

## Summary

Restructure action-pipeline + flow specs so every spec is **HTTP-observable**, **endpoint-named**, **covered by HTTP E2E test** in `tests/http/`. Deliver atomically: three endpoint-named specs (`actions.md` / `reset.md` / `story_log.md`), three matching HTTP test files, HTTP E2E coverage for every surviving scenario, dead `Ok(ShuttingDown)` arms in `process_action` + `retry` + `retrigger` made live (consistent 503 across the action surface), 3 new handler unit tests via real pipeline, 14 SCENARIO tags removed from `pipeline_tests.rs`. Resolves long-standing S2.4 Idle-vs-Error drift (deferred from ticket 02).

Final count: **17 declared** (12 `actions.md` + 2 `reset.md` + 3 `story_log.md`), 17 covered, 0 gaps, 0 orphans.

## Key Changes

- **Specs**: `action_pipeline.md` → `actions.md` (S1–S5 reframed HTTP, S6 in from `flow.md`); `flow.md` split into `reset.md` (S7) + `story_log.md` (S8) + S6→`actions.md`; both old files deleted. S3.1, S3.2, S3.4 (both cases), S4.1–4.3, S5.1–5.2, I.5 leave spec (not HTTP-observable). I.1–I.4 stay as invariants. No `snapshots.md`.
- **S2.4 spec corrected**: was `status == Idle`, actual code sets `status == Error` (trigger narration failure → `set_error`). Fix spec to Error. Resolves drift deferred from ticket 02 ("trigger-only failure → Error"). HTTP test uses `with_trigger_narration_fail()` (not `with_fail()`), asserts Error + main narration preserved + System log mentions "Trigger narration failed".
- **Shutdown fix (all three arms)**: top-of-fn `is_shutting_down()` check in `process_action`, `retry`, `retrigger` → return `Ok(ShuttingDown)`. Makes dead arm in `dispatch_action` + `retry_handler` + `retrigger_handler` live. Today `/action`, `/swipe/new`, `/retrigger` wrongly return 200 on cancelled shutdown; should be 503. `/action` path: `dispatch_action` → `service_unavailable(503)`. `/swipe/new` + `/retrigger` paths: `Err(ApplicationError::ShuttingDown)` → `IntoResponse` → 503 (already wired in `error.rs`). Consistent 503 across action surface.
- **Handler unit tests** (`actions_tests.rs`): 3 new, real pipeline, no trait/fake/`with_failure`:
  - `Err`→500 (game→missing persona; `process_action` errors at `require_persona` before `try_claim`/spawn; slot-release invariant holds — `try_claim` releases on save failure)
  - `Ok(ConcurrentGeneration)`→200 "Still thinking..." (pre-claim slot via `state.generation_gate.try_claim(state.game_catalogue.current_game_id(), &mut gs, &state.message_service)`)
  - `Ok(ShuttingDown)`→503 (cancel `state.shutdown_token`, relies on shutdown fix)
- **HTTP E2E tests**: `tests/http/actions.rs` replaced (delete 8 failure-path tests; add 13 S1.1–S1.5, S2.1–S2.4, S3.3, S6.1–S6.3); `tests/http/reset.rs` new (S7.1–7.2 from `flow_sequence`); `tests/http/story_log.rs` new (S8.1–8.3 from `flow_sequence`); `tests/http/flow_sequence.rs` deleted; `tests/http/mod.rs` rewired. Custom backends via `TestAppBuilder.pipeline(...)` + `make_test_pipeline_with_backends`/`make_test_pipeline_with_mock_quantifier` (patterns from `pipeline_tests.rs`). S1.3 asserts destination room name via `Swipe.location_header` (set by `push_message` from `pending_location`, refreshed by `update_current_room` after movement) — observable through `load_messages()`.
- **`pipeline_tests.rs`**: 14 SCENARIO tags removed (8 removed-from-spec, 6 HTTP-covered), replaced with plain `//` comments. Tests stay for branch coverage.
- **`tests/STRATEGY.md`**: rewrite "can't be expressed through HTTP" section generally — cancellation / internal state / mid-flight / call sequencing → unit tier. No hardcoded scenario IDs. Snapshot clause dropped (snapshots leave spec; no driven-adapter SCENARIO tags exist).

## Implementation

### Phase 1: Spec restructure (5 SP)

- [ ] #### Task 1.1: Create `docs/specs/actions.md` (3 SP)
  - Rename `action_pipeline.md` → `actions.md`. Reframe S1.1–S1.5, S2.1–S2.4, S3.3 to endpoint Given/When/Then (`POST /action` → `wait_idle` → assert `load_messages()`/status). Drop `execute_action_impl returns Ok(...)`. Move S6.1–S6.3 from `flow.md`. Remove S3.1, S3.2, S3.4 (both cases), S4.1–4.3, S5.1–5.2, I.5. Keep I.1–I.4 invariants. **S2.4 corrected**: `status == Error` (not Idle), main narration preserved, System log mentions trigger failure.
- [ ] #### Task 1.2: Create `docs/specs/reset.md` + `docs/specs/story_log.md` (1 SP)
  - `reset.md`: S7.1, S7.2 (POST /reset). `story_log.md`: S8.1, S8.2, S8.3 (POST /history/delete). Moved from `flow.md`.
- [ ] #### Task 1.3: Delete `docs/specs/flow.md` (1 SP)

### Phase 2: Make `Ok(ShuttingDown)` arms live (1 SP)

- [ ] #### Task 2.1: Add shutdown check to `process_action` + `retry` + `retrigger` (1 SP)
  - Top of each fn in `pipeline.rs`: if `self.is_shutting_down()` return `Ok(ProcessActionResult::ShuttingDown)`. All three — consistent 503 across `/action`, `/swipe/new`, `/retrigger`. No test calls these directly; `run_from_input` tests use spawned-task path (already checks `is_shutting_down`) — unaffected.

### Phase 3: Handler unit tests (3 SP)

- [ ] #### Task 3.1: 3 new tests in `actions_tests.rs` (3 SP)
  - `test_action_handler_returns_500_on_pipeline_error` (missing-persona storage, `skip_seeding(true)`)
  - `test_action_handler_returns_200_still_thinking_on_concurrent_generation` (pre-claim slot via `state.generation_gate.try_claim(state.game_catalogue.current_game_id(), &mut gs, &state.message_service)`)
  - `test_action_handler_returns_503_on_shutdown` (`state.shutdown_token.cancel()`, relies on Phase 2)

### Phase 4: HTTP E2E tests (8 SP)

- [ ] #### Task 4.0: Spike — prove custom-pipeline-at-HTTP seam (1 SP)
  - Write S2.2 (failing narrator → status Error) first, alone. `MockBackend::with_fail()` via `make_test_pipeline_with_backends` + `TestAppBuilder.pipeline(...)`. `POST /action` → `wait_idle` → assert `load_messages()` status Error. Verifies the unproven-at-HTTP-tier wiring (`build_app_graph_for_tests` → `rebind_for_test` rebinds storage/message_service, keeps recorder/agent_registry) before writing the other 12 tests. If it fails, debug one test, not 13.
- [ ] #### Task 4.1: Replace `tests/http/actions.rs` with 13 S1–S6 tests (4 SP)
  - [ ] ##### SubTask 4.1.1: Delete all 8 existing failure-path tests (1 SP)
  - [ ] ##### SubTask 4.1.2: Add S1.1–S1.5 HTTP E2E tests (1 SP) — S1.3 uses `make_test_pipeline_with_mock_quantifier` (movement), assert destination room name via `swipes[i].location_header`; S1.4 trigger NPC (pattern from `test_pipeline_trigger_happy_path` line 270)
  - [ ] ##### SubTask 4.1.3: Add S2.1–S2.4 HTTP E2E tests (1 SP) — S2.2 reuses spike; S2.3 `with_empty_response()`; S2.4 `with_trigger_narration_fail()` + custom quantifier + NPC with trigger, assert Error + main preserved + System log
  - [ ] ##### SubTask 4.1.4: Add S3.3 + S6.1–S6.3 HTTP E2E tests (1 SP) — S3.3 `with_delay(200)`; S6.x moved from `flow_sequence.rs`
- [ ] #### Task 4.2: New `tests/http/reset.rs` (1 SP) — S7.1 + S7.2 from `flow_sequence.rs`, tags → `docs/specs/reset.md`
- [ ] #### Task 4.3: New `tests/http/story_log.rs` (1 SP) — S8.1–8.3 from `flow_sequence.rs`, tags → `docs/specs/story_log.md`
- [ ] #### Task 4.4: Delete `flow_sequence.rs` + wire `mod.rs` (1 SP) — remove `mod flow_sequence;`, add `mod reset;` + `mod story_log;`. Move `post_action`/`post_empty`/`wait_idle` helpers to `tests/http/test_helpers.rs` (or inline) since `flow_sequence.rs` is deleted.

### Phase 5: Remove SCENARIO tags from `pipeline_tests.rs` (1 SP)

- [ ] #### Task 5.1: Strip all 14 SCENARIO tags (1 SP) — lines 900, 941, 971, 1007, 1042, 1081, 1124, 1151, 1181, 1245, 1277, 1305, 1350, 1430; replace with plain `//` comments. Tests stay.

### Phase 6: Generalize `tests/STRATEGY.md` (1 SP)

- [ ] #### Task 6.1: Rewrite "can't be expressed through HTTP" section (1 SP) — general rule (cancellation / internal state / mid-flight / call sequencing → unit tier), no hardcoded IDs, no snapshot clause.

### Phase 7: Verify (1 SP)

- [ ] #### Task 7.1: Run validators (1 SP) — `validate_feature_spec.py` (18/18/0/0), `cargo nextest run`, `cargo nextest run --test guardrails`, `cargo nextest run --test http`.

## Test Plan

- **Spec pilot**: 17 declared (12 `actions.md` + 2 `reset.md` + 3 `story_log.md`), 17 covered, 0 gaps, 0 orphans.
- **Handler branch coverage**: 3 new unit tests cover `Err`/`Ok(ConcurrentGeneration)`/`Ok(ShuttingDown)` arms via real pipeline; `Ok(Started)` already covered by `test_action_handler_started`.
- **HTTP E2E**: every surviving scenario (S1.1–S6.3, S7.x, S8.x) has test in `tests/http/` with SCENARIO tag.
- **Unit tier**: `pipeline_tests.rs` keeps all tests, loses all 14 SCENARIO tags.
- **Guardrails**: SCENARIO-tag placement (tags only in `tests/http/`, `src/`, `tests/integration/storage/`) holds.
- **Shutdown consistency**: `/action`, `/swipe/new`, `/retrigger` all return 503 on cancelled shutdown.

## Per Task/Sub Task Validation Steps

- Task 1.x: `ls chronicler_engine/docs/specs/` shows `actions.md`, `reset.md`, `story_log.md`, `retry.md`; no `action_pipeline.md`, no `flow.md`.
- Task 2.1: `grep -n "is_shutting_down" chronicler_engine/src/application/pipeline/pipeline.rs` shows new check at top of `process_action`, `retry`, `retrigger`; `cargo nextest run -p chronicler_engine` green.
- Task 3.1: 3 new tests pass; each asserts exact status (500 / 200 / 503).
- Task 4.0: S2.2 spike passes; custom-pipeline-at-HTTP seam proven.
- Task 4.1: `tests/http/actions.rs` has exactly 13 `#[tokio::test]` fns, each with SCENARIO tag → `docs/specs/actions.md`; none of 8 old names remain.
- Task 4.2 / 4.3: `reset.rs` (2 tests) + `story_log.rs` (3 tests) exist with correct tags.
- Task 4.4: `flow_sequence.rs` gone; `mod.rs` has `mod reset;` + `mod story_log;`, no `mod flow_sequence;`; helpers relocated.
- Task 5.1: `grep SCENARIO chronicler_engine/src/application/pipeline/pipeline_tests.rs` → no matches.
- Task 6.1: `STRATEGY.md` "can't be expressed" section names no specific scenario IDs; no snapshot clause.
- Task 7.1: all commands green with expected counts.

## Assumptions

- **Scenario IDs stay stable** (S1.1 stays S1.1); pilot dedups by ID across `docs/specs/*.md`. No renumbering. Dropped scenarios don't get reused.
- **S2.4 drift resolved here** (not deferred again). Ticket 02 explicitly said "trigger-only failure → Error"; ticket 11 implements that resolution. Existing unit test `test_pipeline_trigger_complete_failure` (line 422) stays mislabeled (`with_fail()` instead of `with_trigger_narration_fail()`) — fog for a unit-tier cleanup ticket, not ticket 11.
- **`retry`/`retrigger` shutdown arms fixed in scope** (Finding 2). All three endpoints get the same one-line guard. Consistent 503 across the action surface.
- **STRATEGY.md's referenced "mechanical guardrail in `tests/infrastructure/guardrails/`" for SCENARIO-tag placement** isn't implemented today (no such guardrail in guardrails binary). Rule enforced socially + by pilot's TEST_DIRS scan. Out of scope to add here.
- **S3.4 (both cases) leaves spec** — phase-stays-Narrating is internal state (not HTTP-observable); phase-resets-on-success is redundant with S1.1 (same HTTP-observable behaviour). Stays unit tier in `pipeline_tests.rs` (line 1007), loses SCENARIO tag. Count: 18 (not 19).
- **Custom-backends HTTP pattern**: `TestAppBuilder.pipeline(p)` + `make_test_pipeline_with_backends`/`make_test_pipeline_with_mock_quantifier`; `build_app_graph_for_tests` rebinds via `rebind_for_test` (shares app storage/message_service, keeps custom recorder/quantifier). Unproven at HTTP tier today — Phase 4.0 spike de-risks before batch.
- **S1.3 room-change observable** via `Swipe.location_header` (set from `pending_location` on every narration; refreshed by `update_current_room` after movement). No `/fragment/visual-sidebar` needed.
- **`Err` arm slot-release invariant**: `process_action`'s only `Err` paths are `require_game`, `require_persona`, `try_claim` — all before spawn. `try_claim` releases slot on save failure. After spawn, errors go to spawned task (not returned). `Err` arm safe: slot always released before `Err` propagates.
- **No `snapshots.md`** (snapshots not HTTP-observable; leave spec entirely).
