# Blueprint: Scene Quantification V2 (Dual-LLM with Dynamic Room Presence)

> [!NOTE]
> This plan updates the old `scene_quantification_old.md` architecture for the current codebase state.

## TL;DR

Implement a dual-LLM pipeline: fast "Quantifier" model determines room occupants via LLM inference; main "Storyteller" model handles narrative generation.

**Deliverables**: `QuantifierBackend` trait, reduced-context prompt builder, JSON+fallback response parser, `QUANTIFIER_MODEL` env var, integration in `narrative_flow`.

**Critical Path**: Quantifier backend → Prompt builder → Response parser → Integration



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

## Execution Strategy

1. **Add quantifier model configuration** — `QUANTIFIER_MODEL` env var, default to free model
2. **Create QuantifierPromptBuilder** — Reduced context (3-4 history entries)
3. **Implement response parser** — JSON + text fallback, validation
4. **Create QuantifierBackend** — `quantify_room()` method
5. **Integrate into game flow** — Call on room entry
6. **Final integration test** — Mock full flow

---

## TODOs

- [x] 1. Add quantifier model to LLM module
  - Add `QUANTIFIER_MODEL` env var (defaults to free model)
  - Refs: `src/narrative/openrouter_client.rs:18`

- [x] 2. Create QuantifierPromptBuilder
  - Reduced context: last 3-4 history entries, previous room NPCs
  - Compact format for fast inference
  - Refs: `prompt.rs:109`, `prompt.rs:408-431`

- [x] 3. Implement response parser
  - JSON primary, text fallback, validation against `GameState.npcs`
  - Refs: `state.rs:85`

- [x] 4. Create QuantifierBackend
  - `quantify_room()` method using QuantifierPromptBuilder
  - Refs: `llm.rs:89-103`

- [x] 5. Integrate into game flow
  - Call on room entry, pass `npcs_in_area` to PromptContext
  - Refs: `llm.rs:105-135`, `state.rs:86`

- [x] 6. Integration test
  - Mock full flow, verify fallback
  - Refs: `llm.rs:222-253`

---

## Final Verification

- [x] F1. Plan Compliance — all "Must Have" implemented
- [x] F2. Code Quality — cargo fmt, clippy, test
- [x] F3. Integration Test — mock full flow
- [x] F4. Scope Fidelity — no creep

## Commit

- `feat(quantifier): add dual-LLM scene quantification`
- `test(quantifier): add quantifier tests`

## Success Criteria
- [x] Separate model via `QUANTIFIER_MODEL`
- [x] Reduced prompt (<1000 tokens)
- [x] JSON + text fallback parser
- [x] NPC validation
- [x] Fallback to map.json
- [x] All tests pass