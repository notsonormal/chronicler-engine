# Move lifecycle flow tests to HTTP E2E and grow the games spec

Type: task (HITL)
Status: closed
Graduated from: 06
Asset: [lifecycle-arrival-disposition.md](../assets/lifecycle-arrival-disposition.md)

## Resolution

Implemented Option B: three endpoint-named specs (`games_create.md`, `games_switch.md`, `games_delete.md`) with 8 scenarios total (3 + 2 + 3). Consolidated HTTP E2E coverage by moving 5 existing untagged tests from `tests/http/fragment.rs`, porting 2 from `tests/http/games_fragment_handlers.rs` with tightened assertions, adding 1 new idempotent-delete test, and deleting the non-branch cross-world switch test. Inline form-POST boilerplate; no new helper.

- Specs: `docs/specs/games_create.md` (3), `docs/specs/games_switch.md` (2), `docs/specs/games_delete.md` (3).
- Tests: `tests/http/games_create.rs` (3), `tests/http/games_switch.rs` (2), `tests/http/games_delete.rs` (3).
- Deletions: 6 tests from `fragment.rs` (5 moved + 1 non-branch), 2 from `games_fragment_handlers.rs` (ported).
- Validator: `52 declared, 52 covered, 0 gaps, 0 orphans`.
- Suite: `cargo nextest run` 1348 passed, 2 skipped; guardrails 101/101; clippy clean.

## Question

Grow the `games.md` spec for the three games CRUD endpoints and add the
HTTP E2E tests that validate it. HITL because the spec scenarios need
human review (Option A vs Option B naming; scenario phrasing).

## Scope

Three HTTP endpoints, all on the `games` resource:

- `POST /games` — create game (returns `ok_refresh()` on success)
- `POST /games/:id/switch` — switch game (returns `ok_refresh()`)
- `POST /games/:id/delete` — delete game (returns `ok("")`)

`POST /reset` already has a spec (`docs/specs/reset.md`, S7.x) — not in
scope here.

## Spec

Create `docs/specs/games.md` per §3 of the asset. Seven scenarios:

- 9.1 create-success (200 + HX-Refresh)
- 9.2 world-not-found (400, "World not found")
- 9.3 persona-not-found (400, "Persona not found")
- 10.1 switch-success (200 + HX-Refresh)
- 10.2 switch-unknown (400, "Game not found")
- 11.1 delete-success (200)
- 11.2 delete-active (400, "Cannot delete the active game")
- 11.3 delete-unknown (200, idempotent)

**Naming decision (HITL):** recommend Option A (one `games.md` with three
subsections — 9.x create, 10.x switch, 11.x delete) for CRUD cohesion.
Option B (three endpoint-named specs `games_create.md` /
`games_switch.md` / `games_delete.md`) is the strict alternative per
ticket 11's "one endpoint per spec" rule. Pick A unless the human prefers
strict endpoint-naming.

Scenario IDs `9.x`–`11.x` (pilot dedups across `docs/specs/*.md`).
Format: Given/When/Then/And with hard line breaks (per
`validate_feature_spec.py` format check added in ticket 11).

## HTTP E2E tests

Create `tests/http/games.rs` with 7 tests, one per scenario, each tagged
`// [docs/specs/games.md] SCENARIO: N.N`:

- 9.1 — `POST /games` with valid `world_key` + `persona_key` → 200 +
  `HX-Refresh` header. Use `TestAppBuilder::default_test().build_with_state()`
  pattern from `tests/http/reset.rs`.
- 9.2 — `POST /games` with `world_key=no_such_world` → 400 + body
  mentions "World not found". Tighten the existing
  `tests/http/games_fragment_handlers.rs::test_create_game_handler_empty_world_key`
  (currently asserts loose `is_client_error() || is_server_error()`) to
  `assert_eq!(status, 400)` + body check, tag it 9.2, and **delete** the
  original from `games_fragment_handlers.rs`.
- 9.3 — `POST /games` with `persona_key=nonexistent` → 400 + "Persona
  not found". Port
  `games_fragment_handlers.rs::test_create_game_handler_validates_persona_key`
  into `games.rs`, tag 9.3, delete the original.
- 10.1 — create 2 games, `POST /games/{id1}/switch` → 200 + HX-Refresh.
- 10.2 — `POST /games/99999999/switch` → 400 + "Game not found".
- 11.1 — create 2 games, `POST /games/{id2}/delete` → 200.
- 11.2 — create 1 game, `POST /games/{id1}/delete` (active) → 400 +
  "Cannot delete the active game".
- 11.3 — `POST /games/99999999/delete` → 200 (idempotent).

Wire `mod games;` in `tests/http/mod.rs`.

### Helpers

`tests/http/test_helpers.rs` has `post_action` (url-encoded `command=...`)
and `post_empty` (no-body POST). Add a `post_form(app, uri, body: &str)`
helper for arbitrary url-encoded form POSTs (used by 9.x). Or inline the
form-POST boilerplate (matches existing
`games_fragment_handlers.rs` style — there it's inlined).

## Tier-rule check

These tests use real axum router + in-memory `Storage` +
`TestAppBuilder::default_test()`. LLM is not involved in catalogue
operations (no narration generated on create/switch/delete) — no
`MockBackend` needed. This is HTTP E2E per STRATEGY.md: "real axum
router, real or in-memory storage".

## Cleanup of `games_fragment_handlers.rs`

After porting 9.2/9.3 out:
- `test_create_game_handler_empty_world_key` → deleted (ported to
  `games.rs` as 9.2).
- `test_create_game_handler_validates_persona_key` → deleted (ported to
  `games.rs` as 9.3).
- Remaining tests in `games_fragment_handlers.rs` are `GET /fragment/games`
  (browser-tier rendering). **Leave them in place** — re-tiering those
  to `tests/browser/` is ticket 07/08's job (browser tier design +
  execution), not this ticket's.

## Acceptance

- `docs/specs/games.md` created with 7 scenarios (9.1–9.3, 10.1, 10.2,
  11.1–11.3), Option A or B per human review.
- `tests/http/games.rs` created with 7 tagged HTTP E2E tests.
- 2 tests ported out of `games_fragment_handlers.rs` (originals deleted).
- `mod games;` wired in `tests/http/mod.rs`.
- `validate_feature_spec.py` reports the new spec covered with 0 gaps, 0
  orphans, 0 format violations (final count: 52 declared, 52 covered).
- `cargo nextest run` green; test count unchanged (8 reorganized into 3 new
  files, 8 deleted from old files).
- Guardrails green: `tests/infrastructure/guardrails/` — SCENARIO tags
  only in `tests/http/`, etc.

## Notes for the agent

- Predecessor pattern: ticket 03 (action pipeline HTTP E2E) and ticket
  11 (spec restructure) did the same kind of work. Read
  `issues/03-action-pipeline-http-e2e.md` and
  `issues/11-spec-restructure-http-observable-only.md` for the
  established spec + HTTP test conventions.
- `ApplicationError` → HTTP status mapping (`src/adapters/driving/http/error.rs`):
  `Validation(msg)` → 400. All the 400 scenarios above are
  `ApplicationError::Validation` from `GameCatalogue`.
- `ok_refresh()` adds `HX-Refresh: true` header — assert via
  `response.headers().get("hx-refresh").is_some()`.
- Blocked by: nothing. Independent of tickets 04, 05, 07, 08, 09, 10.
  May run in parallel with ticket 12 (unit). The unit ticket adds branch
  coverage; this ticket adds spec coverage — they don't overlap.
