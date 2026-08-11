# Chronicler Engine Test Inventory

Subdir/file purpose reference for `tests/` and unit tests.

**File lists drift on every PR — run `ls` for current membership.** This file captures *purpose*, not a manifest.

# Unit tests — `src/**/*_tests.rs`

Sibling files alongside source, declared via `#[cfg(test)] mod <name>_tests;` in the parent `mod.rs`. Convention description lives in `SKILL.md`; this section is examples only.

Examples:

```
src/application/llm_recorder.rs              → src/application/llm_recorder_tests.rs
src/domain/model/world.rs                   → src/domain/model/world_tests.rs
src/adapters/driven/storage/backend/games.rs → src/adapters/driven/storage/backend/games_tests.rs
src/adapters/driving/http/fragments/actions.rs → src/adapters/driving/http/fragments/actions_tests.rs
src/bootstrap/llm_factory.rs                → src/bootstrap/llm_factory_tests.rs
src/error.rs                                → src/error_tests.rs
src/cli.rs                                  → src/cli_tests.rs
```

Some `mod.rs` files contain inline `#[cfg(test)] mod` smoke checks too.

# Integration tests — `tests/integration/`

|Subdir|Purpose|
|---|---|
|`adapters/`|Driven-adapter integration. `adapters/driven/llm/` holds LLM client tests (`llm_client.rs`).|
|`application/`|Application-layer orchestration: `game_service.rs`, `lifecycle.rs`, `application_service.rs`, `wiring.rs`, plus `action_pipeline/` subdir (`actions.rs`, `pipeline.rs`, `retry.rs`).|
|`flow/`|Action sequence / retry behavior: `sequence.rs`, `retry_main.rs`, `retry_event.rs`, `arrival_persistence.rs`.|
|`model/`|Domain model: `world.rs`, `settings.rs`, `state_patch.rs`, `css.rs`.|
|`storage/`|Persistence: `snapshot_storage.rs`, `preset_storage.rs`, `llm_message_storage.rs`, `prompt_presets.rs`, `message_storage.rs`, `world_storage.rs`.|

# HTTP/component tests — `tests/http/`

HTTP endpoint, WebSocket, and HTMX fragment tests.

|File/Dir|Purpose|
|---|---|
|`endpoints/`|Per-endpoint checks (e.g. `text_check.rs`)|
|`actions.rs`|HTTP action endpoints|
|`connections.rs`|WebSocket connection tests|
|`debug.rs`|Debug endpoint tests|
|`fragment.rs`|HTMX fragment endpoints|
|`games_fragment_handlers.rs`|Games fragment handlers|
|`worlds_fragment_handlers.rs`|Worlds fragment handlers|
|`server_impl_wiring.rs`|Server implementation wiring|
|`test_helpers.rs`|HTTP test helpers|
|`mod.rs`|Module declarations|

# Browser tests — `tests/browser/`

Playwright-driven.

|File|Purpose|
|---|---|
|`editing.rs`|Text editing, input handling|
|`interaction.rs`|UI interaction flows|
|`structure.rs`|Page structure validation|
|`trigger.rs`|Trigger system via browser|

# LLM tests — `tests/llm/`

|File|Purpose|
|---|---|
|`flow_llm_tests.rs`|End-to-end LLM flow tests (uses real OpenRouter API)|

`#[ignore]`d by default. Gated by `OPENROUTER_API_KEY`. See `SKILL.md` LLM test policy.

# Infrastructure tests — `tests/infrastructure/`

|File/Dir|Purpose|
|---|---|
|`architecture.rs`|Architecture lint guardrails|
|`guardrails/`|Style and structure guardrails: `layers.rs`, `location.rs`, `structure.rs`, `style.rs`|
|`invariant_contract.rs`|Cross-module invariant contract checks|

# Other test files (root of `tests/`)

|File|Purpose|
|---|---|
|`poison_recovery.rs`|Mutex poison recovery tests|
|`test_config.json`|Test configuration|

# Test runner configuration

|File|Purpose|
|---|---|
|`.config/nextest.toml`|cargo-nextest configuration: threads, retries, timeouts, LLM profile|

# Helpers

|Location|Purpose|
|---|---|
|`tests/test_utils/`|Shared utilities. See `WAIT_HELPERS.md` for the wait API. Also: `mod.rs` (exports + `TEST_WORLD`, `CONFIG_PATH` constants), `browser.rs`, `server.rs`, `settings_guard.rs`.|
|`tests/helpers/`|Fixture builders and pipeline helpers: `fixtures.rs`, `pipeline_helpers.rs`.|
