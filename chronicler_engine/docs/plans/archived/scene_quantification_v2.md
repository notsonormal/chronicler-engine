# Blueprint: Scene Quantification V2 (Dual-LLM with Dynamic Room Presence)

> [!NOTE]
> This plan updates the old `scene_quantification_old.md` architecture for the current codebase state.

## TL;DR

> **Quick Summary**: Implement a dual-LLM pipeline where a fast "Quantifier" model dynamically determines room occupants via LLM inference, while the main "Storyteller" model handles narrative generation.

> **Deliverables**:
> - New `QuantifierBackend` trait implementation for scene quantification
> - Quantifier-specific prompt builder with reduced context (last 3-4 history entries, previous room NPCs)
> - Robust response parser with JSON + fallback text extraction
> - Environment-configurable secondary model via `QUANTIFIER_MODEL` env var
> - Integration point in `narrative_flow` to call quantifier before narration

> **Estimated Effort**: Medium
> **Parallel Execution**: NO - sequential (depends on existing LLM infrastructure)
> **Critical Path**: Quantifier backend → Prompt builder → Response parser → Integration

---

## Context

### Original Request (from old plan)
- Enable complex physical world interactions (Pushing, Grabbing, Brawling)
- Separate natural language understanding from deterministic state changes

### Updated Requirements (User Input)

1. **Add Second LLM Model for Quantifier**
   - Use free model (like existing `z-ai/glm-4.5-air:free` or `llama3.1:8b` via Ollama)
   - Separate model from main narration engine
   - Configure via `QUANTIFIER_MODEL` environment variable

2. **Dynamic Room Presence Detection**
   - Instead of relying entirely on room state from `map.json`
   - Use LLM to determine who is in the room dynamically
   - Example: If 'Carla' follows player from front gate, both Carla and Gabriella should be in entrance hall (not just Gabriella from map.json)

3. **Quantifier-Specific Prompt**
   - Only include subset of information:
     - Character info of whoever was in the previous room
     - Characters mentioned in recent LLM narration
     - Last 3-4 conversation history records (instead of everything)

### Current Codebase Understanding

**LLM Integration** (`src/narrative/`):
- `llm.rs`: `LlmBackend` trait with `OpenRouterBackend`, `DeepSeekBackend`, `MockBackend`
- `openrouter_client.rs`: Uses `LLM_MODEL` env var, defaults to `z-ai/glm-4.5-air:free`
- `prompt.rs`: `PromptBuilder` with layer-based structure (System, GameState, NpcCards, Player, WorldInfo, History, User, Phi)

**Room/NPC Tracking** (`src/model/`):
- `state.rs`: `GameState` tracks `npcs: HashMap<String, NpcCard>` and `current_room_id`
- `map.json`: Static NPC lists per room (e.g., `"npcs": ["carla"]`)
- Currently NO dynamic tracking of NPCs following or accompanying player

**Conversation History**:
- Stored in `GameState.narration_history: Vec<LogEntry>`
- `LogEntry` contains: sender, text, log_type, timestamp

---

## Work Objectives

### Core Objective
Implement a Quantifier LLM that dynamically determines which NPCs are present in the current room based on:
- Who was in the previous room
- Who the LLM mentioned following or accompanying the player
- Last few conversation entries

### Concrete Deliverables
1. `src/narrative/quantifier.rs` - New quantifier module with:
   - `QuantifierBackend` trait (similar to `LlmBackend`)
   - `quantify_room()` function that calls the LLM
   - Response parser with JSON + text fallback
2. Updated `src/narrative/llm.rs` - Add quantifier model selection
3. Updated `src/narrative/prompt.rs` - Add `QuantifierPromptBuilder` with reduced context
4. Integration in game flow - Call quantifier before narration to determine `npcs_in_area`

### Definition of Done
- [x] Quantifier uses separate model from narration (configurable via env var)
- [x] Quantifier prompt includes only: previous room NPCs + recent history (3-4 entries) + player input
- [x] Response parser handles both JSON and natural language output
- [x] Extracted NPC IDs validated against known NPCs in `GameState`
- [x] Fallback to map.json static data when LLM fails or returns ambiguous results
- [x] Integration point passes dynamic `npcs_in_area` to narration

### Must Have
- Robust parsing that doesn't break when LLM returns non-JSON
- Clear separation between quantifier and storyteller models
- Minimal context for quantifier (fast/cheap inference)

### Must NOT Have (Guardrails)
- Single point of failure - must have fallback when LLM fails
- Hallucinated NPCs - must validate against known NPC IDs
- Excessive context - quantifier must use reduced prompt (not full prompt builder)

---

## Verification Strategy

### Test Decision
- **Infrastructure exists**: YES
- **Automated tests**: Tests-after
- **Framework**: Native Rust `#[cfg(test)]` + integration tests

### QA Policy
Every task includes agent-executed QA scenarios:
- **Mock LLM tests**: Verify quantifier calls use correct model and prompt
- **Response parsing tests**: JSON parsing, text fallback, validation
- **Integration tests**: Full flow with mock quantifier

---

## Execution Strategy

### Sequential Tasks

```
Task 1: Add quantifier model configuration to LLM module
├── Update LlmBackendType to include quantifier
├── Add QUANTIFIER_MODEL env var support
└── Add test for model selection

Task 2: Create QuantifierPromptBuilder
├── Reduced context (3-4 history entries max)
├── Previous room NPCs information
├── Compact NPC info format
└── Add tests for token estimation

Task 3: Implement quantifier response parser
├── JSON parsing with serde
├── Text fallback using NPC name extraction
├── Validation against GameState.npcs
├── Fallback to map.json when uncertain
└── Add comprehensive tests

Task 4: Create QuantifierBackend trait implementation
├── New function: quantify_room()
├── Call LLM with quantifier-specific prompt
├── Parse and validate response
└── Return Vec<NpcCard> of detected NPCs

Task 5: Integrate quantifier into game flow
├── Add quantifier call before narration
├── Pass dynamic npcs_in_area to PromptContext
└── Update tests for integration

Task 6: Final integration test
├── Full flow: player enters room → quantifier runs → narration uses result
├── Verify fallback works when LLM fails
└── Verify mock backend works for testing
```

---

## TODOs

- [x] 1. Add quantifier model configuration to LLM module

  **What to do**:
  - Add `QUANTIFIER_MODEL` environment variable support in `src/narrative/openrouter_client.rs`
  - Default to free model (e.g., `z-ai/glm-4.5-air:free` or `llama3.1:8b`)
  - Add helper function `get_quantifier_model() -> String`

  **Must NOT do**:
  - Break existing LLM_MODEL functionality for narration

  **Recommended Agent Profile**:
  - **Category**: `quick`
  - **Skills**: [`domain-web`] (for HTTP client patterns)

  **Parallelization**:
  - **Can Run In Parallel**: NO
  - **Sequential**: Task 1 starts immediately

  **References**:
  - `src/narrative/openrouter_client.rs:18` - Current model selection pattern

  **Acceptance Criteria**:
  - [x] QUANTIFIER_MODEL env var is read (or defaults to free model)
  - [x] Existing tests pass

- [x] 2. Create QuantifierPromptBuilder with reduced context

  **What to do**:
  - Create new `QuantifierPromptBuilder` struct in `src/narrative/prompt.rs`
  - Include only:
    - Last 3-4 conversation history entries (not full history)
    - Previous room's NPCs with basic info (name, description)
    - NPCs mentioned in recent narration (from history)
    - Player's current input
  - Compact format for fast/cheap inference

  **Must NOT do**:
  - Use full PromptBuilder (too much context)

  **Recommended Agent Profile**:
  - **Category**: `quick`
  - **Skills**: []

  **Parallelization**:
  - **Can Run In Parallel**: NO
  - **Blocked By**: Task 1

  **References**:
  - `src/narrative/prompt.rs:109` - PromptBuilder structure to follow
  - `src/narrative/prompt.rs:408-431` - render_history_layer() for truncation

  **Acceptance Criteria**:
  - [x] QuantifierPromptBuilder produces prompts under 1000 tokens
  - [x] Includes exactly last 3-4 history entries
  - [x] Includes previous room NPC info

- [x] 3. Implement robust response parser

  **What to do**:
  - Create `parse_quantifier_response()` function
  - **Primary**: Try to parse JSON response
    - Expected format: `{"npcs_in_room": ["carla", "gabriella"]}`
  - **Fallback**: Extract NPC names from natural language
    - Use regex to find capitalized names
    - Cross-reference with known NPC IDs from `GameState.npcs`
  - **Confidence scoring**:
    - High confidence: JSON parsed successfully, all IDs valid
    - Medium confidence: Text parsed, some valid IDs found
    - Low confidence: No valid IDs found → use fallback to map.json

  **Must NOT do**:
  - Accept hallucinated NPCs not in game data

  **Recommended Agent Profile**:
  - **Category**: `unspecified-high`
  - **Skills**: []

  **Parallelization**:
  - **Can Run In Parallel**: NO
  - **Blocked By**: Task 2

  **References**:
  - `src/model/state.rs:85` - `npcs: HashMap<String, NpcCard>` for validation

  **Acceptance Criteria**:
  - [x] JSON parsing works for valid JSON responses
  - [x] Text fallback extracts valid NPC IDs
  - [x] Invalid NPC names are filtered out
  - [x] Returns empty vec when LLM completely fails

  **QA Scenarios**:

  ```
  Scenario: JSON response parsing
    Tool: Rust test
    Preconditions: Mock LLM returns valid JSON
    Steps:
      1. Call parse_quantifier_response with JSON input
      2. Assert returned NPC IDs match expected
    Expected Result: ["carla", "gabriella"]
    Evidence: Test passes

  Scenario: Natural language fallback
    Tool: Rust test
    Preconditions: Mock LLM returns "Both Carla and Gabriella are in the room"
    Steps:
      1. Call parse_quantifier_response with text
      2. Assert NPC IDs extracted correctly
    Expected Result: ["carla", "gabriella"]
    Evidence: Test passes

  Scenario: Invalid NPC filtered out
    Tool: Rust test
    Preconditions: LLM returns "Harry is also there" but Harry not in game
    Steps:
      1. Call parse_quantifier_response
      2. Assert Harry is not in result
    Expected Result: ["carla"] (only valid NPCs)
    Evidence: Test passes
  ```

- [x] 4. Create QuantifierBackend implementation

  **What to do**:
  - Add `quantify_room()` method to `LlmBackend` trait (or new trait)
  - Implement in `OpenRouterBackend`
  - Use `QuantifierPromptBuilder` to build prompt
  - Call `parse_quantifier_response()` on result
  - Return `Vec<String>` of NPC IDs

  **Must NOT do**:
  - Duplicate code from existing narration methods

  **Recommended Agent Profile**:
  - **Category**: `unspecified-high`
  - **Skills**: [`domain-web`]

  **Parallelization**:
  - **Can Run In Parallel**: NO
  - **Blocked By**: Task 3

  **References**:
  - `src/narrative/llm.rs:89-103` - narrate_action() pattern to follow

  **Acceptance Criteria**:
  - [x] quantify_room() calls correct model (QUANTIFIER_MODEL)
  - [x] Returns Vec<NpcId> of detected NPCs
  - [x] Uses QuantifierPromptBuilder

- [x] 5. Integrate quantifier into game flow

  **What to do**:
  - Find where room transitions happen in game logic
  - Call quantifier after player enters new room
  - Pass dynamic `npcs_in_area` to `PromptContext`
  - Update narration to use dynamic list

  **Must NOT do**:
  - Break existing room transition logic

  **Recommended Agent Profile**:
  - **Category**: `unspecified-high`
  - **Skills**: []

  **Parallelization**:
  - **Can Run In Parallel**: NO
  - **Blocked By**: Task 4

  **References**:
  - `src/narrative/llm.rs:105-135` - narrate_arrival() shows context flow
  - `src/model/state.rs:86` - current_room_id tracking

  **Acceptance Criteria**:
  - [x] Quantifier called on room entry
  - [x] Dynamic NPCs passed to narration
  - [x] Fallback to map.json when quantifier fails

- [x] 6. Integration test with mock

  **What to do**:
  - Add mock quantifier for testing
  - Test full flow: enter room → quantifier runs → narration uses result
  - Verify fallback behavior

  **Must NOT do**:
  - Require real API key for tests

  **Recommended Agent Profile**:
  - **Category**: `quick`
  - **Skills**: []

  **Parallelization**:
  - **Can Run In Parallel**: NO
  - **Blocked By**: Task 5

  **References**:
  - `src/narrative/llm.rs:222-253` - MockBackend pattern

  **Acceptance Criteria**:
  - [x] All tests pass (cargo test)
  - [x] cargo fmt passes
  - [x] cargo clippy passes

---

## Final Verification Wave

- [x] F1. **Plan Compliance Audit** — `oracle`
  Verify all "Must Have" items implemented, "Must NOT Have" absent

- [x] F2. **Code Quality Review** — `unspecified-high`
  Run cargo fmt, cargo clippy, cargo test

- [x] F3. **Integration Test** — `unspecified-high`
  Full flow test with mock quantifier

- [x] F4. **Scope Fidelity Check** — `deep`
  Verify only intended changes made, no scope creep

---

## Commit Strategy

- **1**: `feat(quantifier): add dual-LLM scene quantification` - core implementation files
- **2**: `test(quantifier): add quantifier tests` - test files

---

## Success Criteria

### Verification Commands
```bash
cargo fmt -- --check
cargo clippy -- -D warnings
cargo test
```

### Final Checklist
- [x] Quantifier uses separate model from narration
- [x] Quantifier prompt is reduced context (< 1000 tokens)
- [x] Response parser handles JSON and text fallback
- [x] NPC IDs validated against known NPCs
- [x] Fallback to map.json when uncertain
- [x] All tests pass