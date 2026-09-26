# `tests/`

## Test Strategy

See [`STRATEGY.md`](STRATEGY.md) for the normative placement rule (which of the three UI tiers a test belongs to) and the overlap/SCENARIO-tag conventions.

## Failure handling

When tests fail, you MUST:

1. **Show the actual test output** — quote the failure message verbatim.
2. **Read the test code** — understand what the test is actually checking before explaining why it failed.
3. **Verify your assumptions** — if you claim "this test skips when X is missing", verify X is actually missing and the skip logic exists.
4. **Never rationalize failures away** — a test failure is a real signal that requires investigation, not dismissal.
5. **Investigate pre-existing and flaky failures too** — a failure that looks unrelated is often related; even when it isn't, failing tests need fixing regardless.

If you're unsure why a test failed, say so and investigate — don't invent explanations.

## LLM Testing

`python build.py` runs the fast suite only. LLM tests are `#[ignore]`d by default.

## Engine log tee

Every spawned test server tees engine stdout/stderr incrementally to `tmp/test_server_logs/{port}_{stream}.log` (`tests/test_utils/server.rs::drain_to_buffer_and_tee`). Files persist after the run, pass or fail — read them to diagnose engine-side behaviour without editing tests (one log per port; ports are recycled, so check the timestamp).

### Test Mirror Convention

Integration test structure mirrors `src/` paths **within each test binary**. The test **binary** is chosen by fixture weight (architecture/http/browser/llm/storage/infrastructure); inside each binary, file paths mirror `src/` subpaths.

Examples:
- `src/application/pipeline/action_pipeline/core.rs` ↔ `src/application/pipeline/action_pipeline/core_tests.rs` (unit test mirror)
- `src/application/games/catalogue.rs` ↔ `src/application/games/catalogue_tests.rs` (unit test mirror)
- `src/adapters/driving/http/action/handlers/actions.rs` ↔ `tests/http/actions.rs` (http test binary mirrors the http subset)
- `src/adapters/driven/storage/db.rs` ↔ `tests/storage/message_storage.rs` (driven-adapter storage seam)

## Seam recipes

Recurring HTTP/spec-test seams — the exemplar file is the documentation; keep API details there.

- Assert what the narrator was sent → storage-backed recorder (`make_test_recorder_with_storage`, `src/test_support/fixtures.rs`) + `Storage::list_latest_llm_messages`
- Inject a storage failure → `Storage::with_failure` + `TestOverride::internal`, exemplar `tests/http/settings.rs`
- Wire an options agent → `OptionsAgent::with_provider` registered in `AgentRegistry`, exemplar `src/application/pipeline/action_pipeline/options_tests.rs`
- Observe generated options in the UI → `GET /fragment/options-dock` (`src/adapters/driving/http/builders/router.rs`)

## STRUCTURE
<!-- AUTO-STRUCTURE-TESTS START -->
- **bootstrap/**
    - `mod.rs` — Bootstrap smoke test binary: startup branches in `bootstrap::run()`.
    - `run_branches.rs` — Smoke tests covering uncovered startup branches in `bootstrap::run()`.
- **browser/**
    - `dashboard.rs` — Browser dashboard-chrome tests: static command form, status display. Tagged against `docs/specs/browser_dashboard.md`.
    - `games.rs` — Browser games-panel tests: per-game posture auto-save wiring guard. Tagged against `docs/specs/browser_games.md`.
    - `mod.rs` — Browser test binary root (Playwright-driven): per-surface behaviour modules mirroring the `docs/specs/browser_<feature>.md` specs (`dashboard`, `games`, `options`, `prompt_presets`, `worlds`) plus `stub/` (stub-browser tests against a fake engine, incl. `invariants` — CSS/layout rendering invariants, declared exemption, no spec, test code is the definition).
    - `options.rs` — Browser options-dock tests: reload persistence and the Use-click wiring guard. Tagged against `docs/specs/browser_options.md`.
    - `prompt_presets.rs` — Browser prompt-presets tests: the duplicate → edit → save click chain wiring guard. Tagged against `docs/specs/browser_prompt_presets.md`.
    - `worlds.rs` — Browser worlds-panel tests: world posture auto-save wiring guard. Tagged against `docs/specs/browser_worlds.md`.
    - **stub/**
      - `dashboard.rs` — Stub-browser tests for dashboard chrome: the error toast's response-body handling. Tagged against `docs/specs/browser_dashboard.md`.
      - `invariants.rs` — Rendering invariants (declared exemption in the spec-coverage validator): no spec link, test code is the definition. CSS computed styles, layout measurements, text-wrap behavior — only a real browser can observe these. Nine checks share one server+browser (no server-state mutation); each runs on a fresh page via `run_subtest` with panic isolation and a per-check timing summary.
      - `mod.rs` — Stub-browser tests: browser-only behaviour against a fake engine.
      - `options.rs` — Stub-browser tests for the options dock: the client-side edit action filling the command input. Tagged against `docs/specs/browser_options.md`.
      - `slash_menu.rs` — Stub-browser tests for the slash menu: the client-side command palette rendered from the shipped shell's `input` listener. Tagged against `docs/specs/browser_slash_menu.md`.
      - `story_log.rs` — Stub-browser tests for the story log: the client-side edit-mode flow over a canned entry. Tagged against `docs/specs/browser_story_log.md`.
- **helpers/**
    - `application_ext.rs` — Test-only `AppState` extension trait for driving pipeline scenarios.
    - `fixtures.rs` — Shared fixtures for integration tests: builds storage instances with deterministic defaults so tests can focus on the behaviour under test.
    - `sqlite_test_app_builder.rs` — Integration-only SQLite-backed application builder for integration tests.
    - `storage_ext.rs` — Test-only `Storage` extension trait for seeding deterministic test worlds.
- **http/**
    - `actions.rs` — HTTP E2E tests for the action endpoint (POST /action).
    - `games_config.rs` — HTTP E2E tests for the per-game config endpoints (posture, presets, mode): storage failures surface as 500 error spans instead of panics.
    - `games_create.rs` — HTTP E2E tests for game creation (POST /games).
    - `games_delete.rs` — HTTP E2E tests for game deletion (POST /games/:id/delete).
    - `games_fragment.rs` — HTTP E2E tests for the games panel fragment (`GET /fragment/games`) — the posture fragment's rendered selects and preset pickers.
    - `games_switch.rs` — HTTP E2E tests for game switching (POST /games/:id/switch).
    - `mod.rs` — HTTP test binary root: real-request integration tests for action handlers, fragment rendering, connections UI, debug endpoints, server wiring, and the per-endpoint text-check suite.
    - `narrator_mode.rs` — HTTP E2E tests for narrator mode: world-to-game posture inheritance, mode switching, and steering availability.
    - `options.rs` — HTTP E2E tests for options-autogeneration: on-demand /options, the always-on turn-end hook, and response-shape parsing.
    - `prompt_presets.rs` — HTTP E2E tests for the prompt-presets endpoints.
    - `reset.rs` — HTTP E2E tests for the reset endpoint (POST /reset).
    - `retrigger.rs` — HTTP E2E tests for the retrigger endpoint (POST /retrigger).
    - `settings.rs` — HTTP E2E tests for the settings endpoints: panel rendering and POST /settings.
    - `story_log.rs` — HTTP E2E tests for the story-log delete endpoint (POST /history/delete).
    - `swipe_new.rs` — HTTP E2E tests for the retry endpoint (POST /swipe/new).
    - `worlds.rs` — HTTP E2E tests for the worlds update endpoint: the posture merge contract, the options-toggle checkbox grammar, and the auto-save posture endpoint.
    - **requires_migration/**
      - `connections.rs` — HTTP integration tests for the connections UI: add OpenRouter/DeepSeek connections, switch the narrator, and switch the quantifier.
      - `core.rs` — HTTP integration test for reset-handler error handling.
      - `debug.rs` — HTTP integration tests for the debug endpoints: `/debug/state` returns the expected JSON shape and `/debug/is_generating` reflects the actual generation status.
      - `fragment.rs` — HTTP integration tests for fragment rendering.
      - `games_fragment_handlers.rs` — HTTP E2E tests for the games list fragment (GET /fragment/games).
      - `index_handler.rs` — HTTP integration test for the dashboard index handler.
      - `mod.rs` — Quarantined HTTP tests pending specs.
      - `server_impl_wiring.rs` — HTTP wiring tests for `server_impl.rs` (real request routing lives in the `http` test binary).
      - `text_check.rs` — HTTP integration tests for the text-check endpoints: action-check dispatch (disabled vs. enabled), empty-command handling, and confirm-flow returning the full action area with check results.
      - `worlds_fragment_handlers.rs` — HTTP adapter tests for worlds_fragment handlers
    - **support/**
      - `app_wiring.rs` — Test-app builders for the http test binary: narrator-wired AppState + router bundles.
      - `http_assertions.rs` — Rendered-HTML assertions shared by the http test binary.
      - `http_fixtures.rs` — HTTP test fixtures: storage seeding and world-card builders for the endpoint tests.
      - `http_requests.rs` — HTTP request plumbing for the http test binary: POST builders, body readers, generation-idle polling.
      - `mod.rs` — Concept modules for the HTTP test binary's shared surface (requests, fixtures, assertions, app wiring).
- **infrastructure/**
    - `architecture.rs` — Architecture guardrail tests using arch-lint — fail the build on any violation defined in `arch-lint.toml`; run with `cargo nextest run --test architecture`.
    - **guardrails/**
      - `enums.rs` — Enum variant doc guardrail: every enum variant must carry `///` doc, OR the enum must be marked `/// [TRIVIAL_ENUM]` with all variants bare.
      - `free_fn.rs` — Free fn location guardrail: top-level free fns must live in a folder named `mappers`, `utils`, `builders`, `test_support`, `bootstrap`, or `handlers`.
      - `free_fn_tests.rs` — Tests for `free_fn.rs` guardrail.
      - `inherent_impl.rs` — Inherent impl locality guardrail: every inherent impl must live in the type's defining file or a folder named after the type.
      - `inherent_impl_tests.rs` — Tests for the inherent impl locality guardrail.
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
    - `browser.rs` — Browser test helpers: Playwright bootstrap (`TestServer`, `LaunchOptions`), page builders, and the tab/panel open helpers.
    - `html.rs` — HTML slicing helpers for panel markup without element ids — locate a region by a stable anchor string.
    - `htmx_settle.rs` — htmx settle harness primitive: an `htmx:afterSettle` counter installed at page load, and a target-scoped wait for an interaction's own swap.
    - `mod.rs` — Shared test utilities re-exported across all test binaries: `browser`, `server`, `settings_guard`, `wait`, plus the `TEST_WORLD` / `TEST_PERSONA` constants.
    - `server.rs` — Test server helpers: spawn the real engine binary on a free port, track lifecycle via `SERVER_MANAGED`, and expose `TestServer` / `wait_for_server` / `get_config_port`.
    - `settings_guard.rs` — `SettingsTestGuard` — serializes tests that mutate global settings state via a process-wide `Mutex`.
    - `stub_server.rs` — Stub-browser server: the real dashboard shell plus canned fragments, with no engine behind it.
    - `wait.rs` — Polling helpers: `wait_for_llm_idle`, `wait_for_status_ready`, and `wait_for_element_children` — retry-based waits used by browser and HTTP tests.
<!-- AUTO-STRUCTURE-TESTS END -->