# Plan: Mapless Worlds via Freeform Location Names

**Status:** Decision-complete
**Date:** 2026-06-25
**Branch:** (target branch TBD at implementation time)

---

## Summary

Enable worlds with no configured map. `WorldCard` declares `has_map: bool`; when `false` the map is empty and every location is a free-text string the LLM produces each turn. Starting location moves from `WorldCard` to `StartingScenario` (as `starting_location_id` / `starting_location_name`; exactly one set per `has_map` value).

Replaces the current `current_room_id: String` + `dynamic_rooms: HashMap<String, Room>` runtime model with `current_room_id: Option<String>` + `current_room_name: Option<String>` for both world kinds. Mapped worlds keep authored `Room` lookup via `map.get_room_by_id(current_room_id)`; mapless and off-map-drift worlds use `current_room_name` only and rely on conversation history for spatial continuity — same model as a vanilla chat engine.

Deletes the dynamic-rooms subsystem in its entirety. Fixes the latent return-travel bug (keys were `dynamic_<timestamp>` rather than names; nothing read the HashMap except `current_room()`) by eliminating the subsystem rather than patching its keying scheme.

Splits the quantifier's `movement.destination` conflated field into `destination_id` + `destination_name` so authored-room ids (snake_case) and display names (Title Case) stop being conflated.

**BREAKING:** Existing saved games are not migrated. `python build.py --cleanup` is the supported reset path (precedent: ADR-026).

---

## Decisions Locked

All decisions captured as Q1–Q14 in the grilling session. Highlights:

1. `WorldCard.has_map: bool` declares world kind (positive polarity).
2. `WorldCard.starting_room_id` removed; starting location moves to `StartingScenario`.
3. `StartingScenario.starting_location_id: Option<String>` + `starting_location_name: Option<String>` (strict validator matrix).
4. Strict validation: both-Some always rejected regardless of flag. Exactly one of id/name must be set, matching `has_map`.
5. Engine keeps `current_room_id` + `current_room_name` for both world kinds. Mapless world = empty `MapDef` + `current_room_id=None` always.
6. `dynamic_rooms: HashMap<String, Room>` deleted entirely. Off-map drift on mapped worlds writes `current_room_id=None`, `current_room_name=Some(destination_name)`; self-heals when quantifier later emits a valid map id.
7. `NarrativeState.pending_location` deleted. `push_message` reads `state.movement.current_room_name` for the `location_header`. `pending_event` unchanged (separate staging concept).
8. Constructor signature option (B): `GameState::new` / `GameStateBuilder::new` take `current_room_id: Option<String>` + `current_room_name: Option<String>` directly. Caller resolves from scenario before constructing. Tests only need `Some("room1")` + `Some("Room 1")`.
9. Quantifier emits `destination_id` + `destination_name` (no legacy fallback; `build.py --cleanup` wipes old data).
10. `attempt_semantic_walk` matches by id only; misses → drift/mapless with name. Authored `room.name` wins over LLM's `destination_name` when on-map (map authoritative).
11. Synthetic `Room` for quantifier input when `current_room_id=None`: build transient `Room` from `current_room_name` at the call site. Quantifier code unchanged downstream.
12. `build.py --cleanup` wipes DB before first boot — no snapshot back-compat code needed.

---

## Key Changes

### Section 1 — `WorldCard` / `WorldManifest` (`src/model/world.rs`)

Parallel edits to both structs (runtime card + bootstrap manifest):

- **`WorldCard`**: drop `starting_room_id: String` field; add `has_map: bool`.
- **`WorldManifest`**: drop `starting_room_id: String` field; add `has_map: bool`.
- Delete `fn default_starting_room()` helper.
- Add `fn default_has_map_true() -> bool { true }` for serde back-compat (existing worlds default to `has_map=true`).
- Update `Default for WorldCard` impl: drop `starting_room_id` field, set `has_map: true`.
- Update `From<WorldManifest> for WorldCard` mapping: drop `starting_room_id` line, add `has_map: manifest.has_map`.
- Serde attributes: `#[serde(default = "default_has_map_true")] pub has_map: bool` on both structs.

### Section 2 — `StartingScenario` (`src/model/scenario.rs`)

- Remove `starting_room_id: String`.
- Add `starting_location_id: Option<String>` (`#[serde(default)]`).
- Add `starting_location_name: Option<String>` (`#[serde(default)]`).

### Section 3 — `MovementState` (`src/model/state.rs`)

```rust
struct MovementState {
    #[serde(default)]
    pub current_room_id: Option<String>,   // Some for on-map mapped worlds, None for mapless/off-map
    #[serde(default)]
    pub current_room_name: Option<String>, // Both kinds, canonical persistent
    // dynamic_rooms: DELETED
}
```

### Section 4 — `NarrativeState` (`src/model/state.rs`)

- Delete `pending_location: Option<String>` field (line 136).
- Delete from `from_snapshot` reader (line 157).

### Section 4b — `NarrativeState` field refactor blast radius

1. Delete `NarrativeState.pending_location: Option<String>` (`state.rs:136`).
2. Update `NarrativeSnapshot` (`model/state_snapshot.rs:20, 34, 70`): delete `pending_location` field. (No replacement — `current_room_name` lives on `MovementState`, serialized via `movement: state.movement.clone()`.)
3. `push_message` (`state.rs:307-329`) replacement:

   ```rust
   // state.rs:308 — was:
   let location_header = self.narrative.pending_location.take();
   // becomes:
   let location_header = self.movement.current_room_name.clone();  // NOT drained
   ```

   Note: `current_room_name` is canonical persistent (not drained). `pending_event` continues to drain unchanged (Q13 decision).
4. Rewrite `log_movement_completion` (`engine/action_processing.rs:67-72`):
   - On-map (mapped, `id` resolved): `state.movement.current_room_name = Some(current_room.name.clone())`.
   - Off-map/mapless: `state.movement.current_room_name = Some(destination_name.unwrap_or(destination_id.clone()))`.
5. Rewrite `bootstrap/state.rs:25` + `bootstrap/scenario.rs:21`: write `state.movement.current_room_name = Some(room_name)` (not `state.narrative.pending_location`).
6. Update all `current_room_id` String readers to `Option<String>`:
   - `engine/logic.rs:29` (`state.movement.current_room_id = room_id.to_string()` → `Some(room_id.to_string())`)
   - `bootstrap/run.rs:100` (`let room_id = state.movement.current_room_id.clone()` — type changes to `Option<String>`)
   - `application/action_pipeline/phases.rs:63` (`map.get_room_by_id(&state.movement.current_room_id)` → `state.movement.current_room_id.as_deref().and_then(|id| map.get_room_by_id(id))`)
   - `application/query_handlers.rs:173` (cloning `Option<String>`, transparent)
   - `engine/state_diagnostics.rs:32` (see Section 9)
   - Engine tests asserting `current_room_id == "room1"` → `Some("room1".to_string())`: `engine/logic_tests.rs:190,199,222`, `application/context_tests.rs:131,147,154,323`, `storage/backend/snapshots_tests.rs:80,86,90,99,104,109,151,155`, `storage/mappers/state_snapshot_tests.rs:31,32`, `engine/trigger_eval_tests.rs:194,216`, `engine/action_processing_tests.rs:93,98,207,212,219,234,240,245`.
7. Update tests asserting `pending_location == Some(...)` → assert `current_room_name`:
   - `model/state_snapshot_tests.rs:17,38,51,52`
   - `storage/backend/snapshots_tests.rs:164,170`
   - `engine/action_processing_tests.rs:250-257`
   - `bootstrap/run_tests.rs:78` (SQL INSERT for `message_swipes.location_header` stays — that column is per-message, unchanged).

### Section 5 — Bootstrap (`bootstrap/state.rs`, `bootstrap/scenario.rs`, `bootstrap/init_game.rs`)

- Add new helper `resolve_starting_location` (single source of truth — avoids three inline copies):

  ```rust
  // bootstrap/scenario.rs (or new module)
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

- All three bootstrap sites (`bootstrap/state.rs:build_fresh_initial_state`, `bootstrap/scenario.rs:inject_scenario_logs`, `bootstrap/init_game.rs:74,133`) call `resolve_starting_location(...)` instead of inlining room-name resolution.
- Mapped boot: `current_room_id = Some(scenario.starting_location_id)`, `current_room_name = Some(map.get_room_by_id(id).name)`.
- Mapless boot: `current_room_id = None`, `current_room_name = Some(scenario.starting_location_name)`.

### Section 6 — `GameState::new` / `GameStateBuilder::new` signature change (`src/model/state.rs`)

Option (B): constructor takes resolved values directly.

- `GameStateBuilder::new` (`state.rs:198`): signature changes — `starting_room: impl Into<String>` → `current_room_id: Option<String>` + `current_room_name: Option<String>`.
- `GameState::new` (`state.rs:278`): same signature change. Forwards to builder.
- Callers (27+ sites): pass `(Some("room1".to_string()), Some("Room 1".to_string()))` instead of `"room1".to_string()`. Production sites (`bootstrap/state.rs:10`, `bootstrap/init_game.rs:69,128`, `application/context.rs:143,162`) call `resolve_starting_location` first.
- Test sites (`bootstrap/run_tests.rs:186`, `test_support/test_app_builder.rs:266`, `test_support/fixtures.rs:197,208,230,240,264`, `test_support/context_tests.rs:8`, `application/query_handlers_tests.rs:10`, `application/message_editing_tests.rs:174`, `application/game_service_tests.rs:69,83,119,140,161,187`, `application/context_tests.rs:82`, `application/action_pipeline/pipeline_tests.rs:330,395,457`, `engine/logic_tests.rs:85,151`, `engine/trigger_eval_tests.rs:91,334`, `model/state_snapshot_tests.rs:23,58`, `engine/action_processing_tests.rs:440`, `narrative/agents/quantifier/agent_tests.rs:59`) pass explicit `Option<String>` pairs.

### Section 7 — `current_room()` method semantics (`src/model/state.rs:337`)

```rust
pub fn current_room(&self) -> Option<&Room> {
    self.movement.current_room_id.as_deref()
        .and_then(|id| self.map.get_room_by_id(id))
}
```

- Returns `None` for mapless/off-map.
- Does NOT read `current_room_name` (keeps method focused on authored room).
- Six call sites already Option-tolerant — no extra blast-radius changes:
  - `engine/action_processing.rs:69` (`if let Some(...)`)
  - `engine/state_diagnostics.rs:29` (`.is_none()`)
  - `application/query_handlers.rs:99`
  - `application/game_service.rs:143` (`current_room: state.current_room()`)
  - `application/action_pipeline/phases.rs:327` (`?`)
  - `narrative/agents/quantifier/orchestration_tests.rs:230` (`.unwrap()`)

### Section 8 — Engine logic (`src/engine/logic.rs`) + validator (`src/bootstrap/validate.rs`)

**Delete `create_dynamic_room` function entirely** (`logic.rs:31`).

**Rewrite `attempt_semantic_walk`** (`logic.rs:15`):

- Signature: `attempt_semantic_walk(state: &mut GameState, destination_id: Option<&str>, destination_name: Option<&str>) -> Result<String>`.
- If `destination_id` resolves via `find_room_in_world_map` → on-map: `current_room_id = Some(id)`, `current_room_name = Some(room.name)` (authored name wins, ignore LLM's `destination_name`).
- Else (miss or `None`) → drift/mapless: `current_room_id = None`, `current_room_name = Some(destination_name.unwrap_or(destination_id.unwrap_or("unknown")))`.
- Return movement message.

**Delete `action_processing.rs:46-48`** dynamic_room creation block; pass `destination_id` + `destination_name` parsed from quantifier result directly to `attempt_semantic_walk`.

**Validator (`bootstrap/validate.rs`):**

- Delete `if !valid_room_ids.contains(&world.starting_room_id)` check (lines 23-29).
- Delete `fn default_starting_room()` helper if still present.
- Add scenario validator per Q6 strict matrix:

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

- Trigger `room_id` validation (lines 33-44) unchanged — authored triggers reference map rooms only.
- Rewrite 5 `validate_tests.rs` cases:
  - `test_validate_loaded_data_success` (line 9): map id scenario → `starting_location_id: Some("room_a")`.
  - `test_validate_loaded_data_missing_starting_room` (line 29): rewrite as "missing scenario location_id".
  - `test_validate_loaded_data_basic_manifest_succeeds` (line 57): add `has_map: false` + `starting_location_name: Some("...")` scenario.
  - `test_validate_loaded_data_invalid_trigger_room` (line 79): unchanged (trigger validation same).
  - `test_validate_loaded_data_multiple_errors` (line 125): update expected error message to `"location_id"` instead of `"starting_room_id"`.

### Section 9 — State diagnostics (`src/engine/state_diagnostics.rs`)

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

### Section 10 — Narrator prompt builder (`src/narrative/prompt/`)

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
  - `application/action_pipeline/phases.rs:332` (trigger re-narration phase — same pattern; `state.current_room()?` already returns Option)
  - `bootstrap/init_game.rs:161` (initial narration — same)
  - Tests: `context_tests.rs:104`, `assembler_tests.rs:114, 207, 237, 277`.

### Section 11 — Quantifier (`src/narrative/agents/quantifier/`)

**Parser (`parser.rs`):**

- `MovementJson` (lines 17-28): drop `destination: Option<String>` field. Add `destination_id: Option<String>` + `destination_name: Option<String>`.
- `parser.rs:106` extraction: read new fields directly. **No legacy fallback** (`build.py --cleanup` wipes old data).
- `parser.rs:246` `extract_destination` helper: returns `(Option<String>, Option<String>)` instead of `Option<String>`.
- `parser.rs:117, 133` default build sites: produce `(None, None)` instead of `destination: None`.

**Domain (`model/quantifier.rs:75`):**

- `MovementParseResult`: drop `destination: Option<String>`. Add `destination_id: Option<String>` + `destination_name: Option<String>`. Update `Default` impl.

**Prompt template (`data/prompt_presets/quantifier/default.json`):**

- `instructions`: instruct LLM to emit `destination_id` (snake_case stable id reusable on return-travel) AND `destination_name` (human-readable display name).
- `output_format`:

  ```
  {"npcs_in_room": ["id1", "id2"], "movement": {"type": "entering|in|leaving", "destination_id": "room_id_or_null", "destination_name": "display_name_or_null"}}
  ```

- Update all examples to use new field names.

**Synthetic Room for off-map/mapless** (`quantifier/agent.rs:88-94`):

- When `ctx.current_room` is `None`, build transient `Room` from `state.movement.current_room_name`:

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

- Note: synthetic Room is borrowed via `Cow` or refactor to take `Room` by value. TBD at implementation: prefer `&Room` reference via `Cow::Owned` or refactor quantifier entry to accept `Option<&Room>` + `location_name: &str` (cleaner). Decision deferred to first failing test.
- `quantifier/agent.rs:120-123`: emit both new patch fields from `result.movement.destination_id` / `.destination_name`.

**State patch (`model/agent.rs:97-100`):**

- `StatePatch::Scene`: replace `movement_destination: Option<String>` with `movement_destination_id: Option<String>` + `movement_destination_name: Option<String>`.
- `StatePatch::merge` (`model/agent.rs:38-55`): merge both fields independently (first non-None wins for each).
- `game_service.rs:166`: write both back to `result.movement.destination_id` and `.destination_name`.
- `model/agent.rs_temp` deleted (Section 19).

**Orchestration log** (`orchestration.rs:75-77`):

- Update `tracing::debug!` log to print both new fields.

### Section 12 — Trigger eval (`src/engine/trigger_eval.rs`)

- Line 13: `let current_room_id = &state.movement.current_room_id;` → type is `&Option<String>`.
- Line 22 rewrite:

  ```rust
  if Some(room_id) != current_room_id.as_deref() {
      // skip
  }
  ```

- Semantics: authored triggers with `room_id=Some(...)` never fire when `current_room_id=None` (off-map/mapless) — correct.

### Section 13 — Server views

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

### Section 14 — World form (`src/server/worlds_fragment/handlers.rs` + `fragments.rs`)

1. `WorldForm` struct: drop `starting_room_id: Option<String>` field. Add `has_map: Option<bool>` (HTML checkbox unchecked = None → default true).
2. `into_world_card()`:
   - Delete `starting_room_id: self.starting_room_id.unwrap_or_else(|| "start".to_string())` line.
   - Add `has_map: self.has_map.unwrap_or(true)`.
3. `render_world_edit_form` (server `fragments` module): render `has_map` checkbox, drop `starting_room_id` input field.
4. Per-scenario HTML form fields: replace single `starting_room_id` input with `starting_location_id` + `starting_location_name` inputs (both visible; validator enforces exactly one set).
5. Existing mapped worlds (redmist/test) must still save correctly via the form — `has_map` defaults true, scenarios carry new optional fields.

### Section 16 — Storage migration v14 (`src/storage/db.rs`)

**Schema-only migration** (matches ADR-026 precedent). No snapshot back-compat code.

```rust
if version < 14 {
    let exec = |sql: &str| {
        conn.execute(sql, [])
            .map_err(|e| crate::error::EngineError::Config(format!("Migration failed: {e}")))
    };

    // SQLite 3.35+ supports DROP COLUMN (rusqlite 0.34 bundled = 3.46+)
    exec("ALTER TABLE worlds DROP COLUMN starting_room_id")?;
    exec("ALTER TABLE worlds ADD COLUMN has_map INTEGER NOT NULL DEFAULT 1")?;
    // Existing world rows default has_map=1 (true) — matches prior behavior (all worlds had maps)

    conn.pragma_update(None, "user_version", 14).map_err(|e| {
        crate::error::EngineError::Config(format!("Failed to set user_version: {e}"))
    })?;
}
```

- World JSON reload from disk provides scenario field migration automatically (`data/worlds/redmist_estate/world.json` + `test/world.json` updated per Section 17 with new `has_map` + scenario fields).
- Existing games abandoned — `python build.py --cleanup` wipes target dir + DBs. New game auto-creates with CLI persona per existing `resolve_game_id` flow.
- `GameState::from_snapshot` gets NO special back-compat code. Old snapshots are not loaded.

### Section 17 — Data files

- `data/worlds/redmist_estate/world.json`: add `"has_map": true`, move top-level `starting_room_id` value (`"front_gates"`) into each scenario as `starting_location_id` (string preserved, `"front_gates"`) + leave `starting_location_name: null`.
- `data/worlds/test/world.json`: same pattern.
- `data/schemas/world.schema.json`: add `has_map`, remove top-level `starting_room_id`, add optional `starting_location_id` / `starting_location_name` to scenario schema.
- `data/prompt_presets/quantifier/default.json`: per Section 11.
- **No new mapless seed world created.** Mapless behavior covered by unit + integration tests with in-memory `WorldCard` fixtures (`WorldCard { has_map: false, scenarios: vec![StartingScenario { starting_location_name: Some("..."), ... }], ... }`).

### Section 18 — Implementation order

Per chronicler-dev-workflow skill: **Architecture First**, **Tests First**.

1. **Plan doc** — this file (`docs/plans/mapless-worlds-plan.md`). [DONE at write]
2. **ADR-027** — `docs/adr/adr-027-mapless-worlds.md`.
3. **Architecture** — `docs/architecture/system.md` updates (MovementState, StartingScenario, WorldCard sections).
4. **Other docs** — per Section 19 list.
5. **Delete `chronicler_engine/src/model/agent.rs_temp`** (stale temp file noticed during plan review — line 111 duplicate of `agent.rs`).
6. **Model layer tests first** — write `MovementState` serde test (Option round-trip), `resolve_starting_location` test (all branches), `WorldCard`/`WorldManifest` serde test (has_map default true), `StartingScenario` validator matrix test (all 6 matrix cells).
7. **Model layer code** — `world.rs`, `scenario.rs`, `state.rs`, `state_snapshot.rs`, `agent.rs` (`StatePatch::Scene` split), `quantifier.rs` (`MovementParseResult`).
8. **Bootstrap tests first** — `validate_tests.rs` rewrites (5 cases), `resolve_starting_location` integration.
9. **Bootstrap code** — `bootstrap/scenario.rs` (helper), `state.rs`, `init_game.rs`, `validate.rs`.
10. **Engine tests first** — `attempt_semantic_walk` tests (on-map hit, on-map miss → drift, mapless always drift), `trigger_eval.rs` type-check test, `state_diagnostics.rs` INV-ROOM rewrite test.
11. **Engine code** — `engine/logic.rs` (delete `create_dynamic_room`, rewrite `attempt_semantic_walk`), `engine/trigger_eval.rs`, `engine/state_diagnostics.rs`, `engine/action_processing.rs` (delete dynamic_room creation block, rewrite `log_movement_completion`).
12. **Narrator tests first** — `make_prompt_context` Option signature test, `render_game_state_layer` on-map vs off-map branch test.
13. **Narrator code** — `prompt/types.rs`, `prompt/assembler.rs`, `prompt/context.rs`, `phases.rs:73,332`, `bootstrap/init_game.rs:161`.
14. **Quantifier tests first** — parser `destination_id`/`destination_name` split test, no-movement test, synthetic-Room path test (`current_room=None`), `StatePatch::merge` two-field test.
15. **Quantifier code** — `parser.rs`, `model/quantifier.rs`, `agent.rs` (synthetic Room fallback), `orchestration.rs` (tracing log), `game_service.rs:166` (patch application), `data/prompt_presets/quantifier/default.json` update.
16. **Server form tests first** — `into_world_card` form-to-card test (has_map default true, scenario location id/name).
17. **Server code** — `worlds_fragment/handlers.rs` + `fragments.rs` (form fields), `application_service.rs` `DebugStateView`, `query_handlers.rs:165,172`, `server/debug.rs`.
18. **Storage migration v14** — `storage/db.rs` migration block + migration test (schema diff assertion).
19. **Data files** — `data/worlds/redmist_estate/world.json`, `data/worlds/test/world.json`, `data/schemas/world.schema.json`, `data/prompt_presets/quantifier/default.json`.
20. **Test sweep** — update all `current_room_id == "room1"` assertions to `Some("room1".to_string())`; update all `pending_location` assertions to `current_room_name`. (~20 test files.)
21. **Validate** — `python build.py --cleanup` (fresh DB + full validation: fmt + clippy `-D warnings` + nextest).
22. **Archive plan** — move `docs/plans/mapless-worlds-plan.md` → `docs/plans/archived/`.

### Section 19 — Documentation (`docs/`)

1. `docs/plans/mapless-worlds-plan.md` — this file.
2. `docs/adr/adr-027-mapless-worlds.md` — Problem (can't author worlds without maps; dynamic_rooms subsystem overhead), Decision (mapless via empty MapDef + freeform `current_room_name`; delete dynamic_rooms; split quantifier `destination_id`/`destination_name`), Alternatives (Q4-β both-kinds drop `current_room_id` — rejected, mapped worlds need it; Q9-A key by name — rejected, synonym drift; Q9-B normalized name — rejected, same engine limit), Consequences (BREAKING: existing saved games must be reset with `build.py --cleanup`), Architecture Impact (Modified Modules table), Verification (per Section 18 test plan), Related (ADR-026 migration precedent, ADR-008 snapshot persistence).
3. `docs/architecture/system.md` — MovementState section, StartingScenario section, WorldCard section.
4. `docs/system/dynamic_rooms.md` — **DELETE entirely** (subsystem removed).
5. `docs/system/navigation.md` — update: map lookup, off-map drift to `current_room_name`, scenario-driven starting location, no dynamic rooms, `attempt_semantic_walk` new signature.
6. `docs/system/startup.md` — bootstrap section: scenario resolves starting location via `resolve_starting_location` helper; both world kinds.
7. `docs/system/worlds.md` — `WorldCard.has_map` field, `StartingScenario` optional id/name, strict validator matrix.
8. `docs/system/triggers.md` — note: authored triggers with `room_id` never fire in mapless worlds or when mapped-world drift is active.
9. `docs/architecture/invariants.md` — INV-ROOM description: `current_room_id=None` valid for mapless/off-map; `current_room_name` must always be `Some` after bootstrap.
10. `docs/reference/data_schemas.md` — world JSON schema: add `has_map`, drop top-level `starting_room_id`, add scenario `starting_location_id`/`starting_location_name`.
11. `docs/reference/data_layer.md` — mention v14 migration + `build.py --cleanup` requirement.
12. `docs/system/agent_system.md` — quantifier emits `destination_id` + `destination_name` split.
13. `docs/system/game_flow.md` — update movement pipeline section (new `attempt_semantic_walk` signature, `current_room_name` parallel write).
14. `docs/system/prompt_system.md` — if documents GameState layer, note Option-based room context.
15. `docs/CHANGELOG.md` — under "Unreleased":
    - **BREAKING:** Removed `dynamic_rooms` subsystem; existing saved games must be reset with `python build.py --cleanup`.
    - Added `WorldCard.has_map` boolean + `StartingScenario.starting_location_id` / `starting_location_name`. Worlds without maps now supported.
    - Split quantifier `movement.destination` into `destination_id` + `destination_name`.
    - Storage migration v14: worlds table gains `has_map`, drops `starting_room_id`.
16. `docs/README.md` auto-index — regenerated by build process (delete `dynamic_rooms.md` from index).
17. ADR-027 Consequences section must explicitly note: **breaks in-progress games**. Existing saves on upgrading DBs will attempt to load with old `current_room_id` String → serde → `Some("legacy_id")`, `current_room_name=None` → off-map drift with no name. `build.py --cleanup` is the supported reset path.

### Section 20 — Test plan

**Unit tests:**

- `MovementState` serde round-trip (Option fields, no `dynamic_rooms` in serialized output).
- `resolve_starting_location` all branches: mapped id-found, mapped id-not-found, mapless name-set, mapless name-empty.
- `WorldCard` / `WorldManifest` serde back-compat: missing `has_map` defaults to `true`.
- `StartingScenario` validator matrix — all 6 cells (has_map × id-set × name-set).
- `current_room()` returns `None` when `current_room_id=None`, `Some(&Room)` when id resolves.
- `attempt_semantic_walk` on-map hit, on-map miss → drift, mapless always drift.
- `push_message` reads `current_room_name` without draining.
- INV-ROOM diagnostics: `Some(id)` missing from map → error; `None` → ok (if current_room_name Some); `current_room_name=None` → error.
- `make_prompt_context` Option signature; `render_game_state_layer` on-map vs off-map branches.
- Quantifier parser: `destination_id`/`destination_name` parse, no-movement, malformed → None.
- Synthetic-Room path: `ctx.current_room=None` → transient Room built from `current_room_name`, quantifier succeeds.
- `StatePatch::merge`: two-field independent merge, first non-None wins per field.
- `trigger_eval`: authored triggers never fire when `current_room_id=None`.
- `into_world_card` form parsing: `has_map` default true, scenario location id/name carry-through.

**Integration tests:**

- Full mapped-world turn: player walks into authored room → `current_room_id=Some(id)`, `current_room_name=Some(room.name)`, narrator receives authored description.
- Full mapped-world drift: quantifier emits `destination_id="hidden_cellar"` (not in map) → `current_room_id=None`, `current_room_name=Some("Hidden Cellar")` (from `destination_name`); narrator receives only name; next turn quantifier emits `destination_id="tavern"` (in map) → self-heals to `Some("tavern")` + `Some("Tavern")`.
- Full mapless-world turn: bootstrap seeds `current_room_id=None`, `current_room_name=Some("...")`; every movement keeps `current_room_id=None`, updates `current_room_name`; narrator never receives authored description (correct — there isn't one).
- Storage migration v14: fresh DB schema has `has_map` column, no `starting_room_id`.

**E2E (browser, only if form changes break):**

- World edit form: create new mapped world with `has_map=true`, scenario `starting_location_id` set → save → verify DB row.
- World edit form: create new mapless world with `has_map=false`, scenario `starting_location_name` set → save → verify DB row.
- (No CSS changes — existing templates handle new form fields if input name attributes match.)

---

## Assumption Notes (Deviation Protocol)

Any scope discovered mid-implementation triggers the STOP-and-report protocol per project `AGENTS.md` "Plan Adherence" rule:

- Discovered bug in unrelated code → STOP, surface to user, offer A) fix now (deviate) or B) add to plan and continue.
- Mid-implementation refactor opportunity → STOP, surface, wait for direction.
- Plan gap discovered → STOP, surface, wait.
- "Better" approach than agreed → STOP, surface, wait.

User has 61% plan-failure rate caused by agent drift. Do not be helpful at the expense of predictability.

---

## Plan Review Amendments Log

This plan was reviewed against `improve-ai-plan` skill. 22 acceptance points applied during review:

1. Section 10 expanded — `PromptContext`, `LayerRenderer`, `render_game_state_layer`, all 4 test call sites listed.
2. Section 7 — `current_room()` semantics + 6 call sites classified (all Option-tolerant).
3. Section 13 — `DebugStateView` field changes + `query_handlers.rs:165,172` builder + `server/debug.rs` struct.
4. Section 11 — quantifier off-map/mapless hard error gap; synthetic `Room` from `current_room_name` amendment.
5. Section 11 — `MovementParseResult.destination` split added.
6. Section 12 — `trigger_eval.rs:22` type fix `Some(room_id) != current_room_id.as_deref()`.
7. Section 8 — `create_dynamic_room` deletion + `attempt_semantic_walk` signature rewrite, `action_processing.rs:46-48` deletion, validator code + 5 `validate_tests.rs` rewrites.
8. Section 4b (NEW) — `NarrativeState` field refactor blast radius: `state_snapshot.rs`, `log_movement_completion`, bootstrap writes, all `current_room_id` String→Option readers (~20 test sites).
9. Section 5 — `resolve_starting_location` helper (single source of truth, three bootstrap call sites).
10. Section 16 — v14 schema-only migration; `build.py --cleanup` required reset path; no snapshot back-compat code.
11. Section 1 — `WorldManifest` parallel edits + `From` impl + `Default` impl + helpers.
12. Section 11 — quantifier prompt template lives in `data/prompt_presets/quantifier/default.json` (DB preset, not source); no legacy fallback (build.py --cleanup wipes old data).
13. Section 11 — `StatePatch::Scene` split into `movement_destination_id` + `movement_destination_name`; `StatePatch::merge` per-field first-non-None; `game_service.rs:166` patch application; `model/agent.rs_temp` deletion.
14. Section 14 — WorldForm `has_map` checkbox + per-scenario `starting_location_id`/`starting_location_name` inputs.
15. Section 8 — `validate.rs` scenario matrix code + `validate_tests.rs` cases.

---

## NOT in scope

1. NPC spawn/encounter system in mapless worlds — quantifier gets synthetic `&Room`; NPC detection uses existing `state.npcs` values path. No new NPC routing.
2. Return-travel bug for mapped-world off-map drift — partially addressed by `destination_id`/`destination_name` split (LLM can reuse stable id), but no engine-side name-normalization. Off-map drift locations still rely on conversation history for re-identification.
3. Mapless seed world JSON — no `data/worlds/mapless_demo/`. Mapless behavior covered by unit + integration test fixtures.
4. Migration of existing saved games — `build.py --cleanup` required. In-progress games abandoned (ADR-026 precedent).
5. `pending_event` staging semantics — unchanged per Q13 decision.
6. Trigger firing for mapless worlds — authored triggers with `room_id=Some(...)` never fire when `current_room_id=None`. Correct behavior, no change.
7. CSS / visual UI changes — none. Plan section 14 (HTML form field changes) is structural; existing templates handle rendering.

## What already exists

1. `AgentContext.current_room: Option<&'a Room>` (`model/agent.rs:115`) — already Option-typed. No struct change.
2. `current_room()` method returning `Option<&Room>` (`model/state.rs:337`) — already Option; refactor simplifies body (drop `dynamic_rooms` fallback), signature unchanged.
3. Bootstrap migration precedent — ADR-026 for schema-only migrations + `build.py --cleanup` reset path.
4. Seed-driven preset loading — `data/prompt_presets/quantifier/default.json` already source of truth.
5. `Trigger.room_id: Option<String>` (`model/trigger.rs:32`) — already Option.
6. `StartingScenario` deserializes from `scenarios_json` in WorldForm — transparent to form handler; only HTML inputs change.
7. `build.py --cleanup` — supported reset path, wipes + re-seeds from `data/`.
8. Quantifier post-generation patch pipeline (`StatePatch::Scene` + `StatePatch::merge`) — already handles Option merging via first-non-None. Splitting into two Option fields reuses same pattern.

## Failure modes

1. **Bootstrap: scenario missing for mapless world.** Validator catches: both `starting_location_id` and `starting_location_name` = None → error names the scenario. World load fails loudly.
2. **Bootstrap: mapless world has `starting_location_id` set.** Validator catches: `has_map=false` but `id=Some(...)` → error. Strict matrix.
3. **Mapped world drift: id missing from map.** `attempt_semantic_walk` falls through to drift: `current_room_id=None`, `current_room_name=Some(destination_name)`. Self-heals on next turn if LLM emits valid map id. No hard error.
4. **Narrator receives `current_room_id=None` + `current_room_name=None`.** Cannot happen in normal flow — bootstrap seeds at least `current_room_name`. If it does (corrupt state): `render_game_state_layer` emits empty `<GameState>Current Location: </GameState>`. Narrator tolerates empty field. Guard test added.
5. **Quantifier LLM omits `destination_id`/`destination_name`.** Parser produces None for both. Treated as no-movement. Existing behavior for malformed quantifier responses — no regression.
6. **`current_room_id` orphaned (Some(id) where id ∉ map).** Caught by `assert_room_exists` (INV-ROOM, diagnostics feature only). In production (no diagnostics): `current_room()` returns None, narrator falls to off-map path. Self-heals on next movement.
7. **Old-format quantifier JSON `{"destination": "kitchen"}` reaches parser.** No legacy fallback: parse fails → `MovementParseResult.movement_type=None`, treated as no-movement. Acceptable because `build.py --cleanup` wipes old LLM logs.
8. **`build.py --cleanup` not run before first boot after upgrade.** Old snapshots load → `current_room_id="front_gates"` (legacy String) → serde coerced to `Some("front_gates")`; `current_room_name=None` (new field default). Map lookup succeeds for mapped worlds → narrator on-map path works. `current_room_name` self-heals on next movement via `log_movement_completion`. Only breaks if player never moves and never re-saves. Mitigated by required reset directive.
9. **Live game with player in a dynamic room at upgrade time.** Per Section 16: `build.py --cleanup` required. If not run: `current_room_id="dynamic_<ts>"` loads as `Some("dynamic_<ts>")`, map lookup misses, `current_room_name=None`, treated as off-map drift with no name. Narrator may render empty location. Supported reset path is `build.py --cleanup`.

## Unresolved decisions

None. Plan is decision-complete per Plan-mode rules. All 27 review issues either accepted as amendments or resolved via codebase truth.
