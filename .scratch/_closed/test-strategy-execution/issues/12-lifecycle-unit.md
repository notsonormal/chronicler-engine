# Port lifecycle + arrival unit tests down from the component tier

Type: task (AFK)
Status: resolved
Graduated from: 06
Asset: [lifecycle-arrival-disposition.md](../assets/lifecycle-arrival-disposition.md)

## Question

Dissolve the remaining component-tier files for the lifecycle and arrival
areas: port the net-new branch coverage down to unit tier, delete the
redundant tests, and unwire the files. Pure mechanical work — all
dispositions settled in ticket 06's asset.

## Scope

Two component-tier files to dissolve:

- `tests/integration/application/lifecycle.rs` (10 tests) — 8 delete, 2
  port down.
- `tests/integration/flow/arrival_persistence.rs` (3 tests) — all 3 port
  down.

## Work

### Unit ports down (5 new tests)

In `src/application/games/catalogue_tests.rs`, add:

1. `create_game_persists_scenario_message_and_swipe` — after
   `catalogue.create_game(&world_key, &persona_key)`, assert
   `storage.load_message_rows()` has exactly 1 `Narration` message and
   `storage.count_swipes_for_message(id) > 0`. Covers
   `persist_initial_state_with_swipes` (shared by `create_game` and
   `reset`). Use the existing `seeded_catalogue()` helper.

2. `delete_game_succeeds_silently_for_nonexistent_game` —
   `catalogue.delete_game(99999).expect("ok")`. Covers the idempotent
   branch (`storage.delete_game` is `DELETE FROM games WHERE id=?` —
   silent on no-row match).

Create `src/application/arrival_service_tests.rs` with the 3 ported
arrival tests (all in-memory, both ports faked — `MockBackend` + in-memory
`Storage`, per STRATEGY.md unit tier):

3. `run_produces_and_persists_narration` — ported from
   `test_arrival_narration_survives_reload`. Drop the reload round-trip
   (already covered by `message_service_tests.rs` at unit +
   `message_storage.rs` at driven-adapter). Assert:
   - `load_or_fresh().narrative.history` has at least 1 `Narration`.
   - `status` is `Idle`.
   - `storage.load_messages()` has the narration.
   - `storage.list_latest_llm_messages(50)` has a narrator row.
   Use `ArrivalTaskContext::new_for_test` with in-memory storage (not
   `SqliteTestAppBuilder`).

4. `run_falls_back_to_fresh_state_on_load_failure` — ported unchanged
   from `arrival_service_tests_falls_back_to_fresh_state_on_load_error`
   (already in-memory + failure-injected). Rename per convention.

5. `run_returns_early_without_narration_on_world_fetch_failure` — ported
   unchanged from
   `arrival_service_returns_early_without_narration_on_world_fetch_failure`.
   Rename per convention.

### Deletions

- `tests/integration/application/lifecycle.rs` (10 tests) — delete file.
- `tests/integration/flow/arrival_persistence.rs` (3 tests) — delete
  file.
- Unwire from `tests/integration/mod.rs`:
  - `#[path = "application/lifecycle.rs"] mod lifecycle;` (lines ~59-60)
  - `#[path = "flow/arrival_persistence.rs"] mod flow_arrival_persistence;`
    (line ~65)

### Not in scope

- `ArrivalTaskContext::run` has 5 uncovered branches (both-fail path,
  room_id-not-in-map early return, `arrival_preset`-None Config-error,
  recorder-Err status, `save_message_and_snapshot` failure path). Do NOT
  add tests for these — they are fog on the map's "Branch coverage
  pattern" entry, not part of this ticket. The destination is component
  tier dissolved, not exhaustive branch coverage.
- HTTP E2E for the games CRUD endpoints — separate ticket (13).
- `games_fragment_handlers.rs` cleanup — leave the `GET /fragment/games`
  tests in place; re-tiering those to browser is ticket 07/08's job.

## Acceptance

- 5 new unit tests added (2 in `catalogue_tests.rs`, 3 in new
  `arrival_service_tests.rs`).
- 13 component-tier tests deleted across 2 files; both files unwired
  from `tests/integration/mod.rs`.
- `cargo nextest run` green; test count drops by 8 net (13 deleted, 5
  added) — confirm the delta.
- No new `SCENARIO:` tags in `src/` (per STRATEGY.md — tags live in
  `tests/http/` only).
- Guardrails green (`tests/infrastructure/guardrails/`).

## Answer

- 5 new unit tests landed: `create_game_persists_scenario_message_and_swipe`, `delete_game_succeeds_silently_for_nonexistent_game` in `src/application/games/catalogue_tests.rs`; `run_produces_and_persists_narration`, `run_falls_back_to_fresh_state_on_load_failure`, `run_returns_early_without_narration_on_world_fetch_failure` in new `src/application/arrival_service_tests.rs`.
- `arrival_service_tests.rs` uses direct `MessageService::new` construction (no `WiredApp`/`AppState`/`pipeline` dead weight), and `run_produces_and_persists_narration` uses `TestDataBuilder::default_test()` so the scenario-inject branch is exercised (asserted `narrations.len() >= 2`).
- 13 component-tier tests deleted: `tests/integration/application/lifecycle.rs` and `tests/integration/flow/arrival_persistence.rs`; both unwired from `tests/integration/mod.rs`; empty directories removed.
- Final counts: **1348 passed, 2 skipped** (net -8 from ticket 08 baseline 1356/2); 26/26 guardrails; `validate_feature_spec.py`: 52/52; `build.py --coverage`: 89.4% total, `arrival_service.rs` 100%.

## Notes for the agent

- Predecessor pattern: ticket 02 (action pipeline unit) and ticket 04
  (retry unit) did the same kind of work. Read
  `issues/02-action-pipeline-unit.md` and `issues/04-retry-unit.md` for
  the established conventions on `persist_initial_state_with_swipes`
  coverage and handler-test style.
- `GameCatalogue::persist_initial_state_with_swipes` is private; test
  via the public `create_game` / `reset` (same as existing
  `catalogue_tests.rs` tests do).
- For `arrival_service_tests.rs`, the 3 existing tests already construct
  `ArrivalTaskContext::new_for_test` correctly — port the construction
  verbatim, only swap `SqliteTestAppBuilder` → in-memory `Storage` for
  test #1.
- Blocked by: nothing. Independent of tickets 04, 05, 07, 08, 09, 10.
  May run in parallel with ticket 13 (HTTP E2E + spec).
