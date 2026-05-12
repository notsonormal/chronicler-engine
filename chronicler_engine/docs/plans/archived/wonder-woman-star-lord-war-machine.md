# Plan: Address Code Review Findings

## Overview

Fix the 6 issues identified in the external review of the chronicler_engine codebase. The `relationships` task is expanded to include wiring into both the normal narration prompt and the quantifier prompt.

## Architecture Decisions

- **Quantifier fallback**: Use `state.scene.npcs_in_area` (previous turn's NPCs) as the fallback when `room_npc_ids` is empty, rather than returning an empty room.
- **Character relationships**: Add the `Relationship` struct and `relationships` field to `NpcCard`, deserialize from JSON, and render into:
  - Layer 2 of the main prompt builder (NPC cards for in-room NPCs)
  - The quantifier prompt context (so the quantifier knows which NPCs have established relationships)
- **Cancel-token helper**: Add a method on `AppState` rather than a free function.

## Task List

### Phase 1: Correctness

- [ ] **Task 1: Fix dead quantifier fallback**
  - **What:** When `determine_npcs_in_room` receives `&[]` for `room_npc_ids`, fallback paths return empty NPC lists. Change `static_npc_result` to fall back to `state.scene.npcs_in_area` when `room_npc_ids` is empty.
  - **Files:** `src/narrative/agents/quantifier/core.rs`, `src/narrative/agents/quantifier/agent.rs`
  - **Verification:** Write a test where quantifier fails/low-confidence and assert that previous room NPCs are preserved, not wiped to empty.
  - **Scope:** Small (2 files)

- [ ] **Task 2: Add relationships to model and both prompts**
  - **What:**
    1. Define `Relationship { with: String, dynamic: String, static_text: String }` in `character.rs`.
    2. Add `#[serde(default)] relationships: Vec<Relationship>` to `NpcCard`.
    3. In the **main prompt builder** (`builder.rs`, Layer 2 / NPC cards): for each in-room NPC, append their relationships with other in-room NPCs to the prompt text.
    4. In the **quantifier prompt** (`quantifier/prompt.rs`): include relationship context so the quantifier understands NPC groupings.
  - **Files:** `src/model/character.rs`, `src/narrative/prompt/builder.rs`, `src/narrative/agents/quantifier/prompt.rs`, `src/narrative/prompt/types.rs` (if context needs extending)
  - **Verification:**
    - Deserialization test: load a Redmist character JSON and assert relationships are populated.
    - Prompt test: build a prompt with two related NPCs and assert the relationship text appears.
  - **Scope:** Medium (3-4 files)

- [ ] **Task 3: Rename misleading test**
  - **What:** `bootstrap_tests.rs:275` `test_validate_loaded_data_missing_npc_reference` has `scenarios: vec![]` and no NPCs. Rename to `test_validate_loaded_data_basic_manifest_succeeds` and update the assertion message.
  - **Files:** `src/bootstrap_tests.rs`
  - **Verification:** Test passes after rename.
  - **Scope:** XS (1 file)

### Checkpoint: After Tasks 1-3
- [ ] `cargo nextest run` passes
- [ ] New quantifier fallback test passes
- [ ] New character + prompt tests pass

### Phase 2: Hygiene

- [ ] **Task 4: Remove dead helper**
  - **What:** `test_support/fixtures.rs:166` `room_with_npc` ignores its `_npc_id` parameter. Rename to `room` or inline callers and delete it.
  - **Files:** `src/test_support/fixtures.rs`, callers in test files
  - **Verification:** `cargo check` passes, no remaining references to old name.
  - **Scope:** Small (2-3 files)

- [ ] **Task 5: Extract cancel-token helper**
  - **What:** Replace the repeated `match state.cancel_token.read() { Ok(g) => g.clone(), Err(p) => p.into_inner().clone() }` pattern (~4 occurrences) with `AppState::current_cancel_token(&self) -> CancellationToken`.
  - **Files:** `src/server/mod.rs`, `src/server/fragments/actions.rs`, `src/server/fragments/misc.rs`
  - **Verification:** `cargo check` passes, all existing tests pass.
  - **Scope:** Small (3 files)

- [ ] **Task 6: Deduplicate scenario NPC initialization**
  - **What:** Extract the identical loop from `bootstrap/run.rs` and `server/fragments/misc.rs` into a shared helper `init_scenario_npcs(state: &mut GameState, scenario: &StartingScenario, npcs: &HashMap<String, NpcCard>)`.
  - **Files:** `src/bootstrap/run.rs`, `src/server/fragments/misc.rs`, new helper location (e.g. `src/model/state.rs` or `src/engine/logic.rs`)
  - **Verification:** Reset tests and startup tests still pass; no behavioural change.
  - **Scope:** Small (3 files)

### Checkpoint: Complete
- [ ] `python build.py` passes (fmt + clippy + tests)
- [ ] No regressions in existing behaviour

## Risks and Mitigations

| Risk | Impact | Mitigation |
|------|--------|------------|
| Quantifier fallback change affects expected test behaviour | Medium | Dedicated fallback test; verify existing quantifier tests. |
| Relationship prompt injection increases token usage | Low | Only include relationships between NPCs *both* present in the room; keep text concise. |
| `NpcCard` field addition changes cached world data shape | Low | `NpcCard` is cached in `AppState` via `Arc`, not snapshotted. Adding a field is safe. |

## Open Questions

- **Relationship prompt format:** Should we render `dynamic` only, or `dynamic + static_text`? Suggest starting with `dynamic` only to keep prompts tight, and including `static_text` only if the user later asks for deeper relationship context.
