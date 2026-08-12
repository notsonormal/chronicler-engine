# Collapse `PresetStore` and the separate `preset_storage` handle

> **Status:** Chartered.
> **Scope:** 5 SP mechanical collapse. No production behavior change.

## Summary

Two artifacts are leftover from `ADR-027` (removed in `e0f22e0`):

1. `PresetStore` — a 40-line newtype wrapping `Arc<Storage>` that delegates four
   preset methods straight back to `Storage` (`src/adapters/driven/storage/preset_store.rs`).
2. A **second `Arc<Storage>` instance** (`preset_storage`, `game_id = 1`) carried
   through `WiredApp`, `run.rs`, and test builders, distinct from the active-game
   `storage` (`game_id = active_game_id`).

Both exist for the same historical reason: a hexagonal boundary between "preset
storage" and "game storage" that ADR-027 mandated and that nobody re-enforced
after the ADR was deleted.

They enforce no real boundary:

- `prompt_presets`, `worlds`, `personas`, `characters` are **global tables** —
  no `game_id` column. `get_preset` / `list_presets` filter on `preset_type`
  only. `list_worlds` / `require_world` / `require_persona` / `list_characters`
  take no game_id either.
- Only `games`, `game_state_snapshots`, `messages`, `message_swipes` are
  game-scoped, keyed by `game_id`.
- In **production** both instances share one `DbPool` (one `.db` file); the
  `game_id` field only selects which game-scoped rows `current_game_id()`
  resolves. For all global-table reads (presets, worlds, personas, NPCs), the
  two instances are functionally identical.
- There is **no** separate storage object for worlds, personas, characters, or
  settings. Only presets got this treatment — a one-off.

`Storage` already has a `set_game_id` setter, and `resolve_game_id` works off
the `DbPool` directly (game_id-agnostic). So one `Storage` instance can serve
both bootstrap (global-table seeding + world/persona lookup) and runtime
(game-scoped ops), re-pointed to the active game once resolved.

This plan removes both artifacts: the newtype **and** the second instance. One
`storage: Arc<Storage>` everywhere.

## Key Changes

### Production wiring

- `run.rs`: create a single `storage` (`game_id = 1` placeholder) instead of
  `preset_storage` + `storage`. Seed presets + game data, look up world/persona
  /NPCs, resolve `active_game_id`, then `storage.set_game_id(active_game_id)`.
  Drop `PRESET_STORAGE_GAME_ID` and the `preset_storage` field on
  `PreparedData`. All current `preset_storage.*` calls become `storage.*`.
- `wiring.rs`: `WiredApp` loses `preset_storage`. `build_app_graph` and
  `build_app_graph_for_tests` lose the `preset_storage` parameter. The single
  `storage` feeds `SettingsService`, `PromptPresetService`,
  `MessageService`, `WorldCatalogue`, `PersonaCatalogue`, `GameCatalogue`,
  `GameViewQuery`, `ActionPipeline`, and `AgentRegistry::from_configs_with_storage`.
- `ActionPipeline`: drop the `preset_store` field. `with_storage`,
  `with_backends`, `with_mock_quantifier`, and `rebind_for_test` lose the
  `preset_store` parameter. Preset reads in `phases.rs` become
  `self.pipeline.storage.get_preset(...)`.
- `GameViewQuery`: drop the `preset_store` field and constructor param. Preset
  read at line 196 becomes `self.storage.get_preset(...)`.

### Storage layer

- Delete `src/adapters/driven/storage/preset_store.rs`.
- Remove `pub mod preset_store;` and `pub use preset_store::PresetStore;` from
  `src/adapters/driven/storage/mod.rs`.
- `Storage::get_preset` / `save_preset` / `delete_preset` / `list_presets`
  (in `backend/presets.rs`) are unchanged — they are the real implementation.

### Test wiring

- `test_support/context.rs`: `default_test_preset_storage()` returns a fresh
  in-memory `Storage` with the `system_default` preset seeded. Under the
  collapse this becomes a single test storage. Two options (pick at
  implementation):
  - **(a)** Keep `default_test_preset_storage()` returning a `Storage` that is
    then used as the *single* storage for the test (seed world/snapshot into
    it too). `make_test_pipeline_with_backends` /
    `make_test_pipeline_with_mock_quantifier` stop building a separate preset
    store and pass their `storage` arg straight through (callers must ensure
    `system_default` is seeded — see assumption).
  - **(b)** Replace `default_test_preset_storage()` with
    `seed_default_preset(&storage)` that seeds `system_default` into an
    existing storage. Every test builder that currently calls
    `default_test_preset_storage()` calls `seed_default_preset(&storage)` on
    its single storage instead.
  - Prefer **(b)** — makes the "one storage" invariant explicit.
- `build_test_wired_app` / `build_test_wired_app_with_settings` /
  `build_app_graph_for_tests` lose the `preset_storage` parameter.
- `orchestrator_tests.rs` (4 sites), `tests/helpers/sqlite_test_app_builder.rs`
  (1 construction + 3 arg passes), `tests/infrastructure/invariant_contract.rs`
  (3 sites): stop constructing a second storage; seed `system_default` into the
  single game storage.

## Implementation

### Phase 1: Collapse production wiring

- [ ] #### Task 1.1: `run.rs` single-storage bootstrap (1.5 SP)
  - [ ] ##### SubTask 1.1.1: Replace `preset_storage` + `storage` construction
    with one `storage` at `game_id = 1`.
  - [ ] ##### SubTask 1.1.2: After `resolve_game_id`, call
    `storage.set_game_id(active_game_id)`.
  - [ ] ##### SubTask 1.1.3: Drop `preset_storage` from `PreparedData`; rename
    all `preset_storage.*` call sites to `storage.*`.
  - [ ] ##### SubTask 1.1.4: Drop `PRESET_STORAGE_GAME_ID` const.

- [ ] #### Task 1.2: `wiring.rs` single-storage graph (1 SP)
  - [ ] ##### SubTask 1.2.1: Remove `preset_storage` from `WiredApp`,
    `build_wired_app`, `build_app_graph`, `build_app_graph_for_tests`.
  - [ ] ##### SubTask 1.2.2: Feed `PromptPresetService::new` and
    `AgentRegistry::from_configs_with_storage` the single `storage`.
  - [ ] ##### SubTask 1.2.3: Drop `PresetStore` import; remove the
    `PresetStore::new(...)` construction sites.

- [ ] #### Task 1.3: Drop the preset field from `ActionPipeline` and `GameViewQuery` (1 SP)
  - [ ] ##### SubTask 1.3.1: `pipeline.rs` — remove `preset_store` field and
    the parameter from `with_storage`, `with_backends`, `with_mock_quantifier`,
    `rebind_for_test`.
  - [ ] ##### SubTask 1.3.2: `phases.rs:393` —
    `self.pipeline.preset_store.get_preset(...)` →
    `self.pipeline.storage.get_preset(...)`.
  - [ ] ##### SubTask 1.3.3: `view_query.rs` — remove `preset_store` field and
    constructor param; line 196 → `self.storage.get_preset(...)`.

### Phase 2: Collapse test wiring

- [ ] #### Task 2.1: Single-storage test helpers (1 SP)
  - [ ] ##### SubTask 2.1.1: `test_support/context.rs` — replace
    `default_test_preset_storage()` with `seed_default_preset(&storage)`;
    update `make_test_pipeline_with_backends`,
    `make_test_pipeline_with_mock_quantifier`, `build_test_wired_app`,
    `build_test_wired_app_with_settings`, `build_test_app` to use one storage.
  - [ ] ##### SubTask 2.1.2: `test_support/test_app_builder.rs` — single
    storage, seed default preset.
  - [ ] ##### SubTask 2.1.3: `src/application/orchestrator_tests.rs` — 4 sites.
  - [ ] ##### SubTask 2.1.4: `tests/helpers/sqlite_test_app_builder.rs` —
    construction + 3 arg passes.
  - [ ] ##### SubTask 2.1.5: `tests/infrastructure/invariant_contract.rs` — 3
    sites.
  - [ ] ##### SubTask 2.1.6: `src/application/arrival_service_tests.rs` — uses
    `default_test_preset_storage()`; switch to `seed_default_preset`.

### Phase 3: Delete file, regenerate index, verify

- [ ] #### Task 3.1: Cleanup and validate (0.5 SP)
  - [ ] ##### SubTask 3.1.1: `git rm src/adapters/driven/storage/preset_store.rs`.
  - [ ] ##### SubTask 3.1.2: Remove `pub mod preset_store;` and
    `pub use preset_store::PresetStore;` from
    `src/adapters/driven/storage/mod.rs`.
  - [ ] ##### SubTask 3.1.3: Run `python scripts/generate_structure_index.py`
    so the `preset_store.rs` row leaves `AGENTS.md` STRUCTURE (pre-commit hook
    also does this).
  - [ ] ##### SubTask 3.1.4: `cargo fmt && cargo clippy --all-targets -- -D warnings`.
  - [ ] ##### SubTask 3.1.5: `python build.py` — full gate.

## Test Plan

- `cargo check --all-targets` — every collapsed call site compiles.
- `rg -n "PresetStore|preset_storage|PRESET_STORAGE_GAME_ID" src/ tests/`
  returns zero matches after the edit.
- `python build.py` green (fmt + clippy + guardrails + architecture + tests).
- Production behavior unchanged: preset reads route through the same `DbPool`
  and the same `prompt_presets` table as before; only the handle count drops.
- Tests: `system_default` preset is seeded into the single game storage, so
  `get_preset("system_default")` returns the same row it did when presets lived
  in a separate in-memory store.

## Assumptions

- `set_game_id` is safe to call after seeding: it updates `games.updated_at`
  for the target row and re-points the instance's `AtomicU64`. Seeding and
  global-table lookups are game_id-agnostic, so the placeholder `game_id = 1`
  during bootstrap is harmless. Confirm at implementation that no game-scoped
  write occurs between construction and `set_game_id` (none expected — bootstrap
  only seeds global tables and reads world/persona).
- `resolve_game_id` operates on `DbPool`, not `Storage`, so it is unaffected by
  the instance's `game_id`.
- `PresetStore::inner()` is unused (confirmed: 0 matches for `\.inner\(\)` in
  `src/` and `tests/`).
- No architecture/guardrails test enumerates `PresetStore` by name (confirmed:
  0 matches in `tests/architecture`, `tests/guardrails`).
- `docs/diataxis/reference/storage.md` does not mention `PresetStore` or
  `preset_storage` (confirmed: 0 matches) — no doc edit needed beyond the
  STRUCTURE index.

## Story Points

5 SP total (3.5 SP production + 1 SP tests + 0.5 SP cleanup/verify).

## Relationships

- **Motivated by:** review of `preset_store.rs` — orphaned newtype and orphaned
  second-storage instance, both left after `ADR-027` removal in `e0f22e0`.
- **Supersedes:** the earlier rename-only draft (which preserved the redundant
  second `Arc<Storage>` field). This plan removes both artifacts.
