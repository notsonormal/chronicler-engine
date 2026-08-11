# Classify lifecycle, sequencing, and arrival tests — disposition and spec-worthiness

Type: research (AFK)
Status: closed
Blocked by: 01 (cleared — ticket 01 landed)
Resolved: 2026-08-04
Asset: [lifecycle-arrival-disposition.md](../assets/lifecycle-arrival-disposition.md)

## Scope (revised after ticket 11)

Ticket 03 moved `flow/sequence.rs` (8 tests) to `tests/http/flow_sequence.rs`.
Ticket 11 dissolved `flow_sequence.rs` into three endpoint-named files:
`tests/http/actions.rs` (S6 sequencing), `tests/http/reset.rs` (S7),
`tests/http/story_log.rs` (S8). Specs: `actions.md`, `reset.md`,
`story_log.md`.

**The sequencing portion of this ticket is DONE.** This ticket now covers
only the two remaining component-tier areas not yet classified:

- **`lifecycle.rs` (10 tests)** — Game create, reset, switch, delete.
- **`flow/arrival_persistence.rs` (3 tests)** — Arrival narration
  persistence across state reload.

## Question

Classify each of the 13 remaining tests and decide whether each
component earns a spec, producing a disposition artifact that
implementation tickets graduate from.

Blocked by ticket 01 because some sequencing tests involve retry (but
those are now handled by ticket 11 — this blocker may be stale for the
remaining 13 tests; reassess).

### What to produce

A markdown asset (linked from the resolution) that classifies each of
the 21 tests:

- **Port down to unit** — branch-coverage question asserted on internal
  state with fakes (validation branches, error paths, ordering
  invariants).
- **Move up to HTTP E2E** — spec scenario observed through the endpoint
  (happy path, error responses, multi-call sequences).
- **Delete** — redundant with existing coverage at another tier.
- **Drift** — the test asserts something that doesn't match the settled
  behaviour; flag for spec resolution.

For each component, answer:

- **Spec-worthy?** Enough behavioural surface (multiple scenarios,
  failure modes, invariants) to earn its own spec? Or too small (fold
  into an existing spec, or no spec needed)?
- **If spec-worthy, what scenarios?** Sketch the Given/When/Then
  scenarios at the HTTP level (same format as action_pipeline / retry
  specs).
- **What's the unit-vs-E2E split?** Which tests are branch-coverage
  (unit) and which are spec scenarios (HTTP E2E)?

### The two areas (scope reduced)

**lifecycle.rs (10 tests):** Game create, reset, switch, delete — all
map to HTTP endpoints. Validation branches: nonexistent game, can't-
delete-active, world mismatch, name uniqueness, reset-without-existing.
Likely spec-worthy (CRUD with validation). Check overlap with the
`GameCatalogue` unit gap filled by ticket 02.

**flow/arrival_persistence.rs (3 tests):** Arrival narration persistence
across state reload. Only 3 tests — may be too small for its own spec.

~~`flow/sequence.rs` (8 tests):~~ DONE via tickets 03 + 11 (sequencing
split into `actions.md` S6.x, `reset.md` S7.x, `story_log.md` S8.x).

### Acceptance

- Every test classified with a destination tier and rationale.
- Each component's spec-worthiness decided, with sketched scenarios if
  yes.
- The asset is linked from the resolution; implementation tickets
  graduate from it.
- Drift cases flagged for spec resolution.

## Resolution

All 13 tests classified; asset at
[lifecycle-arrival-disposition.md](../assets/lifecycle-arrival-disposition.md).
Net: 5 port down, 8 delete, 1 new spec (`games.md`), 0 drift.

**lifecycle.rs (10 tests) — 8 delete, 2 port down:**

- **Delete (8)** as redundant with existing unit coverage in
  `src/application/games/catalogue_tests.rs` (12 tests) or
  `src/domain/model/game_tests.rs`: #2 `test_reset_creates_scenario_message`
  (shared-helper coverage from #1's port-down; reset.md S7.1 covers HTTP),
  #3 `test_switch_game_loads_correct_state` (snapshots keyed by game_id;
  `switch_game` only calls `set_game_id`, never deletes a snapshot), #4
  `test_switch_to_nonexistent_game` (covered by `switch_game_errors_when_game_missing`),
  #5 `test_reset_without_existing_game` (happy-path reset covered), #6
  `test_create_game_name_uniqueness` (suffix pattern covered by
  `game_tests.rs::test_generate_game_name_*`; uniqueness by
  `create_game_generates_unique_names`), #7 `test_switch_game_world_mismatch`
  (not a real branch — `switch_game` has no world-validation), #8
  `test_delete_game_removes` (covered by `delete_game_removes_non_active_game`),
  #9 `test_delete_game_active_rejected` (covered by
  `delete_game_errors_when_deleting_active_game`).
- **Port down (2 net-new branches):**
  - From #1 `test_create_game_with_scenario`: add
    `create_game_persists_scenario_message_and_swipe` to
    `catalogue_tests.rs` (asserts `persist_initial_state_with_swipes` saves
    snapshot + scenario message + swipe). Also covers #2's reset path
    (shared helper).
  - From #10 `test_delete_game_nonexistent`: add
    `delete_game_succeeds_silently_for_nonexistent_game` to
    `catalogue_tests.rs` (idempotent `DELETE FROM games WHERE id=?`).

**arrival_persistence.rs (3 tests) — all 3 port down:**

All 3 are branch coverage of `ArrivalTaskContext::run`
(`src/application/arrival_service.rs`), not HTTP-observable (spawned from
`bootstrap/run.rs` at server startup, not on any HTTP call). Move to new
`src/application/arrival_service_tests.rs` with in-memory storage (both
ports faked per STRATEGY.md):
- #1 `test_arrival_narration_survives_reload` → `run_produces_and_persists_narration`
  (drop the reload round-trip — covered by `message_service_tests.rs` at
  1unit and `message_storage.rs` at driven-adapter; just assert narration
  produced + persisted once).
- #2 → `run_falls_back_to_fresh_state_on_load_failure` (already in-memory).
- #3 → `run_returns_early_without_narration_on_world_fetch_failure`
  (already in-memory).

**Spec-worthiness:**

- **`lifecycle.rs` / games CRUD: SPEC-WORTHY.** Three HTTP endpoints (`POST
  /games` create, `POST /games/:id/switch`, `POST /games/:id/delete`) form
  a CRUD surface with validation + idempotent-delete behaviour. 7
  HTTP-observable scenarios: 9.1 create-success (HX-Refresh), 9.2
  world-not-found 400, 9.3 persona-not-found 400, 10.1 switch-success
  (HX-Refresh), 10.2 switch-unknown-400, 11.1 delete-success 200, 11.2
  delete-active 400, 11.3 delete-unknown 200 (idempotent). Recommend one
  `docs/specs/games.md` with three subsections (9.x create, 10.x switch,
  11.x delete) — CRUD cohesion; 3 endpoint-named specs (Option B) is the
  strict alternative. Sketched Given/When/Then in §3 of the asset.
- **`arrival_persistence.rs`: NOT SPEC-WORTHY.** Bootstrap-triggered, not
  HTTP. Branch coverage at unit tier is the right home.

**Drift cases: 0.** No test asserts behaviour contradicting the settled
spec. The loose `is_client_error() || is_server_error()` assertion in
`tests/http/games_fragment_handlers.rs::test_create_game_handler_empty_world_key`
is weaker than 9.2 requires (`400 BAD_REQUEST`) but not a contradiction —
the test just needs tightening when 9.2 is tagged onto it.

**Uncovered `ArrivalTaskContext::run` branches (fog, not ticketed here):**
both-fail path (load fails + fresh fails), room_id-not-in-map early
return, arrival_preset-None Config-error, recorder-Err status,
save_message_and_snapshot failure path. Not for this ticket — the
destination is component tier dissolved, not exhaustive branch coverage.
Fog for the map's existing "Branch coverage pattern" entry; graduates as
its own ticket only if it turns out to be a pattern across
application-layer classes.

**Tickets that graduate:**
1. Lifecycle unit ticket (AFK): 2 new `catalogue_tests.rs` tests + 3
   ported `arrival_service_tests.rs` tests; delete `lifecycle.rs` +
   `arrival_persistence.rs` + unwire from `tests/integration/mod.rs`.
2. Lifecycle HTTP E2E + spec ticket (HITL — spec review): create
   `docs/specs/games.md` (7 scenarios) + `tests/http/games.rs` (7 tagged
   tests); port 9.2/9.3 from `games_fragment_handlers.rs` (tighten to
   `assert_eq!(status, 400)`) and delete originals; wire `mod games;` in
   `tests/http/mod.rs`; run `validate_feature_spec.py`.

**Blocking:** both graduating tickets are independent of each other and
of the retry tracks (04, 05). Can run in parallel with retry work.
