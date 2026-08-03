# Ticket 3 — Move action-pipeline flow tests to HTTP E2E + grow spec (flow scenarios only)

## Summary

Move 8 tests from `tests/integration/flow/sequence.rs` (component-tier, drives
`pipeline.execute_action()` directly) to HTTP E2E in a new
`tests/http/flow_sequence.rs` (drives `POST /action`, `POST /swipe/new`,
`POST /history/delete`, `POST /reset`). Grow `docs/specs/action_pipeline.md`
with **new** HTTP-framed S6 (sequencing), S7 (reset), S8 (delete) scenarios —
S1–S5 left untouched (their reframe is split off to a separate ticket 11).
Tag new tests with `SCENARIO:`. Delete `flow/sequence.rs` + unwire. Fix
`validate_feature_spec.py` scan dirs to match STRATEGY.md.

## Key Changes

- New file `chronicler_engine/tests/http/flow_sequence.rs` with 8 HTTP E2E
  tests. Register in `tests/http/mod.rs`.
- `docs/specs/action_pipeline.md`: append S6/S7/S8 HTTP-framed scenarios.
- Delete `chronicler_engine/tests/integration/flow/sequence.rs`; remove
  `mod flow_sequence;` line + `#[path]` from `tests/integration/mod.rs`.
- Update `chronicler_engine/scripts/validate_feature_spec.py` scan dirs:
  `tests/integration/` → `[tests/http/, src/, tests/integration/storage/]`.
- Create split-off ticket 11 (spec restructure) + record scope reduction on
  ticket 3. Close ticket 3; update map `Decisions so far`.

## Implementation

### Phase 0: Scope-boundary bookkeeping (1 SP)

- [ ] #### Task 0.1: Create split-off ticket 11 + record ticket 3 scope reduction (1 SP)
  - Create `.scratch/test-strategy-execution/issues/11-spec-restructure-http-observable-only.md`: child of map, type task (HITL), `Status: ready-for-agent`. Body: reframe S1.x/S2.x/S3.3/S3.4-success to HTTP; remove S3.1/S3.2/S4.x/S5.1/S5.2/I.5 from `action_pipeline.md`; remove 14 SCENARIO tags from `pipeline_tests.rs`, replace with plain `//` comments; no `snapshots.md`; verify `validate_feature_spec.py` clean.
  - Add `## Scope (revised)` block to `issues/03-action-pipeline-http-e2e.md`: S1–S5 reframe + tag removal split to ticket 11. Mark "spec rewritten in HTTP framing" acceptance as partial (flow scenarios only).
  - Add ticket 11 to `map.md` (child issue, no blocking edge — independent of ticket 3).
  - Update map `Not yet specified` if the spec-restructure fog graduates (it does — becomes ticket 11, remove from fog).

### Phase 1: Wiring + spec skeleton + pilot fix (4 SP)

- [ ] #### Task 1.1: Claim ticket + create test file skeleton (1 SP)
  - Set `Status: in-progress` in `issues/03-action-pipeline-http-e2e.md`.
  - Create `tests/http/flow_sequence.rs` with module doc + imports.
  - Use `test_utils::wait_for_condition_async` for idle polling (already in scope via `tests/http/mod.rs`); no new helper.
  - Inline `fn latest_state(state: &AppState) -> GameState` (2 lines: `load_or_fresh` + `load_messages_into_state`) — or call inline at each site.
  - Add `mod flow_sequence;` to `tests/http/mod.rs`.
  - `cargo check --test http` compiles.
- [ ] #### Task 1.2: Add flow spec scenarios to action_pipeline.md (2 SP)
  - S6.1 three-action sequence; S6.2 execute→retry→execute; S6.3 async sequence→retry.
  - S7.1 reset clears story-log history; S7.2 action after reset produces fresh input+narration.
  - S8.1 delete-last between actions (deleted narration stays absent); S8.2 delete mid-sequence; S8.3 retry after delete of last input does not leave state generating.
  - All HTTP-framed (Given/When/Then at POST /action, /swipe/new, /history/delete, /reset, GET /fragment/story-log).
  - Note: S7.1 history-clear is HTTP-observable; full state-restoration covered at unit tier (`catalogue_tests.rs::reset_replaces_current_game`).
- [ ] #### Task 1.3: Update `validate_feature_spec.py` scan dirs (1 SP)
  - `TESTS_DIR` single → list: `[tests/http, src, tests/integration/storage]`.
  - Update docstring (line 16 usage; line 5–6 "walks `tests/integration/**`").
  - `python chronicler_engine/scripts/validate_feature_spec.py` runs clean against current state (S1–S5 declared, 14 tags in `pipeline_tests.rs` covered, no orphans, no gaps).
  - No build-gate wiring (pilot not in `build.py`).

### Phase 2: Non-retry flow tests (5 SP)

- [ ] #### Task 2.1: test_three_actions_in_sequence → S6.1 (1 SP)
  - 3× `POST /action` (command=examine room / look around / check inventory); `wait_for_condition_async` after each.
  - Assert `load_messages()` has 3 Input + ≥2 Narration.
  - SCENARIO tag `S6.1`. Delete from `flow/sequence.rs`.
- [ ] #### Task 2.2: delete tests → S8.1, S8.2 (2 SP)
  - `test_sequential_execute_delete_execute` (S8.1): POST /action A → idle → POST /history/delete → POST /action B → idle → assert narration A absent, 2 Input present, narration B present.
  - `test_delete_mid_sequence` (S8.2): POST /action A → POST /action B → POST /history/delete (removes narration B) → POST /action C → assert 3 Input, narration B absent.
  - SCENARIO tags. Delete both from `flow/sequence.rs`.
- [ ] #### Task 2.3: reset tests → S7.1, S7.2 (2 SP)
  - `test_reset_clears_history_and_state` → S7.1: POST /action → idle → assert history non-empty → POST /reset → assert `load_messages()` empty.
  - `test_reset_then_execute_works` → S7.2: POST /action A → idle → POST /reset → POST /action B → idle → assert exactly 1 Input (B only) + 1 Narration.
  - SCENARIO tags. Delete both from `flow/sequence.rs`.

### Phase 3: Retry flow tests + finalize (5 SP)

- [ ] #### Task 3.1: retry tests → S6.2, S6.3 (2 SP)
  - `test_sequential_execute_retry_execute` (S6.2): POST /action A → idle → POST /swipe/new → idle → POST /action B → idle → assert 2 Input + ≥2 Narration.
  - `test_async_action_sequence_then_retry` (S6.3): POST /action A → idle → POST /action B → idle → POST /swipe/new → idle → assert 2 Input.
  - SCENARIO tags. Delete both from `flow/sequence.rs`.
- [ ] #### Task 3.2: test_delete_input_then_retry_fails_gracefully → S8.3 (new test, 1 SP)
  - Seed input + narration via POST /action → idle → POST /history/delete (clears last) → POST /swipe/new → assert response not 500 + `wait_for_condition_async` shows `!is_generating()` within 1s.
  - Validates delete-then-retry graceful path end-to-end (NOT covered by `test_retry_handler_valid_context_error`, which hits the early validation guard on fresh app).
  - SCENARIO tag S8.3. Delete from `flow/sequence.rs`.
- [ ] #### Task 3.3: Delete flow/sequence.rs + unwire + suite green (1 SP)
  - Delete `chronicler_engine/tests/integration/flow/sequence.rs`.
  - Remove `#[path = "flow/sequence.rs"] mod flow_sequence;` from `tests/integration/mod.rs`.
  - `cargo nextest run` green; guardrails 26/26; `validate_feature_spec.py` clean.
- [ ] #### Task 3.4: Close ticket 3 + update map (1 SP)
  - Post resolution comment to `issues/03-action-pipeline-http-e2e.md`; `Status: closed`.
  - Append `Decisions so far` line to `map.md`.

## Test Plan

- `cargo nextest run --test http` — 8 new tests pass.
- `cargo nextest run` — full suite green.
- `cargo nextest run --test infrastructure` — guardrails 26/26, SCENARIO-tag placement rule passes.
- `python chronicler_engine/scripts/validate_feature_spec.py` — no gaps, no orphans (S6/S7/S8 declared + covered in `tests/http/`).

## Per Task/Sub Task Validation Steps

- After each task: `cargo check --test http` (compiles).
- After each test-written task: `cargo nextest run --test http <test_name>`.
- After Phase 3.3: `cargo nextest run` full + `--test infrastructure` + `python chronicler_engine/scripts/validate_feature_spec.py`.
- After Phase 3.4: grep `map.md` for new Decisions line; grep `issues/03-*.md` for `Status: closed`.
- After Phase 0.1: `ls .scratch/test-strategy-execution/issues/11-*.md` exists; grep `issues/03-*.md` for `Scope (revised)`.

## Assumptions

1. **New file** `tests/http/flow_sequence.rs` (not appending to 1358-line `fragment.rs`).
2. **Reuse `test_utils::wait_for_condition_async`** for idle polling — already in scope via `tests/http/mod.rs`. No new helper. Inline `latest_state` kept (2 lines).
3. **No custom quantifier / multi-room map** for reset tests. Reset scenario scoped to HTTP-observable "history cleared + fresh action works." Full state-restoration (movement room reset) covered at unit tier (`catalogue_tests.rs::reset_replaces_current_game`). Multi-room map + custom quantifier + sidebar room-image assertion rejected as over-build (ponytail lite).
4. **`test_delete_input_then_retry_fails_gracefully` is NOT covered** by existing `test_retry_handler_valid_context_error` — different code paths (early validation guard vs `find_retry_anchor` → `persist_generation_error`). Task 3.2 writes a new HTTP test (S8.3).
5. **`/history/delete` deletes last message** — all delete tests qualify (deleted narration is always the last message at delete time).
6. **Observation via `message_service.load_messages()`** (`Vec<Message>`) — readable from `AppState` inside the test, same data `/fragment/story-log` renders. Acceptable HTTP-tier observation per STRATEGY.md overlap rule.
7. **Ticket 01 (retry code changes) is closed** → retry tests unblocked; no waiting.
8. **S1–S5 reframe is out of scope** — split off to ticket 11 (created in Phase 0). Ticket 3 appends S6/S7/S8 only; does not touch S1–S5 or existing `pipeline_tests.rs` SCENARIO tags.
9. **`validate_feature_spec.py` is not build-gated** (pilot, not in `build.py`). Task 1.3 fixes its scan dirs for correctness against STRATEGY.md; no build-gate wiring.
10. **Tracker is local-markdown** at `.scratch/test-strategy-execution/`. Ticket 11 created as a new file in `issues/`; map updated in Phase 0.1 and Phase 3.4.
