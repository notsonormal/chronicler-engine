# Subplan C: Atomic mapless enablement

**Parent Plan:** [mapless-worlds-plan.md](./mapless-worlds-plan.md)
**Status:** Decision-complete
**Date:** 2026-06-25
**Depends on:** Subplan A (relocate starting_room_id), Subplan B (quantifier field split)
**Blocks:** none (terminal subplan)

---

## Summary

The atomic switch. After Subplans A and B ship, this subplan:

1. Deletes the `dynamic_rooms` subsystem + `create_dynamic_room`.
2. Changes `current_room_id: String → Option<String>` + adds `current_room_name: Option<String>`.
3. Deletes `NarrativeState.pending_location`; `push_message` reads `current_room_name` (not drained).
4. Adds `WorldCard.has_map: bool`.
5. Replaces Subplan A's `StartingScenario.starting_room_id: String` with `starting_location_id: Option<String>` + `starting_location_name: Option<String>` (strict validator matrix from parent plan Q6).
6. Adds `resolve_starting_location` helper (returns Option types).
7. Refactors narrator prompt builder to accept `Option<&Room>` + `location_name: &str`.
8. Adds synthetic `Room` fallback for quantifier when `current_room_id=None`.
9. Updates `trigger_eval.rs:22` type comparison for Option.
10. Rewrites INV-ROOM diagnostics.
11. Updates debug views.
12. Storage migration v15 (add `has_map` column).

**BREAKING:** Existing saved games must be reset with `python build.py --cleanup` (per parent plan Section 16, ADR-026 precedent). ADR-027 lands with this subplan.

---

## Key Changes

### 1. `WorldCard` + `WorldManifest` (`src/model/world.rs`)

- Add `has_map: bool` field with `#[serde(default = "default_has_map_true")]` on both structs.
- Add `fn default_has_map_true() -> bool { true }`.
- Update `Default for WorldCard` impl: set `has_map: true`.
- Update `From<WorldManifest> for WorldCard` mapping: `has_map: manifest.has_map`.
- Subplan A already removed `starting_room_id` from these structs — no change here.

### 2. `StartingScenario` (`src/model/scenario.rs`)

- Subplan A added `starting_room_id: String`. Subplan C replaces it with:

  ```rust
  #[serde(default)]
  pub starting_location_id: Option<String>,
  #[serde(default)]
  pub starting_location_name: Option<String>,
  ```

- Delete `fn default_starting_room()` (no longer needed).
- During load: existing `starting_room_id` field in DB scenario JSON deserializes fine if it's there but is ignored (field name doesn't match `starting_location_id`/`starting_location_name` — serde drops it; `build.py --cleanup` ensures fresh seed).

### 3. `MovementState` (`src/model/state.rs`)

```rust
struct MovementState {
    #[serde(default)]
    pub current_room_id: Option<String>,   // Some for on-map mapped worlds, None for mapless/off-map
    #[serde(default)]
    pub current_room_name: Option<String>, // Both kinds, canonical persistent
    // dynamic_rooms: DELETED (was HashMap<String, Room>)
}
```

- Serde: old `current_room_id: "front_gates"` (String) load → serde coerces to `Some("front_gates")` (Option<String> null-coercion). Old `dynamic_rooms` field in snapshot JSON drops silently (no `deny_unknown_fields`). `current_room_name` `#[serde(default)]` → `None` for old snapshots; self-heals via `log_movement_completion` on next movement. `build.py --cleanup` required directive makes this academic.

### 4. `NarrativeState` (`src/model/state.rs`)

- Delete `pending_location: Option<String>` field.
- Delete from `from_snapshot` reader.

### 4b. `NarrativeState` field refactor blast radius

1. `NarrativeSnapshot` (`model/state_snapshot.rs:20, 34, 70`): delete `pending_location` field. (No replacement — `current_room_name` lives on `MovementState`, serialized via `movement: state.movement.clone()`.)
2. `push_message` (`state.rs:307-329`) replacement:

   ```rust
   // state.rs:308 — was:
   let location_header = self.narrative.pending_location.take();
   // becomes:
   let location_header = self.movement.current_room_name.clone();  // NOT drained
   ```

   `current_room_name` is canonical persistent (not drained). `pending_event` continues to drain unchanged (parent plan Q13 decision).
3. Rewrite `log_movement_completion` (`engine/action_processing.rs:67-72`):
   - On-map (mapped, `id` resolved): `state.movement.current_room_name = Some(current_room.name.clone())`.
   - Off-map/mapless: `state.movement.current_room_name = Some(destination_name.unwrap_or(destination_id.unwrap_or("unknown")))`.
4. Rewrite `bootstrap/state.rs:25` + `bootstrap/scenario.rs:21`: write `state.movement.current_room_name = Some(room_name)` (not `state.narrative.pending_location`).
5. All `current_room_id` String readers switch to `Option<String>` (Subplan B's `attempt_semantic_walk` already takes `Option<&str>` — Subplan C changes the field being assigned).
   - `bootstrap/run.rs:100` (type changes to `Option<String>`, transparent)
   - `application/action_pipeline/phases.rs:63` (`map.get_room_by_id(&state.movement.current_room_id)` → `map.get_room_by_id(id)?` inside `if let Some(id) = state.movement.current_room_id.as_deref()`)
   - `application/query_handlers.rs:173` (cloning `Option<String>`, transparent)
   - `engine/state_diagnostics.rs:32` (see Section 9)
6. All `current_room_id == "room1"` test assertions → `Some("room1".to_string())`. Full list in parent plan Section 4b blast radius — includes `engine/logic_tests.rs:190,199,222`, `application/context_tests.rs:131,147,154,323`, `storage/backend/snapshots_tests.rs:80,86,90,99,104,109,151,155`, `storage/mappers/state_snapshot_tests.rs:31,32`, `engine/trigger_eval_tests.rs:194,216`, `engine/action_processing_tests.rs:93,98,207,212,219,234,240,245`.
7. All `pending_location == Some(...)` test assertions → `current_room_name`: `model/state_snapshot_tests.rs:17,38,51,52`, `storage/backend/snapshots_tests.rs:164,170`, `engine/action_processing_tests.rs:250-257`.

### 5. Bootstrap (`src/bootstrap/`)

To avoid three inline copies of room-name resolution, extract `resolve_starting_location` helper:

```rust
// bootstrap/scenario.rs (or new module bootstrap/location.rs)
pub fn resolve_starting_location(
    world: &WorldCard,
    map: &MapDef,
    scenario: &StartingScenario,
) -> (Option<String>, Option<String>) {  // (current_room_id, current_room_name)
    match world.has_map {
        true => {
            let id = scenario.starting_location_id.clone();
            let name = id.as_ref()
                .and_then(|i| map.get_room_by_id(i))
                .map(|r| r.name.clone());
            (id, name)
        }
        false => (None, scenario.starting_location_name.clone()),
    }
}
```

All three bootstrap sites (`bootstrap/state.rs:build_fresh_initial_state`, `bootstrap/scenario.rs:inject_scenario_logs`, `bootstrap/init_game.rs:74,133`) call `resolve_starting_location(...)` instead of inlining.

- Mapped boot: `current_room_id = Some(scenario.starting_location_id)`, `current_room_name = Some(map.get_room_by_id(id).name)`.
- Mapless boot: `current_room_id = None`, `current_room_name = Some(scenario.starting_location_name)`.

### 6. `GameState::new` / `GameStateBuilder::new` signature change (`src/model/state.rs`)

Option (B) per parent plan: constructor takes resolved values directly.

- `GameStateBuilder::new`: signature change — `starting_room: impl Into<String>` → `current_room_id: Option<String>` + `current_room_name: Option<String>`.
- `GameState::new`: same signature change. Forwards to builder.
- Callers (27+ sites): pass `(Some("room1".to_string()), Some("Room 1".to_string()))` instead of `"room1".to_string()`. Production sites (`bootstrap/state.rs:10`, `bootstrap/init_game.rs:69,128`, `application/context.rs:143,162`) call `resolve_starting_location` first. Test sites pass explicit `Option<String>` pairs. Full caller list in parent plan Section 6.

### 7. `current_room()` method semantics (`src/model/state.rs:337`)

```rust
pub fn current_room(&self) -> Option<&Room> {
    self.movement.current_room_id.as_deref()
        .and_then(|id| self.map.get_room_by_id(id))
}
```

- Returns `None` for mapless/off-map.
- Does NOT read `current_room_name` (keeps method focused on authored room).
- Six call sites already Option-tolerant — no extra blast-radius changes (parent plan Section 7).

### 8. Engine logic (`src/engine/logic.rs`) + validator (`src/bootstrap/validate.rs`)

**Delete `create_dynamic_room` function entirely** (`logic.rs:31`).

**Rewrite `attempt_semantic_walk` (Subplan B already changed signature)** — Subplan C changes field assignment side-effects:

- If `destination_id` is `Some(id)` and resolves via `find_room_in_world_map` → on-map: `current_room_id = Some(id)`, `current_room_name = Some(room.name)` (authored name wins, ignore LLM's `destination_name`).
- Else (miss or `None`) → drift/mapless: `current_room_id = None`, `current_room_name = Some(destination_name.unwrap_or(destination_id.unwrap_or("unknown")))`. **No dynamic_room creation** (subsystem deleted).
- Return movement message.
- `action_processing.rs:46-48` dynamic_room creation block: deleted (already done in Subplan A → may already be partially gone; final verify in this subplan).

**Validator (`bootstrap/validate.rs`):**

- Subplan A added per-scenario `starting_room_id ∈ valid_room_ids` check. Subplan C replaces with strict matrix per parent plan Q6:

  ```rust
  for scenario in &world.scenarios {
      let id = &scenario.starting_location_id;
      let name = &scenario.starting_location_name;
      match (world.has_map, id, name) {
          (true,  Some(_), Some(_))  => errors.push(format!(
              "scenario '{}': both location_id and location_name set", scenario.id)),
          (true,  Some(rid), None)
                 if !valid_room_ids.contains(rid)
                                 => errors.push(format!(
              "scenario '{}': location_id '{}' not found in map", scenario.id, rid)),
          (true,  Some(_), None)     => {},  // valid
          (false, Some(_), _)         => errors.push(format!(
              "scenario '{}': has_map=false but location_id set", scenario.id)),
          (false, None,    Some(n)) if n.trim().is_empty()
                                     => errors.push(format!(
              "scenario '{}': empty location_name", scenario.id)),
          (false, None,    Some(_))  => {},  // valid
          (_,      None,    None)    => errors.push(format!(
              "scenario '{}': neither location_id nor location_name set", scenario.id)),
      }
  }
  ```

- Trigger `room_id` validation unchanged.

### 9. State diagnostics (`src/engine/state_diagnostics.rs`)

INV-ROOM rewrite:

```rust
fn assert_room_exists(state: &GameState) -> Result<(), EngineError> {
    if let Some(id) = &state.movement.current_room_id {
        if state.map.get_room_by_id(id).is_none() {
            return Err(EngineError::Internal(internal_error(format!(
                "current_room_id '{id}' not found in map"
            ))));
        }
    }
    if state.movement.current_room_name.is_none() {
        return Err(EngineError::Internal(internal_error(
            "current_room_name is None (must always be Some after bootstrap)"
        )));
    }
    Ok(())
}
```

### 10. Narrator prompt builder (`src/narrative/prompt/`)

- `PromptContext` (`prompt/types.rs:29`): replace `room: &'a Room` field with `room: Option<&'a Room>` + new `location_name: &'a str`.
- `LayerRenderer` (`prompt/assembler.rs:163`): same field change.
- `make_prompt_context` (`prompt/context.rs:109`): signature change — `room: &'a Room` → `room: Option<&'a Room>` + `location_name: &'a str`.
- `render_game_state_layer` (`prompt/assembler.rs:204-212`) rewrite:

  ```rust
  fn render_game_state_layer(&self) -> String {
      let mut output = String::from("<GameState>\nCurrent Location: ");
      if let Some(room) = self.room {
          output.push_str(&room.name);
          output.push_str("\n\n");
          output.push_str(&render_template(&room.description, self.template_vars));
      } else {
          output.push_str(self.location_name);
      }
      output.push_str("\n\n</GameState>\n");
      output
  }
  ```

- All `make_prompt_context` call sites updated:
  - `application/action_pipeline/phases.rs:73` (narration phase — pass `Option<&Room>` from `state.current_room()` + `state.movement.current_room_name.clone().unwrap_or_default()`)
  - `application/action_pipeline/phases.rs:332` (trigger re-narration — same; `state.current_room()?` already Option)
  - `bootstrap/init_game.rs:161` (initial narration — same)
  - Tests: `context_tests.rs:104`, `assembler_tests.rs:114, 207, 237, 277`.

### 11. Quantifier synthetic Room fallback (`src/narrative/agents/quantifier/agent.rs`)

Subplan B made `MovementJson` emit `destination_id`/`destination_name` and `StatePatch::Scene` carry both fields. Subplan C adds the synthetic Room fallback for when `ctx.current_room` is `None` (mapless / off-map):

```rust
let current_room = match ctx.current_room {
    Some(r) => r,
    None => {
        let name = state.movement.current_room_name
            .as_deref()
            .unwrap_or("unknown");
        // Synthetic Room — quantifier signature unchanged downstream
        Room {
            id: name.to_string(),
            name: name.to_string(),
            description: String::new(),
            exits: HashMap::new(),
            items: vec![],
            image_path: None,
            navigation_description: None,
        }
    }
};
```

- Note: synthetic Room is borrowed via `Cow` or the quantifier entry is refactored to accept `Option<&Room>` + `location_name: &str` (cleaner). Decision deferred to first failing test: prefer `&Room` reference via `Cow::Owned`, or refactor `determine_npcs_in_room` signature to `Option<&Room>` + `location_name: &str`.
- If the refactor signature is chosen, `determine_npcs_in_room` (`orchestration.rs:181`) and its callers update.

### 12. Trigger eval (`src/engine/trigger_eval.rs`)

- Line 13: `let current_room_id = &state.movement.current_room_id;` → type is `&Option<String>`.
- Line 22 rewrite:

  ```rust
  if Some(room_id) != current_room_id.as_deref() {
      // skip
  }
  ```

- Semantics: authored triggers with `room_id=Some(...)` never fire when `current_room_id=None` (off-map/mapless) — correct.

### 13. Server views

**`DebugStateView` (`src/application/application_service.rs:85-94`):**

- `current_room_id: String` → `Option<String>`.
- Add `current_room_name: Option<String>`.
- Drop `dynamic_rooms: Vec<String>` + `dynamic_room_count: usize`.

**`Query handler builder` (`src/application/query_handlers.rs:165,172`):**

- Line 165: delete `dynamic_rooms.keys().cloned().collect()`.
- Line 172: `current_room_id: game_state.movement.current_room_id.clone()` (transparent — cloning Option).
- Add: `current_room_name: game_state.movement.current_room_name.clone()`.

**`server/debug.rs:17`:**

- `current_room_id: String` → `Option<String>`.
- Add `current_room_name: Option<String>`.
- Delete `dynamic_rooms` and `dynamic_room_count` fields.

### 14. World form (`src/server/worlds_fragment/handlers.rs` + `fragments.rs`)

- `WorldForm` struct: add `has_map: Option<bool>` field (HTML checkbox unchecked = None → default true).
- `into_world_card()`: add `has_map: self.has_map.unwrap_or(true)`.
- `render_world_edit_form`: render `has_map` checkbox. Per-scenario sub-form: replace `starting_room_id` input (added in Subplan A) with `starting_location_id` + `starting_location_name` inputs.
- Existing mapped worlds (redmist/test) must still load + save correctly via the form.
- Note: HTML form's per-scenario inputs are dynamic (added via JavaScript when scenarios are added). Update the scenario template + the parse logic that builds `scenarios_json` from form fields.

### 16. Storage migration v15 (`src/storage/db.rs`)

Subplan A migration v14 dropped `worlds.starting_room_id`. This subplan adds v15:

```rust
if version < 15 {
    let exec = |sql: &str| {
        conn.execute(sql, [])
            .map_err(|e| crate::error::EngineError::Config(format!("Migration failed: {e}")))
    };

    exec("ALTER TABLE worlds ADD COLUMN has_map INTEGER NOT NULL DEFAULT 1")?;
    // Existing world rows default has_map=1 (true) — matches prior behavior (all worlds had maps)

    conn.pragma_update(None, "user_version", 15).map_err(|e| {
        crate::error::EngineError::Config(format!("Failed to set user_version: {e}"))
    })?;
}
```

- Schema-only migration (ADR-026 precedent).
- World JSON reload from disk provides scenario fields. `build.py --cleanup` required for fresh state.

### 17. Data files

- `data/worlds/redmist_estate/world.json`: add `"has_map": true`. In each scenario, replace `"starting_room_id": "front_gates"` (added in Subplan A) with `"starting_location_id": "front_gates", "starting_location_name": null`.
- `data/worlds/test/world.json`: same pattern.
- `data/schemas/world.schema.json`: add `has_map` (boolean, default true). Replace scenario `starting_room_id` (required string) with `starting_location_id` (optional string) + `starting_location_name` (optional string).
- `data/prompt_presets/quantifier/default.json`: unchanged (updated in Subplan B).

### 18. Implementation order

Per chronicler-dev-workflow: **Architecture First**, **Tests First**.

1. Plan doc (this file).
2. ADR-027 — `docs/adr/adr-027-mapless-worlds.md`.
3. Architecture + system + reference docs (see Section 19).
4. Model layer tests first — `MovementState` serde round-trip (Option round-trip), `resolve_starting_location` all branches, `WorldCard`/`WorldManifest` serde (has_map default true), `StartingScenario` validator matrix (6 cells).
5. Model layer code — `world.rs`, `scenario.rs`, `state.rs`, `state_snapshot.rs`, `agent.rs` (`StatePatch::Scene` already split by Subplan B), `quantifier.rs` (`MovementParseResult` already split by Subplan B).
6. Bootstrap tests first — `validate_tests.rs` strict matrix rewrite (Subplan A's 5 cases + new matrix cases), `resolve_starting_location` integration.
7. Bootstrap code — `bootstrap/scenario.rs` (helper), `state.rs`, `init_game.rs`, `validate.rs`.
8. Engine tests first — `attempt_semantic_walk` tests (on-map hit, on-map miss → drift, mapless always drift), `trigger_eval.rs` type-check test, `state_diagnostics.rs` INV-ROOM rewrite test.
9. Engine code — `engine/logic.rs` (delete `create_dynamic_room`, update `attempt_semantic_walk`), `engine/trigger_eval.rs`, `engine/state_diagnostics.rs`, `engine/action_processing.rs` (rewrite `log_movement_completion`).
10. Narrator tests first — `make_prompt_context` Option signature test, `render_game_state_layer` on-map vs off-map branch test.
11. Narrator code — `prompt/types.rs`, `prompt/assembler.rs`, `prompt/context.rs`, `phases.rs:73,332`, `bootstrap/init_game.rs:161`.
12. Quantifier tests first — synthetic-Room path test (`current_room=None`).
13. Quantifier code — `agent.rs` synthetic Room fallback.
14. Server form tests first — `into_world_card` form-to-card test (has_map default true, scenario location id/name).
15. Server code — `worlds_fragment/handlers.rs` + `fragments.rs`, `application_service.rs` `DebugStateView`, `query_handlers.rs:165,172`, `server/debug.rs`.
16. Storage migration v15 — `storage/db.rs` migration block + migration test.
17. Data files — `data/worlds/redmist_estate/world.json`, `data/worlds/test/world.json`, `data/schemas/world.schema.json`.
18. Test sweep — update all `current_room_id == "room1"` assertions to `Some("room1".to_string())`; update all `pending_location` assertions to `current_room_name`. (~20 test files.)
19. Validate — `python build.py --cleanup`.
20. Archive — `docs/plans/archived/subplan-c-mapless-enablement.md`.

### 19. Documentation

Updated BEFORE code per chronicler-dev-workflow:

1. `docs/plans/subplan-c-mapless-enablement.md` — this file.
2. `docs/adr/adr-027-mapless-worlds.md` — Problem (can't author worlds without maps; dynamic_rooms subsystem overhead; `destination` field conflation), Decision (mapless via empty MapDef + freeform `current_room_name`; delete dynamic_rooms; `destination_id`/`destination_name` split — note Subplan B), Alternatives (Q4-β both-kinds drop `current_room_id` — rejected, mapped worlds need it; Q9-A key by name — rejected, synonym drift; Q9-B normalized name — rejected, same engine limit), Consequences (BREAKING: existing saved games must be reset with `build.py --cleanup`; migration v14 from Subplan A + v15 from Subplan C), Architecture Impact (Modified Modules table), Verification (Section 18 test plan), Related (ADR-026 migration precedent, ADR-008 snapshot persistence).
3. `docs/architecture/system.md` — MovementState section, StartingScenario section, WorldCard section.
4. `docs/system/dynamic_rooms.md` — **DELETE entirely** (subsystem removed). Remove from `docs/README.md` auto-index.
5. `docs/system/navigation.md` — update: map lookup, off-map drift to `current_room_name`, scenario-driven starting location, no dynamic rooms, `attempt_semantic_walk` final signature.
6. `docs/system/startup.md` — bootstrap section: scenario resolves starting location via `resolve_starting_location` helper; both world kinds.
7. `docs/system/worlds.md` — `WorldCard.has_map` field, `StartingScenario` optional id/name, strict validator matrix.
8. `docs/system/triggers.md` — authored triggers with `room_id` never fire in mapless worlds or when mapped-world drift is active.
9. `docs/architecture/guardrails.md` §5 Runtime Invariants — INV-ROOM description: `current_room_id=None` valid for mapless/off-map; `current_room_name` must always be `Some` after bootstrap.
10. `docs/reference/data_schemas.md` — world JSON schema: add `has_map`, replace scenario `starting_room_id` with `starting_location_id`/`starting_location_name`.
11. `docs/reference/data_layer.md` — mention v14 (Subplan A) + v15 (this subplan) migrations + `build.py --cleanup` requirement.
12. `docs/system/agent_system.md` — quantifier consumes `destination_id`/`destination_name` (Subplan B); synthetic Room fallback when `current_room=None`.
13. `docs/system/game_flow.md` — update movement pipeline section (final `attempt_semantic_walk` signature, `current_room_name` parallel write).
14. `docs/system/prompt_system.md` — if documents GameState layer, note Option-based room context.
15. `docs/CHANGELOG.md` — under "Unreleased":
    - **BREAKING:** Removed `dynamic_rooms` subsystem; existing saved games must be reset with `python build.py --cleanup`.
    - Added `WorldCard.has_map` boolean; worlds without maps now supported.
    - Replaced `StartingScenario.starting_room_id` with `starting_location_id` + `starting_location_name` (optional, strict validator matrix).
    - `current_room_id: Option<String>` + new `current_room_name: Option<String>` (canonical persistent location).
    - Deleted `NarrativeState.pending_location`; `push_message` reads `current_room_name` (not drained).
    - Narrator prompt accepts `Option<&Room>` + `location_name: &str` for off-map/mapless contexts.
    - Quantifier synthetic Room fallback when `current_room_id=None`.
    - Storage migration v15: worlds table gains `has_map`.
16. `docs/README.md` auto-index — regenerated by build process.
17. ADR-027 Consequences section explicitly notes: **breaks in-progress games**. `build.py --cleanup` is the supported reset path.

### 20. Test plan

**Unit tests:**

- `MovementState` serde round-trip (Option fields, no `dynamic_rooms` in serialized output).
- `resolve_starting_location` all branches: mapped id-found, mapped id-not-found, mapless name-set, mapless name-empty.
- `WorldCard` / `WorldManifest` serde back-compat: missing `has_map` defaults to `true`.
- `StartingScenario` validator matrix — all 6 cells (has_map × id-set × name-set).
- `current_room()` returns `None` when `current_room_id=None`, `Some(&Room)` when id resolves.
- `attempt_semantic_walk` on-map hit, on-map miss → drift (no dynamic_room creation), mapless always drift.
- `push_message` reads `current_room_name` without draining.
- INV-ROOM diagnostics: `Some(id)` missing from map → error; `None` → ok (if current_room_name Some); `current_room_name=None` → error.
- `make_prompt_context` Option signature; `render_game_state_layer` on-map vs off-map branches.
- Synthetic-Room path: `ctx.current_room=None` → transient Room built from `current_room_name`, quantifier succeeds.
- `trigger_eval`: authored triggers never fire when `current_room_id=None`.
- `into_world_card` form parsing: `has_map` default true, scenario location id/name carry-through.

**Integration tests:**

- Full mapped-world turn: player walks into authored room → `current_room_id=Some(id)`, `current_room_name=Some(room.name)`, narrator receives authored description.
- Full mapped-world drift: quantifier emits `destination_id="hidden_cellar"` (not in map) → `current_room_id=None`, `current_room_name=Some("Hidden Cellar")` (from `destination_name`); narrator receives only name; next turn quantifier emits `destination_id="tavern"` (in map) → self-heals to `Some("tavern")` + `Some("Tavern")`.
- Full mapless-world turn: bootstrap seeds `current_room_id=None`, `current_room_name=Some("...")`; every movement keeps `current_room_id=None`, updates `current_room_name`; narrator never receives authored description (correct — there isn't one).
- Storage migration v15: fresh DB schema has `has_map` column.

**E2E (browser, only if form changes break):**

- World edit form: create new mapped world with `has_map=true`, scenario `starting_location_id` set → save → verify DB row.
- World edit form: create new mapless world with `has_map=false`, scenario `starting_location_name` set → save → verify DB row.

---

## Assumption Notes (Deviation Protocol)

Any scope discovered mid-implementation triggers the STOP-and-report protocol per project `AGENTS.md` "Plan Adherence" rule:

- Discovered bug in unrelated code → STOP, surface to user, offer A) fix now (deviate) or B) add to plan and continue.
- Mid-implementation refactor opportunity → STOP, surface, wait for direction.
- Plan gap discovered → STOP, surface, wait.
- "Better" approach than agreed → STOP, surface, wait.

User has 61% plan-failure rate caused by agent drift. Do not be helpful at the expense of predictability.

## NOT in scope

1. NPC spawn/encounter system in mapless worlds — quantifier gets synthetic `&Room`; NPC detection uses existing `state.npcs` values path. No new NPC routing.
2. Return-travel bug for mapped-world off-map drift — partially addressed by Subplan B's `destination_id`/`destination_name` split (LLM can reuse stable id), but no engine-side name-normalization. Off-map drift locations still rely on conversation history for re-identification.
3. Mapless seed world JSON — no `data/worlds/mapless_demo/`. Mapless behavior covered by unit + integration test fixtures.
4. Migration of existing saved games — `build.py --cleanup` required. In-progress games abandoned (ADR-026 precedent).
5. `pending_event` staging semantics — unchanged per Q13 decision.
6. Trigger firing for mapless worlds — authored triggers with `room_id=Some(...)` never fire when `current_room_id=None`. Correct behavior, no change.
7. CSS / visual UI changes — none. Section 14 (HTML form field changes) is structural; existing templates handle rendering.
8. Subplan A's `starting_room_id` relocation — already shipped. Subplan C replaces with `starting_location_id`/`starting_location_name`.
9. Subplan B's quantifier field split — already shipped. Subplan C only adds the synthetic-Room fallback on top.

## What already exists

1. `AgentContext.current_room: Option<&'a Room>` (`model/agent.rs:115`) — already Option-typed. No struct change.
2. `current_room()` method returning `Option<&Room>` (`model/state.rs:337`) — already Option; refactor simplifies body (drop `dynamic_rooms` fallback), signature unchanged.
3. Bootstrap migration precedent — ADR-026 for schema-only migrations + `build.py --cleanup` reset path.
4. `Trigger.room_id: Option<String>` (`model/trigger.rs:32`) — already Option.
5. `StartingScenario` deserializes from `scenarios_json` in WorldForm — transparent to form handler; only HTML inputs change.
6. `build.py --cleanup` — supported reset path, wipes + re-seeds from `data/`.
7. Quantifier post-generation patch pipeline (`StatePatch::Scene` + `StatePatch::merge`) — already handles Option merging via first-non-None. Subplan B split into two Option fields already reuses same pattern.
8. Subplan A's per-scenario `starting_room_id` validator check — replaced by this subplan's strict matrix.

## Failure modes

1. **Bootstrap: scenario missing for mapless world.** Validator catches: both `starting_location_id` and `starting_location_name` = None → error names the scenario. World load fails loudly.
2. **Bootstrap: mapless world has `starting_location_id` set.** Validator catches: `has_map=false` but `id=Some(...)` → error. Strict matrix.
3. **Mapped world drift: id missing from map.** `attempt_semantic_walk` falls through to drift: `current_room_id=None`, `current_room_name=Some(destination_name)`. Self-heals on next turn if LLM emits valid map id. No hard error.
4. **Narrator receives `current_room_id=None` + `current_room_name=None`.** Cannot happen in normal flow — bootstrap seeds at least `current_room_name`. If it does (corrupt state): `render_game_state_layer` emits empty `<GameState>Current Location: </GameState>`. Narrator tolerates empty field. Guard test added.
5. **Quantifier LLM omits `destination_id`/`destination_name`.** Parser produces None for both (Subplan B behavior). Treated as no-movement. Existing behavior for malformed quantifier responses — no regression.
6. **`current_room_id` orphaned (Some(id) where id ∉ map).** Caught by `assert_room_exists` (INV-ROOM, diagnostics feature only). In production (no diagnostics): `current_room()` returns None, narrator falls to off-map path. Self-heals on next movement.
7. **Old-format quantifier JSON `{"destination": "kitchen"}` reaches parser.** Subplan B already handled — no legacy fallback, parse fails → no-movement. Subplan C changes nothing here.
8. **`build.py --cleanup` not run before first boot after upgrade.** Old snapshots load → `current_room_id="front_gates"` (legacy String) → serde coerced to `Some("front_gates")`; `current_room_name=None` (new field default). Map lookup succeeds for mapped worlds → narrator on-map path works. `current_room_name` self-heals on next movement via `log_movement_completion`. Only breaks if player never moves and never re-saves. Mitigated by required reset directive.
9. **Live game with player in a dynamic room at upgrade time.** Per Section 16: `build.py --cleanup` required. If not run: `current_room_id="dynamic_<ts>"` loads as `Some("dynamic_<ts>")`, map lookup misses, `current_room_name=None`, treated as off-map drift with no name. Narrator may render empty location. Supported reset path is `build.py --cleanup`.

## Unresolved decisions

None. Plan is decision-complete. Synthetic-Room implementation detail (`Cow::Owned` vs signature refactor) is a tactical choice deferred to first failing test — not architectural.
