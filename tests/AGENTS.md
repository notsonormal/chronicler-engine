# `tests/`

### TEST STRATEGY

See [`STRATEGY.md`](STRATEGY.md) for the normative tier-placement rules (unit / HTTP E2E / browser / driven-adapter) and the overlap/SCENARIO-tag conventions.

### TEST MIRROR CONVENTION

Integration test structure mirrors `src/` paths **within each test binary**. The test **binary** is chosen by fixture weight (integration/http/browser/llm/infrastructure); inside each binary, file paths mirror `src/` subpaths.

Examples:
- `src/application/action_pipeline/pipeline.rs` ↔ `src/application/action_pipeline/pipeline_tests.rs` (unit test mirror)
- `src/application/game_service.rs` ↔ `src/application/game_service_tests.rs` (unit test mirror)
- `src/adapters/driving/http/action/handlers/actions.rs` ↔ `tests/http/actions.rs` (http test binary mirrors the http subset)
- `src/adapters/driven/storage/db.rs` ↔ `tests/storage/message_storage.rs` (driven-adapter storage seam)

## STRUCTURE
<!-- AUTO-STRUCTURE-TESTS START -->
- **bootstrap/**
    - `mod.rs` — Bootstrap smoke test binary: startup branches in `bootstrap::run()`.
    - `run_branches.rs` — Smoke tests covering uncovered startup branches in `bootstrap::run()`.
- **browser/**
    - `behaviour.rs` — Browser behaviour tests: click→DOM change, htmx swap persistence, polling-pause, status wiring. Tagged against `docs/specs/browser.md`.
    - `invariants.rs` — Rendering invariants (named exemption in STRATEGY.md): no spec link, test code is the definition. CSS computed styles, layout measurements, text-wrap behavior — only a real browser can observe these.
    - `mod.rs` — Browser test binary root (Playwright-driven): `behaviour` (client-side JS interaction, tagged against `docs/specs/browser.md`) + `invariants` (CSS/layout rendering invariants, named exemption — no spec, test code is the definition).
- **helpers/**
    - `application_ext.rs` — Test-only `AppState` extension trait for driving pipeline scenarios.
    - `fixtures.rs` — Shared fixtures for integration tests: builds storage, world, character, and game-state instances with deterministic defaults so tests can focus on the behaviour under test.
    - `sqlite_test_app_builder.rs` — Integration-only SQLite-backed application builder for integration tests.
    - `storage_ext.rs` — Test-only `Storage` extension trait for seeding deterministic test worlds.
- **http/**
    - `actions.rs` — HTTP E2E tests for the action endpoint (POST /action).
    - `games_create.rs` — HTTP E2E tests for game creation (POST /games).
    - `games_delete.rs` — HTTP E2E tests for game deletion (POST /games/:id/delete).
    - `games_switch.rs` — HTTP E2E tests for game switching (POST /games/:id/switch).
    - `mod.rs` — HTTP test binary root: real-request integration tests for action handlers, fragment rendering, connections UI, debug endpoints, server wiring, and the per-endpoint text-check suite.
    - `prompt_presets.rs` — HTTP E2E tests for the prompt-presets endpoints.
    - `reset.rs` — HTTP E2E tests for the reset endpoint (POST /reset).
    - `retrigger.rs` — HTTP E2E tests for the retrigger endpoint (POST /retrigger).
    - `settings.rs` — HTTP E2E tests for the settings endpoints: panel rendering and POST /settings.
    - `story_log.rs` — HTTP E2E tests for the story-log delete endpoint (POST /history/delete).
    - `swipe_new.rs` — HTTP E2E tests for the retry endpoint (POST /swipe/new).
    - `test_helpers.rs` — Shared test helpers for HTTP tests
    - **requires_migration/**
      - `connections.rs` — HTTP integration tests for the connections UI: add OpenRouter/DeepSeek connections, switch the narrator, and switch the quantifier.
      - `core.rs` — HTTP integration test for reset-handler error handling.
      - `debug.rs` — HTTP integration tests for the debug endpoints: `/debug/state` returns the expected JSON shape and `/debug/is_generating` reflects the actual generation status.
      - `fragment.rs` — HTTP integration tests for fragment rendering.
      - `games_fragment_handlers.rs` — HTTP E2E tests for the games list fragment (GET /fragment/games).
      - `index_handler.rs` — HTTP integration test for the dashboard index handler.
      - `mod.rs` — Quarantined HTTP tests pending specs.
      - `server_impl_wiring.rs` — HTTP wiring tests for `server_impl.rs` (real request routing lives in `tests/http/fragment.rs`).
      - `text_check.rs` — HTTP integration tests for the text-check endpoints: action-check dispatch (disabled vs. enabled), empty-command handling, and confirm-flow returning the full action area with check results.
      - `worlds_fragment_handlers.rs` — HTTP adapter tests for worlds_fragment handlers
- **infrastructure/**
    - `architecture.rs` — Architecture guardrail tests using arch-lint — fail the build on any violation defined in `arch-lint.toml`; run with `cargo nextest run --test architecture`.
    - `invariant_contract.rs` — Runtime invariant contract tests — fast regression guards.
    - **guardrails/**
      - `enums.rs` — Enum variant doc guardrail: every enum variant must carry `///` doc, OR the enum must be marked `/// [TRIVIAL_ENUM]` with all variants bare.
      - `free_fn.rs` — Free fn location guardrail: top-level free fns must live in a folder named `mappers`, `utils`, `builders`, `test_support`, `bootstrap`, or `handlers`.
      - `free_fn_tests.rs` — Tests for `free_fn.rs` guardrail.
      - `layers.rs` — Layer-boundary guardrail tests: server vs. application vs. storage separation, handler return-type enforcement, and tests-vs-messages/swipes separation.
      - `location.rs` — Location guardrail tests: ensures `#[test]` / `#[cfg(test)]` units live in the correct directory (e.g., unit tests stay in `src/`, integration tests stay in `tests/`).
      - `location_tests.rs` — Tests for `location.rs` guardrail.
      - `mod.rs` — Infrastructure test binary root: shared guardrail harness (rule definitions, `Violation` type, file discovery, `check_src_files` / `check_tests_files` runners).
      - `nesting.rs` — Nesting depth guardrail — reports function-body control-flow nesting depth violations (probe only; does not gate the build).
      - `structure.rs` — Structure guardrail tests: doc-anchor standards, mod.rs purity, no-std-thread, file length, and the new test module-header rule.
      - `structure_tests.rs` — Tests for `structure.rs` guardrail.
      - `style.rs` — Style guardrail tests: import ordering, single-letter variable usage, separator comments, long comment runs, and per-file `cfg(test)` tracking.
- **llm/**
    - `flow_llm_tests.rs` — LLM-driven flow tests: exercises real LLM provider flows end-to-end (ignored by default; run with `python build.py --llm-only`).
    - `mod.rs` — LLM test binary: real LLM provider flows (ignored by default; run with `python build.py --llm-only`).
- **storage/**
    - `llm_message_storage.rs` — Integration tests for LLM message persistence: save/list, error-message preservation, global-cap pruning, and pagination across a real SQLite-backed `Storage`.
    - `message_storage.rs` — Integration tests for `Message` persistence: soft-delete, restore, purge, and swipe insert/load round-trips against a real SQLite-backed `Storage`.
    - `mod.rs` — Driven-adapter storage seam tests: repositories exercised against a real SQLite-backed `Storage`.
    - `preset_storage.rs` — Tests for Storage preset methods: list_presets, get_preset, save_preset, delete_preset
    - `snapshot_storage.rs` — Integration tests for game-state snapshot persistence: save/load, missing-snapshot errors, and message/swipe round-tripping against a real SQLite-backed `Storage`.
    - `world_storage.rs` — Integration tests for world persistence: create/list/delete `WorldCard`s and the referential-integrity rule that blocks world deletion when games still reference it.
- **test_utils/**
    - `browser.rs` — Browser test helpers: Playwright bootstrap (`TestServer`, `LaunchOptions`), page builders, and DOM helpers (`wait_for_element_children`, `wait_for_status_ready`).
    - `mod.rs` — Shared test utilities re-exported across all test binaries: `browser`, `server`, `settings_guard`, `wait`, plus the `TEST_WORLD` / `TEST_PERSONA` constants.
    - `server.rs` — Test server helpers: spawn the real engine binary on a free port, track lifecycle via `SERVER_MANAGED`, and expose `TestServer` / `wait_for_server` / `get_config_port`.
    - `settings_guard.rs` — `SettingsTestGuard` — serializes tests that mutate global settings state via a process-wide `Mutex`.
    - `wait.rs` — Polling helpers: `wait_for_llm_idle`, `wait_for_status_ready`, and `wait_for_element_children` — retry-based waits used by browser and HTTP tests.
<!-- AUTO-STRUCTURE-TESTS END -->