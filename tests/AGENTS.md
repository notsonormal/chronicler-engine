# `tests/`

## Test Strategy

Read [`STRATEGY.md`](STRATEGY.md) before placing a test or writing a spec scenario: the tier of every `tests/` directory, the tier-1 **domain outcome** rule, the UI placement rule, and the SCENARIO-tag rules.

## Failure handling

When tests fail, you must:

1. **Show the actual test output** — quote the failure message verbatim.
2. **Read the test code** — understand what the test is actually checking before explaining why it failed.
3. **Verify your assumptions** — if you claim "this test skips when X is missing", verify X is actually missing and the skip logic exists.
4. **Never rationalize failures away** — a test failure is a real signal that requires investigation, not dismissal.
5. **Investigate pre-existing and flaky failures too** — a failure that looks unrelated is often related; even when it isn't, failing tests need fixing regardless.

If you're unsure why a test failed, say so and investigate — don't invent explanations.

## LLM Testing

`python build.py` runs the non-LLM suite only. LLM tests are `#[ignore]`d by default.

## Engine log tee

Every spawned test server tees engine stdout/stderr incrementally to `tmp/test_server_logs/{port}_{stream}.log` (`tests/test_utils/server.rs::drain_to_buffer_and_tee`). Files persist after the run, pass or fail — read them to diagnose engine-side behaviour without editing tests (one log per port; ports are recycled, so check the timestamp).

### Test Mirror Convention

Integration test structure mirrors `src/` paths **within each test binary**. The tier table in [`STRATEGY.md`](STRATEGY.md) picks the directory; inside it, file paths mirror `src/` subpaths.

Examples:
- `src/application/narration/narrator.rs` ↔ `src/application/narration/narrator_tests.rs` (unit test mirror)
- `src/application/games/catalogue.rs` ↔ `src/application/games/catalogue_tests.rs` (unit test mirror)
- `src/adapters/driving/http/action/handlers/actions.rs` ↔ `tests/http/actions.rs` (http test binary mirrors the http subset)
- `src/adapters/driven/storage/stories.rs` ↔ `tests/storage/story_storage.rs` (driven-adapter storage seam)

## Seam recipes

Recurring HTTP/spec-test seams — the exemplar file is the documentation; keep API details there.

- Assert what the narrator was sent → storage-backed recorder (`make_test_recorder_with_storage`, `src/test_support/fixtures.rs`) + `GameViewQuery::list_latest_llm_messages`
- Inject a storage failure → `Storage::with_failure` + `TestOverride::internal`, exemplar `tests/http/settings.rs`
- Wire an options agent → an `options`-type `AgentConfig` through `AgentRunner::from_configs` (`configured_runner` in test support; `configured_runner_without` leaves one type out), exemplar `src/application/agents/options/agent_tests.rs`
- Observe generated options in the UI → `GET /fragment/options-dock` (`src/adapters/driving/http/builders/router.rs`)

## STRUCTURE
<!-- AUTO-STRUCTURE-TESTS START -->
- **bootstrap/**
    - `mod.rs` — Bootstrap smoke test binary: startup branches in `bootstrap::run()`.
    - `run_branches.rs` — Smoke tests covering uncovered startup branches in `bootstrap::run()`.
- **browser/**
    - `autoplay.rs` — Browser Autoplay tests: the Autoplay control's full-stack hop, and the view-only Game while a run holds it. A run that must hold its Game narrates through a `SilentModel`. Tagged against `docs/specs/browser_autoplay.md`.
    - `dashboard.rs` — Browser dashboard-chrome tests: static command form, status display. Tagged against `docs/specs/browser_dashboard.md`.
    - `games.rs` — Browser games-panel tests: per-game posture auto-save wiring guard. Tagged against `docs/specs/browser_games.md`.
    - `mod.rs` — Browser test binary root (Playwright-driven): per-surface behaviour modules mirroring the `docs/specs/browser_<feature>.md` specs (`autoplay`, `dashboard`, `games`, `options`, `prompt_presets`, `worlds`) plus `stub/` (stub-browser tests against a fake engine, incl. `layout` — CSS and layout checks, declared exemption, no spec, test code is the definition).
    - `options.rs` — Browser options-dock tests: reload persistence and the Use-click wiring guard. Tagged against `docs/specs/browser_options.md`.
    - `prompt_presets.rs` — Browser prompt-presets tests: the duplicate → edit → save click chain wiring guard. Tagged against `docs/specs/browser_prompt_presets.md`.
    - `worlds.rs` — Browser worlds-panel tests: world posture auto-save wiring guard. Tagged against `docs/specs/browser_worlds.md`.
    - **stub/**
      - `announcements.rs` — Stub-browser tests for the live-region announcements: each change is announced exactly once. Tagged against `docs/specs/browser_dashboard.md`.
      - `autoplay.rs` — Stub-browser tests for the Autoplay control: the progress status and the action area while a run holds the Game. Tagged against `docs/specs/browser_autoplay.md`.
      - `dashboard.rs` — Stub-browser tests for dashboard chrome: the action-area state machine, the tab bar, the landmarks and the status display. Tagged against `docs/specs/browser_dashboard.md`.
      - `dialog.rs` — Stub-browser tests for the edit dialog: an htmx swap into its body opens it, Cancel and Escape close it and return focus to the opener, a save closes it only when the engine accepts it, and an answer from outside never replaces its form. Tagged against `docs/specs/browser_dashboard.md`.
      - `drafts.rs` — Stub-browser tests for the rules that keep a draft out of a region: a load skips a control whose request is in flight, and no region of any tab holds a form that waits for a Save. Tagged against `docs/specs/browser_live_regions.md`.
      - `failure_display.rs` — Stub-browser tests for the failure display: the banner, the in-place slots and the clamped disclosures. Tagged against `docs/specs/browser_dashboard.md`.
      - `form_failures.rs` — Stub-browser tests for refused form actions: the form keeps its place and reports the failure inline. Tagged against `docs/specs/browser_dashboard.md`.
      - `games.rs` — Stub-browser tests for the Games panel: the keyboard-focus landing after a saved-game delete. Tagged against `docs/specs/browser_games.md`.
      - `layout.rs` — Layout checks (declared exemption in the spec-coverage validator): no spec link, test code is the definition. CSS computed styles, layout measurements, text-wrap behavior — only a real browser can observe these. Twelve checks share one browser through the shared runner; each runs on a fresh page against its own stub server, with panic isolation and a per-check timing summary.
      - `live_regions.rs` — Stub-browser tests for the live regions: the ask loop, the loads it starts, the entry in edit that a load skips, and the state a load keeps. The dialog forms that survive a load are here too. Tagged against `docs/specs/browser_live_regions.md`.
      - `llm_messages.rs` — Stub-browser tests for the LLM Messages panel: the shipped client's keyboard path over a canned row, the record that opens in the dialog, and the load that leaves the open dialog alone. Tagged against `docs/specs/browser_llm_messages.md`.
      - `mod.rs` — Stub-browser tests: browser-only behaviour against a fake engine.
      - `options.rs` — Stub-browser tests for the options dock: the client-side edit action filling the command input. Tagged against `docs/specs/browser_options.md`.
      - `prompt_presets.rs` — Stub-browser tests for the Prompt Presets panel: the keyboard-focus landing after a preset card delete and when an editor opens, and the report of a refused editor. Tagged against `docs/specs/browser_prompt_presets.md`.
      - `settings.rs` — Stub-browser tests for the Settings panel: the client-side sub-tabs and the connection form in the dialog. Tagged against `docs/specs/browser_settings.md`.
      - `slash_menu.rs` — Stub-browser tests for the slash menu: the client-side command palette rendered from the shipped shell's `input` listener. Tagged against `docs/specs/browser_slash_menu.md`.
      - `story_log.rs` — Stub-browser tests for the story log's load: the morph that keeps selection, focus, scroll position and the surviving entries. Tagged against `docs/specs/browser_story_log.md`.
      - `story_log_edit.rs` — Stub-browser tests for the story log's edit flow: edit mode, the entry locks and the failure slots. Tagged against `docs/specs/browser_story_log.md`.
      - `support.rs` — Shared stub-browser helpers: the check runner, page probes, node reads and the on-demand region load used by more than one stub test module.
      - `swipes.rs` — Stub-browser test for switching swipes: the client's restore handling and the dock it leaves behind.
      - `worlds.rs` — Stub-browser tests for the Worlds panel: the keyboard-focus landing after a world row delete. Tagged against `docs/specs/browser_worlds.md`.
- **helpers/**
    - `fixtures.rs` — Shared fixtures for integration tests: builds storage instances with deterministic defaults so tests can focus on the behaviour under test.
    - `storage_ext.rs` — Test-only `Storage` extension trait for seeding deterministic test worlds.
- **http/**
    - `actions.rs` — HTTP E2E tests for the action endpoint (POST /action).
    - `changes.rs` — HTTP E2E tests for the change numbers (`GET /changes`): which numbers each save path moves, and which numbers a read, a failed save and a rolled-back transaction leave as they were. Tagged against `docs/specs/changes.md`.
    - `dashboard.rs` — HTTP E2E tests for dashboard chrome: the shell's icon sprite, the busy-status surfaces, and the debug state. Tagged against `docs/specs/dashboard.md`.
    - `failure_display.rs` — HTTP E2E tests for the failure display, tagged against `docs/specs/failure_display.md`.
    - `games_config.rs` — HTTP E2E tests for the per-game config endpoints (posture, presets, mode): storage failures surface as 500 error fragments instead of panics.
    - `games_create.rs` — HTTP E2E tests for game creation (POST /games).
    - `games_delete.rs` — HTTP E2E tests for game deletion (POST /games/:id/delete).
    - `games_fragment.rs` — HTTP E2E tests for the games panel fragment (`GET /fragment/games`) — the posture fragment's rendered selects and preset pickers.
    - `games_rename.rs` — HTTP E2E tests for game rename (POST /games/:id/rename).
    - `games_switch.rs` — HTTP E2E tests for game switching (POST /games/:id/switch).
    - `llm_messages.rs` — HTTP E2E tests for the LLM Messages panel: a failed attempt is recorded and its failure text renders, the list sends a header for each attempt, and the record of one attempt opens in the dialog.
    - `mod.rs` — HTTP test binary root: real-request integration tests for action handlers, fragment rendering, connections UI, debug endpoints, and server wiring.
    - `narrator_mode.rs` — HTTP E2E tests for narrator mode: world-to-game posture inheritance, mode switching, and steering availability.
    - `options.rs` — HTTP E2E tests for options-autogeneration: on-demand /options, the always-on Options run at the end of a Job, and response-shape parsing.
    - `prompt_presets.rs` — HTTP E2E tests for the prompt-presets endpoints.
    - `races.rs` — HTTP E2E tests for the write path while a Job runs: the races (actions.md 3.4, 3.5; options.md 24.21), the stop at each model call (dashboard.md 39.4, 39.5), the reset (reset.md 7.3), the Job of another Game (games.md 18.4), the edit, delete and swipe switch (story_log.md 8.7; swipe_switch.md 36.3), and the preset an event Narration uses (actions.md 6.4).
    - `reset.rs` — HTTP E2E tests for the reset endpoint (POST /reset).
    - `retrigger.rs` — HTTP E2E tests for the retrigger endpoint (POST /retrigger).
    - `settings.rs` — HTTP E2E tests for the settings endpoints: sub-tabs, role rows, and the text-check auto-save.
    - `settings_connection_form.rs` — HTTP E2E tests for the connection form: the context window and the reasoning effort the Add, Edit and Test actions take.
    - `story_log.rs` — HTTP E2E tests for the story-log fragment: delete flows through POST /history/delete, the fragment's shape, and the controls each entry renders.
    - `swipe_new.rs` — HTTP E2E tests for the retry endpoint (POST /swipe/new).
    - `swipe_switch.rs` — HTTP E2E tests for switching swipes (POST /message/:id/swipe/:index).
    - `visual_sidebar.rs` — HTTP E2E tests for the visual sidebar fragment (`GET /fragment/visual-sidebar`) — the portrait labels.
    - `worlds.rs` — HTTP E2E tests for the worlds endpoints: the update posture merge contract, the options-toggle checkbox grammar, the auto-save posture endpoint, the duplicate-identifier create refusal, and the World delete: its removal, its counted refusal and its storage failure.
    - **autoplay/**
      - `fragments.rs` — HTTP E2E tests for the view-only Game of Autoplay (autoplay.md 41.18-41.20, 41.22): the controls the story log, the dock and the Games panel render while a run holds the Game.
      - `mod.rs` — HTTP E2E tests for Autoplay (autoplay.md 41.*), shared helpers: `run` holds the run of Jobs, its ends, the progress status and the refused writes; `fragments` holds the view-only Game's rendering.
      - `run.rs` — HTTP E2E tests for the run of Autoplay (autoplay.md 41.1-41.17, 41.21): the Jobs of a run, its ends, the budget, the refused writes, and the progress status.
    - **requires_migration/**
      - `connections.rs` — HTTP integration tests for the connections UI: add OpenRouter/DeepSeek connections, switch the narrator, and switch the quantifier.
      - `core.rs` — HTTP integration test for reset-handler error handling.
      - `debug.rs` — HTTP integration tests for the debug endpoints: `/debug/state` returns the expected JSON shape and `/debug/is_busy` reflects the actual busy state.
      - `fragment.rs` — HTTP integration tests for fragment rendering.
      - `games_fragment_handlers.rs` — HTTP E2E tests for the games list fragment (GET /fragment/games).
      - `index_handler.rs` — HTTP integration test for the dashboard index handler.
      - `mod.rs` — Quarantined HTTP tests pending specs.
      - `server_impl_wiring.rs` — HTTP wiring tests for `server_impl.rs` (real request routing lives in the `http` test binary).
      - `text_check.rs` — HTTP integration tests for the action-check text-check dispatch: disabled vs. enabled, auto-check off, issues, and clean.
      - `worlds_fragment_handlers.rs` — HTTP adapter tests for worlds_fragment handlers
    - **support/**
      - `app_wiring.rs` — Test-app builders for the http test binary: scripted-model AppState + router bundles.
      - `http_assertions.rs` — Rendered-HTML assertions shared by the http test binary.
      - `http_fixtures.rs` — HTTP test fixtures: storage seeding, story reads and world-card builders for the endpoint tests.
      - `http_requests.rs` — HTTP request plumbing for the http test binary: POST builders, body readers, and the waits for a Job to end (`wait_idle`, `wait_settled`, `wait_for_jobs`).
      - `mod.rs` — Concept modules for the HTTP test binary's shared surface (requests, fixtures, assertions, app wiring).
- **infrastructure/**
    - `architecture.rs` — Architecture guardrail tests using arch-lint — fail the build on any violation defined in `arch-lint.toml`; run with `cargo nextest run --test architecture`.
    - **guardrails/**
      - `enums.rs` — Enum variant doc guardrail: every enum variant must carry `///` doc, OR the enum must be marked `/// [TRIVIAL_ENUM]` with all variants bare.
      - `free_fn.rs` — Free fn location guardrail: top-level free fns must live in a folder named `mappers`, `utils`, `builders`, `test_support`, `bootstrap`, or `handlers`.
      - `free_fn_tests.rs` — Tests for `free_fn.rs` guardrail.
      - `inherent_impl.rs` — Inherent impl locality guardrail: every inherent impl must live in the type's defining file or a folder named after the type.
      - `inherent_impl_tests.rs` — Tests for the inherent impl locality guardrail.
      - `layers.rs` — Layer-boundary guardrail tests: server vs. application vs. storage separation, handler return-type enforcement, urlencoded form safety, and the HTTP adapter's storage leak.
      - `live_regions.rs` — Live-region guardrails: each element of the dashboard shell that lists data kinds in `data-live` loads through the one load path, and no script or handler writes a region by another path.
      - `live_regions_tests.rs` — Tests for the live-region guardrail.
      - `location.rs` — Location guardrail tests: ensures `#[test]` / `#[cfg(test)]` units live in the correct directory (e.g., unit tests stay in `src/`, integration tests stay in `tests/`).
      - `location_tests.rs` — Tests for `location.rs` guardrail.
      - `mod.rs` — Infrastructure test binary root: shared guardrail harness (rule definitions, `Violation` type, file discovery, `check_src_files` / `check_tests_files` runners).
      - `nesting.rs` — Nesting depth guardrail — fails the build when function-body control-flow nesting exceeds `MAX_NESTING_DEPTH`.
      - `registry_tests.rs` — Guardrail registry tests: every rule is reachable from `mod.rs` and has a detection test, and every design-rule walker table is registered with a detection case.
      - `structure.rs` — Structure guardrail tests: doc-anchor standards, mod.rs purity, no-std-thread, file length, and the new test module-header rule.
      - `structure_tests.rs` — Tests for `structure.rs` guardrail.
      - `style.rs` — Style guardrail tests: import ordering, single-letter variable usage, separator comments, long comment runs, and per-file `cfg(test)` tracking.
      - `style_tests.rs` — Tests for `style.rs` guardrail.
      - `test_hygiene.rs` — Test hygiene guardrails: no `#[ignore]`d test outside `tests/llm/`, and no bare sleep in a test without a comment that gives its reason.
      - `test_hygiene_tests.rs` — Tests for the `test_hygiene.rs` guardrails.
      - `unit_test_allowlist.rs` — The legacy unit-test list and its allowlist guardrail: a unit test lives in a seam contract, a `domain/model/` file, or a listed legacy file.
      - `unit_test_allowlist_tests.rs` — Detection tests for the legacy and seam-contract allowlist guardrail: a unit test lives in a listed seam contract, a `domain/model/` file, or a listed legacy file.
      - `walker_rows.rs` — The four design-rule walker row tables: one table per kind, and each row's reason on the comments above it.
      - `walker_tests.rs` — Detection tests for the four design-rule walkers: one table-driven test per kind, each case running its walker over synthetic files.
      - `walkers.rs` — Table-driven design-rule walkers: the four kinds, their row types, and the checks that run one row table.
- **llm/**
    - `flow_llm_tests.rs` — LLM-driven flow tests: exercises real LLM provider flows end-to-end (ignored by default; run with `python build.py --llm-only`).
    - `mod.rs` — LLM test binary: real LLM provider flows (ignored by default; run with `python build.py --llm-only`).
- **storage/**
    - `changes.rs` — The storage seam of the change counters: what a committed transaction, a rolled-back transaction, a storage call, a failed call and a read do to the numbers.
    - `llm_message_storage.rs` — Integration tests for LLM message persistence: save/list/get by id, error-message preservation, global-cap pruning, and pagination across a real SQLite-backed `Storage`.
    - `mod.rs` — Driven-adapter storage seam tests: repositories exercised against a real SQLite-backed `Storage`.
    - `preset_storage.rs` — Tests for Storage preset methods: list_presets, get_preset, save_preset, delete_preset
    - `story_storage.rs` — The storage seam of the new records: the story reads, the dock record, the Game's failure line and the record writes of a transaction.
    - `transaction_storage.rs` — The storage transaction seam: one unit of record writes that lands whole, and the newest Game of a World.
    - `world_storage.rs` — Integration tests for world persistence: create/list/re-seed/delete `WorldCard`s, the referential-integrity rule that blocks world deletion when games still reference it, and the characters a deleted world drops.
- **test_utils/**
    - `browser.rs` — Browser test helpers: Playwright bootstrap (`TestServer`, `LaunchOptions`), page builders, and the tab/panel open helpers.
    - `convergence.rs` — The convergence check: once the page is quiet, each visible live region equals a fresh GET of its own route, apart from the story-log entry in edit. The stub and the full-stack browser tests run it at the end of each check.
    - `html.rs` — HTML slicing helpers for panel markup — locate a region by a stable anchor string.
    - `htmx_settle.rs` — htmx settle harness primitive: an `htmx:afterSettle` counter installed at page load, and a target-scoped wait for an interaction's own swap.
    - `live_region_table.rs` — The live regions of the dashboard shell, as the stub-browser tests list them: the selector, the fragment path, the kinds of change that load it, and its section. The guardrails check the table against `assets/index.html`.
    - `mod.rs` — Shared test utilities re-exported across all test binaries: `browser`, `convergence`, `server`, `settings_guard`, `wait`, plus the `TEST_WORLD` / `TEST_PERSONA` constants.
    - `server.rs` — Test server helpers: spawn the real engine binary on a free port, track lifecycle via `SERVER_MANAGED`, and expose `TestServer` / `wait_for_server` / `get_config_port`.
    - `settings_guard.rs` — `SettingsTestGuard` — serializes tests that mutate global settings state via a process-wide `Mutex`.
    - `stub_server.rs` — Stub-browser server: the real dashboard shell plus canned fragments, with no engine behind it.
    - `wait.rs` — Polling helpers: `wait_for_llm_idle`, `wait_for_status_ready`, and `wait_for_element_children` — retry-based waits used by browser and HTTP tests.
<!-- AUTO-STRUCTURE-TESTS END -->