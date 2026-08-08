# Ticket 13: Grow games CRUD specs and consolidate HTTP E2E tests (Option B, revised)

## Summary

Create three endpoint-named specs (`games_create.md`, `games_switch.md`, `games_delete.md`) for the 7 games CRUD scenarios (9.1–9.3 create, 10.1–10.2 switch, 11.1–11.3 delete). Consolidate HTTP E2E tests into three matching endpoint-named test files: **move 5** existing untagged tests out of `tests/http/fragment.rs`, **port 2** from `tests/http/games_fragment_handlers.rs` with tightened assertions, **write 1** new (11.3 idempotent delete), **delete 1** non-branch test (`test_switch_game_handler_cross_world_allowed`). Inline form-POST boilerplate (no new helper). Strengthen 10.2 + 11.2 body assertions. Validate with `validate_feature_spec.py` + `cargo nextest run`.

**Human decisions locked:**
- Option B: three endpoint-named specs (strict consistency with `actions.md` / `reset.md` / `story_log.md` / `swipe_new.md` / `retrigger.md`) + three matching endpoint-named test files (ticket 11 rule).
- Finding 1: no `post_form` helper — inline (matches 6+ existing sites).
- Finding 2: move + tag the 5 existing tests from `fragment.rs`; delete the cross-world non-branch test.
- Finding 3: preserve moved tests' `Storage` + `.build()` setup style; strengthen 10.2/11.2 body assertions.

## Key Changes

- **New specs (3 files in `chronicler_engine/docs/specs/`):**
  - `games_create.md` — 9.1 create-success, 9.2 world-not-found 400, 9.3 persona-not-found 400
  - `games_switch.md` — 10.1 switch-success, 10.2 switch-unknown 400
  - `games_delete.md` — 11.1 delete-success, 11.2 delete-active 400, 11.3 delete-unknown idempotent 200
  - Format: `#### Scenario N.N: title` heading + ```gherkin fenced block, matching `reset.md` / `story_log.md`.
- **New test files (3 in `chronicler_engine/tests/http/`):**
  - `games_create.rs` — 3 tests (9.1 moved from `fragment.rs::test_create_game_handler`; 9.2 + 9.3 ported from `games_fragment_handlers.rs` with tightened `assert_eq!(status, 400)` + body-contains checks).
  - `games_switch.rs` — 2 tests (10.1 + 10.2 moved from `fragment.rs`; 10.2 strengthened to add body-contains `"Game not found"`).
  - `games_delete.rs` — 3 tests (11.1 + 11.2 moved from `fragment.rs`; 11.2 strengthened to add body-contains `"Cannot delete the active game"`; 11.3 new — `POST /games/99999999/delete` → 200).
- **Deletions:**
  - `tests/http/fragment.rs`: remove `test_create_game_handler`, `test_switch_game_handler_success`, `test_switch_game_handler_not_found`, `test_switch_game_handler_cross_world_allowed` (non-branch, asset §1 row 7), `test_delete_game_handler_success`, `test_delete_game_handler_active_game` — 6 tests removed.
  - `tests/http/games_fragment_handlers.rs`: remove `test_create_game_handler_empty_world_key`, `test_create_game_handler_validates_persona_key` — 2 tests removed (ported to `games_create.rs` as 9.2/9.3).
  - Remove now-unused imports from both files.
- **Wiring:** `mod games_create; mod games_switch; mod games_delete;` added to `tests/http/mod.rs`.
- **No new helper:** form-POST inlined per existing convention (6+ sites in `worlds_fragment_handlers.rs` + `games_fragment_handlers.rs`).

## Implementation

### Task 1: Create three endpoint-named specs (3 SP)

- [ ] ##### SubTask 1.1: Write `docs/specs/games_create.md` (1 SP)
  - Header: `# Feature Spec: Games Create` / `Endpoint: POST /games`. Intro paragraph + scenario-ID note (9.x, stable, pilot dedups).
  - 3 scenarios (9.1/9.2/9.3) per asset §3, in ```gherkin blocks. 9.2 asserts 400 + body mentions `"World not found"`; 9.3 asserts 400 + body mentions `"Persona not found"`.
- [ ] ##### SubTask 1.2: Write `docs/specs/games_switch.md` (1 SP)
  - Header: `Endpoint: POST /games/:id/switch`. 2 scenarios (10.1/10.2). 10.2 asserts 400 + body mentions `"Game not found"`.
- [ ] ##### SubTask 1.3: Write `docs/specs/games_delete.md` (1 SP)
  - Header: `Endpoint: POST /games/:id/delete`. 3 scenarios (11.1/11.2/11.3). 11.2 asserts 400 + body mentions `"Cannot delete the active game"`. 11.3 notes idempotent semantics (`DELETE FROM games WHERE id=?` silent on no-row match).

### Task 2: Create `tests/http/games_create.rs` (3 SP)

- [ ] ##### SubTask 2.1: Move 9.1 from `fragment.rs` (1 SP)
  - Cut `test_create_game_handler` (L475) from `fragment.rs`. Paste into `games_create.rs`. Preserve its `Storage::new_in_memory()` + `TestWorld::minimal()` + `TestMap::single_room("start")` + `TestPersona::standard()` + `storage.seed_world/seed_persona/create_game/set_game_id` + `TestAppBuilder::default_test().storage(Arc::clone(&storage)).build()` setup. Preserve all assertions (200 + HX-Refresh + snapshot + messages + swipe — superset of spec). Add `// [chronicler_engine/docs/specs/games_create.md] SCENARIO: 9.1` tag above `#[tokio::test]`.
- [ ] ##### SubTask 2.2: Port 9.2 from `games_fragment_handlers.rs` (1 SP)
  - Cut `test_create_game_handler_empty_world_key`. Paste into `games_create.rs`. Tighten: `assert_eq!(status, 400)` (was loose `is_client_error()||is_server_error()`) + assert body contains `"World not found"`. Inline form-POST (preserve existing inline style). Tag `SCENARIO: 9.2`. Use `world_key=no_such_world&persona_key=test_player` per spec 9.2 (unknown-world path; same `validation("World not found")` branch as empty-key).
- [ ] ##### SubTask 2.3: Port 9.3 from `games_fragment_handlers.rs` (1 SP)
  - Cut `test_create_game_handler_validates_persona_key`. Paste into `games_create.rs`. Already asserts `assert_eq!(status, 400)` + body contains `"Persona not found"` — keep. Inline form-POST. Tag `SCENARIO: 9.3`. Use `world_key=test&persona_key=no_such_persona`.

### Task 3: Create `tests/http/games_switch.rs` (3 SP)

- [ ] ##### SubTask 3.1: Move 10.1 from `fragment.rs` (1 SP)
  - Cut `test_switch_game_handler_success` (L~570). Paste into `games_switch.rs`. Preserve setup style (storage + 2 games + `set_game_id(initial_game_id)`, then `POST /games/{other_id}/switch`). Preserve assertions (200 + HX-Refresh + `current_game_id` changed). Tag `SCENARIO: 10.1`.
- [ ] ##### SubTask 3.2: Move + strengthen 10.2 from `fragment.rs` (2 SP)
  - Cut `test_switch_game_handler_not_found` (L~605). Paste into `games_switch.rs`. Strengthen: keep `assert_eq!(status, 400)` + add body-contains `"Game not found"` assertion (spec 10.2 requirement; `render_error` wraps as `<div class="error-message">Error: Game not found</div>`). Tag `SCENARIO: 10.2`.

### Task 4: Create `tests/http/games_delete.rs` (3 SP)

- [ ] ##### SubTask 4.1: Move 11.1 from `fragment.rs` (1 SP)
  - Cut `test_delete_game_handler_success` (L~708). Paste into `games_delete.rs`. Preserve setup + assertions (200 + `storage.get_game(other_id).is_none()`). Tag `SCENARIO: 11.1`.
- [ ] ##### SubTask 4.2: Move + strengthen 11.2 from `fragment.rs` (1 SP)
  - Cut `test_delete_game_handler_active_game` (L~750). Paste into `games_delete.rs`. Strengthen: keep `assert_eq!(status, 400)` + add body-contains `"Cannot delete the active game"`. Tag `SCENARIO: 11.2`.
- [ ] ##### SubTask 4.3: Write new 11.3 test (1 SP)
  - `let app = TestAppBuilder::default_app();` (or `default_test().build()` — no setup needed, unknown id). `post_empty(&app, "/games/99999999/delete")`; assert `assert_eq!(status, 200)` (idempotent). Tag `SCENARIO: 11.3`.

### Task 5: Delete cross-world test + remove ported originals + wire mods (3 SP)

- [ ] ##### SubTask 5.1: Delete cross-world test from `fragment.rs` (1 SP)
  - Cut `test_switch_game_handler_cross_world_allowed` (L~655). Asset §1 row 7: `switch_game` has no world-validation branch; test asserts an absence of a check that doesn't exist. Non-branch. Delete, do not move.
- [ ] ##### SubTask 5.2: Remove ported tests from `games_fragment_handlers.rs` (1 SP)
  - `test_create_game_handler_empty_world_key` and `test_create_game_handler_validates_persona_key` already cut in Task 2. Confirm file has only the 3 `GET /fragment/games` tests remaining. Remove now-unused imports (`Body`, `Request`, `Method` if unreferenced).
- [ ] ##### SubTask 5.3: Clean `fragment.rs` imports + wire 3 new mods (1 SP)
  - After removing 6 tests from `fragment.rs`, remove now-unused imports (`Storage`, `TestWorld`, `TestMap`, `TestPersona`, `MessageType`, etc. — verify with `cargo check` before deleting each).
  - Add `mod games_create; mod games_switch; mod games_delete;` to `tests/http/mod.rs` (alphabetical, near `mod games_fragment_handlers;`).

### Task 6: Validate (3 SP)

- [ ] ##### SubTask 6.1: Run spec validator (1 SP)
  - `python chronicler_engine/scripts/validate_feature_spec.py` → expect `52 declared, 52 covered, 0 gaps, 0 orphans` (baseline 52 from ticket 11's 17 + ticket 05's 17 + ticket 08's 16 browser + story_log 3 = need to recheck baseline; actual: count current `docs/specs/*.md` scenarios first). At minimum: +7 new games scenarios covered, 0 gaps, 0 orphans.
- [ ] ##### SubTask 6.2: Run nextest (1 SP)
  - `cargo nextest run` → green. Net test-count change: **+1** (7 new tagged tests across 3 files − 6 deleted from `fragment.rs` − 2 deleted from `games_fragment_handlers.rs` + 1 new 11.3 = wait, recompute: moved tests don't change count. 5 moved (net 0) + 2 ported (net 0) + 1 new (11.3, +1) + 1 deleted (cross-world, −1) = **net 0**. Test count unchanged.) **Correction: net 0 tests.** Validator still gains +7 covered scenarios (previously untagged, now tagged).
- [ ] ##### SubTask 6.3: Guardrails + warnings (1 SP)
  - Guardrail tests green. `cargo build` / `cargo clippy` → 0 warnings.

## Test Plan

- `python chronicler_engine/scripts/validate_feature_spec.py` — 0 gaps, 0 orphans, +7 covered scenarios (9.1–9.3, 10.1, 10.2, 11.1–11.3).
- `cargo nextest run` — full suite green, net 0 test-count change (5 moved + 2 ported + 1 new − 1 deleted cross-world − 2 deleted originals = 0).
- Guardrails green: SCENARIO tags only in `tests/http/` + `tests/browser/behaviour.rs`; 3 new files compliant.
- 0 build warnings.
- Spot-check 10.2 + 11.2 body assertions actually contain the expected strings (wrap in `render_error` → `<div class="error-message">Error: ...</div>`; body-contains works).

## Per Task/Sub Task Validation Steps

- After Task 1: `ls chronicler_engine/docs/specs/games_*.md` = 3 files; `grep -c "^#### Scenario"` = 3/2/3.
- After Tasks 2–4: each new test file compiles (`cargo check --tests`); each has `// SCENARIO:` tags above `#[tokio::test]`.
- After Task 5: `grep -c "test_create_game_handler\|test_switch_game_handler\|test_delete_game_handler" tests/http/fragment.rs` = 0; `grep "mod games_" tests/http/mod.rs` = 4 lines; `games_fragment_handlers.rs` has 3 tests remaining (the `GET /fragment/games` ones).
- After Task 6: validator `0 gap(s), 0 orphan(s)`; nextest exit 0; clippy clean.

## Assumptions

- **Setup style preserved:** moved tests keep `Storage::new_in_memory()` + manual seed + `.storage(Arc::clone).build()` (not `build_with_state()`). `storage` direct access genuinely needed for 10.1/11.1 to create the "other" game without HTTP.
- **No `build_with_state()` rewrite:** keeps diff small, avoids introducing a second style. 9.1 could use either but stays consistent with siblings.
- **9.1 assertions are a superset of spec 9.1:** 200 + HX-Refresh + snapshot + messages + swipe count. STRATEGY.md overlap rule permits richer same-tier assertions. No trim.
- **10.2 + 11.2 strengthened:** existing tests only assert status; spec requires body-contains. `render_error` wraps as `<div class="error-message">Error: <msg></div>` — body-contains works (existing 9.3 test in `games_fragment_handlers.rs` proves the pattern).
- **11.3 uses `default_app()`:** no setup needed — unknown id, no world/persona required. Simpler than `default_test()`.
- **Body-text assertions:** `ApplicationError::Validation(msg)` → 400 + `render_error(msg)` body. Confirmed in `src/adapters/driving/http/error.rs` + `utils/error.rs`. Messages: `"World not found"`, `"Persona not found"`, `"Game not found"`, `"Cannot delete the active game"` (from `catalogue.rs`).
- **Net test count 0:** 5 moved + 2 ported + 1 new (11.3) − 1 deleted (cross-world) − 2 deleted (ported originals) = 0. Original plan's "+5" / "+7" both wrong — they miscounted moves as additions.
- **Cross-world test deletion rationale:** asset §1 row 7 — `switch_game` has no world-validation branch; test asserts an absence of a check that doesn't exist. Non-branch, redundant. User-approved (Q2b=A).
- **Out of scope:** re-tiering the 3 remaining `GET /fragment/games` tests in `games_fragment_handlers.rs` (ticket 07/08's job); renaming `games_fragment_handlers.rs`; lifecycle unit work (ticket 12, already landed uncommitted); any other tests in `fragment.rs` (large catchall, future tickets).

## NOT in scope

- Re-tiering `GET /fragment/games` tests (browser tier — ticket 07/08).
- Renaming `games_fragment_handlers.rs` or `fragment.rs`.
- Lifecycle unit work (ticket 12).
- Other untagged tests in `fragment.rs` (future tickets).
- Exhaustive branch coverage of `GameCatalogue` (map's "Not yet specified" — future effort).

## What already exists

- `tests/http/fragment.rs` — 5 of 7 games scenarios, untagged (move + tag).
- `tests/http/games_fragment_handlers.rs` — 9.2/9.3 tests with loose assertions (port + tighten).
- `tests/http/test_helpers.rs::post_empty` — reusable for 10.x/11.x (no-body POSTs).
- Inline form-POST pattern (6+ sites in `worlds_fragment_handlers.rs` + `games_fragment_handlers.rs`) — reuse, no new helper.
- `TestWorld::minimal()` / `TestMap::single_room()` / `TestPersona::standard()` — existing test fixtures used by moved tests.
- `reset.md` / `story_log.md` — spec format templates.

## Failure modes

- **Moved test fails to compile in new file:** missing import (e.g. `Storage`, `TestWorld`). → `cargo check --tests` after each move; fix imports.
- **10.2/11.2 body-contains fails:** `render_error` wraps message but escapes HTML. Raw message text (no HTML chars) survives. Verified by existing 9.3 test. If it fails, check `render_error` output.
- **11.3 returns non-200:** `delete_game` returns `Ok(())` for unknown id (only active-game check rejects). If fails, investigate `storage.delete_game` behavior on missing id.
- **Validator reports orphans:** tag format mismatch. Check `// [path] SCENARIO: N.N` exactly matches `SCENARIO_COMMENT_RE` in `validate_feature_spec.py`.
- **Unused imports after deletions:** `cargo check --tests` warns; remove imports incrementally.

## Unresolved decisions

None. All decisions locked via Finding 1 (no helper), Finding 2 (move+tag, delete cross-world), Finding 3 (preserve setup, strengthen 10.2/11.2).
