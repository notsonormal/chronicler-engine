# Chronicler Engine Test Inventory

Purpose reference for `tests/` binaries and unit-test layout.

Directory and file membership is auto-generated in `tests/AGENTS.md`
(`AUTO-STRUCTURE-TESTS`) and drifts on every PR — run `ls` for current
membership. This file captures *purpose and conventions*, not manifests.

# Unit tests — `src/**/*_tests.rs`

Sibling files alongside source, declared via `#[cfg(test)] mod <name>_tests;` in the parent `mod.rs`. Convention description lives in `SKILL.md`; this section is examples only.

```
src/application/games/catalogue.rs     → src/application/games/catalogue_tests.rs
src/domain/model/state/game_state.rs   → src/domain/model/state/game_state_tests.rs
src/adapters/driven/storage/games.rs   → src/adapters/driven/storage/games_tests.rs
src/application/llm_recorder.rs        → src/application/llm_recorder_tests.rs
src/error.rs                           → src/error_tests.rs
```

Some `mod.rs` files contain inline `#[cfg(test)] mod` smoke checks too.

# Test binaries — `tests/`

The test **binary** is chosen by fixture weight; inside each binary, file
paths mirror `src/` subpaths (see `tests/AGENTS.md` for the mirror
convention). Tier-placement rules live in `tests/STRATEGY.md`.

|Binary|Purpose|
|---|---|
|`tests/bootstrap/`|Smoke tests for `bootstrap::run()` startup branches|
|`tests/storage/`|Driven-adapter tier: repositories against a real SQLite-backed `Storage`|
|`tests/http/`|HTTP E2E: spec scenarios through the real router; `requires_migration/` is the quarantine pinned by `REQUIRES_MIGRATION_TEST_COUNT`|
|`tests/browser/`|Playwright: DOM, CSS, and JS-interaction scenarios per `docs/specs/browser_<feature>.md`|
|`tests/llm/`|Real LLM provider flows; `#[ignore]`d by default, gated by `OPENROUTER_API_KEY`|
|`tests/infrastructure/`|Architecture lint (`arch-lint`) + guardrail binaries|

# Test runner configuration

|File|Purpose|
|---|---|
|`.config/nextest.toml`|cargo-nextest configuration: threads, retries, timeouts, LLM profile|

# Helpers

- `tests/test_utils/` — shared utilities (`server`, `browser`, `wait`, `settings_guard`); wait API catalog in `WAIT_HELPERS.md`.
- `tests/helpers/` — fixture builders and pipeline-driving extensions.
