# Subplan A: Relocate `starting_room_id` to `StartingScenario`

**Parent Plan:** [mapless-worlds-plan.md](./mapless-worlds-plan.md)
**Status:** Decision-complete
**Date:** 2026-06-25
**Depends on:** none
**Blocks:** Subplan C (atomic mapless enablement)

---

## Summary

Behavior-preserving relocation: move `starting_room_id: String` from `WorldCard` to `StartingScenario` (same name, same type). Each scenario now declares its own starting room. Bootstrap, validator, form handler, data files, and tests updated. No `has_map`, no `Option<String>`, no `dynamic_rooms` deletion, no quantifier split — those land in Subplan C / B.

Enables future mapless work by putting starting location on the scenario where it belongs, decoupled from any map-specific world-level field.

**BREAKING to existing saved games** — `build.py --cleanup` required (scenarios column JSON shape changes; `worlds.starting_room_id` DB column dropped).

---

## Key Changes

### 1. `WorldCard` + `WorldManifest` (`src/model/world.rs`)

- Drop `starting_room_id: String` field from both structs.
- Delete `fn default_starting_room()` helper.
- Update `Default for WorldCard` impl: drop `starting_room_id`.
- Update `From<WorldManifest> for WorldCard`: drop `starting_room_id` mapping.
- Serde change: existing world JSON files without `starting_room_id` at world level load cleanly (field removed). Missing field at scenario level → `"start"` default for back-compat during load (see Section 4).

### 2. `StartingScenario` (`src/model/scenario.rs`)

- Add `starting_room_id: String` field with `#[serde(default = "default_starting_room")]`.
- Add `fn default_starting_room() -> String { "start".to_string() }` (moved from world.rs).
- Each scenario now self-describes its starting room.

### 3. Bootstrap (`src/bootstrap/`)

All three sites that read `world.starting_room_id` switch to `scenario.starting_room_id`:

- `bootstrap/state.rs:15,21,23` (`build_fresh_initial_state`): resolve scenario first via `ctx.world.default_scenario()`, then read `scenario.starting_room_id`. If `default_scenario()` returns `None`, fall back to `"start"` literal (matches current behavior when scenarios vector is empty).
- `bootstrap/scenario.rs:17,19` (`inject_scenario_logs`): `world.starting_room_id` → `scenario.starting_room_id` (scenario already in scope via `world.default_scenario()` early return).
- `bootstrap/init_game.rs:74,133`: `world_arc.starting_room_id.clone()` / `self.world.starting_room_id.clone()` → resolve scenario, clone `scenario.starting_room_id`.

Sig change is local — no new helper introduced yet (the `resolve_starting_location` helper from parent plan Section 5 is a Subplan C concern when types become Option).

### 4. Validator (`src/bootstrap/validate.rs`)

- Delete `if !valid_room_ids.contains(&world.starting_room_id)` check (lines 23-29).
- Add per-scenario check:

  ```rust
  for scenario in &world.scenarios {
      if !valid_room_ids.contains(&scenario.starting_room_id) {
          errors.push(format!(
              "scenario '{}' starting_room_id '{}' not found in map",
              scenario.id, scenario.starting_room_id
          ));
      }
  }
  ```

- Empty scenarios vector: skip the loop (matches current behavior — empty scenarios validates ok).
- Trigger `room_id` validation unchanged.

### 5. World form (`src/server/worlds_fragment/handlers.rs` + `fragments.rs`)

- `WorldForm` struct: drop `starting_room_id: Option<String>` field at form level. (scenarios are encoded as JSON in `scenarios_json`, transparent.)
- `into_world_card()` (line 45): delete `starting_room_id: self.starting_room_id.unwrap_or_else(|| "start".to_string())` line. The scenario JSON carries the per-scenario field.
- `render_world_edit_form`: drop the world-level `starting_room_id` input. Add per-scenario `starting_room_id` input to the scenario sub-form (alongside name/description/text/npcs).
- Existing mapped worlds (redmist/test) must still load + save correctly.

### 6. Storage migration v14 (`src/storage/db.rs`)

Schema-only migration (ADR-026 precedent):

```rust
if version < 14 {
    let exec = |sql: &str| {
        conn.execute(sql, [])
            .map_err(|e| crate::error::EngineError::Config(format!("Migration failed: {e}")))
    };

    // SQLite 3.35+ supports DROP COLUMN (rusqlite 0.34 bundled = 3.46+)
    exec("ALTER TABLE worlds DROP COLUMN starting_room_id")?;
    // No new column — starting_room_id moves INTO scenarios JSON

    conn.pragma_update(None, "user_version", 14).map_err(|e| {
        crate::error::EngineError::Config(format!("Failed to set user_version: {e}"))
    })?;
}
```

- `worlds.starting_room_id` column dropped. The `scenarios` column (JSON) already serializes `Vec<StartingScenario>` which now includes `starting_room_id` per scenario.
- Existing in-progress games are abandoned per `build.py --cleanup` reset path.

### 7. Data files

- `data/worlds/redmist_estate/world.json`: remove top-level `"starting_room_id": "front_gates"`. Add `"starting_room_id": "front_gates"` to each scenario object (redmist has one: `bodyguard_intro`).
- `data/worlds/test/world.json`: same pattern — move top-level `starting_room_id` value into each scenario.
- `data/schemas/world.schema.json`: remove top-level `starting_room_id`, add `starting_room_id` (required string, default `"start"`) to scenario schema.

### 8. World tests (`src/model/world_tests.rs`)

- Existing `assert!(manifest.default_scenario().is_none())` assertions (lines 32, 69) unchanged — empty scenarios still valid.
- Add test: `StartingScenario` serde round-trip preserves `starting_room_id`.
- Add test: `WorldCard` deserializes JSON missing top-level `starting_room_id` cleanly (field removed).
- Add test: `WorldManifest` deserializes JSON missing top-level `starting_room_id` cleanly.

### 9. Validator tests (`src/bootstrap/validate_tests.rs`)

- `test_validate_loaded_data_success` (line 9): move `starting_room_id: "room_a".to_string()` from `WorldCard` literal into the scenario literal.
- `test_validate_loaded_data_missing_starting_room` (line 29): now means "scenario references missing room", not "world references missing room". Move `starting_room_id: "missing_room"` into scenario, assert error message contains `"scenario"` + `"starting_room_id"`.
- `test_validate_loaded_data_basic_manifest_succeeds` (line 57): move `starting_room_id: "room_a"` into scenario.
- `test_validate_loaded_data_invalid_trigger_room` (line 79): move `starting_room_id: "room_a"` into scenario.
- `test_validate_loaded_data_multiple_errors` (line 125): move `starting_room_id: "missing"` into scenario; if the test asserts multiple errors, ensure scenario + trigger errors both still fire.

### 10. Bootstrap tests (`src/bootstrap/run_tests.rs`)

- Line 13: `starting_room_id: "gates".to_string()` moves into scenario literal.
- Lines 153, 158: same.
- Line 191: `world_card.starting_room_id.clone()` → resolve scenario, `scenario.starting_room_id.clone()`.

### 11. Other test sweep

Grep for `starting_room_id` across `src/**/*_tests.rs` and update each WorldCard/StartingScenario literal:

- `test_support/fixtures.rs` (if any WorldCard literals)
- `test_support/test_app_builder.rs:266`
- `application/game_service_tests.rs`, `application/context_tests.rs`, `application/query_handlers_tests.rs`, `application/message_editing_tests.rs`
- `engine/logic_tests.rs`, `engine/trigger_eval_tests.rs`, `engine/action_processing_tests.rs`
- `model/state_snapshot_tests.rs`
- `narrative/agents/quantifier/agent_tests.rs`
- `bootstrap/load_tests.rs:48` (JSON literal in test — update to put `starting_room_id` inside scenario)

## Documentation

Updated BEFORE code per chronicler-dev-workflow:

1. `docs/plans/subplan-a-relocate-starting-room.md` — this file.
2. `docs/architecture/system.md` — StartingScenario section mentions `starting_room_id`; WorldCard section drops it.
3. `docs/system/startup.md` — bootstrap reads `scenario.starting_room_id`.
4. `docs/system/worlds.md` — WorldCard no longer has top-level `starting_room_id`; scenarios declare their own.
5. `docs/system/navigation.md` — note starting room sourced from scenario.
6. `docs/reference/data_schemas.md` — world JSON schema updated.
7. `docs/reference/data_layer.md` — mention v14 migration + `build.py --cleanup`.
8. `docs/CHANGELOG.md` — under "Unreleased":
   - Relocate `starting_room_id` from `WorldCard` to `StartingScenario`.
   - Storage migration v14: drop `worlds.starting_room_id` column.
   - BREAKING: existing saved games must be reset with `python build.py --cleanup`.

No ADR for this subplan — it's a pure refactor with no architectural decision. The mapless-worlds ADR-027 lands in Subplan C.

## Implementation Order

1. Plan doc (this file).
2. Architecture + system + reference docs.
3. Model layer: `world.rs`, `scenario.rs` (move field, move helper).
4. Model layer tests: serde round-trip, default, missing-field tolerance.
5. Validator: per-scenario check + 5 `validate_tests.rs` rewrites.
6. Bootstrap: 3 sites updated + `run_tests.rs` + `load_tests.rs` test updates.
7. Form: `WorldForm` + `render_world_edit_form` + scenario sub-form field.
8. Storage: v14 migration block + migration test.
9. Data files: 2 world JSONs + schema.
10. Test sweep: all `starting_room_id` literal moves.
11. Validate: `python build.py --cleanup`.
12. Archive: `docs/plans/archived/subplan-a-relocate-starting-room.md`.

## Failure Modes

1. **Scenario with `starting_room_id` missing from JSON.** Serde default `"start"` kicks in. If map has no `"start"` room, validator catches: scenario starting_room_id `"start"` not in map. World load fails loudly.
2. **World with empty scenarios vector.** Bootstrap `default_scenario()` returns `None`, boot code falls back to literal `"start"` (existing behavior). If map lacks `"start"` room → error at `map.get_room_by_id("start")`. Same as today.
3. **Form save with scenario missing `starting_room_id` input.** `into_world_card()` accepts whatever JSON the form posts; if `starting_room_id` field absent in scenario JSON, serde default fills it. Validator catches downstream.
4. **Old DB with `worlds.starting_room_id` column.** Migration v14 drops it. `scenarios` JSON in existing rows may or may not include per-scenario `starting_room_id`. If absent, `StartingScenario` serde default fills `"start"`. `build.py --cleanup` ensures fresh seed.
5. **Two scenarios with different `starting_room_id`.** Valid — each scenario can start in a different room. New capability unlocked by this subplan.

## NOT in scope

- `has_map` boolean — Subplan C.
- `current_room_id: Option<String>` — Subplan C.
- `starting_location_id` / `starting_location_name` Option fields — Subplan C replaces this subplan's `starting_room_id` with those.
- `dynamic_rooms` deletion — Subplan C.
- Quantifier `destination_id`/`destination_name` split — Subplan B.
- ADR-027 — Subplan C.
- `resolve_starting_location` helper extraction — Subplan C (when types become Option).

## Verification

```bash
python build.py --cleanup
```

Must pass: fmt + clippy `-D warnings` + nextest. All world loads must succeed. Test sweep covers every `starting_room_id` literal in `src/`.
