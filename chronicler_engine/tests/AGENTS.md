# `chronicler_engine/tests/`

### TEST MIRROR CONVENTION

Integration test structure mirrors `src/` paths **within each test binary**. The test **binary** is chosen by fixture weight (integration/http/browser/llm/infrastructure); inside each binary, file paths mirror `src/` subpaths.

Examples:
- `src/application/action_pipeline/pipeline.rs` ↔ `tests/integration/application/action_pipeline/pipeline.rs`
- `src/application/game_service.rs` ↔ `tests/integration/application/game_service.rs`
- `src/adapters/driving/http/fragments/actions.rs` ↔ `tests/http/fragments/actions.rs` (http test binary naturally mirrors the http subset)
- `src/adapters/driven/llm/transport.rs` ↔ `tests/integration/adapters/driven/llm/transport.rs` (or `tests/integration/adapters/driven/llm_client.rs` for legacy reasons)

## STRUCTURE
<!-- AUTO-STRUCTURE-TESTS START -->
- `poison_recovery.rs` — Tests for poison-recovery behaviour: confirms that a poisoned `RwLock` inside the settings layer and `CancellationToken` machinery does not crash subsequent operations.
- **browser/**
    - `editing.rs` — Browser tests for message editing: edit and delete buttons, edit-mode activation on click, cancel-restores-original, and save-persistence through the HTTP API.
    - `interaction.rs` — Browser tests for form submission interactions (e.g., submitting a command via the action form and observing the resulting UI state).
    - `mod.rs` — Browser test binary root (Playwright-driven): editing, form interaction, DOM structure, and trigger-driven narration against a real running server.
    - `structure.rs` — Browser tests for DOM structure on page load: header shows the game title, connection-status indicator renders, and the action area exposes the expected input affordances.
    - `trigger.rs` — Browser tests for trigger-driven narration: `look` command emits narration entries, subsequent quantifier passes detect NPCs in the current room, and NPCs without triggers produce no narration.
- **helpers/**
    - `application_ext.rs` — Test-only `DefaultApplicationService` extension trait for driving pipeline scenarios.
    - `fixtures.rs` — Shared fixtures for integration tests: builds storage, world, character, and game-state instances with deterministic defaults so tests can focus on the behaviour under test.
    - `sqlite_test_app_builder.rs` — Integration-only SQLite-backed application builder for integration tests.
    - `storage_ext.rs` — Test-only `Storage` extension trait for seeding deterministic test worlds.
- **http/**
    - `actions.rs` — HTTP integration tests for the action and action-confirm handlers: graceful degradation when state load or message insert fails, and snapshot-save failure paths.
    - `connections.rs` — HTTP integration tests for the connections UI: add OpenRouter/DeepSeek connections, switch the narrator, and switch the quantifier.
    - `debug.rs` — HTTP integration tests for the debug endpoints: `/debug/state` returns the expected JSON shape and `/debug/is_generating` reflects the actual generation status.
    - `fragment.rs` — HTTP integration tests for fragment rendering: basic fragments return HTML, visual sidebar renders the room image, action area fragment renders, and the action handler accepts commands.
    - `games_fragment_handlers.rs` — Integration tests for games_fragment handlers
    - `mod.rs` — HTTP test binary root: real-request integration tests for action handlers, fragment rendering, connections UI, debug endpoints, server wiring, and the per-endpoint text-check suite.
    - `server_impl_wiring.rs` — HTTP wiring tests for `server_impl.rs` (real request routing lives in `tests/http/fragment.rs`).
    - `test_helpers.rs` — Shared test helpers for HTTP tests
    - `worlds_fragment_handlers.rs` — Unit tests for worlds_fragment handlers
    - **endpoints/**
      - `mod.rs` — HTTP integration tests for endpoint-specific behaviours (currently: text-check).
      - `text_check.rs` — HTTP integration tests for the text-check endpoints: action-check dispatch (disabled vs. enabled), empty-command handling, and confirm-flow returning the full action area with check results.
- **infrastructure/**
    - `architecture.rs` — Architecture guardrail tests using arch-lint — fail the build on any violation defined in `arch-lint.toml`; run with `cargo nextest run --test architecture`.
    - `invariant_contract.rs` — Runtime invariant contract tests — fast regression guards.
    - **guardrails/**
      - `enums.rs` — Enum variant doc guardrail: every enum variant must carry `///` doc, OR the enum must be marked `/// [TRIVIAL_ENUM]` with all variants bare.
      - `layers.rs` — Layer-boundary guardrail tests: server vs. application vs. storage separation, handler return-type enforcement, and tests-vs-messages/swipes separation.
      - `location.rs` — Location guardrail tests: ensures `#[test]` / `#[cfg(test)]` units live in the correct directory (e.g., unit tests stay in `src/`, integration tests stay in `tests/`).
      - `mod.rs` — Infrastructure test binary root: shared guardrail harness (rule definitions, `Violation` type, file discovery, `check_src_files` / `check_tests_files` runners).
      - `nesting.rs` — Nesting depth guardrail — reports function-body control-flow nesting depth violations (probe only; does not gate the build).
      - `structure.rs` — Structure guardrail tests: doc-anchor standards, mod.rs purity, no-std-thread, file length, and the new test module-header rule.
      - `style.rs` — Style guardrail tests: import ordering, single-letter variable usage, separator comments, long comment runs, and per-file `cfg(test)` tracking.
- **integration/**
    - `mod.rs` — Integration test binary root: wires shared helpers (`test_utils`, `fixtures`, `storage_ext`, `application_ext`) and re-exports factory helpers (`failing_service`, `working_service`, `SettingsTestGuard`) used by the application / storage / flow / model / adapter sub-suites.
    - **adapters/**
      - **driven/**
        - **llm/**
          - `llm_client.rs` — Integration tests for LLM client HTTP communication
    - **application/**
      - `application_service.rs` — Integration tests for DefaultApplicationService
      - `game_service.rs` — GameService integration tests
      - `lifecycle.rs` — Integration tests for game lifecycle operations — cross-cutting over `src/application/` rather than a mirror of a single src file; kept here for simplicity until the suite grows enough to split per-module.
      - **action_pipeline/**
        - `actions.rs` — Integration tests for the action pipeline: verifies that user actions are persisted to state, that narrations from the LLM are stored, and that error paths (room not found, LLM failure) are surfaced gracefully.
        - `pipeline.rs` — Integration tests for the action pipeline: delayed LLM completion, quantifier detection of movement and NPCs (with trigger firing), and graceful handling of empty LLM responses.
        - `retry.rs` — Integration tests for action retry behaviour: re-running the pipeline against the last user input, no-op on empty history, recovery after a previous LLM failure, and the missing-snapshot error path.
    - **bootstrap/**
      - `mod.rs` — Bootstrap startup-branch smoke tests for `bootstrap::run()`.
      - `run_branches.rs` — Smoke tests covering uncovered startup branches in `bootstrap::run()`.
    - **flow/**
      - `arrival_persistence.rs` — Integration flow tests for arrival narration persistence — confirms the arrival narration survives a state reload, exercising the `ArrivalTaskContext` end-to-end against SQLite storage.
      - `retry_event.rs` — Integration flow tests for the retry-event handler: no extra swipe on narration retry, quantifier-result preservation on continuation, and trigger continuations re-running the quantifier to detect newly-relevant NPCs.
      - `retry_main.rs` — Integration flow tests for the retry-main handler: new quantifier result on re-narration, re-running the quantifier on different text, double-retry swipe increment, and the no-extra-swipe guarantee when input is preserved.
      - `sequence.rs` — Integration flow tests for action sequencing: execute→retry→execute, execute→delete→execute, async action ordering, and three-action sequence under realistic state churn.
    - **model/**
      - `css.rs` — HTTP-level test for the static CSS asset endpoint; confirms the served stylesheet parses as valid CSS and contains the expected selectors.
      - `mod.rs` — Integration tests for model types exercised through the HTTP layer (world loading, settings, CSS asset, state-patch merge).
      - `settings.rs` — HTTP-level tests for the settings fragment endpoint; verifies the settings panel renders and that settings state changes persist correctly.
      - `state_patch.rs` — Unit tests for StatePatch merge semantics
      - `world.rs` — Integration tests for the world loading path: confirms the `WorldCard` and `MapDef` loaded from on-disk JSON expose expected room metadata (image paths, room ids).
    - **storage/**
      - `llm_message_storage.rs` — Integration tests for LLM message persistence: save/list, error-message preservation, global-cap pruning, and pagination across a real SQLite-backed `Storage`.
      - `message_storage.rs` — Integration tests for `Message` persistence: soft-delete, restore, purge, and swipe insert/load round-trips against a real SQLite-backed `Storage`.
      - `mod.rs` — Integration tests for storage repositories exercised against a real SQLite-backed `Storage`: messages, snapshots, worlds, LLM message log, prompt presets, and the prompt-presets HTTP fragment.
      - `preset_storage.rs` — Tests for Storage preset methods: list_presets, get_preset, save_preset, delete_preset
      - `prompt_presets.rs` — HTTP-level tests for the prompt-presets fragment: add/activate/delete system and quantifier presets and confirm the rendered panel reflects each state change.
      - `snapshot_storage.rs` — Integration tests for game-state snapshot persistence: save/load, missing-snapshot errors, and message/swipe round-tripping against a real SQLite-backed `Storage`.
      - `world_storage.rs` — Integration tests for world persistence: create/list/delete `WorldCard`s and the referential-integrity rule that blocks world deletion when games still reference it.
- **llm/**
    - `flow_llm_tests.rs` — LLM-driven flow tests: exercises real LLM provider flows end-to-end (ignored by default; run with `python build.py --llm-only`).
    - `mod.rs` — LLM test binary: real LLM provider flows (ignored by default; run with `python build.py --llm-only`).
- **test_utils/**
    - `browser.rs` — Browser test helpers: Playwright bootstrap (`TestServer`, `LaunchOptions`), page builders, and DOM helpers (`wait_for_element_children`, `wait_for_status_ready`).
    - `mod.rs` — Shared test utilities re-exported across all test binaries: `browser`, `server`, `settings_guard`, `wait`, plus the `TEST_WORLD` / `TEST_PERSONA` constants.
    - `server.rs` — Test server helpers: spawn the real engine binary on a free port, track lifecycle via `SERVER_MANAGED`, and expose `TestServer` / `wait_for_server` / `get_config_port`.
    - `settings_guard.rs` — `SettingsTestGuard` — serializes tests that mutate global settings state via a process-wide `Mutex`.
    - `wait.rs` — Polling helpers: `wait_for_llm_idle`, `wait_for_status_ready`, and `wait_for_element_children` — retry-based waits used by browser and HTTP tests.
<!-- AUTO-STRUCTURE-TESTS END -->