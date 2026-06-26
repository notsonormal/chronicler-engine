# Subplan B: Quantifier `destination` field split

**Parent Plan:** [mapless-worlds-plan.md](./mapless-worlds-plan.md)
**Status:** Decision-complete
**Date:** 2026-06-25
**Depends on:** none (independent of Subplan A)
**Blocks:** Subplan C (atomic mapless enablement)

---

## Summary

Split the quantifier's conflated `destination` field into `destination_id` + `destination_name`. Today the LLM emits a single string used as both map-lookup key AND display name — snake_case ids ("security_office") become ugly display text, or Title Case names ("Security Office") miss map lookup forcing drift even on mapped worlds. Split fixes both.

Engine-side consumption: `attempt_semantic_walk` matches by id only; misses fall to drift path (still using existing `dynamic_rooms` subsystem — full deletion lands in Subplan C). Authored `room.name` wins over LLM's `destination_name` when on-map.

**Not breaking to saved games** — `build.py --cleanup` still recommended (old LLM logs in `llm_messages` table contain old-format JSON but are not parsed at runtime; they're raw text). New games work end-to-end with new prompt + parser.

---

## Key Changes

### 1. Prompt template (`data/prompt_presets/quantifier/default.json`)

Update `instructions` + `output_format` + `examples`:

- `instructions`: instruct LLM to emit `destination_id` (snake_case stable identifier the LLM can reuse on return-travel) AND `destination_name` (human-readable display name).
- `output_format`:

  ```
  Respond ONLY with a JSON object in this exact format:
  {"npcs_in_room": ["id1", "id2"], "movement": {"type": "entering|in|leaving", "destination_id": "room_id_or_null", "destination_name": "display_name_or_null"}}
  ```

- Examples updated:
  - `"You walk through the door into the kitchen."` → `{"movement": {"type": "entering", "destination_id": "kitchen", "destination_name": "Kitchen"}}`
  - `"The foyer felt claustrophobic."` → `{"movement": {"type": "entering", "destination_id": "entrance_hall", "destination_name": "Entrance Hall"}}`
  - No-movement examples: `{"movement": {"type": null, "destination_id": null, "destination_name": null}}`

### 2. Parser (`src/narrative/agents/quantifier/parser.rs`)

- `MovementJson` struct (lines 17-28): drop `destination: Option<String>` field. Add:

  ```rust
  #[serde(default)]
  pub destination_id: Option<String>,
  #[serde(default)]
  pub destination_name: Option<String>,
  ```

- **No legacy fallback** — `build.py --cleanup` wipes old data.
- `parser.rs:106` extraction (`extract_movement_result` / equivalent): produce `destination_id` + `destination_name` from the new JSON fields directly.
- `parser.rs:117, 133` default construction: `(None, None)` instead of `destination: None`.
- `parser.rs:246` `extract_destination` (currently returns `Option<String>` from `all_rooms` name match): becomes `extract_destination_pair` returning `(Option<String>, Option<String>)` — `(Some(room.id.clone()), Some(room.name.clone()))` when match found, `(None, None)` otherwise. The current heuristic of matching by name substring stays (the LLM may omit `destination_id` when none matches; `extract_destination_pair` is the fallback for ambiguity).

### 3. Domain (`src/model/quantifier.rs:75`)

- `MovementParseResult`: drop `destination: Option<String>`. Add:

  ```rust
  pub destination_id: Option<String>,
  pub destination_name: Option<String>,
  ```

- Update `Default` impl.

### 4. State patch (`src/model/agent.rs:97-100`)

- `StatePatch::Scene`: replace `movement_destination: Option<String>` with:

  ```rust
  movement_destination_id: Option<String>,
  movement_destination_name: Option<String>,
  ```

- `StatePatch::merge` (lines 38-55): merge both fields independently, first non-None wins per field. Update match arms to construct new variant.
- Delete `chronicler_engine/src/model/agent.rs_temp` (stale temp file, line 111 duplicate of `agent.rs`). This deletion is a prerequisite — the `_temp` file already references the old single-`movement_destination` field and would drift further.

### 5. Quantifier agent (`src/narrative/agents/quantifier/agent.rs`)

- Lines 120-123: emit new patch fields:

  ```rust
  Ok(AgentResult::StatePatch(StatePatch::Scene {
      npc_ids: result.npcs.npc_ids,
      movement_destination_id: result.movement.destination_id,
      movement_destination_name: result.movement.destination_name,
      confidence,
  }))
  ```

### 6. Patch application (`src/application/game_service.rs:160-167`)

- Destructure new fields:

  ```rust
  let StatePatch::Scene {
      npc_ids,
      movement_destination_id,
      movement_destination_name,
      confidence,
  } = first_patch;
  result.npcs.npc_ids = npc_ids;
  result.movement.destination_id = movement_destination_id;
  result.movement.destination_name = movement_destination_name;
  result.npcs.confidence = confidence.into();
  ```

### 7. Engine consumption (`src/engine/logic.rs` + `src/engine/action_processing.rs`)

- `attempt_semantic_walk` signature change:

  ```rust
  pub fn attempt_semantic_walk(
      state: &mut GameState,
      destination_id: Option<&str>,
      destination_name: Option<&str>,
  ) -> Result<String>
  ```

- Logic:
  - If `destination_id` is `Some(id)` and `find_room_in_world_map(state, id)` returns `Some(room)` → on-map: `state.movement.current_room_id = id.to_string()` (still `String` type in this subplan — Subplan C changes to Option). `log_movement_completion` writes `pending_location = Some(room.name.clone())` (authored name wins, LLM's `destination_name` ignored on-map).
  - If `destination_id` is `None` or doesn't resolve → drift path: existing `create_dynamic_room(destination_name.unwrap_or(destination_id.unwrap_or("unknown")), "A place you have never seen before.")` (still using dynamic_rooms; deletion is Subplan C). `pending_location = Some(destination_name.unwrap_or(destination_id.unwrap_or("unknown")))`.
  - **Note:** drift path keeps the latent return-travel bug (timestamp keys) for this subplan. Subplan C deletes the entire `dynamic_rooms` subsystem rather than patching it. Acceptable — this subplan's scope is the field split, not the deletion.
- `action_processing.rs:37,46-48` (create_dynamic_room call site): pass both fields:

  ```rust
  let destination_id = result.movement.destination_id.as_deref();
  let destination_name = result.movement.destination_name.as_deref();
  // ... attempt_semantic_walk(state, destination_id, destination_name) ...
  ```

### 8. Orchestration logging (`src/narrative/agents/quantifier/orchestration.rs:75-77`)

- Update `tracing::debug!`:

  ```rust
  tracing::debug!(
      "[Quantifier] Detected movement: {:?} destination_id: {:?} destination_name: {:?}",
      result.movement.movement_type,
      result.movement.destination_id,
      result.movement.destination_name
  );
  ```

## Documentation

Updated BEFORE code per chronicler-dev-workflow:

1. `docs/plans/subplan-b-quantifier-field-split.md` — this file.
2. `docs/system/agent_system.md` — quantifier emits `destination_id` + `destination_name`. Update examples.
3. `docs/reference/quantifier_prompt.md` — new output format documented.
4. `docs/system/navigation.md` — `attempt_semantic_walk` new signature (matches by id only; name used for drift path).
5. `docs/CHANGELOG.md` — under "Unreleased":
   - Split quantifier `movement.destination` into `destination_id` + `destination_name`.
   - Updated `data/prompt_presets/quantifier/default.json` to emit new format.
   - `StatePatch::Scene` field split for independent id/name merging.
   - BREAKING for LLM log format: old `{"destination": "..."}` JSON no longer parsed; `build.py --cleanup` recommended to wipe stale `llm_messages` rows (not required — old rows are raw text, not parsed at runtime).

No ADR for this subplan — field shape convention, not architectural decision. The mapless-worlds ADR-027 lands in Subplan C.

## Implementation Order

1. Plan doc (this file).
2. System + reference docs.
3. Delete `src/model/agent.rs_temp` first (prevents stale file from drifting further during edits).
4. Model layer: `model/quantifier.rs` (`MovementParseResult` split), `model/agent.rs` (`StatePatch::Scene` split + `merge` rewrite).
5. Model layer tests: `StatePatch::merge` two-field independent first-non-None; `MovementParseResult` default.
6. Parser: `parser.rs` `MovementJson` field split + `extract_destination_pair` + default construction.
7. Parser tests rewritten: `parser_tests.rs:130,151,192,211,253,414,508` — update JSON literals + field assertions.
8. Quantifier agent: `agent.rs:120-123` patch emission.
9. `game_service.rs:160-167` patch application.
10. Engine: `logic.rs` `attempt_semantic_walk` signature + drift path. `action_processing.rs:37,46-48` call site.
11. Orchestration: `orchestration.rs:75-77` log update.
12. Preset data file: `data/prompt_presets/quantifier/default.json` rewrite.
13. Orchestration tests: `orchestration_tests.rs:115,259,451,459,480,488` — update JSON literals + assertions.
14. Validate: `python build.py --cleanup`.
15. Archive: `docs/plans/archived/subplan-b-quantifier-field-split.md`.

## Failure Modes

1. **LLM emits old-format `{"destination": "kitchen"}`.** No legacy fallback — parser produces `destination_id=None, destination_name=None`. Treated as no-movement. Acceptable per `build.py --cleanup` wiping old LLM behavior. If LLM still emits old format post-preset-update: quantifier effectively stops detecting movement. Mitigation: preset JSON clearly instructs new format with examples; integration test verifies new format parses.
2. **LLM emits `destination_id` that doesn't match any map room but `destination_name` is Title Case.** Drift path: creates dynamic_room keyed by timestamp, name=`destination_name`. Return-travel still broken (Subplan C deletes this path entirely). Acceptable for this subplan.
3. **LLM emits `destination_id` matching a room but `destination_name` is wrong.** On-map path uses `room.name` (authored) and ignores `destination_name`. Correct — map authoritative.
4. **LLM emits `destination_id=None, destination_name=Some("Kitchen")`.** Drift path: dynamic_room named "Kitchen" with timestamp id. Same as today's behavior plus a name field the engine doesn't yet usefully consume (Subplan C's `current_room_name` will use it).
5. **`StatePatch::merge` with two patches both setting `destination_id`.** First non-None wins, second one dropped silently with warn-log. Existing pattern preserved for both fields.
6. **Parser test JSON literals using old `destination` field fail to deserialize.** Expected — tests are rewritten in step 7. Any missed test fails loudly during `cargo nextest`.

## NOT in scope

- `has_map` boolean — Subplan C.
- `current_room_id: Option<String>` — Subplan C.
- `current_room_name: Option<String>` — Subplan C.
- Delete `dynamic_rooms` / `create_dynamic_room` — Subplan C.
- Delete `pending_location` — Subplan C.
- `resolve_starting_location` helper — Subplan C.
- Strict validator matrix (id/name vs has_map) — Subplan C.
- Synthetic `Room` for quantifier when off-map — Subplan C.
- ADR-027 — Subplan C.
- Subplan A's `starting_room_id` relocation — independent, ships separately.

## Verification

```bash
python build.py --cleanup
```

Must pass: fmt + clippy `-D warnings` + nextest. Integration test must verify a quantifier response with new format parses to `(Some(id), Some(name))`. Integration test must verify old-format response is treated as no-movement (not a crash).
