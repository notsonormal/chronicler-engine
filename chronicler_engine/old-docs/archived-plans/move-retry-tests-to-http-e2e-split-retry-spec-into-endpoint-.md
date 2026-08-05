# Move retry tests to HTTP E2E, split retry spec into endpoint-named specs

## Summary

Ticket 05 (wayfinder map `test-strategy-execution`). Dissolve the component tier for retry: port `flow/retry_main.rs` (10 tests) + `flow/retry_event.rs` (3 tests) to HTTP E2E, write fresh HTTP E2E for every other HTTP-observable retry/retrigger spec scenario, and **move 6 existing untagged retry/retrigger tests out of `tests/http/fragment.rs`** into the new endpoint-named files (retagged). Split `docs/specs/retry.md` into two endpoint-named specs per ticket 11's rule — `swipe_new.md` (POST /swipe/new) + `retrigger.md` (POST /retrigger) — with N.M IDs continuing the existing 1.x–8.x sequence (9.x–15.x). Delete the two component-tier files. Suite stays green; pilot validates the new specs.

Branch: `testing-refactor`. Baseline: 1361 pass, 17/17 spec, 0 violations.

## Key Changes

- `docs/specs/retry.md` → **deleted**; replaced by:
  - `docs/specs/swipe_new.md` — 9.1–9.6 (main retry), 10.1–10.2 (event retry), 11.1–11.8 (retry errors), 12.1 (retry concurrency). Invariants I.1–I.7, I.8 retry side.
  - `docs/specs/retrigger.md` — 13.1–13.3 (retrigger), 14.1–14.6 (retrigger errors), 15.1 (retrigger concurrency). Invariants I.3, I.7, I.8 retrigger side, I.9.
  - R5.1/R5.2 (cancellation mid-flight) **dropped from specs** — unit-only (needs CancellationToken), per STRATEGY.md. Already covered by `retry_tests.rs` (ticket 04).
- `docs/specs/actions.md` — add S1.6 (trigger continuation re-runs quantifier, detects new NPC) — regression guard for the component-tier test being deleted.
- `tests/http/test_helpers.rs` — add `app_with_narrator_and_quantifier(narrator, quantifier)` (was inlined into actions.rs during ticket 11; reusable here). **One helper only.**
- `tests/http/swipe_new.rs` — **new**, 14 from-scratch + 3 moved-from-fragment = 17 tests, SCENARIO-tagged.
- `tests/http/retrigger.rs` — **new**, 6 from-scratch + 3 moved-from-fragment = 9 tests, SCENARIO-tagged. (13.2 folds into 13.1 — assert N+1 in same test.)
- `tests/http/actions.rs` — add S1.6 test.
- `tests/http/fragment.rs` — **remove 6 retry/retrigger tests** (moved to new files, retagged).
- `tests/http/mod.rs` — wire `mod swipe_new; mod retrigger;`.
- `tests/integration/flow/retry_main.rs` + `tests/integration/flow/retry_event.rs` — **deleted**.
- `tests/integration/mod.rs` — remove `mod flow_retry_main;` + `mod flow_retry_event;`.

## Implementation

### Phase 1: Split retry spec into endpoint-named specs (3 SP)

- [ ] #### Task 1.1: Write `docs/specs/swipe_new.md` (2 SP)
  - [ ] ##### SubTask 1.1.1: Endpoint header + 9.1–9.6 (R1.1 swipe swap, R1.2 quantifier moves player, R1.3 input preserved, R1.4 edited input, R1.5 re-evaluates triggers, R1.6 quantifier-None completion). HTTP-framed Given/When/Then, hard line breaks (two trailing spaces). (1 SP)
  - [ ] ##### SubTask 1.1.2: 10.1–10.2 (R2.1 swipe swap, R2.2 no quantifier rerun). (0.5 SP)
  - [ ] ##### SubTask 1.1.3: 11.1–11.8 (400 no-input, 400 no-game, 500 no-snapshot, 500 snapshot-deleted, Error LLM-fail, Error empty, Error room-not-found, Error trigger-fail + System log). (0.5 SP)
  - [ ] ##### SubTask 1.1.4: 12.1 (concurrency: "Still thinking...") + invariants I.1–I.7, I.8. Verify R4.3/R4.4/R4.8/R5.3 still match real code after ticket 01. (folded)
- [ ] #### Task 1.2: Write `docs/specs/retrigger.md` (1 SP)
  - [ ] ##### SubTask 1.2.1: 13.1–13.3 (new event message, no rollback, no quantifier rerun). 14.1–14.6 (400 no-trigger, 400 no-messages, 400 last-not-narration, 400 last-is-event, Error trigger-fail, 400 no-game). 15.1 (concurrency). Invariants I.3, I.7, I.8 retrigger side, I.9. (1 SP)
- [ ] #### Task 1.3: Delete `docs/specs/retry.md` (folded into 1.1)

### Phase 2: Helpers (0.5 SP)

- [ ] #### Task 2.1: Add `app_with_narrator_and_quantifier` to `tests/http/test_helpers.rs` (0.5 SP)
  - [ ] ##### SubTask 2.1.1: Signature `(narrator: Arc<MockBackend>, quantifier: Arc<dyn LlmProvider>) -> (Router, AppState)`. Mirrors `app_with_narrator` but uses `make_test_pipeline_with_mock_quantifier`. Used by 9.2, 9.5, 10.1, 10.2, 11.8, 13.x. NPC-trigger tests (9.5, 11.8, S1.6) use the inline `TestAppBuilder::default_test().data(data).pipeline(...)` pattern from actions.rs 1.4 — **no second helper**. (0.5 SP)

### Phase 3: `tests/http/swipe_new.rs` — 14 new + 3 moved = 17 tests (7 SP)

- [ ] #### Task 3.1: Main retry 9.1–9.6 (6 from-scratch tests) (3 SP)
  - [ ] ##### SubTask 3.1.1: 9.1 — `with_narrations(["First.", "Second."])`; POST /action → wait_idle → POST /swipe/new → wait_idle; assert one Narration, 2 swipes, active "Second.", id unchanged. (0.5 SP)
  - [ ] ##### SubTask 3.1.2: 9.2 — quantifier `with_prompt_responses(["{npcs:[]}", "{movement:{room2}}"])`; assert `current_room_id == "room2"` after retry. (0.5 SP)
  - [ ] ##### SubTask 3.1.3: 9.3 — POST /action "walk around" → POST /swipe/new; assert Input text still "walk around". (0.5 SP)
  - [ ] ##### SubTask 3.1.4: 9.4 — POST /action → mutate Input active swipe to "sprint forward" via `message_service` + save fresh snapshot → POST /swipe/new; assert retry narration contains "sprint forward". (1 SP — mutation seam; if snapshot not refreshed, retry loads pre-edit → test fails, fix by saving fresh snapshot)
  - [ ] ##### SubTask 3.1.5: 9.5 — NPC trigger in room2, quantifier moves to room2 on retry; POST /action (no trigger) → POST /swipe/new; assert event_header narration appears. (1 SP)
  - [ ] ##### SubTask 3.1.6: 9.6 — quantifier returns `{npcs:[]}` (no movement) on retry; POST /action → POST /swipe/new; assert status Idle, not generating. (0.5 SP — distinct backend, not foldable)
- [ ] #### Task 3.2: Event retry 10.1–10.2 (2 from-scratch tests) (1 SP)
  - [ ] ##### SubTask 3.2.1: 10.1 — set up event narration (NPC trigger fires on action), POST /swipe/new; assert exactly 2 Narration messages, main has text. (0.5 SP)
  - [ ] ##### SubTask 3.2.2: 10.2 — same setup; assert `current_room_id` unchanged after event retry. (0.5 SP)
- [ ] #### Task 3.3: Retry errors 11.3–11.8 (6 from-scratch tests — 11.1/11.2 moved) (2 SP)
  - [ ] ##### SubTask 3.3.1: 11.3 — seed narration with `snapshot_id == None`, POST /swipe/new; assert 500 + `GenerationStatus::Error` mentioning snapshot missing. (0.5 SP)
  - [ ] ##### SubTask 3.3.2: 11.4 — seed narration with snapshot_id, delete snapshot row, POST /swipe/new; assert 500 + Error mentioning snapshot not found. (0.5 SP)
  - [ ] ##### SubTask 3.3.3: 11.5 — `with_fail()` narrator, POST /action → POST /swipe/new; assert `Error` status, no new swipe. (0.5 SP)
  - [ ] ##### SubTask 3.3.4: 11.6 — `with_empty_response()` narrator, same flow; assert `Error` containing "empty". (0.5 SP — distinct backend from 11.5)
  - [ ] ##### SubTask 3.3.5: 11.7 — mutate `current_room_id` to non-existent, POST /action → POST /swipe/new; assert `Error` mentioning room invalid. (folded — same shape as actions.rs 2.1; verify retry path hits room-check)
  - [ ] ##### SubTask 3.3.6: 11.8 — NPC trigger, `with_trigger_narration_fail()`, POST /action (trigger fires) → POST /swipe/new (event retry); assert Error mentioning "Trigger narration failed", main preserved, System log present. (folded into 11.5 if same test, else 0.5 SP)
- [ ] #### Task 3.4: Move 3 fragment.rs tests → swipe_new.rs, retagged (1 SP)
  - [ ] ##### SubTask 3.4.1: Move `test_retry_handler_valid_context_error` → swipe_new.rs as 11.1 test, add `// [chronicler_engine/docs/specs/swipe_new.md] SCENARIO: 11.1` tag. (0.5 SP)
  - [ ] ##### SubTask 3.4.2: Move `test_retry_handler_requires_context` → swipe_new.rs as 11.2 test, tag 11.2. (folded)
  - [ ] ##### SubTask 3.4.3: Move `test_retry_handler_concurrent_generation` → swipe_new.rs as 12.1 test, tag 12.1. **Keep the deterministic `try_claim` pre-claim pattern — do NOT switch to `with_delay`.** (0.5 SP)

### Phase 4: `tests/http/retrigger.rs` — 6 new + 3 moved = 9 tests (4 SP)

- [ ] #### Task 4.1: Retrigger happy 13.1–13.3 (2 from-scratch tests — 13.2 folds into 13.1) (1.5 SP)
  - [ ] ##### SubTask 4.1.1: 13.1 — trigger fires on action (last_trigger set, last message is main narration), POST /retrigger; assert one new Narration with event_header. **Fold 13.2 in:** assert history length N+1 (append, not replace) in the same test. (1 SP)
  - [ ] ##### SubTask 4.1.2: 13.3 — same setup; assert `current_room_id` unchanged. (0.5 SP)
- [ ] #### Task 4.2: Retrigger errors 14.2–14.5 (4 from-scratch — 14.1/14.6 moved) (1.5 SP)
  - [ ] ##### SubTask 4.2.1: 14.2 — `skip_seeding(true)` or no-messages state, POST /retrigger; assert 400. (0.5 SP)
  - [ ] ##### SubTask 4.2.2: 14.3 — seed last message as Input, POST /retrigger; assert 400. (0.5 SP)
  - [ ] ##### SubTask 4.2.3: 14.4 — seed last message as event Narration, POST /retrigger; assert 400. (0.5 SP)
  - [ ] ##### SubTask 4.2.4: 14.5 — `with_trigger_narration_fail()`, trigger fires, POST /retrigger; assert Error, no new event message. (folded)
- [ ] #### Task 4.3: Move 3 fragment.rs tests → retrigger.rs, retagged (1 SP)
  - [ ] ##### SubTask 4.3.1: Move `test_retrigger_handler_valid_context_error` → retrigger.rs as 14.1 test, tag 14.1. (0.5 SP)
  - [ ] ##### SubTask 4.3.2: Move `test_retrigger_handler_requires_context` → retrigger.rs as 14.6 test, tag 14.6. (folded)
  - [ ] ##### SubTask 4.3.3: Move `test_retrigger_handler_concurrent_generation` → retrigger.rs as 15.1 test, tag 15.1. **Keep `try_claim` pre-claim pattern.** (0.5 SP)

### Phase 5: Actions S1.6 + delete component tier (3 SP)

- [ ] #### Task 5.1: Port `test_trigger_continuation_runs_quantifier_and_detects_new_npc` to actions.rs (2 SP)
  - [ ] ##### SubTask 5.1.1: Add scenario S1.6 to `docs/specs/actions.md`: "Action trigger continuation re-runs the quantifier and detects newly-present NPCs". ID S1.6 free in actions.md 1.x namespace (pilot dedups across specs). (1 SP)
  - [ ] ##### SubTask 5.1.2: Write `tests/http/actions.rs::test_trigger_continuation_reruns_quantifier_detects_new_npc_http` — room_npcs contains "gabriella", quantifier returns `["gabriella"]` on 2nd call, POST /action; assert gabriella in `scene.npcs_in_area`, `times_met == 1`, `currently_meeting`. (1 SP)
- [ ] #### Task 5.2: Delete component-tier retry files + unwire (1 SP)
  - [ ] ##### SubTask 5.2.1: Delete `tests/integration/flow/retry_main.rs` + `tests/integration/flow/retry_event.rs`. (0.5 SP)
  - [ ] ##### SubTask 5.2.2: Remove `#[path = "flow/retry_main.rs"] mod flow_retry_main;` + `#[path = "flow/retry_event.rs"] mod flow_retry_event;` from `tests/integration/mod.rs`. (0.5 SP)

### Phase 6: Verify (1 SP)

- [ ] #### Task 6.1: Full suite + pilot + guardrails (1 SP)
  - [ ] ##### SubTask 6.1.1: `cargo nextest run -p chronicler_engine` — green. Net: +21 new-from-scratch (14 swipe_new + 6 retrigger + 1 actions) + 6 moved (no count change) - 13 deleted component-tier = ~1369 pass. (0.5 SP)
  - [ ] ##### SubTask 6.1.2: `python scripts/validate_feature_spec.py` — 45 declared / 45 covered / 0 gaps / 0 orphans / 0 format violations. (17 existing + 17 swipe_new + 10 retrigger + 1 S1.6 = 45.) (0.5 SP)

## Test Plan

- `cargo nextest run -p chronicler_engine` — full suite green.
- `python scripts/validate_feature_spec.py` — 45 declared / 45 covered / 0 gaps / 0 orphans / 0 format violations.
- `cargo nextest run -p chronicler_engine tests::http::swipe_new tests::http::retrigger tests::http::actions` — targeted green.
- Guardrails: `cargo nextest run -p chronicler_engine --test infrastructure` — green (no new SCENARIO tags in `src/` or `tests/integration/`; new tags only in `tests/http/`).

## Per Task/Sub Task Validation Steps

- 3.x from-scratch: each test fails if the branch it targets is reverted (e.g. 11.3 fails if pre-spawn anchor check removed). `git stash`-revert to confirm, restore.
- 3.4.3 / 4.3.3 (moved concurrency): keep `try_claim` pre-claim; if switched to `with_delay`, becomes flake. Test passes because gate is deterministically busy.
- 5.1.2: fails if quantifier not re-run on trigger continuation (gabriella missing from `npcs_in_area`).
- 6.1.2: pilot gap report is load-bearing — any new scenario without a test shows as a gap.

## Assumptions

- Ticket 01 code changes landed (R4.3/R4.4 pre-spawn anchor 500, R5.3/R5.4 generation gate, R4.8 System log persistence) — verified in `pipeline.rs` (`check_retry_anchor` + top-of-fn gate). HTTP E2E for 11.3/11.4/12.1/15.1 asserts these.
- Ticket 04 unit tests for retry branches stay — HTTP E2E complements (overlap rule, each tier asserts what it sees). No unit test deletion.
- `skip_seeding(true)` on `TestAppBuilder` produces a no-game / no-messages state for 11.2/14.2/14.6. Verified `TestAppBuilder` has `skip_seeding` field. If not usable for HTTP, fall back to `storage.delete_game()` post-build (pattern used by existing `test_retry_handler_requires_context`).
- `post_empty` (in test_helpers.rs) handles /swipe/new and /retrigger. Reuse, don't re-implement.
- Pilot `SCENARIO_RE = \d+\.\d+` matches new IDs 9.x–15.x and S1.6 — no regex change (split dropped R-prefix). User declined regex complication.
- R5.1/R5.2 (cancellation) stay unit-only — covered by `retry_tests.rs::test_retry_last_response_cancelled_at_phase_boundary` + `test_retrigger_event_cancels_cleanly`. Dropping from HTTP specs matches ticket 11 precedent.
- `test_movement_with_arrival_narration_retry` (old #9) drops as same-tier overlap with 9.2 — both assert movement-to-room2-on-retry. Ponytail: weaker test goes.
- 6 existing fragment.rs tests use the deterministic `try_claim` pre-claim pattern for concurrency — superior to `with_delay`. Move preserves it.
- Branch `testing-refactor`; no branch switch.

## NOT in scope

- Widening `validate_feature_spec.py` SCENARIO_RE for R-prefix IDs — user declined. New specs use N.M, pilot tracks as-is.
- HTTP E2E for R5.1/R5.2 cancellation — unit-only by STRATEGY.md.
- Touching `src/application/pipeline/retry_tests.rs` (unit tier, ticket 04).
- Browser tier (ticket 07/08). Nextest config (ticket 09). Codebase-wide audit (ticket 10).
- Second helper `app_with_narrator_and_data` — dropped; NPC-trigger tests use inline pattern.

## What already exists (reuse)

- `tests/http/test_helpers.rs` — `post_action`, `post_empty`, `wait_idle`, `app_with_narrator`. Extend with one helper, don't re-implement.
- `tests/http/fragment.rs` — 6 retry/retrigger handler tests (11.1, 11.2, 12.1, 14.1, 14.6, 15.1). Move + retag, don't rewrite.
- `tests/http/actions.rs` 1.4 — inline `TestAppBuilder + make_test_pipeline_with_mock_quantifier` pattern for NPC-trigger tests. Reuse for 9.5, 11.8, S1.6.
- `MockBackend` — `with_narrations`, `with_prompt_responses`, `with_fail`, `with_empty_response`, `with_trigger_narration_fail`, `with_delay`, `narration_started`/`trigger_started` sync primitives.
- `TestAppBuilder` — `last_trigger`, `log`, `skip_seeding`, `data`, `pipeline`, `build_with_state`.

## Failure modes

- 11.3/11.4 (no-snapshot 500) — handler maps `ApplicationError::internal` to 500 via `IntoResponse`. If status stays Idle (500 returned before async task), assert `StatusCode::INTERNAL_SERVER_ERROR` + `GenerationStatus::Error(msg)`. Verify against `check_retry_anchor` which returns `Err(ApplicationError::internal(...))` pre-spawn.
- 9.4 (edited input) — `message_service.save_state` may not propagate to the snapshot retry loads. If retry loads pre-edit snapshot, test fails; fix by saving a fresh `GameStateSnapshot` after mutation (mirror `retry_tests.rs::add_input_and_save`).
- Moved tests: if `try_claim` pre-claim pattern breaks after move (different AppState instance), fall back to in-test re-seed via `state.message_service.save_message_and_snapshot`.
- ID collision: if 9.x–15.x clashes with existing spec, pilot reports orphan/dup. Pre-check: existing IDs are 1.x–8.x only.

## Unresolved decisions

- None remaining. All findings resolved via plan_mode_question: S1.6 kept (regression guard), 6 existing tests moved+retagged (drop dups, no flake), one helper only.
