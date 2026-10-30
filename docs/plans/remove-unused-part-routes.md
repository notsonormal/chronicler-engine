# Plan: Remove the part routes that the page does not call, and the second Action route

**Status:** Approved by the owner on 2026-10-30 (O1 = A, O2 = remove `/action/confirm`, per-stage commits). Not started.
**Goal:** Every screen part has one route, `GET /screen?parts=…`, and sending an Action has one route, `POST /action`. The tests use the routes that the page uses, so they check the answers that the page gets.
**Branch:** `wf/change-driven-dashboard-refresh` (HEAD `a4dc7228` when this draft was written).
**Origin:** owner decisions Q9 = A (2026-10-29) and O2 = A (2026-10-30), after the review of the dashboard design (`tmp/dashboard-design-review/impl-review.md`, finding M3). Inventory: `tmp/part-routes/inventory.md` (all counts and lines below come from it).

## Changes the player will see

None. The page calls none of these routes (known: no `hx-get`, `hx-post`, `fetch` or `htmx.ajax` in `assets/` or in a server template names them).

## The problem

Twelve routes render a part beside `GET /screen`: `/fragment/header`, `/fragment/failure-banner`, `/fragment/story-log`, `/fragment/visual-sidebar`, `/fragment/options-dock`, `/fragment/character-headshots`, `/fragment/llm-messages`, `/fragment/games`, `/fragment/worlds`, `/fragment/settings`, `/fragment/prompt-presets` and `/status/busy`. Only tests and the stub server call them: about 182 call sites, 12 stub mounts, and 94 scenario mentions in 15 spec files. The tests therefore check answers that the page never gets, and the design's rule "one renderer for each part name" has two entry routes for each part.

## Decisions

| # | Decision | Reason |
|---|---|---|
| D1 | Remove the 12 routes and their route handlers. The part renderers stay: `/screen` calls them (`screen/handlers/parts.rs`). In `layout/handlers/endpoints.rs`, `render_status_display`, `idle_answer`, `busy_answer` and `failure_answer` stay (`parts.rs:96`, `stub_server.rs:46`). | One route per part. |
| D2 | A tier-1 test reads a part with the existing `fetch_part(app, name)` (`tests/http/support/http_requests.rs:64-70`). It returns the inner HTML of the part's `<template>`, which scenario 45.9 proves equal to the old route body, so substring assertions keep working. A second helper returns the status and the raw body, for the five sites that assert a failed read (`400` or `500` from `/screen` carries no `<template>`). | The helper exists and is proven. |
| D3 | The three callers on a real server over `reqwest` (`tests/test_utils/wait.rs:15`, `server.rs:338`, `browser.rs:310`) get a string helper in `tests/test_utils/` that cuts one part out of a `/screen` body. | The `http` binary's helpers are private to it. |
| D4 | The stub server drops its 12 route copies. Its `/screen` already renders every part from the same builders (`screen_part_html`). The load counter (`count_and_fail_reads_middleware`, `is_load`, `PANEL_LOAD_PATHS`, `Handles::loads(path)`) counts by part name, not by path. | The stub mirrors the engine. |
| D5 | Each scenario whose When line names a removed route names `GET /screen?parts=<part>` instead. Its Then lines stay. `validate_docs.py` (`UNKNOWN_ROUTE`) fails on any spec line left with a removed route, so this is required. | The spec states what a client can do. |
| D6 | Delete scenario 45.9 and its test (`screen_load.rs:92-110`, with `PART_ROUTES`): it compares each part with its own route, which no longer exists. | Nothing left to compare. |
| D7 | Delete `/fragment/character-headshots` with its renderer `AppState::render_character_headshots` (`app_state.rs:154`), `CharacterHeadshotsTemplate` (`templates.rs:66`) and its one untagged test (`requires_migration/fragment.rs:182-194`). No screen part shows it, and its template calls `toggleVisualSidebar`, which no script defines. Owner decision O1 = A. | Dead code. |
| D8 | Remove `POST /action/confirm`. Its handler `action_confirm_handler` is a copy of `action_handler` (both call `dispatch_action`, `action/handlers/actions.rs:41-57`). The two preview forms (`templates.rs:115`, `:119`) post to `POST /action`. Delete the route, the handler, its words row (`answer_words.rs:156`) and the stub's copy (`stub_server.rs:909`, `stub_request_layers.rs:17`). Scenario `actions.md:382` and the Game-bound route lists in `screen.md` (`:7`, `:262`) name `/action`; the tests that post `/action/confirm` (`tests/http/actions.rs:1072`, `screen.rs:268`, `:380`, `answer_words.rs:194`, `:430`, `requires_migration/fragment.rs:313`, `tests/browser/stub/game_header.rs:259`) post `/action` or lose a now-duplicate row. Owner decision O2 = A: `POST /action` stays because the specs (about 101 mentions) and `post_action` (221 call lines) use it. | Two routes for one request. |
| D9 | Keep the debug routes (`/debug/state`, `/debug/is_busy`, `/debug/backend`). They render no part. | Out of scope. |

No pinned behaviour is lost (inventory §4, known): no test pins the `400` with no Game of a part route; `/status/busy` text and the failure disclosure come from the same function in the `status-display` part; a failed read through `/screen` is also `500` with `class="error-message"` and no `HX-Reswap`, so 38.2 and 38.18 keep their Then lines.

## Stages

Each stage is one commit, built in slice mode (`.agents/skills/simple-after-plan/SKILL.md`, Slice mode): the implementer cleans its comments and runs `python build.py --no-browser`; one reviewer checks spec, standards and docs from one bundle; fixes go back to the implementer; then the full gate `python build.py`, then the commit. A stage that changes a stub check runs its runner 10 times in a row.

| Stage | Scope | Role | Exit check |
|---|---|---|---|
| 1. Tier-1 callers | Every call in `tests/http/` (with `requires_migration/`) and `src/**/*_tests.rs` moves to `fetch_part` or the status helper (D2). The When lines of their scenarios change (D5). The routes stay, so each moved test is green before and after. About 150 call sites. | `implementer` | `rg -n '"/fragment/(header\|failure-banner\|story-log\|visual-sidebar\|options-dock\|llm-messages\|games\|worlds\|settings\|prompt-presets)"\|"/status/busy"' tests/http src` finds only `screen_load.rs` (45.9 and `PART_ROUTES`) and the `.route` calls in `builders/router.rs`. |
| 2. Real server and stub | D3 and D4. Every stub check that counts loads by path counts by part name. The stub's 12 route copies go. The When lines of the browser scenarios change (D5). | `strong-implementer` (the load counter touches many stub checks at once) | `python build.py browser` green; each changed stub runner 10 of 10. `rg` for the 12 paths finds nothing in `tests/browser`, `tests/test_utils`. |
| 3. Remove the routes | D1, D6, D7, D8. Delete the 12 `.route` calls and their handlers, the handler unit tests in `endpoints_tests.rs`, 45.9. Regenerate `http_routes.md` (`python scripts/extract_http_routes.py`). Reword the spec lines that still name a removed route: the endpoint lists and section intros, and property S.1 of `docs/specs/screen.md` (which the `rg` check does not find). Edit the two hand-written lines of `tests/AGENTS.md` (`:44`, `:120`) that give `GET /fragment/options-dock` as the seam recipe. | `implementer` | `rg` for the 12 paths and for `/action/confirm` finds them only in `old-docs/`, `docs/plans/` and `.scratch/`. The preview send stub check (game_header) passes 10 of 10. `validate-docs` and the full gate green. |

After stage 3, a final check of the whole plan: the reviewer compares the branch with this plan (every decision built, nothing beyond it), then the plan is archived to `old-docs/archived-plans/`.

## Stop conditions

Stop and go to the owner when:
- a test pins a behaviour that only a removed route has (inventory §4 found none);
- a scenario's Then line must change, not only its When line;
- a stage needs a new helper, counter or flag beyond D2–D4.

## Owner decisions

- **O1 = A** (2026-10-30): delete `/fragment/character-headshots` and its dead renderer and template (D7).
- **O2 = A** (2026-10-30): remove `/action/confirm`, not `POST /action` (D8).

## Gate log

| Stage | Commit | Gate | Notes |
|---|---|---|---|
| 1 | (this commit) | Full gate `logs/build_20261030_193918.log` (309 guardrail, 1522 integration, 2 skipped, 44 browser). About 179 tier-1 call sites read `/screen`; one status-and-body helper `fetch_answer`. Review `tmp/review/pr-stage1/review.md`. | Owner decisions in review: S1 = A (the 43 spec lines whose test moved name `GET /screen?parts=` in this stage), S2 = A (the Then lines of `dashboard.md` 39.1, 39.9 and 39.11 name the template of `status-display`). Open, older than this plan: scenarios 38.2 and 20.23 say "asks the client to swap nothing", and their tests check that `HX-Reswap` is absent (review L6). |
