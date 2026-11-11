# Integration-tier migration disposition

Asset for ticket [17 — Migrate existing integration tests to the right tier](../issues/17-integration-migration.md).

Storage tests are **spec-less per ticket 16**: the test code is the definition, no `docs/specs/` file. This is a classification only — no files are moved.

## Disposition table

| File | Tests today | Tier today | HTTP-observable? | Disposition | Reason |
|---|---|---|---|---|---|
| `tests/integration/adapters/driven/llm/llm_client.rs` | 4 | integration (driven-adapter seam) | No | `stay` | Exercises `call_chat_completions` against a hand-rolled TCP mock server. Not unit-tier (no trait fake; uses real reqwest) and not HTTP E2E (not through the axum router). It is a driven-adapter seam test for the LLM transport. |
| `tests/integration/bootstrap/run_branches.rs` | 3 | integration (bootstrap) | No | `stay` | Covers `bootstrap::run` startup branches against real seed data and SQLite. It is a wiring/infra smoke test, not spec-observable and not unit-fakeable without a bootstrap refactor. |
| `tests/integration/model/css.rs` | 4 | integration (HTTP-level) | Yes | `port-to-http-e2e` | Hits `/assets/styles.css` through `TestAppBuilder` and asserts on the response body. HTTP-observable; belongs in `tests/http/` (e.g. `tests/http/assets.rs`). If scenario-tagged, it will need a spec entry; otherwise it is an HTTP smoke test without a spec tag. |
| `tests/integration/model/settings.rs` | 7 | integration (HTTP-level) | Yes | `port-to-http-e2e` | Tests `/fragment/settings` and `POST /settings` through the real axum router. Belongs in `tests/http/settings.rs` (or similar) with a new `settings.md` spec. |
| `tests/integration/model/world.rs` | 3 | integration (mixed) | Partial | `port-to-http-e2e` | Two data-loading tests only assert fixture JSON → model fields; the same serde branches are already covered by `src/domain/model/map_tests.rs` and `src/domain/model/world_tests.rs`, so they should be deleted as redundant. The `test_visual_sidebar_with_real_world_data` test calls `/fragment/visual-sidebar` and should move to `tests/http/`. |
| `tests/integration/storage/preset_storage.rs` | 28 | integration (driven-adapter) | No | `stay` | CRUD, ordering, type filtering, upsert, unicode, and edge cases for `Storage` preset methods against real SQLite. This is the storage seam per `STRATEGY.md`; no spec per ticket 16. |
| `tests/integration/storage/snapshot_storage.rs` | 18 | integration (driven-adapter) | No | `stay` | Save/load/latest/by-id, bad JSON/date handling, game isolation, and message round-trips for snapshots. Storage seam, no spec. |
| `tests/integration/storage/llm_message_storage.rs` | 5 | integration (driven-adapter) | No | `stay` | Save/list, error preservation, global-cap pruning, and limit behaviour for `Storage` LLM-message methods. Storage seam, no spec. |
| `tests/integration/storage/message_storage.rs` | 14 | integration (driven-adapter) | No | `stay` | Soft-delete/restore/purge, swipe insert/update/shift/load, empty cases, and an in-memory variant. Storage seam, no spec. |
| `tests/integration/storage/world_storage.rs` | 19 | integration (driven-adapter) | No | `stay` | Create/list/get/delete, referential-integrity, idempotency, and both SQLite and in-memory variants. Storage seam, no spec. |
| `tests/integration/storage/prompt_presets.rs` | 11 | integration (HTTP-level) | Yes | `port-to-http-e2e` | Tests `/fragment/prompt-presets`, `/prompt-presets`, `/prompt-presets/{id}/activate`, `/prompt-presets/{id}/delete`, and `/prompt-presets/{id}` through the real axum router. Mis-located in `storage/`. Move to `tests/http/prompt_presets.rs` and add a `prompt_presets.md` spec. |
| `tests/llm/flow_llm_tests.rs` | 2 | LLM E2E | Yes (browser + real LLM) | `stay` | Real-LLM end-to-end flows, ignored by default. Requires `OPENROUTER_API_KEY`; separate tier outside the four-tier strategy. |
| `tests/llm/mod.rs` | 0 | LLM E2E | N/A | `stay` | Module root for `tests/llm/flow_llm_tests.rs`; no tests. Keep as long as the LLM E2E tier exists. |

## Scaffolding / module-root files

These files contain no `#[test]` attributes but organize the `tests/integration/` binary. Their fate is mechanical once the contents above move:

- `tests/integration/mod.rs` — can be deleted once all submodules are moved out.
- `tests/integration/model/mod.rs` — can be deleted after `css.rs`, `settings.rs`, and `world.rs` are moved/deleted.
- `tests/integration/storage/mod.rs` — should stay, because the storage seam tests remain in the driven-adapter tier.

## Spec implications

The `port-to-http-e2e` files currently lack matching specs:

- `settings.md` — for `/fragment/settings` and `POST /settings`.
- `prompt_presets.md` — for the prompt-preset endpoints.
- `assets.md` (optional) — for `/assets/styles.css` if the CSS tests are promoted to tagged scenarios.
- `visual_sidebar.md` or equivalent — for `/fragment/visual-sidebar` if it becomes a tagged scenario.

These gaps belong to the codebase-wide missing-spec audit (ticket 18), not to this disposition map.

## Test counts note

Counts above are from `grep -E '^\s*#\[(tokio::)?test\]'` on the current codebase. They may differ from the map's earlier estimate because the integration directory is a single Cargo binary with submodules; the same submodule can be reachable from multiple binaries, so Cargo's run-time count is not necessarily the file-level count. The disposition is unaffected.
