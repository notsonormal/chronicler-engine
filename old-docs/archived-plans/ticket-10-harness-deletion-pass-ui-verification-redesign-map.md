# Ticket 10 — Harness deletion pass (ui-verification-redesign map)

## Summary
Execute the deletion list ratified in ticket 04 (Q5) on the post-rollout tree, now that blockers 06–09 are all resolved. Verified findings: delete the orphan `/status/ready` endpoint (route + handler + re-export + two coverage-only tests), delete the two dead wait primitives from `tests/test_utils/wait.rs`, fold the duplicated server-ready probe out of `goto_with_connection_check`, and record that the `networkidle` convention write-up belongs to ticket 11 and the `invariants.rs` story-log twin no longer exists.

## Key Changes
- Delete `.route("/status/ready", ...)` at `src/adapters/driving/http/builders/router.rs:82`
- Delete `status_ready_handler` at `src/adapters/driving/http/layout/handlers/endpoints.rs:45` and its re-export in `.../layout/handlers/mod.rs:8`
- Delete `test_status_ready_handler` from `src/adapters/driving/http/layout/handlers/endpoints_tests.rs` (drop the import) and from `tests/http/requires_migration/fragment.rs`; lower `REQUIRES_MIGRATION_TEST_COUNT` 86→85 in `scripts/validate_feature_spec.py`
- Regenerate `docs/diataxis/reference/frontend/http_routes.md` — **mandatory**: the gate's `http-routes-check` step runs `extract_http_routes.py --check` and fails on a stale doc
- Delete `wait_for_condition_sync` (wait.rs:248) and private `extract_port_from_url` (wait.rs:114) — zero call sites across tests/
- Delete the `wait_for_server(port, 100)` call in `goto_with_connection_check` (browser.rs:27) and drop `wait_for_server` from the import; TestServer's own 30 s probe (`server.rs:518`) is the single boot readiness check (the fn itself stays — TestServer still uses it)

## Implementation

### Phase 1: Deletions

- [ ] #### Task 1.1: Claim ticket 10 and delete the `/status/ready` surface (3 SP)
  - [ ] ##### SubTask 1.1.1: Set `Status: claimed` in `.scratch/ui-verification-redesign/issues/10-harness-deletion-pass.md` (1 SP)
  - [ ] ##### SubTask 1.1.2: Delete route, handler, `mod.rs` re-export, both `test_status_ready_handler` tests (fix the `endpoints_tests.rs` import list), and set `REQUIRES_MIGRATION_TEST_COUNT = 85`; run `python scripts/extract_http_routes.py` to regenerate the routes doc (2 SP)
- [ ] #### Task 1.2: Prune `wait.rs` and fold the duplicated probe (3 SP)
  - [ ] ##### SubTask 1.2.1: Delete `wait_for_condition_sync` and `extract_port_from_url` from `tests/test_utils/wait.rs`; the `//!` module summary names only surviving helpers (no change expected) (2 SP)
  - [ ] ##### SubTask 1.2.2: Remove the `wait_for_server(port, 100)` block and the `wait_for_server` import in `tests/test_utils/browser.rs`; `flow_llm_tests.rs` needs no change (it constructs `TestServer::new` first, which probes) (1 SP)

### Phase 2: Gate and resolution

- [ ] #### Task 2.1: Full gate green (2 SP)
- [ ] #### Task 2.2: Record the Answer, resolve the ticket, append the map pointer, commit (1 SP)

## Test Plan
- `python build.py unit` — `endpoints_tests.rs` compiles and passes without the deleted handler/import
- `python build.py spec-coverage` — pin satisfied at 85; scenario/tag counts unchanged (141/141 — quarantine tests are untagged)
- `python build.py http-routes-check` — regenerated doc passes the freshness check
- `python build.py nextest browser` — tier-3 binary compiles and passes without the wait.rs pair and the probe (pure removal of a duplicate check; no behaviour change)
- `python build.py` — full gate (clippy catches any leftover unused import; the two bookkeeping steps above run in GATE_ORDER before the test suite)

## Per Task/Sub Task Validation Steps
- 1.1: after edits, `python build.py check` compiles; `python build.py unit` passes; `python build.py http-routes-check` green; `git diff docs/diataxis/reference/frontend/http_routes.md` shows exactly one removed row (a wider diff means the doc was stale pre-edit — review, not revert)
- 1.2: `python build.py clippy` clean (no unused-import warnings); `python build.py nextest browser` green
- 2.1: `python build.py` full gate green; tail the log per AGENTS.md
- 2.2: ticket 10's `## Answer` records each item with the verified evidence (incl. the two declined/deviated items and wait.rs/browser.rs line counts before/after), `Status:` set to `resolved`, one-line gist + link appended to the map's Decisions-so-far; commit follows the repo message convention

## Assumptions
- The `tests/STRATEGY.md` `networkidle` ban text is **not** written here: ticket 11's question explicitly owns that fold-in ("one convention, one place"); ticket 10's Answer passes over the zero-use grep evidence. This matches ticket 10 item 4's own "same edit as ticket 11's rule write-up; go together."
- `wait_for_status_ready` (the browser helper) stays — it polls `#status-display` DOM text and is unrelated to the `/status/ready` endpoint.
- Lowering `REQUIRES_MIGRATION_TEST_COUNT` 86→85 is the deliberate-decrement convention documented in `tests/STRATEGY.md` (the count may only go down; deleting a quarantined test lowers the constant deliberately).
- `wait.rs` live-surface check is complete: `wait_for_llm_idle` (llm flows), `wait_until_hidden` (tier2 ×3), `wait_for_element_persist` (tier2.rs:418), `wait_for_condition_async` (settle_gate ×2, invariants:371, http test_helpers:115), `wait_for_story_log` (SharedBrowser::open_page), status waits — all load-bearing; nothing beyond item 1's pair is deletable.
- Runs against the tree at commit `33ef5fc` (ticket 09 landed); ticket 11 may run in parallel — no file overlap (10 touches no STRATEGY.md).
- Not in scope (reviewed): STRATEGY.md text (ticket 11), ticket-12 acceptance work, ticket-13 sweep, any `wait.rs` primitive beyond the item-1 pair, any src change beyond the one orphaned route.
