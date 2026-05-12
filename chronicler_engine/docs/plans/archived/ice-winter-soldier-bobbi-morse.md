# Implementation Plan: Redmist Estate Data Overhaul

## Overview

Remove hardcoded NPCs from map rooms entirely, replace broken SillyTavern world lore with objective world-building, add cross-character relationships, expand the estate map, and support scenario-driven initial NPC placement.

Key principle: Remove the `npcs` field from `Room` struct and all associated engine code. The single source of truth for "who is present" becomes `scene.npcs_in_area`, which is initialised from `StartingScenario.npcs` at bootstrap and updated by the quantifier during play.

---

## Architecture Decisions

1. `Room.npcs` is deleted from the `Room` struct. This touches map loading, validation, prompt building, quantifier fallback, and test fixtures.
2. `StartingScenario.npcs` becomes the only way to pre-place NPCs at game start.
3. `get_static_npcs` is removed. The engine no longer has any concept of "static" room NPCs.
4. Relationships are static only. No engine mutation yet. The LLM reads them as context.
5. AI-generated text is acceptable for room descriptions, relationship blurbs, and world lore.
6. Both `test` and `redmist` worlds are updated in parallel.

---

## Task List

### Phase 1: Engine - Add Scenario NPCs

#### Task 1: Add `npcs` to `StartingScenario`

Description: Add `npcs: Vec<String>` to the `StartingScenario` Rust struct with `#[serde(default)]`. Derive `Default` so manual struct construction does not break.

Files:
- `src/model/scenario.rs` - add field, derive `Default`
- `data/schemas/world.schema.json` - add `npcs` to scenario items
- All manual `StartingScenario { ... }` constructions in tests - add `npcs: vec![]` or use `..Default::default()`

Acceptance:
- `cargo test` compiles (no broken struct literals)
- `serde_json::from_str` on old scenario JSON without `npcs` still works

Verification: `cargo test`

---

### Phase 2: Engine - Remove `npcs` from `Room`

#### Task 2: Remove `npcs` from `Room` struct and schema

Files:
- `src/model/map.rs` - delete `pub npcs: Vec<String>` and `#[serde(default)]` attribute above it
- `data/schemas/map.schema.json` - delete the `npcs` property from room items

Acceptance:
- `cargo test` compiles (any code referencing `room.npcs` will now fail - that is expected; we fix in next tasks)

---

#### Task 3: Remove all engine code that reads `room.npcs`

Description: Find every place that accesses `room.npcs` or `get_static_npcs` and replace with `scene.npcs_in_area` or remove entirely.

Files and exact changes:

1. `src/bootstrap/run.rs` (lines 40-45 and ~130)
   - Delete: `let room_npc_ids = current_room.npcs.clone(); let nearby_npcs = get_static_npcs(...)`
   - Replace with: `let nearby_npcs = state.scene.npcs_in_area.clone();`
   - In the spawned fallback block (~line 106), do the same: use `state.scene.npcs_in_area` instead of `room.npcs`

2. `src/bootstrap/validate.rs` (lines 29-42)
   - Delete the entire loop that validates `room.npcs` against loaded NPCs

3. `src/model/state.rs` (`GameState::new`, lines 216-230)
   - Delete the nested loop that reads `room.npcs` and initialises `character_state` / `times_met`
   - `character_state` initialisation moves to `run.rs` (Task 4)

4. `src/engine/game_service/actions.rs` (lines 171-174 and 220)
   - Lines 171-174: delete `room_npc_ids` and `nearby_npcs` from `get_current_room`; use `state.scene.npcs_in_area.clone()` as `nearby_npcs`
   - Line 220: `default_quantifier_result(&room.npcs)` -> `default_quantifier_result(&[])` (empty fallback)

5. `src/engine/action_processing.rs` (`get_static_npcs`, lines 45-50)
   - Delete the `get_static_npcs` function entirely
   - Remove its import from `src/engine/game_service/actions.rs`

6. `src/server/fragments/renderers.rs` (lines 73-79)
   - Delete the `else` fallback branch that reads `room.npcs`
   - The `if !state.scene.npcs_in_area.is_empty()` branch becomes the only path; if empty, `npc_data` is simply `vec![]`

7. `src/narrative/agents/quantifier/agent.rs` (lines 57-65)
   - Delete `room_npc_ids` read from `get_current_room(state)`
   - Pass `&[]` (empty slice) to `determine_npcs_in_room` as the `room_npc_ids` argument
   - The `previous_room_npcs` (= `state.scene.npcs_in_area`) remains

8. `src/narrative/agents/quantifier/core.rs` (`determine_npcs_in_room`)
   - Keep the function signature for now (to minimise change surface), but `room_npc_ids` will always receive `&[]` from the agent
   - The fallback `static_npc_result` call on line 167-175 will receive an empty list, which is correct

Acceptance:
- `cargo test` compiles
- No remaining references to `room.npcs` or `get_static_npcs` in `src/`

Verification: `cargo test` + `grep -r "room\.npcs" src/` should return nothing

---

#### Task 4: Bootstrap initialisation from scenario NPCs

Description: After `GameState::new` and `inject_scenario_logs`, read `manifest.default_scenario().npcs` and:
1. For each NPC id found in `state.npcs`, set `character_state.npcs[id].times_met = 1` and `currently_meeting = true`
2. Push the NPC card into `state.scene.npcs_in_area`

Apply the same logic in the fallback `GameState::new` path inside the spawned task (`run.rs` ~line 106).

Files:
- `src/bootstrap/run.rs` - add post-initialisation logic in both `GameState` creation sites

Acceptance:
- Starting the Redmist world with the bodyguard scenario results in `character_state.npcs["carla"].times_met == 1`
- `scene.npcs_in_area` contains Carla's card at startup
- Starting the `test` world (no scenario or empty scenario) continues to work normally

Verification: `cargo test` + manual engine startup check

---

#### Task 5: Update all test fixtures and tests

Description: Every test that constructs a `Room` struct or references `room.npcs` / `get_static_npcs` must be updated.

Known locations:
- `src/bootstrap_tests.rs` - ~15 `Room` constructions with `npcs: vec![...]` or `npcs: vec![]`
- `src/test_support/fixtures.rs` - `Room` constructions in `make_test_map()`, `with_npcs()` helper (this helper can be deleted or repurposed to mutate `state.scene.npcs_in_area` directly)
- `src/engine/action_processing_tests.rs` - `room.npcs.push(...)` on line 417, `get_static_npcs` tests (lines 271-284)
- `src/engine/logic_tests.rs` - `npcs: vec![]` in `Room` constructions
- `src/narrative/agents/quantifier/test_support.rs` - `npcs: vec!["gabriella"]` in room construction
- `src/narrative/prompt/builder_tests.rs` - any `Room` constructions (check for `npcs` field)

Strategy for `with_npcs` helper in fixtures:
Instead of pushing NPC ids into `room.npcs`, the helper should directly populate `state.scene.npcs_in_area` and `state.character_state.npcs`.

Acceptance:
- `cargo test` compiles and passes
- No remaining `npcs:` field in any `Room { ... }` literal in test code

Verification: `cargo test` + `grep -r "npcs:" src/ | grep -i "room"` should return nothing relevant

---

### Checkpoint 1: Engine Complete
- [ ] `cargo test` passes
- [ ] `python build.py` passes
- [ ] `grep -r "room\.npcs" src/` returns nothing
- [ ] Engine starts Redmist world, Carla is in `npcs_in_area`

---

### Phase 3: Data - Both Worlds

#### Task 6: Update `test` world data

Files:
- `data/worlds/test/map.json` - delete all `"npcs"` keys from every room
- `data/worlds/test/world.json` - add `"npcs": ["bartender", "shopkeeper"]` to the `test_intro` scenario (these NPCs were previously hardcoded in the `start` room)

Acceptance:
- Test world loads and validates cleanly
- Bartender and shopkeeper appear in `npcs_in_area` at startup

---

#### Task 7: Rewrite Redmist `world.json`

The new file contains:
- `id`: "redmist_estate", `name`: "Redmist Estate"
- A proper `description` paragraph about Islaport and the estate
- 6 objective `global_rules` (no `{{char}}`)
- `starting_room_id`: "front_gates"
- `scenarios` array with one scenario: `bodyguard_intro`
  - `starting_room_id`: "front_gates"
  - `text`: the existing bodyguard intro text
  - `npcs`: ["carla"]

Acceptance:
- No `{{char}}` references
- 6 objective global rules
- Scenario has `"npcs": ["carla"]`

---

#### Task 8: Expand Redmist `map.json`

New layout: 11 rooms, **no `npcs` key anywhere**.

Rooms:
1. `front_gates` - exits: north -> courtyard
2. `courtyard` - exits: south -> front_gates, north -> entrance_hall, west -> gardens
3. `gardens` - exits: east -> courtyard
4. `entrance_hall` - exits: south -> courtyard, west -> kitchen, east -> financial_office, north -> library
5. `library` - exits: south -> entrance_hall, north -> master_quarters
6. `master_quarters` - exits: south -> library
7. `kitchen` - exits: east -> entrance_hall, north -> dining_room, west -> staff_quarters
8. `dining_room` - exits: south -> kitchen
9. `staff_quarters` - exits: east -> kitchen
10. `financial_office` - exits: west -> entrance_hall, north -> guest_wing
11. `guest_wing` - exits: south -> financial_office

Key changes from current map:
- No `npcs` key in any room (field removed entirely)
- Added courtyard, gardens, library, dining_room, staff_quarters, guest_wing
- front_gates now connects north to courtyard, not directly to entrance_hall
- master_quarters now connects south to library (was entrance_hall)

Acceptance:
- 11 rooms total
- No `npcs` keys anywhere in the file
- All room exits are bidirectional and valid
- `starting_room_id` ("front_gates") exists in the map

---

#### Task 9: Update character cards - descriptions and relationships

Changes for ALL characters:
1. Remove the redundant opening sentence: "[Name] lives in the city of Islaport..."
2. Add `"relationships": [...]` array (minimum 2 entries each)
3. Keep everything else intact (personality, scenario, example_dialogue, triggers, images)

Carla relationships:
- with: "gabriella", dynamic: "deep suspicion", static: "Carla considers Gabriella a security risk and resents how she treated Bernard. She watches Gabriella's movements closely."
- with: "jezebel", dynamic: "professional respect", static: "Carla and Jezebel worked together to protect Bernard's interests. They trust each other's competence."
- with: "louise", dynamic: "distant tolerance", static: "Carla finds Louise's enthusiasm slightly exhausting but harmless. She appreciates the meals Louise prepares."

Gabriella relationships:
- with: "jezebel", dynamic: "bitter rivalry", static: "Jezebel made no secret of her disdain for Gabriella during Bernard's life. Gabriella considers her a sanctimonious busybody."
- with: "carla", dynamic: "dismissive hostility", static: "Gabriella resents Carla's presence and the implication that she needs to be watched. She considers Carla a glorified hired thug."
- with: "lisette", dynamic: "casual contempt", static: "Gabriella has berated Lisette more than once for minor mistakes. She sees the staff as beneath her."

Jezebel relationships:
- with: "gabriella", dynamic: "open contempt", static: "Jezebel believes Gabriella married Bernard for money and emotionally manipulated him. She is determined to prevent Gabriella from doing the same to Julian."
- with: "carla", dynamic: "allied trust", static: "Jezebel and Carla share a common goal: protecting Bernard's legacy and Julian's wellbeing. They coordinate when possible."
- with: "lisette", dynamic: "quiet appreciation", static: "Jezebel admires Lisette's dedication and discretion. She has occasionally confided in her about estate matters."

Lisette relationships:
- with: "gabriella", dynamic: "quiet wariness", static: "Lisette has seen Gabriella's cruelty to staff over the years. She stays out of her way but keeps her distance."
- with: "louise", dynamic: "friendly acquaintance", static: "Lisette and Louise share the servant quarters and occasionally talk during breaks. Lisette appreciates Louise's energy."
- with: "jezebel", dynamic: "respectful familiarity", static: "Lisette has served Jezebel tea many times and finds her kind. She trusts Jezebel's judgement about the household."

Louise relationships:
- with: "lisette", dynamic: "warm friendship", static: "Louise appreciates Lisette's calm presence in the chaotic household. They often share meals in the staff kitchen."
- with: "gabriella", dynamic: "nervous avoidance", static: "Louise once served Gabriella a cold meal and was berated for an hour. She now panics whenever Gabriella enters the kitchen."
- with: "carla", dynamic: "friendly respect", static: "Louise is slightly intimidated by Carla but admires her strength. She always makes sure Carla's meals are extra hearty."

Acceptance:
- All 5 characters have `relationships` arrays with at least 2 entries
- No `description` starts with "lives in Islaport"
- `cargo test` still passes (triggers and character parsing tests)

---

#### Task 10: Update `character.schema.json`

Add `relationships` to the schema as an array of objects with:
- `with` (string, required)
- `static` (string, required)
- `dynamic` (string, optional)

Acceptance:
- All Redmist character JSON validates against the updated schema

---

### Checkpoint 2: Data Complete
- [ ] `cargo test` passes
- [ ] `python build.py` passes
- [ ] Engine starts Redmist world without validation errors
- [ ] Engine starts test world without validation errors
- [ ] `grep -r '"npcs"' data/worlds/redmist_estate/map.json` returns nothing
- [ ] `grep -r '"npcs"' data/worlds/test/map.json` returns nothing

---

## Risks and Mitigations

| Risk | Impact | Mitigation |
|------|--------|------------|
| Changing front_gates->courtyard->entrance_hall path breaks existing bootstrap tests | High | Run `cargo test` after map change; fix any hardcoded room-id assumptions in tests |
| `Room` struct change breaks many test fixtures | Medium | Compiler catches every broken `Room { ... }` literal; fix systematically (Task 5) |
| Removing `room.npcs` fallback in renderers causes empty sidebar on first load | Low | Scenario-driven init ensures `npcs_in_area` is never empty at startup; quantifier keeps it updated |
| `determine_npcs_in_room` with empty `room_npc_ids` behaves differently | Low | The function already falls back to `previous_room_npcs` (`npcs_in_area`); empty static list just means no extra fallback |
| Character `description` changes break prompt length budgets | Low | Descriptions are slightly shorter after removing Islaport preamble |

## Open Questions

None. User has clarified:
- Remove `npcs` from `Room` entirely
- Backwards compatibility not required
- AI generation acceptable for lore/descriptions
- Only Carla at front gates initially
- Gabriella trigger unchanged (entrance hall)
- No new room images
- I will handle the creative writing
