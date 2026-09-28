# Settings Review Fixes

## Summary

Close the twenty findings from the two-axis review of the uncommitted settings-source-of-truth work. Two changes alter behaviour: the Harper checker starts honouring the live ignored-word list, and the settings handlers start escaping their error text. The rest removes dead code, closes coverage gaps, and corrects two docs. No commit is made by the agent — the permission config denies `git commit`, so you commit.

Three answers from the plan review change this version: error rendering reuses the existing `render_error` instead of a new helper, the `production_graph` flag is replaced by a helper in the http tier, and the Harper cache recovers from mutex poisoning inline.

## Key Changes

1. **Harper honours the live word list** — `HarperTextChecker` caches the merged dictionary keyed on the word list it receives per call, instead of snapshotting at construction (Q6 = A). Closes the staleness class and the unused `_ignored_words` parameter.
2. **`load_settings` becomes infallible** — returns `AppSettings`, so `run.rs` loses its unreachable second swallow (Q7 = A).
3. **Dead code goes** — `LlmBackendType::from_env` (Q4 = A), `SettingsService::save_settings` (Q10 = A), `http/utils/locks.rs` (Q10 = A), the orphan `tests/helpers/sqlite_test_app_builder.rs` (Q5 = A), and the leftover scaffolding in `src/utils/settings_tests.rs` (I19).
4. **Coverage** — Pattern 2 backend pairs for all fifteen settings storage tests (Q8 = B), an `app_with_production_graph` helper plus a live-switch HTTP test on `GET /debug/backend` (Q9 = A), the shipped-file seed assertion, and the `load_settings` failure-fallback test (I7).
5. **Duplication removed** — one `write_settings` behind a two-variant `SettingsWrite` (Q12 = A), `LlmCallRecorder::provider_label()` (Q13 = A), and the mixed tests split out of `templates/settings_tests.rs` (Q15 = A).
6. **Error text unified and escaped** — every settings handler renders `Html(render_error(&e.to_string()))`, reusing `http/utils/error.rs`; `panel_or_error` is deleted. This drops the label half of Q11 = A and fixes unescaped interpolation in the current spans.
7. **Docs** — `startup.md` contradiction (I1); stale line references dropped and `settings_path: None` deleted in `integration_test_standards.md` (Q14 = A, I3).

## What already exists — reuse, do not reimplement

| Thing | Where | Used by |
|---|---|---|
| `render_error` (escapes HTML) | `http/utils/error.rs` | Task 1.6 |
| `build_app_graph`, `AppState::from_wired`, `build_router` | `wiring.rs:148`, `app_state.rs:47`, `router.rs:21` | Task 2.2 |
| `sqlite_storage()` | `test_support/fixtures.rs:608` | the SQLite half of every Pattern 2 pair |
| `Storage::with_failure` | storage `TestOverride` | the `load_settings` fallback test |
| `#[cfg(test)] mod <name>_tests;` | `http/utils/mod.rs` | registering `handler_helpers_tests.rs` |
| `app_with_narrator*` | `tests/http/support/app_wiring.rs` | the new helper's siblings |
| `GET /debug/backend` | `router.rs:41` | the live-switch observation point |
| Pattern 2 | `unit_test_standards.md:25` | the storage test shape |
| `Storage::save_settings` | storage `settings.rs:84` | kept — test support and ~20 tests use it |

## Implementation

### Phase 1: Production code

- [ ] #### Task 1.1: Harper honours the per-call ignored-word list (3 SP)
  - [ ] ##### SubTask 1.1.1: Rework `HarperTextChecker` (2 SP)
    - `src/adapters/driven/text_check/harper_text_checker.rs` — drop the `ignored_words` field and the `new(ignored_words)` parameter; take `new()`.
    - Replace the `OnceLock<Arc<MergedDictionary>>` with `Mutex<Option<(Vec<String>, Arc<MergedDictionary>)>>`: one entry, holding the list it was built from; rebuild only when the list differs.
    - Recover from a poisoned lock with `.lock().unwrap_or_else(|e| e.into_inner())`, matching the idiom already used in `SettingsTestGuard` and `SETTINGS_DB_LOCK`.
    - Use the `ignored_words` argument in `check` instead of `_ignored_words`. An empty list means the curated dictionary only.
  - [ ] ##### SubTask 1.1.2: Update the construction sites (1 SP)
    - `src/bootstrap/wiring.rs` — both `build_app_graph` and `build_app_graph_for_tests` call `HarperTextChecker::new()`.
    - Update any test that constructs the checker.
- [ ] #### Task 1.2: Make `load_settings` infallible (3 SP)
  - [ ] ##### SubTask 1.2.1: Change the signature (1 SP)
    - `src/utils/settings.rs` — return `AppSettings`; keep the warning and the fallback; reword the doc comment so it states the swallow policy without referring to a return type that no longer exists.
  - [ ] ##### SubTask 1.2.2: Update the call sites (1 SP)
    - `src/bootstrap/run.rs` — `let settings = load_settings(&storage);`, no `?` and no `unwrap_or_else`.
    - `src/utils/settings_tests.rs` — drop `.expect("should load")`.
  - [ ] ##### SubTask 1.2.3: Cover the fallback (1 SP)
    - Add `test_load_settings_falls_back_to_defaults_on_read_failure` using `Storage::with_failure` keyed `get_settings`; assert the result equals `AppSettings::default()`.
- [ ] #### Task 1.3: Remove dead code (3 SP)
  - [ ] ##### SubTask 1.3.1: Delete `from_env` (1 SP)
    - `src/domain/model/llm_backend.rs` — delete `from_env`; delete its tests in `llm_backend_tests.rs`; delete the `LLM_BACKEND` sentence from the `## 2026-09-27` CHANGELOG section.
  - [ ] ##### SubTask 1.3.2: Delete the dead settings-service seam (1 SP)
    - `src/application/settings_service.rs` — delete `save_settings`; delete `test_save_settings_roundtrip` from `settings_service_tests.rs`.
    - `tests/infrastructure/guardrails/layers.rs:421` — replace the fixture's `save_settings` call with a live method (`update_settings`).
  - [ ] ##### SubTask 1.3.3: Delete the unused lock module and the orphan test helper (1 SP)
    - Delete `src/adapters/driving/http/utils/locks.rs` and `locks_tests.rs`; remove both re-exports (`http/mod.rs`, `http/utils/mod.rs`).
    - Delete `tests/helpers/sqlite_test_app_builder.rs`; regenerate `tests/AGENTS.md` with `scripts/generate_tests_structure_index.py`.
- [ ] #### Task 1.4: Add `LlmCallRecorder::provider_label()` (3 SP)
  - [ ] ##### SubTask 1.4.1: Add the method and use it (2 SP)
    - `src/application/llm_recorder.rs` — `provider_label(&self) -> String` returns `"{name} {model}"`, or `"<unresolved: {e}>"` on error.
    - Use it in `src/application/pipeline/action_pipeline/core.rs` (`with_storage` — this also removes its second `provider()` call) and in the options and quantifier orchestration log sites.
  - [ ] ##### SubTask 1.4.2: Leave the debug shape alone (1 SP)
    - Keep `backend_info() -> (String, String)`; confirm `debug_backend_handler` and its template are unchanged.
- [ ] #### Task 1.5: Merge the two settings writes (3 SP)
  - [ ] In `src/adapters/driven/storage/settings.rs`, add a private two-variant `SettingsWrite` enum (`Replace`, `OnlyIfAbsent`, marked `[TRIVIAL_ENUM]`) and one `write_settings(backend, settings, mode)`.
  - The SQLite arm picks `INSERT OR REPLACE` or `INSERT OR IGNORE`; the in-memory arm writes and sets `settings_seeded` for `Replace`, and writes only when the flag is clear for `OnlyIfAbsent`.
  - `save_settings` and `update_settings` pass `Replace`; `seed_settings` passes `OnlyIfAbsent`.
- [ ] #### Task 1.6: Unify and escape the settings handler errors (2 SP)
  - [ ] ##### SubTask 1.6.1: Swap the call sites (1 SP)
    - In `src/adapters/driving/http/settings/handlers/settings.rs`, every error path renders `Html(render_error(&e.to_string()))` using `http/utils/error.rs`. Delete `panel_or_error`.
  - [ ] ##### SubTask 1.6.2: Update the tests that anchor on the old text (1 SP)
    - Audit `handlers/settings_tests.rs`, `tests/http/settings.rs`, and `tests/http/requires_migration/connections.rs` for assertions on `Save failed` / `Add failed` / `Edit failed` / `class='error'`; anchor them on `error-message` or on the error's own text.

### Phase 2: Tests

- [ ] #### Task 2.1: Pattern 2 pairs for the settings storage tests (5 SP)
  - [ ] ##### SubTask 2.1.1: Convert the fifteen tests (4 SP)
    - `src/adapters/driven/storage/settings_tests.rs` — each test becomes a pair: one against `Storage::new_in_memory()` and one named `<name>_sqlite` against `sqlite_storage()`, character-for-character identical but for the construction.
  - [ ] ##### SubTask 2.1.2: Confirm the new branch is covered (1 SP)
    - Verify the in-memory `settings_seeded` arm is exercised by the seeding pair, and that the per-test `DbPool::new(":memory:")` construction disappears.
- [ ] #### Task 2.2: Cover the two centrepiece claims (3 SP)
  - [ ] ##### SubTask 2.2.1: Add the production-graph helper (1 SP)
    - `tests/http/support/app_wiring.rs` — `app_with_production_graph(settings) -> (Router, AppState, Arc<Storage>)`: in-memory storage, save settings, assert every connection is `LlmBackendType::Mock`, call `build_app_graph`, then `build_router(AppState::from_wired(wired))`.
  - [ ] ##### SubTask 2.2.2: Add the live-switch HTTP test (1 SP)
    - Seed two Mock connections (`mock-model-a`, `mock-model-b`) with the first as narrator and quantifier, assert `GET /debug/backend` reports `mock-model-a`, POST `/connections/{b}/set-narrator`, assert it now reports `mock-model-b`.
  - [ ] ##### SubTask 2.2.3: Assert the shipped file seeds (1 SP)
    - Seed the repo's real `data/settings.json` through `bootstrap::load::seed_settings` into SQLite storage; assert the stored narrator is `deepseek-v4-flash` and both role ids resolve.
- [ ] #### Task 2.3: Untangle `templates/settings_tests.rs` (3 SP)
  - [ ] ##### SubTask 2.3.1: Delete the duplicated parse tests (1 SP)
    - Remove `test_deepseek_returns_deepseek`, `test_mock_returns_mock`, `test_openrouter_returns_openrouter`, `test_ollama_returns_ollama`, `test_unknown_provider_is_error`; their coverage lives in `llm_backend_tests.rs`.
  - [ ] ##### SubTask 2.3.2: Move the helper tests (2 SP)
    - Create `src/adapters/driving/http/utils/handler_helpers_tests.rs` holding the two `opt_string` tests, registered as `#[cfg(test)] mod handler_helpers_tests;` in `utils/mod.rs`.
- [ ] #### Task 2.4: Delete the leftover test scaffolding (1 SP)
  - Remove `with_isolated_settings`, `SETTINGS_DB_LOCK`, and the unused `PathBuf` parameter from `src/utils/settings_tests.rs`; call `load_settings(&storage)` directly.

### Phase 3: Docs

- [ ] #### Task 3.1: Fix the `startup.md` contradiction (1 SP)
  - Reword the trailing seeding sentence so the skip-without-halting rule names worlds, personas, and presets, leaving the settings bullet as the one fail-loud exception.
- [ ] #### Task 3.2: Fix `integration_test_standards.md` (1 SP)
  - Drop the stale `server.rs:54-66` and `:76-83` line ranges, keeping the function names.
  - Delete `settings_path: None` from the `Args` sample.
- [ ] #### Task 3.3: Update the CHANGELOG (1 SP)
  - Under the existing `## 2026-09-27` section, add the Harper ignored-words fix to `### Fixed` and the unified error rendering to `### Changed`. Remove the `LLM_BACKEND` sentence (Task 1.3.1).

### Phase 4: Gate

- [ ] #### Task 4.1: Run the full gate and the validators (3 SP)
  - `python build.py`, then `python scripts/validate_docs.py`, `python scripts/validate_data.py`, and `python scripts/validate_feature_spec.py`.
  - Confirm the spec counters stay at 141 declared / 141 covered.

## Failure modes

| Codepath | Failure | Handling |
|---|---|---|
| Harper cache | mutex poisoned | recover with `into_inner()` |
| Harper cache | empty word list | curated dictionary only, as today |
| Harper cache | concurrent checks | serialized briefly on the mutex to clone the `Arc`; acceptable on a per-action path |
| `load_settings` | row read fails | warn, return defaults — unchanged policy, now with no caller able to propagate it |
| `app_with_production_graph` | non-Mock connection | guard panics at build, loudly |
| `app_with_production_graph` | `build_app_graph` returns Err | `.expect(...)`, so the test fails with the resolver's message |
| `SettingsWrite` merge | serialization fails | returns `Err` before the INSERT, as today |
| `SettingsWrite` merge | `OnlyIfAbsent` with the flag set | no write, as today |
| `provider_label` | resolver error | `"<unresolved: {e}>"`; cannot fail, so logging never breaks construction |
| `render_error` in settings handlers | message contains markup | escaped, closing the unescaped-interpolation defect |
| deleting `from_env` | a script sets `LLM_BACKEND` | already inert — nothing read it |

## Test Plan

| Test | File | What it proves |
|---|---|---|
| `test_load_settings_falls_back_to_defaults_on_read_failure` | `src/utils/settings_tests.rs` | a failed row read still yields defaults |
| settings storage pairs (15 → 30) | `src/adapters/driven/storage/settings_tests.rs` | both backends behave alike, including `settings_seeded` |
| live-switch HTTP test | `tests/http/settings.rs` | a narrator switch changes the resolved provider with no restart |
| shipped-file seed assertion | `src/bootstrap/load_tests.rs` | the real `data/settings.json` seeds `deepseek-v4-flash` |
| `opt_string` tests | `src/adapters/driving/http/utils/handler_helpers_tests.rs` | moved coverage, unchanged assertions |

Removed tests: the five `LlmBackendType` parse duplicates, `from_env`'s tests, and `test_save_settings_roundtrip` in `settings_service_tests.rs`.

## Per Task/Sub Task Validation Steps

- After every Phase 1 sub task: `python build.py check` (about 7 s).
- Task 1.1: `cargo nextest run -E 'test(harper) | test(text_check)'`.
- Task 1.2: `cargo nextest run -E 'test(utils::settings_tests) | test(bootstrap::load_tests)'`.
- Task 1.3: `cargo nextest run -E 'test(guardrails) | test(settings_service)'` plus `python build.py architecture`.
- Task 1.4: `cargo nextest run -E 'test(llm_recorder_tests) | test(action_pipeline)'`.
- Task 1.5: `cargo nextest run -E 'test(storage::settings)'`.
- Task 1.6: `cargo nextest run -E 'test(settings)'`.
- Task 2.1: `cargo nextest run -E 'test(storage::settings)'` — 30 tests green.
- Task 2.2: `cargo nextest run -E 'test(settings)'`.
- Task 2.3: `cargo nextest run -E 'test(llm_backend_tests) | test(handler_helpers_tests) | test(settings_templates)'`.
- Task 3.x: `python scripts/validate_docs.py`.
- Task 4.1: `python build.py` — fmt clean, clippy 0 warnings, architecture 1/1, guardrails green, integration and browser green.

## NOT in scope

- The `OPENROUTER_API_KEY` race in `test_connection_resolve_api_key` — recorded as a known test-isolation hazard, not fixed.
- `seed_persona` and `seed_character` insert duplication — only the settings write is merged, so `settings.rs` becomes the one storage file with a write-mode enum. Justified by ~35 duplicated lines there against a small insert elsewhere.
- The spawned-tier harness and `inject_mock_connections` — unchanged.
- Any new production affordance: no reload action, no reset-to-file-defaults.
- The uncommitted settings redesign itself (t1–t10) — this pass only fixes findings on top of it.
- Committing — the permission config denies `git commit`; you commit.
- An ADR, and anything in `CONTEXT.md`.
- `docs/plans/settings-source-of-truth.md` content beyond Assumption 6.
- Lint or prose work outside the two named doc files and the CHANGELOG.

## Assumptions

1. `HarperTextChecker::new` takes no arguments after Task 1.1; the port's per-call list is the only source of ignored words.
2. `Storage::save_settings` stays — test support and about twenty tests use it.
3. The settings storage test file converts wholesale to Pattern 2, including the twelve pre-existing single-backend tests.
4. No commit is made by the agent; the permission config keeps `git commit` denied.
5. `docs/plans/settings-source-of-truth.md` stays as the historical record of the design decisions; its open items are settled by this plan.
6. Docs changes stay inside `docs/diataxis/` plus `docs/CHANGELOG.md`; no ADR is written (Q11 = C from the earlier round).
7. The settings error markup changes from `<span class='error'>` to `<div class="error-message">`; the browser and HTTP tests that assert the old markup are updated in Task 1.6.2.
