# Reactive Auto-Trigger Movement with Continuation Injection

## TL;DR

> **Quick Summary**: Implement Option 1 (Recursive Auto-Trigger) + Option 3 (Continuation Injection) for the Chronicler Engine. When the player moves to a new room, the engine detects NPC triggers based on character state (e.g., `times_met == 0`), fires a second LLM prompt to narrate the event, and combines both narrations into one response delivered via existing HTMX polling.
>
> **Deliverables**:
> - `Trigger` and `NpcEncounterState` data models on `NpcCard`
> - `CharacterState` tracking in `GameState` (in-memory, per-NPC encounter counts)
> - Trigger evaluation engine (pure function: room entry → matching triggers)
> - Recursive auto-trigger flow in `fragments.rs` (second LLM call with continuation prompt)
> - Mock LLM tests for trigger firing, non-firing, multiple triggers, and failure paths
>
> **Estimated Effort**: Medium
> **Parallel Execution**: YES - 4 waves (Wave 0 = docs + test data, Waves 1-3 = implementation)
> **Critical Path**: T0 (docs) → T1 (data models) → T4 (trigger engine) → T7 (auto-trigger flow) → T8 (tests)

---

## Context

### Original Request
Implement Reactive Auto-Trigger movement logic:
```
LLM narration → Quantifier detects movement → attempt_semantic_walk →
  CHECK: does new room have priority events? → YES →
  Second LLM prompt: "Continue the scene" (with room context) →
  return combined narration to player
```

### Interview Summary
**Key Discussions**:
- **Approach**: Option 1 (Recursive Auto-Trigger) + Option 3 (Continuation Injection)
- **Delivery**: Keep HTMX polling — server blocks until both narrations done, returns combined HTML
- **Trigger type**: Character-based triggers on NpcCard, not limited to first-encounter or room-specific
- **Trigger conditions**: Room entry only, based on character state (e.g., `times_met == 0`)
- **NO time-based triggers** — no game clock needed for this scope
- **Trigger firing**: Depends on trigger condition — some one-time, some repeatable
- **Trigger data format**: Full condition + action on NpcCard
- **Tests**: Tests after implementation (not TDD)

**Research Findings**:
- **No streaming/SSE exists** — HTMX with 5-second polling only
- **No trigger system exists** — must be built from scratch
- **No character state tracking** — no `times_met`, no relationship tracking
- **Quantifier uses structured JSON** from LLM for movement detection, runs AFTER full narration
- **Server uses `std::thread::spawn`** (not tokio), `GameState` is `Arc<Mutex<GameState>>`

### Metis Review
**Identified Gaps** (addressed in plan):
- `is_generating` must stay true across both LLM calls (Guardrail GR1)
- Max recursive depth = 1, no cascading triggers (Guardrail GR2)
- No generic event system beyond room entry (Guardrail GR3)
- Character state is in-memory only, no persistence (Guardrail GR4)
- No UI changes beyond existing HTMX polling (Guardrail GR5)
- Edge cases: LLM failure on second call, rapid room changes, trigger narration causing quantifier movement, token budget overflow, empty trigger narration

---

## Work Objectives

### Core Objective
Build a reactive auto-trigger system where NPC priority events fire automatically when the player enters their room, with narrative continuity between the arrival narration and the trigger event.

### Concrete Deliverables
- `docs/architecture/system.md` — Updated with trigger system, character state, auto-trigger flow
- `docs/reference/data_schemas.md` — Updated CharacterSheet and Room schemas
- `docs/system/narration_engine.md` — Updated with continuation narration
- `docs/system/navigation.md` — Updated with auto-trigger movement flow
- `docs/system/game_flow.md` — Updated game loop with trigger evaluation phase
- `src/model/trigger.rs` — Trigger struct with condition + action
- `src/model/character.rs` — Updated NpcCard with triggers field
- `src/model/state.rs` — Updated GameState with CharacterState tracking
- `src/engine/trigger_eval.rs` — Trigger evaluation engine
- `src/narrative/continuation.rs` — Continuation prompt builder
- `src/server/fragments.rs` — Updated process_action with auto-trigger flow
- `tests/trigger_tests.rs` — Mock LLM tests for all trigger scenarios
- `data/worlds/redmist_estate/characters/gabriella.json` — Updated with trigger example
- `data/worlds/test/characters/*.json` — Updated test NPCs with trigger examples

### Definition of Done
- [ ] `cargo test --test trigger_tests` passes (all new tests)
- [ ] `cargo test --test flow_mock_tests` passes (no regression)
- [ ] `python build.py` passes (fmt + clippy + tests + coverage)
- [ ] NPC with `times_met == 0` trigger fires second narration on first room entry
- [ ] Same NPC does NOT re-fire trigger on subsequent room entries

### Must Have
- Character state tracking (`times_met` counter per NPC)
- Trigger evaluation on room entry
- Second LLM call with continuation prompt (includes first narration text)
- `is_generating` stays true through ALL auto-trigger LLM calls
- Max recursive depth = 1 (no cascading triggers)
- Error handling: if second LLM call fails, first narration still displays

### Must NOT Have (Guardrails)
- NO generic event system, pub/sub, or trigger DSL beyond `times_met` conditions
- NO time-based triggers or game clock
- NO JSON persistence for character state (V1 is in-memory only)
- NO recursive trigger chains beyond depth 1
- NO UI changes beyond existing HTMX polling
- NO new endpoints or JavaScript
- NO trigger actions beyond narration (no state mutations, no NPC movement)
- NO conversion to async/tokio — keep `std::thread::spawn` pattern

---

## Verification Strategy

### Test Decision
- **Infrastructure exists**: YES (6 integration test files, `build.py` runs fmt+clippy+tests)
- **Automated tests**: Tests after implementation
- **Framework**: `cargo test` with mock LLM backend
- **Agent-Executed QA**: ALWAYS (mandatory for all tasks)

### QA Policy
Every task MUST include agent-executed QA scenarios. Evidence saved to `.sisyphus/evidence/task-{N}-{scenario-slug}.{ext}`.

---

## Execution Strategy

### Parallel Execution Waves

```
Wave 0 (Start Immediately - docs + test data, MAX PARALLEL):
├── Task 0: Update architecture/system.md [quick]
├── Task 0a: Update data_schemas.md [quick]
├── Task 0b: Update narration_engine.md [quick]
├── Task 0c: Update navigation.md + game_flow.md [quick]
└── Task 0d: Update test character JSON files [quick]

Wave 1 (After Wave 0 - data models + scaffolding):
├── Task 1: Trigger + CharacterState data models [quick]
├── Task 2: Update NpcCard + GameState structs [quick]
└── Task 3: Update production world JSON with trigger examples [quick]

Wave 2 (After Wave 1 - core logic, MAX PARALLEL):
├── Task 4: Trigger evaluation engine [deep]
├── Task 5: Continuation prompt builder [deep]
└── Task 6: Character state management (increment/decrement/query) [quick]

Wave 3 (After Wave 2 - integration):
├── Task 7: Recursive auto-trigger flow in fragments.rs [deep]
└── Task 8: Mock LLM tests for all trigger scenarios [unspecified-high]

Wave FINAL (After ALL tasks — 4 parallel reviews, then user okay):
├── Task F1: Plan compliance audit (oracle)
├── Task F2: Code quality review (unspecified-high)
├── Task F3: Real manual QA (unspecified-high)
└── Task F4: Scope fidelity check (deep)
-> Present results -> Get explicit user okay
```

### Dependency Matrix

- **0-0d**: None — can start immediately
- **1-3**: None — can start immediately (independent of docs)
- **4**: 1, 2 — needs Trigger + CharacterState structs
- **5**: 2 — needs GameState + NpcCard for context building
- **6**: 1, 2 — needs CharacterState + NpcCard
- **7**: 4, 5, 6 — needs trigger eval, continuation builder, state management
- **8**: 7 — needs auto-trigger flow to test
- **F1-F4**: ALL tasks — final verification

### Agent Dispatch Summary

- **Wave 0**: 5 tasks — T0-T0d → `quick`
- **Wave 1**: 3 tasks — T1-T3 → `quick`
- **Wave 2**: 3 tasks — T4 → `deep`, T5 → `deep`, T6 → `quick`
- **Wave 3**: 2 tasks — T7 → `deep`, T8 → `unspecified-high`
- **FINAL**: 4 tasks — F1 → `oracle`, F2 → `unspecified-high`, F3 → `unspecified-high`, F4 → `deep`

---

## TODOs

- [x] 0. Update architecture/system.md

  **What to do**:
  - Update `docs/architecture/system.md` to reflect the new trigger/character state system:
    - Add `Trigger` and `CharacterState` to the Model tier diagram
    - Add `trigger_eval` module to the Engine tier
    - Add `continuation` module to the Narrative tier
    - Update the game flow diagram to show the auto-trigger phase after movement detection
    - Update the file mapping table with new files
  - Follow the existing architecture doc format (module tiers, file mapping, UI specification)

  **Must NOT do**:
  - Do NOT change existing architecture that isn't affected by this feature
  - Do NOT add speculative future features

  **Recommended Agent Profile**:
  - **Category**: `quick`
    - Reason: Documentation update, no code changes
  - **Skills**: []

  **Parallelization**:
  - **Can Run In Parallel**: YES (with Tasks 0a-0d, 1-3)
  - **Parallel Group**: Wave 0 (with Tasks 0a-0d)
  - **Blocks**: None (documentation only)
  - **Blocked By**: None (can start immediately)

  **References**:
  - `docs/architecture/system.md` — Current architecture document to update
  - `src/model/trigger.rs` — New Trigger struct (planned)
  - `src/engine/trigger_eval.rs` — New trigger evaluation engine (planned)
  - `src/narrative/continuation.rs` — New continuation prompt builder (planned)

  **Acceptance Criteria**:
  - [ ] Architecture doc includes Trigger, CharacterState, trigger_eval, continuation modules
  - [ ] Game flow diagram shows auto-trigger phase after movement detection
  - [ ] File mapping table includes all new files

  **QA Scenarios**:

  ```
  Scenario: Architecture doc contains all new modules
    Tool: Bash (grep)
    Preconditions: architecture/system.md updated
    Steps:
      1. Grep for "Trigger" in architecture/system.md
      2. Grep for "CharacterState" in architecture/system.md
      3. Grep for "trigger_eval" in architecture/system.md
      4. Grep for "continuation" in architecture/system.md
    Expected Result: All four terms found in the document
    Evidence: .sisyphus/evidence/task-0-arch-doc.txt
  ```

  **Commit**: YES (groups with 0a-0d)
  - Message: `docs(engine): update architecture for trigger system and character state`
  - Files: `docs/architecture/system.md`
  - Pre-commit: None (markdown only)

- [x] 0a. Update data_schemas.md

  **What to do**:
  - Update `docs/reference/data_schemas.md` to include:
    - `Trigger` schema: condition (TimesMet with operator + value), action (narration_prompt), repeat (boolean)
    - `NpcEncounterState` schema: times_met (u32), trigger_fired (map of trigger index to boolean)
    - `CharacterState` schema: npcs (map of NPC ID to NpcEncounterState)
    - Updated `NpcCard` schema: add `triggers` array field with `#[serde(default)]`
  - Follow existing schema documentation format (JSON examples, field descriptions)

  **Must NOT do**:
  - Do NOT change existing schema definitions (CharacterSheet, Room, etc.) beyond adding the triggers field

  **Recommended Agent Profile**:
  - **Category**: `quick`
    - Reason: Documentation update, schema additions
  - **Skills**: []

  **Parallelization**:
  - **Can Run In Parallel**: YES (with Tasks 0, 0b-0d, 1-3)
  - **Parallel Group**: Wave 0 (with Tasks 0, 0b-0d)
  - **Blocks**: None
  - **Blocked By**: None

  **References**:
  - `docs/reference/data_schemas.md` — Current data schemas to update
  - `src/model/trigger.rs` — Trigger struct definitions (planned)
  - `src/model/character.rs` — NpcCard struct

  **Acceptance Criteria**:
  - [ ] Trigger schema documented with JSON example
  - [ ] NpcEncounterState schema documented
  - [ ] CharacterState schema documented
  - [ ] NpcCard schema updated with triggers field

  **QA Scenarios**:

  ```
  Scenario: Data schemas doc contains all new types
    Tool: Bash (grep)
    Preconditions: data_schemas.md updated
    Steps:
      1. Grep for "Trigger" in data_schemas.md
      2. Grep for "CharacterState" in data_schemas.md
      3. Grep for "NpcEncounterState" in data_schemas.md
    Expected Result: All three terms found with schema definitions
    Evidence: .sisyphus/evidence/task-0a-schemas.txt
  ```

  **Commit**: YES (groups with 0, 0b-0d)
  - Message: `docs(engine): update data schemas for trigger system`
  - Files: `docs/reference/data_schemas.md`
  - Pre-commit: None

- [x] 0b. Update narration_engine.md

  **What to do**:
  - Update `docs/system/narration_engine.md` to include:
    - Continuation narration flow: how the second LLM prompt is built and fired
    - Trigger evaluation: how NPC triggers are checked after movement
    - Updated arrival logic: room entry → movement → trigger evaluation → continuation narration
    - Note about `is_generating` staying true through both LLM calls
  - Follow existing narration engine doc format

  **Must NOT do**:
  - Do NOT change existing narration engine descriptions (GM narration, quantifier, etc.)

  **Recommended Agent Profile**:
  - **Category**: `quick`
    - Reason: Documentation update
  - **Skills**: []

  **Parallelization**:
  - **Can Run In Parallel**: YES (with Tasks 0, 0a, 0c-0d, 1-3)
  - **Parallel Group**: Wave 0 (with Tasks 0, 0a, 0c-0d)
  - **Blocks**: None
  - **Blocked By**: None

  **References**:
  - `docs/system/narration_engine.md` — Current narration engine doc
  - `src/narrative/continuation.rs` — Continuation prompt builder (planned)
  - `src/engine/trigger_eval.rs` — Trigger evaluation (planned)

  **Acceptance Criteria**:
  - [ ] Continuation narration flow documented
  - [ ] Trigger evaluation described in arrival logic
  - [ ] is_generating behavior noted

  **QA Scenarios**:

  ```
  Scenario: Narration engine doc contains continuation flow
    Tool: Bash (grep)
    Preconditions: narration_engine.md updated
    Steps:
      1. Grep for "continuation" in narration_engine.md
      2. Grep for "trigger" in narration_engine.md
    Expected Result: Both terms found with descriptions
    Evidence: .sisyphus/evidence/task-0b-narration.txt
  ```

  **Commit**: YES (groups with 0, 0a, 0c-0d)
  - Message: `docs(engine): update narration engine for continuation triggers`
  - Files: `docs/system/narration_engine.md`
  - Pre-commit: None

- [x] 0c. Update navigation.md + game_flow.md

  **What to do**:
  - Update `docs/system/navigation.md`:
    - Add auto-trigger phase after movement detection
    - Note that trigger narrations do NOT cause further movement (quantifier skipped for them)
  - Update `docs/system/game_flow.md`:
    - Add Phase 3.5: Trigger Evaluation (between movement and narration completion)
    - Update the 5-phase game loop to show: Phase 3 (Process Action) → Movement → Trigger Eval → Phase 4 (LLM Generation, now potentially 2 calls)
    - Update test scenarios to include trigger firing

  **Must NOT do**:
  - Do NOT change existing navigation or game flow descriptions beyond adding the trigger phase

  **Recommended Agent Profile**:
  - **Category**: `quick`
    - Reason: Documentation update, two files
  - **Skills**: []

  **Parallelization**:
  - **Can Run In Parallel**: YES (with Tasks 0, 0a, 0b, 0d, 1-3)
  - **Parallel Group**: Wave 0 (with Tasks 0, 0a, 0b, 0d)
  - **Blocks**: None
  - **Blocked By**: None

  **References**:
  - `docs/system/navigation.md` — Current navigation doc
  - `docs/system/game_flow.md` — Current game flow doc
  - `src/server/fragments.rs` — process_action flow (for accuracy)

  **Acceptance Criteria**:
  - [ ] navigation.md mentions auto-trigger phase after movement
  - [ ] navigation.md notes quantifier skipped for trigger narrations
  - [ ] game_flow.md includes Phase 3.5: Trigger Evaluation
  - [ ] game_flow.md shows potentially 2 LLM calls in Phase 4

  **QA Scenarios**:

  ```
  Scenario: Navigation doc mentions auto-trigger
    Tool: Bash (grep)
    Preconditions: navigation.md updated
    Steps:
      1. Grep for "trigger" in navigation.md
      2. Grep for "auto-trigger" or "continuation" in navigation.md
    Expected Result: Terms found
    Evidence: .sisyphus/evidence/task-0c-navigation.txt

  Scenario: Game flow doc includes trigger phase
    Tool: Bash (grep)
    Preconditions: game_flow.md updated
    Steps:
      1. Grep for "Trigger" or "trigger_eval" in game_flow.md
      2. Verify Phase 3.5 or equivalent is present
    Expected Result: Trigger phase documented
    Evidence: .sisyphus/evidence/task-0c-gameflow.txt
  ```

  **Commit**: YES (groups with 0, 0a, 0b, 0d)
  - Message: `docs(engine): update navigation and game flow for auto-trigger`
  - Files: `docs/system/navigation.md`, `docs/system/game_flow.md`
  - Pre-commit: None

- [x] 0d. Update Test Character JSON Files

  **What to do**:
  - Update test world character files in `data/worlds/test/characters/`:
    - `shopkeeper.json` — Add a trigger with `times_met == 0` condition (first encounter greeting)
    - `ranger.json` — Add a trigger with `times_met < 3` condition (repeatable for first 3 encounters)
    - `bartender.json` — Add NO triggers (control case — NPC without triggers)
  - Ensure all files are valid JSON and match the NpcCard schema with `#[serde(default)]` for triggers
  - Example format:
    ```json
    {
      "id": "shopkeeper",
      "name": "...",
      "description": "...",
      "triggers": [
        {
          "condition": { "TimesMet": ["Eq", 0] },
          "action": {
            "narration_prompt": "The shopkeeper looks up from behind the counter with a warm smile."
          },
          "repeat": false
        }
      ],
      "inventory": []
    }
    ```

  **Must NOT do**:
  - Do NOT modify existing character fields (name, description, personality, etc.)
  - Do NOT add triggers to ALL test NPCs (bartender should have none for control testing)

  **Recommended Agent Profile**:
  - **Category**: `quick`
    - Reason: JSON file edits
  - **Skills**: []

  **Parallelization**:
  - **Can Run In Parallel**: YES (with Tasks 0, 0a-0c, 1-3)
  - **Parallel Group**: Wave 0 (with Tasks 0, 0a-0c)
  - **Blocks**: Task 8 (tests reference these NPCs)
  - **Blocked By**: None (can start immediately — trigger format is defined in the plan)

  **References**:
  - `data/worlds/test/characters/shopkeeper.json` — Existing test NPC
  - `data/worlds/test/characters/ranger.json` — Existing test NPC
  - `data/worlds/test/characters/bartender.json` — Existing test NPC
  - `src/model/trigger.rs` — Trigger struct for JSON field names

  **Acceptance Criteria**:
  - [ ] shopkeeper.json has one trigger (TimesMet Eq 0, non-repeatable)
  - [ ] ranger.json has one trigger (TimesMet Lt 3, repeatable)
  - [ ] bartender.json has NO triggers (empty or missing field)
  - [ ] All files are valid JSON

  **QA Scenarios**:

  ```
  Scenario: Validate test JSON files
    Tool: Bash (cargo test or python json validation)
    Preconditions: All test character JSON files updated
    Steps:
      1. Run: cd chronicler_engine && cargo check (ensures serde can parse)
      2. Or: python -c "import json; [json.load(open(f'data/worlds/test/characters/{f}')) for f in ['shopkeeper.json', 'ranger.json', 'bartender.json']]"
    Expected Result: All files parse as valid JSON
    Evidence: .sisyphus/evidence/task-0d-json-valid.txt
  ```

  **Commit**: YES (groups with 0, 0a-0c)
  - Message: `docs(data): add trigger examples to test character JSON files`
  - Files: `data/worlds/test/characters/shopkeeper.json`, `data/worlds/test/characters/ranger.json`, `data/worlds/test/characters/bartender.json`
  - Pre-commit: None (JSON only)

---

- [x] 1. Trigger + CharacterState Data Models

  **What to do**:
  - Create `src/model/trigger.rs` with:
    - `TriggerCondition` enum: `TimesMet(ComparisonOperator, u32)` where `ComparisonOperator` is `Eq`, `Lt`, `Gte`
    - `TriggerAction` struct: `narration_prompt: String` (text injected into second LLM prompt)
    - `Trigger` struct: `condition: TriggerCondition`, `action: TriggerAction`, `repeat: bool` (if false, fires once then deactivates)
    - `NpcEncounterState` struct: `times_met: u32`, `trigger_fired: HashMap<String, bool>` (tracks which trigger IDs have fired, keyed by trigger description or index)
    - `CharacterState` struct: `npcs: HashMap<String, NpcEncounterState>` (keyed by NPC ID)
  - Derive `Debug, Clone, Serialize, Deserialize, PartialEq` on all structs
  - Follow `std → external → local` import order per Rust conventions

  **Must NOT do**:
  - Do NOT add time-based conditions, inventory checks, or any condition type beyond `TimesMet`
  - Do NOT add JSON persistence — in-memory only
  - Do NOT create a generic event system or pub/sub

  **Recommended Agent Profile**:
  - **Category**: `quick`
    - Reason: Simple struct definitions with serde derives, no complex logic
  - **Skills**: [`coding-guidelines`]
    - `coding-guidelines`: Follow naming conventions, import order, struct design patterns

  **Parallelization**:
  - **Can Run In Parallel**: YES (with Tasks 2, 3)
  - **Parallel Group**: Wave 1 (with Tasks 2, 3)
  - **Blocks**: Tasks 4, 6
  - **Blocked By**: None (can start immediately)

  **References**:
  - `src/model/character.rs:33-40` — NpcCard struct design pattern (derive macros, serde usage)
  - `src/model/state.rs:79-90` — GameState struct design pattern (HashMap fields, defaults)
  - `src/model/map.rs:23-39` — Room struct as reference for serde field patterns
  - `.agents/rules/rust_conventions.md` — Import order, struct design, naming conventions

  **Acceptance Criteria**:
  - [ ] `src/model/trigger.rs` compiles with `cargo check`
  - [ ] All structs derive `Debug, Clone, Serialize, Deserialize, PartialEq`
  - [ ] `TriggerCondition` supports `Eq`, `Lt`, `Gte` operators
  - [ ] `CharacterState` has `HashMap<String, NpcEncounterState>` field

  **QA Scenarios**:

  ```
  Scenario: Compile check for trigger module
    Tool: Bash (cargo check)
    Preconditions: src/model/trigger.rs created with all structs
    Steps:
      1. Run: cd chronicler_engine && cargo check
    Expected Result: Exit code 0, no compilation errors
    Evidence: .sisyphus/evidence/task-1-compile-check.txt

  Scenario: Struct serialization round-trip
    Tool: Bash (cargo test)
    Preconditions: Add a quick test in trigger.rs #[cfg(test)] module
    Steps:
      1. Create Trigger with TimesMet(Eq, 0), serialize to JSON
      2. Deserialize back, assert PartialEq equality
    Expected Result: Round-trip succeeds, structs are equal
    Evidence: .sisyphus/evidence/task-1-serialize-test.txt
  ```

  **Commit**: YES (groups with 2, 3)
  - Message: `feat(engine): add Trigger and CharacterState data models`
  - Files: `src/model/trigger.rs`, `src/model/mod.rs`
  - Pre-commit: `cargo check`

- [x] 2. Update NpcCard + GameState Structs

  **What to do**:
  - Update `NpcCard` in `src/model/character.rs`:
    - Add `triggers: Vec<Trigger>` field with `#[serde(default)]` for backward compatibility with existing JSON
  - Update `GameState` in `src/model/state.rs`:
    - Add `character_state: CharacterState` field with `#[serde(default)]`
  - Update `src/model/mod.rs` to export the new `trigger` module
  - Ensure existing code compiles — no breaking changes to existing fields

  **Must NOT do**:
  - Do NOT remove or rename existing NpcCard fields
  - Do NOT change GameState field types or remove existing fields
  - Do NOT add persistence logic

  **Recommended Agent Profile**:
  - **Category**: `quick`
    - Reason: Adding fields to existing structs with serde defaults
  - **Skills**: [`coding-guidelines`]
    - `coding-guidelines`: Follow struct design conventions (pub fields for DTOs, derives)

  **Parallelization**:
  - **Can Run In Parallel**: YES (with Tasks 1, 3)
  - **Parallel Group**: Wave 1 (with Tasks 1, 3)
  - **Blocks**: Tasks 4, 5, 6
  - **Blocked By**: Task 1 (needs Trigger, CharacterState structs)

  **References**:
  - `src/model/character.rs:33-40` — NpcCard struct to modify
  - `src/model/state.rs:79-90` — GameState struct to modify
  - `src/model/mod.rs` — Module exports to update
  - `data/worlds/redmist_estate/characters/gabriella.json` — Existing NPC JSON (must still deserialize after change)

  **Acceptance Criteria**:
  - [ ] `cargo check` passes with updated structs
  - [ ] Existing `gabriella.json` deserializes correctly (triggers field defaults to empty vec)
  - [ ] `cargo test --test flow_mock_tests` passes (no regression)

  **QA Scenarios**:

  ```
  Scenario: Existing NPC JSON still deserializes
    Tool: Bash (cargo test)
    Preconditions: NpcCard updated with triggers field + #[serde(default)]
    Steps:
      1. Add test: deserialize gabriella.json into NpcCard
      2. Assert triggers field is empty vec
      3. Assert all existing fields (id, sheet, inventory) are correct
    Expected Result: Deserialization succeeds, triggers defaults to []
    Evidence: .sisyphus/evidence/task-2-deserialize-existing.txt

  Scenario: New NPC JSON with triggers deserializes
    Tool: Bash (cargo test)
    Preconditions: Create test JSON with triggers array
    Steps:
      1. Create test JSON string with triggers field populated
      2. Deserialize into NpcCard
      3. Assert triggers vec has correct length and values
    Expected Result: Deserialization succeeds, triggers populated correctly
    Evidence: .sisyphus/evidence/task-2-deserialize-new.txt
  ```

  **Commit**: YES (groups with 1, 3)
  - Message: `feat(engine): add triggers to NpcCard and CharacterState to GameState`
  - Files: `src/model/character.rs`, `src/model/state.rs`, `src/model/mod.rs`
  - Pre-commit: `cargo check && cargo test --test flow_mock_tests`

- [x] 3. Update Test World JSON with Trigger Examples

  **What to do**:
  - Update `data/worlds/redmist_estate/characters/gabriella.json` to include a trigger:
    ```json
    {
      "id": "gabriella",
      "triggers": [
        {
          "condition": { "TimesMet": ["Eq", 0] },
          "action": {
            "narration_prompt": "Gabriella steps forward from the shadows, her eyes locking onto yours."
          },
          "repeat": false
        }
      ]
    }
    ```
  - Create a second test NPC (e.g., `carla.json`) with a different trigger condition for testing
  - Verify both files parse correctly with `cargo check`

  **Must NOT do**:
  - Do NOT modify map.json or room definitions
  - Do NOT modify existing NPC fields beyond adding triggers

  **Recommended Agent Profile**:
  - **Category**: `quick`
    - Reason: JSON file edits, no code changes
  - **Skills**: []

  **Parallelization**:
  - **Can Run In Parallel**: YES (with Tasks 1, 2)
  - **Parallel Group**: Wave 1 (with Tasks 1, 2)
  - **Blocks**: Task 8 (tests reference these NPCs)
  - **Blocked By**: Task 1 (needs Trigger struct to know JSON format)

  **References**:
  - `data/worlds/redmist_estate/characters/gabriella.json` — Existing NPC JSON to modify
  - `src/model/trigger.rs` — Trigger struct for JSON field names
  - `src/model/character.rs` — NpcCard for serde field naming

  **Acceptance Criteria**:
  - [ ] `gabriella.json` has triggers array with one trigger
  - [ ] `carla.json` exists with a different trigger condition
  - [ ] Both files deserialize into NpcCard without errors

  **QA Scenarios**:

  ```
  Scenario: Deserialize updated gabriella.json
    Tool: Bash (cargo test)
    Preconditions: gabriella.json updated with triggers
    Steps:
      1. Write test that reads gabriella.json from filesystem
      2. Deserialize into NpcCard
      3. Assert triggers.len() == 1, condition is TimesMet(Eq, 0)
    Expected Result: Deserialization succeeds, trigger parsed correctly
    Evidence: .sisyphus/evidence/task-3-gabriella-deserialize.txt

  Scenario: Deserialize carla.json
    Tool: Bash (cargo test)
    Preconditions: carla.json created with trigger
    Steps:
      1. Write test that reads carla.json from filesystem
      2. Deserialize into NpcCard
      3. Assert triggers populated correctly
    Expected Result: Deserialization succeeds
    Evidence: .sisyphus/evidence/task-3-carla-deserialize.txt
  ```

  **Commit**: YES (groups with 1, 2)
  - Message: `feat(data): add trigger examples to test NPC JSON files`
  - Files: `data/worlds/redmist_estate/characters/gabriella.json`, `data/worlds/redmist_estate/characters/carla.json`
  - Pre-commit: `cargo check`

- [x] 4. Trigger Evaluation Engine

  **What to do**:
  - Create `src/engine/trigger_eval.rs` with:
    - `evaluate_triggers(state: &GameState, room_id: &str) -> Vec<(NpcCard, Trigger)>` — returns tuples of (NPC, matching trigger)
    - `check_condition(state: &GameState, npc_id: &str, condition: &TriggerCondition) -> bool` — pure function evaluating a single condition
    - `increment_times_met(state: &mut GameState, npc_id: &str)` — increments the counter after a trigger fires
    - `mark_trigger_fired(state: &mut GameState, npc_id: &str, trigger_index: usize)` — marks a non-repeatable trigger as fired
  - Logic:
    1. Get NPCs in the target room from `state.npcs_in_area` (quantifier-confirmed present)
    2. For each NPC, iterate their triggers
    3. For each trigger, evaluate condition against `state.character_state`
    4. If `repeat == false` and trigger already fired, skip
    5. Return list of matching (NPC, trigger) tuples
  - Use `Result` over panic — return `EngineError` for missing NPCs or invalid state

  **Must NOT do**:
  - Do NOT fire LLM calls from this module — it's a pure evaluation function
  - Do NOT modify GameState except for the explicit increment/mark functions
  - Do NOT add any condition type beyond `TimesMet`

  **Recommended Agent Profile**:
  - **Category**: `deep`
    - Reason: Core logic with edge cases (missing NPCs, state initialization, condition evaluation)
  - **Skills**: [`coding-guidelines`, `m06-error-handling`]
    - `coding-guidelines`: Follow import order, naming conventions
    - `m06-error-handling`: Use Result over panic, proper error propagation

  **Parallelization**:
  - **Can Run In Parallel**: YES (with Tasks 5, 6)
  - **Parallel Group**: Wave 2 (with Tasks 5, 6)
  - **Blocks**: Task 7
  - **Blocked By**: Tasks 1, 2

  **References**:
  - `src/model/trigger.rs` — Trigger, TriggerCondition, CharacterState structs
  - `src/model/state.rs` — GameState with character_state field
  - `src/model/character.rs` — NpcCard with triggers field
  - `src/engine/logic.rs` — Existing engine logic patterns (attempt_walk, get_current_room)
  - `src/error.rs` — EngineError enum for error types

  **Acceptance Criteria**:
  - [ ] `evaluate_triggers` returns empty vec when no NPCs have matching triggers
  - [ ] `evaluate_triggers` returns correct triggers when `times_met == 0`
  - [ ] `evaluate_triggers` skips non-repeatable triggers that already fired
  - [ ] `increment_times_met` correctly increments counter (creates entry if missing)
  - [ ] `check_condition` handles missing character state gracefully (defaults to 0)

  **QA Scenarios**:

  ```
  Scenario: No triggers fire for empty room
    Tool: Bash (cargo test)
    Preconditions: GameState with empty npcs_in_area
    Steps:
      1. Call evaluate_triggers with room having no NPCs
    Expected Result: Returns empty vec
    Evidence: .sisyphus/evidence/task-4-empty-room.txt

  Scenario: Trigger fires on first encounter (times_met == 0)
    Tool: Bash (cargo test)
    Preconditions: GameState with NPC "gabriella" in room, trigger condition TimesMet(Eq, 0), character_state empty
    Steps:
      1. Call evaluate_triggers
      2. Assert returns 1 trigger for gabriella
      3. Call increment_times_met for gabriella
      4. Call evaluate_triggers again
      5. Assert returns empty vec (times_met is now 1)
    Expected Result: First call returns trigger, second call returns empty
    Evidence: .sisyphus/evidence/task-4-first-encounter.txt

  Scenario: Non-repeatable trigger does not re-fire after marked
    Tool: Bash (cargo test)
    Preconditions: NPC with repeat: false trigger, times_met == 0
    Steps:
      1. Call evaluate_triggers → returns trigger
      2. Call mark_trigger_fired
      3. Call evaluate_triggers again
      4. Assert returns empty vec
    Expected Result: Trigger does not fire after being marked
    Evidence: .sisyphus/evidence/task-4-non-repeatable.txt

  Scenario: Missing character state defaults to 0
    Tool: Bash (cargo test)
    Preconditions: GameState with NPC but NO entry in character_state for that NPC
    Steps:
      1. Call check_condition with TimesMet(Eq, 0)
    Expected Result: Returns true (defaults to 0, matches Eq 0)
    Evidence: .sisyphus/evidence/task-4-default-state.txt
  ```

  **Commit**: YES
  - Message: `feat(engine): add trigger evaluation engine`
  - Files: `src/engine/trigger_eval.rs`, `src/engine/mod.rs`
  - Pre-commit: `cargo check`

- [x] 5. Continuation Prompt Builder

  **What to do**:
  - Create `src/narrative/continuation.rs` with:
    - `build_continuation_prompt(context: &PromptContext, first_narration: &str, trigger_text: &str) -> Result<(String, String), EngineError>` — returns (system_prompt, user_prompt) for the second LLM call
    - The system prompt should instruct the LLM to:
      - Continue the scene from the first narration
      - Incorporate the trigger text naturally (NPC entrance, dialogue, etc.)
      - NOT repeat or contradict the first narration
      - Keep the response concise (shorter than a normal narration)
    - The user prompt should include:
      - The first narration text (for continuity)
      - The current room context (name, description, NPCs present)
      - The trigger action text
  - Reuse existing `PromptContext` from `prompt.rs` for room/NPC context
  - Use existing `budget::MAX_CONTEXT_TOKENS` — truncate first narration if needed to fit budget

  **Must NOT do**:
  - Do NOT create a new prompt system — extend the existing one
  - Do NOT skip truncation of first narration (token budget must be respected)
  - Do NOT include movement instructions in the continuation prompt

  **Recommended Agent Profile**:
  - **Category**: `deep`
    - Reason: Prompt engineering with token budgeting, integration with existing prompt system
  - **Skills**: [`coding-guidelines`]
    - `coding-guidelines`: Follow import order, naming conventions

  **Parallelization**:
  - **Can Run In Parallel**: YES (with Tasks 4, 6)
  - **Parallel Group**: Wave 2 (with Tasks 4, 6)
  - **Blocks**: Task 7
  - **Blocked By**: Task 2

  **References**:
  - `src/narrative/prompt.rs` — PromptContext struct, prompt building patterns
  - `src/narrative/prompt.rs:budget` — MAX_CONTEXT_TOKENS, token budgeting
  - `src/narrative/llm.rs` — LlmBackend trait, narrate_action/narrate_arrival signatures
  - `src/narrative/quantifier.rs` — QuantifierPromptBuilder as reference for prompt construction

  **Acceptance Criteria**:
  - [ ] `build_continuation_prompt` returns non-empty system and user prompts
  - [ ] First narration is included in the prompt for continuity
  - [ ] Trigger text is included in the prompt
  - [ ] Prompt respects MAX_CONTEXT_TOKENS (truncates first narration if needed)
  - [ ] System prompt instructs LLM to continue, not repeat

  **QA Scenarios**:

  ```
  Scenario: Build continuation prompt with short first narration
    Tool: Bash (cargo test)
    Preconditions: PromptContext with room data, first_narration = "You enter the hall.", trigger_text = "Gabriella steps forward."
    Steps:
      1. Call build_continuation_prompt
      2. Assert system_prompt contains "continue the scene"
      3. Assert user_prompt contains first_narration
      4. Assert user_prompt contains trigger_text
    Expected Result: Both prompts contain expected content
    Evidence: .sisyphus/evidence/task-5-short-narration.txt

  Scenario: First narration truncated when too long
    Tool: Bash (cargo test)
    Preconditions: first_narration = 5000+ characters, MAX_CONTEXT_TOKENS = 4000
    Steps:
      1. Call build_continuation_prompt
      2. Assert user_prompt length is within budget
      3. Assert trigger_text is still included (not truncated)
    Expected Result: First narration truncated, trigger text preserved
    Evidence: .sisyphus/evidence/task-5-truncation.txt
  ```

  **Commit**: YES
  - Message: `feat(narrative): add continuation prompt builder for auto-trigger`
  - Files: `src/narrative/continuation.rs`, `src/narrative/mod.rs`
  - Pre-commit: `cargo check`

- [x] 6. Character State Management

  **What to do**:
  - Implement helper methods on `CharacterState` (in `src/model/trigger.rs` or `src/model/state.rs`):
    - `impl CharacterState`:
      - `fn get_npc_state(&self, npc_id: &str) -> Option<&NpcEncounterState>` — get state for an NPC
      - `fn get_or_create_npc_state(&mut self, npc_id: &str) -> &mut NpcEncounterState` — create if missing
      - `fn increment_times_met(&mut self, npc_id: &str)` — increment counter
      - `fn get_times_met(&self, npc_id: &str) -> u32` — get counter (returns 0 if missing)
      - `fn is_trigger_fired(&self, npc_id: &str, trigger_index: usize) -> bool` — check if fired
      - `fn mark_trigger_fired(&mut self, npc_id: &str, trigger_index: usize)` — mark as fired
  - Implement `Default` for `CharacterState` (empty HashMap)
  - Implement `Default` for `NpcEncounterState` (times_met: 0, empty trigger_fired map)

  **Must NOT do**:
  - Do NOT add persistence/serialization beyond the existing serde derives
  - Do NOT add methods that fire LLM calls or modify game state beyond character state

  **Recommended Agent Profile**:
  - **Category**: `quick`
    - Reason: Simple accessor/mutator methods on existing structs
  - **Skills**: [`coding-guidelines`]
    - `coding-guidelines`: Follow naming conventions, method design patterns

  **Parallelization**:
  - **Can Run In Parallel**: YES (with Tasks 4, 5)
  - **Parallel Group**: Wave 2 (with Tasks 4, 5)
  - **Blocks**: Task 7
  - **Blocked By**: Tasks 1, 2

  **References**:
  - `src/model/trigger.rs` — CharacterState, NpcEncounterState structs
  - `src/model/state.rs` — GameState struct (where CharacterState is embedded)
  - `src/model/character.rs` — NpcCard for reference patterns

  **Acceptance Criteria**:
  - [ ] `get_times_met` returns 0 for missing NPCs
  - [ ] `increment_times_met` creates entry if missing, increments if exists
  - [ ] `is_trigger_fired` returns false for missing entries
  - [ ] `mark_trigger_fired` creates entry if missing, sets flag
  - [ ] `Default` implementations produce empty state

  **QA Scenarios**:

  ```
  Scenario: Get times_met for missing NPC returns 0
    Tool: Bash (cargo test)
    Preconditions: Empty CharacterState
    Steps:
      1. Call get_times_met("unknown_npc")
    Expected Result: Returns 0
    Evidence: .sisyphus/evidence/task-6-missing-npc.txt

  Scenario: Increment creates entry and increments
    Tool: Bash (cargo test)
    Preconditions: Empty CharacterState
    Steps:
      1. Call increment_times_met("gabriella")
      2. Call get_times_met("gabriella")
      3. Assert returns 1
      4. Call increment_times_met("gabriella") again
      5. Call get_times_met("gabriella")
      6. Assert returns 2
    Expected Result: Counter increments correctly
    Evidence: .sisyphus/evidence/task-6-increment.txt

  Scenario: Default CharacterState is empty
    Tool: Bash (cargo test)
    Preconditions: None
    Steps:
      1. Create CharacterState::default()
      2. Assert get_times_met("any") returns 0
    Expected Result: All queries return defaults
    Evidence: .sisyphus/evidence/task-6-default.txt
  ```

  **Commit**: YES (groups with 4, 5)
  - Message: `feat(engine): add character state management methods`
  - Files: `src/model/trigger.rs` or `src/model/state.rs`
  - Pre-commit: `cargo check`

- [x] 7. Recursive Auto-Trigger Flow in fragments.rs

  **What to do**:
  - Modify `process_action` in `src/server/fragments.rs` to implement the recursive auto-trigger flow:
    1. After first LLM narration completes (`narrate_action` returns)
    2. Run quantifier to detect movement (existing logic)
    3. If movement detected and `attempt_semantic_walk` succeeds:
       a. Call `evaluate_triggers(state, new_room_id)` to get matching triggers
       b. If triggers found:
          - For each trigger (up to max 3 to prevent runaway):
            - Build continuation prompt via `build_continuation_prompt`
            - Call LLM with the continuation prompt (reuse `backend.narrate_action` or add `narrate_continuation`)
            - If LLM succeeds: append narration to log, increment `times_met`, mark trigger fired if non-repeatable
            - If LLM fails: log error, continue to next trigger (do NOT abort)
       c. Set `is_generating = false` ONLY after ALL triggers processed
    4. If no movement or no triggers: reset `is_generating = false` as before (existing behavior)
  - Critical: `is_generating` must NOT reset between the first narration and trigger narrations
  - Add error handling: if second LLM call fails, first narration still displays, error logged
  - Prevent infinite loops: max 3 trigger narrations per user action
  - Skip quantifier movement detection for trigger narrations (mark them as non-movement)

  **Must NOT do**:
  - Do NOT convert to async/tokio — keep `std::thread::spawn` pattern
  - Do NOT add new endpoints or change HTMX polling
  - Do NOT allow trigger narrations to cause movement (skip quantifier for them)
  - Do NOT reset `is_generating` until ALL trigger processing is complete

  **Recommended Agent Profile**:
  - **Category**: `deep`
    - Reason: Core server flow modification with threading, mutex, error handling, and state management
  - **Skills**: [`coding-guidelines`, `m06-error-handling`, `m07-concurrency`]
    - `coding-guidelines`: Follow import order, naming conventions
    - `m06-error-handling`: Proper error handling for LLM failures
    - `m07-concurrency`: Thread safety with Arc<Mutex<GameState>>

  **Parallelization**:
  - **Can Run In Parallel**: NO (depends on Tasks 4, 5, 6)
  - **Parallel Group**: Wave 3 (with Task 8, but T8 depends on T7 completion)
  - **Blocks**: Task 8, Final Verification
  - **Blocked By**: Tasks 4, 5, 6

  **References**:
  - `src/server/fragments.rs:602-705` — process_action function, thread spawning, LLM call, quantifier, state update
  - `src/server/fragments.rs:637-672` — Movement detection and attempt_semantic_walk
  - `src/engine/trigger_eval.rs` — evaluate_triggers, increment_times_met, mark_trigger_fired
  - `src/narrative/continuation.rs` — build_continuation_prompt
  - `src/narrative/llm.rs` — LlmBackend trait for LLM calls
  - `src/model/state.rs` — GameState with is_generating field

  **Acceptance Criteria**:
  - [ ] Room entry with matching trigger fires second LLM call
  - [ ] Second narration appended to log after first narration
  - [ ] `is_generating` stays true through both LLM calls
  - [ ] `times_met` incremented after trigger fires
  - [ ] Non-repeatable trigger marked as fired
  - [ ] Room entry without triggers behaves exactly as before (no regression)
  - [ ] LLM failure on second call: first narration displays, error logged, `is_generating` resets

  **QA Scenarios**:

  ```
  Scenario: Auto-trigger fires on room entry (happy path)
    Tool: Bash (cargo test with mock LLM)
    Preconditions: Mock LLM configured to return "You enter the hall." for first call, "Gabriella steps forward." for second call. NPC gabriella has times_met == 0 trigger.
    Steps:
      1. Send FreeAction that causes movement to entrance_hall
      2. Wait for is_generating to become false
      3. Check narration_history has 2 entries
      4. Check gabriella.times_met == 1
    Expected Result: 2 narration entries, times_met incremented
    Evidence: .sisyphus/evidence/task-7-happy-path.txt

  Scenario: No trigger fires for room without triggers
    Tool: Bash (cargo test with mock LLM)
    Preconditions: Mock LLM returns "You enter the kitchen." Room has NPCs with no triggers.
    Steps:
      1. Send FreeAction that causes movement to kitchen
      2. Wait for is_generating to become false
      3. Check narration_history has 1 entry
    Expected Result: 1 narration entry (no second call)
    Evidence: .sisyphus/evidence/task-7-no-trigger.txt

  Scenario: Second LLM call fails gracefully
    Tool: Bash (cargo test with mock LLM)
    Preconditions: Mock LLM succeeds on first call, fails on second call. NPC has trigger.
    Steps:
      1. Send FreeAction that causes movement to room with trigger
      2. Wait for is_generating to become false
      3. Check narration_history has 1 entry (first narration)
      4. Check error_message is set or error logged
    Expected Result: First narration displays, error handled, is_generating resets
    Evidence: .sisyphus/evidence/task-7-llm-failure.txt

  Scenario: is_generating stays true through both calls
    Tool: Bash (cargo test with mock LLM)
    Preconditions: Mock LLM with 500ms delay on both calls.
    Steps:
      1. Send FreeAction that triggers movement
      2. Poll /status/generating every 100ms
      3. Assert "generating" returned until both calls complete
      4. Assert "idle" returned after completion
    Expected Result: Status stays "generating" through both calls, then "idle"
    Evidence: .sisyphus/evidence/task-7-is-generating.txt
  ```

  **Commit**: YES
  - Message: `feat(server): implement recursive auto-trigger flow in process_action`
  - Files: `src/server/fragments.rs`
  - Pre-commit: `cargo check`

- [x] 8. Mock LLM Tests for All Trigger Scenarios

  **What to do**:
  - Create `tests/trigger_tests.rs` with comprehensive mock LLM tests:
    - Test: First encounter trigger fires (times_met == 0 → second narration)
    - Test: Second encounter does NOT re-fire (times_met == 1 → no second narration)
    - Test: Multiple NPCs with triggers fire sequentially
    - Test: Non-repeatable trigger fires once, then never again
    - Test: LLM failure on second call — first narration still displays
    - Test: Empty trigger narration (LLM returns whitespace) — skipped but counter incremented
    - Test: No regression — FreeAction without movement works as before
    - Test: No regression — FreeAction with movement but no triggers works as before
  - Reuse existing mock LLM patterns from `tests/flow_mock_tests.rs`
  - Each test should:
    1. Set up GameState with specific NPC triggers and character state
    2. Configure mock LLM responses
    3. Execute action
    4. Assert narration_history length and content
    5. Assert character state changes

  **Must NOT do**:
  - Do NOT test against real LLM API — use mocks only
  - Do NOT duplicate tests from flow_mock_tests.rs — only add new trigger-specific tests
  - Do NOT modify existing flow_mock_tests.rs — verify they still pass separately

  **Recommended Agent Profile**:
  - **Category**: `unspecified-high`
    - Reason: Comprehensive test suite with multiple scenarios, mock setup, assertions
  - **Skills**: [`coding-guidelines`, `test-police`]
    - `coding-guidelines`: Follow naming conventions, test structure
    - `test-police`: Review test quality, coverage, assertion patterns

  **Parallelization**:
  - **Can Run In Parallel**: NO (depends on Task 7)
  - **Parallel Group**: Wave 3 (runs after T7)
  - **Blocks**: Final Verification
  - **Blocked By**: Task 7

  **References**:
  - `tests/flow_mock_tests.rs` — Existing mock LLM test patterns to follow
  - `src/server/fragments.rs` — process_action function being tested
  - `src/model/trigger.rs` — Trigger, CharacterState structs
  - `src/engine/trigger_eval.rs` — evaluate_triggers function

  **Acceptance Criteria**:
  - [ ] `cargo test --test trigger_tests` passes (all 8+ tests)
  - [ ] `cargo test --test flow_mock_tests` passes (no regression)
  - [ ] Each test has descriptive name: `test_trigger_<scenario>`
  - [ ] Tests cover: happy path, no trigger, multiple triggers, failure, non-repeatable, empty response

  **QA Scenarios**:

  ```
  Scenario: Run all trigger tests
    Tool: Bash (cargo test)
    Preconditions: All tests written in tests/trigger_tests.rs
    Steps:
      1. Run: cd chronicler_engine && cargo test --test trigger_tests
    Expected Result: All tests pass, 0 failures
    Evidence: .sisyphus/evidence/task-8-all-tests.txt

  Scenario: Run regression tests
    Tool: Bash (cargo test)
    Preconditions: Existing flow_mock_tests.rs unchanged
    Steps:
      1. Run: cd chronicler_engine && cargo test --test flow_mock_tests
    Expected Result: All existing tests pass, 0 failures
    Evidence: .sisyphus/evidence/task-8-regression.txt

  Scenario: Full build validation
    Tool: Bash (python build.py)
    Preconditions: All code and tests written
    Steps:
      1. Run: cd chronicler_engine && python build.py
    Expected Result: fmt + clippy + tests + coverage all pass
    Evidence: .sisyphus/evidence/task-8-full-build.txt
  ```

  **Commit**: YES
  - Message: `test(engine): add comprehensive trigger scenario tests`
  - Files: `tests/trigger_tests.rs`
  - Pre-commit: `cargo test --test trigger_tests && cargo test --test flow_mock_tests`

---

## Final Verification Wave (MANDATORY — after ALL implementation tasks)

> 4 review agents run in PARALLEL. ALL must APPROVE. Present consolidated results to user and get explicit "okay" before completing.
>
> **Do NOT auto-proceed after verification. Wait for user's explicit approval before marking work complete.**
> **Never mark F1-F4 as checked before getting user's okay.** Rejection or user feedback -> fix -> re-run -> present again -> wait for okay.

- [x] F1. **Plan Compliance Audit** — `oracle`
  Read the plan end-to-end. For each "Must Have": verify implementation exists (read file, check struct fields, verify function signatures). For each "Must NOT Have": search codebase for forbidden patterns — reject with file:line if found. Check evidence files exist in .sisyphus/evidence/. Compare deliverables against plan.
  Output: `Must Have [N/N] | Must NOT Have [N/N] | Tasks [N/N] | VERDICT: APPROVE/REJECT`

- [x] F2. **Code Quality Review** — `unspecified-high`
  Run `cargo check` + `cargo clippy` + `cargo test`. Review all changed files for: `.unwrap()`/`.expect()` in production code, empty catches, console/debug output, unused imports. Check AI slop: excessive comments, over-abstraction, generic names. Verify Rust 2024 edition conventions.
  Output: `Build [PASS/FAIL] | Clippy [PASS/FAIL] | Tests [N pass/N fail] | Files [N clean/N issues] | VERDICT`

- [x] F3. **Real Manual QA** — `unspecified-high`
  Start from clean state. Execute EVERY QA scenario from EVERY task — follow exact steps, capture evidence. Test cross-task integration (trigger eval + continuation + auto-trigger flow working together). Test edge cases: empty state, invalid input, rapid actions. Save to `.sisyphus/evidence/final-qa/`.
  Output: `Scenarios [N/N pass] | Integration [N/N] | Edge Cases [N tested] | VERDICT`

- [x] F4. **Scope Fidelity Check** — `deep`
  For each task: read "What to do", read actual diff (git log/diff). Verify 1:1 — everything in spec was built (no missing), nothing beyond spec was built (no creep). Check "Must NOT do" compliance. Detect cross-task contamination: Task N touching Task M's files. Flag unaccounted changes.
  Output: `Tasks [N/N compliant] | Contamination [CLEAN/N issues] | Unaccounted [CLEAN/N files] | VERDICT`

---

## Commit Strategy

- **Wave 0 (Tasks 0-0d)**: `docs(engine): update architecture, schemas, and test data for trigger system`
  - Files: `docs/architecture/system.md`, `docs/reference/data_schemas.md`, `docs/system/narration_engine.md`, `docs/system/navigation.md`, `docs/system/game_flow.md`, `data/worlds/test/characters/*.json`
  - Pre-commit: None (docs + JSON only)

- **Wave 1 (Tasks 1-3)**: `feat(engine): add Trigger, CharacterState data models and test JSON`
  - Files: `src/model/trigger.rs`, `src/model/character.rs`, `src/model/state.rs`, `src/model/mod.rs`, `data/worlds/redmist_estate/characters/*.json`
  - Pre-commit: `cargo check`

- **Wave 2 (Tasks 4-6)**: `feat(engine): add trigger evaluation, continuation builder, and state management`
  - Files: `src/engine/trigger_eval.rs`, `src/engine/mod.rs`, `src/narrative/continuation.rs`, `src/narrative/mod.rs`
  - Pre-commit: `cargo check`

- **Wave 3 (Task 7)**: `feat(server): implement recursive auto-trigger flow`
  - Files: `src/server/fragments.rs`
  - Pre-commit: `cargo check`

- **Wave 3 (Task 8)**: `test(engine): add trigger scenario tests`
  - Files: `tests/trigger_tests.rs`
  - Pre-commit: `cargo test --test trigger_tests && cargo test --test flow_mock_tests`

---

## Success Criteria

### Verification Commands
```bash
cd chronicler_engine && cargo check                    # Expected: no errors
cd chronicler_engine && cargo test --test trigger_tests # Expected: all tests pass
cd chronicler_engine && cargo test --test flow_mock_tests # Expected: no regression
cd chronicler_engine && python build.py                  # Expected: fmt + clippy + tests + coverage pass
```

### Final Checklist
- [ ] All "Must Have" present (character state, trigger eval, continuation prompt, auto-trigger flow)
- [ ] All "Must NOT Have" absent (no game clock, no persistence, no generic event system, no async/tokio)
- [ ] All tests pass (trigger_tests + flow_mock_tests)
- [ ] `is_generating` stays true through both LLM calls
- [ ] First narration displays even if second LLM call fails
- [ ] Non-repeatable triggers fire once and never again
- [ ] `times_met` increments correctly on trigger fire
