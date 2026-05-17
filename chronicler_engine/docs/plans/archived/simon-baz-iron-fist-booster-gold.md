# Plan: Rename Domain Vocabulary — CharacterState, TriggerAction, GenerationState

## Goal
Eliminate cognitive tax from three misnamed domain structures by renaming the Rust types and their container fields, while preserving backward compatibility with existing JSON data files and database snapshots via `#[serde(rename)]`.

---

## Rename Map

| Current (type + field) | New (type + field) | Rationale |
|------------------------|-------------------|-----------|
| `model::trigger::CharacterState` | `NpcEncounterLog` | Tracks encounter logs / met status / fired triggers for all NPCs, not a single character's attributes. |
| `GameState.character_state` | `GameState.npc_encounter_log` | Matches new type name; avoids confusion with player status. |
| `GameStateSnapshot.character_state` | `GameStateSnapshot.npc_encounter_log` | Snapshot mirrors GameState. |
| `model::trigger::TriggerAction` | `TriggerEffect` | A narrative effect (name + prompt template), not an executable engine action. |
| `Trigger.action` | `Trigger.effect` | The field holds a trigger effect. |
| `model::state::GenerationState` | `InputBuffer` | UI typing buffer, cursor, scroll — not LLM inference state. |
| `NarrativeState.generation` | `NarrativeState.input_buffer` | Matches new type name. |
| `NarrativeSnapshot.generation` | `NarrativeSnapshot.input_buffer` | Snapshot mirrors NarrativeState. |

---

## Backward Compatibility (Critical)

Existing JSON files and DB snapshots use the old keys. Add `#[serde(rename)]` on renamed fields:

- `Trigger.effect` → `#[serde(rename = "action")]` (world data files use `"action"`)
- `GameStateSnapshot.npc_encounter_log` → `#[serde(rename = "character_state")]` (DB snapshots store `"character_state"`)
- `NarrativeSnapshot.input_buffer` → `#[serde(rename = "generation")]` (DB snapshots store `"generation"`)

This allows old snapshots to deserialize and existing world data to load without migration.

---

## Files to Modify

### Type definitions
- `src/model/trigger.rs` — rename `CharacterState` → `NpcEncounterLog`, `TriggerAction` → `TriggerEffect`, update `Trigger.action` → `Trigger.effect` with serde rename.
- `src/model/state.rs` — rename `GenerationState` → `InputBuffer`, update `NarrativeState.generation` → `input_buffer`, update `GameState.character_state` → `npc_encounter_log`.

### Snapshot / storage models
- `src/model/state_snapshot.rs` — mirror field renames with serde renames on snapshot structs.
- `src/storage/models/game_state_snapshot.rs` — rename `character_state_json` → `npc_encounter_log_json`.
- `src/storage/mappers/state_snapshot.rs` — update mapper field names and JSON round-trip.
- `src/storage/db.rs` — rename DB column `character_state` → `npc_encounter_log` in schema + SQL.
- `src/storage/snapshot_storage.rs` — update INSERT/SELECT column names.

### Engine & application
- `src/engine/trigger_eval.rs` — update all 7 helper parameters from `CharacterState` to `NpcEncounterLog`.
- `src/engine/action_processing.rs` — update `trigger.action.*` → `trigger.effect.*`, `character_state` → `npc_encounter_log`.
- `src/bootstrap/run.rs` — update `state.narrative.generation.*` → `state.narrative.input_buffer.*`.
- `src/application/game_service/actions.rs` — same generation → input_buffer.
- `src/application/game_service/retry.rs` — same.
- `src/application/game_service/helpers_tests.rs` — same.
- `src/server/debug.rs` — update `character_state` clone + `generation.*` accesses.
- `src/server/fragments/renderers.rs` — `generation.*` → `input_buffer.*`.
- `src/server/fragments/endpoints.rs` — same.
- `src/server/fragments/actions.rs` — same.
- `src/server/fragments/misc.rs` — same.

### Tests & fixtures (~20 files)
- `src/model/trigger_tests.rs`, `src/model/state_tests.rs`, `src/model/state_snapshot_tests.rs`
- `src/engine/trigger_eval_tests.rs`
- `src/test_support/fixtures.rs`
- `src/test_support/in_memory_storage_tests.rs`
- `src/storage/snapshot_storage_tests.rs`, `src/storage/mappers/state_snapshot_tests.rs`
- `src/bootstrap_tests.rs`
- `src/narrative/llm/mock_tests.rs`
- `tests/diagnostic/scenarios.rs`
- `tests/flow_mock/retry_main.rs`, `tests/flow_mock/retry_event.rs`
- `tests/helpers/game_service.rs`
- `src/test_support/context_tests.rs`

### Total: ~24 `.rs` files (production + tests).

---

## Validation Steps

1. `cd chronicler_engine && cargo check` — zero errors.
2. `cargo test` — all tests pass.
3. `cargo test --test flow_mock` — integration tests with real JSON data files pass (serde rename compatibility check).
4. Verify no remaining references with: `rg "CharacterState\b|TriggerAction\b|GenerationState\b" src/ tests/`.

---

## Approach

**Single approach — mechanical rename with serde compatibility shim.**

No logic changes. All renames are type/field identifier swaps plus `#[serde(rename)]` attributes to avoid breaking existing world data and player snapshots. The DB schema is updated in `db.rs` to match the new vocabulary (fresh installs get the new column name; existing DBs will auto-migrate on next snapshot write because the Rust code will read the old key via serde rename, but the SQL column name change requires consideration — if the project uses `CREATE TABLE IF NOT EXISTS`, existing DBs retain the old column. The safest path is to also rename the column in `db.rs` and accept that existing local dev DBs may need a reset, or add a one-time `ALTER TABLE` migration step in the plan during execution.)
