# Lifecycle + arrival-persistence disposition

Asset for ticket 06. Classifies the 13 remaining component-tier tests
in `tests/integration/application/lifecycle.rs` (10) and
`tests/integration/flow/arrival_persistence.rs` (3), decides spec-worthiness
for each component, and sketches the HTTP E2E scenarios for the spec-worthy
one. Implementation tickets graduate from this asset.

Tier rules per `tests/STRATEGY.md`:
- **Unit** — both driven ports faked (`MockBackend` + in-memory `Storage`),
  asserts on internal state. Lives in `src/`, `*_tests.rs`.
- **HTTP E2E** — LLM faked, real axum router, real or in-memory storage.
  Spec validation through the driving adapter. Lives in `tests/http/`.
- **Driven-adapter** — nothing faked, real SQLite. Storage seam. Lives in
  `tests/integration/storage/`.

Overlap rule: each tier asserts what it can see; cross-tier overlap is
correct. The violation is same-tier duplication. The component-tier tests
below are being dissolved — they duplicate unit-tier assertions at a heavier
harness cost (real SQLite + `TestAppBuilder`), and the spec-observable
behaviour they exercise belongs at HTTP E2E.

Existing unit-tier coverage that the dispositions lean on
(`src/application/games/catalogue_tests.rs`, 12 tests):
`create_game_returns_positive_id`, `create_game_errors_when_world_missing`,
`create_game_errors_when_persona_missing`, `create_game_generates_unique_names`,
`create_game_restores_current_game_on_persist_failure`,
`switch_game_changes_current_game`, `switch_game_errors_when_game_missing`,
`delete_game_removes_non_active_game`, `delete_game_errors_when_deleting_active_game`,
`list_games_returns_all_games`, `reset_replaces_current_game`,
`current_game_id_matches_storage`.

Existing HTTP coverage for these endpoints (`tests/http/`):
- `tests/http/core.rs::test_reset_handler_failure_returns_internal_server_error`
  — POST /reset storage-failure → 500.
- `tests/http/games_fragment_handlers.rs::test_create_game_handler_empty_world_key`
  — POST /games with empty world_key → 4xx (actually 400 "World not found";
  assertion is loose `is_client_error() || is_server_error()`).
- `tests/http/games_fragment_handlers.rs::test_create_game_handler_validates_persona_key`
  — POST /games with unknown persona_key → 400 "Persona not found".
- `src/adapters/driving/http/games/handlers/games_tests.rs::test_switch_game_ok`
  — unit-tier handler test (calls `switch_game_handler` directly); not HTTP E2E.

HTTP routes (per `src/adapters/driving/http/builders/router.rs`):
- `POST /games` → `create_game_handler` (returns `ok_refresh()` on success)
- `POST /games/:id/switch` → `switch_game_handler` (returns `ok_refresh()`)
- `POST /games/:id/delete` → `delete_game_handler` (returns `ok("")`)
- `POST /reset` → `reset_handler` (already spec'd in `docs/specs/reset.md`)

`ApplicationError` → HTTP status (`src/adapters/driving/http/error.rs`):
`Validation(msg)` → 400 BAD_REQUEST; `Engine(_)` user-displayable → 500;
`Engine(_)` internal → 500 "Internal Server Error".

---

## 1. `tests/integration/application/lifecycle.rs` (10 tests)

All 10 use real SQLite via `DbPool::new(":memory:")` + `Storage::new_sqlite`
and `TestAppBuilder::with_data(...).build_service()`, then call
`app_service.game_catalogue.<method>(...)` directly (not through HTTP). This
is the dissolved component tier: calls pipeline/catalogue methods directly
on `AppState` with real SQLite and asserts on storage + `GameState`. The
branch-coverage questions belong at unit tier (`catalogue_tests.rs`); the
client-observable behaviour belongs at HTTP E2E.

| # | lifecycle.rs test | Asserts | Unit-tier counterpart (existing or proposed) | HTTP-observable? | Disposition | Notes |
|---|---|---|---|---|---|---|
| 1 | `test_create_game_with_scenario` | `create_game` ok; `current_game_id` == new id; `load_latest_snapshot` is Some; `load_message_rows` non-empty; first msg `Narration`; `count_swipes_for_message > 0` | `create_game_returns_positive_id` (id, keys, current_game_id) — **missing**: snapshot saved, scenario message + swipe persisted | Yes — POST /games success → 200 HX-Refresh (already half-covered by `games_fragment_handlers.rs` failure tests; no success HTTP test) | **Port down (partial) + spec** | Branch coverage: `create_game` calls `persist_initial_state_with_swipes` which saves snapshot + initial scenario message + swipe. That persistence is not asserted at unit tier. Add `create_game_persists_scenario_message_and_swipe` to `catalogue_tests.rs` (in-memory, assert `load_message_rows()` has 1 `Narration` with `count_swipes_for_message > 0` after `create_game`). The `current_game_id`/snapshot-existence bits are already covered or follow from the game being persisted. The HTTP success scenario graduates to the new `games.md` spec (see §3). |
| 2 | `test_reset_creates_scenario_message` | After `create_game` + `reset`: `load_message_rows` non-empty, first msg `Narration`, swipe count > 0 | `reset_replaces_current_game` (current_id changes, old game deleted) — **missing**: scenario message persisted after reset | Yes — POST /reset already spec'd (`docs/specs/reset.md` S7.1: "contains exactly 1 Narration entry (the fresh game's scenario opening)") | **Delete** | `reset` calls the same `persist_initial_state_with_swipes` as `create_game`. The "scenario message persisted" branch is covered by the proposed `create_game_persists_scenario_message_and_swipe` unit test (shared helper). HTTP E2E S7.1 already asserts the fresh-game narration count after reset. Lifecycle.rs version is redundant at both tiers. |
| 3 | `test_switch_game_loads_correct_state` | Two `create_game`s; switch back and forth; `current_game_id` matches; each game's `load_latest_snapshot` is Some | `switch_game_changes_current_game` (current_id changes) + proposed `create_game_persists_scenario_message_and_swipe` (snapshot saved per game) | Yes — POST /games/:id/switch success → 200 HX-Refresh | **Delete** | Snapshots are keyed by `game_id` (confirmed `save_snapshot`/`load_latest_snapshot` in `backend/snapshots.rs`); `switch_game` only calls `set_game_id` — it never deletes a snapshot. So "each game has a snapshot after switching" is implied by (a) + (b). No new branch. HTTP success scenario graduates to `games.md`. |
| 4 | `test_switch_to_nonexistent_game` | `switch_game(99999)` → err | `switch_game_errors_when_game_missing` (asserts `ApplicationError::Validation("Game not found")`) | Yes — POST /games/:id/switch with unknown id → 400 | **Delete** + **spec** | Unit-tier covers the branch. HTTP E2E scenario (400 "Game not found") graduates to `games.md` — missing HTTP test. |
| 5 | `test_reset_without_existing_game` | `create_game` then `reset` → ok | `reset_replaces_current_game` (reset succeeds) | Yes — POST /reset success → 200 HX-Refresh | **Delete** | Misleading name (the test does create_game first). Happy-path reset is covered at unit tier; HTTP E2E S7.1/S7.2 cover the spec. No new branch. |
| 6 | `test_create_game_name_uniqueness` | Two `create_game`s; 2 non-default games; names match `Test World_<date>_1` / `_2` suffix pattern | `create_game_generates_unique_names` (3 names unique) + `src/domain/model/game_tests.rs` (`test_generate_game_name_first`/`_increments`/`_max_plus_one` covers the exact suffix pattern) | No — name-uniqueness is not HTTP-observable (the response is just HX-Refresh; the name shows up in `/fragment/games` HTML, but that's a browser-tier rendering concern) | **Delete** | The suffix-pattern assertion is a domain-function concern, fully covered by `game_tests.rs`. The integration-level "two creates produce two distinct names" is covered by `create_game_generates_unique_names`. Redundant. |
| 7 | `test_switch_game_world_mismatch` | `create_game` in world_a; seed world_b via second `TestAppBuilder`; `switch_game(game_id_from_world_a)` → ok | None — but **not a real branch**: `switch_game` only checks `get_game(id).is_none()`; it never reads `world_key` | No — HTTP response is identical (200 HX-Refresh) regardless of world match | **Delete** | `switch_game` has no world-validation branch to cover. The test asserts an absence of a check that doesn't exist. Not a real branch, not a distinct HTTP scenario. (If world-mismatch validation is ever added, that's a new ticket — not this one.) |
| 8 | `test_delete_game_removes` | Two creates; delete one; `list_games` has 1 game; the right one remains | `delete_game_removes_non_active_game` (deletes non-active; `storage.get_game(id2).is_none()`) | Yes — POST /games/:id/delete success → 200 | **Delete** + **spec** | Unit-tier covers the branch. HTTP E2E success scenario graduates to `games.md` — missing HTTP test. |
| 9 | `test_delete_game_active_rejected` | `create_game`; delete active; err is `Validation` | `delete_game_errors_when_deleting_active_game` (asserts `Validation("Cannot delete the active game")`) | Yes — POST /games/:id/delete for active game → 400 | **Delete** + **spec** | Unit-tier covers the branch. HTTP E2E 400 scenario graduates to `games.md` — missing HTTP test. |
| 10 | `test_delete_game_nonexistent` | `delete_game(99999)` → ok (silent) | **None** — `catalogue_tests.rs` does not cover the idempotent-nonexistent branch | Yes — POST /games/:id/delete for unknown id → 200 (idempotent) | **Port down** + **spec** | `GameCatalogue::delete_game` delegates to `storage.delete_game` which is a `DELETE FROM games WHERE id=?` (silent on no-row match). This is a real branch: `delete_game` returns Ok for nonexistent ids (only the active-game check rejects). Add `delete_game_succeeds_silently_for_nonexistent_game` to `catalogue_tests.rs` (in-memory, assert `delete_game(99999).is_ok()`). HTTP E2E idempotent-200 scenario graduates to `games.md`. |

### Summary for lifecycle.rs (10 tests)

- **Delete (8):** #2, #3, #4, #5, #6, #7, #8, #9 — redundant with existing
  unit-tier coverage in `catalogue_tests.rs` / `game_tests.rs`, or assert
  non-branches. Delete on dissolve.
- **Port down (2 — the only net-new branch coverage):**
  - From #1: `create_game_persists_scenario_message_and_swipe` (new unit
    test in `catalogue_tests.rs`) — asserts the scenario message + swipe are
    persisted after `create_game`. Covers `persist_initial_state_with_swipes`.
    This single new unit test also covers #2's "reset persists scenario
    message" (shared helper), so #2 is Delete not port-down.
  - From #10: `delete_game_succeeds_silently_for_nonexistent_game` (new unit
    test in `catalogue_tests.rs`) — asserts idempotent delete on missing id.
- **Spec (new `games.md`):** the HTTP-observable behaviour of
  `POST /games`, `POST /games/:id/switch`, `POST /games/:id/delete` earns a
  spec (see §3).

---

## 2. `tests/integration/flow/arrival_persistence.rs` (3 tests)

All 3 call `ArrivalTaskContext::new_for_test(...)` directly and
`task_ctx.run_sync()` — **not** HTTP-driven. `ArrivalTaskContext` is
spawned only from `bootstrap/run.rs::spawn_arrival_task_if_needed` at
server startup (not on `POST /games` or any HTTP call), so the arrival
narration's *trigger* is not an HTTP surface. Per STRATEGY.md: "Scenarios
that can't be expressed through HTTP surfaces — because their Givens or
Thens touch seams that only exist in-process — live at the unit or
driven-adapter tier instead."

The tests are **branch coverage of `ArrivalTaskContext::run`**
(`src/application/arrival_service.rs`), which has ~10 branches: load
succeeds vs fails+fresh-fallback vs both fail; room_id not in map → early
return; arrival_preset None → Config-error status; recorder Ok → add
message + Idle; recorder Err → Error status; save_message_and_snapshot
failure path. The 3 tests cover the load-fail-fresh-fallback path (#2),
the get_world-fail early-return path (#3), and the happy path (#1).

The "survives reload" assertions in #1 are **not unique** — they're
already covered:
- `src/application/message_service_tests.rs::test_save_message_and_snapshot_assigns_snapshot_id_to_message`
  (unit, in-memory) — covers the snapshot-id ↔ message-id consistency.
- `tests/integration/storage/message_storage.rs` (driven-adapter) — covers
  `load_messages_with_swipes` / `insert_message` / `insert_swipe`
  round-trips.
- `src/adapters/driven/storage/backend/llm_messages_tests.rs` (unit) —
  covers `list_latest_llm_messages`.

So #1's real purpose is happy-path branch coverage of
`ArrivalTaskContext::run` — which can be done at unit tier with in-memory
storage (same harness as #2 and #3).

| # | arrival_persistence.rs test | Storage | Branch of `ArrivalTaskContext::run` | Disposition | Notes |
|---|---|---|---|---|---|
| 1 | `test_arrival_narration_survives_reload` | Real SQLite (`SqliteTestAppBuilder`) | Happy path: load succeeds → `inject_scenario_logs` (empty history) → room found → preset Some → recorder Ok → add message + Idle → save succeeds | **Port down** to unit | Rewrite as a unit test in `src/application/arrival_service_tests.rs` using in-memory `Storage` (both ports faked per STRATEGY.md). Assert: after `run_sync()`, `load_or_fresh().narrative.history` has 1 `Narration`; `status` is `Idle`; `load_messages()` has the narration; `list_latest_llm_messages` has the narrator row. The "survives reload" assertion is the mechanism — it's already covered at unit (`message_service_tests.rs`) and driven-adapter (`message_storage.rs`), so the ported test can drop the reload round-trip and just assert the narration was produced and persisted once. |
| 2 | `arrival_service_tests_falls_back_to_fresh_state_on_load_error` | In-memory + `load_latest_snapshot` failure injected | Load fails → `build_fresh_initial_state` succeeds → proceed → narration added | **Port down** to unit | Already at unit-tier harness (both ports faked). Move file to `src/application/arrival_service_tests.rs` unchanged in substance. Rename to `run_falls_back_to_fresh_state_on_load_failure`. |
| 3 | `arrival_service_returns_early_without_narration_on_world_fetch_failure` | In-memory + `get_world` failure injected | `require_world` fails → `run` returns Err → no narration added (state not saved) | **Port down** to unit | Already at unit-tier harness. Move to `src/application/arrival_service_tests.rs`. Rename to `run_returns_early_without_narration_on_world_fetch_failure`. |

### Uncovered branches of `ArrivalTaskContext::run` (fog for a future ticket, not this one)

The 3 ported tests cover 3 of ~10 branches. Uncovered:
- `build_fresh_initial_state` also fails after load fails (both-fail path).
- `room_id` not in map → early Ok return with no narration.
- `arrival_preset` None → Config-error status.
- Recorder returns Err → `GenerationStatus::Error`.
- `save_message_and_snapshot` fails (logged, returns Ok).

These are branch-coverage gaps at unit tier. They graduate as fog on the
map's "Not yet specified" → "Branch coverage pattern" entry: if the
component-tier dissolution reveals unit-branch gaps beyond `GameCatalogue`,
they earn their own ticket. **Do not ticket here** — the destination is
the component tier dissolved, not exhaustive branch coverage of every
application class. The gaps are noted for the future effort.

### Spec-worthiness for `arrival_service`

**Not spec-worthy.** The arrival narration's trigger is bootstrap, not
HTTP. The HTTP-observable effect (narration appears in story-log after
server start) is not expressible as a clean HTTP scenario — the Given
("server started with no-scenario world") is not an HTTP request. Branch
coverage at unit tier is the right home.

---

## 3. Spec for the games CRUD endpoints

**Spec-worthy.** Three HTTP endpoints form a CRUD surface on the `games`
resource, with validation paths and a distinct idempotent-delete
behaviour. Nine HTTP-observable scenarios across the three endpoints:

### Naming

Two options; **recommend A**, flag B for the implementer.

- **A. One `games.md` spec with three endpoint subsections** (recommended).
  Rationale: the three endpoints are CRUD on the same resource; a reader
  wants the whole surface in one place; the scenarios per endpoint are
  small (2–4 each), so three tiny specs is more files for less value.
  Scenario IDs: `9.x` (POST /games create), `10.x` (POST /games/:id/switch),
  `11.x` (POST /games/:id/delete). Test file: `tests/http/games.rs`
  (extend `tests/http/games_fragment_handlers.rs` or rename — see
  implementation note below).
- **B. Three endpoint-named specs** (`games_create.md`, `games_switch.md`,
  `games_delete.md`) — strict application of ticket 11's "one endpoint per
  spec" rule. More consistent with `actions.md` / `reset.md` /
  `story_log.md` but yields three tiny specs. Pick if the team prefers
  strict endpoint-naming over CRUD cohesion.

The pilot dedups scenario IDs across `docs/specs/*.md`, so IDs stay unique
either way.

### Sketched scenarios (Option A, endpoint-named subsections)

IDs follow the `reset.md` (7.x) / `story_log.md` (8.x) pattern; games CRUD
takes 9.x–11.x.

#### POST /games (create) — `9.x`

```
#### Scenario 9.1: Creating a game with valid world and persona returns success and refreshes
**Given** a seeded world with key "test-world" and a persona with key "test-persona"
**When** the client `POST /games` with `world_key=test-world&persona_key=test-persona`
**Then** the response status is `200 OK`
**And** the response has an `HX-Refresh` header (the games panel refreshes)

#### Scenario 9.2: Creating a game with an unknown world key returns 400
**Given** a seeded persona with key "test-persona"
**When** the client `POST /games` with `world_key=no_such_world&persona_key=test-persona`
**Then** the response status is `400 BAD_REQUEST`
**And** the response body mentions `"World not found"`

#### Scenario 9.3: Creating a game with an unknown persona key returns 400
**Given** a seeded world with key "test-world"
**When** the client `POST /games` with `world_key=test-world&persona_key=no_such_persona`
**Then** the response status is `400 BAD_REQUEST`
**And** the response body mentions `"Persona not found"`
```

(9.2 and 9.3 are already half-covered by
`tests/http/games_fragment_handlers.rs::test_create_game_handler_empty_world_key`
and `::test_create_game_handler_validates_persona_key` — those tests need
their assertions tightened to `assert_eq!(status, 400)` and tagged
`SCENARIO: 9.2` / `9.3`. 9.1 is new.)

#### POST /games/:id/switch — `10.x`

```
#### Scenario 10.1: Switching to an existing game returns success and refreshes
**Given** two created games with ids `id1` and `id2`, where `id2` is current
**When** the client `POST /games/{id1}/switch`
**Then** the response status is `200 OK`
**And** the response has an `HX-Refresh` header

#### Scenario 10.2: Switching to an unknown game id returns 400
**When** the client `POST /games/99999999/switch`
**Then** the response status is `400 BAD_REQUEST`
**And** the response body mentions `"Game not found"`
```

#### POST /games/:id/delete — `11.x`

```
#### Scenario 11.1: Deleting a non-active game returns success
**Given** two created games with ids `id1` (active) and `id2`
**When** the client `POST /games/{id2}/delete`
**Then** the response status is `200 OK`

#### Scenario 11.2: Deleting the active game returns 400
**Given** one created game with id `id1` that is the current game
**When** the client `POST /games/{id1}/delete`
**Then** the response status is `400 BAD_REQUEST`
**And** the response body mentions `"Cannot delete the active game"`

#### Scenario 11.3: Deleting an unknown game id returns success (idempotent)
**When** the client `POST /games/99999999/delete`
**Then** the response status is `200 OK`
```

### Implementation notes (for the graduating ticket)

- Test file: add `tests/http/games.rs` with the 7 spec-tagged scenarios
  above (9.1, 10.1, 10.2, 11.1, 11.2, 11.3 are new; 9.2 and 9.3 either
  port-and-tag from `games_fragment_handlers.rs` or are re-written in
  `games.rs` and the old ones deleted). Wire `mod games;` in
  `tests/http/mod.rs`.
- `tests/http/games_fragment_handlers.rs` currently mixes `GET
  /fragment/games` (browser-tier rendering) with `POST /games` (HTTP E2E
  for create). After porting 9.2/9.3 out, the remaining `GET
  /fragment/games` tests belong at browser tier (presentation) — but that
  re-tiering is ticket 07/08's job (browser tier), not this ticket's. Leave
  the fragment tests in place; just remove the two create-handler tests
  that are ported into `games.rs`.
- Helpers: `tests/http/test_helpers.rs` already has `post_action` and
  `post_empty`. Add a `post_form(app, uri, body)` helper for url-encoded
  form posts (used by 9.x). Or inline the form-POST boilerplate (matches
  the existing `games_fragment_handlers.rs` style).
- Tier-rule check: these tests use real axum router + in-memory storage +
  `TestAppBuilder::default_test()` (LLM not involved in catalogue
  operations — no narration generated on create/switch/delete). This is
  HTTP E2E per STRATEGY.md: "real axum router, real or in-memory storage".
  No `MockBackend` needed for these scenarios.
- `SCENARIO:` tags go on the HTTP tests in `tests/http/games.rs`, not on
  the unit tests in `catalogue_tests.rs`.

---

## 4. Net dispositions (one screen)

| Component | Tests | Disposition | Spec? |
|---|---|---|---|
| `lifecycle.rs` #1 `test_create_game_with_scenario` | Port down (partial: scenario-message-persistence branch) → `catalogue_tests.rs::create_game_persists_scenario_message_and_swipe` | + HTTP E2E 9.1 in `games.rs` | `games.md` 9.x |
| `lifecycle.rs` #2 `test_reset_creates_scenario_message` | Delete (shared-helper coverage from #1's port-down; S7.1 covers HTTP) | — | (reset.md, existing) |
| `lifecycle.rs` #3 `test_switch_game_loads_correct_state` | Delete (snapshots keyed by game_id; `switch_game` doesn't delete; covered by existing unit tests) | + HTTP E2E 10.1 | `games.md` 10.x |
| `lifecycle.rs` #4 `test_switch_to_nonexistent_game` | Delete (covered by `switch_game_errors_when_game_missing`) | + HTTP E2E 10.2 | `games.md` 10.x |
| `lifecycle.rs` #5 `test_reset_without_existing_game` | Delete (happy-path reset covered; S7.1/S7.2 cover HTTP) | — | (reset.md, existing) |
| `lifecycle.rs` #6 `test_create_game_name_uniqueness` | Delete (suffix pattern covered by `game_tests.rs`; uniqueness by `create_game_generates_unique_names`) | — | — |
| `lifecycle.rs` #7 `test_switch_game_world_mismatch` | Delete (not a real branch — `switch_game` doesn't check world) | — | — |
| `lifecycle.rs` #8 `test_delete_game_removes` | Delete (covered by `delete_game_removes_non_active_game`) | + HTTP E2E 11.1 | `games.md` 11.x |
| `lifecycle.rs` #9 `test_delete_game_active_rejected` | Delete (covered by `delete_game_errors_when_deleting_active_game`) | + HTTP E2E 11.2 | `games.md` 11.x |
| `lifecycle.rs` #10 `test_delete_game_nonexistent` | Port down → `catalogue_tests.rs::delete_game_succeeds_silently_for_nonexistent_game` | + HTTP E2E 11.3 | `games.md` 11.x |
| `arrival_persistence.rs` #1 `test_arrival_narration_survives_reload` | Port down → `src/application/arrival_service_tests.rs::run_produces_and_persists_narration` (in-memory) | — | No spec (bootstrap-triggered, not HTTP) |
| `arrival_persistence.rs` #2 `arrival_service_tests_falls_back_to_fresh_state_on_load_error` | Port down → `src/application/arrival_service_tests.rs::run_falls_back_to_fresh_state_on_load_failure` (already in-memory) | — | No spec |
| `arrival_persistence.rs` #3 `arrival_service_returns_early_without_narration_on_world_fetch_failure` | Port down → `src/application/arrival_service_tests.rs::run_returns_early_without_narration_on_world_fetch_failure` (already in-memory) | — | No spec |

**Totals:** 13 tests classified. **5 port down** (2 to
`catalogue_tests.rs`, 3 to new `src/application/arrival_service_tests.rs`).
**8 delete** (all in `lifecycle.rs`). **1 new spec** (`games.md`, 7
scenarios: 9.1–9.3, 10.1, 10.2, 11.1–11.3). **0 drift cases** — no test
asserts behaviour that contradicts the settled spec. The loose
`is_client_error() || is_server_error()` assertion in
`test_create_game_handler_empty_world_key` is weaker than the spec requires
(9.2 specifies `400 BAD_REQUEST`), but it's not a drift contradiction — the
test just needs tightening when the spec scenario is tagged onto it.

### Tickets that graduate from this asset

1. **Lifecycle unit ticket** (AFK task or unit-track implementation):
   - Add `create_game_persists_scenario_message_and_swipe` to
     `src/application/games/catalogue_tests.rs`.
   - Add `delete_game_succeeds_silently_for_nonexistent_game` to
     `src/application/games/catalogue_tests.rs`.
   - Create `src/application/arrival_service_tests.rs` with the 3 ported
     arrival tests (in-memory, both ports faked).
   - Delete `tests/integration/application/lifecycle.rs` (10 tests) and
     `tests/integration/flow/arrival_persistence.rs` (3 tests); unwire
     `mod lifecycle;` and `mod flow_arrival_persistence;` from
     `tests/integration/mod.rs`.
2. **Lifecycle HTTP E2E + spec ticket** (HITL — spec scenario review):
   - Create `docs/specs/games.md` with the 7 scenarios sketched in §3
     (9.1–9.3, 10.1, 10.2, 11.1–11.3), Option A (one spec, three
     subsections) unless the human picks Option B.
   - Create `tests/http/games.rs` with the 7 HTTP E2E tests; tag each
     with `// [docs/specs/games.md] SCENARIO: N.N`.
   - Port 9.2/9.3 from `games_fragment_handlers.rs` (tighten the status
     assertion to `assert_eq!(status, 400)`) and delete the originals.
   - Wire `mod games;` in `tests/http/mod.rs`.
   - Run `validate_feature_spec.py` (per `tests/AGENTS.md`) to confirm
     `games.md` scenarios are all covered with no orphans.

**Blocking:** the lifecycle unit ticket and the lifecycle HTTP E2E + spec
ticket are independent of each other and of tickets 04/05 (retry). They can
run in parallel with the retry tracks.
